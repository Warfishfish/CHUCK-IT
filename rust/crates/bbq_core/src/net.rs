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
pub const PROTOCOL: u16 = 1;
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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    /// Guest to host: "can I play?"
    Hello { protocol: u16, name: String, character: u8 },
    /// Host to one guest: "yes, you are `id`", with everyone already here.
    Welcome { id: u32, members: Vec<Member> },
    /// Host to one guest: "no".
    Refused(Refusal),
    /// Host to everyone: who is here now (sent when somebody joins or leaves).
    Roster(Vec<Member>),
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
        self.u8(m.host as u8);
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
    fn member(&mut self) -> Result<Member, DecodeError> {
        Ok(Member { id: self.u32()?, name: self.text()?, character: self.u8()?, host: self.u8()? != 0 })
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
            Msg::Hello { protocol, name, character } => {
                w.u8(K_HELLO);
                w.u16(*protocol);
                w.text(name);
                w.u8(*character);
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
            Msg::Roster(ms) => {
                w.u8(K_ROSTER);
                w.members(ms);
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
            K_HELLO => Msg::Hello { protocol: r.u16()?, name: r.text()?, character: r.u8()? },
            K_WELCOME => Msg::Welcome { id: r.u32()?, members: r.members()? },
            K_REFUSED => Msg::Refused(match r.u8()? {
                0 => Refusal::WrongVersion,
                1 => Refusal::Full,
                2 => Refusal::InProgress,
                k => return Err(DecodeError::UnknownKind(k)),
            }),
            K_ROSTER => Msg::Roster(r.members()?),
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
        Roster {
            members: vec![Member { id: 1, name: clean(host_name), character, host: true }],
            next_id: 2,
        }
    }

    pub fn members(&self) -> &[Member] {
        &self.members
    }

    /// Somebody said hello: let them in (`Ok(their id)`) or say why not.
    pub fn admit(&mut self, protocol: u16, name: &str, character: u8, round_on: bool) -> Result<u32, Refusal> {
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
        self.members.push(Member { id, name: self.unique_name(&clean(name)), character, host: false });
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
        }
    }

    fn members() -> Vec<Member> {
        vec![
            Member { id: 1, name: "Marcus".into(), character: 3, host: true },
            Member { id: 2, name: "Dävo ✓".into(), character: 0, host: false },
        ]
    }

    #[test]
    fn every_message_survives_the_trip() {
        for m in [
            Msg::Hello { protocol: PROTOCOL, name: "Shazza".into(), character: 2 },
            Msg::Welcome { id: 4, members: members() },
            Msg::Refused(Refusal::Full),
            Msg::Refused(Refusal::WrongVersion),
            Msg::Refused(Refusal::InProgress),
            Msg::Roster(members()),
            Msg::Roster(vec![]),
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
        for m in [Msg::State(state()), Msg::Welcome { id: 2, members: members() }, Msg::Hello { protocol: 1, name: "x".into(), character: 0 }] {
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
        let m = Msg::Hello { protocol: 1, name: "ABCDEFGHIJKLMNOPQRSTUVWXYZ".into(), character: 0 };
        let Ok(Msg::Hello { name, .. }) = Msg::decode(&m.encode()) else { panic!() };
        assert_eq!(name, "ABCDEFGHIJKLMNOP");
        // and 16 four-byte characters still fit the one-byte length
        let m = Msg::Hello { protocol: 1, name: "😀".repeat(20), character: 0 };
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
    fn names_are_cleaned_up() {
        assert_eq!(clean("  Davo \n"), "Davo");
        assert_eq!(clean(""), "Mate");
        assert_eq!(clean("\u{7}\u{7}"), "Mate");
        assert_eq!(clean("ABCDEFGHIJKLMNOPQRSTUVWXYZ").chars().count(), 16);
    }
}
