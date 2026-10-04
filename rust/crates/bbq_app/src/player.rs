//! The player: reads the keyboard and mouse, runs `bbq_core::movement` at a fixed 60 Hz,
//! and puts the camera at the character's eyes with bob, shake, drunk-free FOV easing.

use bbq_core::movement::{self, FOV_DEFAULT, FOV_MAX, FOV_MIN, Modifiers, MoveInput, Mover};
use bbq_core::rng::Rng;
use bbq_core::sim::TICK_HZ;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::yard_scene::YardRes;

#[derive(Resource)]
pub struct Player {
    pub mover: Mover,
    /// Position at the previous fixed step, for smooth drawing between steps.
    pub prev: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub walk: f32,
    pub shake: f32,
    pub fov_base: f32,
    pub fov: f32,
    pub rng: Rng,
}

/// Input gathered each frame and consumed by the fixed step.
#[derive(Resource, Default)]
pub struct Wanted {
    pub wish: (f32, f32),
    /// Left mouse went down / came up since the last fixed step.
    pub throw_down: bool,
    pub throw_up: bool,
    pub catch: bool,
    /// +1 / -1 from Q, E or the mouse wheel.
    pub swap: i32,
    pub slot: Option<usize>,
    /// R went down since the last fixed step (grab a drink).
    pub interact_pressed: bool,
    /// R is being held right now (helping someone up).
    pub interact_held: bool,
    /// F went down / came up (grab, drag, throw someone who's down).
    pub grab_pressed: bool,
    pub grab_released: bool,
    /// Space went down (wriggle, or stand up from smoko).
    pub jump_pressed: bool,
    /// T, G or B was pressed.
    pub emote: Option<bbq_core::pose::Emote>,
}

#[derive(Component)]
pub struct EyeCamera;

#[derive(Component)]
pub struct HudText;

pub const PLAYER_ID: u32 = 1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        let spawn = bbq_core::yard::CHAR_SPAWNS[0];
        app.insert_resource(Time::<Fixed>::from_hz(TICK_HZ as f64))
            .insert_resource(Player {
                mover: Mover::new(spawn.0, spawn.1),
                prev: Vec3::new(spawn.0, 0.0, spawn.1),
                yaw: 0.0,
                pitch: 0.0,
                walk: 0.0,
                shake: 0.0,
                fov_base: FOV_DEFAULT,
                fov: FOV_DEFAULT,
                rng: Rng::new(0xBB9),
            })
            .init_resource::<Wanted>()
            .add_systems(Startup, setup_camera_and_hud)
            .add_systems(
                Update,
                (grab_mouse, read_input, update_camera, update_hud).chain(),
            )
            .add_systems(FixedUpdate, step_player.before(crate::game::step_game));
    }
}

fn setup_camera_and_hud(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: FOV_DEFAULT.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, movement::EYE_HEIGHT, 0.0),
        DistanceFog {
            color: Color::srgb(0.62, 0.8, 0.93),
            falloff: FogFalloff::Linear {
                start: 40.0,
                end: 140.0,
            },
            ..default()
        },
        crate::lighting::camera_style(),
        crate::lighting::hemisphere().0,
        EyeCamera,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(16.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(8.0),
            ..default()
        },
        HudText,
    ));
}

