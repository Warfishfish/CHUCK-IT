//! Builds drawable models out of the shape lists in `bbq_core::looks`, and remembers them so
//! the same teddy mesh is not made twice.

use std::collections::HashMap;

use bbq_core::items::{DildoVariant, ItemKind};
use bbq_core::looks::{Part, Surface, Tex};
use bevy::prelude::*;

use crate::shapes::build_mesh;

/// `0xRRGGBB` as a colour. The browser game does its lighting maths straight on these numbers,
/// so they go in as *linear* values (see `lighting`), not as sRGB.
pub fn hex(c: u32) -> Color {
    Color::linear_rgb(
        ((c >> 16) & 0xff) as f32 / 255.0,
        ((c >> 8) & 0xff) as f32 / 255.0,
        (c & 0xff) as f32 / 255.0,
    )
}

/// The same colour as plain sRGB, for things drawn without lighting (the sky behind everything).
pub fn hex_srgb(c: u32) -> Color {
    Color::srgb_u8(((c >> 16) & 0xff) as u8, ((c >> 8) & 0xff) as u8, (c & 0xff) as u8)
}

/// What to look up: an item (and which size or colour), or a one-off by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelKey {
    Item(ItemKind, Option<DildoVariant>, u32),
    BumCrack,
}

impl ModelKey {
    pub fn item(kind: ItemKind, variant: Option<DildoVariant>, id: u32) -> Self {
        // only the noodle's colour and the dildo's size change the look
        let v = if kind == ItemKind::Dildo {
            Some(variant.unwrap_or(DildoVariant::Classic))
        } else {
            None
        };
        let n = if kind == ItemKind::Noodle { id % 4 } else { 0 };
        ModelKey::Item(kind, v, n)
    }

    fn parts(self) -> Vec<Part> {
        match self {
            ModelKey::Item(kind, v, n) => bbq_core::looks::item(kind, n, v),
            ModelKey::BumCrack => bbq_core::looks::bum_crack(),
        }
    }
}

/// One finished part: its mesh, material and place.
#[derive(Clone)]
pub struct Built {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
}

#[derive(Resource, Default)]
pub struct ModelCache {
    vp_label: Handle<Image>,
    fish: Handle<Image>,
    built: HashMap<ModelKey, Vec<Built>>,
}

pub struct ModelsPlugin;

impl Plugin for ModelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModelCache>().add_systems(Startup, load_textures);
    }
}

fn load_textures(mut cache: ResMut<ModelCache>, assets: Res<AssetServer>) {
    // Pictures are used as they are, with no sRGB step, like the browser game does.
    let raw = |s: &mut bevy::image::ImageLoaderSettings| s.is_srgb = false;
    cache.vp_label = assets.load_builder().with_settings(raw).load("textures/vp_label.png");
    cache.fish = assets.load_builder().with_settings(raw).load("textures/fish.png");
}

/// Turn a browser-style surface into a Bevy material. A matt "Lambert" surface is fully rough;
/// a shiny "Phong" one gets a tighter highlight the higher its shininess.
pub fn material(s: &Surface, cache: &ModelCache) -> StandardMaterial {
    let rough = if s.shine > 0.0 {
        (2.0 / (s.shine + 2.0)).sqrt()
    } else {
        1.0
    };
    let emissive = hex(s.emissive).to_linear();
    StandardMaterial {
        base_color: hex(s.color).with_alpha(s.alpha),
        base_color_texture: s.tex.map(|t| match t {
            Tex::VpLabel => cache.vp_label.clone(),
            Tex::Fish => cache.fish.clone(),
        }),
        emissive,
        perceptual_roughness: rough,
        reflectance: if s.shine > 0.0 { 0.5 } else { 0.0 },
        alpha_mode: if s.alpha < 1.0 {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        },
        double_sided: s.double_sided,
        cull_mode: if s.double_sided {
            None
        } else {
            Some(bevy::render::render_resource::Face::Back)
        },
        ..default()
    }
}

