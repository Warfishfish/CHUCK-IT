//! Gamepad sticks: turning a raw stick reading into movement and looking. Pure numbers, so it can
//! be tested without a controller. The buttons are mapped in `bbq_app/src/gamepad.rs`.

use crate::movement::MOUSE_SENS;

/// Stick readings smaller than this (as a share of full tilt) are ignored: sticks never rest at
/// exactly zero.
pub const DEADZONE: f32 = 0.18;
/// How fast the right stick turns you at full tilt, radians per second (sideways / up and down).
pub const LOOK_YAW_RATE: f32 = 3.2;
pub const LOOK_PITCH_RATE: f32 = 2.2;

/// Remove the dead zone round the middle and rescale, so a stick moves smoothly away from zero
/// instead of jumping, and gives a stick of length 1 at full tilt.
pub fn deadzone(x: f32, y: f32) -> (f32, f32) {
    let l = x.hypot(y);
    if l <= DEADZONE || !l.is_finite() {
        return (0.0, 0.0);
    }
    let k = ((l - DEADZONE) / (1.0 - DEADZONE)).min(1.0) / l;
    (x * k, y * k)
}

/// The walking stick: dead zone, then a gentle curve so small tilts walk slowly and a full tilt
/// runs (the keys are all-or-nothing, a stick is not).
pub fn walk_stick(x: f32, y: f32) -> (f32, f32) {
    let (x, y) = deadzone(x, y);
    let l = x.hypot(y);
    if l < 1e-6 {
        return (0.0, 0.0);
    }
    let curved = l * l * (3.0 - 2.0 * l); // smoothstep
    (x / l * curved, y / l * curved)
}

/// The looking stick, as the same "pixels" the mouse gives (so `movement::look` can be used as it
/// is). Right on the stick turns right; up looks up. A little extra speed for a big tilt, so a
/// small tilt aims finely and a full tilt turns quickly.
pub fn look_pixels(x: f32, y: f32, dt: f32) -> (f32, f32) {
    let (x, y) = deadzone(x, y);
    let boost = |v: f32| v * (0.55 + 0.45 * v.abs());
    (
        boost(x) * LOOK_YAW_RATE * dt / MOUSE_SENS,
        -boost(y) * LOOK_PITCH_RATE * dt / MOUSE_SENS,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement;

    #[test]
    fn a_resting_stick_does_nothing() {
        assert_eq!(deadzone(0.1, -0.1), (0.0, 0.0));
        assert_eq!(walk_stick(0.05, 0.1), (0.0, 0.0));
        assert_eq!(look_pixels(0.1, 0.05, 0.016), (0.0, 0.0));
        assert_eq!(deadzone(f32::NAN, 0.0), (0.0, 0.0));
    }

    #[test]
    fn a_full_stick_is_full_speed_and_never_more() {
        let (x, y) = walk_stick(1.0, 0.0);
        assert!((x - 1.0).abs() < 1e-5 && y == 0.0);
        let (x, y) = walk_stick(1.0, 1.0); // a square gate's corner reads 1.41
        assert!((x.hypot(y) - 1.0).abs() < 1e-4);
        let (x, y) = deadzone(0.0, -1.0);
        assert!(x == 0.0 && (y + 1.0).abs() < 1e-5);
    }

    #[test]
    fn walking_speeds_up_smoothly_with_the_tilt() {
        let mut last = 0.0;
        for i in 20..=100 {
            let (x, _) = walk_stick(i as f32 / 100.0, 0.0);
            assert!(x >= last, "never slows as you tilt further");
            assert!(x - last < 0.06, "no jump at {i}");
            last = x;
        }
        assert!(walk_stick(0.5, 0.0).0 < 0.5, "half a tilt walks slower than half speed");
    }

    #[test]
    fn the_look_stick_turns_the_right_way_at_the_right_rate() {
        // right and up
        let (dx, dy) = look_pixels(1.0, 1.0, 1.0 / 60.0);
        let (yaw, pitch) = movement::look(0.0, 0.0, dx, dy);
        assert!(yaw < 0.0, "right turns right (yaw falls, as with the mouse)");
        assert!(pitch > 0.0, "up looks up");
        // a full second of full tilt turns the full rate
        let mut yaw = 0.0;
        for _ in 0..60 {
            let (dx, dy) = look_pixels(1.0, 0.0, 1.0 / 60.0);
            yaw = movement::look(yaw, 0.0, dx, dy).0;
        }
        assert!((yaw + LOOK_YAW_RATE).abs() < 0.01, "{yaw}");
        // a small tilt turns slowly
        let (dx, _) = look_pixels(0.4, 0.0, 1.0 / 60.0);
        let (full, _) = look_pixels(1.0, 0.0, 1.0 / 60.0);
        assert!(dx.abs() < full.abs() * 0.3);
    }
}
