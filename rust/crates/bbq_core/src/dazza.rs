//! Dazza, the BBQ cook: his six states and how each one looks (spec section 12).
//!
//! His behaviour (anger, chasing, the spatula) arrives in Phase 5; this is his look and the
//! numbers his animation depends on.

use crate::vec::V3;

const PI: f32 = std::f32::consts::PI;

/// Where Dazza stands when cooking: behind the grill.
pub const HOME: V3 = V3::new(-6.0, 0.0, -19.3);
/// How long his speech bubbles stay up.
pub const BUBBLE_TIME: f32 = 2.8;
/// The spatula swing animation.
pub const SWING_TIME: f32 = 0.35;
pub const ANGER_MAX: u32 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DazzaState {
    Cook,
    Angry,
    Chase,
    Return,
    Ko,
    Stunned,
}

impl DazzaState {
    pub const ALL: [DazzaState; 6] = [
        DazzaState::Cook,
        DazzaState::Angry,
        DazzaState::Chase,
        DazzaState::Return,
        DazzaState::Ko,
        DazzaState::Stunned,
    ];
}

/// The pose of Dazza's body and spatula arm.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DazzaPose {
    /// Whole-body rotation about x (lean or lying flat) and z (waddle).
    pub body_rx: f32,
    pub body_rz: f32,
    pub body_y: f32,
    pub body_scale: f32,
    /// Spatula arm rotation (x, y, z).
    pub arm: (f32, f32, f32),
}

/// Dazza's animation state.
#[derive(Clone, Copy, Debug, Default)]
pub struct DazzaAnim {
    pub ko_age: f32,
    /// Walk phase.
    pub walk: f32,
    /// Seconds left in the spatula swing (0 = not swinging).
    pub swing_t: f32,
    pub bubble_t: f32,
}

impl DazzaAnim {
    pub fn start_swing(&mut self) {
        self.swing_t = SWING_TIME;
    }

    pub fn say(&mut self) {
        self.bubble_t = BUBBLE_TIME;
    }

    pub fn bubble_visible(&self) -> bool {
        self.bubble_t > 0.0
    }

