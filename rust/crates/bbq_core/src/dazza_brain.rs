//! Dazza's brain: anger, chasing, the spatula, and what happens when somebody slaps him
//! (spec section 12). His look and animation are in `dazza`.

use std::collections::BTreeMap;

use crate::PlayerId;
use crate::dazza::{ANGER_MAX, DazzaState, HOME};
use crate::rng::Rng;
use crate::vec::V3;
use crate::yard::Yard;
use crate::{YARD_HALF_X, YARD_HALF_Z};

/// How long he stays "angry" after someone takes meat.
pub const ANGRY_TIME: f32 = 3.5;
/// Knocked out for this long.
pub const KO_TIME: f32 = 8.0;
/// Speeds, metres per second.
pub const RETURN_SPEED: f32 = 4.0;
pub const ANGRY_SPEED: f32 = 3.4;
/// He stops this far from whoever he is going for.
pub const STOP_DIST: f32 = 1.2;
/// Spatula: swing when this close, cooldown between swings, the hit lands this much later and
/// only if the victim is still within `SPATULA_HIT_REACH`.
pub const SWING_REACH: f32 = 2.1;
pub const SWING_COOLDOWN: f32 = 1.3;
pub const SPATULA_DELAY: f32 = 0.22;
pub const SPATULA_HIT_REACH: f32 = 2.7;
/// Dizzy stun from a spatula hit.
pub const SPATULA_STUN: f32 = 1.6;
pub const SPATULA_STUN_BERSERK: f32 = 2.8;
/// The spatula's knockback (the number the JavaScript game gives it).
pub const SPATULA_KNOCK: f32 = 7.0;
/// A berserk head start before he first swings.
pub const BERSERK_HEAD_START: f32 = 1.2;
/// He gives a grudge a full chase when it reaches this.
pub const GRUDGE_CHASE: f32 = 2.5;
/// Grudges fade at 1 per 25 s.
pub const GRUDGE_DECAY: f32 = 1.0 / 25.0;
/// Radius against boxes, and margin inside the fence.
pub const RADIUS: f32 = 0.45;
pub const FENCE_MARGIN: f32 = 0.5;
/// Slap range the host allows, and its cooldown.
pub const SLAP_REACH: f32 = 3.6;
pub const SLAP_COOLDOWN: f32 = 0.45;
/// A KO can drop a Bum-Out gnome (Cheeky mode), at most this many per round.
pub const BUM_OUTS_PER_ROUND: u32 = 3;
pub const BUM_OUT_CHANCE: f32 = 0.4;
/// Meat thrown off the grill when the barbie is flipped.
pub const FLIP_MEATS: usize = 5;
/// Where the grill food sits (for the flip).
pub const GRILL: V3 = V3::new(-6.0, 0.0, -18.0);

/// Which kind of line he is saying.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Line {
    Angry,
    Rage,
    Gotcha,
    GiveUp,
    Ko,
    Wake,
    Berserk,
    Stunned,
    Flip,
    Smoko,
    Chat,
}

const ANGRY: &[&str] = &[
    "OI! Hrmblgrr MY SNAGS!",
    "Wha- BLURGH! Put it DOWN!",
    "Yabba grrbl, ya drongo!",
    "HNNG! Oi oi OI!",
    "Mrrph! Bloody thief!",
    "Grrblmph! Me STEAKS!",
    "Oi! Wobba wobba NO!",
];
const RAGE: &[&str] = &[
    "RAAAAH! COME 'ERE!",
    "GRRBLARGH! I'LL GETCHA!",
    "THAT'S IT! OI OI OI!",
    "BLAAARGH! YER DEAD MEAT!",
    "I'VE HAD IT UP TO 'ERE WITH YA!",
    "RIGHT, THAT'S WAR!",
];
const GOTCHA: &[&str] = &[
    "HAH! Hrrmph.",
    "That'll learn ya. Mrrph.",
    "Snags don't grow on trees!",
];
const GIVEUP: &[&str] = &[
    "Pfff... hrrmph. Me snags are burnin'.",
    "Bloody kids. Grrbl.",
];
const KO: &[&str] = &["Urrrgh...", "*snore*"];
const WAKE: &[&str] = &["Wha... who turned the lights off?", "Me head... ME SNAGS!"];
const BERSERK: &[&str] = &[
    "YOU SLAPPED ME WITH ME OWN STEAK?!",
    "RAAAARGH! YOU'RE DONE!",
    "THAT'S IT. NO MORE MR NICE DAZZA!",
];
const STUNNED: &[&str] = &[
    "OI! Right in the... OW!",
    "Me EYE! Ya mongrel!",
    "Cheeky little drongo!",
    "GRRBL! That stings!",
];
const FLIP: &[&str] = &[
    "ME SNAGS!! FLIPPIN' 'ELL!",
    "THAT'S IT! NOBODY EATS!",
    "RAAAH! HAVE IT ALL THEN!",
];
const SMOKO: &[&str] = &[
    "Fair enough. It's smoko.",
    "Hrrmph. Smoko's sacred.",
    "Enjoy yer smoko, ya thief.",
];
const CHAT: &[&str] = &[
    "Snags are nearly done!",
    "Who wants a steak? Not you.",
    "Hrrmph. Perfect sear.",
    "Onions! Where's me onions?",
    "Mmm. Blurgh. Beautiful.",
];
const CHAT_CHEEKY: &[&str] = &[
    "These snags have seen things.",
    "Dunny's blocked again. Don't ask.",
    "Me missus reckons I love this grill more than her. She's not wrong.",
];
const ANGRY_CHEEKY: &[&str] = &[
    "OI! Keep yer hands off me snags, ya pervert!",
    "Get back 'ere, ya cheeky bugger!",
];
const RAGE_CHEEKY: &[&str] = &[
    "RIGHT, I'M TELLIN' YA NAN!",
    "THAT'S IT, PANTS DOWN, OVER ME KNEE!",
];

