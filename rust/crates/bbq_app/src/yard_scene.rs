//! Draws the yard from the same data the rules use (`bbq_core::yard`): lawn, fence, pool,
//! trampoline, every box, the sun, sky colour and fog. Simple shapes for now; real models
//! come in Phase 4.

use bbq_core::yard::{self, Collider, Feature, Features, Kind, Yard};
use bbq_core::{YARD_HALF_X, YARD_HALF_Z};
use bevy::prelude::*;

/// The rules' view of the yard. Rebuilt whenever a feature is switched.
#[derive(Resource)]
pub struct YardRes(pub Yard);

/// Marks anything that belongs to a switchable feature.
#[derive(Component)]
pub struct FeatureTag(pub Feature);

/// Marks the boxes drawn from colliders, so they can be rebuilt with the chest in a new spot.
#[derive(Component)]
struct BoxProp;

pub struct YardScenePlugin;

impl Plugin for YardScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(YardRes(Yard::default()))
            .add_systems(Startup, build_yard)
            .add_systems(Update, toggle_features);
    }
}

fn box_look(kind: Kind) -> Color {
    match kind {
        Kind::Plain => Color::srgb(0.6, 0.6, 0.6),
        Kind::Pole => Color::srgb(0.7, 0.7, 0.72),
        Kind::Shed => Color::srgb(0.55, 0.42, 0.3),
        Kind::Esky => Color::srgb(0.12, 0.44, 0.82),
        Kind::Table => Color::srgb(0.65, 0.5, 0.33),
        Kind::Grill => Color::srgb(0.2, 0.2, 0.22),
        Kind::MeatTable => Color::srgb(0.8, 0.8, 0.8),
        Kind::Bins => Color::srgb(0.1, 0.45, 0.2),
        Kind::Crates => Color::srgb(0.7, 0.55, 0.25),
        Kind::Hedge => Color::srgb(0.12, 0.4, 0.14),
        Kind::Tyres => Color::srgb(0.1, 0.1, 0.1),
        Kind::Woodpile => Color::srgb(0.5, 0.33, 0.18),
        Kind::Wall => Color::srgb(0.7, 0.35, 0.25),
        Kind::Planter => Color::srgb(0.4, 0.28, 0.2),
        Kind::Bar => Color::srgb(0.45, 0.28, 0.12),
        Kind::Chest => Color::srgb(0.9, 0.7, 0.1),
    }
}

fn spawn_box(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    mats: &mut Assets<StandardMaterial>,
    c: &Collider,
) {
    let (w, d) = (c.x1 - c.x0, c.z1 - c.z0);
    let mut e = commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(w, c.h, d))),
        MeshMaterial3d(mats.add(box_look(c.kind))),
        Transform::from_xyz((c.x0 + c.x1) / 2.0, c.h / 2.0, (c.z0 + c.z1) / 2.0),
        BoxProp,
    ));
    if let Some(f) = c.kind.feature() {
        e.insert(FeatureTag(f));
    }
}

