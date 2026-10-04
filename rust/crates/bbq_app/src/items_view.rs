//! Draws the items, what you're holding, the practice dummies and puddles, and the
//! throw-path preview. Simple shapes for now; real models come in Phase 4.

use std::collections::HashMap;

use bbq_core::flight::{ITEM_GRAV, ItemId, ItemState};
use bbq_core::hands;
use bbq_core::items::ItemKind;
use bevy::prelude::*;

use crate::game::{Game, aim_dir, hand_pos};
use crate::player::{EyeCamera, Player};

pub fn kind_name(k: ItemKind) -> &'static str {
    match k {
        ItemKind::Teddy => "Teddy",
        ItemKind::Stubby => "VP can",
        ItemKind::Gnome => "Gnome",
        ItemKind::Dildo => "Cheeky",
        ItemKind::Steak => "Steak",
        ItemKind::Fish => "Fish",
        ItemKind::Noodle => "Noodle",
        ItemKind::Snag => "Snag",
    }
}

#[derive(Component)]
struct ItemVisual(ItemId);
#[derive(Component)]
struct GlowRing(ItemId);
#[derive(Component)]
struct PuddleVisual;
#[derive(Component)]
struct HeldVm;

#[derive(Resource, Default)]
struct Shown {
    items: HashMap<ItemId, (Entity, Entity)>,
}

#[derive(Resource)]
struct Look {
    ring: Handle<Mesh>,
    ring_mat: Handle<StandardMaterial>,
    puddle: Handle<Mesh>,
    puddle_mat: Handle<StandardMaterial>,
}

pub struct ItemsViewPlugin;

impl Plugin for ItemsViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shown>()
            .add_systems(Startup, setup_look)
            .add_systems(
                Update,
                (sync_items, sync_viewmodel, sync_puddles, draw_trajectory),
            );
    }
}

fn setup_look(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(Look {
        ring: meshes.add(Cylinder::new(0.5, 0.02)),
        ring_mat: mats.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.9, 0.2, 0.55),
            emissive: LinearRgba::new(1.0, 0.8, 0.1, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        puddle: meshes.add(Cylinder::new(1.0, 0.01)),
        puddle_mat: mats.add(StandardMaterial {
            base_color: Color::srgba(0.85, 0.59, 0.18, 0.55),
            alpha_mode: AlphaMode::Blend,
            ..default()
        }),
    });
}

fn item_mesh(kind: ItemKind, meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    let r = kind.def().radius;
    match kind {
        ItemKind::Teddy => meshes.add(Sphere::new(r)),
        ItemKind::Stubby => meshes.add(Cylinder::new(0.045, 0.24)),
        ItemKind::Gnome => meshes.add(Cone {
            radius: 0.2,
            height: 0.55,
        }),
        ItemKind::Noodle => meshes.add(Cylinder::new(0.06, 1.2)),
        ItemKind::Steak | ItemKind::Fish => meshes.add(Cuboid::new(0.3, 0.05, 0.2)),
        ItemKind::Dildo | ItemKind::Snag => meshes.add(Capsule3d::new(0.05, 0.3)),
    }
}

pub fn item_colour(kind: ItemKind) -> Color {
    match kind {
        ItemKind::Teddy => Color::srgb(0.62, 0.42, 0.25),
        ItemKind::Stubby => Color::srgb(0.85, 0.6, 0.1),
        ItemKind::Gnome => Color::srgb(0.85, 0.15, 0.12),
        ItemKind::Noodle => Color::srgb(0.95, 0.4, 0.7),
        ItemKind::Steak => Color::srgb(0.7, 0.12, 0.12),
        ItemKind::Fish => Color::srgb(0.6, 0.7, 0.8),
        ItemKind::Dildo => Color::srgb(0.64, 0.3, 0.88),
        ItemKind::Snag => Color::srgb(0.6, 0.3, 0.2),
    }
}

