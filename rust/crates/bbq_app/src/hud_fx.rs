//! On-screen extras for Phase 5: the drunk meter, the big fading pop-ups ("+18 drunk",
//! "STACKED IT!"), the line that says what R does, the drunk tint, and the drink in your hand.
//!
//! The browser game blurs and wobbles the whole picture when you're drunk with a shader. Here
//! that is simplified: the camera sways and the view breathes in and out (see `player.rs`),
//! and a warm tint pulses over the screen. A true wobble shader can come in the polish phase.

use bbq_core::drinks::{Drink, Tier};
use bevy::prelude::*;
use bevy::text::{Justify, TextLayout};

use crate::game::Game;
use crate::player::EyeCamera;

pub struct HudFxPlugin;

impl Plugin for HudFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud_fx).add_systems(
            Update,
            (
                update_popup,
                update_prompt,
                update_meter,
                update_tint,
                sync_drink_vm,
            ),
        );
    }
}

#[derive(Component)]
struct PopupText;
#[derive(Component)]
struct PromptText;
#[derive(Component)]
struct MeterBox;
#[derive(Component)]
struct MeterFill;
#[derive(Component)]
struct MeterLabel;
#[derive(Component)]
struct Tint;
#[derive(Component)]
struct DrinkVm(Drink);

fn spawn_hud_fx(mut commands: Commands) {
    // a warm tint over everything that pulses when you're drunk
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.9, 0.6, 0.3, 0.0)),
        GlobalZIndex(-1),
        Tint,
    ));
    // big pop-up text, a bit above the middle
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(34.0),
            ..default()
        },
        TextColor(Color::srgba(1.0, 0.95, 0.4, 0.0)),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Percent(34.0),
            ..default()
        },
        PopupText,
    ));
    // "R: grab a cold VP" and friends
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::justify(Justify::Center),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            bottom: Val::Px(150.0),
            ..default()
        },
        PromptText,
    ));
    // drunk meter, bottom left
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(14.0),
                bottom: Val::Px(14.0),
                width: Val::Px(220.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                ..default()
            },
            Visibility::Hidden,
            MeterBox,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(15.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                MeterLabel,
            ));
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(14.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ))
            .with_children(|b| {
                b.spawn((
                    Node {
                        width: Val::Percent(0.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.9, 0.7, 0.2)),
                    MeterFill,
                ));
            });
        });
}

fn update_popup(
    game: Res<Game>,
    mut q: Query<(&mut Text, &mut TextColor, &mut TextFont), With<PopupText>>,
) {
    let Ok((mut text, mut col, mut font)) = q.single_mut() else {
        return;
    };
    let Some(p) = game.popups.last() else {
        col.0 = col.0.with_alpha(0.0);
        return;
    };
    if text.0 != p.text {
        text.0 = p.text.clone();
    }
    font.font_size = FontSize::Px(if p.big { 44.0 } else { 26.0 });
    // fade in fast, hold, fade out over the last half second
    let a = (p.t / 0.1)
        .min(1.0)
        .min(((1.6 - p.t) / 0.5).clamp(0.0, 1.0));
    let c = if p.big {
        Color::srgb(1.0, 0.85, 0.25)
    } else {
        Color::srgb(1.0, 0.97, 0.7)
    };
    col.0 = c.with_alpha(a);
}

fn update_prompt(game: Res<Game>, mut q: Query<&mut Text, With<PromptText>>) {
    if let Ok(mut t) = q.single_mut()
        && t.0 != game.prompt
    {
        t.0 = game.prompt.clone();
    }
}

fn tier_label(t: Tier) -> &'static str {
    match t {
        Tier::Sober => "Sober",
        Tier::Tipsy => "Tipsy",
        Tier::Drunk => "Drunk",
        Tier::Maggot => "Maggot",
        Tier::AbsolutelyMaggoted => "Absolutely maggoted",
    }
}

