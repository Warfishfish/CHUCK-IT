//! Draws the items, what you're holding, the practice dummies and puddles, and the
//! throw-path preview. The items are drawn from the shape lists in `bbq_core::looks`.

use std::collections::HashMap;

use bbq_core::flight::{ITEM_GRAV, ItemId, ItemState};
use bbq_core::hands;
use bbq_core::items::ItemKind;
use bevy::prelude::*;

use crate::game::{Game, aim_dir, hand_pos};
use crate::models::{ModelCache, ModelKey};
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

/// How an item lies when resting, as the browser game does it (`orientRest`): a random turn,
/// long things on their side, the fish flat.
fn rest_rotation(kind: ItemKind, id: ItemId) -> Quat {
    let yaw = ((id.wrapping_mul(9301).wrapping_add(49297)) % 233_280) as f32 / 233_280.0
        * std::f32::consts::TAU;
    let long = matches!(kind, ItemKind::Stubby | ItemKind::Dildo | ItemKind::Noodle);
    let z = if long { std::f32::consts::FRAC_PI_2 } else { 0.0 };
    let x = if kind == ItemKind::Fish {
        std::f32::consts::FRAC_PI_2
    } else {
        0.0
    };
    Quat::from_euler(EulerRot::XYZ, x, yaw, z)
}

/// The dildo and the noodle flop about; the others are stiff.
fn is_floppy(kind: ItemKind) -> bool {
    matches!(kind, ItemKind::Dildo | ItemKind::Noodle)
}

fn sync_items(
    mut commands: Commands,
    game: Res<Game>,
    look: Res<Look>,
    mut shown: ResMut<Shown>,
    mut cache: ResMut<ModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut q: Query<(&ItemVisual, &mut Transform, &mut Visibility), Without<GlowRing>>,
    mut rings: Query<(&GlowRing, &mut Transform, &mut Visibility), Without<ItemVisual>>,
) {
    // spawn visuals for new items
    for (id, it) in &game.world.items {
        if !shown.items.contains_key(id) {
            let e = cache.spawn(
                &mut commands,
                ModelKey::item(it.kind, it.variant, *id),
                &mut meshes,
                &mut mats,
                Transform::from_xyz(it.pos.x, it.pos.y, it.pos.z),
            );
            commands.entity(e).insert(ItemVisual(*id));
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
                tf.rotation = rest_rotation(it.kind, it.id);
                if floating {
                    // bobbing on the water, as in the browser game
                    let y = bbq_core::yard::WATER_Y + 0.04 + (t * 2.0 + it.pos.x).sin() * 0.03;
                    tf.translation = Vec3::new(it.pos.x, y, it.pos.z);
                    let (_, yaw, _) = tf.rotation.to_euler(EulerRot::XYZ);
                    tf.rotation = Quat::from_euler(
                        EulerRot::XYZ,
                        (t * 1.3 + it.id as f32).sin() * 0.08,
                        yaw,
                        if matches!(it.kind, ItemKind::Noodle) {
                            std::f32::consts::FRAC_PI_2
                        } else {
                            0.0
                        },
                    );
                } else {
                    tf.translation = Vec3::new(it.pos.x, it.pos.y, it.pos.z);
                }
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
    mut cache: ResMut<ModelCache>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut vm: Query<(Entity, &mut Transform, &mut Visibility), With<HeldVm>>,
    mut shown: Local<Option<(ItemKind, ModelKey)>>,
) {
    let selected = game
        .slots
        .selected()
        .and_then(|id| game.world.items.get(&id))
        .map(|i| (i.kind, ModelKey::item(i.kind, i.variant, i.id)));
    let Ok((root, mut tf, mut vis)) = vm.single_mut() else {
        let e = commands
            .spawn((Transform::default(), Visibility::Hidden, HeldVm))
            .id();
        commands.entity(*cam).add_child(e);
        return;
    };
    let Some((kind, key)) = selected.filter(|_| !game.me.drunk.is_drinking()) else {
        *vis = Visibility::Hidden;
        *shown = None;
        return;
    };
    if shown.map(|s| s.1) != Some(key) {
        // a different thing in the hand: swap the parts
        commands.entity(root).despawn_children();
        let built = cache.built(key, &mut meshes, &mut mats);
        commands.entity(root).with_children(|p| {
            for b in built {
                p.spawn((Mesh3d(b.mesh), MeshMaterial3d(b.material), b.transform));
            }
        });
        *shown = Some((kind, key));
    }
    *vis = Visibility::Inherited;
    let c = if game.wind.charging {
        game.wind.charge
    } else {
        0.0
    };
    let sw = (player.walk).sin() * 0.02;
    let flop = is_floppy(kind);
    let scale = match kind {
        ItemKind::Noodle | ItemKind::Fish => 0.36,
        ItemKind::Dildo => 0.5,
        _ => 0.8,
    };
    let fl = if flop { 0.04 } else { 0.0 };
    tf.translation = Vec3::new(
        0.3 + c * 0.08 + sw + fl,
        -0.3 + c * 0.12 + sw.abs() - fl,
        -0.62 + c * 0.2,
    );
    tf.rotation = Quat::from_euler(
        EulerRot::XYZ,
        (if flop { -0.45 } else { 0.15 }) - c * 0.6,
        0.3,
        c * 0.3,
    );
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
