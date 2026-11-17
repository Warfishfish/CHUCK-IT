//! Pooled wet droplets preserve faces and team colours and never affect collisions.
use bevy::prelude::*;
use bevy::light::NotShadowCaster;
use crate::game::Game;
pub struct WetPlugin;
impl Plugin for WetPlugin {
    fn build(&self, app: &mut App) { app.add_systems(Startup, setup).add_systems(Update, update); }
}
#[derive(Component)]
struct WetDrop { slot: usize, drop: usize, highlight: bool }
#[derive(Component)]
struct WetLabel;
fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let mut shape = Sphere::new(1.0).mesh().uv(10, 8);
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(vertices)) = shape.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        for p in vertices {let taper=0.75-p[1]*0.38;p[0]*=taper;p[2]*=taper;p[1]*=1.55;}
    }
    shape.compute_smooth_normals();
    let mesh = meshes.add(shape);
    let shine = materials.add(StandardMaterial {base_color: Color::srgb(0.83,0.97,1.0),unlit:true,..default()});
    let material = materials.add(StandardMaterial { base_color: Color::srgb(0.12, 0.56, 0.86), emissive: LinearRgba::rgb(0.02, 0.10, 0.15), perceptual_roughness: 0.10, metallic: 0.12, ..default() });
    for slot in 0..(crate::characters::MAX_BOTS + bbq_core::net::MAX_PLAYERS) {
        for drop in 0..14 {
            for highlight in [false,true] {
                commands.spawn((WetDrop { slot, drop, highlight }, Mesh3d(mesh.clone()), MeshMaterial3d(if highlight {shine.clone()}else{material.clone()}),Transform::default(),Visibility::Hidden,NotShadowCaster));
            }
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
        let angle = drop.drop as f32 * std::f32::consts::TAU / 14.0;
        let phase = (game.now * 0.85 + drop.drop as f32 * 0.173).fract();
        let radius = 0.46 + 0.12 * phase;
        tf.translation = Vec3::new(m.x + angle.cos() * radius, m.y + 1.65 - phase * phase * 1.5 - m.sink, m.z + angle.sin() * radius);
        let fade=wet.min(1.0);
        let size=(0.032+(drop.drop%3) as f32*0.008)*fade;
        tf.scale=Vec3::new(size,size*(1.0+phase*0.65),size);
        if drop.highlight {tf.translation+=Vec3::new(-size*0.3,size*0.4,size*0.6);tf.scale*=0.36;}
    }
    let review = crate::visual_review::is_water_review();
    for mut text in &mut label {
        text.0 = if playing && game.me.body.wet_t > 0.0 { if player.mover.in_pool {"SOAKED · drying starts on land".to_string()} else {format!("WET · 80% speed · drying {:.1}s", game.me.body.wet_t)} } else if review && let Some(d)=game.dummies.first() {
            format!("REVIEW TARGET · {} · {:.2} m/s · {:.1}s", if d.body.wet_t>0.0 {"WET — 80% speed"} else {"DRY — normal speed"}, d.mover.speed(), d.body.wet_t)
        } else { String::new() };
    }
}
