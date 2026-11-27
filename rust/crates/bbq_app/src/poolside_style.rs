//! Local art pass for the Backyard pool, cart and Gumdrop.
use crate::lighting::LookMode;
use bevy::prelude::*;
use std::sync::OnceLock;

pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        let args: Vec<_> = std::env::args().collect();
        !(args.iter().any(|a| a == "--review") && args.iter().any(|a| a == "--poolside-baseline"))
    })
}

pub fn active(look: LookMode, mode: bbq_core::GameMode) -> bool {
    enabled()
        && look == LookMode::Polished
        && crate::style::current() == crate::style::Style::Pop
        && crate::wheelbarrow::available(mode)
}

/// Feather the grass treatment out from the pool apron toward the normal lawn.
pub fn apron_weight(x: f32, z: f32) -> f32 {
    let distance = ((x + 15.) / 13.).hypot((z - 10.) / 7.5);
    let u = (1. - distance).clamp(0., 1.);
    u * u * (3. - 2. * u)
}

pub fn satin(colour: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: colour,
        perceptual_roughness: 0.62,
        reflectance: 0.28,
        clearcoat: 0.12,
        clearcoat_perceptual_roughness: 0.56,
        ..default()
    }
}

#[derive(Default)]
pub struct SeatSway {
    rider: Option<bbq_core::PlayerId>,
    heading: f32,
    roll: f32,
}

impl SeatSway {
    pub fn update(&mut self, rider: Option<bbq_core::PlayerId>, heading: f32, dt: f32) -> f32 {
        if rider.is_none() || rider != self.rider {
            *self = Self {
                rider,
                heading,
                roll: 0.,
            };
            return 0.;
        }
        use std::f32::consts::{PI, TAU};
        let delta = (heading - self.heading + PI).rem_euclid(TAU) - PI;
        self.heading = heading;
        let pace = (delta / dt.max(0.001)).clamp(-3., 3.);
        let target = -0.024 * pace;
        let blend = 1. - (-dt * 7.).exp();
        self.roll += (target - self.roll) * blend;
        self.roll
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seat_sway_wraps_angles_and_settles_without_changing_riders() {
        let mut sway = SeatSway::default();
        assert_eq!(sway.update(Some(3), 3.14, 0.016), 0.);
        let turning = sway.update(Some(3), -3.12, 0.016);
        assert!(turning < 0. && turning > -0.007);
        for _ in 0..80 {
            sway.update(Some(3), -3.12, 0.016);
        }
        assert!(sway.roll.abs() < 0.0001);
        assert_eq!(sway.update(Some(4), -1.6, 0.016), 0.);
    }
}
