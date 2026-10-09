//! Faces (customization step 3): the mouth on every blob's head, built from small shapes in code
//! so each kind can be picked in the menu and change with the blob's mood (C10.9).
//!
//! Every head that should wear a face gets a `HeadNode` when its model loads. `sync_faces`
//! builds the mouth on it, and builds it again whenever the wanted look or the mood changes
//! (it is only a handful of little parts). The mouth sits on the front of the head just below
//! the eyes, sized to the head, and moves with the head (and its wonky tilt).

use bbq_core::appearance::{Appearance, Brows, Hair, Mouth, Palette};
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;

use crate::game::Game;
use crate::menu::Settings;

/// Whose face this head wears.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FaceOwner {
    /// A blob in the yard (its index in `Game::dummies`).
    Blob(usize),
    /// The menu preview (your own look).
    Preview,
    /// Somebody's figure in the online lobby's line-up (by their number in the yard).
    Lobby(u32),
}

/// A blob's head: the face goes on it.
#[derive(Component, Clone, Copy)]
pub struct HeadNode {
    pub owner: FaceOwner,
    /// Draw on this render layer (the preview has its own).
    pub layer: Option<usize>,
}

/// What the mouth is doing (C10.9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Mood {
    #[default]
    Normal,
    /// Stunned or knocked down: a round "O".
    Ouch,
    /// Drinking: a small round "o" for the sip.
    Sip,
    /// Winding up a throw: gritted teeth.
    Grit,
    /// Ready in the lobby: a big open grin and raised brows.
    Cheer,
}

/// The mouth that is on a head now (so it is only rebuilt when something changes).
#[derive(Component, Clone, Copy, PartialEq)]
struct FaceShown {
    mouth: Mouth,
    brows: Brows,
    brow_colour: u32,
    hair: Hair,
    hair_colour: u32,
    mood: Mood,
}

/// The mouth's root on the head; drunk blobs' mouths wobble.
#[derive(Component)]
struct MouthRoot {
    owner: FaceOwner,
    /// The mouth wobbles when drunk; brows do not.
    mouth: bool,
}

#[derive(Resource)]
struct FaceKit {
    ball: Handle<Mesh>,
    /// One material per hair or brow colour, made when first needed.
    hair: std::collections::HashMap<u32, Handle<StandardMaterial>>,
    dark: Handle<StandardMaterial>,
    teeth: Handle<StandardMaterial>,
    tongue: Handle<StandardMaterial>,
}

/// Which part of the body a mesh is, so it can be painted in the look's colours.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TintPart {
    Body,
    Head,
    Foot,
    Singlet,
    Thong,
    Shorts,
}

/// One of the shorts shells on a model: the model has both kinds and the look picks which shows.
#[derive(Component, Clone, Copy)]
pub struct ShortsPiece {
    pub owner: FaceOwner,
    pub kind: bbq_core::appearance::Shorts,
}

/// A mesh that wears one of the look's colours.
#[derive(Component, Clone, Copy)]
pub struct LookTint {
    pub owner: FaceOwner,
    pub part: TintPart,
}

/// The colour a tinted mesh has now.
#[derive(Component, Clone, Copy, PartialEq)]
struct TintShown(u32);