impl Line {
    fn base(self) -> &'static [&'static str] {
        match self {
            Line::Angry => ANGRY,
            Line::Rage => RAGE,
            Line::Gotcha => GOTCHA,
            Line::GiveUp => GIVEUP,
            Line::Ko => KO,
            Line::Wake => WAKE,
            Line::Berserk => BERSERK,
            Line::Stunned => STUNNED,
            Line::Flip => FLIP,
            Line::Smoko => SMOKO,
            Line::Chat => CHAT,
        }
    }

    fn cheeky(self) -> &'static [&'static str] {
        match self {
            Line::Angry => ANGRY_CHEEKY,
            Line::Rage => RAGE_CHEEKY,
            Line::Chat => CHAT_CHEEKY,
            _ => &[],
        }
    }

    /// Pick a line. Cheeky mode adds the rude ones to the pool.
    pub fn pick(self, cheeky: bool, rng: &mut Rng) -> &'static str {
        let base = self.base();
        let extra = if cheeky { self.cheeky() } else { &[] };
        let n = base.len() + extra.len();
        let i = rng.index(n);
        if i < base.len() {
            base[i]
        } else {
            extra[i - base.len()]
        }
    }
}

/// Somebody Dazza might go after.
#[derive(Clone, Copy, Debug)]
pub struct Prey {
    pub id: PlayerId,
    pub pos: V3,
    pub down_t: f32,
    pub at_smoko: bool,
}

/// Things that happened that the game should show, sound or act on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DazzaEvent {
    Say { line: Line, text: &'static str },
    /// The spatula swing began.
    Swing,
    /// The spatula connected. Knock the victim along `dir` and stun them.
    Spatula {
        victim: PlayerId,
        dir: V3,
        berserk: bool,
    },
}

/// How a slap on Dazza turned out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Slap {
    /// Steak, fish or noodle: a short stun. Run!
    Stun { seconds: f32 },
    /// Dildo, outcome 1: knocked out cold. `bum_out` means a Bum-Out gnome pops out.
    Ko { bum_out: bool },
    /// Dildo, outcome 2.
    Berserk,
    /// Dildo, outcome 3: the barbie flips and meat flies off the grill.
    Flip,
}

impl Slap {
    /// Index into `scoring::DAZZA_SLAP` (KO, berserk, flip). Stuns pay separately.
    pub fn points_index(self) -> Option<usize> {
        match self {
            Slap::Stun { .. } => None,
            Slap::Ko { .. } => Some(0),
            Slap::Berserk => Some(1),
            Slap::Flip => Some(2),
        }
    }

