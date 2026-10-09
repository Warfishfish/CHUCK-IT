//! Online play, step one: the messages two copies of the game send each other, and how they are
//! turned into bytes and back. Pure code, no sockets (the sockets live in `bbq_app`), so every
//! message can be tested here.
//!
//! The picture: one player hosts and runs the rules (bots, items, Dazza, scores); everybody else
//! joins. Each player moves their own blob, so the controls always feel instant, and says where
//! it is about 20 times a second (`PlayerState`); the others show it moving smoothly. Later steps
//! add the host's picture of the world and the actions (throw, pick up, slap) on top.
//!
//! The bytes are little-endian and tiny. A message that is too short, has an unknown kind, or has
//! junk after it is refused (`DecodeError`) and never panics, because anything on the internet
//! can send us anything.

/// Bump this whenever a message changes, so old and new copies of the game refuse each other
/// politely instead of misreading each other.
pub const PROTOCOL: u16 = 4;
/// The most people in one yard (the browser game's limit).
pub const MAX_PLAYERS: usize = 16;
/// How often each player reports where they are.
pub const STATE_HZ: f32 = 20.0;
/// Names are cut to this many characters.
pub const NAME_MAX: usize = 16;

/// Why the host said no.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Different versions of the game.
    WrongVersion,
    /// 16 people already.
    Full,
    /// A round is on and the host does not take people mid-round (yet).
    InProgress,
}

/// One person in the yard, as the host lists them.
#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub id: u32,
    pub name: String,
    /// Which blob they chose (0 = Classic, 1 = Pear, 2 = Egg, 3 = Gumdrop).
    pub character: u8,
    pub host: bool,
    /// Their customised look (face, hair, colours) and beer belly (0 to 255 over 0.0 to 2.0).
    pub look: crate::appearance::Appearance,
    pub belly: u8,
    /// In the lobby: they have pressed Ready.
    pub ready: bool,
}

impl Member {
    /// A member with the default look (tests and the host's first entry).
    pub fn plain(id: u32, name: &str, character: u8, host: bool) -> Self {
        Member { id, name: name.to_string(), character, host, look: Default::default(), belly: 128, ready: false }
    }
}

/// Where a player is and what they are doing, enough to draw them (not to run the rules).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerState {
    pub id: u32,
    pub pos: (f32, f32, f32),
    /// Which way they face, radians (the game's `yaw`).
    pub yaw: f32,
    pub vel: (f32, f32),
    pub walk: f32,
    pub grounded: bool,
    pub stunned: bool,
    pub down: bool,
    pub drinking: bool,
    /// Winding up a throw, 0 to 1 (0 = not).
    pub charge: f32,
    /// What is in their hand: 0 nothing, else the item kind + 1, and its dildo type.
    pub held: u8,
    pub held_variant: u8,
    /// Beer belly, 0 to 255 scaled over 0.0 to 2.0.
    pub belly: u8,
    /// How deep they have dived in the pool, 0 (treading water) to 255 (down by the floor).
    pub dive: u8,
}