/// Click to grab the mouse, Esc to let go.
fn grab_mouse(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    wheel: Res<AccumulatedMouseScroll>,
    motion: Res<AccumulatedMouseMotion>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    mut player: ResMut<Player>,
    mut wanted: ResMut<Wanted>,
    game: Res<crate::game::Game>,
) {
    let grabbed = cursor.grab_mode != CursorGrabMode::None;
    if grabbed {
        let (y, p) = movement::look(player.yaw, player.pitch, motion.delta.x, motion.delta.y);
        player.yaw = y;
        player.pitch = p;
    }

    let axis = |pos: &[KeyCode], neg: &[KeyCode]| {
        let on = |ks: &[KeyCode]| ks.iter().any(|k| keys.pressed(*k));
        on(pos) as i32 as f32 - on(neg) as i32 as f32
    };
    let mx = axis(
        &[KeyCode::KeyD, KeyCode::ArrowRight],
        &[KeyCode::KeyA, KeyCode::ArrowLeft],
    );
    let my = axis(
        &[KeyCode::KeyW, KeyCode::ArrowUp],
        &[KeyCode::KeyS, KeyCode::ArrowDown],
    );
    wanted.wish = movement::wish_dir(player.yaw, mx, my);

    let stunned = game.me.body.stun > 0.0;
    let sitting = game.life.seated.is_some();
    if keys.just_pressed(KeyCode::Space) {
        wanted.jump_pressed = true;
        if !sitting {
            player.mover.try_jump(stunned);
        }
    }
    wanted.grab_pressed |= keys.just_pressed(KeyCode::KeyF);
    wanted.grab_released |= keys.just_released(KeyCode::KeyF);
    for (k, e) in [
        (KeyCode::KeyT, bbq_core::pose::Emote::Taunt),
        (KeyCode::KeyG, bbq_core::pose::Emote::Dance),
        (KeyCode::KeyB, bbq_core::pose::Emote::Laugh),
    ] {
        if keys.just_pressed(k) {
            wanted.emote = Some(e);
        }
    }
    if (keys.just_pressed(KeyCode::ShiftLeft) || keys.just_pressed(KeyCode::ShiftRight))
        && !sitting
        && game.life.carry.is_none()
    {
        player.mover.try_boost(&Modifiers {
            stunned,
            ..Default::default()
        });
    }
    wanted.interact_pressed |= keys.just_pressed(KeyCode::KeyR);
    wanted.interact_held = keys.pressed(KeyCode::KeyR);
    if grabbed {
        wanted.throw_down |= mouse.just_pressed(MouseButton::Left);
        wanted.catch |= mouse.just_pressed(MouseButton::Right);
    }
    wanted.throw_up |= mouse.just_released(MouseButton::Left);
    if keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::KeyE) {
        wanted.swap = 1;
    }
    if wheel.delta.y != 0.0 {
        wanted.swap = if wheel.delta.y > 0.0 { 1 } else { -1 };
    }
    if keys.just_pressed(KeyCode::Digit1) {
        wanted.slot = Some(0);
    }
    if keys.just_pressed(KeyCode::Digit2) {
        wanted.slot = Some(1);
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        player.fov_base = (player.fov_base - 5.0).max(FOV_MIN);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        player.fov_base = (player.fov_base + 5.0).min(FOV_MAX);
    }
}

fn step_player(
    mut player: ResMut<Player>,
    wanted: Res<Wanted>,
    yard: Res<YardRes>,
    mut game: ResMut<crate::game::Game>,
) {
    let p = &mut *player;
    let g = &mut *game;
    p.prev = Vec3::new(p.mover.x, p.mover.y, p.mover.z);
    // drunk: the walk drifts off line, and in Drunk mode the speed lurches
    let da = g.me.drunk.da();
    let wish = movement::drunk_steer(wanted.wish, da, g.now);
    let env = bbq_core::drunk_state::Env {
        drunk_mode: g.options.drunk_mode,
        ..Default::default()
    };
    let gait =
        g.me.drunk
            .gait(movement::step_dt(), g.now, &env, &mut g.rng);
    let mods = Modifiers {
        charging: g.wind.charging,
        stunned: g.me.body.stun > 0.0,
        drinking: g.me.drunk.is_drinking(),
        at_smoko: g.life.seated.is_some(),
        carrying: g.life.carry.is_some(),
        gait,
        ..Default::default()
    };
    let ev = p
        .mover
        .step(movement::step_dt(), MoveInput { wish }, &mods, &yard.0);
    p.shake = p.shake.max(ev.shake);
    p.walk = movement::advance_walk(
        p.walk,
        p.mover.speed(),
        p.mover.grounded,
        movement::step_dt(),
    );
}