/// The body, head and foot colours from a body colour (the head and hands a little lighter, the
/// feet a little darker, as the blobs have always been).
pub fn shade_of(part: TintPart, base: u32) -> u32 {
    let k = match part {
        TintPart::Head => 1.18,
        TintPart::Foot => 0.78,
        _ => 1.0,
    };
    let ch = |s: u32| {
        let c = ((base >> s) & 0xff) as f32;
        let v = if k > 1.0 { c + (255.0 - c) * (k - 1.0) * 1.6 } else { c * k };
        v.clamp(0.0, 255.0).round() as u32
    };
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

/// Paint every tinted mesh in its owner's colours. Bots keep their own body colours (only their
/// singlet and thongs follow their look); you in the preview, and people online, wear your look.
fn sync_tints(
    mut commands: Commands,
    game: Res<Game>,
    settings: Res<Settings>,
    online: Option<Res<crate::online::Online>>,
    assets: Res<AssetServer>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut cache: Local<std::collections::HashMap<(u32, u8), Handle<StandardMaterial>>>,
    q: Query<(Entity, &LookTint, Option<&TintShown>)>,
) {
    for (e, t, shown) in &q {
        let (look, own_body) = match t.owner {
            FaceOwner::Preview => (settings.look, true),
            FaceOwner::Lobby(id) => match online.as_deref().and_then(|o| crate::lobby::lobby_look(o, &settings, id)) {
                Some(l) => (l.1, true),
                None => continue,
            },
            FaceOwner::Blob(i) => match game.dummies.get(i) {
                Some(d) => (d.look, d.remote.is_some()),
                None => continue,
            },
        };
        let colour = match t.part {
            TintPart::Body | TintPart::Head | TintPart::Foot => {
                if !own_body {
                    continue;
                }
                shade_of(t.part, look.colour(Palette::Body))
            }
            TintPart::Singlet => look.colour(Palette::Singlet),
            TintPart::Thong => look.colour(Palette::Thong),
            TintPart::Shorts => look.colour(Palette::Shorts),
        };
        if shown.map(|s| s.0) == Some(colour) {
            continue;
        }
        let skin = matches!(t.part, TintPart::Body | TintPart::Head | TintPart::Foot);
        let m = cache
            .entry((colour, if skin { 0 } else if t.part == TintPart::Thong { 1 } else { 2 }))
            .or_insert_with(|| {
                if skin {
                    mats.add(crate::style::body_material(crate::models::hex(colour), &assets))
                } else if matches!(t.part, TintPart::Singlet | TintPart::Shorts) {
                    mats.add(crate::style::cloth_material(crate::models::hex(colour), &assets))
                } else {
                    mats.add(StandardMaterial { base_color: crate::models::hex(colour), perceptual_roughness: 0.45, reflectance: 0.35, ..default() })
                }
            })
            .clone();
        commands.entity(e).try_insert((MeshMaterial3d(m), TintShown(colour)));
    }
}

/// Which kind of shorts a model node is (the models carry both kinds).
pub fn shorts_kind(name: &str) -> Option<bbq_core::appearance::Shorts> {
    match name {
        "Shorts_stubbies" => Some(bbq_core::appearance::Shorts::Stubbies),
        "Shorts_boardies" => Some(bbq_core::appearance::Shorts::Boardies),
        _ => None,
    }
}

/// Show the kind of shorts a look wears (and none of the other kind).
fn sync_shorts(
    game: Res<Game>,
    settings: Res<Settings>,
    online: Option<Res<crate::online::Online>>,
    mut q: Query<(&ShortsPiece, &mut Visibility)>,
) {
    for (p, mut v) in &mut q {
        let look = match p.owner {
            FaceOwner::Preview => settings.look,
            FaceOwner::Lobby(id) => match online.as_deref().and_then(|o| crate::lobby::lobby_look(o, &settings, id)) {
                Some(l) => l.1,
                None => continue,
            },
            FaceOwner::Blob(i) => match game.dummies.get(i) {
                Some(d) => d.look,
                None => continue,
            },
        };
        let want = if look.shorts == p.kind { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}

pub struct FacePlugin;

impl Plugin for FacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, make_kit)// after Update, so a model swapped or thrown away this frame is already gone
            .add_systems(PostUpdate, (sync_faces, wobble_mouths, sync_tints, sync_shorts).chain());
    }
}

fn make_kit(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let m = |c: u32, rough: f32| StandardMaterial { base_color: crate::models::hex(c), perceptual_roughness: rough, reflectance: 0.25, ..default() };
    commands.insert_resource(FaceKit {
        ball: meshes.add(Sphere::new(1.0).mesh().uv(16, 10)),
        hair: Default::default(),
        dark: mats.add(m(0x3a1812, 0.55)),
        teeth: mats.add(m(0xf7f4ea, 0.35)),
        tongue: mats.add(m(0xd9606a, 0.5)),
    });
}