impl PlayerState {
    pub fn new(id: u32) -> Self {
        PlayerState {
            id,
            pos: (0.0, 0.0, 0.0),
            yaw: 0.0,
            vel: (0.0, 0.0),
            walk: 0.0,
            grounded: true,
            stunned: false,
            down: false,
            drinking: false,
            charge: 0.0,
            held: 0,
            held_variant: 0,
            belly: 128,
            dive: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    /// Guest to host: "can I play?"
    Hello { protocol: u16, name: String, character: u8, look: crate::appearance::Appearance, belly: u8 },
    /// Host to one guest: "yes, you are `id`", with everyone already here.
    Welcome { id: u32, members: Vec<Member> },
    /// Host to one guest: "no".
    Refused(Refusal),
    /// Host to everyone: who is here now, how they look and who is ready (sent when that
    /// changes). `start_in` is the lobby countdown in tenths of a second (0 = not counting).
    Roster { members: Vec<Member>, start_in: u8 },
    /// Everybody to everybody: where I am (about 20 times a second).
    State(PlayerState),
    /// "I am leaving" (the host also sends it for a guest who vanished).
    Bye { id: u32 },
    /// Host to everybody: how the whole yard looks right now (about 15 times a second).
    World(Box<crate::net_world::WorldSnap>),
    /// Guest to host: "do this for me" (throw, slap, drop, select, catch).
    Act(crate::net_act::GuestAct),
    /// Host to one guest: "this just happened to you" (you were hit).
    Hit(crate::net_act::HitMsg),
    /// Guest to host: "I have changed my look" (in the lobby).
    Look { character: u8, look: crate::appearance::Appearance, belly: u8 },
    /// Guest to host: "I am ready" (or not any more).
    Ready(bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    TooShort,
    UnknownKind(u8),
    BadText,
    TooMany,
    TrailingBytes,
}

// ---------------------------------------------------------------- writing

pub(crate) struct Writer(pub(crate) Vec<u8>);

impl Writer {
    pub(crate) fn i32(&mut self, v: i32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    /// A line of text of up to `max` characters (cut if longer), with a two-byte length.
    pub(crate) fn line(&mut self, s: &str, max: usize) {
        let s: String = s.chars().filter(|c| !c.is_control()).take(max).collect();
        self.u16(s.len() as u16);
        self.0.extend_from_slice(s.as_bytes());
    }
    pub(crate) fn pair(&mut self, p: (f32, f32)) {
        self.f32(p.0);
        self.f32(p.1);
    }
    pub(crate) fn triple(&mut self, p: (f32, f32, f32)) {
        self.f32(p.0);
        self.f32(p.1);
        self.f32(p.2);
    }
    pub(crate) fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    pub(crate) fn u16(&mut self, v: u16) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(crate) fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    pub(crate) fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn text(&mut self, s: &str) {
        let s: String = s.chars().take(NAME_MAX).collect();
        self.u8(s.len() as u8); // at most 16 chars of up to 4 bytes each: always fits
        self.0.extend_from_slice(s.as_bytes());
    }
    fn member(&mut self, m: &Member) {
        self.u32(m.id);
        self.text(&m.name);
        self.u8(m.character);
        self.u8(m.host as u8 | (m.ready as u8) << 1);
        self.0.extend_from_slice(&m.look.to_bytes());
        self.u8(m.belly);
    }
    fn members(&mut self, ms: &[Member]) {
        self.u8(ms.len().min(MAX_PLAYERS) as u8);
        for m in ms.iter().take(MAX_PLAYERS) {
            self.member(m);
        }
    }
}

// ---------------------------------------------------------------- reading

pub(crate) struct Reader<'a>(pub(crate) &'a [u8]);

impl<'a> Reader<'a> {
    pub(crate) fn i32(&mut self) -> Result<i32, DecodeError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub(crate) fn line(&mut self, max: usize) -> Result<String, DecodeError> {
        let n = self.u16()? as usize;
        if n > max * 4 {
            return Err(DecodeError::BadText);
        }
        let s = std::str::from_utf8(self.take(n)?).map_err(|_| DecodeError::BadText)?;
        if s.chars().count() > max || s.chars().any(|c| c.is_control()) {
            return Err(DecodeError::BadText);
        }
        Ok(s.to_string())
    }
    pub(crate) fn pair(&mut self) -> Result<(f32, f32), DecodeError> {
        Ok((self.f32()?, self.f32()?))
    }
    pub(crate) fn triple(&mut self) -> Result<(f32, f32, f32), DecodeError> {
        Ok((self.f32()?, self.f32()?, self.f32()?))
    }
    pub(crate) fn take(&mut self, n: usize) -> Result<&'a [u8], DecodeError> {
        if self.0.len() < n {
            return Err(DecodeError::TooShort);
        }
        let (a, b) = self.0.split_at(n);
        self.0 = b;
        Ok(a)
    }
    pub(crate) fn u8(&mut self) -> Result<u8, DecodeError> {
        Ok(self.take(1)?[0])
    }
    pub(crate) fn u16(&mut self) -> Result<u16, DecodeError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    pub(crate) fn u32(&mut self) -> Result<u32, DecodeError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    pub(crate) fn f32(&mut self) -> Result<f32, DecodeError> {
        let v = f32::from_le_bytes(self.take(4)?.try_into().unwrap());
        // never let a NaN or infinity into the game
        Ok(if v.is_finite() { v } else { 0.0 })
    }
    fn text(&mut self) -> Result<String, DecodeError> {
        let n = self.u8()? as usize;
        let bytes = self.take(n)?;
        let s = std::str::from_utf8(bytes).map_err(|_| DecodeError::BadText)?;
        if s.chars().count() > NAME_MAX || s.chars().any(|c| c.is_control()) {
            return Err(DecodeError::BadText);
        }
        Ok(s.to_string())
    }
    fn look(&mut self) -> Result<crate::appearance::Appearance, DecodeError> {
        use crate::appearance::LOOK_BYTES;
        let b: [u8; LOOK_BYTES] = self.take(LOOK_BYTES)?.try_into().unwrap();
        Ok(crate::appearance::Appearance::from_bytes(b))
    }
    fn member(&mut self) -> Result<Member, DecodeError> {
        let (id, name, character, flags) = (self.u32()?, self.text()?, self.u8()?.min(3), self.u8()?);
        Ok(Member { id, name, character, host: flags & 1 != 0, ready: flags & 2 != 0, look: self.look()?, belly: self.u8()? })
    }
    fn members(&mut self) -> Result<Vec<Member>, DecodeError> {
        let n = self.u8()? as usize;
        if n > MAX_PLAYERS {
            return Err(DecodeError::TooMany);
        }
        (0..n).map(|_| self.member()).collect()
    }
}

// ---------------------------------------------------------------- messages

const K_HELLO: u8 = 1;
const K_WELCOME: u8 = 2;
const K_REFUSED: u8 = 3;
const K_ROSTER: u8 = 4;
const K_STATE: u8 = 5;
const K_BYE: u8 = 6;
const K_WORLD: u8 = 7;
const K_ACT: u8 = 8;
const K_HIT: u8 = 9;
const K_LOOK: u8 = 10;
const K_READY: u8 = 11;
/// For the tests in `net_act.rs`.
#[cfg(test)]
pub(crate) const K_ACT_FOR_TESTS: u8 = K_ACT;

const F_GROUNDED: u16 = 1;
const F_STUNNED: u16 = 2;
const F_DOWN: u16 = 4;
const F_DRINKING: u16 = 8;

impl Msg {
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer(Vec::with_capacity(48));
        match self {
            Msg::Hello { protocol, name, character, look, belly } => {
                w.u8(K_HELLO);
                w.u16(*protocol);
                w.text(name);
                w.u8(*character);
                w.0.extend_from_slice(&look.to_bytes());
                w.u8(*belly);
            }
            Msg::Look { character, look, belly } => {
                w.u8(K_LOOK);
                w.u8(*character);
                w.0.extend_from_slice(&look.to_bytes());
                w.u8(*belly);
            }
            Msg::Ready(on) => {
                w.u8(K_READY);
                w.u8(*on as u8);
            }
            Msg::Welcome { id, members } => {
                w.u8(K_WELCOME);
                w.u32(*id);
                w.members(members);
            }
            Msg::Refused(r) => {
                w.u8(K_REFUSED);
                w.u8(match r {
                    Refusal::WrongVersion => 0,
                    Refusal::Full => 1,
                    Refusal::InProgress => 2,
                });
            }
            Msg::Roster { members, start_in } => {
                w.u8(K_ROSTER);
                w.members(members);
                w.u8(*start_in);
            }
            Msg::State(s) => {
                w.u8(K_STATE);
                w.u32(s.id);
                w.f32(s.pos.0);
                w.f32(s.pos.1);
                w.f32(s.pos.2);
                w.f32(s.yaw);
                w.f32(s.vel.0);
                w.f32(s.vel.1);
                w.f32(s.walk);
                let mut f = 0u16;
                for (on, bit) in [(s.grounded, F_GROUNDED), (s.stunned, F_STUNNED), (s.down, F_DOWN), (s.drinking, F_DRINKING)] {
                    if on {
                        f |= bit;
                    }
                }
                w.u16(f);
                w.u8((s.charge.clamp(0.0, 1.0) * 255.0).round() as u8);
                w.u8(s.held);
                w.u8(s.held_variant);
                w.u8(s.belly);
                w.u8(s.dive);
            }
            Msg::Bye { id } => {
                w.u8(K_BYE);
                w.u32(*id);
            }
            Msg::World(snap) => {
                w.u8(K_WORLD);
                snap.write(&mut w);
            }
            Msg::Act(a) => {
                w.u8(K_ACT);
                a.write(&mut w);
            }
            Msg::Hit(h) => {
                w.u8(K_HIT);
                h.write(&mut w);
            }
        }
        w.0
    }

    pub fn decode(bytes: &[u8]) -> Result<Msg, DecodeError> {
        let mut r = Reader(bytes);
        let msg = match r.u8()? {
            K_HELLO => Msg::Hello { protocol: r.u16()?, name: r.text()?, character: r.u8()?.min(3), look: r.look()?, belly: r.u8()? },
            K_LOOK => Msg::Look { character: r.u8()?.min(3), look: r.look()?, belly: r.u8()? },
            K_READY => Msg::Ready(match r.u8()? {
                0 => false,
                1 => true,
                k => return Err(DecodeError::UnknownKind(k)),
            }),
            K_WELCOME => Msg::Welcome { id: r.u32()?, members: r.members()? },
            K_REFUSED => Msg::Refused(match r.u8()? {
                0 => Refusal::WrongVersion,
                1 => Refusal::Full,
                2 => Refusal::InProgress,
                k => return Err(DecodeError::UnknownKind(k)),
            }),
            K_ROSTER => Msg::Roster { members: r.members()?, start_in: r.u8()? },
            K_STATE => {
                let id = r.u32()?;
                let pos = (r.f32()?, r.f32()?, r.f32()?);
                let yaw = r.f32()?;
                let vel = (r.f32()?, r.f32()?);
                let walk = r.f32()?;
                let f = r.u16()?;
                Msg::State(PlayerState {
                    id,
                    pos,
                    yaw,
                    vel,
                    walk,
                    grounded: f & F_GROUNDED != 0,
                    stunned: f & F_STUNNED != 0,
                    down: f & F_DOWN != 0,
                    drinking: f & F_DRINKING != 0,
                    charge: r.u8()? as f32 / 255.0,
                    held: r.u8()?,
                    held_variant: r.u8()?,
                    belly: r.u8()?,
                    dive: r.u8()?,
                })
            }
            K_BYE => Msg::Bye { id: r.u32()? },
            K_WORLD => Msg::World(Box::new(crate::net_world::WorldSnap::read(&mut r)?)),
            K_ACT => Msg::Act(crate::net_act::GuestAct::read(&mut r)?),
            K_HIT => Msg::Hit(crate::net_act::HitMsg::read(&mut r)?),
            k => return Err(DecodeError::UnknownKind(k)),
        };
        if !r.0.is_empty() {
            return Err(DecodeError::TrailingBytes);
        }
        Ok(msg)
    }
}

// ---------------------------------------------------------------- the host's list of people

/// The host's book of who is in the yard. Pure bookkeeping: hands out ids, says yes or no to
/// a hello, and forgets people who leave.
#[derive(Clone, Debug, Default)]
pub struct Roster {
    members: Vec<Member>,
    next_id: u32,
}

impl Roster {
    /// A yard with just its host (always id 1).
    pub fn new(host_name: &str, character: u8) -> Self {
        Roster { members: vec![Member::plain(1, &clean(host_name), character, true)], next_id: 2 }
    }

