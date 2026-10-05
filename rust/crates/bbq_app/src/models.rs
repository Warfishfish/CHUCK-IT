//! Builds drawable models out of the shape lists in `bbq_core::looks`, and remembers them so
//! the same teddy mesh is not made twice.

use std::collections::HashMap;

use bbq_core::items::{DildoVariant, ItemKind};
use bbq_core::looks::{Part, Surface, Tex};
use bevy::image::{
    ImageAddressMode, ImageFilterMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor,
};
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

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

/// The roughness that makes a matt surface look most like three.js's Lambert.
pub const MATT_ROUGHNESS: f32 = 0.5;
/// The lawn is seen at a glancing angle, where Bevy lights rough things up; this brings it back.
pub const GROUND_ROUGHNESS: f32 = 0.0;

/// The polished look draws items with their extra details (set once at start-up from `LookMode`).
pub static POLISHED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// What to look up: an item (and which size or colour), or a one-off by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModelKey {
    Item(ItemKind, Option<DildoVariant>, u32),
    /// The crack on the Bum-Out gnome (not drawn yet).
    #[allow(dead_code)]
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
            ModelKey::Item(kind, v, n) => bbq_core::looks::item_look(kind, n, v, POLISHED.load(std::sync::atomic::Ordering::Relaxed)),
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
    /// Throws a shadow.
    pub casts: bool,
    /// Gets shadows on it (see-through water and shimmer do not).
    pub receives: bool,
    /// Which segment of a floppy chain this rides on, and where that segment hinges.
    pub seg: Option<(u8, Vec3)>,
}

/// One link of a floppy item's chain: the item bends by turning these.
#[derive(Component)]
pub struct FloppySeg;

/// The links of a floppy item (dildo, noodle) and how it bends.
#[derive(Component)]
pub struct FloppyChain {
    pub segs: Vec<Entity>,
    pub def: bbq_core::looks::Floppy,
}

impl Built {
    /// Spawn this part as a child of `parent`.
    pub fn spawn_under(&self, commands: &mut Commands, parent: Entity) -> Entity {
        self.spawn_under_shifted(commands, parent, Vec3::ZERO)
    }

    /// Spawn as a child of `parent`, moved by `-shift` (the parent's own place in the model).
    pub fn spawn_under_shifted(&self, commands: &mut Commands, parent: Entity, shift: Vec3) -> Entity {
        let mut tf = self.transform;
        tf.translation -= shift;
        let mut e = commands.spawn((
            Mesh3d(self.mesh.clone()),
            MeshMaterial3d(self.material.clone()),
            tf,
        ));
        if !self.casts {
            e.insert(NotShadowCaster);
        }
        if !self.receives {
            e.insert(NotShadowReceiver);
        }
        let id = e.id();
        commands.entity(parent).add_child(id);
        id
    }
}

/// Make one finished part from a part description.
fn build_part(
    p: &Part,
    cache: &ModelCache,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
) -> Built {
    let s = &p.surface;
    Built {
        mesh: meshes.add(crate::shapes::build_mesh_uv(&p.shape, s.uv_per_m)),
        material: mats.add(material(s, cache)),
        transform: transform_of(p),
        casts: !s.no_shadow,
        receives: !(s.no_shadow && (s.additive || s.alpha < 1.0)),
        seg: p.seg.map(|(i, v)| (i, Vec3::new(v.x, v.y, v.z))),
    }
}

#[derive(Resource, Default)]
pub struct ModelCache {
    textures: HashMap<Tex, Handle<Image>>,
    built: HashMap<ModelKey, Vec<Built>>,
}

pub struct ModelsPlugin;

impl Plugin for ModelsPlugin {
    fn build(&self, app: &mut App) {
        let polished = app
            .world()
            .get_resource::<crate::lighting::LookMode>()
            .is_some_and(|l| *l == crate::lighting::LookMode::Polished);
        POLISHED.store(polished, std::sync::atomic::Ordering::Relaxed);
        app.init_resource::<ModelCache>()
            .add_systems(Startup, load_textures)
            .add_systems(Update, add_mipmaps);
    }
}

fn load_textures(mut cache: ResMut<ModelCache>, assets: Res<AssetServer>) {
    // Pictures are used as they are, with no sRGB step, like the browser game does. They repeat
    // (fences, tiles) and blend smoothly at a distance, as three.js does.
    let load = |s: &mut ImageLoaderSettings| {
        s.is_srgb = false;
        s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            mag_filter: ImageFilterMode::Linear,
            min_filter: ImageFilterMode::Linear,
            mipmap_filter: ImageFilterMode::Linear,
            anisotropy_clamp: 8,
            ..default()
        });
    };
    for t in Tex::ALL {
        let h = assets
            .load_builder()
            .with_settings(load)
            .load(format!("textures/{}", t.file()));
        cache.textures.insert(t, h);
    }
}

