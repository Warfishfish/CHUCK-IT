//! Player-selectable rendering budget. Gameplay and simulation stay identical in both modes.

use bevy::prelude::*;

#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GraphicsQuality {
    Low,
    #[default]
    High,
}

impl GraphicsQuality {
    pub fn from_setting(value: &str) -> Self {
        if value.eq_ignore_ascii_case("low") { Self::Low } else { Self::High }
    }
    pub const fn setting_name(self) -> &'static str {
        match self { Self::Low => "low", Self::High => "high" }
    }
}

pub struct GraphicsQualityPlugin;

impl Plugin for GraphicsQualityPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_quality);
    }
}

fn apply_quality(
    settings: Res<crate::menu::Settings>,
    look: Res<crate::lighting::LookMode>,
    mut previous: Local<Option<GraphicsQuality>>,
    cameras: Query<Entity, With<crate::player::EyeCamera>>,
    mut commands: Commands,
    mut shadows: ResMut<bevy::light::DirectionalLightShadowMap>,
    mut effects: Query<
        (
            &mut Visibility,
            Option<&bevy::pbr::decal::ForwardDecal>,
            Option<&bevy_hanabi::prelude::ParticleEffect>,
        ),
        Or<(
            With<bevy::pbr::decal::ForwardDecal>,
            With<bevy_hanabi::prelude::ParticleEffect>,
        )>,
    >,
) {
    let quality = settings.quality;
    if *previous == Some(quality) { return; }
    *previous = Some(quality);

    let high = quality == GraphicsQuality::High;
    shadows.size = if high { 4096 } else { crate::browser_quality::BALANCED_SHADOW_SIZE };
    for (mut visibility, decal, particle) in &mut effects {
        let is_graphics_effect = decal.is_some() || particle.is_some();
        if !is_graphics_effect { continue; }
        *visibility = if high { Visibility::Inherited } else { Visibility::Hidden };
    }

    let args = std::env::args().collect::<Vec<_>>();
    let comparing_aa = args.iter().any(|arg| arg == "--aa");
    let screen_fx = high && !comparing_aa && crate::lighting::post_fx_wanted(*look, &args);
    for camera in &cameras {
        let mut entity = commands.entity(camera);
        if screen_fx {
            entity.insert((
                bevy::anti_alias::smaa::Smaa::default(),
                bevy::pbr::ScreenSpaceAmbientOcclusion {
                    quality_level: if crate::browser_quality::balanced() {
                        bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::Medium
                    } else {
                        bevy::pbr::ScreenSpaceAmbientOcclusionQualityLevel::High
                    },
                    constant_object_thickness: 0.25,
                },
            ));
        } else {
            entity.remove::<bevy::pbr::ScreenSpaceAmbientOcclusion>();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graphics_quality_setting_is_stable_and_defaults_unknown_values_to_high() {
        assert_eq!(GraphicsQuality::from_setting("low"), GraphicsQuality::Low);
        assert_eq!(GraphicsQuality::from_setting("LOW"), GraphicsQuality::Low);
        assert_eq!(GraphicsQuality::from_setting("high"), GraphicsQuality::High);
        assert_eq!(GraphicsQuality::from_setting("unknown"), GraphicsQuality::High);
        assert_eq!(GraphicsQuality::Low.setting_name(), "low");
        assert_eq!(GraphicsQuality::High.setting_name(), "high");
    }
}
