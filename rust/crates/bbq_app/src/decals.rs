//! Decals (step 2c): stains and splats painted onto whatever is underneath, using Bevy's forward
//! decals (a see-through box that projects its picture onto the surfaces inside it).
//!
//! Fixed ones sit about the yard (grease under the BBQ, a beer spill by the bar, a scorch mark,
//! bird droppings under the magpies' perches). Dynamic ones appear in play: a beer stain where a
//! VP can bursts, which fades away after a while.

use bevy::pbr::decal::{ForwardDecal, ForwardDecalMaterial, ForwardDecalMaterialExt};
use bevy::prelude::*;

use crate::game::Game;
use crate::lighting::LookMode;

pub struct DecalPlugin;

impl Plugin for DecalPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_decals)
            .add_systems(Update, (spawn_dynamic, fade_decals));
    }
}

#[derive(Resource)]
struct DecalAssets {
    stain: Handle<Image>,
    splat: Handle<Image>,
    scorch: Handle<Image>,
}

/// A decal that fades out and goes: seconds left, and how many it started with.
#[derive(Component)]
struct Fading {
    left: f32,
    total: f32,
    colour: Color,
    material: Handle<ForwardDecalMaterial<StandardMaterial>>,
}

fn decal_material(
    mats: &mut Assets<ForwardDecalMaterial<StandardMaterial>>,
    image: &Handle<Image>,
    colour: Color,
) -> Handle<ForwardDecalMaterial<StandardMaterial>> {
    mats.add(ForwardDecalMaterial {
        base: StandardMaterial {
            base_color: colour,
            base_color_texture: Some(image.clone()),
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.9,
            reflectance: 0.0,
            ..default()
        },
        extension: ForwardDecalMaterialExt {
            depth_fade_factor: 0.6,
        },
    })
}

/// Spawn one decal lying on the ground (or on a table top at height `y`): `size` metres across and
/// 1 m deep for the projection, turned by `turn`.
fn put(
    commands: &mut Commands,
    mats: &mut Assets<ForwardDecalMaterial<StandardMaterial>>,
    image: &Handle<Image>,
    colour: Color,
    at: Vec3,
    size: Vec2,
    turn: f32,
) -> (Entity, Handle<ForwardDecalMaterial<StandardMaterial>>) {
    let m = decal_material(mats, image, colour);
    let e = commands
        .spawn((
            ForwardDecal,
            MeshMaterial3d(m.clone()),
            Transform::from_translation(at)
                .with_rotation(Quat::from_rotation_y(turn))
                .with_scale(Vec3::new(size.x, 0.6, size.y)),
        ))
        .id();
    (e, m)
}

fn setup_decals(
    mut commands: Commands,
    assets: Res<AssetServer>,
    look: Res<LookMode>,
    mut mats: ResMut<Assets<ForwardDecalMaterial<StandardMaterial>>>,
) {
    let da = DecalAssets {
        stain: assets.load("textures/decal_stain.png"),
        splat: assets.load("textures/decal_splat.png"),
        scorch: assets.load("textures/decal_scorch.png"),
    };
    if *look == LookMode::Polished {
        let c = |r: f32, g: f32, b: f32, a: f32| Color::linear_rgba(r, g, b, a);
        // grease and ash round the BBQ, and a burnt patch at the front
        put(&mut commands, &mut mats, &da.stain, c(0.10, 0.07, 0.04, 0.55), Vec3::new(-6.0, 0.3, -17.6), Vec2::new(3.2, 2.2), 0.3);
        put(&mut commands, &mut mats, &da.scorch, c(0.03, 0.025, 0.02, 0.7), Vec3::new(-6.2, 0.3, -17.0), Vec2::new(1.4, 1.0), -0.4);
        // a beer spill by the bar and where the meat table drips
        put(&mut commands, &mut mats, &da.stain, c(0.55, 0.38, 0.1, 0.4), Vec3::new(1.4, 0.3, -19.4), Vec2::new(1.6, 1.3), 0.9);
        put(&mut commands, &mut mats, &da.stain, c(0.35, 0.05, 0.05, 0.35), Vec3::new(-8.7, 0.3, -17.3), Vec2::new(1.3, 1.0), 0.2);
        // a wet, dark edge round the pool where people climb out
        put(&mut commands, &mut mats, &da.stain, c(0.05, 0.08, 0.1, 0.4), Vec3::new(-20.0, 0.3, 5.4), Vec2::new(5.0, 1.8), 0.0);
        put(&mut commands, &mut mats, &da.stain, c(0.05, 0.08, 0.1, 0.35), Vec3::new(-12.6, 0.3, 10.0), Vec2::new(1.8, 4.5), 0.0);
        // bird droppings below the magpies' perches, and a few about the lawn
        for (x, z) in [(14.0f32, 23.2f32), (13.5, 23.5), (14.6, 23.0), (-32.3, 8.0), (-32.0, 8.5)] {
            put(&mut commands, &mut mats, &da.splat, c(0.93, 0.93, 0.88, 0.95), Vec3::new(x, 0.3, z), Vec2::new(0.5, 0.5), x * 3.0);
        }
        // grime where the paths meet the back door
        put(&mut commands, &mut mats, &da.stain, c(0.12, 0.09, 0.05, 0.4), Vec3::new(3.2, 0.3, -22.2), Vec2::new(2.6, 1.8), 0.6);
    }
    commands.insert_resource(da);
}

/// A beer stain where a VP can bursts. Kept to a few at a time, and each fades out.
fn spawn_dynamic(
    mut commands: Commands,
    mut game: ResMut<Game>,
    look: Res<LookMode>,
    assets: Option<Res<DecalAssets>>,
    mut mats: ResMut<Assets<ForwardDecalMaterial<StandardMaterial>>>,
    live: Query<Entity, With<Fading>>,
) {
    let Some(da) = assets else { return };
    if *look != LookMode::Polished {
        return;
    }
    let mut count = live.iter().count();
    let smashes: Vec<_> = game
        .fx
        .iter()
        .filter(|e| e.kind == crate::fx::FxKind::Smash)
        .map(|e| e.at)
        .collect();
    for at in smashes {
        if count >= 10 {
            break;
        }
        count += 1;
        let colour = Color::linear_rgba(0.6, 0.42, 0.1, 0.55);
        let (e, m) = put(
            &mut commands,
            &mut mats,
            &da.stain,
            colour,
            Vec3::new(at.x, 0.3, at.z),
            Vec2::splat(1.0 + (at.x * 7.0).sin().abs() * 0.5),
            at.x * 5.0,
        );
        commands.entity(e).insert(Fading { left: 25.0, total: 25.0, colour, material: m });
    }
    let _ = &mut game;
}

fn fade_decals(
    mut commands: Commands,
    time: Res<Time>,
    mut mats: ResMut<Assets<ForwardDecalMaterial<StandardMaterial>>>,
    mut q: Query<(Entity, &mut Fading)>,
) {
    for (e, mut f) in &mut q {
        f.left -= time.delta_secs();
        if f.left <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        // hold for most of the time, then fade over the last 6 s
        let k = (f.left / 6.0).clamp(0.0, 1.0);
        if let Some(mut m) = mats.get_mut(&f.material) {
            m.base.base_color = f.colour.with_alpha(f.colour.alpha() * k);
        }
        let _ = f.total;
    }
}