    /// Somebody changed their look (the host for itself, or a guest's `Look`).
    pub fn set_look(&mut self, id: u32, character: u8, look: crate::appearance::Appearance, belly: u8) -> bool {
        match self.members.iter_mut().find(|m| m.id == id) {
            Some(m) if (m.character, m.look, m.belly) != (character.min(3), look.tidy(), belly) => {
                m.character = character.min(3);
                m.look = look.tidy();
                m.belly = belly;
                true
            }
            _ => false,
        }
    }

    /// Somebody pressed Ready (or un-pressed it). Returns true if anything changed.
    pub fn set_ready(&mut self, id: u32, ready: bool) -> bool {
        match self.members.iter_mut().find(|m| m.id == id) {
            Some(m) if m.ready != ready => {
                m.ready = ready;
                true
            }
            _ => false,
        }
    }

    /// Back from a round: nobody is ready any more.
    pub fn clear_ready(&mut self) -> bool {
        let any = self.members.iter().any(|m| m.ready);
        for m in &mut self.members {
            m.ready = false;
        }
        any
    }

    /// Everybody is ready, and there is somebody to play with.
    pub fn all_ready(&self) -> bool {
        self.members.len() >= 2 && self.members.iter().all(|m| m.ready)
    }

    pub fn members(&self) -> &[Member] {
        &self.members
    }

