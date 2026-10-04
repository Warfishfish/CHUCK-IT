//! Smoko and the Naughty Corner (spec section 5): a ring of chairs where you sit, nobody can
//! touch you, but you keep sipping a drink. In Naughty Corner mode the chairs become a
//! punishment instead: a mate who drags you in and drops or throws you there sends you to a
//! chair for 5 seconds.

use crate::drinks::SMOKO_SIP;
use crate::yard::{SMOKO_X, SMOKO_Z};

/// Smoko ends by itself after this long.
pub const MAX_TIME: f32 = 20.0;
/// How often you take a sip (and the glug sound plays).
pub const SIP_EVERY: f32 = 4.0;
/// First sip comes a bit sooner.
pub const FIRST_SIP: f32 = 1.5;
/// Seconds stuck on the chair in the Naughty Corner.
pub const NAUGHTY_SIT: f32 = 5.0;
/// Points for sending someone there.
pub const NAUGHTY_PTS: i32 = 100;
/// A thrown person must land at least this long after the throw, and within this long, to count.
pub const NAUGHTY_LAND_AFTER: f32 = 0.15;
pub const NAUGHTY_LAND_WITHIN: f32 = 3.0;

/// Chairs: one for each player, between 2 and 16.
pub fn chair_count(players: usize) -> usize {
    players.clamp(2, 16)
}

/// Radius of the ring the chairs sit on.
pub fn ring(chairs: usize) -> f32 {
    1.9 + chairs as f32 * 0.2
}

/// How far from the pad's middle still counts as being at smoko.
pub fn zone_radius(players: usize) -> f32 {
    ring(chair_count(players)) + 1.3
}

/// Where chair `i` of `n` stands.
pub fn seat_pos(i: usize, n: usize) -> (f32, f32) {
    let r = ring(n);
    let a = i as f32 / n as f32 * std::f32::consts::TAU + 0.4;
    (SMOKO_X + a.cos() * r, SMOKO_Z + a.sin() * r)
}

/// Which way chair `i` faces: towards the middle.
pub fn seat_facing(i: usize, n: usize) -> f32 {
    let (x, z) = seat_pos(i, n);
    (SMOKO_X - x).atan2(SMOKO_Z - z)
}

/// On the ground and inside the zone.
pub fn in_zone(x: f32, y: f32, z: f32, players: usize) -> bool {
    y < 0.5 && (x - SMOKO_X).hypot(z - SMOKO_Z) < zone_radius(players)
}

/// The free chair nearest to (x, z).
pub fn nearest_free_seat(x: f32, z: f32, n: usize, taken: &[bool]) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for i in 0..n {
        if taken.get(i).copied().unwrap_or(false) {
            continue;
        }
        let (sx, sz) = seat_pos(i, n);
        let d = (sx - x).hypot(sz - z);
        if best.is_none_or(|(_, b)| d < b) {
            best = Some((i, d));
        }
    }
    best.map(|b| b.0)
}

/// Where you end up standing after getting up: 0.6 m further from the middle.
pub fn stand_pos(x: f32, z: f32) -> (f32, f32) {
    let (dx, dz) = (x - SMOKO_X, z - SMOKO_Z);
    let l = dx.hypot(dz).max(1e-3);
    (x + dx / l * 0.6, z + dz / l * 0.6)
}

/// Why smoko ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Over {
    /// 20 seconds up: "Smoko's over! Back to it."
    TimeUp,
    /// Pressed R or Space.
    Chose,
    /// Stunned, or the round stopped.
    Interrupted,
}

/// Someone sitting at smoko.
#[derive(Clone, Copy, Debug)]
pub struct Seated {
    pub seat: usize,
    pub t: f32,
    pub sip_t: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tick {
    /// Add this to the drunk meter.
    pub drunk: f32,
    pub sip: bool,
    pub over: Option<Over>,
}

impl Seated {
    pub fn new(seat: usize) -> Self {
        Seated {
            seat,
            t: 0.0,
            sip_t: FIRST_SIP,
        }
    }

