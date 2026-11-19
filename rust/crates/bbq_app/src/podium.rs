//! Render the roster above the arena for a few seconds after the final whistle.
use crate::{
    carried_avatar::CarriedAvatar,
    characters::{Blob, BlobRoot},
    game::Game,
    player::{EyeCamera, PLAYER_ID},
};
use bevy::prelude::*;
pub struct PodiumPlugin;
impl Plugin for PodiumPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (stage, cheer).before(bevy::transform::TransformSystems::Propagate),
        );
    }
}
#[derive(Component)]
struct Stage;
fn place(rank: usize) -> Vec3 {
    let (x, y, z) = match rank {
        0 => (0., 1.65, 0.),
        1 => (-2.3, 1.05, 0.),
        2 => (2.3, 0.72, 0.),
        i => (
            ((i - 3) % 9) as f32 * 1.35 - 5.4,
            0.2 + ((i - 3) / 9) as f32 * 0.9,
            -2.5 - ((i - 3) / 9) as f32 * 1.5,
        ),
    };
    Vec3::new(x, y + 130., z)
}
fn stage(
    mut commands: Commands,
    g: Res<Game>,
    time: Res<Time>,
    settings: Res<crate::menu::Settings>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    old: Query<Entity, With<Stage>>,
    mut camera: Query<(&Camera, &mut Transform), (With<EyeCamera>, Without<BlobRoot>, Without<CarriedAvatar>)>,
    mut runners: Query<
        (Option<&Blob>, &mut Transform, &mut Visibility),
        (
            Or<(With<BlobRoot>, With<CarriedAvatar>)>,
            Without<EyeCamera>,
        ),
    >,
) {
    if !g.round.results.as_ref().is_some_and(|r| r.match_over)
        || !(0.0..7.0).contains(&g.round.panel_in)
        || g.round.panel_in == 0.0
    {
        for e in &old {
            commands.entity(e).despawn();
        }
        return;
    }
    let stage_camera =
        Transform::from_xyz(0., 134.4, 11.).looking_at(Vec3::new(0., 131.4, -0.8), Vec3::Y);
    if old.is_empty() {
        if let Ok((view, _)) = camera.single() {
            let transform = GlobalTransform::from(stage_camera);
            for (rank, id) in g.round.podium.iter().take(3).enumerate() {
                let Ok(anchor) = view.world_to_viewport(&transform, place(rank) + Vec3::Y * 2.9)
                else {
                    continue;
                };
                let name = if *id == PLAYER_ID {
                    settings.name.clone()
                } else {
                    crate::round::name_of(&g, *id)
                };
                commands.spawn((
                    Stage,
                    Text::new(name),
                    TextLayout::justify(Justify::Center),
                    TextFont { font_size: FontSize::Px(18.), ..default() },
                    TextColor(Color::WHITE),
                    BackgroundColor(Color::srgba(0.05, 0.11, 0.12, 0.94)),
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(anchor.x - 70.0),
                        top: px(anchor.y),
                        width: px(140.0),
                        padding: UiRect::vertical(px(4.0)),
                        border_radius: BorderRadius::all(px(8.0)),
                        ..default()
                    },
                ));
            }
        }

        commands
            .spawn((
                Stage,
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Percent(8.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    column_gap: px(12),
                    ..default()
                },
            ))
            .with_children(|ui| {
                for rank in [1usize, 0, 2] {
                    let Some(id) = g.round.podium.get(rank) else {
                        continue;
                    };
                    let (pts, hits) = g.round.totals[id];
                    ui.spawn((
                        Text::new(format!(
                            "#{}  {}\n{} points · {} hits",
                            rank + 1,
                            crate::round::name_of(&g, *id),
                            pts,
                            hits
                        )),
                        TextLayout::justify(Justify::Center),
                        TextFont {
                            font_size: FontSize::Px(20.),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.05, 0.11, 0.12, 0.94)),
                        Node {
                            padding: UiRect::all(px(9)),
                            ..default()
                        },
                    ));
                }
                ui.spawn((
                    Text::new("Space to skip"),
                    TextFont {
                        font_size: FontSize::Px(14.),
                        ..default()
                    },
                ));
            });
        commands
            .spawn((Stage, Transform::default(), Visibility::default()))
            .with_children(|c| {
                let board = mats.add(StandardMaterial::from(Color::srgb(0.12, 0.24, 0.24)));
                let gold = mats.add(StandardMaterial {
                    base_color: Color::srgb(1.0, 0.75, 0.14),
                    metallic: 0.6,
                    ..default()
                });
                c.spawn((
                    Mesh3d(meshes.add(Cuboid::new(16., 0.5, 10.))),
                    MeshMaterial3d(board.clone()),
                    Transform::from_xyz(0., 129.75, -2.),
                ));
                for (rank, _) in g.round.podium.iter().take(3).enumerate() {
                    let p = place(rank);
                    let h = p.y - 130.;
                    c.spawn((
                        Mesh3d(meshes.add(Cuboid::new(2.25, h, 1.85))),
                        MeshMaterial3d(board.clone()),
                        Transform::from_translation(p - Vec3::Y * h * 0.5),
                    ));
                }
                let p = place(0) + Vec3::new(0.75, 0., 0.3);
                for (mesh, y) in [
                    (meshes.add(Cylinder::new(0.17, 0.08)), 0.05),
                    (meshes.add(Cylinder::new(0.055, 0.25)), 0.19),
                    (
                        meshes.add(ConicalFrustum {
                            radius_top: 0.28,
                            radius_bottom: 0.09,
                            height: 0.35,
                        }),
                        0.48,
                    ),
                ] {
                    c.spawn((
                        Mesh3d(mesh),
                        MeshMaterial3d(gold.clone()),
                        Transform::from_translation(p + Vec3::Y * y),
                    ));
                }
            });
    }
    for (_, mut tf) in &mut camera {
        *tf = stage_camera;
    }
    for (blob, mut tf, mut vis) in &mut runners {
        let id = match blob {
            Some(b) => match g.dummies.get(b.0) {
                Some(d) => d.id,
                None => continue,
            },
            None => PLAYER_ID,
        };
        let Some(rank) = g.round.podium.iter().position(|v| *v == id) else {
            continue;
        };
        let beat = time.elapsed_secs() * (4. + (rank % 3) as f32) + rank as f32 * 1.3;
        let (bounce, tilt) = match rank % 4 {
            0 => (0.18, 0.13),
            1 => (0.07, 0.22),
            2 => (0.12, 0.08),
            _ => (0.035, 0.16),
        };
        *tf = Transform::from_translation(place(rank) + Vec3::Y * beat.sin().max(0.) * bounce)
            .with_rotation(Quat::from_rotation_z(tilt * beat.sin()));
        *vis = Visibility::Visible;
    }
}

