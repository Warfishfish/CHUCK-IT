//! Names on shirts (Marcus, 9 Oct 2026): everybody's name is printed across the front of their
//! singlet (moved from the back on 10 Oct 2026), so you can tell who is who.
//!
//! Each name is drawn once into a small picture (the game's bold font, rasterised with
//! `ab_glyph`) and put on a thin curved strip that wraps round the chest of the singlet, a hair
//! outside the cloth, and moves with the body. The letters are cut out (no see-through edges to
//! sort). The ink is dark on light singlets and white on dark ones. Yard blobs and the lobby
//! line-up wear them.

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use bbq_core::appearance::Palette;
use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::face::{FaceOwner, HeadNode};
use crate::game::Game;
use crate::menu::Settings;

/// The bold display font the menus use (bundled with the game).
const FONT: &[u8] = include_bytes!("../assets/fonts/BowlbyOne-Regular.ttf");
/// How tall the letters are drawn in the picture (pixels).
const LETTER_PX: f32 = 96.0;
/// Room round the letters in the picture (pixels).
const PAD: u32 = 16;

/// The name printed on a head's body, and the decal that prints it.
#[derive(Component)]
struct Printed {
    name: String,
    ink: u32,
    material: Handle<StandardMaterial>,
    /// The print strip, the singlet node (its belly weights), where the strip sits with no belly,
    /// and how far the cloth under its middle moves for the "Belly" and "Sag" shapes.
    strip: Entity,
    singlet: Entity,
    base: Vec3,
    moves: [Vec3; 2],
}

pub struct ShirtPlugin;

impl Plugin for ShirtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, print_names);
    }
}

/// The name on a blob's shirt: an online player's own name, otherwise the bot's.
fn name_of(owner: FaceOwner, game: &Game, settings: &Settings, online: Option<&crate::online::Online>) -> Option<String> {
    match owner {
        FaceOwner::Blob(i) => {
            let d = game.dummies.get(i)?;
            Some(match &d.remote {
                Some(r) => r.name.clone(),
                None => crate::characters::BLOB_NAMES[i % crate::characters::BLOB_NAMES.len()].to_string(),
            })
        }
        FaceOwner::Lobby(id) => {
            let s = online?.session.as_ref()?;
            s.members.iter().find(|m| m.id == id).map(|m| if m.id == s.my_id { settings.name.clone() } else { m.name.clone() })
        }
        // your own blob in the menu wears your name too
        FaceOwner::Preview => Some(settings.name.clone()),
    }
}

/// The singlet colour a blob wears.
fn singlet_of(owner: FaceOwner, game: &Game, settings: &Settings, online: Option<&crate::online::Online>) -> Option<u32> {
    let look = match owner {
        FaceOwner::Blob(i) => game.dummies.get(i)?.look,
        FaceOwner::Lobby(id) => crate::lobby::lobby_look(online?, settings, id)?.1,
        FaceOwner::Preview => settings.look,
    };
    Some(look.colour(Palette::Singlet))
}

/// Dark ink on a light singlet, white ink on a dark one.
pub fn ink_for(singlet: u32) -> u32 {
    let c = |s: u32| ((singlet >> s) & 0xff) as f32 / 255.0;
    let lum = 0.299 * c(16) + 0.587 * c(8) + 0.114 * c(0);
    if lum > 0.55 { 0x1b1b1b } else { 0xf4f1e6 }
}

