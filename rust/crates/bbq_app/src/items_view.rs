//! Draws the items, what you're holding, the practice dummies and puddles, and the
//! throw-path preview. The items are drawn from the shape lists in `bbq_core::looks`.

use std::collections::HashMap;

use bbq_core::flight::{ITEM_GRAV, ItemId, ItemState};
use bbq_core::hands;
use bbq_core::items::{ItemKind, Melee};
use bevy::prelude::*;

use crate::game::{Game, aim_dir, hand_pos};
use crate::models::{FloppyChain, FloppySeg, ModelCache, ModelKey};
use crate::player::{EyeCamera, Player};

/// The name of one item (Cheeky mode's bottles say which they are).
pub fn item_name(it: &bbq_core::flight::Item) -> &'static str {
    match it.bottle {
        Some(b) if it.kind == ItemKind::Stubby => b.label(),
        _ => kind_name(it.kind),
    }
}

pub fn kind_name(k: ItemKind) -> &'static str {
    match k {
        ItemKind::Teddy => "Teddy",
        ItemKind::Stubby => "VP can",
        ItemKind::WaterBalloon => "Water balloon",
        ItemKind::Dildo => "Cheeky",
        ItemKind::Steak => "Steak",
        ItemKind::Fish => "Fish",
        ItemKind::Noodle => "Noodle",
        ItemKind::Snag => "Snag",
    }
}

#[derive(Component)]
struct ItemVisual(ItemId);

/// Visual angular inertia only; the authoritative hit/catch sphere is unchanged.
#[derive(Component)]
struct ItemMotion {
    ready: bool,
    state: ItemState,
    velocity: Vec3,
    spin: f32,
    axis: Vec3,
    rest: Quat,
}
impl Default for ItemMotion {
    fn default() -> Self { Self { ready:false, state:ItemState::Ground, velocity:Vec3::ZERO, spin:0.0, axis:Vec3::X, rest:Quat::IDENTITY } }
}

fn launch_spin(kind: ItemKind, speed: f32, scale: f32) -> f32 {
    if kind == ItemKind::Noodle { (2.0 + speed * 0.22).min(7.0) }
    else { ((3.0 + speed * 0.38) / scale.max(0.5).powf(1.4)).min(15.0) }
}

fn lie_along(rotation: Quat, fallback: Quat) -> Quat {
    let shaft=rotation * Vec3::Y;
    if shaft.x * shaft.x + shaft.z * shaft.z < 0.001 { return fallback; }
    Quat::from_rotation_y(shaft.z.atan2(-shaft.x)) * Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)
}
#[derive(Component)]
struct GlowRing(ItemId);
#[derive(Component)]
struct PuddleVisual;
#[derive(Component)]
struct HeldVm;
/// The floppy thing drawn in your hand (a child of `HeldVm`).
#[derive(Component)]
struct VmFloppy;

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum FlopMode {
    #[default]
    Ground,
    Held,
    Flying,
    /// Sitting in the chest: a slow gentle sway.
    Chest,
}

/// What is shaking a floppy item right now (set by whoever draws it).
#[derive(Component, Default)]
pub struct FlopDrive {
    pub mode: FlopMode,
    /// Collision-induced velocity change, in m/s (visual spring impulse).
    pub impact: f32,
    /// How fast the person holding it is walking.
    pub speed: f32,
    /// It is in your own hand (the droop hangs the other way, as in the browser).
    pub mine: bool,
}

/// The springy state of one floppy item: how far it leans, how fast, and what it saw last frame.
#[derive(Component)]
pub struct Wobble {
    x: f32,
    z: f32,
    vx: f32,
    vz: f32,
    wx: f32,
    wz: f32,
    prev: Quat,
    phase: f32,
    held_speed: f32,
    bends: [Vec2; 7],
    ready: bool,
    seed: f32,
}

impl Wobble {
    pub fn with_seed(seed: f32) -> Self {
        Wobble { seed, ..default() }
    }
}

