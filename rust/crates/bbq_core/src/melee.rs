//! Slaps: who a swing hits, and what each kind of slap does (spec section 3, "Melee slaps").

use crate::PlayerId;
use crate::items::{CRIT_CHANCE, DildoVariant, ItemKind, MEAT_PTS, MEAT_STUN, Melee};
use crate::pose::SlapKind;
use crate::rng::Rng;
use crate::vec::V3;

/// Reach of your own swing (horizontal), and how far above or below counts.
pub const REACH: f32 = 2.6;
pub const REACH_VERTICAL: f32 = 1.6;
/// You hit people inside a cone: `dot(facing, toTarget) >= CONE`.
pub const CONE: f32 = 0.4;
/// Reach the host allows when it checks a slap.
pub const HOST_REACH: f32 = 3.6;
/// Reach for a bare-handed slap (host check).
pub const SILLY_REACH: f32 = 2.9;
/// Cooldown between swings: armed and bare-handed (your own computer).
pub const COOLDOWN_ARMED: f32 = 0.55;
pub const COOLDOWN_UNARMED: f32 = 0.9;
/// The host's own minimum gap between slaps.
pub const HOST_COOLDOWN: f32 = 0.45;
/// The swing animation.
pub const SWING_TIME: f32 = 0.28;
/// A victim who has been down longer than this can't be hit again (you don't see them).
pub const DOWN_PROTECT: f32 = 0.3;
/// Knockback of a bare-handed slap, and how far up it lifts, and the stun.
pub const SILLY_KNOCK: f32 = 8.5;
pub const SILLY_UP: f32 = 3.0;
pub const SILLY_STUN: f32 = 0.6;
/// A critical dildo slap knocks back this much harder.
pub const CRIT_KNOCK_MUL: f32 = 1.9;
/// Names of the bare-handed slaps.
pub const SILLY_NAMES: [&str; 3] = ["WET WILLY!", "NOOGIE!", "WEDGIE!"];
/// Names of the three dildo outcomes, in `SlapKind::ALL` order.
pub const SLAP_NAMES: [&str; 3] = ["SENT FLYING!", "CARTWHEEL!", "TIMBER!"];

/// Somebody a swing could hit.
#[derive(Clone, Copy, Debug)]
pub struct Candidate {
    pub id: PlayerId,
    pub pos: V3,
    pub down_t: f32,
    pub at_smoko: bool,
}

/// The nearest person in front of you inside the cone, or `None`. `facing` is a horizontal
/// direction. Teammates (unless friendly fire) should be left out of `cands` by the caller.
pub fn pick_victim(my_pos: V3, facing: (f32, f32), cands: &[Candidate]) -> Option<PlayerId> {
    let fl = facing.0.hypot(facing.1);
    let (fx, fz) = if fl < 1e-4 {
        (0.0, 1.0)
    } else {
        (facing.0 / fl, facing.1 / fl)
    };
    let mut best: Option<(PlayerId, f32)> = None;
    for c in cands {
        if c.down_t > DOWN_PROTECT || c.at_smoko {
            continue;
        }
        let (dx, dz) = (c.pos.x - my_pos.x, c.pos.z - my_pos.z);
        let l = dx.hypot(dz);
        if l > REACH || (c.pos.y - my_pos.y).abs() > REACH_VERTICAL {
            continue;
        }
        if l > 0.01 && (dx / l * fx + dz / l * fz) < CONE {
            continue;
        }
        if best.is_none_or(|(_, bd)| l < bd) {
            best = Some((c.id, l));
        }
    }
    best.map(|b| b.0)
}

/// Is Dazza (or anything else at `(dx, dz)` from you) in the swing cone?
pub fn in_cone(my_pos: V3, facing: (f32, f32), target: V3) -> bool {
    let fl = facing.0.hypot(facing.1);
    let (fx, fz) = if fl < 1e-4 {
        (0.0, 1.0)
    } else {
        (facing.0 / fl, facing.1 / fl)
    };
    let (dx, dz) = (target.x - my_pos.x, target.z - my_pos.z);
    let l = dx.hypot(dz);
    l < REACH && (l < 0.01 || (dx / l * fx + dz / l * fz) >= CONE)
}

