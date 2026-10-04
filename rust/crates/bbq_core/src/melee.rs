//! Slaps up close (spec section 3, "Melee slaps"): who you can hit, how often, and what each
//! kind of slap does to the person hit. Steak, fish and noodle give a dizzy stun; the dildo
//! knocks someone flat in one of three ways; bare hands (Cheeky mode only) give a quick shove.

use crate::PlayerId;
use crate::hitting::knock_mover;
use crate::items::{DildoVariant, ItemKind, MEAT_STUN, Melee};
use crate::movement::Mover;
use crate::pose::SlapKind;
use crate::rng::Rng;
use crate::stun::Body;
use crate::vec::V3;

/// Start a slap on someone within this distance (and in front of you).
pub const REACH: f32 = 2.6;
/// ...and no more than this far above or below you.
pub const VERTICAL: f32 = 1.6;
/// `dot(facing, direction to them)` must be at least this.
pub const CONE: f32 = 0.4;
/// The host checks armed slaps within this distance, bare-handed ones within 2.9.
pub const HOST_REACH: f32 = 3.6;
pub const BARE_HOST_REACH: f32 = 2.9;
/// Wait this long between swings (armed / bare-handed).
pub const COOLDOWN_ARMED: f32 = 0.55;
pub const COOLDOWN_BARE: f32 = 0.9;
/// The host won't allow armed slaps closer together than this.
pub const HOST_COOLDOWN: f32 = 0.45;
/// How long a swing takes.
pub const SWING_TIME: f32 = 0.28;
/// Someone already knocked flat for longer than this can't be slapped down again.
pub const DOWN_GRACE: f32 = 0.3;

/// Bare-handed slap numbers (Cheeky mode).
pub const SILLY_KNOCK: f32 = 8.5;
pub const SILLY_UP: f32 = 3.0;
pub const SILLY_STUN: f32 = 0.6;
pub const SILLY_NAMES: [&str; 3] = ["WET WILLY!", "NOOGIE!", "WEDGIE!"];

/// Someone who might get slapped.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub id: PlayerId,
    pub pos: V3,
    pub down_t: f32,
    pub at_smoko: bool,
    pub teammate: bool,
}

/// The nearest person within reach, in front of you, who can be slapped.
/// `facing` is a flat direction (it doesn't have to be a unit vector).
pub fn pick_target(
    me: V3,
    facing: (f32, f32),
    cands: &[Candidate],
    friendly_fire: bool,
) -> Option<PlayerId> {
    let fl = facing.0.hypot(facing.1);
    let (fx, fz) = if fl > 1e-4 {
        (facing.0 / fl, facing.1 / fl)
    } else {
        (0.0, 1.0)
    };
    let mut best: Option<(PlayerId, f32)> = None;
    for c in cands {
        if c.down_t > DOWN_GRACE || c.at_smoko || (c.teammate && !friendly_fire) {
            continue;
        }
        let (dx, dz) = (c.pos.x - me.x, c.pos.z - me.z);
        let l = dx.hypot(dz);
        if l > REACH || (c.pos.y - me.y).abs() > VERTICAL {
            continue;
        }
        if l > 0.01 && (fx * dx + fz * dz) / l < CONE {
            continue;
        }
        if best.is_none_or(|(_, b)| l < b) {
            best = Some((c.id, l));
        }
    }
    best.map(|b| b.0)
}

/// Is Dazza (or anything else that isn't a player) in reach in front of you?
pub fn in_front(me: V3, facing: (f32, f32), at: V3) -> bool {
    let fl = facing.0.hypot(facing.1);
    let (fx, fz) = if fl > 1e-4 {
        (facing.0 / fl, facing.1 / fl)
    } else {
        (0.0, 1.0)
    };
    let (dx, dz) = (at.x - me.x, at.z - me.z);
    let l = dx.hypot(dz);
    l < REACH && (l < 0.01 || (fx * dx + fz * dz) / l >= CONE)
}

/// Your swing timer.
#[derive(Clone, Copy, Debug)]
pub struct SlapClock {
    last: f32,
}

impl Default for SlapClock {
    fn default() -> Self {
        SlapClock { last: -9.0 }
    }
}

impl SlapClock {
    pub fn ready(&self, now: f32, armed: bool) -> bool {
        now - self.last >= if armed { COOLDOWN_ARMED } else { COOLDOWN_BARE }
    }
    pub fn mark(&mut self, now: f32) {
        self.last = now;
    }
    /// Seconds since the last swing started (for the swing animation).
    pub fn since(&self, now: f32) -> f32 {
        now - self.last
    }
}