fn update_meter(
    game: Res<Game>,
    mut boxq: Query<&mut Visibility, With<MeterBox>>,
    mut label: Query<&mut Text, With<MeterLabel>>,
    mut fill: Query<(&mut Node, &mut BackgroundColor), With<MeterFill>>,
) {
    let d = &game.me.drunk;
    if let Ok(mut v) = boxq.single_mut() {
        *v = if d.meter > 0.5 || d.is_drinking() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok(mut t) = label.single_mut() {
        let s = format!("{} ({:.0})", tier_label(d.tier()), d.meter);
        if t.0 != s {
            t.0 = s;
        }
    }
    if let Ok((mut n, mut bg)) = fill.single_mut() {
        n.width = Val::Percent(d.meter.clamp(0.0, 100.0));
        let x = (d.meter / 100.0).clamp(0.0, 1.0);
        bg.0 = Color::srgb(0.4 + 0.6 * x, 0.85 - 0.6 * x, 0.25);
    }
}

fn update_tint(game: Res<Game>, time: Res<Time>, mut q: Query<&mut BackgroundColor, With<Tint>>) {
    let da = game.me.drunk.da();
    let pulse = 0.65 + 0.35 * (time.elapsed_secs() * 1.4).sin();
    if let Ok(mut bg) = q.single_mut() {
        bg.0 = Color::srgba(0.9, 0.55, 0.3, 0.16 * da * pulse);
    }
}

/// The drink in your hand while you drink it: rises to your mouth and tips back.
fn sync_drink_vm(
    mut commands: Commands,
    game: Res<Game>,
    cam: Single<Entity, With<EyeCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut vms: Query<(&DrinkVm, &mut Transform, &mut Visibility)>,
    mut spawned: Local<bool>,
) {
    if !*spawned {
        *spawned = true;
        let glass = mats.add(StandardMaterial {
            base_color: Color::linear_rgba(0.87, 0.91, 0.92, 0.5),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let amber = mats.add(Color::linear_rgb(0.85, 0.6, 0.1));
        let wine = mats.add(Color::linear_rgb(0.54, 0.11, 0.23));
        let rum = mats.add(StandardMaterial {
            base_color: Color::linear_rgba(0.9, 0.75, 0.48, 0.9),
            alpha_mode: AlphaMode::Blend,
            ..default()
        });
        let mut part = |commands: &mut Commands,
                        parent: Entity,
                        mesh: Mesh,
                        mat: &Handle<StandardMaterial>,
                        y: f32| {
            commands
                .spawn((
                    Mesh3d(meshes.add(mesh)),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_xyz(0.0, y, 0.0),
                    ChildOf(parent),
                ))
                .id()
        };
        let make = |commands: &mut Commands, d: Drink| {
            commands
                .spawn((
                    Transform::default(),
                    Visibility::Hidden,
                    DrinkVm(d),
                    ChildOf(*cam),
                ))
                .id()
        };
        let beer = make(&mut commands, Drink::Beer);
        part(
            &mut commands,
            beer,
            Cylinder::new(0.036, 0.19).into(),
            &amber,
            0.0,
        );
        let w = make(&mut commands, Drink::Wine);
        part(
            &mut commands,
            w,
            Cylinder::new(0.008, 0.09).into(),
            &glass,
            -0.06,
        );
        part(
            &mut commands,
            w,
            Cylinder::new(0.03, 0.1).into(),
            &glass,
            0.03,
        );
        part(
            &mut commands,
            w,
            Cylinder::new(0.027, 0.06).into(),
            &wine,
            0.01,
        );
        part(
            &mut commands,
            w,
            Cylinder::new(0.04, 0.008).into(),
            &glass,
            -0.105,
        );
        let r = make(&mut commands, Drink::Rum);
        part(
            &mut commands,
            r,
            Cylinder::new(0.032, 0.07).into(),
            &rum,
            0.0,
        );
        return;
    }
    let current = game.me.drunk.drinking;
    for (vm, mut tf, mut vis) in &mut vms {
        match current {
            Some(d) if d.drink == vm.0 => {
                *vis = Visibility::Inherited;
                let p = d.progress();
                let up = (p * 4.0).min(1.0);
                let tip = ((p * 1.15).min(1.0) * std::f32::consts::PI).sin();
                tf.translation = Vec3::new(0.12 - up * 0.1, -0.32 + up * 0.2, -0.5 + up * 0.12);
                tf.rotation = Quat::from_euler(EulerRot::XYZ, up * 0.35 + tip * 0.9, 0.0, -0.2);
            }
            _ => *vis = Visibility::Hidden,
        }
    }
}