/// Unit knock direction from the attacker to the victim (falls back to +z if on top of them).
pub fn knock_dir(from: V3, to: V3) -> V3 {
    let (dx, dz) = (to.x - from.x, to.z - from.z);
    let l = dx.hypot(dz);
    if l < 1e-2 {
        V3::new(0.0, 0.0, 1.0)
    } else {
        V3::new(dx / l, 0.0, dz / l)
    }
}

/// How smelly a fish slap was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Smell {
    None,
    /// "PONG!": a mild cloud for 5 s.
    Mild,
    /// "THAT FISH WAS OFF!": a bad cloud for 9 s.
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

/// Each fish slap: 10% rancid, otherwise 33% mild, otherwise nothing. Pure joke.
pub fn fish_smell(rng: &mut Rng) -> Smell {
    if rng.chance(0.1) {
        Smell::Rancid
    } else if rng.chance(0.33) {
        Smell::Mild
    } else {
        Smell::None
    }
}

/// What a steak, fish or noodle slap does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StunSlap {
    pub stun: f32,
    pub points: i32,
    pub knock: f32,
}

/// Stun slap numbers for an item (noodle: 1.2 s and 50; steak and fish: 2 s and 50).
pub fn stun_slap(kind: ItemKind) -> StunSlap {
    let d = kind.def();
    StunSlap {
        stun: d.slap_stun.unwrap_or(MEAT_STUN),
        points: d.slap_pts.unwrap_or(MEAT_PTS),
        knock: d.knock,
    }
}

/// What a dildo slap does, rolled once so every screen plays the same thing.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DildoSlap {
    pub anim: SlapKind,
    pub crit: bool,
    pub down_time: f32,
    pub knock_mul: f32,
}

pub fn roll_dildo_slap(variant: DildoVariant, rng: &mut Rng) -> DildoSlap {
    let anim = SlapKind::ALL[rng.index(3)];
    let crit = rng.chance(CRIT_CHANCE);
    DildoSlap {
        anim,
        crit,
        down_time: variant.knockdown_time(crit),
        knock_mul: if crit { CRIT_KNOCK_MUL } else { 1.0 },
    }
}

/// Knockback speed and lift for a hit, as the JavaScript `applyKnock` does it. `anim` is the
/// slap pose, if any; `base_knock` is the item's own knock number.
pub fn knock_numbers(anim: Option<SlapKind>, base_knock: f32, mul: f32) -> (f32, f32) {
    let (kn, up) = match anim {
        Some(SlapKind::SentFlying) => (13.0, 5.5),
        Some(SlapKind::Cartwheel) => (base_knock, 2.5),
        Some(SlapKind::Timber) => (2.5, 2.5),
        None => (base_knock, base_knock * 0.38),
    };
    (kn * mul, up)
}

