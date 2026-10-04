//! Smoko and the Naughty Corner (spec section 5).
//!
//! The pad is a ring of folding chairs. Sit down and nobody can touch you, but your drunk meter
//! climbs. In Naughty Corner mode nobody can sit for a safe break: a person dropped or thrown
//! into the zone gets stuck on a chair for 5 seconds and the sender scores.

use crate::yard::{SMOKO_X, SMOKO_Z};

/// A smoko ends by itself after this long.
pub const MAX_TIME: f32 = 20.0;
/// Time stuck on a chair in the Naughty Corner.
pub const NAUGHTY_SIT: f32 = 5.0;
/// A thrown person has to land (grounded, at least this long after the throw)...
pub const NAUGHTY_LAND_MIN: f32 = 0.15;
/// ...and within this many seconds, to count.
pub const NAUGHTY_LAND_MAX: f32 = 3.0;
/// You can't sit while higher than this.
pub const SIT_HEIGHT: f32 = 0.5;
/// When you stand up you land this far outside your chair.
pub const STAND_OFFSET: f32 = 0.6;
/// First drink sound after sitting, then every 4 s (a sound only; nothing for the rules).
pub const FIRST_SIP: f32 = 1.5;

/// How many chairs: one per person, between 2 and 16.
pub fn chair_count(players: usize) -> usize {
    players.clamp(2, 16)
}

/// Radius of the ring of chairs.
pub fn ring_radius(chairs: usize) -> f32 {
    1.9 + chairs as f32 * 0.2
}

/// Radius of the zone you have to stand in to sit (or to be sent to the Naughty Corner).
pub fn zone_radius(players: usize) -> f32 {
    ring_radius(chair_count(players)) + 1.3
}

/// Where chair `i` of `n` stands.
pub fn seat_pos(i: usize, n: usize) -> (f32, f32) {
    let r = ring_radius(n);
    let a = i as f32 / n as f32 * std::f32::consts::TAU + 0.4;
    (SMOKO_X + a.cos() * r, SMOKO_Z + a.sin() * r)
}

/// The yaw of someone sitting in a chair so that they look at the middle of the pad. Yaw is
/// the camera sense used everywhere: looking along `(-sin yaw, -cos yaw)`.
pub fn seat_face(x: f32, z: f32) -> f32 {
    (x - SMOKO_X).atan2(z - SMOKO_Z)
}

/// Is this spot inside the smoko zone (and low enough to count)?
pub fn in_zone(x: f32, z: f32, y: f32, players: usize) -> bool {
    y < SIT_HEIGHT && (x - SMOKO_X).hypot(z - SMOKO_Z) < zone_radius(players)
}