/// How the blob is feeling, from what it is doing.
pub fn mood_of(stunned: bool, down: bool, drinking: bool, winding: bool) -> Mood {
    if stunned || down {
        Mood::Ouch
    } else if drinking {
        Mood::Sip
    } else if winding {
        Mood::Grit
    } else {
        Mood::Normal
    }
}

/// One little piece of a mouth: a squashed ball `size` big at `at` (mouth space: x right, y up,
/// z out of the face), in one of the kit's colours.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blip {
    pub at: Vec3,
    pub size: Vec3,
    pub colour: Colour,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Colour {
    Dark,
    Teeth,
    Tongue,
    /// The look's brow colour.
    Hair,
    /// The look's hair colour.
    HairTop,
}

/// The shapes of a mouth, for a head of radius 1 (scaled to the head when built). Kept apart
/// from the drawing so it can be tested.
pub fn mouth_shape(mouth: Mouth, mood: Mood) -> Vec<Blip> {
    let b = |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32, colour| Blip { at: Vec3::new(x, y, z), size: Vec3::new(sx, sy, sz), colour };
    if mouth == Mouth::None {
        return Vec::new();
    }
    // a curve of little dark beads, `lift` how far the ends turn up, `tilt` for lopsided
    let arc = |n: usize, half_w: f32, lift: f32, tilt: f32, thick: f32| -> Vec<Blip> {
        (0..n)
            .map(|k| {
                let u = k as f32 / (n - 1) as f32 * 2.0 - 1.0;
                b(u * half_w, lift * u * u + tilt * u, 0.0, thick * 1.25, thick, thick * 0.55, Colour::Dark)
            })
            .collect()
    };
    match mood {
        Mood::Ouch => vec![b(0.0, -0.01, 0.025, 0.15, 0.19, 0.08, Colour::Dark), b(0.0, -0.06, 0.05, 0.08, 0.05, 0.04, Colour::Tongue)],
        Mood::Sip => vec![b(0.0, 0.0, 0.02, 0.085, 0.085, 0.07, Colour::Dark)],
        Mood::Grit => vec![b(0.0, 0.0, 0.02, 0.32, 0.1, 0.07, Colour::Dark), b(0.0, 0.0, 0.045, 0.29, 0.065, 0.05, Colour::Teeth)],
        Mood::Cheer => mouth_shape(Mouth::Grin, Mood::Normal),
        Mood::Normal => match mouth {
            Mouth::None => Vec::new(),
            // a small, happy smile
            Mouth::Smile => arc(15, 0.24, 0.1, 0.0, 0.07),
            // a big open grin: a dark D with teeth along the top and a bit of tongue
            Mouth::Grin => vec![
                b(0.0, -0.03, 0.025, 0.4, 0.22, 0.08, Colour::Dark),
                b(0.0, 0.045, 0.05, 0.34, 0.07, 0.05, Colour::Teeth),
                b(0.0, -0.085, 0.05, 0.2, 0.07, 0.05, Colour::Tongue),
            ],
            // a lopsided smirk: flat on one side, curling up on the other
            Mouth::Smirk => {
                let mut v = arc(13, 0.2, 0.06, 0.07, 0.065);
                for p in &mut v {
                    p.at.x += 0.05;
                }
                v
            }
        },
    }
}