/// Can this kind of item be swung at all? Returns what sort of swing it is.
pub fn swing_kind(kind: Option<ItemKind>) -> Melee {
    kind.map_or(Melee::None, |k| k.def().melee)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(id: PlayerId, x: f32, z: f32) -> Candidate {
        Candidate {
            id,
            pos: V3::new(x, 0.0, z),
            down_t: 0.0,
            at_smoko: false,
        }
    }

    #[test]
    fn the_nearest_person_in_front_is_hit() {
        let me = V3::ZERO;
        let c = [cand(2, 0.0, 2.0), cand(3, 0.0, 1.0), cand(4, 0.0, -1.0)];
        assert_eq!(pick_victim(me, (0.0, 1.0), &c), Some(3));
    }

    #[test]
    fn behind_you_or_out_of_reach_is_a_miss() {
        let me = V3::ZERO;
        assert_eq!(pick_victim(me, (0.0, 1.0), &[cand(2, 0.0, -1.5)]), None);
        assert_eq!(pick_victim(me, (0.0, 1.0), &[cand(2, 0.0, 2.7)]), None);
        assert_eq!(pick_victim(me, (0.0, 1.0), &[cand(2, 0.0, 2.5)]), Some(2));
        // way off to the side is outside the cone (dot < 0.4)
        assert_eq!(pick_victim(me, (0.0, 1.0), &[cand(2, 2.0, 0.5)]), None);
    }

    #[test]
    fn someone_right_on_top_of_you_is_always_in_the_cone() {
        assert_eq!(
            pick_victim(V3::ZERO, (0.0, 1.0), &[cand(2, 0.005, 0.0)]),
            Some(2)
        );
    }

    #[test]
    fn people_down_or_seated_or_at_the_wrong_height_are_skipped() {
        let me = V3::ZERO;
        let mut down = cand(2, 0.0, 1.0);
        down.down_t = 0.5;
        let mut barely = cand(3, 0.0, 1.5);
        barely.down_t = 0.3;
        let mut sat = cand(4, 0.0, 1.0);
        sat.at_smoko = true;
        let mut high = cand(5, 0.0, 1.0);
        high.pos.y = 1.7;
        assert_eq!(pick_victim(me, (0.0, 1.0), &[down, sat, high]), None);
        assert_eq!(pick_victim(me, (0.0, 1.0), &[down, barely]), Some(3));
    }

    #[test]
    fn stun_slaps() {
        let steak = stun_slap(ItemKind::Steak);
        assert_eq!((steak.stun, steak.points), (2.0, 50));
        let noodle = stun_slap(ItemKind::Noodle);
        assert_eq!((noodle.stun, noodle.points), (1.2, 50));
        assert_eq!(noodle.knock, 5.0);
    }

    #[test]
    fn fish_smell_odds() {
        let mut rng = Rng::new(11);
        let (mut none, mut mild, mut rancid) = (0, 0, 0);
        for _ in 0..20_000 {
            match fish_smell(&mut rng) {
                Smell::None => none += 1,
                Smell::Mild => mild += 1,
                Smell::Rancid => rancid += 1,
            }
        }
        let n = 20_000.0;
        assert!((rancid as f32 / n - 0.10).abs() < 0.015);
        assert!((mild as f32 / n - 0.9 * 0.33).abs() < 0.02);
        assert!(none > mild);
        assert_eq!(Smell::Rancid.seconds(), 9.0);
        assert_eq!(Smell::Mild.seconds(), 5.0);
    }

    #[test]
    fn dildo_slap_odds_and_times() {
        let mut rng = Rng::new(5);
        let (mut crits, mut anims) = (0, [0; 3]);
        for _ in 0..20_000 {
            let s = roll_dildo_slap(DildoVariant::Classic, &mut rng);
            if s.crit {
                crits += 1;
                assert_eq!(s.down_time, 5.0);
                assert_eq!(s.knock_mul, 1.9);
            } else {
                assert_eq!(s.down_time, 3.5);
                assert_eq!(s.knock_mul, 1.0);
            }
            anims[SlapKind::ALL.iter().position(|k| *k == s.anim).unwrap()] += 1;
        }
        assert!((crits as f32 / 20_000.0 - 0.15).abs() < 0.015);
        for a in anims {
            assert!((a as f32 / 20_000.0 - 1.0 / 3.0).abs() < 0.02);
        }
        assert_eq!(DildoVariant::Mini.knockdown_time(false), 2.5);
        assert_eq!(DildoVariant::Jumbo.knockdown_time(false), 4.5);
        assert_eq!(DildoVariant::Gold.knockdown_time(false), 5.5);
    }

    #[test]
    fn knock_numbers_match_the_old_game() {
        assert_eq!(knock_numbers(Some(SlapKind::SentFlying), 9.0, 1.0), (13.0, 5.5));
        assert_eq!(knock_numbers(Some(SlapKind::Cartwheel), 9.0, 1.0), (9.0, 2.5));
        assert_eq!(knock_numbers(Some(SlapKind::Timber), 9.0, 1.0), (2.5, 2.5));
        let (kn, up) = knock_numbers(None, 7.0, 1.0);
        assert_eq!(kn, 7.0);
        assert!((up - 2.66).abs() < 1e-4);
        assert!((knock_numbers(Some(SlapKind::SentFlying), 9.0, 1.9).0 - 24.7).abs() < 1e-4);
    }

    #[test]
    fn knock_direction_points_away_from_the_attacker() {
        let d = knock_dir(V3::ZERO, V3::new(3.0, 1.0, 4.0));
        assert!((d.x - 0.6).abs() < 1e-5 && (d.z - 0.8).abs() < 1e-5 && d.y == 0.0);
        assert_eq!(knock_dir(V3::ZERO, V3::ZERO), V3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn which_items_can_swing() {
        assert_eq!(swing_kind(None), Melee::None);
        assert_eq!(swing_kind(Some(ItemKind::Teddy)), Melee::None);
        assert_eq!(swing_kind(Some(ItemKind::Steak)), Melee::Stun);
        assert_eq!(swing_kind(Some(ItemKind::Dildo)), Melee::Down);
    }
}
