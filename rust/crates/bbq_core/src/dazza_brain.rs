//! Dazza's behaviour (spec section 12): he cooks, gets angry when you nick his meat, chases
//! you with the spatula, and can be slapped back. `dazza.rs` has how he looks; this is what
//! he does. Pure logic: the game hands in where everyone is and gets back what happened.

use std::collections::BTreeMap;

use crate::dazza::{ANGER_MAX, DazzaState, HOME};
use crate::pose::ang_diff;
use crate::rng::Rng;
use crate::vec::V3;
use crate::yard::Collider;
use crate::{PlayerId, YARD_HALF_X, YARD_HALF_Z};

/// How close the spatula must be to start a swing, and how far it still reaches when it lands.
pub const SWING_RANGE: f32 = 2.1;
pub const HIT_RANGE: f32 = 2.7;
pub const SWING_COOLDOWN: f32 = 1.3;
/// The spatula lands this long after the swing starts.
pub const SWING_DELAY: f32 = 0.22;
/// A spatula hit knocks you back by this much; dizzy for 1.6 s (2.8 s when berserk).
pub const SPATULA_KNOCK: f32 = 7.0;
pub const SPATULA_STUN: f32 = 1.6;
pub const SPATULA_STUN_BERSERK: f32 = 2.8;
/// Dazza keeps this far from the people he chases.
pub const STOP_DISTANCE: f32 = 1.2;
/// His body is this wide against furniture.
pub const RADIUS: f32 = 0.45;
/// A grudge counts down this fast (per second); at 2.5 or more he chases.
pub const GRUDGE_DECAY: f32 = 1.0 / 25.0;
pub const GRUDGE_CHASE: f32 = 2.5;
/// Knocked out for this long.
pub const KO_TIME: f32 = 8.0;
/// Angry for this long when someone takes meat.
pub const ANGRY_TIME: f32 = 3.5;
/// Gnomes that can pop out of a KO'd Dazza in Cheeky mode, per round.
pub const BUM_OUTS: u32 = 3;

/// How long a chase lasts at his current anger level. Longer than the browser game's 8 + 2 per
/// level (8 to 26 s): he now runs after you for 12 + 3 per level (12 to 36 s).
pub fn chase_dur(level: u32) -> f32 {
    (12.0 + level as f32 * 3.0).clamp(12.0, 36.0)
}

/// How fast he can turn round, in radians per second (a full about-turn in about a quarter of a
/// second, not in a single frame).
pub const TURN_RATE: f32 = 13.0;

/// Spatula hits in one chase before he calls it even and walks home.
pub const CHASE_HITS: u32 = 3;
/// After a hit he gloats for this long before swinging again.
pub const GLOAT: f32 = 1.8;
/// At least this much chase is left after a hit, so he keeps coming.
pub const KEEP_CHASING: f32 = 6.0;

/// How long a meat slap stuns him: shorter each time he's been wound up.
pub fn stun_dur(level: u32) -> f32 {
    (1.0 - level as f32 * 0.08).max(0.4)
}

/// Where he walks to when he goes home: behind the grill, round the open right-hand side
/// if he is out the front.
pub fn home_route(pos: V3) -> V3 {
    if pos.z < -18.75 {
        HOME
    } else if pos.x > -4.6 {
        V3::new(-4.3, 0.0, -19.3)
    } else {
        V3::new(-4.3, 0.0, pos.z.max(-17.1))
    }
}

/// Someone Dazza can see.
#[derive(Clone, Copy, Debug)]
pub struct Person {
    pub id: PlayerId,
    pub pos: V3,
    pub down_t: f32,
    pub at_smoko: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// A line for the speech bubble.
    Say(&'static str),
    /// The spatula swing starts (play the animation).
    Swing,
    /// The spatula lands.
    SpatulaHit {
        victim: PlayerId,
        dir: V3,
        berserk: bool,
    },
}

/// What a slap on Dazza did.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SlapResult {
    /// He's knocked out or stunned already, so nothing happens.
    Ignored,
    /// Steak, fish or noodle: a short stun, then he chases.
    Stunned { secs: f32 },
    /// Dildo, outcome 1: out cold for 8 s. `bum_out` means a gnome pops out (Cheeky mode).
    KnockedOut { bum_out: bool },
    /// Dildo, outcome 2: he chases at once, faster.
    Berserk,
    /// Dildo, outcome 3: he flips the barbie, and meat flies off the grill.
    Flipped,
}