fn build_yard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    yard_res: Res<YardRes>,
) {
    // Lawn.
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(YARD_HALF_X * 2.0, YARD_HALF_Z * 2.0),
            ),
        ),
        MeshMaterial3d(mats.add(Color::srgb(0.37, 0.61, 0.26))),
    ));

    // Fence: four low planks around the edge (the rules treat the edge as a wall).
    let fence = mats.add(Color::srgb(0.85, 0.82, 0.74));
    for (x, z, w, d) in [
        (0.0, -YARD_HALF_Z - 0.1, YARD_HALF_X * 2.0, 0.2),
        (0.0, YARD_HALF_Z + 0.1, YARD_HALF_X * 2.0, 0.2),
        (-YARD_HALF_X - 0.1, 0.0, 0.2, YARD_HALF_Z * 2.0),
        (YARD_HALF_X + 0.1, 0.0, 0.2, YARD_HALF_Z * 2.0),
    ] {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(w, 2.0, d))),
            MeshMaterial3d(fence.clone()),
            Transform::from_xyz(x, 1.0, z),
        ));
    }

    // Pool: a blue slab just above the lawn, sitting at the water level's footprint.
    let (pw, pd) = (yard::POOL_X1 - yard::POOL_X0, yard::POOL_Z1 - yard::POOL_Z0);
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(pw, 0.04, pd))),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::srgba(0.25, 0.7, 0.95, 0.9),
            alpha_mode: AlphaMode::Blend,
            ..default()
        })),
        Transform::from_xyz(
            (yard::POOL_X0 + yard::POOL_X1) / 2.0,
            0.03,
            (yard::POOL_Z0 + yard::POOL_Z1) / 2.0,
        ),
    ));

    // Trampoline: a blue pad on a ring.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(yard::TRAMP_R, 0.1))),
        MeshMaterial3d(mats.add(Color::srgb(0.1, 0.1, 0.12))),
        Transform::from_xyz(yard::TRAMP_X, yard::TRAMP_H - 0.05, yard::TRAMP_Z),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Torus::new(yard::TRAMP_R - 0.05, yard::TRAMP_R + 0.1))),
        MeshMaterial3d(mats.add(Color::srgb(0.12, 0.43, 0.82))),
        Transform::from_xyz(yard::TRAMP_X, yard::TRAMP_H, yard::TRAMP_Z),
    ));

    // Smoko pad.
    commands.spawn((
        Mesh3d(meshes.add(Cylinder::new(2.2, 0.03))),
        MeshMaterial3d(mats.add(Color::srgb(0.6, 0.55, 0.45))),
        Transform::from_xyz(yard::SMOKO_X, 0.02, yard::SMOKO_Z),
        FeatureTag(Feature::Smoko),
    ));

    // Smoko chairs: one per person (the game has you plus three dummies for now).
    let chairs = bbq_core::smoko::chair_count(4);
    let frame = mats.add(Color::srgb(0.16, 0.16, 0.16));
    let fabric = mats.add(Color::srgb(0.12, 0.44, 0.82));
    let seat_mesh = meshes.add(Cuboid::new(0.52, 0.06, 0.46));
    let back_mesh = meshes.add(Cuboid::new(0.52, 0.5, 0.05));
    let leg_mesh = meshes.add(Cuboid::new(0.04, 0.44, 0.04));
    for i in 0..chairs {
        let (x, z) = bbq_core::smoko::seat_pos(i, chairs);
        // the chair faces the middle of the pad
        let facing = bbq_core::smoko::seat_facing(i, chairs);
        commands
            .spawn((
                Transform::from_xyz(x, 0.0, z).with_rotation(Quat::from_rotation_y(facing)),
                Visibility::default(),
                FeatureTag(Feature::Smoko),
            ))
            .with_children(|c| {
                c.spawn((
                    Mesh3d(seat_mesh.clone()),
                    MeshMaterial3d(fabric.clone()),
                    Transform::from_xyz(0.0, 0.44, 0.0),
                ));
                c.spawn((
                    Mesh3d(back_mesh.clone()),
                    MeshMaterial3d(fabric.clone()),
                    Transform::from_xyz(0.0, 0.72, -0.24),
                ));
                for (lx, lz) in [(-0.24, -0.2), (0.24, -0.2), (-0.24, 0.2), (0.24, 0.2)] {
                    c.spawn((
                        Mesh3d(leg_mesh.clone()),
                        MeshMaterial3d(frame.clone()),
                        Transform::from_xyz(lx, 0.22, lz),
                    ));
                }
            });
    }

    // Every box the rules know about.
    for c in &yard_res.0.colliders {
        spawn_box(&mut commands, &mut meshes, &mut mats, c);
    }

    // The sun, with shadows.
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            illuminance: 12_000.0,
            ..default()
        },
        Transform::from_xyz(24.0, 40.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// F1 bar, F2 BBQ, F3 chest, F4 smoko.
fn toggle_features(
    keys: Res<ButtonInput<KeyCode>>,
    mut yard_res: ResMut<YardRes>,
    mut tags: Query<(&FeatureTag, &mut Visibility)>,
) {
    let mut changed = false;
    for (key, f) in [
        (KeyCode::F1, Feature::Bar),
        (KeyCode::F2, Feature::Bbq),
        (KeyCode::F3, Feature::Chest),
        (KeyCode::F4, Feature::Smoko),
    ] {
        if keys.just_pressed(key) {
            let mut feats: Features = yard_res.0.features;
            feats.toggle(f);
            yard_res.0 = Yard::new(feats, yard_res.0.chest_spot);
            changed = true;
        }
    }
    if changed {
        let feats = yard_res.0.features;
        for (tag, mut vis) in &mut tags {
            *vis = if feats.get(tag.0) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