    /// Somebody said hello: let them in (`Ok(their id)`) or say why not.
    pub fn admit(&mut self, protocol: u16, name: &str, character: u8, round_on: bool) -> Result<u32, Refusal> {
        self.admit_with(protocol, name, character, Default::default(), 128, round_on)
    }

    /// `admit` with the newcomer's look.
    pub fn admit_with(
        &mut self,
        protocol: u16,
        name: &str,
        character: u8,
        look: crate::appearance::Appearance,
        belly: u8,
        round_on: bool,
    ) -> Result<u32, Refusal> {
        if protocol != PROTOCOL {
            return Err(Refusal::WrongVersion);
        }
        if self.members.len() >= MAX_PLAYERS {
            return Err(Refusal::Full);
        }
        if round_on {
            return Err(Refusal::InProgress);
        }
        let id = self.next_id;
        self.next_id += 1;
        let name = self.unique_name(&clean(name));
        self.members.push(Member { id, name, character: character.min(3), host: false, look: look.tidy(), belly, ready: false });
        Ok(id)
    }

    pub fn remove(&mut self, id: u32) -> bool {
        let before = self.members.len();
        self.members.retain(|m| m.id != id || m.host);
        self.members.len() != before
    }

    /// Two people may not both be "Davo": the second becomes "Davo 2".
    fn unique_name(&self, name: &str) -> String {
        let taken = |n: &str| self.members.iter().any(|m| m.name == n);
        if !taken(name) {
            return name.to_string();
        }
        (2..)
            .map(|k| {
                let suffix = format!(" {k}");
                let base: String = name.chars().take(NAME_MAX - suffix.chars().count()).collect();
                format!("{base}{suffix}")
            })
            .find(|n| !taken(n))
            .unwrap()
    }
}

/// A name people can see: trimmed, no control characters, at most 16 characters, never empty.
pub fn clean(name: &str) -> String {
    let s: String = name.chars().filter(|c| !c.is_control()).take(NAME_MAX).collect();
    let s = s.trim().to_string();
    if s.is_empty() { "Mate".to_string() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> PlayerState {
        PlayerState {
            id: 7,
            pos: (12.5, 0.25, -9.75),
            yaw: 1.5,
            vel: (2.0, -3.0),
            walk: 41.5,
            grounded: true,
            stunned: false,
            down: true,
            drinking: true,
            charge: 1.0,
            held: 4,
            held_variant: 5,
            belly: 200,
            dive: 180,
        }
    }

    fn members() -> Vec<Member> {
        vec![
            Member { ready: true, belly: 200, ..Member::plain(1, "Marcus", 3, true) },
            Member {
                look: crate::appearance::Appearance { hair: crate::appearance::Hair::Mullet, body: 6, ..Default::default() },
                ..Member::plain(2, "Dävo ✓", 0, false)
            },
        ]
    }

    #[test]
    fn every_message_survives_the_trip() {
        for m in [
            Msg::Hello { protocol: PROTOCOL, name: "Shazza".into(), character: 2, look: Default::default(), belly: 77 },
            Msg::Look { character: 1, look: crate::appearance::Appearance { mouth: crate::appearance::Mouth::Grin, ..Default::default() }, belly: 3 },
            Msg::Ready(true),
            Msg::Ready(false),
            Msg::Welcome { id: 4, members: members() },
            Msg::Refused(Refusal::Full),
            Msg::Refused(Refusal::WrongVersion),
            Msg::Refused(Refusal::InProgress),
            Msg::Roster { members: members(), start_in: 0 },
            Msg::Roster { members: vec![], start_in: 25 },
            Msg::State(state()),
            Msg::Bye { id: 9 },
        ] {
            let bytes = m.encode();
            assert_eq!(Msg::decode(&bytes), Ok(m.clone()), "{m:?}");
        }
    }

    #[test]
    fn a_state_message_is_small() {
        // about 20 of these a second from each of 16 people
        assert!(Msg::State(state()).encode().len() <= 42);
    }

    #[test]
    fn charge_is_kept_to_a_part_in_255() {
        let mut s = state();
        s.charge = 0.5;
        let Ok(Msg::State(back)) = Msg::decode(&Msg::State(s).encode()) else { panic!() };
        assert!((back.charge - 0.5).abs() < 0.005);
    }

    #[test]
    fn rubbish_never_panics_and_is_refused() {
        // every cut-off version of every message
        for m in [Msg::State(state()), Msg::Welcome { id: 2, members: members() }, Msg::Hello { protocol: 1, name: "x".into(), character: 0, look: Default::default(), belly: 1 }] {
            let bytes = m.encode();
            for n in 0..bytes.len() {
                assert!(Msg::decode(&bytes[..n]).is_err(), "{m:?} cut to {n}");
            }
            let mut more = bytes.clone();
            more.push(0);
            assert_eq!(Msg::decode(&more), Err(DecodeError::TrailingBytes));
        }
        assert_eq!(Msg::decode(&[]), Err(DecodeError::TooShort));
        assert_eq!(Msg::decode(&[99]), Err(DecodeError::UnknownKind(99)));
        assert_eq!(Msg::decode(&[K_REFUSED, 9]), Err(DecodeError::UnknownKind(9)));
        // a roster claiming 200 members
        assert_eq!(Msg::decode(&[K_ROSTER, 200]), Err(DecodeError::TooMany));
        // a name that is not valid text, or has control characters
        assert_eq!(Msg::decode(&[K_HELLO, 1, 0, 2, 0xff, 0xfe, 0]), Err(DecodeError::BadText));
        assert_eq!(Msg::decode(&[K_HELLO, 1, 0, 1, 7, 0]), Err(DecodeError::BadText));
        // pseudo-random bytes of every length
        let mut x = 0x2545F491u32;
        for len in 0..80 {
            for _ in 0..200 {
                let junk: Vec<u8> = (0..len)
                    .map(|_| {
                        x ^= x << 13;
                        x ^= x >> 17;
                        x ^= x << 5;
                        (x >> 8) as u8
                    })
                    .collect();
                let _ = Msg::decode(&junk);
            }
        }
    }

    #[test]
    fn nan_and_infinity_never_get_in() {
        let mut s = state();
        s.pos = (f32::NAN, f32::INFINITY, 1.0);
        let Ok(Msg::State(back)) = Msg::decode(&Msg::State(s).encode()) else { panic!() };
        assert_eq!(back.pos, (0.0, 0.0, 1.0));
    }

    #[test]
    fn long_names_are_cut_not_refused() {
        let m = Msg::Hello { protocol: 1, name: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".into(), character: 0, look: Default::default(), belly: 1 };
        let Ok(Msg::Hello { name, .. }) = Msg::decode(&m.encode()) else { panic!() };
        assert_eq!(name, "ABCDEFGHIJKLMNOP");
        // and 16 four-byte characters still fit the one-byte length
        let m = Msg::Hello { protocol: 1, name: "😀".repeat(20), character: 0, look: Default::default(), belly: 1 };
        let Ok(Msg::Hello { name, .. }) = Msg::decode(&m.encode()) else { panic!() };
        assert_eq!(name.chars().count(), 16);
    }

    #[test]
    fn the_roster_hands_out_ids_and_says_no_politely() {
        let mut r = Roster::new("Marcus", 3);
        assert_eq!(r.members()[0].id, 1);
        assert_eq!(r.admit(PROTOCOL, "Davo", 0, false), Ok(2));
        assert_eq!(r.admit(PROTOCOL, "Davo", 1, false), Ok(3));
        assert_eq!(r.members()[2].name, "Davo 2", "no two people share a name");
        assert_eq!(r.admit(PROTOCOL + 1, "Kev", 0, false), Err(Refusal::WrongVersion));
        assert_eq!(r.admit(PROTOCOL, "Kev", 0, true), Err(Refusal::InProgress));
        assert!(r.remove(2));
        assert!(!r.remove(2));
        assert!(!r.remove(1), "the host cannot be removed");
        // ids are never reused
        assert_eq!(r.admit(PROTOCOL, "Kev", 0, false), Ok(4));
        // 16 is the limit
        while r.members().len() < MAX_PLAYERS {
            r.admit(PROTOCOL, "x", 0, false).unwrap();
        }
        assert_eq!(r.admit(PROTOCOL, "late", 0, false), Err(Refusal::Full));
    }

    #[test]
    fn the_roster_keeps_looks_and_who_is_ready() {
        use crate::appearance::{Appearance, Hair};
        let mut r = Roster::new("Marcus", 3);
        let mullet = Appearance { hair: Hair::Mullet, ..Default::default() };
        let id = r.admit_with(PROTOCOL, "Davo", 1, mullet, 40, false).unwrap();
        assert_eq!(r.members()[1].look, mullet);
        assert_eq!(r.members()[1].belly, 40);
        assert!(!r.all_ready(), "nobody is ready yet");
        assert!(r.set_ready(1, true));
        assert!(!r.set_ready(1, true), "no change the second time");
        assert!(!r.all_ready());
        assert!(r.set_ready(id, true));
        assert!(r.all_ready(), "both ready");
        // a look change shows up, and the same look again is no change
        let bowl = Appearance { hair: Hair::Bowl, ..Default::default() };
        assert!(r.set_look(id, 2, bowl, 99));
        assert!(!r.set_look(id, 2, bowl, 99));
        assert_eq!((r.members()[1].character, r.members()[1].belly), (2, 99));
        assert!(!r.set_look(77, 0, bowl, 1), "nobody with that id");
        // back from a round
        assert!(r.clear_ready());
        assert!(!r.all_ready());
        // alone in the yard is never "all ready" (there is nobody to play with)
        let mut alone = Roster::new("Marcus", 0);
        alone.set_ready(1, true);
        assert!(!alone.all_ready());
    }

    #[test]
    fn names_are_cleaned_up() {
        assert_eq!(clean("  Davo \n"), "Davo");
        assert_eq!(clean(""), "Mate");
        assert_eq!(clean("\u{7}\u{7}"), "Mate");
        assert_eq!(clean("ABCDEFGHIJKLMNOPQRSTUVWXYZ").chars().count(), 16);
    }
}