    pub fn line(self) -> Line {
        match self {
            Slap::Stun { .. } => Line::Stunned,
            Slap::Ko { .. } => Line::Ko,
            Slap::Berserk => Line::Berserk,
            Slap::Flip => Line::Flip,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Dazza {
    pub pos: V3,
    /// Which way his mesh faces: he looks along `(sin f, cos f)` (0 = looking at the yard).
    pub face: f32,
    pub state: DazzaState,
    pub target: Option<PlayerId>,
    state_t: f32,
    swing_cd: f32,
    pending: Option<(PlayerId, f32)>,
    grudges: BTreeMap<PlayerId, f32>,
    /// Anger level 0..=8.
    pub level: u32,
    stun_after: f32,
    pub berserk: bool,
    chat_t: f32,
    pub bum_out_left: u32,
    /// How far he moved on the last tick (drives his walk cycle).
    pub moved: f32,
    /// Last slap time (the host's cooldown).
    last_slap: f32,
}

impl Default for Dazza {
    fn default() -> Self {
        Self::new()
    }
}

fn clamp_level(l: u32) -> u32 {
    l.min(ANGER_MAX)
}

fn ang_lerp(a: f32, b: f32, t: f32) -> f32 {
    let mut d = (b - a) % std::f32::consts::TAU;
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    if d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    a + d * t
}

impl Dazza {
    pub fn new() -> Self {
        Dazza {
            pos: HOME,
            face: 0.0,
            state: DazzaState::Cook,
            target: None,
            state_t: 0.0,
            swing_cd: 0.0,
            pending: None,
            grudges: BTreeMap::new(),
            level: 0,
            stun_after: 0.0,
            berserk: false,
            chat_t: 14.0,
            bum_out_left: BUM_OUTS_PER_ROUND,
            moved: 0.0,
            last_slap: -9.0,
        }
    }

    /// How long he chases for: `clamp(8 + level * 2, 8, 26)`.
    pub fn chase_dur(&self) -> f32 {
        (8.0 + self.level as f32 * 2.0).clamp(8.0, 26.0)
    }

    /// How long a steak or fish stun lasts: `max(0.4, 1 - level * 0.08)`.
    pub fn stun_dur(&self) -> f32 {
        (1.0 - self.level as f32 * 0.08).max(0.4)
    }

    /// Chase speed: `7.2 + min(2.2, level * 0.3)`, or `8.6 + ...` when berserk.
    pub fn chase_speed(&self) -> f32 {
        (if self.berserk { 8.6 } else { 7.2 }) + (self.level as f32 * 0.3).min(2.2)
    }

    /// Can he be slapped right now?
    pub fn slappable(&self) -> bool {
        !matches!(self.state, DazzaState::Ko | DazzaState::Stunned)
    }

    /// Can he be hit by a swing at all (a KO'd Dazza can't be swung at either)? The client's
    /// check is looser than the host's.
    pub fn swingable(&self) -> bool {
        self.state != DazzaState::Ko
    }

    /// Someone's grudge against... his grudge for this player (for tests and the HUD).
    pub fn grudge(&self, id: PlayerId) -> f32 {
        self.grudges.get(&id).copied().unwrap_or(0.0)
    }

    /// Round start: he walks home, forgives everybody and calms down.
    pub fn reset(&mut self) {
        self.state = DazzaState::Return;
        self.target = None;
        self.grudges.clear();
        self.pending = None;
        self.level = 0;
        self.stun_after = 0.0;
        self.berserk = false;
        self.bum_out_left = BUM_OUTS_PER_ROUND;
    }

    fn say(&self, line: Line, cheeky: bool, rng: &mut Rng) -> DazzaEvent {
        DazzaEvent::Say {
            line,
            text: line.pick(cheeky, rng),
        }
    }

    /// His spot is behind the grill: if he's out the front he walks round the open right side.
    pub fn home_route(&self) -> V3 {
        let p = self.pos;
        if p.z < -18.75 {
            return HOME;
        }
        if p.x > -4.6 {
            return V3::new(-4.3, 0.0, -19.3);
        }
        V3::new(-4.3, 0.0, p.z.max(-17.1))
    }

    /// Going out to meet someone from behind the grill: the JavaScript game walked him straight
    /// at them, so a thief standing right in front of the grill left him pushing against it.
    /// This sends him round the open right side first (the same way he comes home).
    /// **A small fix, not a port.**
    fn out_route(&self, target: V3) -> (V3, f32) {
        let (x, z) = (self.pos.x, self.pos.z);
        if target.z > -18.4 && z < -17.3 && x > -9.6 && x < -3.9 {
            if x < -4.35 {
                // first out to the open right-hand side...
                (V3::new(-4.3, 0.0, -19.3), 0.0)
            } else {
                // ...then up past the front corner of the grill
                (V3::new(-4.3, 0.0, -16.9), 0.0)
            }
        } else {
            (target, STOP_DIST)
        }
    }

    /// Someone took meat off the table (`npcAngry`).
    pub fn meat_stolen(&mut self, id: PlayerId, cheeky: bool, rng: &mut Rng) -> Vec<DazzaEvent> {
        let mut out = Vec::new();
        if matches!(self.state, DazzaState::Ko | DazzaState::Stunned) {
            return out;
        }
        let n = self.grudge(id) + 1.0;
        self.grudges.insert(id, n);
        self.target = Some(id);
        self.level = clamp_level(self.level + 1);
        if n >= GRUDGE_CHASE {
            self.state = DazzaState::Chase;
            self.state_t = self.chase_dur();
            out.push(self.say(Line::Rage, cheeky, rng));
        } else if self.state != DazzaState::Chase {
            self.state = DazzaState::Angry;
            self.state_t = ANGRY_TIME;
            out.push(self.say(Line::Angry, cheeky, rng));
        }
        out
    }

    /// The host's checks for a slap on Dazza: not KO'd or stunned, attacker not sitting down,
    /// within 3.6 m, and at least 0.45 s since the last one. If it passes, the slap resolves.
    #[allow(clippy::too_many_arguments)]
    pub fn try_slap(
        &mut self,
        now: f32,
        attacker: PlayerId,
        attacker_pos: V3,
        attacker_seated: bool,
        stun_melee: bool,
        cheeky: bool,
        rng: &mut Rng,
    ) -> Option<Slap> {
        if !self.slappable()
            || attacker_seated
            || now - self.last_slap < SLAP_COOLDOWN
            || attacker_pos.horiz_dist(self.pos) > SLAP_REACH
        {
            return None;
        }
        self.last_slap = now;
        Some(self.slapped(attacker, stun_melee, cheeky, rng))
    }

    /// Resolve a slap that has been allowed (steak, fish or noodle when `stun_melee`;
    /// otherwise a dildo, with one of three equally likely outcomes).
    pub fn slapped(
        &mut self,
        attacker: PlayerId,
        stun_melee: bool,
        cheeky: bool,
        rng: &mut Rng,
    ) -> Slap {
        self.pending = None;
        if stun_melee {
            let sd = self.stun_dur();
            self.state = DazzaState::Stunned;
            self.state_t = sd;
            self.target = Some(attacker);
            self.stun_after = self.chase_dur();
            self.level = clamp_level(self.level + 1);
            return Slap::Stun { seconds: sd };
        }
        match rng.index(3) {
            0 => {
                self.state = DazzaState::Ko;
                self.state_t = KO_TIME;
                self.target = None;
                self.grudges.clear();
                let bum_out = cheeky && self.bum_out_left > 0 && rng.chance(BUM_OUT_CHANCE);
                if bum_out {
                    self.bum_out_left -= 1;
                }
                Slap::Ko { bum_out }
            }
            1 => {
                self.state = DazzaState::Chase;
                self.target = Some(attacker);
                self.state_t = self.chase_dur();
                self.berserk = true;
                self.swing_cd = BERSERK_HEAD_START;
                self.level = clamp_level(self.level + 1);
                Slap::Berserk
            }
            _ => {
                self.state = DazzaState::Angry;
                self.target = Some(attacker);
                self.state_t = ANGRY_TIME;
                self.level = clamp_level(self.level + 1);
                Slap::Flip
            }
        }
    }

    /// One fixed step (host only). `countdown` stops his chatter.
    pub fn tick(
        &mut self,
        dt: f32,
        prey: &[Prey],
        yard: &Yard,
        countdown: bool,
        cheeky: bool,
        rng: &mut Rng,
    ) -> Vec<DazzaEvent> {
        let mut out = Vec::new();
        self.moved = 0.0;

        if self.state == DazzaState::Ko {
            self.state_t -= dt;
            self.swing_cd = 1.0;
            if self.state_t <= 0.0 {
                self.state = DazzaState::Return;
                out.push(self.say(Line::Wake, cheeky, rng));
            }
            return out;
        }
        let find = |id: Option<PlayerId>| id.and_then(|i| prey.iter().find(|p| p.id == i)).copied();
        if self.state == DazzaState::Stunned {
            self.state_t -= dt;
            self.swing_cd = 1.0;
            if self.state_t <= 0.0 {
                if find(self.target).is_some() {
                    self.state = DazzaState::Chase;
                    self.state_t = if self.stun_after > 0.0 {
                        self.stun_after
                    } else {
                        self.chase_dur()
                    };
                    self.berserk = self.level >= 4;
                    out.push(self.say(Line::Rage, cheeky, rng));
                } else {
                    self.state = DazzaState::Return;
                    self.target = None;
                }
            }
            return out;
        }
        if self.state != DazzaState::Chase {
            self.berserk = false;
        }
        self.swing_cd -= dt;
        self.grudges.retain(|_, v| {
            *v -= dt * GRUDGE_DECAY;
            *v > 0.0
        });

        let mut tgt = find(self.target);
        if let Some(t) = tgt
            && t.at_smoko
            && matches!(self.state, DazzaState::Chase | DazzaState::Angry)
        {
            out.push(self.say(Line::Smoko, cheeky, rng));
            self.state = DazzaState::Return;
            self.target = None;
            self.pending = None;
            tgt = None;
        }
        if matches!(self.state, DazzaState::Angry | DazzaState::Chase) {
            self.state_t -= dt;
            if self.state_t <= 0.0 || tgt.is_none() {
                if self.state == DazzaState::Chase {
                    out.push(self.say(Line::GiveUp, cheeky, rng));
                }
                self.state = DazzaState::Return;
                self.target = None;
                tgt = None;
            }
        }

        // where to go
        let mut goal: Option<V3> = None;
        let (mut speed, mut stop) = (0.0, 0.0);
        match (self.state, tgt) {
            (DazzaState::Chase, Some(t)) => {
                (goal, stop) = {
                    let (g, st) = self.out_route(t.pos);
                    (Some(g), st)
                };
                speed = self.chase_speed();
            }
            (DazzaState::Angry, Some(t)) => {
                if t.pos.horiz_dist(HOME) < 5.0 {
                    (goal, stop) = {
                        let (g, st) = self.out_route(t.pos);
                        (Some(g), st)
                    };
                    speed = ANGRY_SPEED;
                } else {
                    goal = Some(self.home_route());
                    speed = ANGRY_SPEED;
                }
            }
            (DazzaState::Return, _) => {
                goal = Some(self.home_route());
                speed = RETURN_SPEED;
                if self.pos.horiz_dist(HOME) < 0.25 {
                    self.state = DazzaState::Cook;
                    self.pos = HOME;
                }
            }
            _ => {
                self.chat_t -= dt;
                if self.chat_t <= 0.0 {
                    self.chat_t = rng.range(25.0, 35.0);
                    if !countdown {
                        out.push(self.say(Line::Chat, cheeky, rng));
                    }
                }
            }
        }
        if let Some(g) = goal {
            let (dx, dz) = (g.x - self.pos.x, g.z - self.pos.z);
            let d = dx.hypot(dz);
            if d > stop + 0.05 {
                let st = (speed * dt).min(d - stop);
                self.pos.x += dx / d * st;
                self.pos.z += dz / d * st;
                self.moved = st;
            }
            if d > 0.05 {
                self.face = dx.atan2(dz);
            }
        } else {
            self.face = ang_lerp(self.face, 0.0, 1.0 - (-4.0 * dt).exp());
        }

        // keep him out of the furniture and inside the fence
        for c in &yard.colliders {
            if c.h < 0.3 {
                continue;
            }
            let cx = self.pos.x.clamp(c.x0, c.x1);
            let cz = self.pos.z.clamp(c.z0, c.z1);
            let (ex, ez) = (self.pos.x - cx, self.pos.z - cz);
            let d2 = ex * ex + ez * ez;
            if d2 < RADIUS * RADIUS && d2 > 1e-6 {
                let d = d2.sqrt();
                self.pos.x += ex / d * (RADIUS - d);
                self.pos.z += ez / d * (RADIUS - d);
            }
        }
        self.pos.x = self
            .pos
            .x
            .clamp(-YARD_HALF_X + FENCE_MARGIN, YARD_HALF_X - FENCE_MARGIN);
        self.pos.z = self
            .pos
            .z
            .clamp(-YARD_HALF_Z + FENCE_MARGIN, YARD_HALF_Z - FENCE_MARGIN);

        // the spatula
        if let Some(t) = tgt
            && matches!(self.state, DazzaState::Angry | DazzaState::Chase)
            && self.swing_cd <= 0.0
            && self.pending.is_none()
            && t.down_t <= 0.0
            && t.pos.horiz_dist(self.pos) < SWING_REACH
        {
            self.swing_cd = SWING_COOLDOWN;
            out.push(DazzaEvent::Swing);
            self.pending = Some((t.id, SPATULA_DELAY));
        }
        if let Some((id, t)) = self.pending.as_mut() {
            *t -= dt;
            if *t <= 0.0 {
                let id = *id;
                self.pending = None;
                if let Some(v) = prey.iter().find(|p| p.id == id)
                    && !v.at_smoko
                    && v.pos.horiz_dist(self.pos) < SPATULA_HIT_REACH
                {
                    let (dx, dz) = (v.pos.x - self.pos.x, v.pos.z - self.pos.z);
                    let l = dx.hypot(dz);
                    let dir = if l < 1e-2 {
                        V3::new(0.0, 0.0, 1.0)
                    } else {
                        V3::new(dx / l, 0.0, dz / l)
                    };
                    out.push(DazzaEvent::Spatula {
                        victim: v.id,
                        dir,
                        berserk: self.berserk,
                    });
                    if self.state == DazzaState::Chase {
                        self.grudges.insert(v.id, 1.0);
                        self.state = DazzaState::Return;
                        self.target = None;
                        out.push(self.say(Line::Gotcha, cheeky, rng));
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn prey(id: PlayerId, x: f32, z: f32) -> Prey {
        Prey {
            id,
            pos: V3::new(x, 0.0, z),
            down_t: 0.0,
            at_smoko: false,
        }
    }

    fn run(d: &mut Dazza, secs: f32, p: &[Prey], rng: &mut Rng) -> Vec<DazzaEvent> {
        let yard = Yard::default();
        let mut all = Vec::new();
        for _ in 0..(secs / DT) as usize {
            all.extend(d.tick(DT, p, &yard, false, false, rng));
        }
        all
    }

    #[test]
    fn he_starts_cooking_at_home_and_chats_every_25_to_35_seconds() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(1);
        assert_eq!(d.state, DazzaState::Cook);
        assert_eq!(d.pos, HOME);
        // first line after 14 s
        let ev = run(&mut d, 13.5, &[], &mut rng);
        assert!(ev.is_empty());
        let ev = run(&mut d, 1.0, &[], &mut rng);
        assert_eq!(ev.len(), 1);
        // then the gaps are 25..35 s
        let mut times = Vec::new();
        let mut t = 0.0;
        for _ in 0..(300.0 / DT) as usize {
            let e = d.tick(DT, &[], &Yard::default(), false, false, &mut rng);
            t += DT;
            if !e.is_empty() {
                times.push(t);
            }
        }
        assert!(times.len() >= 8, "{times:?}");
        for w in times.windows(2) {
            let gap = w[1] - w[0];
            assert!((24.9..=35.1).contains(&gap), "gap {gap}");
        }
    }

    #[test]
    fn no_chatter_during_the_countdown() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(2);
        let yard = Yard::default();
        let mut said = 0;
        for _ in 0..(40.0 / DT) as usize {
            said += d.tick(DT, &[], &yard, true, false, &mut rng).len();
        }
        assert_eq!(said, 0);
    }

    #[test]
    fn taking_meat_makes_him_angry_then_he_calms_down_and_goes_home() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(3);
        let ev = d.meat_stolen(7, false, &mut rng);
        assert_eq!(ev.len(), 1);
        assert_eq!(d.state, DazzaState::Angry);
        assert_eq!(d.target, Some(7));
        assert_eq!(d.level, 1);
        // thief is far away: he just heads home route and settles
        let p = [prey(7, 20.0, 10.0)];
        run(&mut d, 3.4, &p, &mut rng);
        assert_eq!(d.state, DazzaState::Angry);
        run(&mut d, 0.3, &p, &mut rng);
        // he was already at his spot, so he settles straight back to cooking
        assert_eq!(d.state, DazzaState::Cook);
        assert_eq!(d.pos, HOME);
    }

    #[test]
    fn stealing_three_times_quickly_starts_a_chase() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(4);
        d.meat_stolen(7, false, &mut rng);
        d.meat_stolen(7, false, &mut rng);
        assert_eq!(d.state, DazzaState::Angry);
        d.meat_stolen(7, false, &mut rng);
        assert_eq!(d.state, DazzaState::Chase);
        assert_eq!(d.level, 3);
        assert!((d.chase_dur() - 14.0).abs() < 1e-5);
    }

    #[test]
    fn grudges_fade_so_slow_thieves_only_annoy_him() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(5);
        d.meat_stolen(7, false, &mut rng);
        run(&mut d, 30.0, &[prey(7, 20.0, 10.0)], &mut rng);
        assert!(d.grudge(7) < 0.01, "{}", d.grudge(7));
        d.meat_stolen(7, false, &mut rng);
        assert_eq!(d.state, DazzaState::Angry);
    }

    #[test]
    fn chase_numbers_scale_with_anger() {
        let mut d = Dazza::new();
        assert!((d.chase_speed() - 7.2).abs() < 1e-5);
        assert!((d.stun_dur() - 1.0).abs() < 1e-5);
        d.level = 8;
        assert!((d.chase_speed() - 9.4).abs() < 1e-5);
        assert!((d.stun_dur() - 0.4).abs() < 1e-5);
        assert_eq!(d.chase_dur(), 24.0);
        d.berserk = true;
        assert!((d.chase_speed() - 10.8).abs() < 1e-4);
        d.level = 12;
        assert_eq!(d.chase_dur(), 26.0);
    }

    #[test]
    fn a_chase_catches_the_thief_and_the_spatula_lands_once() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(6);
        for _ in 0..3 {
            d.meat_stolen(7, false, &mut rng);
        }
        assert_eq!(d.state, DazzaState::Chase);
        // thief stands still in the open
        let p = [prey(7, -6.0, -10.0)];
        let ev = run(&mut d, 6.0, &p, &mut rng);
        let swings = ev.iter().filter(|e| matches!(e, DazzaEvent::Swing)).count();
        let hits: Vec<_> = ev
            .iter()
            .filter_map(|e| match e {
                DazzaEvent::Spatula {
                    victim,
                    dir,
                    berserk,
                } => Some((*victim, *dir, *berserk)),
                _ => None,
            })
            .collect();
        assert_eq!(swings, 1);
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, 7);
        assert!(!hits[0].2);
        // he comes from behind the grill, so the knock points away from it (up the yard, +z)
        assert!(hits[0].1.z > 0.5);
        // after a chase hit he goes home
        assert!(matches!(d.state, DazzaState::Return | DazzaState::Cook));
        assert!(ev.iter().any(|e| matches!(
            e,
            DazzaEvent::Say {
                line: Line::Gotcha,
                ..
            }
        )));
    }

    #[test]
    fn the_spatula_misses_if_the_victim_ran_off_or_sat_down() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(7);
        let yard = Yard::default();
        d.state = DazzaState::Chase;
        d.target = Some(9);
        d.state_t = 10.0;
        d.pos = V3::new(0.0, 0.0, 0.0);
        let close = [prey(9, 0.0, 1.5)];
        let ev = d.tick(DT, &close, &yard, false, false, &mut rng);
        assert!(ev.contains(&DazzaEvent::Swing));
        // the victim leaps away before the 0.22 s are up
        let far = [prey(9, 0.0, 6.0)];
        let mut hit = false;
        for _ in 0..30 {
            hit |= d
                .tick(DT, &far, &yard, false, false, &mut rng)
                .iter()
                .any(|e| matches!(e, DazzaEvent::Spatula { .. }));
        }
        assert!(!hit);
    }