/// The free chair closest to `(x, z)`, if any. `taken` lists the chairs in use.
pub fn nearest_free_seat(x: f32, z: f32, chairs: usize, taken: &[usize]) -> Option<usize> {
    (0..chairs)
        .filter(|i| !taken.contains(i))
        .map(|i| {
            let (sx, sz) = seat_pos(i, chairs);
            (i, (sx - x).hypot(sz - z))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// The chair closest to `(x, z)`, taken or not (the Naughty Corner puts you on the nearest one).
pub fn nearest_seat(x: f32, z: f32, chairs: usize) -> usize {
    nearest_free_seat(x, z, chairs, &[]).unwrap_or(0)
}

/// Where you end up when you stand: pushed 0.6 m straight out from the middle of the pad.
pub fn stand_spot(x: f32, z: f32) -> (f32, f32) {
    let (dx, dz) = (x - SMOKO_X, z - SMOKO_Z);
    let l = dx.hypot(dz);
    let l = if l < 1e-6 { 1.0 } else { l };
    (x + dx / l * STAND_OFFSET, z + dz / l * STAND_OFFSET)
}

/// Does a thrown person's landing count for the Naughty Corner? They must be on the ground,
/// have been in the air at least 0.15 s and no more than 3 s.
pub fn thrown_lands_in_corner(grounded: bool, since_throw: f32, in_zone: bool) -> bool {
    grounded && in_zone && (NAUGHTY_LAND_MIN..=NAUGHTY_LAND_MAX).contains(&since_throw)
}

/// Someone sitting at smoko.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Seated {
    pub seat: usize,
    pub t: f32,
    /// Counts down to the next sip sound.
    pub sip_t: f32,
}

impl Seated {
    pub fn new(seat: usize) -> Self {
        Seated {
            seat,
            t: 0.0,
            sip_t: FIRST_SIP,
        }
    }

    /// Advance. Returns true on the tick the smoko times out.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.t += dt;
        self.sip_t -= dt;
        if self.sip_t <= 0.0 {
            self.sip_t = 4.0;
        }
        self.t >= MAX_TIME
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chairs_follow_the_player_count() {
        assert_eq!(chair_count(0), 2);
        assert_eq!(chair_count(1), 2);
        assert_eq!(chair_count(5), 5);
        assert_eq!(chair_count(40), 16);
    }

    #[test]
    fn ring_and_zone_sizes() {
        assert!((ring_radius(2) - 2.3).abs() < 1e-5);
        assert!((zone_radius(2) - 3.6).abs() < 1e-5);
        assert!((zone_radius(16) - (1.9 + 3.2 + 1.3)).abs() < 1e-4);
        // one player still gets a 2-chair zone
        assert_eq!(zone_radius(1), zone_radius(2));
    }

    #[test]
    fn chair_zero_sits_at_angle_0_4() {
        let (x, z) = seat_pos(0, 4);
        let r = ring_radius(4);
        assert!((x - (SMOKO_X + 0.4f32.cos() * r)).abs() < 1e-5);
        assert!((z - (SMOKO_Z + 0.4f32.sin() * r)).abs() < 1e-5);
        // every chair is on the ring
        for i in 0..4 {
            let (x, z) = seat_pos(i, 4);
            assert!(((x - SMOKO_X).hypot(z - SMOKO_Z) - r).abs() < 1e-4);
        }
    }

    #[test]
    fn zone_needs_you_low_and_close() {
        assert!(in_zone(SMOKO_X, SMOKO_Z, 0.0, 4));
        assert!(!in_zone(SMOKO_X + 10.0, SMOKO_Z, 0.0, 4));
        assert!(!in_zone(SMOKO_X, SMOKO_Z, 0.6, 4), "up on a trampoline bounce");
    }

    #[test]
    fn nearest_free_seat_skips_taken_chairs() {
        let (x, z) = seat_pos(2, 4);
        assert_eq!(nearest_free_seat(x, z, 4, &[]), Some(2));
        let next = nearest_free_seat(x, z, 4, &[2]).unwrap();
        assert_ne!(next, 2);
        assert_eq!(nearest_free_seat(x, z, 2, &[0, 1]), None);
        assert_eq!(nearest_seat(x, z, 4), 2);
    }

    #[test]
    fn standing_up_pushes_you_out_from_the_middle() {
        let (x, z) = seat_pos(0, 4);
        let (nx, nz) = stand_spot(x, z);
        let before = (x - SMOKO_X).hypot(z - SMOKO_Z);
        let after = (nx - SMOKO_X).hypot(nz - SMOKO_Z);
        assert!((after - before - STAND_OFFSET).abs() < 1e-4);
    }

    #[test]
    fn a_chair_faces_the_middle() {
        for i in 0..4 {
            let (x, z) = seat_pos(i, 4);
            let f = seat_face(x, z);
            let (fx, fz) = (-f.sin(), -f.cos());
            let (cx, cz) = (SMOKO_X - x, SMOKO_Z - z);
            let l = cx.hypot(cz);
            assert!((fx * cx + fz * cz) / l > 0.999, "chair {i}");
        }
    }

    #[test]
    fn a_smoko_times_out_after_20_seconds() {
        let mut s = Seated::new(0);
        let mut secs = 0.0;
        while !s.tick(1.0 / 60.0) {
            secs += 1.0 / 60.0;
            assert!(secs < 30.0);
        }
        assert!((secs - MAX_TIME).abs() < 0.05, "{secs}");
    }

    #[test]
    fn naughty_corner_landing_window() {
        assert!(thrown_lands_in_corner(true, 0.5, true));
        assert!(!thrown_lands_in_corner(true, 0.1, true), "too soon");
        assert!(!thrown_lands_in_corner(true, 3.5, true), "too late");
        assert!(!thrown_lands_in_corner(false, 0.5, true), "still airborne");
        assert!(!thrown_lands_in_corner(true, 0.5, false), "outside the zone");
    }
}
