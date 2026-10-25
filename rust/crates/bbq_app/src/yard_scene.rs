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

/// Shown only in Cheeky mode (`true`: the bar's grog, empty stubbies on the lawn) or only in
/// family mode (`false`: the soft-drinks stand). Marcus, 11 Oct 2026: no alcohol in family mode.
#[derive(Component)]
pub struct CheekyOnly(pub bool);

/// Where the trees and clouds go. Any number gives a believable yard.
#[derive(Component)]
struct NormalYard;
#[derive(Component)]
struct BackyardCover;
#[derive(Component)]
struct RaceMap;

const LOOK_SEED: u64 = 0xBB0;

/// Fixed clothesline frame; shared geometry keeps multiplayer clearance exact.
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
#[derive(Component)]
struct MagpieWing { bird:usize, side:f32, base:Transform }
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
            .add_systems(Update, (crate::grass::sync, crate::grass::distance_cull).chain())
            .add_systems(PostUpdate, crate::lighting::follow_sky.before(bevy::transform::TransformSystems::Propagate))
            .add_systems(Update, (toggle_features, animate_yard, sync_chest, sync_decor_eskies, sync_smoko_map, sync_race_map, sync_race_return_marker, sync_backyard_cover, animate_magpies));
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
    let yard_root=commands.spawn((Transform::default(),Visibility::default(),NormalYard)).id();
    let look = looks_yard::yard_styled(LOOK_SEED, *mode == crate::lighting::LookMode::Polished);
    if *mode == crate::lighting::LookMode::Polished && crate::style::current() == crate::style::Style::Pop {
        crate::lighting::spawn_afternoon_sky(&mut commands, &mut meshes, &mut mats);
    }

    let cover=cache.spawn_parts(&mut commands,&looks_yard::backyard_cover(),&mut meshes,&mut mats,Transform::default());
    commands.entity(cover).insert(BackyardCover);commands.entity(yard_root).add_child(cover);

    if crate::backyard_finish::enabled() {
        let trim=cache.spawn_parts(&mut commands,&crate::backyard_finish::house_trim(),&mut meshes,&mut mats,Transform::default());
        commands.entity(yard_root).add_child(trim);
    }
    // everything that is always there (lawn, fence, house, trees, pool, props)
    let (world_root, kids) = cache.spawn_parts_with_children(
        &mut commands,
        &look.world,
        &mut meshes,
        &mut mats,
        Transform::default(),
    );
    commands.entity(yard_root).add_child(world_root);
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
    ] {
        let e = cache.spawn_parts(
            &mut commands,
            parts,
            &mut meshes,
            &mut mats,
            Transform::default(),
        );
        commands.entity(e).insert(FeatureTag(feature));
        commands.entity(yard_root).add_child(e);
    }
    // what is on the bar depends on the mode: grog in Cheeky mode, soft drinks in family mode
    for (parts, cheeky) in [(&look.bar_grog, true), (&look.bar_soft, false)] {
        let e = cache.spawn_parts(&mut commands, parts, &mut meshes, &mut mats, Transform::default());
        commands.entity(e).insert((FeatureTag(Feature::Bar), CheekyOnly(cheeky), Visibility::Hidden));
        commands.entity(yard_root).add_child(e);
    }
    let e = cache.spawn_parts(&mut commands, &look.litter, &mut meshes, &mut mats, Transform::default());
    commands.entity(e).insert((CheekyOnly(true), Visibility::Hidden));
    commands.entity(yard_root).add_child(e);

    // The former Smoko area is open lawn. Shared prop builders remain in looks_yard.

    // the clothesline head
    let e = cache.spawn_parts(
        &mut commands,
        &look.hoist_head,
        &mut meshes,
        &mut mats,
        Transform::from_xyz(HOIST_AT.x, HOIST_AT.y, HOIST_AT.z),
    );
    commands.entity(e).insert(HoistHead);
    commands.entity(yard_root).add_child(e);
    if *mode == crate::lighting::LookMode::Polished {
        // Frame transform shares the authoritative clearance geometry.
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
    commands.entity(yard_root).add_child(root);
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
    for i in 0..look.magpies.len().max(bbq_core::magpie::MAX_FLOCK+1) {
        let (at,yaw)=look.magpies.get(i).copied().unwrap_or((bbq_core::magpie::PERCH,0.0));
        let m = looks_yard::magpie();
        let root = commands
            .spawn((
                Transform::from_xyz(at.x, at.y, at.z)
                    .with_rotation(Quat::from_rotation_y(yaw))
                    .with_scale(Vec3::splat(1.35)),
                Visibility::default(),
                MagpieBody(i as f32),
            ))
            .id();
        let (body,parts) = cache.spawn_parts_with_children(&mut commands, &m.body, &mut meshes, &mut mats, Transform::default());
        for (index,side) in [(4,-1.0),(5,-1.0),(8,1.0),(9,1.0)] {commands.entity(parts[index]).insert(MagpieWing{bird:i,side,base:crate::models::transform_of(&m.body[index])});}
        let head = cache.spawn_parts(
            &mut commands,
            &m.head,
            &mut meshes,
            &mut mats,
            Transform::from_xyz(m.neck.x, m.neck.y, m.neck.z),
        );
        commands.entity(head).insert(MagpieHead { body_yaw: yaw, seed: i as f32 * 3.7, yaw: 0.0, pitch: 0.0 });
        commands.entity(yard_root).add_child(root);
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
        commands.entity(yard_root).add_child(root);
    commands.entity(root).add_children(&[base, lid]);
    }

    // the sun, with shadows, and the soft light from the sky
    let sun = commands.spawn(crate::lighting::sun(*mode)).id();
    if *mode == crate::lighting::LookMode::Polished {
        commands.entity(sun).insert(crate::lighting::sun_cascades());
    }
    if *mode == crate::lighting::LookMode::Polished {
        // a cool, weak light from the far side: it lifts the shaded sides so blobs and props stand
        // out against the lawn
        commands.spawn(crate::lighting::rim_light());
    }
    let (_, sky, sky_at) = crate::lighting::hemisphere(*mode);
    commands.spawn((sky, sky_at));
}