/// One eyebrow (the right one; the left is its mirror), for a head of radius 1: the pieces in
/// the brow's own space (x along it, towards the middle of the face; y up) and how it sits:
/// (lift above its resting place, tilt in radians: positive = the inner end lower, angry).
pub fn brow_shape(brows: Brows, mood: Mood) -> (Vec<Blip>, f32, f32) {
    let bead = |x: f32, y: f32, w: f32, h: f32| Blip { at: Vec3::new(x, y, 0.0), size: Vec3::new(w, h, h * 0.7), colour: Colour::Hair };
    let pieces: Vec<Blip> = match brows {
        Brows::None => return (Vec::new(), 0.0, 0.0),
        // thick and straight
        Brows::Flat => (0..7).map(|k| bead(-0.17 + k as f32 * 0.057, 0.0, 0.13, 0.1)).collect(),
        // thin and arched: the middle higher than the ends
        Brows::Arched => (0..8)
            .map(|k| {
                let u = k as f32 / 7.0 * 2.0 - 1.0;
                bead(u * 0.19, (1.0 - u * u) * 0.08, 0.09, 0.065)
            })
            .collect(),
        // big and bushy: fat, a bit uneven
        Brows::Bushy => (0..6)
            .map(|k| {
                let wob = [0.0, 0.02, -0.012, 0.024, -0.006, 0.016][k];
                bead(-0.18 + k as f32 * 0.072, wob, 0.17, 0.15)
            })
            .collect(),
    };
    let (lift, tilt) = match mood {
        Mood::Normal => (0.0, 0.0),
        // winding up: angry, inner ends pulled down
        Mood::Grit => (-0.02, 0.38),
        // hurt: worried, inner ends up
        Mood::Ouch => (0.04, -0.35),
        // sipping: eyebrows up, pleased
        Mood::Sip => (0.05, -0.1),
        // ready to go: eyebrows right up
        Mood::Cheer => (0.07, -0.15),
    };
    (pieces, lift, tilt)
}

/// The hair, for a head of radius 1 centred on the origin (x right, y up, z out of the face).
pub fn hair_shape(hair: Hair) -> Vec<Blip> {
    let b = |x: f32, y: f32, z: f32, sx: f32, sy: f32, sz: f32| Blip { at: Vec3::new(x, y, z), size: Vec3::new(sx, sy, sz), colour: Colour::HairTop };
    match hair {
        Hair::None => Vec::new(),
        // a little tuft sticking up on top
        Hair::Tuft => vec![
            b(0.0, 1.02, 0.05, 0.4, 0.8, 0.4),
            b(-0.2, 0.96, 0.0, 0.32, 0.6, 0.32),
            b(0.19, 0.97, -0.1, 0.32, 0.64, 0.32),
            b(0.05, 1.28, 0.2, 0.22, 0.44, 0.22),
        ],
        // business at the front, party at the back: a short cap on top and a long flap behind
        Hair::Mullet => vec![
            b(0.0, 0.7, -0.3, 1.98, 0.95, 1.75),
            b(0.0, 0.1, -0.7, 1.7, 1.5, 0.9),
            b(0.0, -0.45, -0.78, 1.45, 1.0, 0.7),
            b(0.0, -0.85, -0.72, 1.1, 0.6, 0.5),
        ],
        // a round bowl cut with a straight fringe
        Hair::Bowl => vec![b(0.0, 0.58, -0.12, 2.14, 1.15, 2.1), b(0.0, 0.84, 0.55, 1.35, 0.3, 0.5)],
    }
}

