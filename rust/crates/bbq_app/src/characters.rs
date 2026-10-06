//! The blob characters (same design as the browser game: a capsule body, a lighter round head,
//! big eyes, squashed feet, round hands), their animation, and the things that go with them:
//! name tags, leader crown, stun stars, team sash, HELP ME sign, stink cloud and speech
//! bubbles. Also Dazza the BBQ cook with his six states.
//!
//! The blob itself is a Blender model (`assets/models/blob.glb`, made by `tools/make_blob.py`).
//! Its hands are moved by the animation maths in `bbq_core::pose`.

use bbq_core::character::Character;
use bbq_core::dazza::{self, DazzaAnim};
use bbq_core::pose::{Emote, Inputs, SlapKind};
use bbq_core::teams::Team;
use bbq_core::vec::{Quat as CQuat, V3};
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

use crate::game::Game;
use crate::player::{EyeCamera, Player};

const TEAM_COLOURS: [(Team, Color); 4] = [
    (Team::Red, Color::linear_rgb(0.85, 0.2, 0.18)),
    (Team::Blue, Color::linear_rgb(0.18, 0.4, 0.85)),
    (Team::Green, Color::linear_rgb(0.2, 0.7, 0.3)),
    (Team::Yellow, Color::linear_rgb(0.95, 0.8, 0.2)),
];

pub fn team_colour(t: Team) -> Color {
    match t {
        Team::Wildcard => Color::linear_rgb(0.7, 0.3, 0.9),
        other => TEAM_COLOURS
            .iter()
            .find(|(k, _)| *k == other)
            .map_or(Color::WHITE, |(_, c)| *c),
    }
}

/// Every part of a blob carries this: its place in `Game::dummies`.
#[derive(Component, Clone, Copy)]
pub struct Blob(pub usize);
#[derive(Component)]
struct BlobRoot;
#[derive(Component)]
struct BlobBody;
#[derive(Component)]
struct HandR;
#[derive(Component)]
struct HandL;
/// An eyeball: it bobs and pops a little as the blob walks, each eye in its own way.
#[derive(Component)]
struct EyePart {
    right: bool,
    base: Vec3,
    /// How far the eyeball has been jolted from its socket, and how fast it is moving.
    off: Vec3,
    vel: Vec3,
}
/// A pupil: it sloshes about inside the eye on a bouncy spring.
#[derive(Component)]
struct PupilPart {
    right: bool,
    base: Vec3,
    pos: Vec2,
    vel: Vec2,
}
/// The body or singlet of a blob with a beer belly: its "Belly" and "Sag" shape keys are
/// bounced as the blob moves.
#[derive(Component)]
struct BellyNode;

/// A foot (or its thong or strap): it steps as the blob walks.
#[derive(Component)]
struct FootPart {
    right: bool,
    base: Vec3,
    /// A thong or its strap: it lifts and slaps with each step (C10.7).
    thong: bool,
}
#[derive(Component)]
struct Crown;
#[derive(Component)]
struct Stars;
#[derive(Component)]
struct Sash;
/// The see-through bubble round a blob in their team's colour.
#[derive(Component)]
struct Balloon;
/// Teddy Heist: an inflatable pool ring round the torso in the team colour, instead of the bubble.
#[derive(Component)]
struct SwimRing;
/// The white stripes on the ring (they stay white).
#[derive(Component)]
struct RingStripe;
#[derive(Component)]
struct SmokoCan;
/// A thing that floats above a blob and faces the camera.
#[derive(Component)]
struct Floating {
    target: Entity,
    height: f32,
}
#[derive(Component)]
struct NameTag;
#[derive(Component)]
struct HelpSign;
#[derive(Component)]
struct StinkCloud;

// Dazza
#[derive(Component)]
struct DazzaRoot;
#[derive(Component)]
struct DazzaBody;
#[derive(Component)]
struct DazzaArm;
#[derive(Component)]
struct DazzaBubble;

#[derive(Resource)]
struct Dazza {
    anim: DazzaAnim,
    say_seen: u32,
    swing_seen: u32,
    last: Vec3,
    face: f32,
    /// His walking speed, smoothed over frames.
    speed: f32,
    label: Entity,
}

/// Body colours for the bots, from the browser game's palette (the first one is the player's).
pub const BLOB_COLOURS: [Color; 11] = [
    Color::linear_rgb(1.0, 0.478, 0.349),
    Color::linear_rgb(0.098, 0.702, 0.651),
    Color::linear_rgb(0.549, 0.416, 0.871),
    Color::linear_rgb(0.298, 0.788, 0.941),
    Color::linear_rgb(0.949, 0.361, 0.604),
    Color::linear_rgb(0.482, 0.82, 0.282),
    Color::linear_rgb(1.0, 0.624, 0.11),
    Color::linear_rgb(0.9, 0.3, 0.25),
    Color::linear_rgb(0.2, 0.7, 0.65),
    Color::linear_rgb(0.95, 0.75, 0.2),
    Color::linear_rgb(0.6, 0.6, 0.9),
];
pub const BLOB_NAMES: [&str; 11] = [
    "Shazza", "Davo", "Kev", "Bazza", "Trish", "Robbo", "Mick", "Nev", "Gaz", "Bruce", "Sheila",
];
/// The most bots in a yard (you make twelve, one for each spawn spot).
pub const MAX_BOTS: usize = 11;

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_blobs, spawn_dazza))
            .init_resource::<Poses>()
            .init_resource::<Cast>()
            .init_resource::<Looks>()
            .add_systems(
                Update,
                (
                    viewer_keys,
                    sync_cast_overrides,
                    rebuild_blobs,
                    swap_models,
                    compute_poses,
                    apply_roots,
                    apply_bodies,
                    apply_hands,
                    animate_face_and_feet,
                    apply_crown,
                    apply_stars,
                    apply_sash,
                    apply_balloon,
                    apply_can,
                    apply_help_signs,
                    apply_clouds,
                    animate_dazza,
                    face_camera,
                )
                    .chain(),
            );
    }
}

/// Which character the player picked. Everyone you see in the yard is shown in turn from the
/// four, starting with your pick, so pressing G lets you look at all of them.
/// (A proper character-select screen comes with the menus in a later phase.)
#[derive(Resource, Default)]
pub struct Cast {
    pub mine: Character,
    /// Blobs that are people playing online wear their own pick (set from the game every frame).
    pub overrides: Vec<Option<Character>>,
}

impl Cast {
    /// The character shown on practice blob `i`: blob 0 wears your pick, the others the next ones.
    fn character_of(&self, i: usize) -> Character {
        if let Some(Some(c)) = self.overrides.get(i) {
            return *c;
        }
        let start = Character::ALL
            .iter()
            .position(|c| *c == self.mine)
            .unwrap_or(0);
        Character::ALL[(start + i) % Character::ALL.len()]
    }
}

/// Online players' blobs wear what they picked, and carry their names.
fn sync_cast_overrides(game: Res<Game>, mut cast: ResMut<Cast>) {
    let want: Vec<Option<Character>> = game.dummies.iter().map(|d| d.remote.as_ref().map(|r| r.character)).collect();
    if cast.overrides != want {
        cast.overrides = want;
    }
}

/// Everything needed to (re)build one blob's model: its tints and where it hangs.
struct Look {
    body: Handle<StandardMaterial>,
    head: Handle<StandardMaterial>,
    foot: Handle<StandardMaterial>,
    can_mesh: Handle<Mesh>,
    can_mat: Handle<StandardMaterial>,
    body_entity: Entity,
}

#[derive(Resource, Default)]
struct Looks(Vec<Look>);

/// Marks the loaded model so it can be thrown away and swapped for another character.
#[derive(Component)]
struct BlobModel;