impl FlopDrive {
    pub fn in_chest() -> Self {
        FlopDrive { mode: FlopMode::Chest, ..default() }
    }
}

impl Default for Wobble {
    fn default() -> Self {
        Wobble { x: 0.0, z: 0.0, vx: 0.0, vz: 0.0, wx: 0.0, wz: 0.0, prev: Quat::IDENTITY, phase: 0.0, held_speed: 0.0, bends: [Vec2::ZERO; 7], ready: false, seed: 0.0 }
    }
}

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

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HeldViewSet;

pub struct ItemsViewPlugin;

impl Plugin for ItemsViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Shown>()
            .add_systems(Startup, (setup_look, setup_trajectory))
            .add_systems(
                Update,
                (
                    sync_items,
                    sync_viewmodel.in_set(HeldViewSet),
                    sync_puddles,
                    draw_trajectory,
                    attach_wobble,
                    wobble_items.after(sync_items).after(sync_viewmodel),
                ),
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
            base_color: Color::linear_rgba(1.0, 0.9, 0.2, 0.55),
            emissive: LinearRgba::new(1.0, 0.8, 0.1, 1.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        }),
        puddle: meshes.add(Cylinder::new(1.0, 0.01)),
        puddle_mat: mats.add(StandardMaterial {
            base_color: Color::linear_rgba(0.85, 0.59, 0.18, 0.55),
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
    let z = if long {
        std::f32::consts::FRAC_PI_2
    } else {
        0.0
    };
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
    mut q: Query<(&ItemVisual, &mut Transform, &mut Visibility, &mut ItemMotion), Without<GlowRing>>,
    mut rings: Query<(&GlowRing, &mut Transform, &mut Visibility), Without<ItemVisual>>,
    mut drives: Query<&mut FlopDrive>,
) {
    // spawn visuals for new items
    for (id, it) in &game.world.items {
        if !shown.items.contains_key(id) {
            let e = cache.spawn(
                &mut commands,
                ModelKey::of(it),
                &mut meshes,
                &mut mats,
                Transform::from_xyz(it.pos.x, it.pos.y, it.pos.z),
            );
            commands.entity(e).insert((ItemVisual(*id), ItemMotion::default()));
            // a Heist teddy carries a little flag in its team's colour
            if let Some(team) = it.team.and_then(crate::heist_app::team_of_index) {
                let flag = cache.spawn_parts(
                    &mut commands,
                    &bbq_core::looks_yard::teddy_flag(team),
                    &mut meshes,
                    &mut mats,
                    Transform::default(),
                );
                commands.entity(e).add_child(flag);
            }
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
    for (vis, mut tf, mut v, mut motion) in &mut q {
        let Some(it) = game.world.items.get(&vis.0) else {
            continue;
        };
        let dt=time.delta_secs().clamp(0.0001,0.05);
        let velocity=Vec3::new(it.vel.x,it.vel.y,it.vel.z);
        let impact=if motion.ready && motion.state==ItemState::Flying {
            (velocity-motion.velocity+Vec3::Y * ITEM_GRAV * it.kind.def().grav * dt).length().clamp(0.0,12.0)
        } else { 0.0 };
        let changed=!motion.ready || motion.state!=it.state;
        if changed && is_floppy(it.kind) {
            if it.state==ItemState::Flying {
                motion.spin=launch_spin(it.kind,velocity.length(),it.variant.map_or(1.0,|v| v.def().scale));
                motion.axis=if velocity.x.hypot(velocity.z)>0.5 { Vec3::new(velocity.z,0.15*velocity.length(),-velocity.x).normalize() } else { Vec3::new(1.0,0.12,0.35).normalize() };
            } else if it.state==ItemState::Ground {
                motion.rest=if motion.ready { lie_along(tf.rotation,rest_rotation(it.kind,it.id)) } else { rest_rotation(it.kind,it.id) };
                if !motion.ready { tf.rotation=motion.rest; }
            }
        } else if impact>1.5 { motion.spin *= if it.kind==ItemKind::Noodle { 0.3 } else { 0.5 }; }
        if let Some((e, _)) = shown.items.get(&vis.0)
            && let Ok(mut d) = drives.get_mut(*e)
        {
            d.mode = match it.state {
                ItemState::Held => FlopMode::Held,
                ItemState::Flying => FlopMode::Flying,
                ItemState::Ground => FlopMode::Ground,
            };
            d.mine = false;
            d.impact = if impact>1.5 { impact } else { 0.0 };
            d.speed = it
                .holder
                .and_then(|h| game.dummies.iter().find(|x| x.id == h))
                .map_or(0.0, |x| if x.mover.grounded { x.mover.speed() } else { 0.0 });
            if it.state==ItemState::Flying { d.speed=velocity.length(); }
        }
        match it.state {
            ItemState::Held => {
                // yours is drawn by the viewmodel; a bot's is in its right hand
                let bot_hand = it
                    .holder
                    .filter(|h| *h != crate::player::PLAYER_ID)
                    .filter(|h| crate::bots_app::is_selected_by_bot(&game, *h, it.id))
                    .and_then(|h| crate::bots_app::held_item_pos(&game, h));
                match bot_hand {
                    Some(pos) => {
                        *v = Visibility::Inherited;
                        tf.translation = Vec3::new(pos.x, pos.y, pos.z);
                        tf.rotation = rest_rotation(it.kind, it.id);
                        if is_floppy(it.kind) || it.kind == ItemKind::Stubby {
                            tf.rotation = Quat::IDENTITY;
                        }
                        tf.scale = Vec3::splat(0.85);
                    }
                    None => *v = Visibility::Hidden,
                }
            }
            ItemState::Flying => {
                tf.scale = Vec3::ONE;
                *v = Visibility::Inherited;
                tf.translation = Vec3::new(it.pos.x, it.pos.y, it.pos.z);
                let sp = it.vel.len();
                let spin = bbq_core::flight::spin_rate(it.kind, sp) * time.delta_secs();
                let axis =
                    Vec3::new(1.0 + (vis.0 % 3) as f32, 0.3, 1.0 + (vis.0 % 5) as f32).normalize();
                if is_floppy(it.kind) {
                    motion.spin *= (-(if it.kind==ItemKind::Noodle { 0.65 } else { 0.14 }) * dt).exp();
                    tf.rotate(Quat::from_axis_angle(motion.axis, motion.spin * dt));
                    if !it.live && it.pos.y-it.ground_y<0.6 && it.vel.y<0.0 {
                        let target=lie_along(tf.rotation,rest_rotation(it.kind,it.id));
                        tf.rotation=tf.rotation.slerp(target,1.0-(-9.0*dt).exp());
                    }
                } else { tf.rotate(Quat::from_axis_angle(axis, spin)); }
            }
            ItemState::Ground => {
                tf.scale = Vec3::ONE;
                *v = Visibility::Inherited;
                let floating =
                    it.ground_y < 0.1 && bbq_core::yard::in_pool_rect(it.pos.x, it.pos.z);
                if is_floppy(it.kind) && !floating {
                    tf.rotation=tf.rotation.slerp(motion.rest,1.0-(-14.0*dt).exp());
                    if tf.rotation.angle_between(motion.rest)<0.003 { tf.rotation=motion.rest; }
                } else { tf.rotation = rest_rotation(it.kind, it.id); }
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
                    if is_floppy(it.kind) {
                        let scale=it.variant.map_or(1.0,|v| v.def().scale);
                        let radius=if it.kind==ItemKind::Dildo { 0.20*scale } else { 0.08 };
                        // Keep the long shaft above the support surface during the short
                        // settling turn; variants use their own visual base radius.
                        let shaft=tf.rotation * Vec3::Y;
                        let length=if it.kind==ItemKind::Dildo { 1.05*scale } else { 2.0 };
                        tf.translation.y=it.pos.y+radius-bbq_core::flight::rest_height(it.kind)+(-shaft.y).max(0.0)*length;
                    }
                }
            }
        }
        motion.ready=true;
        motion.state=it.state;
        motion.velocity=velocity;
    }
    // Teddy Heist: your own team's teddies are not yours to take, so they do not glow
    let my_team = game.teams.get(crate::player::PLAYER_ID);
    for (ring, mut tf, mut v) in &mut rings {
        let Some(it) = game.world.items.get(&ring.0) else {
            continue;
        };
        let own_teddy = my_team.is_some() && it.team.and_then(crate::heist_app::team_of_index) == my_team;
        if it.state == ItemState::Ground && !own_teddy {
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
    mut vm_floppy: Query<&mut FlopDrive, With<VmFloppy>>,
    mut shown: Local<Option<(ItemKind, ModelKey)>>,
) {
    let selected = game
        .slots
        .selected()
        .and_then(|id| game.world.items.get(&id))
        .map(|i| (i.kind, ModelKey::of(i)));
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
        if bbq_core::looks::floppy(kind).is_some() {
            // floppy things hang off a chain of links that the wobble bends
            let e = cache.spawn(&mut commands, key, &mut meshes, &mut mats, Transform::default());
            commands.entity(e).insert(VmFloppy);
            commands.entity(root).add_child(e);
        } else {
            let built = cache.built(key, &mut meshes, &mut mats);
            for b in &built {
                b.spawn_under(&mut commands, root);
            }
        }
        *shown = Some((kind, key));
    }
    *vis = Visibility::Inherited;
    for mut d in &mut vm_floppy {
        d.mode = FlopMode::Held;
        d.mine = true;
        d.speed = if player.mover.grounded { player.mover.speed() } else { 0.0 };
    }
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
        ItemKind::WaterBalloon => 0.62,
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
    if kind==ItemKind::WaterBalloon {
        // Visual water weight: restrained lag and volume-preserving squash, no physics change.
        let movement=(player.mover.speed()/8.0).clamp(0.0,1.0);
        let flex=(player.walk*0.7-0.5).sin()*0.055*movement;
        let sy=1.06+flex-c*0.10;
        tf.scale*=Vec3::new(1.0/sy.sqrt(),sy,1.0/sy.sqrt());
        tf.translation+=Vec3::new(0.015,-0.035,-0.035);
        tf.rotation*=Quat::from_rotation_z(-0.12+flex*1.8);
    }
    // your own slap: the browser's swing across the screen (right to left, tip whipping through)
    if game.life.me_swing > 0.0 && bbq_core::items::ItemKind::def(kind).melee != Melee::None {
        let p = 1.0 - game.life.me_swing / (bbq_core::melee::SWING_TIME * crate::life::swing_mul(&game));
        let a = (std::f32::consts::PI * p).sin();
        tf.translation = Vec3::new(0.42 - 0.8 * p, -0.2 + 0.06 * a, -0.55 - 0.2 * a);
        tf.rotation = Quat::from_euler(EulerRot::XYZ, -0.9 + 0.3 * a, 0.2, -1.3 + 2.2 * p);
    }
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

/// The browser's throw-path preview: 48 small white dots a little over a hundredth of a
/// second apart (two 0.018 s steps each) that stop at the ground or the first thing in the
/// way, and a white ring where it would land.
const TRAJ_DOTS: usize = 48;

#[derive(Component)]
struct TrajDot(usize);
#[derive(Component)]
struct TrajRing;

fn setup_trajectory(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let dot = meshes.add(Rectangle::new(0.11, 0.11));
    let dot_mat = mats.add(StandardMaterial {
        base_color: Color::linear_rgba(1.0, 1.0, 1.0, 0.85),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        cull_mode: None,
        ..default()
    });
    for i in 0..TRAJ_DOTS {
        commands.spawn((
            Mesh3d(dot.clone()),
            MeshMaterial3d(dot_mat.clone()),
            Transform::default(),
            Visibility::Hidden,
            TrajDot(i),
        ));
    }
    commands.spawn((
        Mesh3d(meshes.add(Annulus::new(0.3, 0.42))),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color: Color::linear_rgba(1.0, 1.0, 1.0, 0.8),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })),
        Transform::default(),
        Visibility::Hidden,
        TrajRing,
    ));
}

/// While winding up, show where the throw would go.
fn draw_trajectory(
    game: Res<Game>,
    player: Res<Player>,
    yard: Res<crate::yard_scene::YardRes>,
    mut dots: Query<(&TrajDot, &mut Transform, &mut Visibility), Without<TrajRing>>,
    mut ring: Query<(&mut Transform, &mut Visibility), (With<TrajRing>, Without<TrajDot>)>,
) {
    let kind = game
        .wind
        .charging
        .then(|| {
            game.slots
                .selected()
                .and_then(|id| game.world.items.get(&id))
                .map(|i| i.kind)
        })
        .flatten();
    let Some(kind) = kind else {
        for (_, _, mut v) in &mut dots {
            *v = Visibility::Hidden;
        }
        for (_, mut v) in &mut ring {
            *v = Visibility::Hidden;
        }
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
    let mut pts: Vec<Vec3> = Vec::with_capacity(TRAJ_DOTS);
    let mut hit_y = None;
    'dots: for _ in 0..TRAJ_DOTS {
        for _ in 0..2 {
            vel.y -= ITEM_GRAV * def.grav * 0.018;
            pos += vel * 0.018;
        }
        pts.push(Vec3::new(pos.x, pos.y, pos.z));
        if pos.y <= 0.0 {
            hit_y = Some(0.0);
            break;
        }
        for c in &yard.0.colliders {
            if pos.y < c.h && pos.x > c.x0 && pos.x < c.x1 && pos.z > c.z0 && pos.z < c.z1 {
                hit_y = Some(c.h);
                break 'dots;
            }
        }
        if pos.x.abs() > bbq_core::YARD_HALF_X + 1.0 || pos.z.abs() > bbq_core::YARD_HALF_Z + 1.0 {
            break;
        }
    }
    // the dots are flat squares that face the camera
    let face = Quat::from_euler(EulerRot::YXZ, player.yaw, player.pitch, 0.0);
    for (d, mut tf, mut v) in &mut dots {
        if let Some(p) = pts.get(d.0) {
            tf.translation = *p;
            tf.rotation = face;
            *v = Visibility::Inherited;
        } else {
            *v = Visibility::Hidden;
        }
    }
    for (mut tf, mut v) in &mut ring {
        match (hit_y, pts.last()) {
            (Some(y), Some(p)) => {
                tf.translation = Vec3::new(p.x, y + 0.03, p.z);
                tf.rotation = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                *v = Visibility::Inherited;
            }
            _ => *v = Visibility::Hidden,
        }
    }
}

/// Every floppy chain gets its spring state (a little different each, so they do not move in step).
fn attach_wobble(
    mut commands: Commands,
    q: Query<Entity, (With<FloppyChain>, Without<Wobble>)>,
) {
    for (n, e) in q.iter().enumerate() {
        commands.entity(e).insert((
            Wobble { seed: (e.index_u32() % 97) as f32 + n as f32, ..default() },
            FlopDrive::default(),
        ));
    }
}

/// Damped flex follows turns and gait; a held tip lags behind the grip, then
/// settles after stopping. Throws keep their separate impact/airflow response.
fn wobble_items(
    time: Res<Time>,
    mut roots: Query<(&FloppyChain, &mut Wobble, &FlopDrive, &GlobalTransform)>,
    mut segs: Query<&mut Transform, With<FloppySeg>>,
) {
    let dt = time.delta_secs().min(0.05);
    if dt <= 0.0 {
        return;
    }
    let t = time.elapsed_secs();
    for (chain, mut w, drive, gt) in &mut roots {
        let d = chain.def;
        if drive.mode == FlopMode::Chest {
            // in the chest: each link sways a little more than the one before
            for (i, e) in chain.segs.iter().enumerate() {
                if let Ok(mut tf) = segs.get_mut(*e) {
                    tf.rotation = Quat::from_rotation_x((t * 2.0 + w.seed).sin() * 0.06 * (i as f32 + 1.0) * 0.5);
                }
            }
            continue;
        }
        let q = gt.to_scale_rotation_translation().1;
        // how fast it is turning, in its own frame
        let dq = q * w.prev.inverse();
        let (axis, angle) = dq.to_axis_angle();
        let angle = if angle > std::f32::consts::PI { angle - std::f32::consts::TAU } else { angle };
        let omega = q.inverse() * (axis * (angle / dt));
        let (mut wx, mut wz) = (omega.x, omega.z);
        if !w.ready || angle.abs() > 1.4 || !omega.is_finite() {
            wx = 0.0;
            wz = 0.0;
            w.wx = 0.0;
            w.wz = 0.0;
        }
        w.prev = q;
        w.ready = true;
        wx = wx.clamp(-30.0, 30.0);
        wz = wz.clamp(-30.0, 30.0);
        let held = drive.mode == FlopMode::Held;
        // Filter mouse/camera impulses before feeding the spring. Grip motion should
        // lead the tip, without a one-frame turn producing an oversized whip.
        if held {
            let blend = 1.0 - (-14.0 * dt).exp();
            wx = w.wx + (wx - w.wx) * blend;
            wz = w.wz + (wz - w.wz) * blend;
        }
        let g = if held { 0.38 } else { 0.5 };
        w.vx -= (wx - w.wx) * g;
        w.vz -= (wz - w.wz) * g;
        w.wx = wx;
        w.wz = wz;
        if held {
            advance_held_drive(&mut w, drive.speed, dt, d.step);
        } else {
            w.held_speed = 0.0;
        }
        if drive.impact > 0.0 {
            w.vx -= drive.impact * 0.45;
            w.vz += drive.impact * 0.15;
        }
        if drive.mode == FlopMode::Flying {
            // Airflow-driven flutter fades as the item slows; freefall does not
            // continually pump energy into an otherwise unforced spring.
            let air=(drive.speed/23.0).clamp(0.0,1.0);
            w.vx += (t * 11.0 + w.seed).sin() * dt * 5.0 * air;
            w.vz += (t * 9.0 + w.seed).cos() * dt * 3.0 * air;
        }
        let (stiffness, damping, limit) = if held {
            if chain.silicone { silicone_response(d) } else { held_response(d) }
        } else {
            (d.k, if drive.mode == FlopMode::Ground { d.c * 3.0 } else { d.c }, d.limit)
        };
        integrate_wobble(&mut w, stiffness, damping, limit, dt);
        if drive.mode==FlopMode::Ground {
            // The ground supports the chain: settle in the surface plane, not through it.
            w.z=0.0; w.vz=0.0;
            w.x=w.x.clamp(-0.22,0.22);
        }
        let dr = if held {
            (d.droop * 1.12 + 0.006 * (t * 2.1 + w.seed).sin() + 0.009 * (t * 0.55 + w.seed * 1.7).sin()) * if drive.mine { -0.6 } else { 1.0 }
        } else {
            0.0
        };
        for (i, e) in chain.segs.iter().enumerate() {
            if let Ok(mut tf) = segs.get_mut(*e) {
                let f = if chain.silicone { silicone_flex(i) } else { d.base + i as f32 * d.per_link };
                let extra = if i > 0 { dr } else { 0.0 };
                let mut bend = Vec2::new(w.x * f + extra, w.z * f + extra * 0.35);
                if chain.silicone {
                    // The grip stays put; successive sections take a little longer to follow.
                    // Skin weights blend these joint poses into a continuous surface.
                    let rate = 24.0 - i.min(6) as f32 * 2.5;
                    w.bends[i] = w.bends[i].lerp(bend, 1.0 - (-rate * dt).exp());
                    bend = w.bends[i];
                }
                tf.rotation = Quat::from_euler(
                    EulerRot::XYZ,
                    bend.x,
                    0.0,
                    bend.y,
                );
            }
        }
    }
}

fn silicone_response(d: bbq_core::looks::Floppy) -> (f32, f32, f32) {
    (d.k * 0.48, d.c * 0.80, 0.42)
}

/// A stable grip with increasingly compliant shaft sections.
fn silicone_flex(i: usize) -> f32 {
    [0.0, 0.16, 0.34, 0.54, 0.75, 0.94, 1.10][i.min(6)]
}

/// Held-only response; flight and ground springs keep their existing tuning.
fn held_response(d: bbq_core::looks::Floppy) -> (f32, f32, f32) {
    // Softer under the grip, with enough damping to settle after a comic rebound.
    // Foam gets a wider bend; the rubber retains a firmer, heavier response.
    (d.k * 0.68, d.c + 0.25, if d.k < 90.0 { 0.48 } else { 0.40 })
}

/// Continuous gait and comic start/stop recoil, without random idle kicks.
fn advance_held_drive(w: &mut Wobble, speed: f32, dt: f32, weight: f32) {
    let previous = w.held_speed;
    w.held_speed += (speed.clamp(0.0, 12.0) - previous) * (1.0 - (-10.0 * dt).exp());
    let amount = (w.held_speed / 7.0).min(1.2);
    if w.held_speed > 0.05 {
        w.phase = (w.phase + (6.0 + w.held_speed * 0.7) * dt) % std::f32::consts::TAU;
    }
    let variation = 0.94 + 0.12 * rand01(0, w.seed, 0);
    w.vx -= (w.held_speed - previous) * 0.32;
    w.vx += (w.phase * 2.0).cos() * amount * dt * 6.5 * weight * variation;
    w.vz += w.phase.sin() * amount * dt * 5.0 * variation;
}

fn integrate_wobble(w: &mut Wobble, stiffness: f32, damping: f32, limit: f32, dt: f32) {
    let n = (dt * 120.0).ceil().max(1.0) as u32;
    let hd = dt / n as f32;
    for _ in 0..n {
        w.vx += (-stiffness * w.x - damping * w.vx) * hd;
        w.vz += (-stiffness * w.z - damping * w.vz) * hd;
        w.x += w.vx * hd;
        w.z += w.vz * hd;
    }
    w.x = w.x.clamp(-limit, limit);
    w.z = w.z.clamp(-limit, limit);
    // Discard outward energy at the stop rather than accumulating it behind a clamp.
    if w.x.abs() >= limit && w.x * w.vx > 0.0 { w.vx = 0.0; }
    if w.z.abs() >= limit && w.z * w.vz > 0.0 { w.vz = 0.0; }
}

/// A repeatable pseudo-random number between 0 and 1 from a few numbers (the step, the item's
/// own seed, which question is being asked).
fn rand01(a: i32, seed: f32, k: u32) -> f32 {
    let mut x = (a as u32).wrapping_mul(0x9E37_79B1) ^ (seed.to_bits()).wrapping_mul(0x85EB_CA6B) ^ k.wrapping_mul(0xC2B2_AE35);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    x = x.wrapping_mul(0x297A_2D39);
    x ^= x >> 15;
    (x & 0xFFFF) as f32 / 65535.0
}

#[cfg(test)]
mod flop_tests {
    use super::*;

    #[test]
    fn large_rubber_items_tumble_slower_and_rest_preserves_heading() {
        assert!(launch_spin(ItemKind::Dildo,20.0,1.38)<launch_spin(ItemKind::Dildo,20.0,0.66));
        assert!(launch_spin(ItemKind::Noodle,20.0,1.0)<launch_spin(ItemKind::Dildo,20.0,1.0));
        for a in [0.0,0.5,1.2,2.5] {
            let q=Quat::from_euler(EulerRot::XYZ,a,0.4,0.8);
            let flat=lie_along(q,Quat::IDENTITY)*Vec3::Y;
            let previous=q*Vec3::Y;
            assert!(flat.y.abs()<1e-5);
            assert!(Vec2::new(flat.x,flat.z).normalize().dot(Vec2::new(previous.x,previous.z).normalize())>0.999);
        }
    }

    #[test]
    fn held_running_is_bounded_and_decays_after_stopping() {
        for kind in [ItemKind::Dildo, ItemKind::Noodle] {
            let d = bbq_core::looks::floppy(kind).unwrap();
            let mut peaks = Vec::new();
            for hz in [30, 60, 120] {
                let dt = 1.0 / hz as f32;
                let mut w = Wobble::default();
                let mut peak = 0.0_f32;
                for frame in 0..hz * 7 {
                    let speed = if frame < hz * 3 { 8.0 } else { 0.0 };
                    advance_held_drive(&mut w, speed, dt, d.step);
                    let (k, c, limit) = held_response(d);
                    integrate_wobble(&mut w, k, c, limit, dt);
                    assert!(w.x.is_finite() && w.z.is_finite());
                    peak = peak.max(w.x.abs()).max(w.z.abs());
                }
                assert!(peak > 0.02 && peak < 0.25, "{kind:?} {hz} Hz peak {peak}");
                assert!(w.x.abs() + w.z.abs() < 0.001);
                assert!(w.vx.abs() + w.vz.abs() < 0.01);
                peaks.push(peak);
            }
            assert!(peaks.iter().copied().fold(0.0_f32, f32::max)
                - peaks.iter().copied().fold(f32::INFINITY, f32::min) < 0.035);
        }
    }

    #[test]
    fn silicone_has_a_firm_grip_and_settles_at_multiple_frame_rates() {
        assert_eq!(silicone_flex(0), 0.0);
        assert!((1..7).all(|i| silicone_flex(i) > silicone_flex(i-1)));
        let d = bbq_core::looks::floppy(ItemKind::Dildo).unwrap();
        let (k,c,limit) = silicone_response(d);
        for hz in [30,60,120] {
            let dt=1.0/hz as f32; let mut w=Wobble::default();
            let mut peak=0.0_f32;
            for frame in 0..hz*8 {
                advance_held_drive(&mut w, if frame<hz*3 {8.0} else {0.0}, dt, d.step);
                integrate_wobble(&mut w,k,c,limit,dt);
                peak=peak.max(w.x.abs()).max(w.z.abs());
                assert!(w.x.is_finite() && w.z.is_finite());
            }
            assert!(peak>0.03 && peak<0.3,"{hz} Hz: {peak}");
            assert!(w.x.abs()+w.z.abs()<0.001);
            assert!(w.vx.abs()+w.vz.abs()<0.01);
        }
    }

    #[test]
    fn held_idle_has_no_random_kicks_and_limits_release_outward_energy() {
        let mut w = Wobble::default();
        for _ in 0..600 { advance_held_drive(&mut w, 0.0, 1.0/60.0, 1.8); }
        assert_eq!((w.vx,w.vz), (0.0,0.0));
        w.x = 0.5; w.vx = 20.0;
        integrate_wobble(&mut w, 65.0, 6.5, 0.38, 1.0/60.0);
        assert_eq!(w.x, 0.38);
        assert_eq!(w.vx, 0.0);
    }

    #[test]
    fn the_random_numbers_are_even_and_repeatable() {
        assert_eq!(rand01(7, 3.0, 1), rand01(7, 3.0, 1));
        assert_ne!(rand01(7, 3.0, 1), rand01(8, 3.0, 1));
        assert_ne!(rand01(7, 3.0, 1), rand01(7, 4.0, 1));
        let n = 4000;
        let mean: f32 = (0..n).map(|i| rand01(i, 1.5, 2)).sum::<f32>() / n as f32;
        assert!((mean - 0.5).abs() < 0.02, "{mean}");
        assert!((0..n).all(|i| (0.0..=1.0).contains(&rand01(i, 9.0, 0))));
    }
}