/// How an item lies when resting: long things lie flat.
fn rest_rotation(kind: ItemKind, id: ItemId) -> Quat {
    let yaw = ((id.wrapping_mul(9301).wrapping_add(49297)) % 233_280) as f32 / 233_280.0
        * std::f32::consts::TAU;
    let base = Quat::from_rotation_y(yaw);
    match kind {
        ItemKind::Stubby | ItemKind::Dildo | ItemKind::Noodle => {
            base * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
        }
        ItemKind::Gnome => base * Quat::from_rotation_x(std::f32::consts::PI), // cone point up looks like a hat: flip so it stands
        _ => base,
    }
}

fn sync_items(
    mut commands: Commands,
    game: Res<Game>,
    look: Res<Look>,
    mut shown: ResMut<Shown>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut q: Query<(&ItemVisual, &mut Transform, &mut Visibility), Without<GlowRing>>,
    mut rings: Query<(&GlowRing, &mut Transform, &mut Visibility), Without<ItemVisual>>,
) {
    // spawn visuals for new items
    for (id, it) in &game.world.items {
        if !shown.items.contains_key(id) {
            let mesh = item_mesh(it.kind, &mut meshes);
            let e = commands
                .spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mats.add(item_colour(it.kind))),
                    Transform::from_xyz(it.pos.x, it.pos.y, it.pos.z),
                    ItemVisual(*id),
                ))
                .id();
            let ring = commands
                .spawn((
                    Mesh3d(look.ring.clone()),
                    MeshMaterial3d(look.ring_mat.clone()),
                    Transform::from_xyz(it.pos.x, 0.03, it.pos.z),
                    Visibility::Hidden,
                    GlowRing(*id),
                ))
                .id();
            shown.items.insert(*id, (e, ring));
        }
    }
    // remove visuals for items that are gone
    let gone: Vec<ItemId> = shown
        .items
        .keys()
        .copied()
        .filter(|id| !game.world.items.contains_key(id))
        .collect();
    for id in gone {
        if let Some((e, r)) = shown.items.remove(&id) {
            commands.entity(e).despawn();
            commands.entity(r).despawn();
        }
    }

    let t = time.elapsed_secs();
    for (vis, mut tf, mut v) in &mut q {
        let Some(it) = game.world.items.get(&vis.0) else {
            continue;
        };
        match it.state {
            ItemState::Held => *v = Visibility::Hidden, // the viewmodel shows it
            ItemState::Flying => {
                *v = Visibility::Inherited;
                tf.translation = Vec3::new(it.pos.x, it.pos.y, it.pos.z);
                let sp = it.vel.len();
                let spin = bbq_core::flight::spin_rate(it.kind, sp) * time.delta_secs();
                let axis =
                    Vec3::new(1.0 + (vis.0 % 3) as f32, 0.3, 1.0 + (vis.0 % 5) as f32).normalize();
                tf.rotate(Quat::from_axis_angle(axis, spin));
            }
            ItemState::Ground => {
                *v = Visibility::Inherited;
                let floating =
                    it.ground_y < 0.1 && bbq_core::yard::in_pool_rect(it.pos.x, it.pos.z);
                let bob = if floating {
                    (t * 2.0 + it.pos.x).sin() * 0.03
                } else {
                    0.0
                };
                tf.translation = Vec3::new(it.pos.x, it.pos.y + bob, it.pos.z);
                tf.rotation = rest_rotation(it.kind, it.id);
            }
        }
    }
    for (ring, mut tf, mut v) in &mut rings {
        let Some(it) = game.world.items.get(&ring.0) else {
            continue;
        };
        if it.state == ItemState::Ground {
            *v = Visibility::Inherited;
            let floating = it.ground_y < 0.1 && bbq_core::yard::in_pool_rect(it.pos.x, it.pos.z);
            let y = if floating {
                bbq_core::yard::WATER_Y + 0.02
            } else {
                it.ground_y + 0.03
            };
            let pulse = 0.5 + 0.5 * (t * 4.0 + it.pos.x).sin();
            tf.translation = Vec3::new(it.pos.x, y, it.pos.z);
            tf.scale = Vec3::splat(0.9 + pulse * 0.25);
        } else {
            *v = Visibility::Hidden;
        }
    }
}

