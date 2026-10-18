//! Pooled wet droplets preserve faces and team colours and never affect collisions.
use bevy::prelude::*;
use bevy::light::NotShadowCaster;
use crate::game::Game;
pub struct WetPlugin;
impl Plugin for WetPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Startup, setup).add_systems(Update, update); }
}
#[derive(Component)]
struct WetDrop { slot: usize, drop: usize }
#[derive(Component)]
struct WetLabel;
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mesh = meshes.add(Sphere::new(0.035).mesh().uv(6, 4));
    let material = materials.add(StandardMaterial { base_color: Color::srgb(0.23, 0.72, 0.98), emissive: LinearRgba::rgb(0.02, 0.10, 0.15), perceptual_roughness: 0.15, ..default() });
    for slot in 0..(crate::characters::MAX_BOTS + bbq_core::net::MAX_PLAYERS) {
        for drop in 0..8 {
            commands.spawn((WetDrop { slot, drop }, Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Hidden, NotShadowCaster));
        }
    }
    commands.spawn((WetLabel, Text::new(""), crate::ui::body_font(19.0, true), TextColor(Color::srgb(0.60, 0.91, 1.0)), TextShadow::default(), Node { position_type: PositionType::Absolute, left: percent(22.0), top: percent(74.0), ..default() }));
}
fn update(game: Res<Game>, player: Res<crate::player::Player>, screen: Res<crate::menu::Screen>, mut drops: Query<(&WetDrop, &mut Transform, &mut Visibility)>, mut label: Query<&mut Text, With<WetLabel>>) {
    let playing = matches!(*screen, crate::menu::Screen::Playing | crate::menu::Screen::Paused);
    for (drop, mut tf, mut vis) in &mut drops {
        let (m, wet) = if drop.slot == 0 { (&player.mover, game.me.body.wet_t) }
            else if let Some(d) = game.dummies.get(drop.slot - 1) { (&d.mover, d.body.wet_t) }
            else { *vis = Visibility::Hidden; continue; };
        *vis = if playing && wet > 0.0 { Visibility::Visible } else { Visibility::Hidden };
        if wet <= 0.0 {continue;}
        let angle = drop.drop as f32 * std::f32::consts::TAU / 8.0;
        let phase = (game.now * 1.2 + drop.drop as f32 * 0.173).fract();
        let radius = 0.48 + 0.08 * phase;
        tf.translation = Vec3::new(m.x + angle.cos() * radius, m.y + 1.5 - phase * 1.3 - m.sink, m.z + angle.sin() * radius);
        tf.scale = Vec3::new(1.0, 1.6 + phase, 1.0);
    }
    let review = crate::visual_review::is_water_review();
    for mut text in &mut label {
        text.0 = if playing && game.me.body.wet_t > 0.0 { format!("WET · 80% speed · {:.1}s", game.me.body.wet_t) } else if review && let Some(d)=game.dummies.first() {
            format!("REVIEW TARGET · {} · {:.2} m/s · {:.1}s", if d.body.wet_t>0.0 {"WET — 80% speed"} else {"DRY — normal speed"}, d.mover.speed(), d.body.wet_t)
        } else { String::new() };
    }
}