pub fn update_camera(
    time: Res<Time>,
    fixed: Res<Time<Fixed>>,
    mut player: ResMut<Player>,
    game: Res<crate::game::Game>,
    mut cam: Single<(&mut Transform, &mut Projection), With<EyeCamera>>,
) {
    let dt = time.delta_secs();
    let p = &mut *player;

    // Smooth between the last two fixed steps.
    let now = Vec3::new(p.mover.x, p.mover.y, p.mover.z);
    let pos = p.prev.lerp(now, fixed.overstep_fraction());

    p.shake = (p.shake - dt * 0.5).max(0.0);
    let (sx, sy) = if p.shake > 0.0 {
        (
            p.rng.range(-1.0, 1.0) * p.shake,
            p.rng.range(-1.0, 1.0) * p.shake,
        )
    } else {
        (0.0, 0.0)
    };
    let bob = movement::head_bob(p.walk, p.mover.grounded);
    // drunk sway, and the drop to the ground when you've stacked it or been knocked flat
    let me = &game.me;
    let fx = bbq_core::drunk_state::camera_fx(
        me.drunk.da(),
        time.elapsed_secs(),
        me.drunk.fall_factor(&me.body),
        bbq_core::drunk_state::flat_amount(me.body.down_t, bbq_core::items::DOWN_TIME),
        p.pitch,
    );
    let eye = movement::eye_y(pos.y, p.mover.sink, bob) + sy * 0.3 + fx.dy;

    let (tf, proj) = &mut *cam;
    tf.translation = Vec3::new(pos.x + sx * 0.3, eye, pos.z);
    tf.rotation = Quat::from_euler(
        EulerRot::YXZ,
        p.yaw + sx * 0.2 + fx.yaw,
        p.pitch + sy * 0.2 + fx.pitch,
        fx.roll,
    );

    p.fov = movement::ease_fov(
        p.fov,
        p.fov_base,
        p.mover.sprinting(&Modifiers::default()),
        dt,
    );
    if let Projection::Perspective(persp) = &mut **proj {
        // drunk vision breathes in and out a little
        let breathe = me.drunk.da() * 3.0 * (time.elapsed_secs() * 1.3).sin();
        persp.fov = (p.fov + breathe).to_radians();
    }
}

fn update_hud(
    player: Res<Player>,
    game: Res<crate::game::Game>,
    cast: Res<crate::characters::Cast>,
    time: Res<Time>,
    mut text: Single<&mut Text, With<HudText>>,
) {
    let m = &player.mover;
    let boost = if m.boost_t > 0.0 {
        format!("BOOST {:.1}s", m.boost_t)
    } else if m.boost_cd > 0.0 {
        format!("boost in {:.1}s", m.boost_cd)
    } else {
        "boost ready".to_string()
    };
    let fps = 1.0 / time.delta_secs().max(0.0001);
    let me = game.board.get(PLAYER_ID).cloned().unwrap_or_default();
    let held: Vec<String> = game
        .slots
        .ids()
        .iter()
        .map(|id| {
            let name = game
                .world
                .items
                .get(id)
                .map_or("?", |i| crate::items_view::kind_name(i.kind));
            if Some(*id) == game.slots.selected() {
                format!("[{name}]")
            } else {
                name.to_string()
            }
        })
        .collect();
    let holding = if held.is_empty() {
        "nothing (walk over a glowing item)".to_string()
    } else {
        held.join(" ")
    };
    let charge = if game.wind.charging {
        let power = if game.wind.charge >= bbq_core::stun::POWER_CHARGE {
            " POWER"
        } else {
            ""
        };
        format!(" | charge {:.0}%{power}", game.wind.charge * 100.0)
    } else {
        String::new()
    };
    let feed: Vec<&str> = game
        .feed
        .iter()
        .rev()
        .take(5)
        .map(|(s, _)| s.as_str())
        .collect();
    let pool = if m.in_pool { "in pool | " } else { "" };
    let tramp = if m.tramp_chain {
        format!("tramp x{} | ", m.tramp_n + 1)
    } else {
        String::new()
    };
    text.0 = format!(
        "Click to grab mouse (Esc lets go) | WASD walk, Space jump, Shift boost, hold+release LMB throw (tap = slap), RMB catch, Q/E/wheel swap, R bar/meat/chest/smoko/help up, F grab someone who's down (tap put down, hold chuck), T taunt, G dance, B laugh\n\
         F1-F4 features, F5 falls, F6 Drunk mode, F7 Cheeky mode ({}), F8 Naughty Corner ({}), F9 character ({}) | viewer: J/K/L slapped, N stacked it, M emote, Y drunk, C crown, V sash, X stink, P +30 drunk\n\
         SCORE {} | hits {} | taken {} | catches {} | streak {} | holding: {holding}{charge}\n\
         pos {:.1}, {:.1}, {:.1} | speed {:.1} m/s | {boost} | FOV {:.0} | {pool}{tramp}{fps:.0} fps\n{}",
        if game.options.adult { "on" } else { "off" },
        if game.options.naughty { "on" } else { "off" },
        cast.mine.name(),
        me.score,
        me.hits,
        me.taken,
        me.catches,
        me.streak,
        m.x,
        m.y,
        m.z,
        m.speed(),
        player.fov_base,
        feed.join("\n")
    );
}