/// How many levels of smaller pictures a texture of this size has.
pub fn mip_count(w: u32, h: u32) -> u32 {
    32 - w.max(h).max(1).leading_zeros()
}

/// Make the chain of smaller copies of an RGBA picture (each half the size, averaged, with
/// transparent pixels counting for less), laid out one after another as the GPU wants them.
/// Bevy does not do this by itself, and without it far-away fences and lawn shimmer.
pub fn build_mips(w: u32, h: u32, base: &[u8]) -> Vec<u8> {
    let mut out = base.to_vec();
    let (mut cw, mut ch) = (w as usize, h as usize);
    let mut prev = base.to_vec();
    while cw > 1 || ch > 1 {
        let (nw, nh) = ((cw / 2).max(1), (ch / 2).max(1));
        let mut next = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                let (mut r, mut g, mut b, mut a) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
                let mut n = 0.0;
                for dy in 0..(ch / nh).max(1) {
                    for dx in 0..(cw / nw).max(1) {
                        let i = (((y * (ch / nh) + dy).min(ch - 1)) * cw
                            + (x * (cw / nw) + dx).min(cw - 1))
                            * 4;
                        let al = prev[i + 3] as f32;
                        r += prev[i] as f32 * al;
                        g += prev[i + 1] as f32 * al;
                        b += prev[i + 2] as f32 * al;
                        a += al;
                        n += 1.0;
                    }
                }
                let o = (y * nw + x) * 4;
                if a > 0.0 {
                    next[o] = (r / a).round() as u8;
                    next[o + 1] = (g / a).round() as u8;
                    next[o + 2] = (b / a).round() as u8;
                }
                next[o + 3] = (a / n).round() as u8;
            }
        }
        out.extend_from_slice(&next);
        prev = next;
        cw = nw;
        ch = nh;
    }
    out
}

/// When one of our pictures finishes loading, give it its smaller copies.
fn add_mipmaps(mut events: MessageReader<AssetEvent<Image>>, mut images: ResMut<Assets<Image>>) {
    for ev in events.read() {
        let AssetEvent::LoadedWithDependencies { id } = ev else {
            continue;
        };
        let Some(mut img) = images.get_mut(*id) else {
            continue;
        };
        let (w, h) = (img.width(), img.height());
        if img.texture_descriptor.mip_level_count > 1
            || img.texture_descriptor.format != TextureFormat::Rgba8Unorm
        {
            continue;
        }
        let Some(data) = img.data.as_ref() else {
            continue;
        };
        if data.len() != (w * h * 4) as usize || (w <= 1 && h <= 1) {
            continue;
        }
        let mips = build_mips(w, h, data);
        img.texture_descriptor.mip_level_count = mip_count(w, h);
        img.data = Some(mips);
    }
}