    /// `can_act` is false when the round isn't running; `stunned` ends it too.
    pub fn tick(&mut self, dt: f32, can_act: bool, stunned: bool) -> Tick {
        self.t += dt;
        let mut out = Tick {
            drunk: SMOKO_SIP * dt,
            ..Default::default()
        };
        self.sip_t -= dt;
        if self.sip_t <= 0.0 {
            self.sip_t = SIP_EVERY;
            out.sip = true;
        }
        if self.t >= MAX_TIME {
            out.over = Some(Over::TimeUp);
        } else if !can_act || stunned {
            out.over = Some(Over::Interrupted);
        }
        out
    }
}

/// Did a thrown person land in the corner in time to count?
pub fn naughty_lands(since_throw: f32, grounded: bool, in_corner: bool) -> bool {
    grounded && in_corner && since_throw > NAUGHTY_LAND_AFTER && since_throw < NAUGHTY_LAND_WITHIN
}

/// Can this person be sent to the Naughty Corner right now?
pub fn can_send(
    naughty_on: bool,
    round_running: bool,
    victim_sitting: bool,
    same_person: bool,
) -> bool {
    naughty_on && round_running && !victim_sitting && !same_person
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chairs_between_2_and_16() {
        assert_eq!(chair_count(0), 2);
        assert_eq!(chair_count(1), 2);
        assert_eq!(chair_count(5), 5);
        assert_eq!(chair_count(40), 16);
    }

    #[test]
    fn ring_and_zone_grow_with_the_crowd() {
        assert!((ring(4) - 2.7).abs() < 1e-5);
        assert!((zone_radius(4) - 4.0).abs() < 1e-5);
        assert!(zone_radius(16) > zone_radius(2));
    }

    #[test]
    fn seats_sit_on_the_ring_and_face_the_middle() {
        for n in [2usize, 5, 16] {
            for i in 0..n {
                let (x, z) = seat_pos(i, n);
                let d = (x - SMOKO_X).hypot(z - SMOKO_Z);
                assert!((d - ring(n)).abs() < 1e-4);
                let f = seat_facing(i, n);
                // a step forward moves closer to the middle
                let (nx, nz) = (x + f.sin() * 0.1, z + f.cos() * 0.1);
                assert!((nx - SMOKO_X).hypot(nz - SMOKO_Z) < d);
            }
        }
        // chair 0 is at angle 0.4 radians
        let (x, z) = seat_pos(0, 4);
        assert!((x - (SMOKO_X + 0.4f32.cos() * 2.7)).abs() < 1e-4);
        assert!((z - (SMOKO_Z + 0.4f32.sin() * 2.7)).abs() < 1e-4);
    }

    #[test]
    fn the_zone_is_on_the_ground_around_the_pad() {
        assert!(in_zone(SMOKO_X, 0.0, SMOKO_Z, 4));
        assert!(in_zone(SMOKO_X + 3.9, 0.0, SMOKO_Z, 4));
        assert!(!in_zone(SMOKO_X + 4.1, 0.0, SMOKO_Z, 4));
        assert!(!in_zone(SMOKO_X, 1.0, SMOKO_Z, 4), "not in mid-air");
    }

    #[test]
    fn you_get_the_nearest_free_chair() {
        let n = 4;
        let (x0, z0) = seat_pos(0, n);
        assert_eq!(nearest_free_seat(x0, z0, n, &[false; 4]), Some(0));
        let taken = [true, false, false, false];
        let s = nearest_free_seat(x0, z0, n, &taken).unwrap();
        assert_ne!(s, 0);
        assert_eq!(nearest_free_seat(x0, z0, n, &[true; 4]), None);
    }

    #[test]
    fn standing_up_moves_you_out_a_bit() {
        let (x, z) = seat_pos(0, 4);
        let (sx, sz) = stand_pos(x, z);
        let before = (x - SMOKO_X).hypot(z - SMOKO_Z);
        let after = (sx - SMOKO_X).hypot(sz - SMOKO_Z);
        assert!((after - before - 0.6).abs() < 1e-3);
    }

    #[test]
    fn smoko_sips_at_4_per_second_and_lasts_20_s() {
        let mut s = Seated::new(0);
        let mut drunk = 0.0;
        let mut sips = 0;
        let mut over = None;
        for _ in 0..(21.0 * 60.0) as usize {
            let t = s.tick(1.0 / 60.0, true, false);
            drunk += t.drunk;
            sips += t.sip as i32;
            if t.over.is_some() && over.is_none() {
                over = t.over;
                break;
            }
        }
        assert_eq!(over, Some(Over::TimeUp));
        assert!((drunk - 80.0).abs() < 1.0, "{drunk}");
        // first sip at 1.5 s, then every 4 s: 1.5, 5.5, 9.5, 13.5, 17.5
        assert_eq!(sips, 5);
    }

    #[test]
    fn being_stunned_or_the_round_ending_gets_you_up() {
        let mut s = Seated::new(1);
        assert_eq!(s.tick(0.1, true, true).over, Some(Over::Interrupted));
        let mut s = Seated::new(1);
        assert_eq!(s.tick(0.1, false, false).over, Some(Over::Interrupted));
        let mut s = Seated::new(1);
        assert_eq!(s.tick(0.1, true, false).over, None);
    }

    #[test]
    fn a_thrown_person_must_land_in_the_corner_in_time() {
        assert!(naughty_lands(1.0, true, true));
        assert!(
            !naughty_lands(0.1, true, true),
            "too soon, still leaving the hands"
        );
        assert!(!naughty_lands(3.5, true, true), "too long ago");
        assert!(!naughty_lands(1.0, false, true), "still in the air");
        assert!(!naughty_lands(1.0, true, false), "landed outside");
    }

    #[test]
    fn the_naughty_corner_rules() {
        assert!(can_send(true, true, false, false));
        assert!(!can_send(false, true, false, false), "mode off");
        assert!(!can_send(true, false, false, false), "round not running");
        assert!(!can_send(true, true, true, false), "already on a chair");
        assert!(!can_send(true, true, false, true), "not yourself");
    }
}
