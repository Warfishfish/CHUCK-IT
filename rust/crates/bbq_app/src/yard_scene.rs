//! Draws the yard from the shape lists in `bbq_core::looks_yard`, which are copied from the
//! browser game. The rules' own view of the yard (`YardRes`, from `bbq_core::yard`) is separate:
//! this file only draws.

use bbq_core::looks::{Part, Tex};
use bbq_core::looks_yard::{self, HOIST_AT};
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
/// A magpie's turning head: it watches you.
#[derive(Component)]
struct MagpieHead {
    /// Which way the body faces (turn about y).
    body_yaw: f32,
    seed: f32,
    yaw: f32,
    pitch: f32,
}
#[derive(Component)]
struct MagpieBody(f32);
/// Part of the smoko area: it is moved when the round moves the pad (Teddy Heist).
#[derive(Component)]
struct SmokoShift(Vec3);
/// One of the yard's other eskies and its lid.
#[derive(Component)]
struct DecorLid(usize);
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
            .add_systems(Update, (toggle_features, animate_yard, sync_chest, sync_decor_eskies, sync_smoko_place, animate_magpies));
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
    mode: Res<crate::lighting::LookMode>,
) {
    let look = looks_yard::yard_styled(LOOK_SEED, *mode == crate::lighting::LookMode::Polished);

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
        if feature == Feature::Smoko {
            commands.entity(e).insert(SmokoShift(Vec3::ZERO));
        }
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
    commands.entity(e).insert((FeatureTag(Feature::Smoko), SmokoShift(Vec3::ZERO)));
    for i in 0..chairs {
        let (x, z) = bbq_core::smoko::seat_pos(i, chairs);
        let facing = bbq_core::smoko::seat_facing(i, chairs);
        let e = cache.spawn_parts(
            &mut commands,
            &if *mode == crate::lighting::LookMode::Polished {
                looks_yard::chair_styled(i)
            } else {
                looks_yard::chair()
            },
            &mut meshes,
            &mut mats,
            Transform::from_xyz(x, 0.0, z).with_rotation(Quat::from_rotation_y(facing)),
        );
        commands.entity(e).insert((FeatureTag(Feature::Smoko), SmokoShift(Vec3::new(x, 0.0, z))));
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
    if *mode == crate::lighting::LookMode::Polished {
        // the Hills Hoist leans: its head sits on the tilted pole's top
        let top = looks_yard::hoist_lean().1;
        commands.entity(e).insert(Transform::from_xyz(top.x, top.y, top.z));
    }

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
        Transform::from_xyz(look.chest_pivot.x, look.chest_pivot.y, look.chest_pivot.z),
    );
    commands.entity(lid).insert(ChestLid);
    commands.entity(root).add_children(&[base, lid]);
    for k in 0..bbq_core::chest::MAX_STOCK as usize {
        let (at, rot, scale) = looks_yard::chest_toy_place(k);
        let toy = cache.spawn(
            &mut commands,
            crate::models::ModelKey::item(
                bbq_core::items::ItemKind::Dildo,
                Some(looks_yard::chest_toy_variant(k)),
                0,
            ),
            &mut meshes,
            &mut mats,
            Transform {
                translation: Vec3::new(at.x, at.y, at.z),
                rotation: Quat::from_xyzw(rot.x, rot.y, rot.z, rot.w),
                scale: Vec3::splat(scale),
            },
        );
        commands.entity(toy).insert((
            ChestToy(k),
            crate::items_view::Wobble::with_seed(k as f32),
            crate::items_view::FlopDrive::in_chest(),
        ));
        commands.entity(root).add_child(toy);
    }

    // magpies on the fence, watching you
    for (i, (at, yaw)) in look.magpies.iter().enumerate() {
        let m = looks_yard::magpie();
        let root = commands
            .spawn((
                Transform::from_xyz(at.x, at.y, at.z)
                    .with_rotation(Quat::from_rotation_y(*yaw))
                    .with_scale(Vec3::splat(1.35)),
                Visibility::default(),
                MagpieBody(i as f32),
            ))
            .id();
        let body = cache.spawn_parts(&mut commands, &m.body, &mut meshes, &mut mats, Transform::default());
        let head = cache.spawn_parts(
            &mut commands,
            &m.head,
            &mut meshes,
            &mut mats,
            Transform::from_xyz(m.neck.x, m.neck.y, m.neck.z),
        );
        commands.entity(head).insert(MagpieHead { body_yaw: *yaw, seed: i as f32 * 3.7, yaw: 0.0, pitch: 0.0 });
        commands.entity(root).add_children(&[body, head]);
    }

    // the yard's other eskies: they open, but hold nothing yet
    for (i, e) in look.eskies.iter().enumerate() {
        let root = commands
            .spawn((
                Transform::from_xyz(e.at.x, e.at.y, e.at.z).with_rotation(Quat::from_rotation_y(e.turn)),
                Visibility::default(),
            ))
            .id();
        let base = cache.spawn_parts(&mut commands, &e.base, &mut meshes, &mut mats, Transform::default());
        let lid = cache.spawn_parts(
            &mut commands,
            &e.lid,
            &mut meshes,
            &mut mats,
            Transform::from_xyz(look.chest_pivot.x, look.chest_pivot.y, look.chest_pivot.z)
                .with_rotation(Quat::from_rotation_x(-0.08)),
        );
        commands.entity(lid).insert(DecorLid(i));
        commands.entity(root).add_children(&[base, lid]);
    }

    // the sun, with shadows, and the soft light from the sky
    commands.spawn(crate::lighting::sun(*mode));
    if *mode == crate::lighting::LookMode::Polished {
        // a cool, weak light from the far side: it lifts the shaded sides so blobs and props stand
        // out against the lawn
        commands.spawn(crate::lighting::rim_light());
    }
    let (_, sky, sky_at) = crate::lighting::hemisphere(*mode);
    commands.spawn((sky, sky_at));
}