/// Turn a browser-style surface into a Bevy material. A matt "Lambert" surface is fully rough;
/// a shiny "Phong" one gets a tighter highlight the higher its shininess.
pub fn material(s: &Surface, cache: &ModelCache) -> StandardMaterial {
    // Bevy's diffuse light is brighter at glancing angles the rougher a surface is, which three's
    // flat "Lambert" never is. A roughness of about 0.7 keeps it close to Lambert (see compare.md).
    let rough = if s.shine > 0.0 {
        (2.0 / (s.shine + 2.0)).sqrt()
    } else if s.ground {
        GROUND_ROUGHNESS
    } else {
        MATT_ROUGHNESS
    };
    let emissive = hex(s.emissive).to_linear();
    StandardMaterial {
        base_color: hex(s.color).with_alpha(s.alpha),
        base_color_texture: s.tex.and_then(|t| cache.textures.get(&t).cloned()),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(s.repeat.0, s.repeat.1)),
        emissive,
        perceptual_roughness: rough,
        reflectance: if s.shine > 0.0 { 0.5 } else { 0.0 },
        unlit: s.unlit,
        alpha_mode: if s.additive {
            AlphaMode::Add
        } else if s.alpha < 1.0 {
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
            .map(|p| build_part(p, self, meshes, mats))
            .collect();
        self.built.insert(key, built.clone());
        built
    }

    /// Spawn a list of parts as a group (an empty parent with one child per part).
    pub fn spawn_parts(
        &mut self,
        commands: &mut Commands,
        parts: &[Part],
        meshes: &mut Assets<Mesh>,
        mats: &mut Assets<StandardMaterial>,
        at: Transform,
    ) -> Entity {
        self.spawn_parts_with_children(commands, parts, meshes, mats, at)
            .0
    }

    /// Like `spawn_parts`, but also hands back each part's own entity (in the order of `parts`),
    /// so a few of them can be animated.
    pub fn spawn_parts_with_children(
        &mut self,
        commands: &mut Commands,
        parts: &[Part],
        meshes: &mut Assets<Mesh>,
        mats: &mut Assets<StandardMaterial>,
        at: Transform,
    ) -> (Entity, Vec<Entity>) {
        let root = commands.spawn((at, Visibility::default())).id();
        let kids: Vec<Entity> = parts
            .iter()
            .map(|p| build_part(p, self, meshes, mats).spawn_under(commands, root))
            .collect();
        (root, kids)
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
        let root = commands.spawn((at, Visibility::default())).id();
        let floppy = match key {
            ModelKey::Item(kind, ..) => bbq_core::looks::floppy(kind),
            _ => None,
        };
        let Some(def) = floppy else {
            for b in &built {
                b.spawn_under(commands, root);
            }
            return root;
        };
        // a chain of links, each hinged on the one before: parts ride on their own link
        let mut pivots: Vec<Vec3> = Vec::new();
        for b in &built {
            if let Some((i, pv)) = b.seg {
                let i = i as usize;
                if pivots.len() <= i {
                    pivots.resize(i + 1, pv);
                }
                pivots[i] = pv;
            }
        }
        let mut segs: Vec<Entity> = Vec::new();
        for (i, pv) in pivots.iter().enumerate() {
            let (parent, rel) = if i == 0 { (root, *pv) } else { (segs[i - 1], *pv - pivots[i - 1]) };
            let e = commands
                .spawn((Transform::from_translation(rel), Visibility::default(), FloppySeg))
                .id();
            commands.entity(parent).add_child(e);
            segs.push(e);
        }
        for b in &built {
            match b.seg {
                Some((i, pv)) => {
                    b.spawn_under_shifted(commands, segs[i as usize], pv);
                }
                None => {
                    b.spawn_under(commands, root);
                }
            }
        }
        commands.entity(root).insert(FloppyChain { segs, def });
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_go_in_as_linear_numbers() {
        let c = hex(0xff8000).to_linear();
        assert!(
            (c.red - 1.0).abs() < 1e-5 && (c.green - 128.0 / 255.0).abs() < 1e-5 && c.blue == 0.0
        );
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
    fn mip_chains_cover_every_size_down_to_one_pixel() {
        assert_eq!(mip_count(256, 256), 9);
        assert_eq!(mip_count(256, 64), 9);
        assert_eq!(mip_count(1, 1), 1);
        assert_eq!(mip_count(64, 32), 7);
        // 4x2 -> 2x1 -> 1x1
        let base = vec![255u8; 4 * 2 * 4];
        let all = build_mips(4, 2, &base);
        assert_eq!(all.len(), (4 * 2 + 2 + 1) * 4);
        assert!(all.iter().all(|v| *v == 255));
    }

    #[test]
    fn mips_average_colours_and_respect_transparency() {
        // two pixels side by side: white solid and black solid -> grey
        let base = [255, 255, 255, 255, 0, 0, 0, 255];
        let m = build_mips(2, 1, &base);
        assert_eq!(&m[8..12], &[128, 128, 128, 255]);
        // a transparent pixel does not drag the colour towards black
        let base = [255, 255, 255, 255, 0, 0, 0, 0];
        let m = build_mips(2, 1, &base);
        assert_eq!(&m[8..12], &[255, 255, 255, 128]);
    }

    #[test]
    fn shiny_surfaces_are_less_rough_than_matt() {
        let cache = ModelCache::default();
        let matt = material(&Surface::matt(0xffffff), &cache);
        let shiny = material(&Surface::shiny(0xffffff, 0xffffff, 70.0), &cache);
        assert_eq!(matt.perceptual_roughness, MATT_ROUGHNESS);
        assert!(shiny.perceptual_roughness < 0.2);
        assert!(!matt.double_sided);
    }

    #[test]
    fn fins_are_see_through_and_two_sided() {
        let cache = ModelCache::default();
        let fin = Surface::shiny(0x7d9fb2, 0xaaccdd, 40.0)
            .both_sides()
            .see_through(0.88);
        let m = material(&fin, &cache);
        assert!(m.double_sided);
        assert_eq!(m.cull_mode, None);
        assert!(matches!(m.alpha_mode, AlphaMode::Blend));
    }
}