    #[test]
    fn someone_down_is_never_swung_at() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(8);
        let yard = Yard::default();
        d.state = DazzaState::Chase;
        d.target = Some(9);
        d.state_t = 10.0;
        d.pos = V3::new(0.0, 0.0, 0.0);
        let mut p = prey(9, 0.0, 1.5);
        p.down_t = 2.0;
        for _ in 0..120 {
            let ev = d.tick(DT, &[p], &yard, false, false, &mut rng);
            assert!(!ev.contains(&DazzaEvent::Swing));
        }
    }

    #[test]
    fn sitting_down_at_smoko_makes_him_give_up() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(9);
        for _ in 0..3 {
            d.meat_stolen(7, false, &mut rng);
        }
        d.pos = V3::new(-6.0, 0.0, -10.0);
        let mut p = prey(7, -20.0, 0.0);
        p.at_smoko = true;
        let ev = d.tick(DT, &[p], &Yard::default(), false, false, &mut rng);
        assert_eq!(d.state, DazzaState::Return);
        assert!(ev.iter().any(|e| matches!(
            e,
            DazzaEvent::Say {
                line: Line::Smoko,
                ..
            }
        )));
        assert_eq!(d.target, None);
    }

    #[test]
    fn he_gives_up_when_the_chase_time_runs_out_or_the_target_leaves() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(10);
        for _ in 0..3 {
            d.meat_stolen(7, false, &mut rng);
        }
        // the target left the yard: not in the prey list
        d.pos = V3::new(-6.0, 0.0, -10.0);
        let ev = d.tick(DT, &[], &Yard::default(), false, false, &mut rng);
        assert_eq!(d.state, DazzaState::Return);
        assert!(ev.iter().any(|e| matches!(
            e,
            DazzaEvent::Say {
                line: Line::GiveUp,
                ..
            }
        )));
    }

    #[test]
    fn a_steak_slap_stuns_then_he_chases_for_the_longer_time() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(11);
        d.level = 2;
        let s = d.slapped(5, true, false, &mut rng);
        match s {
            Slap::Stun { seconds } => assert!((seconds - 0.84).abs() < 1e-5),
            other => panic!("{other:?}"),
        }
        assert_eq!(d.state, DazzaState::Stunned);
        assert_eq!(d.level, 3);
        assert!(!d.slappable());
        let p = [prey(5, -6.0, -8.0)];
        // still stunned just before the time is up
        run(&mut d, 0.8, &p, &mut rng);
        assert_eq!(d.state, DazzaState::Stunned);
        run(&mut d, 0.1, &p, &mut rng);
        assert_eq!(d.state, DazzaState::Chase);
        // the chase length was fixed from the level BEFORE the stun bumped it: 8 + 2*2
        assert!(d.state_t > 11.0 && d.state_t <= 12.0, "{}", d.state_t);
        assert!(!d.berserk, "level 3 is under 4");
    }

    #[test]
    fn he_comes_out_of_a_stun_berserk_at_level_four_or_more() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(12);
        d.level = 4;
        d.slapped(5, true, false, &mut rng);
        run(&mut d, 1.0, &[prey(5, -6.0, -8.0)], &mut rng);
        assert_eq!(d.state, DazzaState::Chase);
        assert!(d.berserk);
    }

    #[test]
    fn a_stunned_dazza_whose_attacker_left_just_goes_home() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(13);
        d.pos = V3::new(-6.0, 0.0, -10.0);
        d.slapped(5, true, false, &mut rng);
        run(&mut d, 1.2, &[], &mut rng);
        assert_eq!(d.state, DazzaState::Return);
    }

    #[test]
    fn dildo_outcomes_are_ko_berserk_or_flip_equally() {
        let mut rng = Rng::new(14);
        let (mut ko, mut bz, mut fl) = (0, 0, 0);
        for _ in 0..6000 {
            let mut d = Dazza::new();
            match d.slapped(5, false, false, &mut rng) {
                Slap::Ko { bum_out } => {
                    ko += 1;
                    assert!(!bum_out, "not in Cheeky mode");
                    assert_eq!(d.state, DazzaState::Ko);
                    assert_eq!(d.target, None);
                }
                Slap::Berserk => {
                    bz += 1;
                    assert_eq!(d.state, DazzaState::Chase);
                    assert!(d.berserk);
                    assert_eq!(d.target, Some(5));
                    assert_eq!(d.level, 1);
                }
                Slap::Flip => {
                    fl += 1;
                    assert_eq!(d.state, DazzaState::Angry);
                    assert_eq!(d.level, 1);
                }
                Slap::Stun { .. } => panic!("a dildo never just stuns"),
            }
        }
        for n in [ko, bz, fl] {
            assert!((n as f32 / 6000.0 - 1.0 / 3.0).abs() < 0.03, "{ko} {bz} {fl}");
        }
    }

    #[test]
    fn a_berserk_dazza_has_a_head_start_of_1_2_seconds() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(15);
        d.state = DazzaState::Chase;
        d.target = Some(5);
        d.state_t = 10.0;
        d.pos = V3::new(0.0, 0.0, 0.0);
        d.berserk = true;
        d.swing_cd = BERSERK_HEAD_START;
        let yard = Yard::default();
        let close = [prey(5, 0.0, 1.5)];
        let mut first_swing = None;
        for i in 0..120 {
            if d.tick(DT, &close, &yard, false, false, &mut rng)
                .contains(&DazzaEvent::Swing)
            {
                first_swing = Some(i as f32 * DT);
                break;
            }
        }
        let t = first_swing.expect("he should swing");
        assert!((t - 1.2).abs() < 0.05, "swung at {t}");
    }

    #[test]
    fn a_ko_lasts_8_seconds_then_he_wakes_and_walks_home() {
        let mut d;
        let mut rng = Rng::new(16);
        // force a KO
        loop {
            d = Dazza::new();
            if matches!(d.slapped(5, false, false, &mut rng), Slap::Ko { .. }) {
                break;
            }
        }
        d.pos = V3::new(-6.0, 0.0, -10.0);
        d.meat_stolen(5, false, &mut rng); // ignored while KO'd
        assert_eq!(d.state, DazzaState::Ko);
        assert_eq!(d.level, 0);
        let ev = run(&mut d, 7.9, &[], &mut rng);
        assert!(ev.is_empty());
        assert_eq!(d.state, DazzaState::Ko);
        let ev = run(&mut d, 0.2, &[], &mut rng);
        assert!(ev.iter().any(|e| matches!(
            e,
            DazzaEvent::Say {
                line: Line::Wake,
                ..
            }
        )));
        assert_eq!(d.state, DazzaState::Return);
    }

    #[test]
    fn bum_out_gnomes_only_in_cheeky_mode_and_at_most_three() {
        let mut rng = Rng::new(17);
        let mut d = Dazza::new();
        let mut gnomes = 0;
        for _ in 0..400 {
            d.state = DazzaState::Cook;
            if let Slap::Ko { bum_out: true } = d.slapped(5, false, true, &mut rng) {
                gnomes += 1;
            }
        }
        assert_eq!(gnomes, 3);
        assert_eq!(d.bum_out_left, 0);
        d.reset();
        assert_eq!(d.bum_out_left, 3);
    }

    #[test]
    fn slap_rules_on_the_host() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(18);
        let near = V3::new(-6.0, 0.0, -16.0);
        // too far
        assert!(d
            .try_slap(0.0, 5, V3::new(-6.0, 0.0, -10.0), false, true, false, &mut rng)
            .is_none());
        // sitting at smoko
        assert!(d.try_slap(0.0, 5, near, true, true, false, &mut rng).is_none());
        // fine
        assert!(d.try_slap(1.0, 5, near, false, true, false, &mut rng).is_some());
        // he's stunned now, so a second slap is refused
        assert!(d.try_slap(1.2, 5, near, false, true, false, &mut rng).is_none());
    }

    #[test]
    fn the_flip_slap_points_index() {
        assert_eq!(Slap::Ko { bum_out: false }.points_index(), Some(0));
        assert_eq!(Slap::Berserk.points_index(), Some(1));
        assert_eq!(Slap::Flip.points_index(), Some(2));
        assert_eq!(Slap::Stun { seconds: 1.0 }.points_index(), None);
    }

    #[test]
    fn walking_home_from_the_front_goes_round_the_open_right_side() {
        let mut d = Dazza::new();
        d.pos = V3::new(-6.0, 0.0, -17.5);
        assert_eq!(d.home_route(), V3::new(-4.3, 0.0, -17.1));
        d.pos = V3::new(-6.0, 0.0, -10.0);
        assert_eq!(d.home_route(), V3::new(-4.3, 0.0, -10.0));
        d.pos = V3::new(0.0, 0.0, -10.0);
        assert_eq!(d.home_route(), V3::new(-4.3, 0.0, -19.3));
        d.pos = V3::new(-6.0, 0.0, -19.0);
        assert_eq!(d.home_route(), HOME);
    }

    #[test]
    fn a_dazza_out_front_walks_all_the_way_home_without_getting_stuck() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(19);
        d.state = DazzaState::Return;
        d.pos = V3::new(-6.0, 0.0, -10.0);
        run(&mut d, 20.0, &[], &mut rng);
        assert_eq!(d.state, DazzaState::Cook, "stuck at {:?}", d.pos);
        assert_eq!(d.pos, HOME);
    }

    #[test]
    fn he_stays_inside_the_fence() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(20);
        d.state = DazzaState::Chase;
        d.target = Some(1);
        d.state_t = 100.0;
        // a target well outside the fence
        let p = [prey(1, 80.0, 80.0)];
        run(&mut d, 30.0, &p, &mut rng);
        assert!(d.pos.x <= YARD_HALF_X - FENCE_MARGIN + 1e-3);
        assert!(d.pos.z <= YARD_HALF_Z - FENCE_MARGIN + 1e-3);
    }

    #[test]
    fn he_never_walks_through_the_grill_or_other_furniture() {
        let mut d = Dazza::new();
        let mut rng = Rng::new(21);
        let yard = Yard::default();
        d.state = DazzaState::Chase;
        d.target = Some(1);
        d.state_t = 100.0;
        // run him back and forth across the yard
        for k in 0..6000 {
            let tz = if (k / 600) % 2 == 0 { 20.0 } else { -20.0 };
            let p = [prey(1, 25.0 * ((k / 300) % 2) as f32 - 12.0, tz)];
            d.tick(DT, &p, &yard, false, false, &mut rng);
            for c in &yard.colliders {
                if c.h < 0.3 {
                    continue;
                }
                let cx = d.pos.x.clamp(c.x0, c.x1);
                let cz = d.pos.z.clamp(c.z0, c.z1);
                let dist = (d.pos.x - cx).hypot(d.pos.z - cz);
                assert!(dist > RADIUS - 0.05, "inside {:?} at {:?}", c.kind, d.pos);
            }
        }
    }

    #[test]
    fn cheeky_mode_adds_rude_lines() {
        let mut rng = Rng::new(22);
        let mut rude = 0;
        for _ in 0..2000 {
            let t = Line::Chat.pick(true, &mut rng);
            if CHAT_CHEEKY.contains(&t) {
                rude += 1;
            }
        }
        assert!(rude > 500 && rude < 1000, "{rude}");
        for _ in 0..500 {
            assert!(!CHAT_CHEEKY.contains(&Line::Chat.pick(false, &mut rng)));
        }
        // lines with no rude version never invent one
        assert!(KO.contains(&Line::Ko.pick(true, &mut rng)));
    }
}