fn spawn_model(
    commands: &mut Commands,
    assets: &AssetServer,
    look: &Look,
    i: usize,
    character: Character,
) {
    let (bm, hm, fm) = (look.body.clone(), look.head.clone(), look.foot.clone());
    let (can_mesh, can_mat) = (look.can_mesh.clone(), look.can_mat.clone());
    commands
        .spawn((
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(crate::style::model_path(character)))),
            Transform::default(),
            BlobModel,
            Blob(i),
            ChildOf(look.body_entity),
        ))
        .observe(
            move |ready: On<WorldInstanceReady>,
                  children: Query<&Children>,
                  names: Query<&Name>,
                  has_mat: Query<(), With<MeshMaterial3d<StandardMaterial>>>,
                  tfs: Query<&Transform>,
                  mesh_nodes: Query<(&Mesh3d, Option<&bevy::mesh::morph::MeshMorphWeights>)>,
                  mut meshes: ResMut<Assets<Mesh>>,
                  mut mats: ResMut<Assets<StandardMaterial>>,
                  mut commands: Commands| {
                // clay style: the skin texture needs UV coordinates and tangents, which the models
                // do not have; make them here from where each point sits on the part (a simple
                // spherical wrap, which suits these round bodies)
                if crate::style::current() == crate::style::Style::Clay {
                    for node in children.iter_descendants(ready.entity) {
                        let Ok((m3, _)) = mesh_nodes.get(node) else { continue };
                        let Some(mut m) = meshes.get_mut(&m3.0) else { continue };
                        if m.attribute(Mesh::ATTRIBUTE_UV_0).is_none()
                            && let Some(bevy::mesh::VertexAttributeValues::Float32x3(pos)) =
                                m.attribute(Mesh::ATTRIBUTE_POSITION).cloned()
                        {
                            let c = pos.iter().fold([0.0f32; 3], |a, p| [a[0] + p[0], a[1] + p[1], a[2] + p[2]]);
                            let n = pos.len().max(1) as f32;
                            let c = [c[0] / n, c[1] / n, c[2] / n];
                            let uv: Vec<[f32; 2]> = pos
                                .iter()
                                .map(|p| {
                                    let (x, y, z) = (p[0] - c[0], p[1] - c[1], p[2] - c[2]);
                                    let u = 0.5 + z.atan2(x) / std::f32::consts::TAU;
                                    let v = 0.5 + y.atan2((x * x + z * z).sqrt()) / std::f32::consts::PI;
                                    [u, v]
                                })
                                .collect();
                            m.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
                        }
                        if m.attribute(Mesh::ATTRIBUTE_UV_0).is_some() && m.attribute(Mesh::ATTRIBUTE_TANGENT).is_none() {
                            let _ = m.generate_tangents();
                        }
                    }
                }
                // pop style: a black, slightly bigger, inside-out copy of every part is its outline.
                // Small parts get a thinner line, and the pupils and straps none at all (their
                // lines were the stray marks round the eyes).
                if crate::style::current() == crate::style::Style::Pop {
                    let om = crate::style::outline_material(&mut mats);
                    for node in children.iter_descendants(ready.entity) {
                        let Ok(name) = names.get(node) else { continue };
                        let width = match name.as_str() {
                            n if n.starts_with("Pupil") || n.starts_with("Eye") || n.starts_with("Strap") || n.starts_with("Thong") => continue,
                            n if n.starts_with("Eye") => 0.009,
                            n if n.starts_with("Hand") => 0.011,
                            n if n.starts_with("Foot") => 0.011,
                            "Singlet" => 0.009,
                            _ => 0.013,
                        };
                        for e in std::iter::once(node).chain(children.iter_descendants(node)) {
                            let Ok((m3, morph)) = mesh_nodes.get(e) else { continue };
                            let Some(src) = meshes.get(&m3.0) else { continue };
                            let mut big = src.clone();
                            if let (
                                Some(bevy::mesh::VertexAttributeValues::Float32x3(pos)),
                                Some(bevy::mesh::VertexAttributeValues::Float32x3(nor)),
                            ) = (
                                src.attribute(Mesh::ATTRIBUTE_POSITION).cloned(),
                                src.attribute(Mesh::ATTRIBUTE_NORMAL).cloned(),
                            ) {
                                let out: Vec<[f32; 3]> = pos
                                    .iter()
                                    .zip(nor.iter())
                                    .map(|(p, n)| [p[0] + n[0] * width, p[1] + n[1] * width, p[2] + n[2] * width])
                                    .collect();
                                big.insert_attribute(Mesh::ATTRIBUTE_POSITION, out);
                            }
                            let h = meshes.add(big);
                            let mut oe = commands.spawn((
                                Mesh3d(h),
                                MeshMaterial3d(om.clone()),
                                Transform::default(),
                                bevy::light::NotShadowCaster,
                                ChildOf(e),
                            ));
                            if let Some(mw) = morph {
                                oe.insert(mw.clone());
                            }
                        }
                    }
                }
                // once loaded: tint it for this player and find the hands, which the animation moves
                for node in children.iter_descendants(ready.entity) {
                    let Ok(name) = names.get(node) else { continue };
                    let tint = match name.as_str() {
                        "Torso" | "Belly" => Some(bm.clone()),
                        "Head" | "HandR" | "HandL" => Some(hm.clone()),
                        "FootL" | "FootR" => Some(fm.clone()),
                        _ => None,
                    };
                    if let Some(m) = tint {
                        let parts = std::iter::once(node).chain(children.iter_descendants(node));
                        for e in parts {
                            if has_mat.contains(e) {
                                commands.entity(e).insert(MeshMaterial3d(m.clone()));
                            }
                        }
                    }
                    let base = tfs.get(node).map(|t| t.translation).unwrap_or_default();
                    if matches!(name.as_str(), "Torso" | "Singlet") {
                        commands.entity(node).insert((BellyNode, Blob(i)));
                    }
                    match name.as_str() {
                        "EyeL" | "EyeR" => {
                            commands.entity(node).insert((
                                EyePart { right: name.as_str() == "EyeR", base, off: Vec3::ZERO, vel: Vec3::ZERO },
                                Blob(i),
                            ));
                        }
                        "PupilL" | "PupilR" => {
                            commands.entity(node).insert((
                                PupilPart {
                                    right: name.as_str() == "PupilR",
                                    base,
                                    pos: Vec2::ZERO,
                                    vel: Vec2::ZERO,
                                },
                                Blob(i),
                            ));
                        }
                        "FootL" | "FootR" | "ThongL" | "ThongR" | "StrapL" | "StrapR" => {
                            commands.entity(node).insert((
                                FootPart { right: name.as_str().ends_with('R'), base, thong: name.starts_with("Thong") || name.starts_with("Strap") },
                                Blob(i),
                            ));
                        }
                        "HandR" => {
                            commands.entity(node).insert((HandR, Blob(i)));
                            commands.spawn((
                                Mesh3d(can_mesh.clone()),
                                MeshMaterial3d(can_mat.clone()),
                                Transform::from_xyz(0.0, 0.12, 0.08),
                                Visibility::Hidden,
                                SmokoCan,
                                Blob(i),
                                ChildOf(node),
                            ));
                        }
                        "HandL" => {
                            commands.entity(node).insert((HandL, Blob(i)));
                        }
                        _ => {}
                    }
                }
            },
        );
}

/// When the pick changes, swap every blob's model for its new character.
fn swap_models(
    mut commands: Commands,
    assets: Res<AssetServer>,
    cast: Res<Cast>,
    looks: Res<Looks>,
    mut shown: Local<Option<Character>>,
    old: Query<Entity, With<BlobModel>>,
) {
    // the first models were spawned at startup with the default character, before the saved
    // pick (or `--char`) was applied, so compare against that, not against "whatever is picked"
    let on_screen = *shown.get_or_insert(Character::default());
    if on_screen == cast.mine {
        return;
    }
    *shown = Some(cast.mine);
    for e in &old {
        commands.entity(e).despawn();
    }
    for (i, look) in looks.0.iter().enumerate() {
        spawn_model(&mut commands, &assets, look, i, cast.character_of(i));
    }
}

pub fn lighter(c: Color) -> Color {
    let l = c.to_linear();
    Color::linear_rgb(
        l.red + (1.0 - l.red) * 0.35,
        l.green + (1.0 - l.green) * 0.35,
        l.blue + (1.0 - l.blue) * 0.35,
    )
}

pub fn darker(c: Color) -> Color {
    let l = c.to_linear();
    Color::linear_rgb(l.red * 0.6, l.green * 0.6, l.blue * 0.6)
}

fn spawn_blobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    cast: Res<Cast>,
    mut looks: ResMut<Looks>,
    game: Res<Game>,
) {
    build_blobs(
        &mut commands,
        &mut meshes,
        &mut mats,
        &assets,
        &cast,
        &mut looks,
        &game,
    );
}

/// When the number of people in the yard changes (a new round with more or fewer bots), throw the
/// old blobs away and build the right number.
fn rebuild_blobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
    cast: Res<Cast>,
    mut looks: ResMut<Looks>,
    game: Res<Game>,
    mut shown: Local<Option<(usize, u32)>>,
    old: Query<
        Entity,
        Or<(
            With<BlobRoot>,
            With<NameTag>,
            With<HelpSign>,
            With<StinkCloud>,
        )>,
    >,
) {
    let n = (game.dummies.len(), game.dummies_version);
    match *shown {
        None => {
            *shown = Some(n); // the first ones were built at startup
            return;
        }
        Some(k) if k == n => return,
        _ => {}
    }
    *shown = Some(n);
    for e in &old {
        commands.entity(e).despawn();
    }
    looks.0.clear();
    build_blobs(
        &mut commands,
        &mut meshes,
        &mut mats,
        &assets,
        &cast,
        &mut looks,
        &game,
    );
}