/// What a slap does to the person hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlapEffect {
    /// Push, along the line from you to them.
    pub knock: f32,
    /// At least this much upward speed.
    pub up: f32,
    /// Stun length (only if they aren't already stunned or in grace).
    pub stun: f32,
    /// Dizzy time (stars and a wobbly view), always applied.
    pub dizzy: f32,
    /// `Some(seconds)`: knocked flat, playing this pose.
    pub down: Option<(SlapKind, f32)>,
    /// The body spins from the push (stun slaps; the flat ones play a set pose instead).
    pub tumble: bool,
}

/// Steak, fish and noodle: dizzy stun, no knockdown. The noodle slaps for 1.2 s, the others 2 s.
pub fn stun_slap(kind: ItemKind) -> SlapEffect {
    let d = kind.def();
    let st = d.slap_stun.unwrap_or(MEAT_STUN);
    SlapEffect {
        knock: d.knock,
        up: d.knock * 0.38,
        stun: kind.hit_stun().max(st),
        dizzy: st,
        down: None,
        tumble: true,
    }
}

/// The three ways a dildo slap can send someone down, picked at random.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DildoRoll {
    pub pose: SlapKind,
    pub crit: bool,
    pub down: f32,
}

pub fn roll_dildo(rng: &mut Rng, variant: DildoVariant) -> DildoRoll {
    let r = rng.f32();
    let pose = if r < 1.0 / 3.0 {
        SlapKind::SentFlying
    } else if r < 2.0 / 3.0 {
        SlapKind::Cartwheel
    } else {
        SlapKind::Timber
    };
    let crit = rng.chance(crate::items::CRIT_CHANCE);
    DildoRoll {
        pose,
        crit,
        down: variant.knockdown_time(crit),
    }
}

/// What a rolled dildo slap does. A critical pushes 1.9 times as hard.
pub fn dildo_slap(roll: &DildoRoll) -> SlapEffect {
    let d = ItemKind::Dildo.def();
    let (knock, up) = match roll.pose {
        SlapKind::SentFlying => (13.0, 5.5),
        SlapKind::Cartwheel => (d.knock, 2.5),
        SlapKind::Timber => (2.5, 2.5),
    };
    let mul = if roll.crit { 1.9 } else { 1.0 };
    SlapEffect {
        knock: knock * mul,
        up,
        stun: ItemKind::Dildo.hit_stun(),
        dizzy: 0.0,
        down: Some((roll.pose, roll.down)),
        tumble: false,
    }
}

/// Wet willy, noogie or wedgie.
pub fn silly_slap() -> SlapEffect {
    SlapEffect {
        knock: SILLY_KNOCK,
        up: SILLY_UP,
        stun: SILLY_STUN,
        dizzy: 0.0,
        down: None,
        tumble: false,
    }
}

/// What really happened to them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applied {
    pub stunned: bool,
    pub knocked_down: bool,
}

/// Push and stun (or flatten) someone. Stuns never stack, and a knockdown needs them not
/// to be down already. `dir` is the flat direction from you to them.
pub fn apply(body: &mut Body, mover: &mut Mover, dir: V3, fx: &SlapEffect) -> Applied {
    knock_mover(mover, dir, fx.knock, fx.up);
    let hit = body.apply_hit(fx.stun, fx.down.map(|d| d.1));
    Applied {
        stunned: hit.stunned,
        knocked_down: hit.knocked_down,
    }
}

/// Fish slaps can leave a pong: 10% a rancid cloud for 9 s, otherwise a 33% chance of a mild
/// one for 5 s.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Smell {
    None,
    Mild,
    Rancid,
}

impl Smell {
    pub fn seconds(self) -> f32 {
        match self {
            Smell::None => 0.0,
            Smell::Mild => 5.0,
            Smell::Rancid => 9.0,
        }
    }
}

pub fn fish_smell(rng: &mut Rng) -> Smell {
    if rng.chance(0.1) {
        Smell::Rancid
    } else if rng.chance(0.33) {
        Smell::Mild
    } else {
        Smell::None
    }
}

/// Can this kind of item slap at all, and how?
pub fn kind_of(item: ItemKind) -> Melee {
    item.def().melee
}