fn cheer(
    g: Res<Game>,
    time: Res<Time>,
    mut hands: Query<
        (&Blob, Option<&crate::characters::HandL>, &mut Transform),
        Or<(
            With<crate::characters::HandL>,
            With<crate::characters::HandR>,
        )>,
    >,
) {
    if !g.round.results.as_ref().is_some_and(|r| r.match_over)
        || !(0.01..7.0).contains(&g.round.panel_in)
    {
        return;
    }
    for (blob, left, mut tf) in &mut hands {
        let Some(d) = g.dummies.get(blob.0) else {
            continue;
        };
        let Some(i) = g.round.podium.iter().position(|id| *id == d.id) else {
            continue;
        };
        let beat = (time.elapsed_secs() * 5.0 + i as f32 * 1.7).sin();
        let side = if left.is_some() { 1.0 } else { -1.0 };
        let (spread, height) = match i {
            0 => (0.55, 1.65 + beat * 0.15),
            1 => (0.42, 1.35 + side * beat * 0.27),
            2 => (0.16 + beat.abs() * 0.2, 1.1),
            _ => match i % 4 {
                0 => (0.5, 1.5 + beat * 0.16),
                1 => (0.12 + beat.abs() * 0.22, 1.0),
                2 => (0.18, 1.48 + beat * 0.07),
                _ => (0.38, 0.64 + beat * 0.09),
            },
        };
        tf.translation = Vec3::new(spread * side, height, 0.3);
    }
}