/// The clothesline turns, the pool water slides and the clouds drift.
fn animate_yard(
    mode: Res<crate::lighting::LookMode>,
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
        if *mode == crate::lighting::LookMode::Polished {
            // spin on the leaning pole
            let q = looks_yard::hoist_lean().0;
            let lean = Quat::from_xyzw(q.x, q.y, q.z, q.w);
            let (_, spin, _) = tf.rotation.to_euler(EulerRot::YXZ);
            let _ = spin;
            tf.rotation = lean * Quat::from_rotation_y(t * 0.12);
        } else {
            tf.rotate_y(dt * 0.12);
        }
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
    mut root: Query<
        (&mut Transform, &mut Visibility),
        (With<ChestRoot>, Without<ChestLid>, Without<ChestToy>),
    >,
    mut lid: Query<&mut Transform, (With<ChestLid>, Without<ChestRoot>, Without<ChestToy>)>,
    mut toys: Query<(&ChestToy, &mut Visibility), (Without<ChestRoot>, Without<ChestLid>)>,
) {
    let stock = game.life.chest.stock as usize;
    let on = yard.0.features.chest && game.options.adult;
    let (at, turn) = looks_yard::chest_place(yard.0.chest_spot);
    for (mut tf, mut vis) in &mut root {
        tf.translation = Vec3::new(at.x, 0.0, at.z);
        tf.rotation = Quat::from_rotation_y(turn);
        *vis = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
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

/// The yard's other eskies: the lid eases open for a few seconds after R, then shuts.
fn sync_decor_eskies(
    game: Res<Game>,
    time: Res<Time>,
    mut lids: Query<(&DecorLid, &mut Transform)>,
) {
    let k = 1.0 - (-6.0 * time.delta_secs()).exp();
    for (l, mut tf) in &mut lids {
        let want = if game.life.decor_open.get(l.0).copied().unwrap_or(0.0) > 0.0 { -1.15 } else { -0.08 };
        let (x, _, _) = tf.rotation.to_euler(EulerRot::XYZ);
        tf.rotation = Quat::from_rotation_x(x + (want - x) * k);
    }
}

/// Magpies turn their heads to follow you, cock them now and then, and bob a little.
fn animate_magpies(
    time: Res<Time>,
    player: Res<crate::player::Player>,
    mut heads: Query<(&mut MagpieHead, &mut Transform, &GlobalTransform), Without<MagpieBody>>,
    mut bodies: Query<(&MagpieBody, &mut Transform), Without<MagpieHead>>,
) {
    let t = time.elapsed_secs();
    let k = 1.0 - (-7.0 * time.delta_secs()).exp();
    for (mut h, mut tf, gt) in &mut heads {
        let at = gt.translation();
        let (dx, dz) = (player.mover.x - at.x, player.mover.z - at.z);
        let dist = dx.hypot(dz).max(0.1);
        // turn towards you, but only as far as a neck goes
        let mut want = dx.atan2(dz) - h.body_yaw;
        want = (want + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI;
        let want = want.clamp(-1.9, 1.9);
        let pitch = (-(1.5 - at.y).atan2(dist) * 0.6).clamp(-0.5, 0.5);
        h.yaw += (want - h.yaw) * k;
        h.pitch += (pitch - h.pitch) * k;
        // now and then it cocks its head the way magpies do
        let cock = ((t * 0.5 + h.seed).sin() - 0.7).max(0.0) * 1.6;
        let nod = (t * 3.1 + h.seed).sin().max(0.97) - 0.97;
        tf.rotation = Quat::from_euler(EulerRot::YXZ, h.yaw, h.pitch + nod * 6.0, cock * 0.5);
    }
    for (b, mut tf) in &mut bodies {
        tf.translation.y = bbq_core::looks_yard::FENCE_H + ((t * 1.7 + b.0 * 2.0).sin().max(0.92) - 0.92) * 0.25;
    }
}

/// Slide the smoko pad, chairs and umbrella to wherever the round put the pad.
fn sync_smoko_place(game: Res<Game>, mut q: Query<(&SmokoShift, &mut Transform)>) {
    let (dx, dz) = (
        game.life.smoko_at.0 - bbq_core::yard::SMOKO_X,
        game.life.smoko_at.1 - bbq_core::yard::SMOKO_Z,
    );
    for (s, mut tf) in &mut q {
        let want = Vec3::new(s.0.x + dx, tf.translation.y, s.0.z + dz);
        if tf.translation != want {
            tf.translation = want;
        }
    }
}

/// F1 bar, F2 BBQ, F3 chest, F4 smoko.
fn toggle_features(
    keys: Res<ButtonInput<KeyCode>>,
    mut yard_res: ResMut<YardRes>,
    mut tags: Query<(&FeatureTag, &mut Visibility)>,
    mut last: Local<Option<Features>>,
) {
    // the menu changes the features too, not only the F keys
    let mut changed = *last != Some(yard_res.0.features);
    for (key, f) in [
        (KeyCode::F1, Feature::Bar),
        (KeyCode::F2, Feature::Bbq),
        (KeyCode::F3, Feature::Chest),
        (KeyCode::F4, Feature::Smoko),
    ] {
        if keys.just_pressed(key) {
            let mut feats: Features = yard_res.0.features;
            feats.toggle(f);
            let spot = yard_res.0.chest_spot;
            yard_res.0 = match yard_res.0.heist {
                Some(teams) => Yard::heist(feats, spot, teams),
                None => Yard::new(feats, spot),
            };
            changed = true;
        }
    }
    if changed {
        let feats = yard_res.0.features;
        *last = Some(feats);
        for (tag, mut vis) in &mut tags {
            *vis = if feats.get(tag.0) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}
