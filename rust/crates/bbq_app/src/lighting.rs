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

/// Which look the game has: the browser game's exact lights (for the comparison toolkit), or the
/// sun-baked, warmer-and-moodier look of step 2c (`GRAPHICS_2C.md`).
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LookMode {
    Browser,
    Polished,
}

impl LookMode {
    /// `--look browser|polished`. Screenshots default to `browser` so old comparisons still
    /// work; playing defaults to `polished`.
    pub fn from_args(args: &[String]) -> LookMode {
        let asked = args
            .iter()
            .position(|a| a == "--look")
            .and_then(|i| args.get(i + 1))
            .map(String::as_str);
        match asked {
            Some("browser") => LookMode::Browser,
            Some("polished") => LookMode::Polished,
            _ if args.iter().any(|a| a == "--shot") => LookMode::Browser,
            _ => LookMode::Polished,
        }
    }

    /// The lights for this look.
    pub fn rig(self) -> Rig {
        match self {
            LookMode::Browser => Rig {
                sky: SKY,
                ground: GROUND,
                hemi: HEMI_INTENSITY,
                sun: SUN,
                sun_intensity: SUN_INTENSITY,
                grade: Grade::NONE,
            },
            // warm hard sun, cooler and a little dimmer sky fill, dry dusty ground bounce
            LookMode::Polished => match crate::style::current() {
                // pop: vivid colours with clay's strong, low, golden sun, cool shade and warm bounce
                crate::style::Style::Pop => Rig {
                    sky: 0x9db4e0,
                    ground: 0xb8864c,
                    hemi: 0.38,
                    sun: 0xffb868,
                    sun_intensity: 1.42,
                    grade: Grade { saturation: 1.2, warmth: 0.10, contrast: 1.12, vignette: 0.32 },
                },
                // clay: a strong, low, golden sun with a cool blue shade and warm bounce light
                crate::style::Style::Clay => Rig {
                    sky: 0x9db4e0,
                    ground: 0xb8864c,
                    hemi: 0.36,
                    sun: 0xffb260,
                    sun_intensity: 1.5,
                    grade: Grade { saturation: 1.0, warmth: 0.13, contrast: 1.14, vignette: 0.45 },
                },
                crate::style::Style::Current => Rig {
                sky: 0xbfd6f2,
                ground: 0x8a7a48,
                hemi: 0.50,
                sun: 0xffe3b0,
                sun_intensity: 1.0,
                grade: Grade {
                    saturation: 1.08,
                    warmth: 0.05,
                    contrast: 1.07,
                    vignette: 0.28,
                },
                },
            },
        }
    }
}

/// Everything that differs between the looks.
#[derive(Clone, Copy, Debug)]
pub struct Rig {
    pub sky: u32,
    pub ground: u32,
    pub hemi: f32,
    pub sun: u32,
    pub sun_intensity: f32,
    pub grade: Grade,
}

/// The finishing touches the display shader puts on the picture.
#[derive(Clone, Copy, Debug)]
pub struct Grade {
    pub saturation: f32,
    pub warmth: f32,
    pub contrast: f32,
    pub vignette: f32,
}

impl Grade {
    pub const NONE: Grade = Grade {
        saturation: 1.0,
        warmth: 0.0,
        contrast: 1.0,
        vignette: 0.0,
    };
}

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
    pub saturation: f32,
    /// Warm (positive) or cool tint, around 0.05.
    pub warmth: f32,
    pub contrast: f32,
    /// How much the corners darken, 0 to 1.
    pub vignette: f32,
}

impl DisplayRaw {
    pub fn on() -> Self {
        Self::with_grade(Grade::NONE)
    }

    pub fn with_grade(g: Grade) -> Self {
        DisplayRaw {
            on: 1.0,
            saturation: g.saturation,
            warmth: g.warmth,
            contrast: g.contrast,
            vignette: g.vignette,
        }
    }
}

impl FullscreenMaterial for DisplayRaw {
    fn fragment_shader() -> ShaderRef {
        "shaders/display_raw.wgsl".into()
    }
}

/// The camera parts that go with `DisplayRaw`. Not HDR on purpose: three clamps each colour to
/// 0..1 before see-through things are blended, and an ordinary 8-bit picture does the same.
pub fn camera_style(look: LookMode) -> (Tonemapping, Exposure, DisplayRaw) {
    (
        Tonemapping::None,
        Exposure::default(),
        DisplayRaw::with_grade(look.rig().grade),
    )
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
pub fn hemisphere(look: LookMode) -> (AmbientLight, DirectionalLight, Transform) {
    let r = look.rig();
    let (s, g) = (channels(r.sky), channels(r.ground));
    let avg = [0, 1, 2].map(|i| (s[i] + g[i]) / 2.0 * r.hemi);
    let up = [0, 1, 2].map(|i| (s[i] - g[i]) / 2.0 * r.hemi);
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
pub fn sun(look: LookMode) -> (DirectionalLight, Transform) {
    let r = look.rig();
    (
        DirectionalLight {
            color: hex(r.sun),
            illuminance: lux(r.sun_intensity) * SUN_FIX,
            shadow_maps_enabled: true,
            ..default()
        },
        // clay: a lower sun, so shadows are long and the light rakes across the surfaces
        Transform::from_translation(if crate::style::current() != crate::style::Style::Current {
            Vec3::new(SUN_AT.x, SUN_AT.y * 0.62, SUN_AT.z)
        } else {
            SUN_AT
        })
        .looking_at(Vec3::ZERO, Vec3::Y),
    )
}

/// Whether the polished look's screen effects (ambient occlusion, bloom, SMAA) are on. They are
/// on by default in the polished look; `--fx off` switches them off (to compare, or on a slow
/// computer).
pub fn post_fx_wanted(look: LookMode, args: &[String]) -> bool {
    let off = args
        .iter()
        .position(|a| a == "--fx")
        .and_then(|i| args.get(i + 1))
        .is_some_and(|v| v == "off");
    look == LookMode::Polished && !off
}

/// The screen effects of the polished look (step 2c):
///  * screen-space ambient occlusion: darkens the sky/ambient light in creases and where things
///    meet the ground, so props and blobs sit into the lawn instead of floating on it;
///  * SMAA, because ambient occlusion cannot be used with the 4x multi-sampling.
pub fn post_fx() -> impl Bundle {
    use bevy::anti_alias::smaa::Smaa;
    use bevy::pbr::{ScreenSpaceAmbientOcclusion, ScreenSpaceAmbientOcclusionQualityLevel};
    // No bloom: whenever something shiny caught the sun for a frame its glow spread across the
    // whole sky, which made the sky flicker (Marcus, 5 Oct 2026).
    (
        Msaa::Off,
        Smaa::default(),
        ScreenSpaceAmbientOcclusion {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            constant_object_thickness: 0.25,
        },
    )
}

/// A weak cool light from the opposite side of the sun (polished look): a rim light that
/// separates things from the lawn without flattening the sun's shadows.
pub fn rim_light() -> (DirectionalLight, Transform) {
    (
        DirectionalLight {
            color: hex(0x9cc4ff),
            illuminance: lux(0.20),
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(-24.0, 14.0, -20.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        let (amb, up, _) = hemisphere(LookMode::Browser);
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
