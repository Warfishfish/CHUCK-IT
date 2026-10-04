//! Australian BBQ, Rust version. Phase 0: just prove the engine runs.
//! Opens a window with a sky, a lawn, a sun and a spinning purple cube.

use bevy::prelude::*;

// Yard half-sizes in metres, straight from BEHAVIOUR_SPEC.md section 1.
const YARD_HALF_X: f32 = 33.0;
const YARD_HALF_Z: f32 = 24.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Australian BBQ (Rust, Phase 0)".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.62, 0.85, 0.95)))
        .add_systems(Startup, setup)
        .add_systems(Update, spin)
        .run();
}

#[derive(Component)]
struct Spinner;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // The lawn.
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::default()
                    .mesh()
                    .size(YARD_HALF_X * 2.0, YARD_HALF_Z * 2.0),
            ),
        ),
        MeshMaterial3d(materials.add(Color::srgb(0.37, 0.61, 0.26))),
    ));

    // A purple cube so there is something to look at (stand-in for a mascot).
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.64, 0.30, 0.88))),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Spinner,
    ));

    // The sun.
    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(24.0, 40.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // A camera at roughly eye height, the same 1.55 m as the browser game.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 1.55, 8.0).looking_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y),
    ));
}

fn spin(time: Res<Time>, mut q: Query<&mut Transform, With<Spinner>>) {
    for mut t in &mut q {
        t.rotate_y(time.delta_secs());
    }
}
