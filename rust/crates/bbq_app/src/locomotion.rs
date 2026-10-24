//! Presentation only. Movement, aim, collisions and network state never read this state.
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

pub const TURN_RESPONSE: f32 = 13.0;
pub const STEP_RESPONSE: f32 = 12.0;
pub const STANCE_FRACTION: f32 = 0.55;
pub const MAX_CADENCE: f32 = 13.0;
pub const BODY_BOB: f32 = 0.055;
pub const BODY_SWAY: f32 = 0.035;
pub const IDLE_BREATH: f32 = 0.009;
pub const BELLY_STIFFNESS: f32 = 125.0;
pub const BELLY_DAMPING: f32 = 6.5;
pub const CAMERA_BOB_SCALE: f32 = 0.7;

pub fn angle_delta(from: f32, to: f32) -> f32 {
    (to - from + PI).rem_euclid(TAU) - PI
}
pub fn ease(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    current + (target - current) * (1.0 - (-rate * dt.max(0.0)).exp())
}

#[derive(Clone, Copy, Default)]
pub struct Gait {
    pub yaw: f32,
    pub phase: f32,
    pub blend: f32,
    pub speed: f32,
    pub direction: Vec3,
    position: Vec3,
    ready: bool,
}
impl Gait {
    pub fn tick(&mut self, dt: f32, position: Vec3, velocity: Vec3, facing: f32,
        leg: f32, walking: bool, direct_facing: bool) {
        let dt = dt.clamp(0.0, 0.05);
        if !self.ready || self.position.distance(position) > 3.0 {
            *self = Self { yaw: facing, position, ready: true, ..Default::default() };
        }
        self.position = position;
        let previous_yaw = self.yaw;
        self.yaw = if direct_facing { facing } else {
            self.yaw + angle_delta(self.yaw, facing) * (1.0 - (-TURN_RESPONSE * dt).exp())
        };
        let speed = if walking { velocity.x.hypot(velocity.z) } else { 0.0 };
        self.speed = ease(self.speed, speed, STEP_RESPONSE, dt);
        let pivot = if walking && !direct_facing && dt > 0.0 {
            (angle_delta(previous_yaw, self.yaw).abs() / dt * 0.06).min(0.3)
        } else { 0.0 };
        self.blend = ease(self.blend, (speed / 2.5).clamp(0.0, 1.0).max(pivot), STEP_RESPONSE, dt);
        if speed <= 0.15 && pivot > 0.01 {
            self.direction = self.direction.lerp(Vec3::Z, 1.0 - (-STEP_RESPONSE * dt).exp());
            self.phase = (self.phase + pivot * 14.0 * dt).rem_euclid(TAU);
        }
        if speed > 0.15 {
            let local = Quat::from_rotation_y(-self.yaw) * Vec3::new(velocity.x, 0.0, velocity.z) / speed;
            self.direction = self.direction.lerp(local, 1.0 - (-STEP_RESPONSE * dt).exp());
            // Roughly match stance travel to world speed. Cap tiny-legged blobs' cadence.
            let stride = 0.16 + leg * 0.6;
            let cadence = (speed * PI / (2.0 * stride)).min(MAX_CADENCE);
            self.phase = (self.phase + cadence * dt).rem_euclid(TAU);
        }
    }

    pub fn foot(&self, right: bool, leg: f32) -> (Vec3, f32) {
        let u = ((self.phase / TAU) + if right { 0.5 } else { 0.0 }).fract();
        let (travel, lift) = if u < STANCE_FRACTION {
            (1.0 - 2.0 * u / STANCE_FRACTION, 0.0)
        } else {
            let t = (u - STANCE_FRACTION) / (1.0 - STANCE_FRACTION);
            let smooth = t * t * (3.0 - 2.0 * t);
            (-1.0 + 2.0 * smooth, (PI * t).sin().powi(2))
        };
        let offset = self.direction * travel * (0.16 + leg * 0.6) * self.blend
            + Vec3::Y * lift * (0.055 + leg * 0.24) * self.blend;
        (offset, -lift * 0.24 * self.blend)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn turns_take_short_path_across_wrap_and_are_frame_rate_independent() {
        let start=179f32.to_radians();let target=(-179f32).to_radians();
        assert!((angle_delta(start,target)-2f32.to_radians()).abs()<0.0001);
        let run=|hz:u32| {let mut g=Gait{yaw:start,ready:true,..Default::default()};for _ in 0..hz {
            g.tick(1.0/hz as f32,Vec3::ZERO,Vec3::ZERO,target,0.3,true,false);
        }g.yaw};
        assert!((run(30)-run(144)).abs()<0.0001);
    }
    #[test]
    fn feet_plant_then_lift_and_follow_strafing_for_all_leg_lengths() {
        for leg in [0.0,0.3,0.6] {
            let mut g=Gait{blend:1.0,direction:Vec3::X,..Default::default()};
            for u in [0.0,0.2,0.5] {g.phase=u*TAU;let (p,_)=g.foot(false,leg);assert_eq!(p.y,0.0);assert_eq!(p.z,0.0);}
            g.phase=0.775*TAU;assert!(g.foot(false,leg).0.y>0.05);
            g.phase=(1.0-0.00001)*TAU;let last=g.foot(false,leg).0;
            g.phase=0.0;assert!(last.distance(g.foot(false,leg).0)<0.0001);
        }
    }
    #[test]
    fn stops_settle_and_respawns_reset_without_rotating_through_old_heading() {
        let mut g=Gait::default();
        for _ in 0..60 {g.tick(1.0/60.0,Vec3::ZERO,Vec3::Z*5.0,0.0,0.6,true,false);}
        assert!(g.blend>0.99);
        for _ in 0..60 {g.tick(1.0/60.0,Vec3::ZERO,Vec3::ZERO,0.0,0.6,true,false);}
        assert!(g.foot(false,0.6).0.length()<0.0001);
        g.tick(0.016,Vec3::X*20.0,Vec3::ZERO,2.0,0.6,false,false);
        assert_eq!(g.yaw,2.0);assert_eq!(g.blend,0.0);
        g.tick(0.016,Vec3::X*20.0,Vec3::ZERO,-2.0,0.6,false,true);
        assert_eq!(g.yaw,-2.0);
    }
}
