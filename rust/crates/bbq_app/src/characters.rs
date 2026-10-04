//! The blob characters (same design as the browser game: a capsule body, a lighter round head,
//! big eyes, squashed feet, round hands), their animation, and the things that go with them:
//! name tags, leader crown, stun stars, team sash, HELP ME sign, stink cloud and speech
//! bubbles. Also Dazza the BBQ cook with his six states.
//!
//! Code-made shapes for now; a rigged model from Blender can replace them later without
//! changing the animation maths in `bbq_core::pose`.

use bbq_core::dazza::{self, DazzaAnim, DazzaState};
use bbq_core::pose::{Emote, Inputs, SlapKind};
use bbq_core::teams::Team;
use bbq_core::vec::{Quat as CQuat, V3};
use bevy::prelude::*;

use crate::game::Game;
use crate::player::{EyeCamera, Player};

const TEAM_COLOURS: [(Team, Color); 4] = [
    (Team::Red, Color::srgb(0.85, 0.2, 0.18)),
    (Team::Blue, Color::srgb(0.18, 0.4, 0.85)),
    (Team::Green, Color::srgb(0.2, 0.7, 0.3)),
    (Team::Yellow, Color::srgb(0.95, 0.8, 0.2)),
];

pub fn team_colour(t: Team) -> Color {
    match t {
        Team::Wildcard => Color::srgb(0.7, 0.3, 0.9),
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
#[derive(Component)]
struct Crown;
#[derive(Component)]
struct Stars;
#[derive(Component)]
struct Sash;
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
    state_idx: usize,
    anim: DazzaAnim,
    last: Vec3,
    face: f32,
    line: usize,
    label: Entity,
}

const BLOB_COLOURS: [Color; 3] = [
    Color::srgb(0.9, 0.3, 0.25),
    Color::srgb(0.2, 0.7, 0.65),
    Color::srgb(0.95, 0.75, 0.2),
];
const BLOB_NAMES: [&str; 3] = ["Bruce", "Sheila", "Davo"];

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_blobs, spawn_dazza))
            .init_resource::<Poses>()
            .add_systems(
                Update,
                (
                    viewer_keys,
                    compute_poses,
                    apply_roots,
                    apply_bodies,
                    apply_hands,
                    apply_crown,
                    apply_stars,
                    apply_sash,
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

fn lighter(c: Color) -> Color {
    let l = c.to_linear();
    Color::linear_rgb(
        l.red + (1.0 - l.red) * 0.35,
        l.green + (1.0 - l.green) * 0.35,
        l.blue + (1.0 - l.blue) * 0.35,
    )
}

fn darker(c: Color) -> Color {
    let l = c.to_linear();
    Color::linear_rgb(l.red * 0.6, l.green * 0.6, l.blue * 0.6)
}

fn spawn_blobs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    game: Res<Game>,
) {
    let white = mats.add(Color::WHITE);
    let black = mats.add(Color::srgb(0.07, 0.07, 0.07));
    let gold = mats.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.81, 0.2),
        emissive: LinearRgba::new(0.27, 0.2, 0.0, 1.0),
        cull_mode: None,
        ..default()
    });
    let star_mat = mats.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.88, 0.4),
        emissive: LinearRgba::new(0.53, 0.4, 0.0, 1.0),
        ..default()
    });
    let eye = meshes.add(Sphere::new(0.085));
    let pupil = meshes.add(Sphere::new(0.042));
    let foot = meshes.add(Sphere::new(0.13));
    let hand = meshes.add(Sphere::new(0.1));
    let torso = meshes.add(Capsule3d::new(0.36, 0.55));
    let head = meshes.add(Sphere::new(0.3));
    let crown_ring = meshes.add(Cylinder::new(0.21, 0.13));
    let crown_pt = meshes.add(Cone {
        radius: 0.05,
        height: 0.14,
    });
    let star = meshes.add(Sphere::new(0.07).mesh().ico(1).unwrap());
    let sash = meshes.add(Torus::new(0.32, 0.43));
    let can = meshes.add(Cylinder::new(0.035, 0.12));
    let tag_font = TextFont {
        font_size: FontSize::Px(34.0),
        ..default()
    };

    for (i, d) in game.dummies.iter().enumerate() {
        let colour = BLOB_COLOURS[i % 3];
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

        commands.spawn((
            Mesh3d(torso.clone()),
            MeshMaterial3d(body_mat),
            Transform::from_xyz(0.0, 0.78, 0.0),
            ChildOf(body),
        ));
        commands.spawn((
            Mesh3d(head.clone()),
            MeshMaterial3d(head_mat.clone()),
            Transform::from_xyz(0.0, 1.5, 0.0),
            ChildOf(body),
        ));
        for s in [-1.0f32, 1.0] {
            commands.spawn((
                Mesh3d(eye.clone()),
                MeshMaterial3d(white.clone()),
                Transform::from_xyz(s * 0.11, 1.56, 0.24),
                ChildOf(body),
            ));
            commands.spawn((
                Mesh3d(pupil.clone()),
                MeshMaterial3d(black.clone()),
                Transform::from_xyz(s * 0.11, 1.56, 0.315),
                ChildOf(body),
            ));
            commands.spawn((
                Mesh3d(foot.clone()),
                MeshMaterial3d(foot_mat.clone()),
                Transform::from_xyz(s * 0.16, 0.09, 0.04).with_scale(Vec3::new(1.0, 0.6, 1.4)),
                ChildOf(body),
            ));
        }
        let hand_r = commands
            .spawn((
                Mesh3d(hand.clone()),
                MeshMaterial3d(head_mat.clone()),
                Transform::from_xyz(-0.47, 0.88, 0.05),
                HandR,
                Blob(i),
                ChildOf(body),
            ))
            .id();
        commands.spawn((
            Mesh3d(can.clone()),
            MeshMaterial3d(mats.add(Color::srgb(0.85, 0.6, 0.1))),
            Transform::from_xyz(0.0, 0.12, 0.08),
            Visibility::Hidden,
            SmokoCan,
            Blob(i),
            ChildOf(hand_r),
        ));
        commands.spawn((
            Mesh3d(hand.clone()),
            MeshMaterial3d(head_mat.clone()),
            Transform::from_xyz(0.47, 0.88, 0.05),
            HandL,
            Blob(i),
            ChildOf(body),
        ));

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
            Text2d::new(BLOB_NAMES[i % 3]),
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
            TextColor(Color::srgb(0.84, 0.18, 0.13)),
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
                base_color: Color::srgba(0.72, 0.81, 0.44, 0.28),
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
    let skin = m(Color::srgb(0.91, 0.71, 0.55));
    let singlet = m(Color::srgb(0.11, 0.25, 0.45));
    let apron = m(Color::srgb(0.96, 0.97, 0.95));
    let belt = m(Color::srgb(0.7, 0.23, 0.17));
    let hat = m(Color::srgb(0.48, 0.35, 0.23));
    let dark = m(Color::srgb(0.07, 0.07, 0.07));
    let nose = m(Color::srgb(0.85, 0.56, 0.42));
    let boot = m(Color::srgb(0.23, 0.17, 0.1));
    let tool = m(Color::srgb(0.16, 0.16, 0.16));
    let plate = m(Color::srgb(0.85, 0.87, 0.88));

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
        TextColor(Color::srgb(1.0, 0.62, 0.11)),
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
            TextColor(Color::srgb(0.09, 0.14, 0.09)),
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
        state_idx: 0,
        anim: DazzaAnim::default(),
        last: Vec3::new(dazza::HOME.x, 0.0, dazza::HOME.z),
        face: 0.0,
        line: 0,
        label,
    });
}