    /// Advance and pose. `t` is game time; `dist_home` is how far he is from his spot;
    /// `moved` is how far he moved this frame (drives the walk cycle).
    pub fn tick(
        &mut self,
        dt: f32,
        state: DazzaState,
        t: f32,
        dist_home: f32,
        moved: f32,
    ) -> DazzaPose {
        self.bubble_t = (self.bubble_t - dt).max(0.0);
        // 2.4 rad of walk cycle per metre, as in the browser game (it was 30, which spun the
        // bob about 19 times a second and made him judder)
        self.walk += moved * 2.4;
        if state == DazzaState::Ko {
            self.ko_age += dt;
            let f = (self.ko_age / 0.3).min(1.0);
            return DazzaPose {
                body_rx: -PI / 2.0 * f,
                body_rz: (t * 1.3).sin() * 0.04,
                body_y: 0.42 * f,
                body_scale: 1.0,
                arm: (-0.3, 0.0, 0.6),
            };
        }
        self.ko_age = 0.0;
        let chasing_far = state == DazzaState::Chase && dist_home > 2.0;
        let moving = matches!(state, DazzaState::Chase | DazzaState::Return)
            || (state == DazzaState::Angry && dist_home > 0.3);
        let mut pose = DazzaPose {
            body_scale: if chasing_far {
                1.0 + (t * 14.0).sin() * 0.03
            } else {
                1.0
            },
            body_y: if moving {
                self.walk.sin().abs() * 0.1
            } else {
                0.0
            },
            body_rz: if moving { self.walk.sin() * 0.06 } else { 0.0 },
            body_rx: if state == DazzaState::Chase {
                0.18
            } else {
                0.0
            },
            arm: (-2.2, 0.0, -0.4),
        };
        if self.swing_t > 0.0 {
            self.swing_t = (self.swing_t - dt).max(0.0);
            let p = 1.0 - self.swing_t / SWING_TIME;
            pose.arm = (-2.2 + p * 3.2, 0.0, -0.4 + p * 0.6);
        } else if state == DazzaState::Cook {
            let f = (t * 1.3).sin().max(0.0);
            pose.arm = (-0.6 - 0.9 * f.powi(6), 0.0, 0.0);
        }
        pose
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn cooking_he_flips_the_snags_with_a_slow_arm() {
        let mut a = DazzaAnim::default();
        let p = a.tick(DT, DazzaState::Cook, 0.0, 0.0, 0.0);
        assert_eq!(p.body_y, 0.0);
        assert!((p.arm.0 + 0.6).abs() < 1e-5);
        let peak = (0..300)
            .map(|i| a.tick(DT, DazzaState::Cook, i as f32 * DT, 0.0, 0.0).arm.0)
            .fold(0.0f32, f32::min);
        assert!((-1.5 - 1e-3..-1.3).contains(&peak), "{peak}");
    }

    #[test]
    fn chasing_he_leans_forward_and_bobs() {
        let mut a = DazzaAnim::default();
        let mut maxy = 0.0f32;
        for i in 0..60 {
            let p = a.tick(DT, DazzaState::Chase, i as f32 * DT, 5.0, 0.1);
            assert_eq!(p.body_rx, 0.18);
            maxy = maxy.max(p.body_y);
        }
        assert!(maxy > 0.05 && maxy <= 0.1 + 1e-4);
    }

    #[test]
    fn angry_at_the_grill_he_stands_still_but_walking_home_he_bobs() {
        let mut a = DazzaAnim {
            walk: 1.0,
            ..Default::default()
        };
        let still = a.tick(DT, DazzaState::Angry, 0.0, 0.1, 0.0);
        assert_eq!(still.body_y, 0.0);
        let walking = a.tick(DT, DazzaState::Angry, 0.0, 3.0, 0.0);
        assert!(walking.body_y > 0.0);
        assert_eq!(walking.arm, (-2.2, 0.0, -0.4), "spatula held up");
    }

    #[test]
    fn knocked_out_he_falls_flat_in_0_3_s() {
        let mut a = DazzaAnim::default();
        let mut p = DazzaPose::default();
        for _ in 0..30 {
            p = a.tick(DT, DazzaState::Ko, 0.0, 0.0, 0.0);
        }
        assert!((p.body_rx + PI / 2.0).abs() < 1e-3);
        assert!((p.body_y - 0.42).abs() < 1e-3);
        // waking up resets the fall
        a.tick(DT, DazzaState::Cook, 0.0, 0.0, 0.0);
        assert_eq!(a.ko_age, 0.0);
    }

    #[test]
    fn a_swing_takes_0_35_s_and_sweeps_the_arm_forward() {
        let mut a = DazzaAnim::default();
        a.start_swing();
        let first = a.tick(DT, DazzaState::Chase, 0.0, 1.0, 0.0).arm.0;
        let mut last = first;
        for _ in 0..20 {
            last = a.tick(DT, DazzaState::Chase, 0.0, 1.0, 0.0).arm.0;
        }
        assert!(first < -2.0 && last > 0.8, "{first} {last}");
        assert_eq!(a.swing_t, 0.0);
    }

    #[test]
    fn stunned_he_just_stands_there() {
        let mut a = DazzaAnim::default();
        let p = a.tick(DT, DazzaState::Stunned, 1.0, 0.0, 0.0);
        assert_eq!(p.body_y, 0.0);
        assert_eq!(p.body_rx, 0.0);
    }

    #[test]
    fn speech_bubbles_last_2_8_seconds() {
        let mut a = DazzaAnim::default();
        a.say();
        assert!(a.bubble_visible());
        for _ in 0..170 {
            a.tick(DT, DazzaState::Cook, 0.0, 0.0, 0.0);
        }
        assert!(!a.bubble_visible());
    }

    #[test]
    fn all_six_states_are_listed() {
        assert_eq!(DazzaState::ALL.len(), 6);
        assert_eq!(HOME, V3::new(-6.0, 0.0, -19.3));
    }
}