/// Build or rebuild the mouth on every head whose look or mood has changed.
fn sync_faces(
    mut commands: Commands,
    kit: Option<ResMut<FaceKit>>,
    online: Option<Res<crate::online::Online>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    game: Res<Game>,
    settings: Res<Settings>,
    heads: Query<(Entity, &HeadNode, Option<&FaceShown>)>,
    kids: Query<&Children>,
    meshes_on: Query<&Mesh3d>,
    roots: Query<(Entity, &ChildOf), With<MouthRoot>>,
    meshes: Res<Assets<Mesh>>,
) {
    let Some(mut kit) = kit else { return };
    for (head, node, shown) in &heads {
        let (look, mood): (Appearance, Mood) = match node.owner {
            FaceOwner::Preview => (settings.look, Mood::Normal),
            FaceOwner::Lobby(id) => match online.as_deref().and_then(|o| crate::lobby::lobby_look(o, &settings, id)) {
                // ready: a big grin with teeth (the ready face), otherwise their own mouth
                Some(l) => (l.1, if l.3 { Mood::Cheer } else { Mood::Normal }),
                None => continue,
            },
            FaceOwner::Blob(i) => {
                let Some(d) = game.dummies.get(i) else { continue };
                (d.look, mood_of(d.body.stun > 0.0, d.body.is_down(), d.bot.drunk.is_drinking(), d.bot.winding))
            }
        };
        let want = FaceShown { mouth: look.mouth, brows: look.brows, brow_colour: look.colour(Palette::Brow), hair: look.hair, hair_colour: look.colour(Palette::Hair), mood };
        if shown == Some(&want) {
            continue;
        }
        // the head's own middle and size, from its mesh. Some models keep the head's points
        // round the head's own origin, others keep them where they sit on the body, so work
        // out the middle from the points themselves (the box round them)
        let mut lo = Vec3::splat(f32::MAX);
        let mut hi = Vec3::splat(f32::MIN);
        let mut any = false;
        for e in std::iter::once(head).chain(kids.iter_descendants(head)) {
            // only the head's own mesh, not a mouth we built earlier
            if roots.iter().any(|(r, _)| r == e) {
                continue;
            }
            let Ok(m) = meshes_on.get(e) else { continue };
            let Some(m) = meshes.get(&m.0) else { continue };
            if let Some(bevy::mesh::VertexAttributeValues::Float32x3(p)) = m.attribute(Mesh::ATTRIBUTE_POSITION) {
                for v in p {
                    lo = lo.min(Vec3::from(*v));
                    hi = hi.max(Vec3::from(*v));
                }
                any = true;
            }
            break; // the first mesh found is the head itself
        }
        let r = any.then(|| (hi - lo).max_element() * 0.5);
        let centre = (lo + hi) * 0.5;
        let Some(r) = r else { continue }; // the mesh is not loaded yet: try next frame
        for (e, parent) in &roots {
            if parent.parent() == head {
                commands.entity(e).despawn();
            }
        }
        commands.entity(head).try_insert(want);
        let hair_mat = kit
            .hair
            .entry(want.brow_colour)
            .or_insert_with(|| mats.add(StandardMaterial { base_color: crate::models::hex(want.brow_colour), perceptual_roughness: 0.8, ..default() }))
            .clone();
        let top_mat = kit
            .hair
            .entry(want.hair_colour)
            .or_insert_with(|| mats.add(StandardMaterial { base_color: crate::models::hex(want.hair_colour), perceptual_roughness: 0.8, ..default() }))
            .clone();
        let pick = |c: Colour| match c {
            Colour::Dark => kit.dark.clone(),
            Colour::Teeth => kit.teeth.clone(),
            Colour::Tongue => kit.tongue.clone(),
            Colour::Hair => hair_mat.clone(),
            Colour::HairTop => top_mat.clone(),
        };
        // a point on the face: (x, y) across and up from the head's middle, in head radii; the
        // part sits on the surface facing out, turned `roll` about that facing
        let on_face = |x: f32, y: f32, roll: f32| -> Transform {
            let (x, y) = (x * r, y * r);
            let z = (r * r - x * x - y * y).max(0.0).sqrt();
            let normal = Vec3::new(x, y, z).normalize();
            Transform::from_translation(centre + normal * r * 0.985)
                .with_rotation(Quat::from_rotation_arc(Vec3::Z, normal) * Quat::from_rotation_z(roll))
        };
        let mut parts: Vec<(Transform, Vec<Blip>, bool)> = Vec::new();
        let mouth = mouth_shape(look.mouth, mood);
        if !mouth.is_empty() {
            parts.push((on_face(0.0, -0.45, 0.0), mouth, true));
        }
        let hair = hair_shape(look.hair);
        if !hair.is_empty() {
            // hair is placed round the head's middle, not on the face
            parts.push((Transform::from_translation(centre), hair, false));
        }
        let (brow, lift, tilt) = brow_shape(look.brows, mood);
        if !brow.is_empty() {
            for side in [1.0f32, -1.0] {
                // above each eye; the right brow's x runs towards the middle, the left is mirrored
                let mut pieces = brow.clone();
                for p in &mut pieces {
                    p.at.x *= -side;
                }
                parts.push((on_face(side * 0.36, 0.55 + lift, side * tilt), pieces, false));
            }
        }
        for (tf, blips, is_mouth) in parts {
            let mut root = commands.spawn((tf, Visibility::default(), ChildOf(head)));
            root.insert(MouthRoot { owner: node.owner, mouth: is_mouth });
            if let Some(l) = node.layer {
                root.insert(RenderLayers::layer(l));
            }
            let root = root.id();
            for p in blips {
                let mut e = commands.spawn((
                    Mesh3d(kit.ball.clone()),
                    MeshMaterial3d(pick(p.colour)),
                    Transform::from_translation(p.at * r).with_scale(p.size * r * 0.5),
                    bevy::light::NotShadowCaster,
                    ChildOf(root),
                ));
                if let Some(l) = node.layer {
                    e.insert(RenderLayers::layer(l));
                }
            }
        }
    }
}

