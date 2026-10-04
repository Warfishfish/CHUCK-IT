//! Draws the yard from the shape lists in `bbq_core::looks_yard`, which are copied from the
//! browser game. The rules' own view of the yard (`YardRes`, from `bbq_core::yard`) is separate:
//! this file only draws.

use bbq_core::looks::{Part, Tex};
use bbq_core::looks_yard::{self, CHEST_PIVOT, HOIST_AT};
use bbq_core::yard::{Feature, Features, Yard};
use bevy::prelude::*;

use crate::game::Game;
use crate::models::ModelCache;

/// The rules' view of the yard. Rebuilt whenever a feature is switched.
#[derive(Resource)]
pub struct YardRes(pub Yard);

/// Marks anything that belongs to a switchable feature.
#[derive(Component)]
pub struct FeatureTag(pub Feature);

/// Where the trees and clouds go. Any number gives a believable yard.
const LOOK_SEED: u64 = 0xBB0;

/// The clothesline's turning head (it turns 0.12 radians a second).
#[derive(Component)]
struct HoistHead;
/// A cloud drifting along +x at 1.2 m/s, wrapping round at 110.
#[derive(Component)]
struct Cloud;
/// The water, whose picture slides along, and the shimmer on the pool floor.
#[derive(Component)]
struct PoolWater;
#[derive(Component)]
struct PoolShimmer;
/// The chest as a whole, its lid, and one of its toys.
#[derive(Component)]
struct ChestRoot;
#[derive(Component)]
struct ChestLid;
#[derive(Component)]
struct ChestToy(usize);

/// How far the clothesline, water and shimmer have moved.
#[derive(Resource, Default)]
struct Drift {
    water: Vec2,
    shimmer_y: f32,
}

pub struct YardScenePlugin;

impl Plugin for YardScenePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(YardRes(Yard::default()))
            .init_resource::<Drift>()
            .add_systems(Startup, build_yard)
            .add_systems(Update, (toggle_features, animate_yard, sync_chest));
    }
}

fn find(parts: &[Part], tex: Tex) -> Option<usize> {
    parts.iter().position(|p| p.surface.tex == Some(tex))
}

fn build_yard(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<ModelCache>,
) {
    let look = looks_yard::yard(LOOK_SEED);

    // everything that is always there (lawn, fence, house, trees, pool, props)
    let (_, kids) = cache.spawn_parts_with_children(
        &mut commands,
        &look.world,
        &mut meshes,
        &mut mats,
        Transform::default(),
    );
    if let Some(i) = find(&look.world, Tex::PoolWater) {
        commands.entity(kids[i]).insert(PoolWater);
    }
    if let Some(i) = find(&look.world, Tex::Caustics) {
        commands.entity(kids[i]).insert(PoolShimmer);
    }

    // the switchable parts
    for (parts, feature) in [
        (&look.bar, Feature::Bar),
        (&look.bbq, Feature::Bbq),
        (&look.smoko, Feature::Smoko),
    ] {
        let e = cache.spawn_parts(
            &mut commands,
            parts,
            &mut meshes,
            &mut mats,
            Transform::default(),
        );
        commands.entity(e).insert(FeatureTag(feature));
    }

    // the smoko pad and its chairs (one per person; the game has you plus three dummies for now)
    let chairs = bbq_core::smoko::chair_count(4);
    let r = bbq_core::smoko::zone_radius(4);
    let pad = looks_yard::smoko_pad(r);
    let e = cache.spawn_parts(
        &mut commands,
        &[pad],
        &mut meshes,
        &mut mats,
        Transform::default(),
    );
    commands.entity(e).insert(FeatureTag(Feature::Smoko));
    for i in 0..chairs {
        let (x, z) = bbq_core::smoko::seat_pos(i, chairs);
        let facing = bbq_core::smoko::seat_facing(i, chairs);
        let e = cache.spawn_parts(
            &mut commands,
            &looks_yard::chair(),
            &mut meshes,
            &mut mats,
            Transform::from_xyz(x, 0.0, z).with_rotation(Quat::from_rotation_y(facing)),
        );
        commands.entity(e).insert(FeatureTag(Feature::Smoko));
    }

    // the clothesline head
    let e = cache.spawn_parts(
        &mut commands,
        &look.hoist_head,
        &mut meshes,
        &mut mats,
        Transform::from_xyz(HOIST_AT.x, HOIST_AT.y, HOIST_AT.z),
    );
    commands.entity(e).insert(HoistHead);

    // clouds
    for (at, puffs) in &look.clouds {
        let e = cache.spawn_parts(
            &mut commands,
            puffs,
            &mut meshes,
            &mut mats,
            Transform::from_xyz(at.x, at.y, at.z),
        );
        commands.entity(e).insert(Cloud);
    }

    // the chest: a box, a lid on a hinge, and up to three toys inside
    let root = commands
        .spawn((Transform::default(), Visibility::Hidden, ChestRoot))
        .id();
    let base = cache.spawn_parts(
        &mut commands,
        &look.chest_base,
        &mut meshes,
        &mut mats,
        Transform::default(),
    );
    let lid = cache.spawn_parts(
        &mut commands,
        &look.chest_lid,
        &mut meshes,
        &mut mats,
        Transform::from_xyz(CHEST_PIVOT.x, CHEST_PIVOT.y, CHEST_PIVOT.z),
    );
    commands.entity(lid).insert(ChestLid);
    commands.entity(root).add_children(&[base, lid]);
    for k in 0..bbq_core::chest::MAX_STOCK as usize {
        let toy = cache.spawn_parts(
            &mut commands,
            &looks_yard::chest_toy(k),
            &mut meshes,
            &mut mats,
            Transform::default(),
        );
        commands.entity(toy).insert(ChestToy(k));
        commands.entity(root).add_child(toy);
    }

    // the sun, with shadows, and the soft light from the sky
    commands.spawn(crate::lighting::sun());
    let (_, sky, sky_at) = crate::lighting::hemisphere();
    commands.spawn((sky, sky_at));
}