#[allow(clippy::too_many_arguments)]
fn build_blobs(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    assets: &AssetServer,
    cast: &Cast,
    looks: &mut Looks,
    game: &Game,
) {
    let gold = mats.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.81, 0.2),
        emissive: LinearRgba::new(0.27, 0.2, 0.0, 1.0),
        cull_mode: None,
        ..default()
    });
    let star_mat = mats.add(StandardMaterial {
        base_color: Color::linear_rgb(1.0, 0.88, 0.4),
        emissive: LinearRgba::new(0.53, 0.4, 0.0, 1.0),
        ..default()
    });
    let crown_ring = meshes.add(Cylinder::new(0.21, 0.13));
    let crown_pt = meshes.add(Cone {
        radius: 0.05,
        height: 0.14,
    });
    let star = meshes.add(Sphere::new(0.07).mesh().ico(1).unwrap());
    let sash = meshes.add(Torus::new(0.32, 0.43));
    let balloon = meshes.add(Sphere::new(1.0).mesh().uv(32, 20));
    let swim_ring = meshes.add(Torus::new(0.4, 0.6));
    let ring_stripe = meshes.add(Sphere::new(0.105).mesh().uv(10, 8));
    let stripe_mat = mats.add(StandardMaterial { base_color: Color::WHITE, perceptual_roughness: 0.4, ..default() });
    let can = meshes.add(Cylinder::new(0.035, 0.12));
    let tag_font = TextFont {
        font_size: FontSize::Px(34.0),
        ..default()
    };

    for (i, d) in game.dummies.iter().enumerate() {
        let colour = BLOB_COLOURS[i % BLOB_COLOURS.len()];
        let body_mat = mats.add(crate::style::body_material(colour, assets));
        let head_mat = mats.add(crate::style::body_material(lighter(colour), assets));
        let foot_mat = mats.add(crate::style::body_material(darker(colour), assets));
        let root = commands
            .spawn((
                Transform::from_xyz(d.mover.x, 0.0, d.mover.z),
                Visibility::default(),
                Blob(i),
                BlobRoot,
            ))
            .id();
        let body = commands
            .spawn((
                Transform::default(),
                Visibility::default(),
                BlobBody,
                Blob(i),
                ChildOf(root),
            ))
            .id();

        // the blob model from Blender: which of the four depends on the pick (see `Cast`)
        let look = Look {
            body: body_mat,
            head: head_mat,
            foot: foot_mat,
            can_mesh: can.clone(),
            can_mat: mats.add(Color::linear_rgb(0.85, 0.6, 0.1)),
            body_entity: body,
        };
        spawn_model(commands, assets, &look, i, cast.character_of(i));
        looks.0.push(look);

        // leader crown
        let crown = commands
            .spawn((
                Transform::from_xyz(0.0, 1.86, 0.0),
                Visibility::Hidden,
                Crown,
                Blob(i),
                ChildOf(body),
            ))
            .id();
        commands.spawn((
            Mesh3d(crown_ring.clone()),
            MeshMaterial3d(gold.clone()),
            Transform::default(),
            ChildOf(crown),
        ));
        for k in 0..5 {
            let a = k as f32 / 5.0 * std::f32::consts::TAU;
            commands.spawn((
                Mesh3d(crown_pt.clone()),
                MeshMaterial3d(gold.clone()),
                Transform::from_xyz(a.cos() * 0.2, 0.13, a.sin() * 0.2),
                ChildOf(crown),
            ));
        }
        // stun stars
        let stars = commands
            .spawn((
                Transform::from_xyz(0.0, 1.95, 0.0),
                Visibility::Hidden,
                Stars,
                Blob(i),
                ChildOf(body),
            ))
            .id();
        for k in 0..3 {
            let a = k as f32 / 3.0 * std::f32::consts::TAU;
            commands.spawn((
                Mesh3d(star.clone()),
                MeshMaterial3d(star_mat.clone()),
                Transform::from_xyz(a.cos() * 0.3, 0.0, a.sin() * 0.3),
                ChildOf(stars),
            ));
        }
        // team balloon: a see-through bubble in the team colour (not on the body, so it does
        // not tumble when the blob falls over)
        commands.spawn((
            Mesh3d(balloon.clone()),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::linear_rgba(1.0, 1.0, 1.0, BALLOON_ALPHA),
                unlit: true,
                alpha_mode: AlphaMode::Blend,
                ..default()
            })),
            Transform::from_xyz(0.0, 0.92, 0.0).with_scale(Vec3::splat(BALLOON_RADIUS)),
            Visibility::Hidden,
            bevy::light::NotShadowCaster,
            Balloon,
            Blob(i),
            ChildOf(root),
        ));
        // Teddy Heist: a swim ring round the middle (on the body, so it moves with the torso)
        let ring_mat = mats.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.35,
            reflectance: 0.5,
            ..default()
        });
        let ring = commands
            .spawn((
                Mesh3d(swim_ring.clone()),
                MeshMaterial3d(ring_mat),
                // worn over one shoulder and across the chest, like the sash in the browser game
                Transform::from_xyz(0.0, 0.86, 0.0).with_rotation(Quat::from_rotation_z(-0.72)),
                Visibility::Hidden,
                SwimRing,
                Blob(i),
                ChildOf(body),
            ))
            .id();
        for k in 0..8 {
            let a = k as f32 / 8.0 * std::f32::consts::TAU;
            commands.spawn((
                Mesh3d(ring_stripe.clone()),
                MeshMaterial3d(stripe_mat.clone()),
                Transform::from_xyz(a.cos() * 0.5, 0.0, a.sin() * 0.5)
                    .with_rotation(Quat::from_rotation_y(-a))
                    .with_scale(Vec3::new(1.15, 0.9, 0.55)),
                RingStripe,
                ChildOf(ring),
            ));
        }
        // team sash
        commands.spawn((
            Mesh3d(sash.clone()),
            MeshMaterial3d(mats.add(Color::WHITE)),
            Transform::from_xyz(0.0, 0.92, 0.0).with_rotation(Quat::from_euler(
                EulerRot::XYZ,
                std::f32::consts::FRAC_PI_2,
                0.45,
                0.0,
            )),
            Visibility::Hidden,
            Sash,
            Blob(i),
            ChildOf(body),
        ));

        // name tag
        commands.spawn((
            Text2d::new(match &d.remote {
                Some(r) => r.name.clone(),
                None => BLOB_NAMES[i % BLOB_NAMES.len()].to_string(),
            }),
            tag_font.clone(),
            TextColor(Color::WHITE),
            Transform::from_xyz(0.0, 3.0, 0.0).with_scale(Vec3::splat(0.0085)),
            Floating {
                target: root,
                height: 2.3,
            },
            NameTag,
        ));
        // HELP ME sign (hidden until they stack it)
        commands.spawn((
            Text2d::new("HELP\nME!"),
            TextFont {
                font_size: FontSize::Px(56.0),
                ..default()
            },
            TextColor(Color::linear_rgb(0.84, 0.18, 0.13)),
            TextBackgroundColor(Color::WHITE),
            Transform::from_xyz(0.0, 1.5, 0.0).with_scale(Vec3::splat(0.012)),
            Visibility::Hidden,
            Floating {
                target: root,
                height: 1.5,
            },
            HelpSign,
            Blob(i),
        ));
        // stink cloud
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(1.0))),
            MeshMaterial3d(mats.add(StandardMaterial {
                base_color: Color::linear_rgba(0.72, 0.81, 0.44, 0.28),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_xyz(d.mover.x, 1.0, d.mover.z),
            Visibility::Hidden,
            StinkCloud,
            Blob(i),
        ));
    }
}