/// Keys to look at each pose (until the real game causes them). Mostly for checking the look.
fn viewer_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut dazza: ResMut<Dazza>,
    mut text: Query<&mut Text2d, With<DazzaBubble>>,
) {
    let g = &mut *game;
    let dur = bbq_core::items::DOWN_TIME;
    let kinds = [
        (KeyCode::KeyJ, SlapKind::SentFlying),
        (KeyCode::KeyK, SlapKind::Cartwheel),
        (KeyCode::KeyL, SlapKind::Timber),
    ];
    for (i, (k, kind)) in kinds.iter().enumerate() {
        if keys.just_pressed(*k) {
            g.dummies[i].anim.start_slap(*kind, dur);
            g.dummies[i].anim.tumble(V3::new(0.0, 0.0, 1.0), 6.0, 1.0);
        }
    }
    if keys.just_pressed(KeyCode::KeyN) {
        for d in &mut g.dummies {
            d.fallen = !d.fallen;
        }
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
        g.dummies[0].crown = !g.dummies[0].crown;
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
        g.dummies[1].smelly = !g.dummies[1].smelly;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        for d in &mut g.dummies {
            d.body.apply_hit(1.5, None);
            d.anim.dizzy = 1.5;
        }
    }
    if keys.just_pressed(KeyCode::KeyZ) {
        dazza.state_idx = (dazza.state_idx + 1) % DazzaState::ALL.len();
        dazza.line += 1;
        let (state, line) = match DazzaState::ALL[dazza.state_idx] {
            DazzaState::Cook => ("Cook", "Snags are nearly done!"),
            DazzaState::Angry => ("Angry", "OI! Hrmblgrr MY SNAGS!"),
            DazzaState::Chase => ("Chase", "RAAAAH! COME 'ERE!"),
            DazzaState::Return => ("Return", "Hrrmph. Perfect sear."),
            DazzaState::Ko => ("Ko", "Urrrgh..."),
            DazzaState::Stunned => ("Stunned", "Me EYE! Ya mongrel!"),
        };
        dazza.anim.say();
        if let Ok(mut t) = text.get_mut(dazza.label) {
            t.0 = format!("[{state}] {line}");
        }
    }
    if keys.just_pressed(KeyCode::KeyH) {
        dazza.anim.start_swing();
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
    for d in game.dummies.iter_mut() {
        let face = (px - d.mover.x).atan2(pz - d.mover.z);
        let inputs = Inputs {
            t,
            face,
            vel: V3::new(d.mover.vx, d.mover.vy, d.mover.vz),
            grounded: d.mover.grounded,
            drunk: d.drunk,
            down_t: d.body.down_t,
            stun: d.body.stun,
            fallen: d.fallen,
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
        let (d, p) = (&game.dummies[b.0], &poses.0[b.0]);
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
        let p = &poses.0[b.0];
        tf.rotation = to_bevy(p.body_rot);
        tf.translation = Vec3::new(0.0, p.body_y, 0.0);
        tf.scale = Vec3::new(p.body_scale.0, p.body_scale.1, p.body_scale.0);
    }
}

fn apply_hands(
    poses: Res<Poses>,
    mut right: Query<(&Blob, &mut Transform), (With<HandR>, Without<HandL>)>,
    mut left: Query<(&Blob, &mut Transform), (With<HandL>, Without<HandR>)>,
) {
    for (b, mut tf) in &mut right {
        let h = poses.0[b.0].hand_r;
        tf.translation = Vec3::new(h.x, h.y, h.z);
    }
    for (b, mut tf) in &mut left {
        let h = poses.0[b.0].hand_l;
        tf.translation = Vec3::new(h.x, h.y, h.z);
    }
}

fn apply_crown(game: Res<Game>, mut q: Query<(&Blob, &mut Visibility), With<Crown>>) {
    for (b, mut v) in &mut q {
        *v = if game.dummies[b.0].crown {
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
        let on = poses.0[b.0].stars;
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

fn apply_sash(
    game: Res<Game>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut q: Query<(&Blob, &mut Visibility, &MeshMaterial3d<StandardMaterial>), With<Sash>>,
) {
    for (b, mut v, m) in &mut q {
        match game.dummies[b.0].team {
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
        let p = &poses.0[b.0];
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
        *v = if game.dummies[b.0].fallen {
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
        let d = &game.dummies[b.0];
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
    mut body: Query<&mut Transform, (With<DazzaBody>, Without<DazzaArm>)>,
    mut arm: Query<&mut Transform, (With<DazzaArm>, Without<DazzaBody>)>,
    mut bubble: Query<&mut Visibility, With<DazzaBubble>>,
) {
    let dt = time.delta_secs();
    let state = DazzaState::ALL[dz.state_idx];
    let pos = Vec3::new(dazza::HOME.x, 0.0, dazza::HOME.z);
    let moved = pos.distance(dz.last);
    dz.last = pos;
    let pose = dz.anim.tick(dt, state, game.now, 0.0, moved);
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