/// The clothesline turns, the pool water slides and the clouds drift.
fn animate_yard(
    time: Res<Time>,
    mut drift: ResMut<Drift>,
    mut head: Query<&mut Transform, (With<HoistHead>, Without<Cloud>)>,
    mut clouds: Query<&mut Transform, (With<Cloud>, Without<HoistHead>)>,
    water: Query<&MeshMaterial3d<StandardMaterial>, (With<PoolWater>, Without<PoolShimmer>)>,
    shimmer: Query<&MeshMaterial3d<StandardMaterial>, (With<PoolShimmer>, Without<PoolWater>)>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();
    for mut tf in &mut head {
        tf.rotate_y(dt * 0.12);
    }
    for mut tf in &mut clouds {
        tf.translation.x += dt * 1.2;
        if tf.translation.x > 110.0 {
            tf.translation.x = -110.0;
        }
    }
    drift.water += Vec2::new(dt * 0.02, dt * 0.013);
    drift.shimmer_y += dt * 0.018;
    for h in &water {
        if let Some(mut m) = mats.get_mut(&h.0) {
            m.uv_transform = bevy::math::Affine2::from_scale_angle_translation(
                Vec2::new(2.0, 1.3),
                0.0,
                drift.water,
            );
        }
    }
    let (pw, pd) = (
        bbq_core::yard::POOL_X1 - bbq_core::yard::POOL_X0,
        bbq_core::yard::POOL_Z1 - bbq_core::yard::POOL_Z0,
    );
    for h in &shimmer {
        if let Some(mut m) = mats.get_mut(&h.0) {
            m.uv_transform = bevy::math::Affine2::from_scale_angle_translation(
                Vec2::new(pw / 2.5, pd / 2.5),
                0.0,
                Vec2::new((t * 0.35).sin() * 0.12, drift.shimmer_y),
            );
        }
    }
}

/// Put the chest at its spot, open the lid when there is something in it, show the toys.
fn sync_chest(
    game: Res<Game>,
    yard: Res<YardRes>,
    time: Res<Time>,
    mut root: Query<(&mut Transform, &mut Visibility), (With<ChestRoot>, Without<ChestLid>, Without<ChestToy>)>,
    mut lid: Query<&mut Transform, (With<ChestLid>, Without<ChestRoot>, Without<ChestToy>)>,
    mut toys: Query<(&ChestToy, &mut Visibility), (Without<ChestRoot>, Without<ChestLid>)>,
) {
    let stock = game.life.chest.stock as usize;
    let on = yard.0.features.chest && game.options.adult;
    let (at, turn) = looks_yard::chest_place(yard.0.chest_spot);
    for (mut tf, mut vis) in &mut root {
        tf.translation = Vec3::new(at.x, 0.0, at.z);
        tf.rotation = Quat::from_rotation_y(turn);
        *vis = if on { Visibility::Inherited } else { Visibility::Hidden };
    }
    // the lid eases open (-1.15 radians) when there is stock, shut (-0.08) when empty
    let want = if stock > 0 { -1.15 } else { -0.08 };
    let k = 1.0 - (-6.0 * time.delta_secs()).exp();
    for mut tf in &mut lid {
        let (x, _, _) = tf.rotation.to_euler(EulerRot::XYZ);
        tf.rotation = Quat::from_rotation_x(x + (want - x) * k);
    }
    for (toy, mut vis) in &mut toys {
        *vis = if toy.0 < stock {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
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
