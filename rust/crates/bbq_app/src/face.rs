//! Faces (customization step 3): the mouth on every blob's head, built from small shapes in code
//! so each kind can be picked in the menu and change with the blob's mood (C10.9).
//!
//! Every head that should wear a face gets a `HeadNode` when its model loads. `sync_faces`
//! builds the mouth on it, and builds it again whenever the wanted look or the mood changes
//! (it is only a handful of little parts). The mouth sits on the front of the head just below
//! the eyes, sized to the head, and moves with the head (and its wonky tilt).

use bbq_core::appearance::{Appearance, Mouth};
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
}

/// The mouth that is on a head now (so it is only rebuilt when something changes).
#[derive(Component, Clone, Copy, PartialEq)]
struct FaceShown {
    mouth: Mouth,
    mood: Mood,
}

/// The mouth's root on the head; drunk blobs' mouths wobble.
#[derive(Component)]
struct MouthRoot {
    owner: FaceOwner,
}

#[derive(Resource)]
struct FaceKit {
    ball: Handle<Mesh>,
    dark: Handle<StandardMaterial>,
    teeth: Handle<StandardMaterial>,
    tongue: Handle<StandardMaterial>,
}

pub struct FacePlugin;

impl Plugin for FacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, make_kit).add_systems(Update, (sync_faces, wobble_mouths).chain());
    }
}

fn make_kit(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let m = |c: u32, rough: f32| StandardMaterial { base_color: crate::models::hex(c), perceptual_roughness: rough, reflectance: 0.25, ..default() };
    commands.insert_resource(FaceKit {
        ball: meshes.add(Sphere::new(1.0).mesh().uv(16, 10)),
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

/// Build or rebuild the mouth on every head whose look or mood has changed.
fn sync_faces(
    mut commands: Commands,
    kit: Option<Res<FaceKit>>,
    game: Res<Game>,
    settings: Res<Settings>,
    heads: Query<(Entity, &HeadNode, Option<&FaceShown>)>,
    kids: Query<&Children>,
    meshes_on: Query<&Mesh3d>,
    roots: Query<(Entity, &ChildOf), With<MouthRoot>>,
    meshes: Res<Assets<Mesh>>,
) {
    let Some(kit) = kit else { return };
    for (head, node, shown) in &heads {
        let (look, mood): (Appearance, Mood) = match node.owner {
            FaceOwner::Preview => (settings.look, Mood::Normal),
            FaceOwner::Blob(i) => {
                let Some(d) = game.dummies.get(i) else { continue };
                (d.look, mood_of(d.body.stun > 0.0, d.body.is_down(), d.bot.drunk.is_drinking(), d.bot.winding))
            }
        };
        let want = FaceShown { mouth: look.mouth, mood };
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
        commands.entity(head).insert(want);
        let blips = mouth_shape(look.mouth, mood);
        if blips.is_empty() {
            continue;
        }
        // on the front of the head, a little below the eyes, facing out
        let y = -0.45 * r;
        let normal = Vec3::new(0.0, y, (r * r - y * y).max(0.0).sqrt()).normalize();
        let at = centre + normal * r * 0.985;
        let mut root = commands.spawn((
            Transform::from_translation(at).with_rotation(Quat::from_rotation_arc(Vec3::Z, normal)),
            Visibility::default(),
            MouthRoot { owner: node.owner },
            ChildOf(head),
        ));
        if let Some(l) = node.layer {
            root.insert(RenderLayers::layer(l));
        }
        let root = root.id();
        for p in blips {
            let mat = match p.colour {
                Colour::Dark => kit.dark.clone(),
                Colour::Teeth => kit.teeth.clone(),
                Colour::Tongue => kit.tongue.clone(),
            };
            let mut e = commands.spawn((
                Mesh3d(kit.ball.clone()),
                MeshMaterial3d(mat),
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

/// Drunk blobs' mouths wobble about.
fn wobble_mouths(time: Res<Time>, game: Res<Game>, mut q: Query<(&MouthRoot, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (m, mut tf) in &mut q {
        let drunk = match m.owner {
            FaceOwner::Blob(i) => game.dummies.get(i).map_or(0.0, |d| d.drunk),
            FaceOwner::Preview => 0.0,
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
    fn mouths_fit_on_the_face() {
        for m in Mouth::ALL {
            for mood in [Mood::Normal, Mood::Ouch, Mood::Sip, Mood::Grit] {
                for b in mouth_shape(*m, mood) {
                    assert!(b.at.x.abs() + b.size.x * 0.5 < 0.45, "{m:?} {mood:?} too wide");
                    assert!(b.at.y.abs() + b.size.y * 0.5 < 0.25, "{m:?} {mood:?} too tall");
                }
            }
        }
    }
}