fn spawn_dazza(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    look: Res<crate::lighting::LookMode>,
) {
    let polished = *look == crate::lighting::LookMode::Polished;
    let mut m = |c: Color| mats.add(c);
    // step 2c: sunburnt skin and a caricature (belly, cork hat, sunnies, stubbies, thongs)
    let skin = if polished { m(crate::models::hex(0xf2a98a)) } else { m(Color::linear_rgb(0.91, 0.71, 0.55)) };
    let singlet = m(Color::linear_rgb(0.11, 0.25, 0.45));
    let apron = m(Color::linear_rgb(0.96, 0.97, 0.95));
    let belt = m(Color::linear_rgb(0.7, 0.23, 0.17));
    let hat = m(Color::linear_rgb(0.48, 0.35, 0.23));
    let dark = m(Color::linear_rgb(0.07, 0.07, 0.07));
    let nose = if polished { m(crate::models::hex(0xe0614a)) } else { m(Color::linear_rgb(0.85, 0.56, 0.42)) };
    let boot = m(Color::linear_rgb(0.23, 0.17, 0.1));
    let tool = m(Color::linear_rgb(0.16, 0.16, 0.16));
    let plate = m(Color::linear_rgb(0.85, 0.87, 0.88));
    let khaki = m(crate::models::hex(0xb59a6a));
    let cork = m(crate::models::hex(0xc9a06a));
    let lens = mats.add(StandardMaterial {
        base_color: crate::models::hex(0x2b5fa8),
        perceptual_roughness: 0.15,
        reflectance: 0.6,
        ..default()
    });
    let holder = mats.add(crate::models::hex(0xe08a1e));
    let can = mats.add(crate::models::hex(0xc9ced3));
    let thong = mats.add(crate::models::hex(0x2f8ee8));

    let root = commands
        .spawn((
            Transform::from_xyz(dazza::HOME.x, 0.0, dazza::HOME.z),
            Visibility::default(),
            DazzaRoot,
        ))
        .id();
    let body = commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            DazzaBody,
            ChildOf(root),
        ))
        .id();
    // pop style: a bigger head with its hat and sunnies, bigger hands and feet, and a black outline
    let pop = crate::style::current() == crate::style::Style::Pop;
    let outline_mat = crate::style::outline_material(&mut mats);
    let mut part = |mesh: Mesh, mat: Handle<StandardMaterial>, mut t: Transform| {
        if pop {
            let pivot = Vec3::new(0.0, 1.4, 0.0);
            if t.translation.y >= 1.38 {
                // head, hat, sunnies, nose, moustache, corks: grow about the neck
                t.translation = pivot + (t.translation - pivot) * 1.28;
                t.scale *= 1.28;
            } else if t.translation.y < 0.2 {
                t.scale *= Vec3::new(1.25, 1.25, 1.25); // feet and thongs
            } else if (t.translation.x + 0.52).abs() < 0.04 && (t.translation.y - 0.9).abs() < 0.2 {
                t.scale *= 1.3; // the free hand
            }
        }
        let aabb = bevy::camera::primitives::MeshAabb::compute_aabb(&mesh);
        let h = meshes.add(mesh);
        commands.spawn((
            Mesh3d(h.clone()),
            MeshMaterial3d(mat),
            t,
            ChildOf(body),
        ));
        if pop && let Some(a) = aabb && Vec3::from(a.half_extents).max_element() * t.scale.max_element() > 0.04 {
            let mut o = t;
            o.scale = outline_scale_for(a.half_extents.into(), t.scale, 0.013);
            commands.spawn((
                Mesh3d(h),
                MeshMaterial3d(outline_mat.clone()),
                o,
                bevy::light::NotShadowCaster,
                ChildOf(body),
            ));
        }
    };
    part(
        Capsule3d::new(0.42, 0.5).into(),
        singlet.clone(),
        Transform::from_xyz(0.0, 0.8, 0.0),
    );
    if polished {
        // a big round belly that the singlet stretches over, and the apron riding up on it
        part(Sphere::new(0.47).into(), singlet.clone(), Transform::from_xyz(0.0, 0.68, 0.17).with_scale(Vec3::new(1.05, 0.95, 1.0)));
        part(Cuboid::new(0.62, 0.66, 0.04).into(), apron.clone(), Transform::from_xyz(0.0, 0.66, 0.64).with_rotation(Quat::from_rotation_x(-0.22)));
        part(Cuboid::new(0.7, 0.07, 0.05).into(), belt.clone(), Transform::from_xyz(0.0, 0.92, 0.58).with_rotation(Quat::from_rotation_x(-0.22)));
        // apron straps over the shoulders
        for sx in [-1.0f32, 1.0] {
            part(Cuboid::new(0.06, 0.5, 0.03).into(), apron.clone(), Transform::from_xyz(sx * 0.2, 1.12, 0.36).with_rotation(Quat::from_rotation_x(-0.5)));
        }
        // stubby shorts
        part(Cylinder::new(0.44, 0.3).into(), khaki, Transform::from_xyz(0.0, 0.36, 0.02));
    } else {
        part(
            Sphere::new(0.4).into(),
            singlet,
            Transform::from_xyz(0.0, 0.72, 0.12).with_scale(Vec3::new(1.0, 1.0, 0.9)),
        );
        part(
            Cuboid::new(0.66, 0.78, 0.04).into(),
            apron,
            Transform::from_xyz(0.0, 0.72, 0.47).with_rotation(Quat::from_rotation_x(-0.08)),
        );
        part(
            Cuboid::new(0.66, 0.07, 0.045).into(),
            belt,
            Transform::from_xyz(0.0, 0.93, 0.475),
        );
    }
    part(
        Sphere::new(0.3).into(),
        skin.clone(),
        Transform::from_xyz(0.0, 1.55, 0.0),
    );
    if polished {
        // wraparound sunnies with shiny blue lenses
        part(Cuboid::new(0.46, 0.1, 0.06).into(), dark.clone(), Transform::from_xyz(0.0, 1.6, 0.27));
        for sx in [-1.0f32, 1.0] {
            part(Cuboid::new(0.17, 0.08, 0.02).into(), lens.clone(), Transform::from_xyz(sx * 0.11, 1.6, 0.305));
        }
        // a big sunburnt nose and a bushy moustache in two halves
        part(Sphere::new(0.095).into(), nose, Transform::from_xyz(0.0, 1.49, 0.31));
        for sx in [-1.0f32, 1.0] {
            part(
                Capsule3d::new(0.045, 0.13).into(),
                hat.clone(),
                Transform::from_xyz(sx * 0.08, 1.41, 0.28).with_rotation(Quat::from_rotation_z(sx * 1.25)),
            );
        }
        // the cork hat: a wide brim, a crown, and corks dangling on strings all round
        part(Cylinder::new(0.6, 0.03).into(), hat.clone(), Transform::from_xyz(0.0, 1.8, 0.0));
        part(Cylinder::new(0.27, 0.22).into(), hat.clone(), Transform::from_xyz(0.0, 1.92, 0.0));
        for k in 0..9 {
            let a = k as f32 / 9.0 * std::f32::consts::TAU + 0.2;
            let (x, z) = (a.sin() * 0.52, a.cos() * 0.52);
            part(Cylinder::new(0.006, 0.12).into(), dark.clone(), Transform::from_xyz(x, 1.72, z));
            part(Cylinder::new(0.028, 0.075).into(), cork.clone(), Transform::from_xyz(x, 1.63, z));
        }
        // big feet in thongs
        for sx in [-1.0f32, 1.0] {
            part(Sphere::new(0.16).into(), skin.clone(), Transform::from_xyz(sx * 0.18, 0.1, 0.1).with_scale(Vec3::new(1.0, 0.5, 1.6)));
            part(Sphere::new(0.17).into(), thong.clone(), Transform::from_xyz(sx * 0.18, 0.03, 0.11).with_scale(Vec3::new(1.05, 0.15, 1.75)));
        }
        // the free hand holds a stubby in a holder
        part(Sphere::new(0.11).into(), skin.clone(), Transform::from_xyz(-0.52, 0.9, 0.12));
        part(Cylinder::new(0.065, 0.14).into(), holder, Transform::from_xyz(-0.52, 1.0, 0.2));
        part(Cylinder::new(0.05, 0.02).into(), can, Transform::from_xyz(-0.52, 1.08, 0.2));
    } else {
        part(
            Cuboid::new(0.4, 0.08, 0.05).into(),
            dark,
            Transform::from_xyz(0.0, 1.6, 0.27),
        ); // sunnies
        part(
            Sphere::new(0.07).into(),
            nose,
            Transform::from_xyz(0.0, 1.5, 0.3),
        );
        part(
            Cuboid::new(0.22, 0.06, 0.04).into(),
            hat.clone(),
            Transform::from_xyz(0.0, 1.4, 0.27),
        ); // moustache
        part(
            Cylinder::new(0.5, 0.03).into(),
            hat.clone(),
            Transform::from_xyz(0.0, 1.8, 0.0),
        ); // hat brim
        part(
            Cylinder::new(0.26, 0.2).into(),
            hat,
            Transform::from_xyz(0.0, 1.9, 0.0),
        );
        for s in [-1.0f32, 1.0] {
            part(
                Sphere::new(0.13).into(),
                boot.clone(),
                Transform::from_xyz(s * 0.17, 0.09, 0.05).with_scale(Vec3::new(1.0, 0.6, 1.4)),
            );
        }
        part(
            Sphere::new(0.11).into(),
            skin.clone(),
            Transform::from_xyz(-0.5, 0.9, 0.05),
        );
    }
    let _ = &boot;
    // the spatula arm (outlined too in the pop style: a slightly bigger black copy of each part)
    let arm = commands
        .spawn((
            Transform::from_xyz(0.5, 1.05, 0.05),
            Visibility::default(),
            DazzaArm,
            ChildOf(body),
        ))
        .id();
    let arm_parts: [(Mesh, Handle<StandardMaterial>, Transform, f32); 3] = [
        (Sphere::new(0.11).into(), skin, Transform::from_xyz(0.0, -0.15, 0.0), 0.11),
        (
            Cylinder::new(0.02, 0.4).into(),
            tool,
            Transform::from_xyz(0.0, -0.05, 0.2).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
            0.1,
        ),
        (Cuboid::new(0.16, 0.015, 0.2).into(), plate, Transform::from_xyz(0.0, -0.05, 0.48), 0.1),
    ];
    for (mesh, mat, tf, _size) in arm_parts {
        let half = bevy::camera::primitives::MeshAabb::compute_aabb(&mesh)
            .map_or(Vec3::splat(0.1), |a| Vec3::from(a.half_extents));
        let h = meshes.add(mesh);
        commands.spawn((Mesh3d(h.clone()), MeshMaterial3d(mat), tf, ChildOf(arm)));
        if pop {
            let mut o = tf;
            o.scale = outline_scale_for(half, tf.scale, 0.013);
            commands.spawn((
                Mesh3d(h),
                MeshMaterial3d(outline_mat.clone()),
                o,
                bevy::light::NotShadowCaster,
                ChildOf(arm),
            ));
        }
    }

    // name tag and speech bubble
    commands.spawn((
        Text2d::new("Dazza · BBQ"),
        TextFont {
            font_size: FontSize::Px(34.0),
            ..default()
        },
        TextColor(Color::linear_rgb(1.0, 0.62, 0.11)),
        Transform::from_xyz(0.0, 3.0, 0.0).with_scale(Vec3::splat(0.0085)),
        Floating {
            target: root,
            height: 2.5,
        },
        NameTag,
    ));
    let label = commands
        .spawn((
            Text2d::new(""),
            TextFont {
                font_size: FontSize::Px(34.0),
                ..default()
            },
            TextColor(Color::linear_rgb(0.09, 0.14, 0.09)),
            TextBackgroundColor(Color::WHITE),
            Transform::from_xyz(0.0, 3.0, 0.0).with_scale(Vec3::splat(0.0085)),
            Visibility::Hidden,
            Floating {
                target: root,
                height: 3.05,
            },
            DazzaBubble,
        ))
        .id();
    commands.insert_resource(Dazza {
        anim: DazzaAnim::default(),
        say_seen: 0,
        swing_seen: 0,
        last: Vec3::new(dazza::HOME.x, 0.0, dazza::HOME.z),
        face: 0.0,
        speed: 0.0,
        label,
    });
}