impl SlapResult {
    /// 0 KO, 1 berserk, 2 flip (the index scoring uses), or `None` for the other results.
    pub fn outcome_index(self) -> Option<usize> {
        match self {
            SlapResult::KnockedOut { .. } => Some(0),
            SlapResult::Berserk => Some(1),
            SlapResult::Flipped => Some(2),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Brain {
    pub pos: V3,
    pub face: f32,
    pub state: DazzaState,
    pub state_t: f32,
    pub target: Option<PlayerId>,
    pub swing_cd: f32,
    /// A swing that has started and is about to land: (who, seconds left).
    pub pending: Option<(PlayerId, f32)>,
    pub grudges: BTreeMap<PlayerId, f32>,
    pub level: u32,
    /// How long the chase lasts after a stun wears off.
    pub stun_after: f32,
    pub berserk: bool,
    /// Walk cycle phase (feeds the animation).
    pub walk: f32,
    pub chat_t: f32,
    pub bum_outs_left: u32,
    /// Distance walked this tick (feeds the animation).
    pub moved: f32,
    /// So he does not shout a new line for every steak: seconds until he may speak again.
    pub say_cd: f32,
    /// Spatula hits landed in this chase.
    pub chase_hits: u32,
    /// Angry and walking out to the thief (with some slack so he does not dither at the edge).
    pub approaching: bool,
}

impl Default for Brain {
    fn default() -> Self {
        Brain {
            pos: HOME,
            face: 0.0,
            state: DazzaState::Cook,
            state_t: 0.0,
            target: None,
            swing_cd: 0.0,
            pending: None,
            grudges: BTreeMap::new(),
            level: 0,
            stun_after: 0.0,
            berserk: false,
            walk: 0.0,
            chat_t: 14.0,
            bum_outs_left: BUM_OUTS,
            moved: 0.0,
            say_cd: 0.0,
            chase_hits: 0,
            approaching: false,
        }
    }
}

impl Brain {
    /// Start of a round.
    pub fn reset(&mut self) {
        self.state = DazzaState::Return;
        self.target = None;
        self.grudges.clear();
        self.pending = None;
        self.level = 0;
        self.stun_after = 0.0;
        self.bum_outs_left = BUM_OUTS;
        self.say_cd = 0.0;
        self.chase_hits = 0;
        self.approaching = false;
    }

    fn bump_level(&mut self) {
        self.level = (self.level + 1).min(ANGER_MAX);
    }

    /// Someone took meat off the table. (Not while he's knocked out or stunned.)
    pub fn meat_taken(&mut self, id: PlayerId, adult: bool, rng: &mut Rng) -> Option<Event> {
        if matches!(self.state, DazzaState::Ko | DazzaState::Stunned) {
            return None;
        }
        let n = self.grudges.get(&id).copied().unwrap_or(0.0) + 1.0;
        self.grudges.insert(id, n);
        self.target = Some(id);
        self.bump_level();
        // grabbing more food while he is already on your tail keeps him coming
        if self.state == DazzaState::Chase {
            self.state_t = self.state_t.max(KEEP_CHASING);
        }
        if n >= GRUDGE_CHASE {
            if self.state != DazzaState::Chase {
                self.state = DazzaState::Chase;
                self.state_t = chase_dur(self.level);
                self.chase_hits = 0;
            }
            self.speak(Kind::Rage, adult, rng)
        } else if self.state != DazzaState::Chase {
            self.state = DazzaState::Angry;
            self.state_t = ANGRY_TIME;
            self.speak(Kind::Angry, adult, rng)
        } else {
            None
        }
    }

    /// A line, unless he has just said one (grabbing three steaks in a row used to restart the
    /// speech bubble each time).
    fn speak(&mut self, kind: Kind, adult: bool, rng: &mut Rng) -> Option<Event> {
        if self.say_cd > 0.0 {
            return None;
        }
        self.say_cd = 2.5;
        Some(Event::Say(line(kind, adult, rng)))
    }

    /// Someone slapped him. `stun_item` is true for steak, fish and noodle, false for the
    /// dildo. Returns what happened and the line he says.
    pub fn slapped(
        &mut self,
        attacker: PlayerId,
        stun_item: bool,
        adult: bool,
        rng: &mut Rng,
    ) -> (SlapResult, Option<&'static str>) {
        if matches!(self.state, DazzaState::Ko | DazzaState::Stunned) {
            return (SlapResult::Ignored, None);
        }
        self.pending = None;
        if stun_item {
            let secs = stun_dur(self.level);
            self.state = DazzaState::Stunned;
            self.state_t = secs;
            self.target = Some(attacker);
            self.stun_after = chase_dur(self.level);
            self.bump_level();
            return (
                SlapResult::Stunned { secs },
                Some(line(Kind::Stunned, adult, rng)),
            );
        }
        let r = rng.f32();
        if r < 1.0 / 3.0 {
            self.state = DazzaState::Ko;
            self.state_t = KO_TIME;
            self.target = None;
            self.grudges.clear();
            let bum_out = adult && self.bum_outs_left > 0 && rng.chance(0.4);
            if bum_out {
                self.bum_outs_left -= 1;
            }
            (
                SlapResult::KnockedOut { bum_out },
                Some(line(Kind::Ko, adult, rng)),
            )
        } else if r < 2.0 / 3.0 {
            self.state = DazzaState::Chase;
            self.target = Some(attacker);
            self.state_t = chase_dur(self.level);
            self.chase_hits = 0;
            self.berserk = true;
            self.swing_cd = 1.2; // a head start to leg it
            self.bump_level();
            (SlapResult::Berserk, Some(line(Kind::Berserk, adult, rng)))
        } else {
            self.state = DazzaState::Angry;
            self.target = Some(attacker);
            self.state_t = ANGRY_TIME;
            self.bump_level();
            (SlapResult::Flipped, Some(line(Kind::Flip, adult, rng)))
        }
    }

    /// Advance one step. `people` is everyone he can see; `colliders` the yard furniture.
    /// `countdown` stops him chatting between rounds.
    pub fn tick(
        &mut self,
        dt: f32,
        people: &[Person],
        colliders: &[Collider],
        countdown: bool,
        adult: bool,
        rng: &mut Rng,
    ) -> Vec<Event> {
        let mut out = Vec::new();
        self.moved = 0.0;
        self.say_cd = (self.say_cd - dt).max(0.0);

        if self.state == DazzaState::Ko {
            self.state_t -= dt;
            self.swing_cd = 1.0;
            if self.state_t <= 0.0 {
                self.state = DazzaState::Return;
                out.push(Event::Say(line(Kind::Wake, adult, rng)));
            }
            return out;
        }
        if self.state == DazzaState::Stunned {
            self.state_t -= dt;
            self.swing_cd = 1.0;
            if self.state_t <= 0.0 {
                let tgt = self.target.and_then(|t| people.iter().find(|p| p.id == t));
                if tgt.is_some() {
                    self.state = DazzaState::Chase;
                    self.state_t = if self.stun_after > 0.0 {
                        self.stun_after
                    } else {
                        chase_dur(self.level)
                    };
                    self.berserk = self.level >= 4;
                    self.chase_hits = 0;
                    out.push(Event::Say(line(Kind::Rage, adult, rng)));
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

        let mut tgt: Option<Person> = self
            .target
            .and_then(|t| people.iter().find(|p| p.id == t).copied());
        if let Some(t) = tgt
            && t.at_smoko
            && matches!(self.state, DazzaState::Chase | DazzaState::Angry)
        {
            out.push(Event::Say(line(Kind::Smoko, adult, rng)));
            self.state = DazzaState::Return;
            self.target = None;
            self.pending = None;
            tgt = None;
        }
        if matches!(self.state, DazzaState::Angry | DazzaState::Chase) {
            self.state_t -= dt;
            if self.state_t <= 0.0 || tgt.is_none() {
                if self.state == DazzaState::Chase {
                    out.push(Event::Say(line(Kind::GiveUp, adult, rng)));
                }
                self.state = DazzaState::Return;
                self.target = None;
                tgt = None;
            }
        }

        // where to, and how fast
        let mut goal: Option<V3> = None;
        let mut speed = 0.0;
        let mut stop = 0.0;
        match (self.state, tgt) {
            (DazzaState::Chase, Some(t)) => {
                goal = Some(t.pos);
                speed = (if self.berserk { 8.6 } else { 7.2 }) + (self.level as f32 * 0.3).min(2.2);
                stop = STOP_DISTANCE;
            }
            (DazzaState::Angry, Some(t)) => {
                // a little slack once he has set off, so he does not dither at the edge
                let reach = if self.approaching { 6.5 } else { 5.0 };
                self.approaching = t.pos.horiz_dist(HOME) < reach;
                if self.approaching {
                    goal = Some(t.pos);
                    stop = STOP_DISTANCE;
                } else {
                    goal = Some(home_route(self.pos));
                }
                speed = 3.4;
            }
            (DazzaState::Return, _) => {
                goal = Some(home_route(self.pos));
                speed = 4.0;
                if self.pos.horiz_dist(HOME) < 0.25 {
                    self.state = DazzaState::Cook;
                    self.pos.x = HOME.x;
                    self.pos.z = HOME.z;
                }
            }
            _ => {
                self.chat_t -= dt;
                if self.chat_t <= 0.0 {
                    self.chat_t = rng.range(25.0, 35.0);
                    if !countdown {
                        out.push(Event::Say(line(Kind::Chat, adult, rng)));
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
                self.walk += st * 2.4;
                self.moved = st;
            }
            if d > 0.05 {
                // turn round at a sensible rate instead of snapping
                let want = ang_diff(dx.atan2(dz), self.face);
                let step = TURN_RATE * dt;
                self.face += want.clamp(-step, step);
            }
        } else {
            let k = 1.0 - (-4.0 * dt).exp();
            self.face += ang_diff(0.0, self.face) * k;
        }

        // keep him out of the furniture and inside the fence
        for c in colliders {
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
        self.pos.x = self.pos.x.clamp(-YARD_HALF_X + 0.5, YARD_HALF_X - 0.5);
        self.pos.z = self.pos.z.clamp(-YARD_HALF_Z + 0.5, YARD_HALF_Z - 0.5);

        // the spatula
        if let Some(t) = tgt
            && matches!(self.state, DazzaState::Angry | DazzaState::Chase)
            && self.swing_cd <= 0.0
            && self.pending.is_none()
            && t.down_t <= 0.0
            && t.pos.horiz_dist(self.pos) < SWING_RANGE
        {
            self.swing_cd = SWING_COOLDOWN;
            out.push(Event::Swing);
            self.pending = Some((t.id, SWING_DELAY));
        }
        if let Some((id, t)) = &mut self.pending {
            *t -= dt;
            if *t <= 0.0 {
                let id = *id;
                self.pending = None;
                if let Some(v) = people.iter().find(|p| p.id == id)
                    && !v.at_smoko
                    && v.pos.horiz_dist(self.pos) < HIT_RANGE
                {
                    let (dx, dz) = (v.pos.x - self.pos.x, v.pos.z - self.pos.z);
                    let l = dx.hypot(dz);
                    let dir = if l < 1e-2 {
                        V3::new(0.0, 0.0, 1.0)
                    } else {
                        V3::new(dx / l, 0.0, dz / l)
                    };
                    out.push(Event::SpatulaHit {
                        victim: id,
                        dir,
                        berserk: self.berserk,
                    });
                    if self.state == DazzaState::Chase {
                        self.grudges.insert(id, 1.0);
                        self.chase_hits += 1;
                        out.push(Event::Say(line(Kind::Gotcha, adult, rng)));
                        if self.chase_hits >= CHASE_HITS {
                            // three whacks and he is satisfied
                            self.state = DazzaState::Return;
                            self.target = None;
                        } else {
                            // gloat for a moment, then keep coming after you
                            self.swing_cd = GLOAT;
                            self.state_t = self.state_t.max(KEEP_CHASING);
                        }
                    }
                }
            }
        }
        out
    }
}

// ---- what he says ----

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
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
const CHAT_ADULT: &[&str] = &[
    "These snags have seen things.",
    "Dunny's blocked again. Don't ask.",
    "Me missus reckons I love this grill more than her. She's not wrong.",
];
const ANGRY_ADULT: &[&str] = &[
    "OI! Keep yer hands off me snags, ya pervert!",
    "Get back 'ere, ya cheeky bugger!",
];
const RAGE_ADULT: &[&str] = &[
    "RIGHT, I'M TELLIN' YA NAN!",
    "THAT'S IT, PANTS DOWN, OVER ME KNEE!",
];

/// Every line he can say for this kind (Cheeky mode adds some rude ones).
pub fn lines(kind: Kind, adult: bool) -> Vec<&'static str> {
    let (base, extra): (&[&str], &[&str]) = match kind {
        Kind::Angry => (ANGRY, ANGRY_ADULT),
        Kind::Rage => (RAGE, RAGE_ADULT),
        Kind::Gotcha => (GOTCHA, &[]),
        Kind::GiveUp => (GIVEUP, &[]),
        Kind::Ko => (KO, &[]),
        Kind::Wake => (WAKE, &[]),
        Kind::Berserk => (BERSERK, &[]),
        Kind::Stunned => (STUNNED, &[]),
        Kind::Flip => (FLIP, &[]),
        Kind::Smoko => (SMOKO, &[]),
        Kind::Chat => (CHAT, CHAT_ADULT),
    };
    let mut v = base.to_vec();
    if adult {
        v.extend_from_slice(extra);
    }
    v
}

fn line(kind: Kind, adult: bool, rng: &mut Rng) -> &'static str {
    let all = lines(kind, adult);
    let i = ((rng.f32() * all.len() as f32) as usize).min(all.len() - 1);
    all[i]
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::yard::Yard;

    const DT: f32 = 1.0 / 60.0;

    fn person(id: u32, x: f32, z: f32) -> Person {
        Person {
            id,
            pos: V3::new(x, 0.0, z),
            down_t: 0.0,
            at_smoko: false,
        }
    }

    fn run(b: &mut Brain, secs: f32, people: &[Person], rng: &mut Rng) -> Vec<Event> {
        let yard = Yard::default();
        let mut all = Vec::new();
        for _ in 0..(secs / DT) as usize {
            all.extend(b.tick(DT, people, &yard.colliders, false, false, rng));
        }
        all
    }

    #[test]
    fn he_cooks_and_chats_every_25_to_35_s_after_the_first_at_14() {
        let mut b = Brain::default();
        let mut rng = Rng::new(1);
        let ev = run(&mut b, 13.5, &[], &mut rng);
        assert!(ev.is_empty());
        let ev = run(&mut b, 1.0, &[], &mut rng);
        assert_eq!(ev.len(), 1);
        assert!((25.0..=35.0).contains(&b.chat_t));
        // no chatter during the countdown
        let yard = Yard::default();
        let mut b = Brain::default();
        b.chat_t = 0.01;
        let ev = b.tick(0.1, &[], &yard.colliders, true, false, &mut rng);
        assert!(ev.is_empty());
    }

    #[test]
    fn taking_meat_once_makes_him_angry_and_three_times_quickly_makes_him_chase() {
        let mut b = Brain::default();
        let mut rng = Rng::new(2);
        assert!(b.meat_taken(7, false, &mut rng).is_some());
        assert_eq!(b.state, DazzaState::Angry);
        assert_eq!(b.state_t, ANGRY_TIME);
        assert_eq!(b.target, Some(7));
        b.meat_taken(7, false, &mut rng);
        assert_eq!(b.state, DazzaState::Angry, "second time: grudge is only 2");
        b.meat_taken(7, false, &mut rng);
        assert_eq!(b.state, DazzaState::Chase);
        assert_eq!(b.level, 3);
        assert_eq!(b.state_t, chase_dur(3));
    }

    #[test]
    fn grudges_fade_so_slow_thieves_only_annoy_him() {
        let mut b = Brain::default();
        let mut rng = Rng::new(2);
        let people = [person(7, 0.0, 0.0)];
        b.meat_taken(7, false, &mut rng);
        b.meat_taken(7, false, &mut rng);
        run(&mut b, 30.0, &people, &mut rng); // 2.0 of grudge fades by 1.2
        assert!(b.grudges.get(&7).copied().unwrap_or(0.0) < 1.0);
    }

    #[test]
    fn anger_level_is_capped_at_8_and_scales_the_chase_and_stun() {
        let mut b = Brain::default();
        let mut rng = Rng::new(3);
        for i in 0..20 {
            b.meat_taken(i, false, &mut rng);
            if b.state == DazzaState::Ko {
                break;
            }
        }
        assert_eq!(b.level, 8);
        // longer than the browser game's 8, 16, 24: he runs after you for 12 to 36 s
        assert_eq!(chase_dur(0), 12.0);
        assert_eq!(chase_dur(4), 24.0);
        assert_eq!(chase_dur(8), 36.0);
        assert!((stun_dur(0) - 1.0).abs() < 1e-5);
        assert!((stun_dur(4) - 0.68).abs() < 1e-5);
        assert!((stun_dur(8) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn a_chase_runs_at_the_right_speed_and_stops_1_2_m_away() {
        let mut b = Brain::default();
        let mut rng = Rng::new(4);
        let me = [person(1, -6.0, 5.0)];
        b.pos = V3::new(-6.0, 0.0, -16.0);
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 8.0;
        run(&mut b, 0.5, &me, &mut rng);
        let moved = b.pos.z - -16.0;
        assert!((moved - 7.2 * 0.5).abs() < 0.2, "moved {moved}");
        let me = [person(1, -6.0, -6.0)];
        run(&mut b, 1.0, &me, &mut rng);
        let gap = (me[0].pos.z - b.pos.z).abs();
        assert!((gap - STOP_DISTANCE).abs() < 0.1, "gap {gap}");
    }

    #[test]
    fn berserk_and_anger_make_him_faster() {
        let mut b = Brain::default();
        b.level = 4;
        b.berserk = true;
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 10.0;
        b.pos = V3::new(-6.0, 0.0, -16.0);
        let mut rng = Rng::new(1);
        run(&mut b, 0.5, &[person(1, -6.0, 5.0)], &mut rng);
        let moved = b.pos.z + 16.0;
        let want = (8.6 + 1.2) * 0.5;
        assert!((moved - want).abs() < 0.25, "{moved} vs {want}");
    }

    #[test]
    fn he_swings_when_close_and_the_spatula_lands_a_moment_later() {
        let mut b = Brain::default();
        let mut rng = Rng::new(5);
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 8.0;
        b.pos = V3::new(-6.0, 0.0, -10.0);
        let me = [person(1, -6.0, -8.5)]; // 1.5 m away: in range already
        let ev = run(&mut b, 0.5, &me, &mut rng);
        assert!(ev.contains(&Event::Swing));
        let hit = ev.iter().find_map(|e| match e {
            Event::SpatulaHit {
                victim,
                dir,
                berserk,
            } => Some((*victim, *dir, *berserk)),
            _ => None,
        });
        let (v, dir, berserk) = hit.expect("the spatula should have landed");
        assert_eq!(v, 1);
        assert!(dir.z > 0.9);
        assert!(!berserk);
        // after a chase hit he gloats and KEEPS CHASING (he used to go home after one hit)
        assert_eq!(b.state, DazzaState::Chase);
        assert!(b.state_t >= KEEP_CHASING - 0.01);
        assert!(b.swing_cd > 1.0, "he gloats before he swings again");
        assert!(
            ev.iter()
                .any(|e| matches!(e, Event::Say(s) if GOTCHA.contains(s)))
        );
    }

    #[test]
    fn he_misses_if_you_step_away_before_it_lands() {
        let mut b = Brain::default();
        let mut rng = Rng::new(5);
        b.state = DazzaState::Angry;
        b.target = Some(1);
        b.state_t = 3.5;
        b.pos = V3::new(-6.0, 0.0, -12.0);
        let yard = Yard::default();
        let near = [person(1, -6.0, -10.5)];
        let ev = b.tick(DT, &near, &yard.colliders, false, false, &mut rng);
        assert!(ev.contains(&Event::Swing));
        let far = [person(1, -6.0, 0.0)];
        let mut hit = false;
        for _ in 0..30 {
            for e in b.tick(DT, &far, &yard.colliders, false, false, &mut rng) {
                if matches!(e, Event::SpatulaHit { .. }) {
                    hit = true;
                }
            }
        }
        assert!(!hit);
    }

    #[test]
    fn he_never_swings_at_someone_already_down_or_on_smoko() {
        let mut rng = Rng::new(6);
        let yard = Yard::default();
        for (down, smoko) in [(2.0, false), (0.0, true)] {
            let mut b = Brain::default();
            b.state = DazzaState::Chase;
            b.target = Some(1);
            b.state_t = 8.0;
            b.pos = V3::new(-6.0, 0.0, -10.0);
            let mut p = person(1, -6.0, -9.0);
            p.down_t = down;
            p.at_smoko = smoko;
            let ev = b.tick(DT, &[p], &yard.colliders, false, false, &mut rng);
            assert!(!ev.contains(&Event::Swing));
        }
    }

    #[test]
    fn he_gives_up_when_you_sit_down_for_smoko() {
        let mut b = Brain::default();
        let mut rng = Rng::new(7);
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 8.0;
        b.pos = V3::new(-6.0, 0.0, -8.0);
        let mut p = person(1, 5.0, 5.0);
        p.at_smoko = true;
        let ev = run(&mut b, 0.1, &[p], &mut rng);
        assert!(
            ev.iter()
                .any(|e| matches!(e, Event::Say(s) if SMOKO.contains(s)))
        );
        assert_eq!(b.state, DazzaState::Return);
        assert_eq!(b.target, None);
    }

    #[test]
    fn he_gives_up_when_the_time_runs_out_or_you_leave() {
        let mut rng = Rng::new(8);
        let mut b = Brain::default();
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 0.5;
        b.pos = V3::new(-6.0, 0.0, -10.0);
        let ev = run(&mut b, 0.7, &[person(1, 20.0, 5.0)], &mut rng);
        assert!(
            ev.iter()
                .any(|e| matches!(e, Event::Say(s) if GIVEUP.contains(s)))
        );
        let mut b = Brain::default();
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 8.0;
        b.pos = V3::new(-6.0, 0.0, -10.0);
        run(&mut b, 0.1, &[], &mut rng);
        assert_eq!(b.state, DazzaState::Return);
    }

    #[test]
    fn going_home_he_walks_round_the_open_side_and_goes_back_to_cooking() {
        let mut b = Brain::default();
        let mut rng = Rng::new(9);
        b.state = DazzaState::Return;
        b.pos = V3::new(2.0, 0.0, -10.0); // out the front of the grill
        run(&mut b, 12.0, &[], &mut rng);
        assert_eq!(b.state, DazzaState::Cook);
        assert!(b.pos.horiz_dist(HOME) < 0.5);
    }

    #[test]
    fn home_route_goes_round_the_right_side_from_the_front() {
        assert_eq!(home_route(V3::new(0.0, 0.0, -19.0)), HOME);
        assert_eq!(
            home_route(V3::new(0.0, 0.0, -10.0)),
            V3::new(-4.3, 0.0, -19.3)
        );
        assert_eq!(
            home_route(V3::new(-8.0, 0.0, -10.0)),
            V3::new(-4.3, 0.0, -10.0)
        );
        assert_eq!(
            home_route(V3::new(-8.0, 0.0, -18.0)),
            V3::new(-4.3, 0.0, -17.1)
        );
    }

    #[test]
    fn steak_slap_stuns_him_then_he_chases() {
        let mut b = Brain::default();
        let mut rng = Rng::new(10);
        let (r, say) = b.slapped(1, true, false, &mut rng);
        assert_eq!(r, SlapResult::Stunned { secs: 1.0 });
        assert!(say.is_some());
        assert_eq!(b.state, DazzaState::Stunned);
        assert_eq!(b.level, 1);
        assert_eq!(b.stun_after, chase_dur(0));
        let ev = run(&mut b, 1.1, &[person(1, -6.0, -5.0)], &mut rng);
        assert!(
            ev.iter()
                .any(|e| matches!(e, Event::Say(s) if RAGE.contains(s)))
        );
        assert_eq!(b.state, DazzaState::Chase);
        assert!((b.state_t - chase_dur(0)).abs() < 0.2);
    }

    #[test]
    fn stunned_and_ko_dazza_cannot_be_slapped_again() {
        let mut b = Brain::default();
        let mut rng = Rng::new(10);
        b.slapped(1, true, false, &mut rng);
        assert_eq!(b.slapped(1, true, false, &mut rng).0, SlapResult::Ignored);
        b.state = DazzaState::Ko;
        assert_eq!(b.slapped(1, false, false, &mut rng).0, SlapResult::Ignored);
    }

    #[test]
    fn dildo_slaps_give_ko_berserk_or_flip_about_equally() {
        let (mut ko, mut bz, mut fl) = (0, 0, 0);
        for seed in 0..900 {
            let mut b = Brain::default();
            let mut rng = Rng::new(seed);
            match b.slapped(1, false, false, &mut rng).0 {
                SlapResult::KnockedOut { .. } => {
                    ko += 1;
                    assert_eq!(b.state, DazzaState::Ko);
                    assert_eq!(b.state_t, 8.0);
                    assert!(b.grudges.is_empty() && b.target.is_none());
                }
                SlapResult::Berserk => {
                    bz += 1;
                    assert_eq!(b.state, DazzaState::Chase);
                    assert!(b.berserk && (b.swing_cd - 1.2).abs() < 1e-5);
                    assert_eq!(b.target, Some(1));
                }
                SlapResult::Flipped => {
                    fl += 1;
                    assert_eq!(b.state, DazzaState::Angry);
                    assert_eq!(b.state_t, 3.5);
                }
                r => panic!("{r:?}"),
            }
        }
        for c in [ko, bz, fl] {
            assert!((c as f32 / 900.0 - 1.0 / 3.0).abs() < 0.06, "{c}");
        }
    }

    #[test]
    fn berserk_head_start_means_no_swing_for_1_2_s() {
        let mut b = Brain::default();
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 10.0;
        b.berserk = true;
        b.swing_cd = 1.2;
        b.pos = V3::new(-6.0, 0.0, -10.0);
        let mut rng = Rng::new(1);
        let me = [person(1, -6.0, -8.9)];
        let ev = run(&mut b, 1.0, &me, &mut rng);
        assert!(!ev.contains(&Event::Swing), "still in his head start");
        let ev = run(&mut b, 0.5, &me, &mut rng);
        assert!(ev.contains(&Event::Swing));
    }

    #[test]
    fn a_ko_lasts_8_s_then_he_wakes_and_goes_home() {
        let mut b = Brain::default();
        b.state = DazzaState::Ko;
        b.state_t = 8.0;
        b.pos = V3::new(-3.0, 0.0, -15.0);
        let mut rng = Rng::new(1);
        let ev = run(&mut b, 7.5, &[], &mut rng);
        assert!(ev.is_empty() && b.state == DazzaState::Ko);
        let ev = run(&mut b, 1.0, &[], &mut rng);
        assert!(
            ev.iter()
                .any(|e| matches!(e, Event::Say(s) if WAKE.contains(s)))
        );
        assert_eq!(b.state, DazzaState::Return);
    }

    #[test]
    fn bum_out_gnomes_only_in_cheeky_mode_and_at_most_three_a_round() {
        let mut total = 0;
        let mut b = Brain::default();
        for seed in 0..200 {
            let mut rng = Rng::new(seed);
            b.state = DazzaState::Cook;
            if let (SlapResult::KnockedOut { bum_out: true }, _) =
                b.slapped(1, false, true, &mut rng)
            {
                total += 1;
            }
        }
        assert_eq!(total, 3);
        let mut b = Brain::default();
        for seed in 0..100 {
            let mut rng = Rng::new(seed);
            b.state = DazzaState::Cook;
            assert!(!matches!(
                b.slapped(1, false, false, &mut rng).0,
                SlapResult::KnockedOut { bum_out: true }
            ));
        }
    }

    #[test]
    fn he_stays_out_of_the_furniture_and_inside_the_fence() {
        let mut b = Brain::default();
        let mut rng = Rng::new(1);
        b.state = DazzaState::Chase;
        b.target = Some(1);
        b.state_t = 60.0;
        // target is on the other side of the whole yard
        let far = [person(1, 60.0, 40.0)];
        let yard = Yard::default();
        for _ in 0..(20.0 / DT) as usize {
            b.tick(DT, &far, &yard.colliders, false, false, &mut rng);
            assert!(b.pos.x.abs() <= YARD_HALF_X - 0.5 + 1e-3);
            assert!(b.pos.z.abs() <= YARD_HALF_Z - 0.5 + 1e-3);
            for c in &yard.colliders {
                if c.h < 0.3 {
                    continue;
                }
                let cx = b.pos.x.clamp(c.x0, c.x1);
                let cz = b.pos.z.clamp(c.z0, c.z1);
                let d = ((b.pos.x - cx).powi(2) + (b.pos.z - cz).powi(2)).sqrt();
                // pushed out one box at a time, so a few centimetres of overlap is normal (as in the browser game)
                assert!(d >= RADIUS - 0.1, "inside a {:?}: {d}", c.kind);
            }
        }
    }

    #[test]
    fn reset_sends_him_home_and_forgets_everything() {
        let mut b = Brain::default();
        let mut rng = Rng::new(1);
        b.meat_taken(3, false, &mut rng);
        b.level = 5;
        b.bum_outs_left = 0;
        b.reset();
        assert_eq!(b.state, DazzaState::Return);
        assert!(b.grudges.is_empty() && b.target.is_none());
        assert_eq!((b.level, b.bum_outs_left), (0, 3));
    }

    #[test]
    fn cheeky_mode_adds_rude_lines() {
        assert!(lines(Kind::Chat, true).len() > lines(Kind::Chat, false).len());
        assert!(lines(Kind::Angry, true).len() > lines(Kind::Angry, false).len());
        assert!(lines(Kind::Rage, true).len() > lines(Kind::Rage, false).len());
        assert_eq!(lines(Kind::Ko, true).len(), lines(Kind::Ko, false).len());
    }

    #[test]
    fn he_never_snaps_round_in_one_frame_even_when_his_goal_flips() {
        let yard = Yard::default();
        let mut rng = Rng::new(5);
        let mut b = Brain::default();
        let me = person(1, -8.7, -17.0);
        let mut last = b.face;
        let mut worst = 0.0f32;
        for tick in 0..(12.0 / DT) as usize {
            if tick % 60 == 30 && (tick as f32 * DT) < 4.0 {
                b.meat_taken(1, false, &mut rng);
            }
            b.tick(DT, &[me], &yard.colliders, false, false, &mut rng);
            worst = worst.max(ang_diff(b.face, last).abs());
            last = b.face;
        }
        // 13 rad/s at 60 Hz is about 0.22 rad a tick (it was up to 3.1)
        assert!(worst <= TURN_RATE * DT + 1e-4, "turned {worst} rad in one tick");
    }

    #[test]
    fn grabbing_steaks_in_a_row_does_not_restart_his_speech_bubble_each_time() {
        let mut b = Brain::default();
        let mut rng = Rng::new(6);
        let mut lines = 0;
        for _ in 0..4 {
            if b.meat_taken(1, false, &mut rng).is_some() {
                lines += 1;
            }
        }
        assert_eq!(lines, 1, "one line, then he holds his tongue for a moment");
        b.tick(3.0, &[], &Yard::default().colliders, false, false, &mut rng);
        assert!(b.meat_taken(1, false, &mut rng).is_some() || b.state == DazzaState::Return);
    }

    #[test]
    fn he_chases_for_a_long_time_and_goes_home_after_three_whacks() {
        let mut b = Brain::default();
        let mut rng = Rng::new(7);
        for _ in 0..3 {
            b.meat_taken(1, false, &mut rng);
        }
        assert_eq!(b.state, DazzaState::Chase);
        assert!(b.state_t >= 12.0);
        // a runner who stays just out of reach for 10 s is still being chased
        let yard = Yard::default();
        let mut me = person(1, -6.0, -5.0);
        for _ in 0..(10.0 / DT) as usize {
            me.pos = V3::new(b.pos.x, 0.0, b.pos.z + 3.2);
            b.tick(DT, &[me], &yard.colliders, false, false, &mut rng);
        }
        assert_eq!(b.state, DazzaState::Chase, "still on your tail after 10 s");
        // let him catch someone standing still: three hits and he is done
        let still = person(1, b.pos.x, b.pos.z + 1.5);
        let mut hits = 0;
        for _ in 0..(14.0 / DT) as usize {
            let me = Person { pos: V3::new(b.pos.x, 0.0, b.pos.z + 1.5), ..still };
            hits += b
                .tick(DT, &[me], &yard.colliders, false, false, &mut rng)
                .iter()
                .filter(|e| matches!(e, Event::SpatulaHit { .. }))
                .count();
            if b.state != DazzaState::Chase {
                break;
            }
        }
        assert_eq!(hits as u32, CHASE_HITS);
        assert_eq!(b.state, DazzaState::Return);
    }
}