/// Drunk blobs' mouths wobble about.
fn wobble_mouths(time: Res<Time>, game: Res<Game>, mut q: Query<(&MouthRoot, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (m, mut tf) in &mut q {
        if !m.mouth {
            continue;
        }
        let drunk = match m.owner {
            FaceOwner::Blob(i) => game.dummies.get(i).map_or(0.0, |d| d.drunk),
            FaceOwner::Preview | FaceOwner::Lobby(_) => 0.0,
        };
        let k = ((drunk - 40.0) / 60.0).clamp(0.0, 1.0);
        // keep the "facing out" part, add a roll about the mouth's own forward axis
        let base = tf.rotation;
        let roll = (t * 2.3 + m_seed(m)).sin() * 0.35 * k;
        let out = base * Vec3::Z;
        tf.rotation = Quat::from_rotation_arc(Vec3::Z, out) * Quat::from_rotation_z(roll);
    }
}

fn m_seed(m: &MouthRoot) -> f32 {
    match m.owner {
        FaceOwner::Blob(i) => i as f32 * 1.7,
        FaceOwner::Preview => 0.0,
        FaceOwner::Lobby(id) => id as f32 * 0.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbq_core::appearance::Choice;

    #[test]
    fn each_mouth_looks_different_and_none_is_nothing() {
        assert!(mouth_shape(Mouth::None, Mood::Normal).is_empty());
        assert!(mouth_shape(Mouth::None, Mood::Ouch).is_empty(), "no mouth means no mouth, even when hurt");
        let kinds: Vec<Vec<Blip>> = [Mouth::Smile, Mouth::Grin, Mouth::Smirk].iter().map(|m| mouth_shape(*m, Mood::Normal)).collect();
        assert!(kinds.iter().all(|k| !k.is_empty()));
        assert_ne!(kinds[0], kinds[1]);
        assert_ne!(kinds[0], kinds[2]);
        assert_ne!(kinds[1], kinds[2]);
        assert!(kinds[1].iter().any(|b| b.colour == Colour::Teeth), "the grin shows teeth");
    }

    #[test]
    fn the_smile_turns_up_and_the_smirk_is_lopsided() {
        let smile = mouth_shape(Mouth::Smile, Mood::Normal);
        let mid = smile[smile.len() / 2].at.y;
        assert!(smile[0].at.y > mid && smile.last().unwrap().at.y > mid, "ends higher than the middle");
        assert!((smile[0].at.y - smile.last().unwrap().at.y).abs() < 1e-5, "even");
        let smirk = mouth_shape(Mouth::Smirk, Mood::Normal);
        assert!(smirk.last().unwrap().at.y > smirk[0].at.y + 0.05, "one side curls up");
    }

    #[test]
    fn moods_change_the_mouth_for_everyone() {
        for m in Mouth::ALL.iter().copied().filter(|m| *m != Mouth::None) {
            assert_eq!(mouth_shape(m, Mood::Ouch), mouth_shape(Mouth::Smile, Mood::Ouch));
            assert_ne!(mouth_shape(m, Mood::Ouch), mouth_shape(m, Mood::Normal));
            assert!(mouth_shape(m, Mood::Grit).iter().any(|b| b.colour == Colour::Teeth), "gritted teeth");
        }
        assert_eq!(mood_of(true, false, true, true), Mood::Ouch, "hurt beats everything");
        assert_eq!(mood_of(false, false, true, true), Mood::Sip);
        assert_eq!(mood_of(false, false, false, true), Mood::Grit);
        assert_eq!(mood_of(false, false, false, false), Mood::Normal);
    }

    #[test]
    fn each_brow_is_different_and_moods_tilt_them() {
        assert!(brow_shape(Brows::None, Mood::Grit).0.is_empty());
        let kinds: Vec<Vec<Blip>> = [Brows::Flat, Brows::Arched, Brows::Bushy].iter().map(|b| brow_shape(*b, Mood::Normal).0).collect();
        assert!(kinds.iter().all(|k| !k.is_empty() && k.iter().all(|b| b.colour == Colour::Hair)));
        assert_ne!(kinds[0], kinds[1]);
        assert_ne!(kinds[1], kinds[2]);
        let arched = &kinds[1];
        assert!(arched[arched.len() / 2].at.y > arched[0].at.y, "arched is higher in the middle");
        let bushy_h = kinds[2][0].size.y;
        assert!(bushy_h > kinds[0][0].size.y && bushy_h > kinds[1][0].size.y, "bushy is the fattest");
        assert!(brow_shape(Brows::Flat, Mood::Grit).2 > 0.2, "angry when winding up");
        assert!(brow_shape(Brows::Flat, Mood::Ouch).2 < -0.2, "worried when hurt");
        assert_eq!(brow_shape(Brows::Flat, Mood::Normal).2, 0.0);
    }

    #[test]
    fn each_hair_style_is_different_and_leaves_the_eyes_and_mouth_clear() {
        assert!(hair_shape(Hair::None).is_empty());
        let kinds: Vec<Vec<Blip>> = [Hair::Tuft, Hair::Mullet, Hair::Bowl].iter().map(|h| hair_shape(*h)).collect();
        assert!(kinds.iter().all(|k| !k.is_empty()));
        assert_ne!(kinds[0], kinds[1]);
        assert_ne!(kinds[1], kinds[2]);
        // the eyes sit at about (±0.37, 0.2, 0.85) and the mouth at (0, -0.45, 0.89): no hair there
        for h in &kinds {
            for p in h {
                for spot in [Vec3::new(0.37, 0.2, 0.9), Vec3::new(-0.37, 0.2, 0.9), Vec3::new(0.0, -0.45, 0.9)] {
                    let d = (spot - p.at) / (p.size * 0.5);
                    assert!(d.length() > 1.0, "{p:?} covers {spot}");
                }
            }
        }
        assert!(kinds[1].iter().any(|p| p.at.z < -0.5 && p.at.y < 0.1), "the mullet hangs down the back");
    }

    #[test]
    fn mouths_fit_on_the_face() {
        for m in Mouth::ALL {
            for mood in [Mood::Normal, Mood::Ouch, Mood::Sip, Mood::Grit, Mood::Cheer] {
                for b in mouth_shape(*m, mood) {
                    assert!(b.at.x.abs() + b.size.x * 0.5 < 0.45, "{m:?} {mood:?} too wide");
                    assert!(b.at.y.abs() + b.size.y * 0.5 < 0.25, "{m:?} {mood:?} too tall");
                }
            }
        }
    }
}