/// Keys to look at each pose (until the real game causes them). Mostly for checking the look.
fn viewer_keys(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, mut cast: ResMut<Cast>) {
    let g = &mut *game;
    if keys.just_pressed(KeyCode::F9) {
        cast.mine = cast.mine.next();
    }
    if keys.just_pressed(KeyCode::F7) {
        g.options.adult = !g.options.adult;
    }
    if keys.just_pressed(KeyCode::F8) {
        g.options.naughty = !g.options.naughty;
    }
    let dur = bbq_core::items::DOWN_TIME;
    let kinds = [
        (KeyCode::KeyJ, SlapKind::SentFlying),
        (KeyCode::KeyK, SlapKind::Cartwheel),
        (KeyCode::KeyL, SlapKind::Timber),
    ];
    for (i, (k, kind)) in kinds.iter().enumerate() {
        if keys.just_pressed(*k)
            && let Some(d) = g.dummies.get_mut(i)
        {
            d.anim.start_slap(*kind, dur);
            d.anim.tumble(V3::new(0.0, 0.0, 1.0), 6.0, 1.0);
        }
    }
    if keys.just_pressed(KeyCode::KeyN) {
        // everyone stacks it, or (if anyone is down) everyone gets up
        let anyone = g.dummies.iter().any(|d| d.body.fall_t > 0.0);
        for d in &mut g.dummies {
            if anyone {
                d.body.get_up();
            } else {
                d.body.start_fall(g.options.fall_duration);
            }
        }
    }
    if keys.just_pressed(KeyCode::F5) {
        g.options.falls_on = !g.options.falls_on;
    }
    if keys.just_pressed(KeyCode::F6) {
        g.options.drunk_mode = !g.options.drunk_mode;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        g.me.drunk.add(30.0); // a quick way to get drunk without walking to the bar
    }
    if keys.just_pressed(KeyCode::KeyM) {
        let n = g.now as usize;
        let emotes = [Emote::Taunt, Emote::Laugh, Emote::Dance];
        for (i, d) in g.dummies.iter_mut().enumerate() {
            d.anim.start_emote(emotes[(n + i) % 3]);
        }
    }
    if keys.just_pressed(KeyCode::KeyY) {
        for d in &mut g.dummies {
            d.drunk = if d.drunk < 10.0 {
                45.0
            } else if d.drunk < 60.0 {
                90.0
            } else {
                0.0
            };
        }
    }
    if keys.just_pressed(KeyCode::KeyC) {
        if let Some(d) = g.dummies.get_mut(0) {
            d.crown = !d.crown;
        }
    }
    if keys.just_pressed(KeyCode::KeyV) {
        let order = [
            None,
            Some(Team::Red),
            Some(Team::Blue),
            Some(Team::Wildcard),
        ];
        for d in &mut g.dummies {
            let idx = order.iter().position(|t| *t == d.team).unwrap_or(0);
            d.team = order[(idx + 1) % order.len()];
        }
    }
    if keys.just_pressed(KeyCode::KeyX) {
        if let Some(d) = g.dummies.get_mut(1) {
            d.smelly = !d.smelly;
        }
    }
}

fn to_bevy(q: CQuat) -> Quat {
    Quat::from_xyzw(q.x, q.y, q.z, q.w)
}

#[derive(Resource, Default)]
struct Poses(Vec<bbq_core::pose::Pose>);

fn compute_poses(
    time: Res<Time>,
    mut game: ResMut<Game>,
    player: Res<Player>,
    mut poses: ResMut<Poses>,
) {
    let dt = time.delta_secs();
    let t = game.now;
    let (px, pz) = (player.mover.x, player.mover.z);
    poses.0.clear();
    let bots_on = game.options.bots_on;
    for d in game.dummies.iter_mut() {
        // bots look where their brains point them; frozen practice blobs just watch you
        let face = if bots_on {
            d.face
        } else {
            (px - d.mover.x).atan2(pz - d.mover.z)
        };
        let inputs = Inputs {
            t,
            face,
            charging: d.bot.winding,
            charge: d.bot.wind_progress,
            vel: V3::new(d.mover.vx, d.mover.vy, d.mover.vz),
            grounded: d.mover.grounded,
            drunk: d.drunk,
            down_t: d.body.down_t,
            stun: d.body.stun,
            fallen: d.fallen,
            seated: d.seat.is_some(),
            naughty: d.naughty_t > 0.0,
            dragged_by: d.dragged,
            ..Default::default()
        };
        poses.0.push(d.anim.tick(dt, &inputs));
    }
}

fn apply_roots(
    game: Res<Game>,
    poses: Res<Poses>,
    mut q: Query<(&Blob, &mut Transform), With<BlobRoot>>,
) {
    for (b, mut tf) in &mut q {
        let (Some(d), Some(p)) = (game.dummies.get(b.0), poses.0.get(b.0)) else {
            continue;
        };
        tf.translation = Vec3::new(
            d.mover.x + p.offset.0,
            d.mover.y - d.mover.sink,
            d.mover.z + p.offset.1,
        );
        tf.rotation = Quat::from_rotation_y(p.yaw);
    }
}

fn apply_bodies(poses: Res<Poses>, mut q: Query<(&Blob, &mut Transform), With<BlobBody>>) {
    for (b, mut tf) in &mut q {
        let Some(p) = poses.0.get(b.0) else {
            continue;
        };
        tf.rotation = to_bevy(p.body_rot);
        tf.translation = Vec3::new(0.0, p.body_y, 0.0);
        tf.scale = Vec3::new(p.body_scale.0, p.body_scale.1, p.body_scale.0);
    }
}

fn apply_hands(
    poses: Res<Poses>,
    cast: Res<Cast>,
    mut right: Query<(&Blob, &mut Transform), (With<HandR>, Without<HandL>)>,
    mut left: Query<(&Blob, &mut Transform), (With<HandL>, Without<HandR>)>,
) {
    for (b, mut tf) in &mut right {
        let Some(pp) = poses.0.get(b.0) else {
            continue;
        };
        let h = pp.hand_r;
        let dx = cast.character_of(b.0).hand_x() - 0.47; // sit against this body shape
        tf.translation = Vec3::new(h.x - dx, h.y, h.z);
    }
    for (b, mut tf) in &mut left {
        let Some(pp) = poses.0.get(b.0) else {
            continue;
        };
        let h = pp.hand_l;
        let dx = cast.character_of(b.0).hand_x() - 0.47;
        tf.translation = Vec3::new(h.x + dx, h.y, h.z);
    }
}