/// Draw a name into a white-on-clear picture (the decal's colour is the ink). The letters are a
/// little wonky (Marcus, 10 Oct 2026), like a cheap iron-on: each one tilted, sized and bounced a
/// bit differently, the same way every time for the same name. Returns the picture's pixels
/// (RGBA), its width and its height.
pub fn draw_name(name: &str) -> (Vec<u8>, u32, u32) {
    let font = FontRef::try_from_slice(FONT).expect("the bundled font");
    let text: String = name.trim().to_uppercase().chars().take(14).collect();
    // a small random number generator seeded from the name, so a name always looks the same
    let mut seed = text.bytes().fold(0x9E37_79B9u32, |h, b| (h ^ b as u32).wrapping_mul(0x0100_0193)) | 1;
    let mut rand = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed & 0xffff) as f32 / 65535.0 * 2.0 - 1.0 // -1 .. 1
    };
    // lay the letters out along a line, each with its own wobble
    struct Letter {
        glyph: ab_glyph::Glyph,
        tilt: f32,
        lift: f32,
    }
    let base = PxScale::from(LETTER_PX);
    let fb = font.as_scaled(base);
    let mut x = 0.0f32;
    let mut letters = Vec::new();
    let mut last = None;
    for ch in text.chars() {
        let size = LETTER_PX * (1.0 + 0.09 * rand());
        let scale = PxScale::from(size);
        let f = font.as_scaled(scale);
        let id = f.glyph_id(ch);
        if let Some(prev) = last {
            x += fb.kern(prev, id);
        }
        letters.push(Letter { glyph: id.with_scale_and_position(scale, ab_glyph::point(x, fb.ascent())), tilt: 0.13 * rand(), lift: 5.0 * rand() });
        x += f.h_advance(id) + 2.0 * rand();
        last = Some(id);
    }
    let w = (x.ceil().max(1.0) as u32) + PAD * 2;
    let h = ((fb.ascent() - fb.descent()).ceil() as u32).max(1) + PAD * 2;
    let mut px = vec![0u8; (w * h * 4) as usize];
    for l in letters {
        let Some(outline) = font.outline_glyph(l.glyph) else { continue };
        let b = outline.px_bounds();
        // draw the letter on its own, then turn it about its middle and lay it on the picture
        let (lw, lh) = (b.width().ceil() as i32 + 1, b.height().ceil() as i32 + 1);
        let mut cover = vec![0f32; (lw * lh) as usize];
        outline.draw(|gx, gy, c| {
            if (gx as i32) < lw && (gy as i32) < lh {
                cover[(gy as i32 * lw + gx as i32) as usize] = c.clamp(0.0, 1.0);
            }
        });
        let (cx, cy) = (b.min.x + b.width() / 2.0 + PAD as f32, b.min.y + b.height() / 2.0 + PAD as f32 + l.lift);
        let (sin, cos) = l.tilt.sin_cos();
        let reach = (lw.max(lh) as f32 * 0.75).ceil() as i32;
        for py in (cy as i32 - reach)..=(cy as i32 + reach) {
            for qx in (cx as i32 - reach)..=(cx as i32 + reach) {
                if qx < 0 || py < 0 || qx >= w as i32 || py >= h as i32 {
                    continue;
                }
                // where this picture pixel comes from in the upright letter
                let (dx, dy) = (qx as f32 + 0.5 - cx, py as f32 + 0.5 - cy);
                let (sx, sy) = (dx * cos + dy * sin + lw as f32 / 2.0, -dx * sin + dy * cos + lh as f32 / 2.0);
                let (ix, iy) = (sx.floor() as i32, sy.floor() as i32);
                if ix < 0 || iy < 0 || ix >= lw || iy >= lh {
                    continue;
                }
                let a = (cover[(iy * lw + ix) as usize] * 255.0) as u8;
                let i = ((py as u32 * w + qx as u32) * 4) as usize;
                px[i..i + 3].copy_from_slice(&[255, 255, 255]);
                px[i + 3] = px[i + 3].max(a);
            }
        }
    }
    (px, w, h)
}

fn name_image(name: &str) -> (Image, f32) {
    let (px, w, h) = draw_name(name);
    let img = Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        px,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    (img, w as f32 / h as f32)
}

/// The singlet's points (from its mesh), in the body's own space.
/// The singlet node, its points (in the body's own space), and how each point moves for the
/// belly shapes (target-major: every point for "Belly", then every point for "Sag").
#[allow(clippy::type_complexity)]
fn singlet_points(body: Entity, kids: &Query<&Children>, names: &Query<&Name>, tfs: &Query<&Transform>, meshes_on: &Query<&Mesh3d>, meshes: &Assets<Mesh>) -> Option<(Entity, Vec<Vec3>, Vec<Vec3>)> {
    let singlet = kids.get(body).ok()?.iter().find(|e| names.get(*e).is_ok_and(|n| n.as_str() == "Singlet"))?;
    let at = tfs.get(singlet).map(|t| t.translation).unwrap_or_default();
    for e in std::iter::once(singlet).chain(kids.iter_descendants(singlet)) {
        let Ok(m) = meshes_on.get(e) else { continue };
        let Some(m) = meshes.get(&m.0) else { continue };
        if let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) = m.attribute(Mesh::ATTRIBUTE_POSITION) {
            let moves = m.try_morph_targets().ok().map(|t| t.iter().map(|a| a.position).collect()).unwrap_or_default();
            return Some((singlet, p.iter().map(|v| Vec3::from(*v) + at).collect(), moves));
        }
        return None;
    }
    None
}