/// A flat unit direction from `a` to `b` (straight ahead if they're on top of each other).
pub fn direction(a: V3, b: V3) -> V3 {
    let (dx, dz) = (b.x - a.x, b.z - a.z);
    let l = dx.hypot(dz);
    if l < 1e-2 {
        V3::new(0.0, 0.0, 1.0)
    } else {
        V3::new(dx / l, 0.0, dz / l)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(id: u32, x: f32, z: f32) -> Candidate {
        Candidate {
            id,
            pos: V3::new(x, 0.0, z),
            down_t: 0.0,
            at_smoko: false,
            teammate: false,
        }
    }

    #[test]
    fn picks_the_nearest_person_in_front() {
        let me = V3::ZERO;
        let c = [cand(1, 0.0, 2.0), cand(2, 0.0, 1.0), cand(3, 0.0, -1.0)];
        assert_eq!(pick_target(me, (0.0, 1.0), &c, false), Some(2));
    }

    #[test]
    fn ignores_people_behind_you_or_out_of_reach() {
        let me = V3::ZERO;
        assert_eq!(
            pick_target(me, (0.0, 1.0), &[cand(1, 0.0, -1.0)], false),
            None
        );
        assert_eq!(
            pick_target(me, (0.0, 1.0), &[cand(1, 0.0, 2.7)], false),
            None
        );
        assert_eq!(
            pick_target(me, (0.0, 1.0), &[cand(1, 0.0, 2.5)], false),
            Some(1)
        );
        // sideways: 90 degrees is outside the cone (dot 0 < 0.4)
        assert_eq!(
            pick_target(me, (0.0, 1.0), &[cand(1, 1.5, 0.0)], false),
            None
        );
        // 60 degrees off is fine (dot 0.5)
        assert_eq!(
            pick_target(me, (0.0, 1.0), &[cand(1, 1.3, 0.75)], false),
            Some(1)
        );
    }

    #[test]
    fn ignores_the_height_gap_down_people_smoko_and_teammates() {
        let me = V3::ZERO;
        let mut high = cand(1, 0.0, 1.0);
        high.pos.y = 1.7;
        assert_eq!(pick_target(me, (0.0, 1.0), &[high], false), None);
        let mut down = cand(2, 0.0, 1.0);
        down.down_t = 0.5;
        assert_eq!(pick_target(me, (0.0, 1.0), &[down], false), None);
        down.down_t = 0.2; // nearly up: still fair game
        assert_eq!(pick_target(me, (0.0, 1.0), &[down], false), Some(2));
        let mut smoko = cand(3, 0.0, 1.0);
        smoko.at_smoko = true;
        assert_eq!(pick_target(me, (0.0, 1.0), &[smoko], false), None);
        let mut mate = cand(4, 0.0, 1.0);
        mate.teammate = true;
        assert_eq!(pick_target(me, (0.0, 1.0), &[mate], false), None);
        assert_eq!(pick_target(me, (0.0, 1.0), &[mate], true), Some(4));
    }

    #[test]
    fn standing_on_top_of_someone_still_counts() {
        assert_eq!(
            pick_target(V3::ZERO, (0.0, 1.0), &[cand(1, 0.0, 0.0)], false),
            Some(1)
        );
    }

    #[test]
    fn in_front_works_for_dazza() {
        assert!(in_front(V3::ZERO, (0.0, 1.0), V3::new(0.0, 0.0, 2.0)));
        assert!(!in_front(V3::ZERO, (0.0, 1.0), V3::new(0.0, 0.0, -2.0)));
        assert!(!in_front(V3::ZERO, (0.0, 1.0), V3::new(0.0, 0.0, 3.0)));
    }

    #[test]
    fn swing_clock_is_faster_when_armed() {
        let mut c = SlapClock::default();
        assert!(c.ready(0.0, false));
        c.mark(10.0);
        assert!(!c.ready(10.4, true));
        assert!(c.ready(10.6, true));
        assert!(!c.ready(10.6, false), "bare hands wait 0.9 s");
        assert!(c.ready(10.95, false));
    }

    #[test]
    fn steak_and_fish_stun_for_2_s_and_the_noodle_for_1_2() {
        for k in [ItemKind::Steak, ItemKind::Fish] {
            let e = stun_slap(k);
            assert_eq!(e.dizzy, 2.0);
            assert!(e.stun >= 2.0);
            assert!(e.down.is_none() && e.tumble);
        }
        let n = stun_slap(ItemKind::Noodle);
        assert!((n.dizzy - 1.2).abs() < 1e-5, "{}", n.dizzy);
    }

    #[test]
    fn dildo_outcomes_are_roughly_even_and_crits_are_15_percent() {
        let mut rng = Rng::new(3);
        let (mut fly, mut cart, mut timber, mut crits) = (0, 0, 0, 0);
        let n = 6000;
        for _ in 0..n {
            let r = roll_dildo(&mut rng, DildoVariant::Classic);
            match r.pose {
                SlapKind::SentFlying => fly += 1,
                SlapKind::Cartwheel => cart += 1,
                SlapKind::Timber => timber += 1,
            }
            if r.crit {
                crits += 1;
                assert_eq!(r.down, 5.0);
            } else {
                assert_eq!(r.down, 3.5);
            }
        }
        for c in [fly, cart, timber] {
            assert!((c as f32 / n as f32 - 1.0 / 3.0).abs() < 0.03, "{c}");
        }
        assert!((crits as f32 / n as f32 - 0.15).abs() < 0.02, "{crits}");
    }

    #[test]
    fn variant_changes_the_time_down() {
        let mut rng = Rng::new(1);
        let mut seen = std::collections::HashSet::new();
        for _ in 0..200 {
            let r = roll_dildo(&mut rng, DildoVariant::Mini);
            if !r.crit {
                seen.insert((r.down * 10.0) as i32);
            }
        }
        assert_eq!(seen, [25].into_iter().collect());
        let mut jumbo = Rng::new(1);
        let r = (0..100)
            .map(|_| roll_dildo(&mut jumbo, DildoVariant::Jumbo))
            .find(|r| !r.crit)
            .unwrap();
        assert_eq!(r.down, 4.5);
    }

    #[test]
    fn dildo_slap_numbers_follow_the_pose() {
        let sf = dildo_slap(&DildoRoll {
            pose: SlapKind::SentFlying,
            crit: false,
            down: 3.5,
        });
        assert_eq!((sf.knock, sf.up), (13.0, 5.5));
        let tb = dildo_slap(&DildoRoll {
            pose: SlapKind::Timber,
            crit: false,
            down: 3.5,
        });
        assert_eq!((tb.knock, tb.up), (2.5, 2.5));
        let crit = dildo_slap(&DildoRoll {
            pose: SlapKind::SentFlying,
            crit: true,
            down: 5.0,
        });
        assert!((crit.knock - 13.0 * 1.9).abs() < 1e-4);
        assert_eq!(crit.down, Some((SlapKind::SentFlying, 5.0)));
    }

    #[test]
    fn apply_pushes_and_stuns_but_stuns_never_stack() {
        let mut body = Body::default();
        let mut m = Mover::new(0.0, 0.0);
        let dir = V3::new(0.0, 0.0, 1.0);
        let fx = stun_slap(ItemKind::Steak);
        let a = apply(&mut body, &mut m, dir, &fx);
        assert!(a.stunned && !a.knocked_down);
        assert!(m.vz >= 2.99 && m.vy > 1.0 && !m.grounded);
        let first = body.stun;
        let b = apply(&mut body, &mut m, dir, &fx);
        assert!(!b.stunned, "already stunned");
        assert_eq!(body.stun, first);
        assert!(m.vz >= 5.99, "but they still get shoved again");
    }

    #[test]
    fn dildo_slap_knocks_flat_unless_already_down() {
        let mut rng = Rng::new(5);
        let roll = roll_dildo(&mut rng, DildoVariant::Classic);
        let fx = dildo_slap(&roll);
        let mut body = Body::default();
        let mut m = Mover::new(0.0, 0.0);
        let a = apply(&mut body, &mut m, V3::new(1.0, 0.0, 0.0), &fx);
        assert!(a.knocked_down && body.down_t > 3.0);
        let mut b2 = Body::default();
        b2.start_fall(10.0);
        let c = apply(&mut b2, &mut m, V3::new(1.0, 0.0, 0.0), &fx);
        assert!(
            !c.knocked_down,
            "someone who stacked it isn't flattened again"
        );
    }

    #[test]
    fn silly_slap_is_a_quick_shove() {
        let s = silly_slap();
        assert_eq!((s.knock, s.up, s.stun), (8.5, 3.0, 0.6));
        assert!(s.down.is_none());
    }

    #[test]
    fn fish_smell_odds() {
        let mut rng = Rng::new(11);
        let (mut rancid, mut mild, mut none) = (0, 0, 0);
        let n = 20000;
        for _ in 0..n {
            match fish_smell(&mut rng) {
                Smell::Rancid => rancid += 1,
                Smell::Mild => mild += 1,
                Smell::None => none += 1,
            }
        }
        let f = |c: i32| c as f32 / n as f32;
        assert!((f(rancid) - 0.10).abs() < 0.01, "{}", f(rancid));
        assert!((f(mild) - 0.9 * 0.33).abs() < 0.015, "{}", f(mild));
        assert!((f(none) - 0.9 * 0.67).abs() < 0.015);
        assert_eq!(Smell::Rancid.seconds(), 9.0);
        assert_eq!(Smell::Mild.seconds(), 5.0);
    }

    #[test]
    fn direction_is_flat_and_safe_when_on_top() {
        let d = direction(V3::ZERO, V3::new(3.0, 5.0, 4.0));
        assert!((d.x - 0.6).abs() < 1e-5 && (d.z - 0.8).abs() < 1e-5 && d.y == 0.0);
        assert_eq!(direction(V3::ZERO, V3::ZERO), V3::new(0.0, 0.0, 1.0));
    }
}