/// The pool water slides and clouds drift; the clothesline stays aligned with collision.
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
            // Fixed frame: no visual-only rotation past its colliders.
            let q = looks_yard::hoist_lean().0;
            let lean = Quat::from_xyzw(q.x, q.y, q.z, q.w);
            let (_, spin, _) = tf.rotation.to_euler(EulerRot::YXZ);
            let _ = spin;
            tf.rotation = lean;
        } else {
            tf.rotation = Quat::IDENTITY;
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
    // the lid eases open when there is stock (right back, past upright, so the tall toys have
    // room), shut (-0.08) when empty
    let want = if stock > 0 { -1.8 } else { -0.08 };
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
    game:Res<Game>,
    player: Res<crate::player::Player>,
    mut heads: Query<(&mut MagpieHead, &mut Transform, &GlobalTransform), (Without<MagpieBody>,Without<MagpieWing>)>,
    mut bodies: Query<(&MagpieBody, &mut Transform, &mut Visibility), (Without<MagpieHead>,Without<MagpieWing>)>,
    mut wings:Query<(&MagpieWing,&mut Transform),(Without<MagpieBody>,Without<MagpieHead>)>,
) {
    let t = time.elapsed_secs();
    let k = 1.0 - (-7.0 * time.delta_secs()).exp();
    for (mut h, mut tf, gt) in &mut heads {
        if h.seed==0.0 && game.magpie.phase>0 && matches!(game.rules.mode,bbq_core::GameMode::FreeForAll|bbq_core::GameMode::Teams) {
            tf.rotation=Quat::IDENTITY;continue;
        }
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
    for (w,mut tf) in &mut wings {
        *tf=w.base;
        if ((w.bird==0 && game.magpie.phase>0) || (w.bird>0 && game.flock.get(w.bird-1).is_some_and(|b|b.phase>0))) && matches!(game.rules.mode,bbq_core::GameMode::FreeForAll|bbq_core::GameMode::Teams) {
            tf.rotation=Quat::from_rotation_z(w.side*(0.7+(t*24.0).sin()*0.65))*tf.rotation;tf.scale.y*=2.0;
        }
    }
    for (b, mut tf,mut visibility) in &mut bodies {
        let index=b.0 as usize;
        if index>0 {
            if let Some(bird)=game.flock.get(index-1) {
                *visibility=Visibility::Inherited;let next=Vec3::new(bird.pos.x,bird.pos.y,bird.pos.z);let delta=next-tf.translation;
                tf.translation=tf.translation.lerp(next,1.0-(-24.0*time.delta_secs()).exp());
                if delta.length_squared()>0.0001 {tf.rotation=Quat::from_rotation_y(delta.x.atan2(delta.z));}
                continue;
            } else if index>=2 {*visibility=Visibility::Hidden;continue;}
        }
        *visibility=Visibility::Inherited;
        if b.0==0.0 && matches!(game.rules.mode,bbq_core::GameMode::FreeForAll|bbq_core::GameMode::Teams) {
            let bird=&game.magpie;let next=Vec3::new(bird.pos.x,bird.pos.y,bird.pos.z);
            let d=next-tf.translation;tf.translation=tf.translation.lerp(next,1.0-(-24.0*time.delta_secs()).exp());
            // Use replicated motion for heading, so guests never need a private target aim.
            if bird.phase>0 {if d.x*d.x+d.z*d.z>0.000001 {tf.rotation=Quat::from_rotation_y(d.x.atan2(d.z))*Quat::from_rotation_x(if bird.phase==2{0.3}else{-0.15});}}
            else {tf.rotation=Quat::from_rotation_y(std::f32::consts::PI);}
            continue;
        }
        if b.0==0.0 {tf.translation.x=14.0;tf.translation.z=bbq_core::YARD_HALF_Z+0.05;tf.rotation=Quat::from_rotation_y(std::f32::consts::PI);}
        tf.translation.y = bbq_core::looks_yard::FENCE_H + ((t * 1.7 + b.0 * 2.0).sin().max(0.92) - 0.92) * 0.25;
    }
}

/// F1 bar, F2 BBQ, F3 chest.
fn toggle_features(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<crate::game::Game>,
    mut yard_res: ResMut<YardRes>,
    mut tags: Query<(Option<&FeatureTag>, Option<&CheekyOnly>, &mut Visibility), Or<(With<FeatureTag>, With<CheekyOnly>)>>,
    mut last: Local<Option<(Features, bool)>>,
) {
    // the menu changes the features (and Cheeky mode) too, not only the F keys
    let cheeky = game.options.adult;
    let mut changed = *last != Some((yard_res.0.features, cheeky));
    for (key, f) in [
        (KeyCode::F1, Feature::Bar),
        (KeyCode::F2, Feature::Bbq),
        (KeyCode::F3, Feature::Chest),
    ] {
        if keys.just_pressed(key) && game.rules.mode != bbq_core::GameMode::Race {
            let mut feats: Features = yard_res.0.features;
            feats.toggle(f);
            let spot = yard_res.0.chest_spot;
            yard_res.0 = match yard_res.0.heist {
                Some(teams) => Yard::heist(feats, spot, teams),
                None if game.rules.mode == bbq_core::GameMode::Smoko => Yard::smoko(feats, spot),
                None => Yard::new(feats, spot),
            };
            changed = true;
        }
    }
    if changed {
        let feats = yard_res.0.features;
        *last = Some((feats, cheeky));
        for (tag, only, mut vis) in &mut tags {
            let on = tag.is_none_or(|t| feats.get(t.0)) && only.is_none_or(|o| o.0 == cheeky);
            *vis = if on { Visibility::Inherited } else { Visibility::Hidden };
        }
    }
}

#[derive(Component)]
struct SmokoMap;
/// Only the selected Smoko map creates these props; other maps keep their open lawn.
fn sync_smoko_map(
    mut commands: Commands, game: Res<Game>, roots: Query<Entity, With<SmokoMap>>,
    mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>,
    mut cache: ResMut<ModelCache>, mode: Res<crate::lighting::LookMode>,
) {
    let active=game.rules.mode==bbq_core::GameMode::Smoko;
    if !active { for e in &roots {commands.entity(e).despawn();} return; }
    if !roots.is_empty(){return;}
    let polished=*mode==crate::lighting::LookMode::Polished;
    let mut parts=looks_yard::yard_styled(LOOK_SEED,polished).smoko;
    parts.push(looks_yard::smoko_pad(crate::smoko_mode::RADIUS));
    parts.push(Part::new(bbq_core::looks::Shape::Ring { inner: crate::smoko_mode::RADIUS-0.16, outer: crate::smoko_mode::RADIUS, seg:64 },
        bbq_core::looks::Surface::matt(0xffd84a).no_shadow())
        .at(bbq_core::yard::SMOKO_X,0.035,bbq_core::yard::SMOKO_Z).turn(-std::f32::consts::FRAC_PI_2,0.0,0.0));
    // Chairs surround the boundary, leaving four wide entry lanes and the centre clear.
    for i in 0..4 {
        let a=std::f32::consts::FRAC_PI_4+i as f32*std::f32::consts::FRAC_PI_2;
        let chair=if polished {looks_yard::chair_styled(i)} else {looks_yard::chair()};
        parts.extend(looks_yard::place(chair,bbq_core::vec::V3::new(bbq_core::yard::SMOKO_X+a.cos()*4.3,0.0,bbq_core::yard::SMOKO_Z+a.sin()*4.3),bbq_core::vec::Quat::IDENTITY,1.0));
    }
    let delta=Vec3::new(crate::smoko_mode::CENTRE.0-bbq_core::yard::SMOKO_X,0.0,crate::smoko_mode::CENTRE.1-bbq_core::yard::SMOKO_Z);
    let e=cache.spawn_parts(&mut commands,&parts,&mut meshes,&mut mats,Transform::from_translation(delta));
    commands.entity(e).insert(SmokoMap);
}

fn sync_race_map(
 mut commands:Commands,game:Res<Game>,mut normal:Query<&mut Visibility,With<NormalYard>>,
 roots:Query<Entity,With<RaceMap>>,mut meshes:ResMut<Assets<Mesh>>,mut mats:ResMut<Assets<StandardMaterial>>,mut cache:ResMut<ModelCache>,
) {
 let active=game.rules.mode==bbq_core::GameMode::Race;
 for mut v in &mut normal {*v=if active{Visibility::Hidden}else{Visibility::Inherited};}
 if !active {for e in &roots{commands.entity(e).despawn();}return;}
 if !roots.is_empty(){return;}
 use bbq_core::looks::{Shape,Surface};
 let block=|w:f32,h:f32,d:f32,x:f32,y:f32,z:f32,c:u32|Part::new(Shape::Cuboid{w,h,d},Surface::matt(c)).at(x,y,z);
 // Reuse the backyard lawn tile across the oval; no extra geometry or texture assets.
 let mut lawn_surface=Surface::matt(0xd4e4b6).textured(bbq_core::looks::Tex::Lawn);
 lawn_surface.repeat=(12.,9.);
 let mut lawn=Part::new(Shape::Cylinder{top:1.,bottom:1.,h:0.2,seg:96,caps:true},lawn_surface).at(0.,-0.12,0.);
 lawn.scale=bbq_core::vec::V3::new(60.,1.,42.);
 let mut parts=vec![lawn];
 // Many short shared strips make a smooth constant-width racing surface.
 for i in 0..96 {
  let t=i as f32*std::f32::consts::TAU/96.;let u=(i+1) as f32*std::f32::consts::TAU/96.;
  let a=crate::race::oval(t);let b=crate::race::oval(u);let dx=b.0-a.0;let dz=b.1-a.1;let len=dx.hypot(dz);let yaw=dx.atan2(dz);
  parts.push(block(crate::race::WIDTH,0.04,len+0.12,(a.0+b.0)*0.5,0.015,(a.1+b.1)*0.5,0xd48b57).turn(0.,yaw,0.));
  for side in [-1.,1.] {let off=crate::race::WIDTH*0.5*side;parts.push(block(0.12,0.035,len+0.08,(a.0+b.0)*0.5+dz/len*off,0.055,(a.1+b.1)*0.5-dx/len*off,0xfff5cf).turn(0.,yaw,0.));}
  let a=(50.*t.sin(),-35.*t.cos());let b=(50.*u.sin(),-35.*u.cos());let dx=b.0-a.0;let dz=b.1-a.1;
  parts.push(block(0.14,1.1,dx.hypot(dz)+0.04,(a.0+b.0)*0.5,0.55,(a.1+b.1)*0.5,0x99623a).turn(0.,dx.atan2(dz),0.));
 }
 // Curved spectator terrace follows the near bend, outside the racing lane.
 for i in 0..40 {let a=-0.85+i as f32*1.7/40.;let b=-0.85+(i+1) as f32*1.7/40.;let p=(44.*a.sin(),-29.5*a.cos());let q=(44.*b.sin(),-29.5*b.cos());let dx=q.0-p.0;let dz=q.1-p.1;
  parts.push(block(3.4,0.035,dx.hypot(dz)+0.12,(p.0+q.0)*0.5,0.01,(p.1+q.1)*0.5,0xe6bd79).turn(0.,dx.atan2(dz),0.));
 }
 for i in 0..8 {
  let a=crate::race::TRACK[i];let b=crate::race::TRACK[(i+1)%8];let dx=b.0-a.0;let dz=b.1-a.1;let len=dx.hypot(dz);let yaw=dx.atan2(dz);let mid=((a.0+b.0)*0.5,(a.1+b.1)*0.5);
  // Repeated arrow chevrons on the racing surface communicate direction without colour alone.
  for t in [0.3,0.65] {let (x,z)=crate::race::oval((i as f32+t)*std::f32::consts::TAU/8.);
   for side in [-1.,1.] {parts.push(block(0.12,0.04,0.85,x+dz/len*side*0.27,0.08,z-dx/len*side*0.27,0xffdf43).turn(0.,yaw-side*0.7,0.));}
  }
  // Checkpoint gate: two posts, overhead numbered panel, open lane below.
  let n=(i+1)%8;let at=crate::race::TRACK[n];
  for side in [-1.,1.] {parts.push(block(0.16,3.3,0.16,at.0+dz/len*2.65*side,1.65,at.1-dx/len*2.65*side,0xfdf2c6));}
  parts.push(block(5.5,0.7,0.16,at.0,3.3,at.1,if n==0{0x173c36}else{0x225c88}).turn(0.,yaw,0.));
  // Seven-segment gate numbers (finish is 8), made from cheap shared cuboids.
  let digit=if n==0{8}else{n};let masks=[0,0b0110000,0b1101101,0b1111001,0b0110011,0b1011011,0b1011111,0b1110000,0b1111111];
  let segments=[(0.,0.24,0.3,0.055),(0.17,0.12,0.055,0.22),(0.17,-0.12,0.055,0.22),(0.,-0.24,0.3,0.055),(-0.17,-0.12,0.055,0.22),(-0.17,0.12,0.055,0.22),(0.,0.,0.3,0.055)];
  for (k,(x,y,w,h)) in segments.iter().enumerate(){if masks[digit]&(1<<(6-k))!=0 {
   parts.push(block(*w,*h,0.2,at.0+x*yaw.cos(),3.3+y,at.1-x*yaw.sin(),0xffdc43).turn(0.,yaw,0.));
  }}
 }
 // Chequered finish stripe across the lane at gate 8.
 for x in 0..2{for z in 0..10 {parts.push(block(0.4,0.04,0.48,-0.2+x as f32*0.4,0.09,-26.4+z as f32*0.48,if (x+z)%2==0{0xffffff}else{0x172823}));}}
 // Timber boundary, trees and spectator seats give the circuit a backyard identity.
 for i in 0..10 {let t=i as f32*std::f32::consts::TAU/10.;let x=47.*t.sin();let z=-32.*t.cos();if z < -25. {continue;}
  parts.push(block(0.5,3.,0.5,x,1.5,z,0x825433));parts.push(Part::new(Shape::Sphere{r:1.7,ws:8,hs:6},Surface::matt(0x386b37)).at(x,4.,z));
 }
 for t in [-0.7_f32,-0.35,0.,0.35,0.7] {parts.extend(looks_yard::place(looks_yard::chair_styled(0),bbq_core::vec::V3::new(45.*t.sin(),0.,-30.5*t.cos()),bbq_core::vec::Quat::from_euler_xyz(0.,-t,0.),1.0));}
 // Four distinct spectator landmarks beyond the racing lane and throwing sightlines.
 // North: a striped marquee; east: corrugated shed; west: tyre/plant garden.
 for (x,z) in [(-17.0,30.0),(17.0,30.0)] {
  for dx in [-2.4,2.4] {for dz in [-1.4,1.4] {parts.push(block(0.12,3.2,0.12,x+dx,1.6,z+dz,0xf7eccf));}}
  for i in 0..8 {parts.push(block(0.64,0.18,3.4,x-2.24+i as f32*0.64,3.2,z,if i%2==0{0xe36e42}else{0xffe4b1}));}
  parts.push(block(3.5,0.18,1.1,x,0.9,z,0x9a673f));
  for dx in [-1.4,1.4] {parts.push(block(0.16,0.9,0.7,x+dx,0.45,z,0x493f38));}
 }
 // Garden shed sits beyond the oval boundary; never an invisible track obstacle.
 parts.push(block(5.,3.,4.,55.,1.5,6.,0x849c8c));
 for i in 0..20 {parts.push(block(0.07,2.9,0.05,52.6+i as f32*0.25,1.5,3.97,0xaac0a7));}
 parts.push(block(5.6,0.18,4.6,55.,3.15,6.,0xb65e42).turn(0.,0.,0.08));
 parts.push(block(1.4,2.2,0.08,55.,1.1,3.94,0x3b6665));
 parts.push(block(0.12,0.12,0.12,55.5,1.2,3.86,0xffd457));
 for (x,z) in [(-44.,-11.),(-45.,7.),(40.,18.),(9.,32.)] {
  for j in 0..3 {let dx=(j as f32-1.)*0.9;
   parts.push(Part::new(Shape::Cylinder{top:0.45,bottom:0.34,h:0.6,seg:8,caps:true},Surface::matt(0xba764e)).at(x+dx,0.3,z));
   parts.push(Part::new(Shape::Sphere{r:0.6,ws:8,hs:6},Surface::matt(if j%2==0{0x69834b}else{0x87974b})).at(x+dx,0.95,z));
  }
 }
 // Bunting sits high above spectators, leaving the near-bend aiming corridor open.
 for side in [-1.,1.] {for i in 0..7 {
  let x=side*12.+i as f32*0.75;let z=31.;
  parts.push(block(0.65,0.55,0.06,x,3.5,z,if i%3==0{0xe96c42}else if i%3==1{0xf3cc4e}else{0x4c99a6}).turn(0.,0.,0.08*(i as f32-3.)));
 }}
 let e=cache.spawn_parts(&mut commands,&parts,&mut meshes,&mut mats,Transform::default());commands.entity(e).insert(RaceMap);
}

fn sync_backyard_cover(game:Res<Game>,mut q:Query<&mut Visibility,With<BackyardCover>>) {
 for mut v in &mut q {*v=if matches!(game.rules.mode,bbq_core::GameMode::FreeForAll|bbq_core::GameMode::Teams) {Visibility::Inherited}else{Visibility::Hidden};}
}

#[derive(Component)]
struct RaceReturnMarker;
fn sync_race_return_marker(mut commands:Commands,game:Res<Game>,mut q:Query<(Entity,&mut Transform),With<RaceReturnMarker>>,mut meshes:ResMut<Assets<Mesh>>,mut mats:ResMut<Assets<StandardMaterial>>) {
    let target=if game.rules.mode==bbq_core::GameMode::Race {game.race.racers.get(&crate::player::PLAYER_ID).and_then(|r|r.return_to)}else{None};
    let Some((x,z))=target else {for (e,_) in &q {commands.entity(e).despawn();}return;};
    if let Ok((_,mut tf))=q.single_mut() {tf.translation=Vec3::new(x,0.14,z);return;}
    commands.spawn((RaceReturnMarker,Mesh3d(meshes.add(Torus::new(0.8,1.0))),MeshMaterial3d(mats.add(StandardMaterial{base_color:Color::srgb(1.0,0.84,0.2),unlit:true,..default()})),Transform::from_xyz(x,0.14,z),bevy::light::NotShadowCaster));
}