/// Where the print sits: (height, how far out the singlet's front is there, half its width
/// there, the singlet's height), so the print can sit just outside the chest of the cloth.
pub fn chest_spot(points: &[Vec3]) -> Option<(f32, f32, f32, f32)> {
    let lo = points.iter().fold(Vec3::splat(f32::MAX), |a, p| a.min(*p));
    let hi = points.iter().fold(Vec3::splat(f32::MIN), |a, p| a.max(*p));
    if !(lo.y < hi.y) {
        return None;
    }
    // up on the chest, above the tummy (which swells with the belly slider)
    let mid_y = lo.y + (hi.y - lo.y) * 0.74;
    let band: Vec<&Vec3> = points.iter().filter(|p| (p.y - mid_y).abs() < (hi.y - lo.y) * 0.12).collect();
    let front = band.iter().filter(|p| p.x.abs() < 0.12).map(|p| p.z).fold(f32::MIN, f32::max);
    let half = band.iter().map(|p| p.x.abs()).fold(0.0f32, f32::max);
    (front > f32::MIN).then_some((mid_y, front, half, hi.y - lo.y))
}

/// Print each blob's name on its back, and reprint it when the name or the singlet colour changes.
#[allow(clippy::too_many_arguments)]
fn print_names(
    mut commands: Commands,
    game: Res<Game>,
    settings: Res<Settings>,
    online: Option<Res<crate::online::Online>>,
    mut images: ResMut<Assets<Image>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut heads: Query<(Entity, &HeadNode, Option<&mut Printed>)>,
    parents: Query<&ChildOf>,
    kids: Query<&Children>,
    names: Query<&Name>,
    mut tfs: Query<&mut Transform>,
    meshes_on: Query<&Mesh3d>,
    weights: Query<&bevy::mesh::morph::MorphWeights>,
) {
    for (head, node, printed) in &mut heads {
        // the print rides on the cloth: as the belly swells and sags, it moves with the cloth under it
        if let Some(p) = printed.as_deref()
            && let Ok(w) = weights.get(p.singlet)
            && let Ok(mut tf) = tfs.get_mut(p.strip)
        {
            let w = w.weights();
            let at = p.base + p.moves[0] * w.first().copied().unwrap_or(0.0) + p.moves[1] * w.get(1).copied().unwrap_or(0.0);
            if tf.translation != at {
                tf.translation = at;
            }
        }
        let Some(name) = name_of(node.owner, &game, &settings, online.as_deref()) else { continue };
        let Some(singlet) = singlet_of(node.owner, &game, &settings, online.as_deref()) else { continue };
        let ink = ink_for(singlet);
        if let Some(mut p) = printed {
            if p.name == name && p.ink == ink {
                continue;
            }
            // a new name or singlet: redraw the picture, change the ink
            if p.name != name {
                let (img, _) = name_image(&name);
                if let Some(mut m) = mats.get_mut(&p.material) {
                    m.base_color_texture = Some(images.add(img));
                }
                p.name = name;
            }
            if let Some(mut m) = mats.get_mut(&p.material) {
                m.base_color = crate::models::hex(ink);
            }
            p.ink = ink;
            continue;
        }
        let Ok(body) = parents.get(head).map(|p| p.parent()) else { continue };
        let read_tfs = tfs.as_readonly();
        let Some((singlet_node, points, moves)) = singlet_points(body, &kids, &names, &read_tfs, &meshes_on, &meshes) else { continue };
        let Some((mid_y, front, half, singlet_tall)) = chest_spot(&points) else { continue };
        // the cloth point under the print's middle, and how it moves for the belly shapes
        let middle = Vec3::new(0.0, mid_y, front);
        let nearest = points.iter().enumerate().min_by(|a, b| a.1.distance_squared(middle).total_cmp(&b.1.distance_squared(middle))).map(|(i, _)| i).unwrap_or(0);
        let n = points.len();
        let shape_moves = if moves.len() >= 2 * n { [moves[nearest], moves[n + nearest]] } else { [Vec3::ZERO; 2] };
        let (img, aspect) = name_image(&name);
        let material = mats.add(StandardMaterial {
            base_color: crate::models::hex(ink),
            base_color_texture: Some(images.add(img)),
            alpha_mode: AlphaMode::Mask(0.5),
            perceptual_roughness: 0.95,
            reflectance: 0.05,
            ..default()
        });
        // across the chest, sized to it, just outside the cloth (the blobs face +z)
        let r = front + 0.015;
        let width = (half * 1.6).min(0.9);
        let tall = (width / aspect).min(singlet_tall * 0.36);
        let width = tall * aspect;
        let strip = meshes.add(curved_strip(r, width / r, tall, 16));
        // the strip is made on the back (-z): turned half round it faces out of the front, still
        // reading left to right for someone in front
        let base = Vec3::new(0.0, mid_y, 0.0);
        let mut print = commands.spawn((
            Mesh3d(strip),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(base).with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
            bevy::light::NotShadowCaster,
            ChildOf(body),
        ));
        if let Some(l) = node.layer {
            print.insert(bevy::camera::visibility::RenderLayers::layer(l));
        }
        let strip = print.id();
        commands.entity(head).try_insert(Printed { name, ink, material, strip, singlet: singlet_node, base, moves: shape_moves });
    }
}