/// The thing in your hand, drawn in front of the camera.
fn sync_viewmodel(
    mut commands: Commands,
    game: Res<Game>,
    player: Res<Player>,
    cam: Single<Entity, With<EyeCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut vm: Query<
        (
            &mut Mesh3d,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Transform,
            &mut Visibility,
        ),
        With<HeldVm>,
    >,
    mut last_kind: Local<Option<ItemKind>>,
) {
    let selected = game
        .slots
        .selected()
        .and_then(|id| game.world.items.get(&id))
        .map(|i| i.kind);
    let Ok((mut mesh, mut mat, mut tf, mut vis)) = vm.single_mut() else {
        let e = commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(0.1))),
                MeshMaterial3d(mats.add(Color::WHITE)),
                Transform::default(),
                Visibility::Hidden,
                HeldVm,
            ))
            .id();
        commands.entity(*cam).add_child(e);
        return;
    };
    let Some(kind) = selected.filter(|_| !game.me.drunk.is_drinking()) else {
        *vis = Visibility::Hidden;
        *last_kind = None;
        return;
    };
    if *last_kind != Some(kind) {
        mesh.0 = item_mesh(kind, &mut meshes);
        mat.0 = mats.add(item_colour(kind));
        *last_kind = Some(kind);
    }
    *vis = Visibility::Inherited;
    let c = if game.wind.charging {
        game.wind.charge
    } else {
        0.0
    };
    let sw = (player.walk).sin() * 0.02;
    let scale = match kind {
        ItemKind::Noodle | ItemKind::Fish => 0.36,
        ItemKind::Dildo => 0.5,
        _ => 0.8,
    };
    tf.translation = Vec3::new(
        0.3 + c * 0.08 + sw,
        -0.3 + c * 0.12 + sw.abs(),
        -0.62 + c * 0.2,
    );
    tf.rotation = Quat::from_euler(EulerRot::XYZ, 0.15 - c * 0.6, 0.3, c * 0.3);
    tf.scale = Vec3::splat(scale);
}

fn sync_puddles(
    mut commands: Commands,
    game: Res<Game>,
    look: Res<Look>,
    old: Query<Entity, With<PuddleVisual>>,
) {
    for e in &old {
        commands.entity(e).despawn();
    }
    for p in &game.world.puddles.0 {
        commands.spawn((
            Mesh3d(look.puddle.clone()),
            MeshMaterial3d(look.puddle_mat.clone()),
            Transform::from_xyz(p.x, 0.03, p.z).with_scale(Vec3::new(p.size(), 1.0, p.size())),
            PuddleVisual,
        ));
    }
}

/// While winding up, draw where the throw would go (ignoring things in the way).
fn draw_trajectory(mut gizmos: Gizmos, game: Res<Game>, player: Res<Player>) {
    if !game.wind.charging {
        return;
    }
    let Some(kind) = game
        .slots
        .selected()
        .and_then(|id| game.world.items.get(&id))
        .map(|i| i.kind)
    else {
        return;
    };
    let def = kind.def();
    let mut vel = hands::throw_velocity(
        aim_dir(player.yaw, player.pitch),
        def.speed,
        game.wind.charge,
        bbq_core::vec::V3::new(player.mover.vx, 0.0, player.mover.vz),
    );
    let mut pos = hand_pos(&player);
    let mut pts = vec![Vec3::new(pos.x, pos.y, pos.z)];
    let h = 0.03;
    for _ in 0..100 {
        vel.y -= ITEM_GRAV * def.grav * h;
        pos += vel * h;
        pts.push(Vec3::new(pos.x, pos.y.max(0.02), pos.z));
        if pos.y <= 0.0 {
            break;
        }
    }
    let power = game.wind.charge >= bbq_core::stun::POWER_CHARGE;
    let colour = if power {
        Color::srgb(1.0, 0.3, 0.2)
    } else {
        Color::srgb(1.0, 1.0, 1.0)
    };
    gizmos.linestrip(pts, colour);
}
