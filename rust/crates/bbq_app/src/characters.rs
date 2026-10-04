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
}
/// A pupil: it sloshes about inside the eye on a bouncy spring.
#[derive(Component)]
struct PupilPart {
    right: bool,
    base: Vec3,
    pos: Vec2,
    vel: Vec2,
}
/// A foot (or its thong or strap): it steps as the blob walks.
#[derive(Component)]
struct FootPart {
    right: bool,
    base: Vec3,
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
}

impl Cast {
    /// The character shown on practice blob `i`: blob 0 wears your pick, the others the next ones.
    fn character_of(&self, i: usize) -> Character {
        let start = Character::ALL
            .iter()
            .position(|c| *c == self.mine)
            .unwrap_or(0);
        Character::ALL[(start + i) % Character::ALL.len()]
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
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(character.model()))),
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
                  mut commands: Commands| {
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
                    match name.as_str() {
                        "EyeL" | "EyeR" => {
                            commands.entity(node).insert((
                                EyePart { right: name.as_str() == "EyeR", base },
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
                                FootPart { right: name.as_str().ends_with('R'), base },
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
    if *shown == Some(cast.mine) {
        return;
    }
    if shown.is_none() {
        *shown = Some(cast.mine); // the first models were spawned at startup
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
    mut shown: Local<Option<usize>>,
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
    let n = game.dummies.len();
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
    let can = meshes.add(Cylinder::new(0.035, 0.12));
    let tag_font = TextFont {
        font_size: FontSize::Px(34.0),
        ..default()
    };

    for (i, d) in game.dummies.iter().enumerate() {
        let colour = BLOB_COLOURS[i % BLOB_COLOURS.len()];
        let body_mat = mats.add(colour);
        let head_mat = mats.add(lighter(colour));
        let foot_mat = mats.add(darker(colour));
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
            Text2d::new(BLOB_NAMES[i % BLOB_NAMES.len()]),
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
) {
    let mut m = |c: Color| mats.add(c);
    let skin = m(Color::linear_rgb(0.91, 0.71, 0.55));
    let singlet = m(Color::linear_rgb(0.11, 0.25, 0.45));
    let apron = m(Color::linear_rgb(0.96, 0.97, 0.95));
    let belt = m(Color::linear_rgb(0.7, 0.23, 0.17));
    let hat = m(Color::linear_rgb(0.48, 0.35, 0.23));
    let dark = m(Color::linear_rgb(0.07, 0.07, 0.07));
    let nose = m(Color::linear_rgb(0.85, 0.56, 0.42));
    let boot = m(Color::linear_rgb(0.23, 0.17, 0.1));
    let tool = m(Color::linear_rgb(0.16, 0.16, 0.16));
    let plate = m(Color::linear_rgb(0.85, 0.87, 0.88));

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
    let mut part = |mesh: Mesh, mat: Handle<StandardMaterial>, t: Transform| {
        commands.spawn((
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(mat),
            t,
            ChildOf(body),
        ));
    };
    part(
        Capsule3d::new(0.42, 0.5).into(),
        singlet.clone(),
        Transform::from_xyz(0.0, 0.8, 0.0),
    );
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
    part(
        Sphere::new(0.3).into(),
        skin.clone(),
        Transform::from_xyz(0.0, 1.55, 0.0),
    );
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
    // the spatula arm
    let arm = commands
        .spawn((
            Transform::from_xyz(0.5, 1.05, 0.05),
            Visibility::default(),
            DazzaArm,
            ChildOf(body),
        ))
        .id();
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.11))),
        MeshMaterial3d(skin),
        Transform::from_xyz(0.0, -0.15, 0.0),
        ChildOf(arm),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(0.02, 0.4))),
        MeshMaterial3d(tool),
        Transform::from_xyz(0.0, -0.05, 0.2)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        ChildOf(arm),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.16, 0.015, 0.2))),
        MeshMaterial3d(plate),
        Transform::from_xyz(0.0, -0.05, 0.48),
        ChildOf(arm),
    ));

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
    mut q: Query<(&Blob, &mut Visibility, &MeshMaterial3d<StandardMaterial>), With<Balloon>>,
) {
    for (b, mut v, m) in &mut q {
        match game.dummies.get(b.0).and_then(|d| d.team) {
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
    for (b, mut v, m) in &mut q {
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
    let pos = Vec3::new(brain.pos.x, 0.0, brain.pos.z);
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
    let speed = if dt > 0.0 { moved / dt } else { 0.0 };
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

/// How a character's eyes move: every character has its own style.
#[derive(Clone, Copy)]
struct EyeStyle {
    /// How fast the bob goes, compared with the walk.
    rate: f32,
    /// How far each eyeball bobs up and down (left, right).
    bob: (f32, f32),
    /// How far the right eyeball pops in and out.
    pop: f32,
    /// How far the pupils wander in the eye, sideways and up and down.
    drift: (f32, f32),
    /// Pupil spring: stiffness and damping (low damping = more wobble).
    k: f32,
    c: f32,
    /// Pupils swing opposite ways (cross-eyed) instead of together.
    cross: f32,
    /// Quick nervous jitter on top.
    jitter: f32,
}

fn eye_style(c: Character) -> EyeStyle {
    match c {
        // steady and cheerful
        Character::Classic => EyeStyle { rate: 1.0, bob: (0.020, 0.016), pop: 0.012, drift: (0.014, 0.022), k: 140.0, c: 7.5, cross: 0.0, jitter: 0.0 },
        // lazy and floaty: slow big sloshes, both eyes drifting together
        Character::Pear => EyeStyle { rate: 0.65, bob: (0.026, 0.026), pop: 0.0, drift: (0.026, 0.020), k: 60.0, c: 3.0, cross: 0.0, jitter: 0.0 },
        // nervous and googly: fast little bobs, eyes popping, twitchy pupils
        Character::Egg => EyeStyle { rate: 1.7, bob: (0.011, 0.013), pop: 0.022, drift: (0.016, 0.014), k: 230.0, c: 9.0, cross: 0.0, jitter: 0.010 },
        // big bouncy cartoon: huge alternating bounces and cross-eyed pupils
        Character::Gumdrop => EyeStyle { rate: 1.0, bob: (0.036, 0.030), pop: 0.016, drift: (0.024, 0.026), k: 95.0, c: 3.8, cross: 1.0, jitter: 0.0 },
    }
}

/// Walking makes the eyeballs bob and pop (each character in its own style, each eye on its own
/// beat), the pupils slosh about on springs and the big feet step. Dizzy blobs get swirling,
/// cross-eyed pupils.
fn animate_face_and_feet(
    time: Res<Time>,
    game: Res<Game>,
    cast: Res<Cast>,
    mut phases: Local<Vec<f32>>,
    mut eyes: Query<(&Blob, &EyePart, &mut Transform), (Without<PupilPart>, Without<FootPart>)>,
    mut pupils: Query<(&Blob, &mut PupilPart, &mut Transform), (Without<EyePart>, Without<FootPart>)>,
    mut feet: Query<(&Blob, &FootPart, &mut Transform), (Without<EyePart>, Without<PupilPart>)>,
) {
    let dt = time.delta_secs().min(0.05);
    let t = time.elapsed_secs();
    let n = game.dummies.len();
    phases.resize(n, 0.0);
    // how fast each blob is walking, and its step phase
    let mut speed = vec![0.0f32; n];
    for (i, d) in game.dummies.iter().enumerate() {
        let s = (d.mover.vx * d.mover.vx + d.mover.vz * d.mover.vz).sqrt();
        speed[i] = if d.mover.grounded { s } else { s * 0.3 };
        phases[i] += speed[i] * dt * 1.7;
    }
    let walk = |i: usize| (speed.get(i).copied().unwrap_or(0.0) / 4.0).clamp(0.0, 1.0);
    // each blob also has its own small quirks on top of its character's style
    let quirk = |i: usize| 1.0 + ((i * 37 % 11) as f32 - 5.0) * 0.03;
    for (b, e, mut tf) in &mut eyes {
        if b.0 >= n {
            continue;
        }
        let st = eye_style(cast.character_of(b.0));
        let (ph, k) = (phases[b.0] * st.rate * quirk(b.0), walk(b.0));
        // the two eyes bob on different beats; the right one also pops in and out
        let (y, z) = if e.right {
            ((ph * 2.0 + 1.9).sin() * st.bob.1 * k, (ph * 2.7).sin() * st.pop * k)
        } else {
            ((ph * 2.0).sin() * st.bob.0 * k, 0.0)
        };
        let idle = (t * 1.3 * st.rate + if e.right { 1.0 } else { 0.0 } + b.0 as f32).sin() * 0.004;
        tf.translation = e.base + Vec3::new(0.0, y + idle, z);
    }
    for (b, mut p, mut tf) in &mut pupils {
        if b.0 >= n {
            continue;
        }
        let Some(d) = game.dummies.get(b.0) else {
            continue;
        };
        let st = eye_style(cast.character_of(b.0));
        let (ph, k) = (phases[b.0] * st.rate * quirk(b.0), walk(b.0));
        let side = if p.right { 1.0 } else { 0.0 };
        // cross-eyed characters send the two pupils opposite ways
        let dir = if p.right && st.cross > 0.0 { -1.0 } else { 1.0 };
        let mut target = Vec2::new(
            (ph + side * 1.3).sin() * st.drift.0 * k * dir,
            -(ph * 2.0 + side).cos() * st.drift.1 * k,
        );
        if st.jitter > 0.0 {
            // twitchy: quick little random-looking flicks while moving
            target += Vec2::new((t * 31.0 + side * 5.0 + b.0 as f32).sin(), (t * 27.0 + side * 3.0).cos()) * st.jitter * (0.3 + k);
        }
        if d.body.stun > 0.0 {
            // seeing stars: swirling, one pupil a beat behind the other
            let a = t * 9.0 + side * 2.2;
            target = Vec2::new(a.cos(), a.sin()) * 0.024;
        } else if d.drunk > 40.0 {
            target += Vec2::new((t * 1.7 + side * 3.0).sin(), (t * 2.3 + side).cos()) * 0.016;
        }
        // an underdamped spring: they overshoot and wobble back
        let acc = (target - p.pos) * st.k - p.vel * st.c;
        p.vel += acc * dt;
        let v = p.vel;
        p.pos += v * dt;
        p.pos = p.pos.clamp_length_max(0.034);
        tf.translation = p.base + Vec3::new(p.pos.x, p.pos.y, 0.0);
    }
    for (b, f, mut tf) in &mut feet {
        if b.0 >= n {
            continue;
        }
        let (ph, k) = (phases[b.0], walk(b.0));
        let a = ph + if f.right { std::f32::consts::PI } else { 0.0 };
        tf.translation = f.base + Vec3::new(0.0, a.cos().max(0.0) * 0.06 * k, a.sin() * 0.11 * k);
    }
}