fn apply_crown(game: Res<Game>, mut q: Query<(&Blob, &mut Visibility), With<Crown>>) {
    for (b, mut v) in &mut q {
        let Some(dd) = game.dummies.get(b.0) else {
            continue;
        };
        *v = if dd.crown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn apply_stars(
    time: Res<Time>,
    poses: Res<Poses>,
    mut q: Query<(&Blob, &mut Visibility, &mut Transform), With<Stars>>,
) {
    for (b, mut v, mut tf) in &mut q {
        let Some(pp) = poses.0.get(b.0) else {
            continue;
        };
        let on = pp.stars;
        *v = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if on {
            tf.rotate_y(time.delta_secs() * 6.0);
        }
    }
}

const BALLOON_RADIUS: f32 = 1.05;
const BALLOON_ALPHA: f32 = 0.42;

fn apply_balloon(
    game: Res<Game>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(&Blob, &mut Visibility, &MeshMaterial3d<StandardMaterial>), (With<Balloon>, Without<SwimRing>)>,
    mut rings: Query<(&Blob, &mut Visibility, &MeshMaterial3d<StandardMaterial>), (With<SwimRing>, Without<Balloon>)>,
) {
    let heist = game.rules.mode == bbq_core::GameMode::Heist;
    for (b, mut v, m) in &mut rings {
        match game.dummies.get(b.0).and_then(|d| d.team).filter(|_| heist) {
            Some(t) => {
                *v = Visibility::Inherited;
                if let Some(mut mat) = mats.get_mut(&m.0) {
                    mat.base_color = team_colour(t);
                }
            }
            None => *v = Visibility::Hidden,
        }
    }
    for (b, mut v, m) in &mut q {
        match game.dummies.get(b.0).and_then(|d| d.team).filter(|_| !heist) {
            Some(t) => {
                *v = Visibility::Inherited;
                if let Some(mut mat) = mats.get_mut(&m.0) {
                    mat.base_color = team_colour(t).with_alpha(BALLOON_ALPHA);
                }
            }
            None => *v = Visibility::Hidden,
        }
    }
}

fn apply_sash(
    game: Res<Game>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(&Blob, &mut Visibility, &MeshMaterial3d<StandardMaterial>), With<Sash>>,
) {
    let heist = game.rules.mode == bbq_core::GameMode::Heist;
    for (b, mut v, m) in &mut q {
        if heist {
            *v = Visibility::Hidden; // the swim ring is the sash there
            continue;
        }
        let Some(dd) = game.dummies.get(b.0) else {
            continue;
        };
        match dd.team {
            Some(t) => {
                *v = Visibility::Inherited;
                if let Some(mut mat) = mats.get_mut(&m.0) {
                    mat.base_color = team_colour(t);
                }
            }
            None => *v = Visibility::Hidden,
        }
    }
}

fn apply_can(
    poses: Res<Poses>,
    mut q: Query<(&Blob, &mut Visibility, &mut Transform), With<SmokoCan>>,
) {
    for (b, mut v, mut tf) in &mut q {
        let Some(p) = poses.0.get(b.0) else {
            continue;
        };
        *v = if p.can_visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        tf.rotation = Quat::from_rotation_x(p.can_pitch);
    }
}

fn apply_help_signs(game: Res<Game>, mut q: Query<(&Blob, &mut Visibility), With<HelpSign>>) {
    for (b, mut v) in &mut q {
        let Some(dd) = game.dummies.get(b.0) else {
            continue;
        };
        *v = if dd.fallen {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn apply_clouds(
    game: Res<Game>,
    time: Res<Time>,
    mut q: Query<(&Blob, &mut Visibility, &mut Transform), With<StinkCloud>>,
) {
    for (b, mut v, mut tf) in &mut q {
        let Some(d) = game.dummies.get(b.0) else {
            continue;
        };
        *v = if d.smelly {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        let k = 1.0 + 0.12 * (time.elapsed_secs() * 5.0).sin();
        tf.translation = Vec3::new(d.mover.x, d.mover.y + 1.0, d.mover.z);
        tf.scale = Vec3::splat(1.15 * k);
    }
}

fn animate_dazza(
    time: Res<Time>,
    game: Res<Game>,
    mut dz: ResMut<Dazza>,
    mut root: Query<&mut Transform, (With<DazzaRoot>, Without<DazzaBody>, Without<DazzaArm>)>,
    mut body: Query<&mut Transform, (With<DazzaBody>, Without<DazzaArm>, Without<DazzaRoot>)>,
    mut arm: Query<&mut Transform, (With<DazzaArm>, Without<DazzaBody>, Without<DazzaRoot>)>,
    mut bubble: Query<&mut Visibility, With<DazzaBubble>>,
    mut text: Query<&mut Text2d, With<DazzaBubble>>,
) {
    let dt = time.delta_secs();
    let brain = &game.life.dazza;
    // he moves in fixed ticks but is drawn every frame: ease the drawn position after the real
    // one (about 25 ms behind) so the steps do not show as judder at any frame rate. Snap if he
    // was moved far (sent home, or the round restarted).
    let target = Vec3::new(brain.pos.x, 0.0, brain.pos.z);
    let pos = if target.distance(dz.last) > 2.0 {
        target
    } else {
        dz.last + (target - dz.last) * (1.0 - (-40.0 * dt).exp())
    };
    if game.life.say_seq != dz.say_seen {
        dz.say_seen = game.life.say_seq;
        dz.anim.say();
        if let Ok(mut t) = text.get_mut(dz.label) {
            t.0 = game.life.say_text.clone();
        }
    }
    if game.life.swing_seq != dz.swing_seen {
        dz.swing_seen = game.life.swing_seq;
        dz.anim.start_swing();
    }
    let moved = pos.distance(dz.last);
    dz.last = pos;
    // he moves in fixed steps but is drawn every frame: smooth his speed so the walk does not
    // stutter when the frame rate is not a multiple of the tick rate
    let raw = if dt > 0.0 { moved / dt } else { 0.0 };
    dz.speed += (raw - dz.speed) * (1.0 - (-14.0 * dt).exp());
    let speed = dz.speed;
    let pose = dz.anim.tick(dt, brain.state, game.now, speed, moved);
    if let Ok(mut tf) = root.single_mut() {
        tf.translation = pos;
        tf.rotation = Quat::from_rotation_y(brain.face);
    }
    if let Ok(mut tf) = body.single_mut() {
        tf.rotation = Quat::from_euler(EulerRot::XYZ, pose.body_rx, 0.0, pose.body_rz);
        tf.translation = Vec3::new(0.0, pose.body_y, 0.0);
        tf.scale = Vec3::splat(pose.body_scale);
    }
    if let Ok(mut tf) = arm.single_mut() {
        tf.rotation = Quat::from_euler(EulerRot::XYZ, pose.arm.0, pose.arm.1, pose.arm.2);
    }
    if let Ok(mut v) = bubble.single_mut() {
        *v = if dz.anim.bubble_visible() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    let _ = dz.face;
}

/// Name tags, speech bubbles and signs float above their blob and face the camera.
fn face_camera(
    cam: Single<&Transform, With<EyeCamera>>,
    targets: Query<&GlobalTransform, Without<Floating>>,
    mut q: Query<(&Floating, &mut Transform), Without<EyeCamera>>,
) {
    for (f, mut tf) in &mut q {
        if let Ok(gt) = targets.get(f.target) {
            let p = gt.translation();
            tf.translation = Vec3::new(p.x, p.y + f.height, p.z);
            tf.rotation = cam.rotation;
        }
    }
}

/// How a character's eyes move: every character has its own style. The eyeballs sit on springs
/// in their sockets: every footstep kicks them, speeding up and stopping throws them back and
/// forward, and they overshoot and wobble back. The pupils slosh about inside on springs of
/// their own.
#[derive(Clone, Copy)]
struct EyeStyle {
    /// Eyeball spring stiffness and damping (low damping = more boing).
    k: f32,
    c: f32,
    /// How hard each footstep kicks the eyeballs (up and down, and in and out).
    kick: f32,
    pop: f32,
    /// How much speeding up and slowing down throws the eyeballs about.
    lag: f32,
    /// The two eyes kick on opposite footsteps (1) or together (0).
    alternate: f32,
    /// How far an eyeball may leave its socket.
    reach: f32,
    /// Squash and stretch of the eyeball as it bounces.
    squash: f32,
    /// How much the eyeball swells when it pops out (googly).
    bulge: f32,
    /// A slow side-to-side swing of both eyes with each stride (floaty).
    sway: f32,
    /// Pupil spring stiffness and damping, how far they wander, and cross-eyed (opposite ways).
    pk: f32,
    pc: f32,
    wander: f32,
    cross: bool,
    /// Quick nervous darting of the pupils.
    jitter: f32,
}

/// How strong all the eye movement is (1 = the full cartoon version). Marcus: a little less.
const EYE_STRENGTH: f32 = 0.6;

fn eye_style(c: Character) -> EyeStyle {
    let s = eye_style_full(c);
    let k = EYE_STRENGTH;
    EyeStyle {
        kick: s.kick * k,
        pop: s.pop * k,
        lag: s.lag * k,
        reach: s.reach * k,
        squash: s.squash * k,
        bulge: s.bulge * k,
        sway: s.sway * k,
        wander: s.wander * k,
        jitter: s.jitter * k,
        ..s
    }
}

fn eye_style_full(c: Character) -> EyeStyle {
    match c {
        // steady and cheerful: a firm bounce, both eyes together
        Character::Classic => EyeStyle {
            k: 240.0, c: 8.0, kick: 0.7, pop: 0.15, lag: 0.012, alternate: 0.0, reach: 0.055, squash: 0.0,
            bulge: 0.0, sway: 0.0, pk: 150.0, pc: 8.0, wander: 0.016, cross: false, jitter: 0.0,
        },
        // lazy and floaty: very soft springs, the eyes swing wide side to side and trail behind
        Character::Pear => EyeStyle {
            k: 38.0, c: 1.8, kick: 0.9, pop: 0.05, lag: 0.04, alternate: 0.4, reach: 0.1, squash: 0.0,
            bulge: 0.0, sway: 0.05, pk: 30.0, pc: 1.6, wander: 0.034, cross: false, jitter: 0.0,
        },
        // nervous and googly: stiff fast jiggles, the eyes bulge out on every step, pupils dart
        Character::Egg => EyeStyle {
            k: 600.0, c: 12.0, kick: 0.8, pop: 1.1, lag: 0.008, alternate: 1.0, reach: 0.07, squash: 0.0,
            bulge: 9.0, sway: 0.0, pk: 320.0, pc: 9.0, wander: 0.014, cross: false, jitter: 0.03,
        },
        // big cartoon boings: huge bounces on alternate steps, squash and stretch, cross-eyed
        Character::Gumdrop => EyeStyle {
            k: 120.0, c: 3.2, kick: 1.0, pop: 0.2, lag: 0.018, alternate: 1.0, reach: 0.09, squash: 2.2,
            bulge: 0.0, sway: 0.0, pk: 80.0, pc: 3.0, wander: 0.026, cross: true, jitter: 0.0,
        },
    }
}

/// Per blob: the step phase, the last footstep counted, and last frame's velocity (local).
#[derive(Default, Clone, Copy)]
struct EyeDrive {
    phase: f32,
    step: i32,
    vel: Vec3,
    grounded: bool,
    /// The belly's bounce: how far it sags (down and forward) and how far it swells or squashes,
    /// each on a soft spring.
    sag: f32,
    sag_v: f32,
    swell: f32,
    swell_v: f32,
    /// The body's lean (forward and sideways) and its squash on impacts, each on a spring.
    lean: f32,
    lean_v: f32,
    roll: f32,
    roll_v: f32,
    sq: f32,
    sq_v: f32,
    /// Last frame's facing and stun, to see turns and hits.
    face: f32,
    stunned: bool,
}

/// Walking makes the eyeballs bounce and pop (each character in its own style), the pupils
/// slosh about on springs and the big feet step. Dizzy blobs get swirling, cross-eyed pupils.
fn animate_face_and_feet(
    time: Res<Time>,
    game: Res<Game>,
    cast: Res<Cast>,
    mut drive: Local<Vec<EyeDrive>>,
    mut eyes: Query<(&Blob, &mut EyePart, &mut Transform), (Without<PupilPart>, Without<FootPart>)>,
    mut pupils: Query<(&Blob, &mut PupilPart, &mut Transform), (Without<EyePart>, Without<FootPart>)>,
    mut feet: Query<(&Blob, &FootPart, &mut Transform), (Without<EyePart>, Without<PupilPart>)>,
    mut bellies: Query<(&Blob, &mut bevy::mesh::morph::MorphWeights), With<BellyNode>>,
    mut bodies: Query<(&Blob, &mut Transform), (With<BlobBody>, Without<EyePart>, Without<PupilPart>, Without<FootPart>, Without<HandR>, Without<HandL>)>,
    mut hands_r: Query<(&Blob, &mut Transform), (With<HandR>, Without<HandL>, Without<EyePart>, Without<PupilPart>, Without<FootPart>, Without<BlobBody>)>,
    mut hands_l: Query<(&Blob, &mut Transform), (With<HandL>, Without<HandR>, Without<EyePart>, Without<PupilPart>, Without<FootPart>, Without<BlobBody>)>,
) {
    let dt = time.delta_secs().clamp(0.0001, 0.05);
    let t = time.elapsed_secs();
    let n = game.dummies.len();
    drive.resize(n, EyeDrive::default());
    // what each blob's body is doing: speed, a footstep kick, a jolt from speeding up or stopping
    // (in the blob's own frame: x sideways, y up, z forward), and landing
    let mut speed = vec![0.0f32; n];
    let mut kick = vec![0.0f32; n];
    let mut kick_side = vec![1.0f32; n];
    let mut accel = vec![Vec3::ZERO; n];
    for (i, d) in game.dummies.iter().enumerate() {
        let dr = &mut drive[i];
        let s = (d.mover.vx * d.mover.vx + d.mover.vz * d.mover.vz).sqrt();
        speed[i] = if d.mover.grounded { s } else { 0.0 };
        dr.phase += speed[i] * dt * 1.7;
        let step = (dr.phase / std::f32::consts::PI).floor() as i32;
        if step != dr.step {
            kick[i] = (speed[i] / 5.0).min(1.4);
            kick_side[i] = if step % 2 == 0 { 1.0 } else { -1.0 };
            dr.step = step;
        }
        let local = Quat::from_rotation_y(-d.face) * Vec3::new(d.mover.vx, d.mover.vy, d.mover.vz);
        accel[i] = (local - dr.vel) / dt;
        if d.mover.grounded && !dr.grounded {
            kick[i] = kick[i].max((-dr.vel.y / 6.0).clamp(0.4, 1.6)); // a landing
        }
        // the belly: every footstep drops it, it springs back up and jiggles; speeding up
        // squashes it back, stopping throws it forward, a landing squashes it hard
        if kick[i] > 0.0 {
            dr.sag_v += kick[i] * 4.2;
            dr.swell_v -= kick[i] * 1.2;
        }
        let fwd = accel[i].z.clamp(-40.0, 40.0);
        dr.swell_v -= fwd * 0.02;
        dr.sag_v += (dr.sag * -150.0 - dr.sag_v * 6.0) * dt;
        dr.swell_v += (dr.swell * -110.0 - dr.swell_v * 5.0) * dt;
        dr.sag = (dr.sag + dr.sag_v * dt).clamp(-0.6, 1.0);
        dr.swell = (dr.swell + dr.swell_v * dt).clamp(-0.35, 0.35);
        // body lean: forward when speeding up, back when stopping, a roll into turns (C10.5)
        let fwd_lean = (accel[i].z * 0.0045 + speed[i] * 0.012).clamp(-0.22, 0.3);
        let yaw_rate = {
            let mut dy = d.face - dr.face;
            while dy > std::f32::consts::PI {
                dy -= std::f32::consts::TAU;
            }
            while dy < -std::f32::consts::PI {
                dy += std::f32::consts::TAU;
            }
            (dy / dt).clamp(-8.0, 8.0)
        };
        let roll_target = (yaw_rate * speed[i] * 0.006 - accel[i].x * 0.004).clamp(-0.18, 0.18);
        dr.lean_v += ((fwd_lean - dr.lean) * 90.0 - dr.lean_v * 11.0) * dt;
        dr.lean += dr.lean_v * dt;
        dr.roll_v += ((roll_target - dr.roll) * 90.0 - dr.roll_v * 11.0) * dt;
        dr.roll += dr.roll_v * dt;
        // squash on hard landings and when something big hits (C10.8)
        if d.mover.grounded && !dr.grounded {
            dr.sq_v += (-dr.vel.y / 8.0).clamp(0.0, 1.6) * 9.0;
        }
        let stunned = d.body.stun > 0.0 || d.body.is_down();
        if stunned && !dr.stunned {
            dr.sq_v += 7.0;
        }
        dr.stunned = stunned;
        dr.sq_v += (dr.sq * -190.0 - dr.sq_v * 9.0) * dt;
        dr.sq = (dr.sq + dr.sq_v * dt).clamp(-0.22, 0.28);
        dr.face = d.face;
        dr.vel = local;
        dr.grounded = d.mover.grounded;
    }
    for (b, mut tf) in &mut bodies {
        let Some(dr) = drive.get(b.0) else { continue };
        // each blob stands a little crooked, one shoulder lower than the other (C9.4)
        let crooked = ((b.0 * 53 % 9) as f32 - 4.0) * 0.007;
        tf.rotation *= Quat::from_rotation_x(dr.lean) * Quat::from_rotation_z(dr.roll + crooked);
        tf.scale *= Vec3::new(1.0 + dr.sq * 0.55, 1.0 - dr.sq, 1.0 + dr.sq * 0.55);
    }
    for (b, mut w) in &mut bellies {
        let Some(dr) = drive.get(b.0) else { continue };
        let size = game.dummies.get(b.0).map_or(1.0, |d| d.belly);
        let ws = w.weights_mut();
        if ws.len() >= 2 {
            ws[0] = size.powf(0.8) * (1.0 + dr.swell);
            // with outlines or a bigger body the flop is harder to see, so the styles show it a bit more
            let flop = match crate::style::current() {
                crate::style::Style::Current => 1.0,
                crate::style::Style::Pop => 4.4,
                crate::style::Style::Clay => 3.0,
            };
            ws[1] = dr.sag * size.max(0.3) * flop;
        }
    }
    let walk = |i: usize| (speed.get(i).copied().unwrap_or(0.0) / 4.0).clamp(0.0, 1.0);
    // each blob also differs a little from the others of its kind
    let quirk = |i: usize| 1.0 + ((i * 37 % 11) as f32 - 5.0) * 0.04;
    let mut eye_off: std::collections::HashMap<(usize, bool), Vec3> = Default::default();
    for (b, mut e, mut tf) in &mut eyes {
        if b.0 >= n {
            continue;
        }
        let st = eye_style(cast.character_of(b.0));
        let q = quirk(b.0);
        // a footstep (or a landing) kicks the eyeball down, then up it springs; alternate-step
        // characters kick only the eye on that step's side harder
        if kick[b.0] > 0.0 {
            let side = if e.right { 1.0 } else { -1.0 };
            let bias = 1.0 + st.alternate * 0.8 * side * kick_side[b.0];
            e.vel.y -= kick[b.0] * st.kick * bias.max(0.15) * q;
            e.vel.z += kick[b.0] * st.pop * if e.right { 1.0 } else { 0.6 } * q;
            e.vel.x += kick[b.0] * st.kick * 0.25 * side * kick_side[b.0];
        }
        // speeding up throws them back, stopping throws them forward; turning swings them out
        let a = accel[b.0].clamp_length_max(60.0);
        e.vel -= Vec3::new(a.x, 0.0, a.z) * st.lag * dt * 60.0 * 0.05;
        // a fast turn leaves the eyes behind for a moment (C10.4)
        e.vel.x += drive[b.0].roll_v * st.lag * 6.0;
        // the spring back into the socket
        let force = -e.off * st.k * q - e.vel * st.c;
        e.vel += force * dt;
        let v = e.vel;
        e.off += v * dt;
        e.off = e.off.clamp_length_max(st.reach);
        let idle = (t * 1.3 + if e.right { 1.0 } else { 0.0 } + b.0 as f32).sin() * 0.003;
        // floaty eyes also drift side to side with the stride, a beat behind
        let sway = (drive[b.0].phase * 0.5 - 0.8).sin() * st.sway * walk(b.0);
        let off = e.off + Vec3::new(sway, idle, 0.0);
        tf.translation = e.base + off;
        if st.squash > 0.0 {
            // squash when falling into the socket, stretch when flying up out of it
            let sq = (1.0 + e.vel.y * st.squash * 0.12).clamp(0.7, 1.35);
            tf.scale = Vec3::new(1.0 / sq.sqrt(), sq, 1.0 / sq.sqrt());
        } else if st.bulge > 0.0 {
            // googly: the eyeball swells as it pops out of the head
            tf.scale = Vec3::splat((1.0 + e.off.z.max(0.0) * st.bulge).min(1.5));
        }
        eye_off.insert((b.0, e.right), off);
    }
    for (b, mut p, mut tf) in &mut pupils {
        if b.0 >= n {
            continue;
        }
        let Some(d) = game.dummies.get(b.0) else {
            continue;
        };
        let st = eye_style(cast.character_of(b.0));
        let (ph, k) = (drive[b.0].phase * quirk(b.0), walk(b.0));
        let side = if p.right { 1.0 } else { 0.0 };
        let dir = if p.right && st.cross { -1.0 } else { 1.0 };
        let mut target = Vec2::new(
            (ph + side * 1.3).sin() * st.wander * k * dir,
            -(ph * 2.0 + side).cos() * st.wander * k,
        );
        if st.jitter > 0.0 {
            // twitchy: quick darts this way and that while moving
            let flick = ((t * 7.0 + side * 3.0 + b.0 as f32).floor() * 12.9898).sin();
            target += Vec2::new(flick, (flick * 7.3).sin()) * st.jitter * (0.3 + k);
        }
        if d.body.stun > 0.0 {
            // seeing stars: swirling, one pupil a beat behind the other
            let a = t * 9.0 + side * 2.2;
            target = Vec2::new(a.cos(), a.sin()) * 0.024;
        } else if d.drunk > 40.0 {
            target += Vec2::new((t * 1.7 + side * 3.0).sin(), (t * 2.3 + side).cos()) * 0.016;
        }
        // footsteps and jolts shake the pupils too
        if kick[b.0] > 0.0 {
            p.vel.y += kick[b.0] * 0.9;
        }
        let acc = (target - p.pos) * st.pk - p.vel * st.pc;
        p.vel += acc * dt;
        let v = p.vel;
        p.pos += v * dt;
        p.pos = p.pos.clamp_length_max(0.036);
        // the pupil rides on its eyeball
        let eo = eye_off.get(&(b.0, p.right)).copied().unwrap_or(Vec3::ZERO);
        tf.translation = p.base + eo + Vec3::new(p.pos.x, p.pos.y, 0.0);
    }
    for (b, f, mut tf) in &mut feet {
        if b.0 >= n {
            continue;
        }
        let (ph, k) = (drive[b.0].phase, walk(b.0));
        let a = ph + if f.right { std::f32::consts::PI } else { 0.0 };
        tf.translation = f.base + Vec3::new(0.0, a.cos().max(0.0) * 0.06 * k, a.sin() * 0.11 * k);
        if f.thong {
            // the thong lifts with the foot and slaps back down on the heel (C10.7)
            let lift = a.cos().max(0.0);
            let slap = (-a.sin()).max(0.0) * (1.0 - lift);
            tf.rotation = Quat::from_rotation_x(-(lift * 0.5 + slap * 0.22) * k);
        }
    }
    // arms swing opposite the feet while walking (C10.6); the throwing hand stays up when winding up
    for (b, mut tf) in &mut hands_l {
        let Some(dr) = drive.get(b.0) else { continue };
        let k = walk(b.0);
        let ph = dr.phase;
        tf.translation += Vec3::new(0.0, (ph.cos()).abs() * 0.02 * k, ph.sin() * 0.16 * k);
    }
    for (b, mut tf) in &mut hands_r {
        let Some(dr) = drive.get(b.0) else { continue };
        let Some(d) = game.dummies.get(b.0) else { continue };
        let busy = d.bot.winding || d.body.stun > 0.0 || d.body.is_down() || d.seat.is_some() || d.bot.swing_t > 0.0;
        let k = if busy { 0.0 } else { walk(b.0) };
        let ph = dr.phase + std::f32::consts::PI;
        tf.translation += Vec3::new(0.0, (ph.cos()).abs() * 0.02 * k, ph.sin() * 0.16 * k);
        // the lower shoulder hangs a touch lower
        tf.translation.y -= ((b.0 * 53 % 9) as f32 - 4.0) * 0.004;
    }
}

/// The scale of an outline copy: every axis grows by `line` metres on each side whatever its
/// size, so thin flat things (a hat brim, a spatula) get a line as well as fat round ones.
fn outline_scale_for(half: Vec3, scale: Vec3, line: f32) -> Vec3 {
    let g = |h: f32, s: f32| 1.0 + line / (h.max(0.004) * s.max(0.05));
    Vec3::new(g(half.x, scale.x), g(half.y, scale.y), g(half.z, scale.z)) * scale
}