pub fn transform_of(p: &Part) -> Transform {
    Transform {
        translation: Vec3::new(p.pos.x, p.pos.y, p.pos.z),
        rotation: Quat::from_xyzw(p.rot.x, p.rot.y, p.rot.z, p.rot.w),
        scale: Vec3::new(p.scale.x, p.scale.y, p.scale.z),
    }
}

impl ModelCache {
    /// The finished parts of a model, made on first use.
    pub fn built(
        &mut self,
        key: ModelKey,
        meshes: &mut Assets<Mesh>,
        mats: &mut Assets<StandardMaterial>,
    ) -> Vec<Built> {
        if let Some(b) = self.built.get(&key) {
            return b.clone();
        }
        let parts = key.parts();
        let built: Vec<Built> = parts
            .iter()
            .map(|p| Built {
                mesh: meshes.add(build_mesh(&p.shape)),
                material: mats.add(material(&p.surface, self)),
                transform: transform_of(p),
            })
            .collect();
        self.built.insert(key, built.clone());
        built
    }

    /// Spawn a model as a group (an empty parent with one child per part). Put the returned
    /// entity where you want the model.
    pub fn spawn(
        &mut self,
        commands: &mut Commands,
        key: ModelKey,
        meshes: &mut Assets<Mesh>,
        mats: &mut Assets<StandardMaterial>,
        at: Transform,
    ) -> Entity {
        let built = self.built(key, meshes, mats);
        commands
            .spawn((at, Visibility::default()))
            .with_children(|p| {
                for b in built {
                    p.spawn((
                        Mesh3d(b.mesh),
                        MeshMaterial3d(b.material),
                        b.transform,
                    ));
                }
            })
            .id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_go_in_as_linear_numbers() {
        let c = hex(0xff8000).to_linear();
        assert!((c.red - 1.0).abs() < 1e-5 && (c.green - 128.0 / 255.0).abs() < 1e-5 && c.blue == 0.0);
        // the sky is the one thing drawn as a proper sRGB colour
        let s = hex_srgb(0x9fd8f2).to_srgba();
        assert!((s.red - 159.0 / 255.0).abs() < 1e-3);
    }

    #[test]
    fn only_looks_that_differ_get_their_own_key() {
        let a = ModelKey::item(ItemKind::Teddy, None, 7);
        let b = ModelKey::item(ItemKind::Teddy, Some(DildoVariant::Gold), 99);
        assert_eq!(a, b);
        assert_ne!(
            ModelKey::item(ItemKind::Noodle, None, 1),
            ModelKey::item(ItemKind::Noodle, None, 2)
        );
        assert_eq!(
            ModelKey::item(ItemKind::Noodle, None, 1),
            ModelKey::item(ItemKind::Noodle, None, 5)
        );
        assert_ne!(
            ModelKey::item(ItemKind::Dildo, Some(DildoVariant::Mini), 0),
            ModelKey::item(ItemKind::Dildo, None, 0)
        );
    }

    #[test]
    fn shiny_surfaces_are_less_rough_than_matt() {
        let cache = ModelCache::default();
        let matt = material(&Surface::matt(0xffffff), &cache);
        let shiny = material(&Surface::shiny(0xffffff, 0xffffff, 70.0), &cache);
        assert_eq!(matt.perceptual_roughness, 1.0);
        assert!(shiny.perceptual_roughness < 0.2);
        assert!(!matt.double_sided);
    }

    #[test]
    fn fins_are_see_through_and_two_sided() {
        let cache = ModelCache::default();
        let fin = Surface::shiny(0x7d9fb2, 0xaaccdd, 40.0).both_sides().see_through(0.88);
        let m = material(&fin, &cache);
        assert!(m.double_sided);
        assert_eq!(m.cull_mode, None);
        assert!(matches!(m.alpha_mode, AlphaMode::Blend));
    }
}
