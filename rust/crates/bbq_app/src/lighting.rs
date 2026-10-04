//! Makes the Rust game light things the way the browser game's three.js did.
//!
//! The browser game (three.js r149) does its colour maths straight on the numbers in the hex
//! codes, with no sRGB step, and its lights are plain multipliers. Bevy works in linear light
//! and encodes to sRGB at the end. To get the same picture we:
//!   * feed hex colours in as *linear* values (`models::hex`),
//!   * turn Bevy's tone mapping off and undo the sRGB encoding with a small full-screen shader
//!     (`assets/shaders/display_raw.wgsl`, see `DisplayRaw`),
//!   * give lights an illuminance that works out to the same plain multiplier (`lux`).
//!
//! Three's hemisphere light has no Bevy twin, so it is a flat ambient light plus a soft light
//! from straight above (`hemisphere`).

use bevy::camera::Exposure;
use bevy::core_pipeline::fullscreen_material::FullscreenMaterial;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

use crate::models::hex;

/// The browser game's lights.
pub const SKY: u32 = 0xe6f6ff;
pub const GROUND: u32 = 0x4f6e30;
pub const HEMI_INTENSITY: f32 = 0.62;
pub const SUN: u32 = 0xfff0d2;
pub const SUN_INTENSITY: f32 = 0.88;
/// Where the sun is.
pub const SUN_AT: Vec3 = Vec3::new(24.0, 40.0, 20.0);

/// Bevy's lighting comes out a little different from three's for the same numbers. These
/// were measured with the white-ball test (`--calib`, `rust/tools/fit_light.py`) and bring
/// each light to what the browser game shows.
pub const AMBIENT_FIX: f32 = 2.22;
pub const UP_FIX: f32 = 1.03;
pub const SUN_FIX: f32 = 1.0;

/// What one unit of "browser light" is worth in Bevy's own units (the camera's exposure undone).
pub fn unit() -> f32 {
    1.0 / Exposure::default().exposure()
}

/// Illuminance for a directional light that gives the same plain `colour * n.l * intensity`
/// as a three.js directional light of that intensity.
pub fn lux(intensity: f32) -> f32 {
    intensity * std::f32::consts::PI * unit()
}

/// Undoes Bevy's final sRGB step, so the colour numbers come out as the browser game shows them.
/// Put it on the camera and add `FullscreenMaterialPlugin::<DisplayRaw>`.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
pub struct DisplayRaw {
    /// 1 = on, 0 = off (pass the picture through untouched).
    pub on: f32,
}

impl DisplayRaw {
    pub fn on() -> Self {
        DisplayRaw { on: 1.0 }
    }
}

impl FullscreenMaterial for DisplayRaw {
    fn fragment_shader() -> ShaderRef {
        "shaders/display_raw.wgsl".into()
    }
}

/// The camera parts that go with `DisplayRaw`. Not HDR on purpose: three clamps each colour to
/// 0..1 before see-through things are blended, and an ordinary 8-bit picture does the same.
pub fn camera_style() -> (Tonemapping, Exposure, DisplayRaw) {
    (Tonemapping::None, Exposure::default(), DisplayRaw::on())
}

fn channels(c: u32) -> [f32; 3] {
    [
        ((c >> 16) & 0xff) as f32 / 255.0,
        ((c >> 8) & 0xff) as f32 / 255.0,
        (c & 0xff) as f32 / 255.0,
    ]
}

/// Three's hemisphere light, as an ambient light for the camera plus a light from above.
/// For a surface facing along `n` three gives `ground + (sky - ground) * (0.5 + 0.5 * n.y)`.
/// We give the average everywhere and add half the difference from above, which is exact for
/// anything facing up or sideways (undersides come out a little bright).
pub fn hemisphere() -> (AmbientLight, DirectionalLight, Transform) {
    let (s, g) = (channels(SKY), channels(GROUND));
    let avg = [0, 1, 2].map(|i| (s[i] + g[i]) / 2.0 * HEMI_INTENSITY);
    let up = [0, 1, 2].map(|i| (s[i] - g[i]) / 2.0 * HEMI_INTENSITY);
    let peak = up.iter().cloned().fold(0.0, f32::max);
    (
        AmbientLight {
            color: Color::linear_rgb(
                avg[0] / avg[0].max(avg[1]).max(avg[2]),
                avg[1] / avg[0].max(avg[1]).max(avg[2]),
                avg[2] / avg[0].max(avg[1]).max(avg[2]),
            ),
            brightness: avg[0].max(avg[1]).max(avg[2]) * unit() * AMBIENT_FIX,
            ..default()
        },
        DirectionalLight {
            color: Color::linear_rgb(up[0] / peak, up[1] / peak, up[2] / peak),
            illuminance: lux(peak) * UP_FIX,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::ZERO, Vec3::Z),
    )
}

/// The sun, with shadows.
pub fn sun() -> (DirectionalLight, Transform) {
    (
        DirectionalLight {
            color: hex(SUN),
            illuminance: lux(SUN_INTENSITY) * SUN_FIX,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_translation(SUN_AT).looking_at(Vec3::ZERO, Vec3::Y),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lux_is_proportional_to_the_browser_intensity() {
        assert!((lux(0.88) / lux(0.44) - 2.0).abs() < 1e-5);
        assert!(lux(0.88) > 1000.0);
    }

    #[test]
    fn hemisphere_averages_sky_and_ground() {
        let (amb, up, _) = hemisphere();
        // green is the brightest channel of the pair: (0xf6 + 0x6e) / 2, at 62%
        let green = ((0xf6 as f32 + 0x6e as f32) / 2.0 / 255.0) * HEMI_INTENSITY;
        let brightest = amb.brightness / unit() / AMBIENT_FIX;
        assert!((brightest - green).abs() < 1e-4, "{brightest} vs {green}");
        assert!(!up.shadow_maps_enabled);
    }

    #[test]
    fn sun_matches_the_browser_numbers() {
        assert_eq!(SUN_AT, Vec3::new(24.0, 40.0, 20.0));
        assert_eq!(SUN, 0xfff0d2);
        assert_eq!(SUN_INTENSITY, 0.88);
    }
}