/// A strip of a cylinder of radius `r` round the up axis, `span` radians wide and `tall` high,
/// centred on the back (-z), facing out, with the picture reading left to right for someone
/// standing behind.
pub fn curved_strip(r: f32, span: f32, tall: f32, segments: usize) -> Mesh {
    let mut pos = Vec::new();
    let mut nor = Vec::new();
    let mut uv = Vec::new();
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let a = -span / 2.0 + span * t;
        let (x, z) = (r * a.sin(), -r * a.cos());
        for (y, v) in [(tall / 2.0, 0.0), (-tall / 2.0, 1.0)] {
            pos.push([x, y, z]);
            nor.push([a.sin(), 0.0, -a.cos()]);
            // seen from behind, +x is on the left: the words start there
            uv.push([1.0 - t, v]);
        }
    }
    let mut idx = Vec::new();
    for i in 0..segments as u32 {
        let (a, b, c, d) = (i * 2, i * 2 + 1, i * 2 + 2, i * 2 + 3);
        idx.extend_from_slice(&[a, c, b, b, c, d]);
    }
    Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, nor)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(bevy::mesh::Indices::U32(idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_draws_wider_than_tall_and_has_ink() {
        let (px, w, h) = draw_name("Marcus");
        assert!(w > h * 2, "{w}x{h}");
        assert_eq!(px.len(), (w * h * 4) as usize);
        let inked = px.chunks(4).filter(|p| p[3] > 128).count();
        assert!(inked > 500, "letters drawn: {inked}");
        // longer names make wider pictures; an empty one is just the border
        assert!(draw_name("Shazza the Great").1 > w);
        let (_, ew, _) = draw_name("");
        assert!(ew <= PAD * 2 + 1);
        // wonky, but the same every time for the same name
        assert_eq!(draw_name("Marcus").0, px);
    }

    #[test]
    fn the_ink_stands_out_from_the_singlet() {
        for s in bbq_core::appearance::SINGLET_COLOURS {
            let ink = ink_for(s);
            let lum = |c: u32| 0.299 * ((c >> 16) & 0xff) as f32 + 0.587 * ((c >> 8) & 0xff) as f32 + 0.114 * (c & 0xff) as f32;
            assert!((lum(ink) - lum(s)).abs() > 90.0, "singlet {s:06x} ink {ink:06x}");
        }
    }
}
