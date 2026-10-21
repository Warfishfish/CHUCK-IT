//! Short, opaque lawn blades batched into spatial tiles. Visual only: no colliders,
//! per-blade entities, alpha overdraw, animation or shadow-casting grass passes.
use bbq_core::{GameMode, YARD_HALF_X, YARD_HALF_Z};
use bbq_core::rng::Rng;
use bbq_core::yard::{Features, Yard, POOL_X0, POOL_X1, POOL_Z0, POOL_Z1};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use crate::{game::Game, lighting::LookMode, player::EyeCamera, yard_scene::YardRes};

// Muted blade colours in linear space, matching the lawn beneath them.
const ROOT_COLOUR: Vec3 = Vec3::new(0.20, 0.37, 0.09);
const TIP_COLOUR: Vec3 = Vec3::new(0.31, 0.52, 0.15);
const TILE: f32 = 5.0;
const NEAR_TUFTS: usize = 1600;
const FAR_TUFTS: usize = 300;
const DETAIL_DISTANCE: f32 = 22.0;
const DRAW_DISTANCE: f32 = 38.0;
#[derive(Component)]
pub struct GrassTile {
    detailed: Handle<Mesh>,
    distant: Handle<Mesh>,
    high: bool,
}
#[derive(Default)]
pub struct GrassState {
    signature: Option<(GameMode, Features, usize)>,
    material: Option<Handle<StandardMaterial>>,
    enabled: Option<bool>,
}

fn on_lawn(x: f32, z: f32, mode: GameMode, yard: &Yard) -> bool {
    if mode == GameMode::Race {
        // Leave the racing lane and spectator terrace clear.
        if (x / 49.0).powi(2) + (z / 34.0).powi(2) > 1.0
            || crate::race::track_distance((x, z)) < crate::race::WIDTH * 0.5 + 0.25 {
            return false;
        }
        if z < -18.0 && (x / 44.0).powi(2) + (z / 29.5).powi(2) > 0.84 { return false; }
        return true;
    }
    if x.abs() > YARD_HALF_X - 0.5 || z.abs() > YARD_HALF_Z - 0.5 { return false; }
    if (POOL_X0 - 0.3..POOL_X1 + 0.3).contains(&x)
        && (POOL_Z0 - 0.3..POOL_Z1 + 0.3).contains(&z) { return false; }
    if mode == GameMode::Smoko
        && (x - crate::smoko_mode::CENTRE.0).hypot(z - crate::smoko_mode::CENTRE.1)
            < crate::smoko_mode::RADIUS + 0.65 { return false; }
    // No blades growing through furniture, steps, crates or fixed paving.
    if yard.colliders.iter().any(|c| x > c.x0 - 0.12 && x < c.x1 + 0.12
        && z > c.z0 - 0.12 && z < c.z1 + 0.12) { return false; }
    if (x - 3.0).abs() < 1.85 && (z + 22.9).abs() < 1.15 { return false; }
    true
}

fn tile_mesh(cx: f32, cz: f32, mode: GameMode, yard: &Yard, tufts: usize) -> Option<Mesh> {
    let seed = (cx as i64 as u64).wrapping_mul(73856093) ^ (cz as i64 as u64).wrapping_mul(19349663) ^ 0x6A455;
    let mut rng = Rng::new(seed);
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colours = Vec::new();
    let mut uv = Vec::new();
    let mut indices = Vec::new();
    for _ in 0..tufts {
        let x = rng.range(-TILE * 0.5, TILE * 0.5);
        let z = rng.range(-TILE * 0.5, TILE * 0.5);
        let wx = cx + x; let wz = cz + z;
        if !on_lawn(wx, wz, mode, yard) { continue; }
        // Slow variation breaks up the carpet; short height leaves pickups visible.
        let lush = 0.5 + 0.25 * (wx * 0.48).sin() + 0.25 * (wz * 0.63).cos();
        let h = rng.range(0.06, 0.125) * (0.8 + lush * 0.5);
        let tint = rng.range(0.86, 1.12);
        for _ in 0..3 {
            let angle = rng.range(0.0, std::f32::consts::TAU);
            let side = Vec3::new(angle.cos(), 0.0, angle.sin());
            let bend = Vec3::new(-angle.sin(), 0.0, angle.cos()) * rng.range(0.012, 0.045);
            let at = Vec3::new(x + rng.range(-0.09, 0.09), 0.008, z + rng.range(-0.09, 0.09));
            let width = rng.range(0.006, 0.013);
            let mid = at + Vec3::Y * h * 0.55 + bend * 0.35;
            let tip = at + Vec3::Y * h + bend;
            let first = positions.len() as u32;
            for (p, t) in [(at-side*width,0.0),(at+side*width,0.0),
                (mid-side*width*0.55,0.55),(mid+side*width*0.55,0.55),(tip,1.0)] {
                positions.push(p.to_array());
                // Upward shading normals blend with the lawn; no black back-facing stripes.
                normals.push([0.0, 1.0, 0.0]);
                let colour = ROOT_COLOUR.lerp(TIP_COLOUR, t) * tint;
                colours.push([colour.x, colour.y, colour.z, 1.0]);
                uv.push([0.0, t]);
            }
            indices.extend([first,first+2,first+1, first+1,first+2,first+3, first+2,first+4,first+3]);
        }
    }
    if positions.is_empty() { return None; }
    Some(Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_indices(Indices::U32(indices)))
}

pub fn sync(
    mut commands: Commands, g: Res<Game>, yard: Res<YardRes>, look: Res<LookMode>,
    mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>,
    tiles: Query<Entity, With<GrassTile>>, mut state: Local<GrassState>,
) {
    let enabled = *state.enabled.get_or_insert_with(|| !std::env::args().any(|a| a == "--grass-off"));
    if !enabled || *look != LookMode::Polished || crate::style::current() != crate::style::Style::Pop { return; }
    let signature = (g.rules.mode, yard.0.features, yard.0.colliders.len());
    if state.signature == Some(signature) { return; }
    state.signature = Some(signature);
    for entity in &tiles { commands.entity(entity).despawn(); }
    let material = state.material.get_or_insert_with(|| materials.add(StandardMaterial {
        base_color: Color::WHITE, perceptual_roughness: 1.0, reflectance: 0.0,
        cull_mode: None, ..default()
    })).clone();
    let (nx, nz) = if g.rules.mode == GameMode::Race { (10, 7) } else { (7, 5) };
    for ix in -nx..=nx { for iz in -nz..=nz {
        let (x, z) = (ix as f32 * TILE, iz as f32 * TILE);
        if let Some(mesh) = tile_mesh(x, z, g.rules.mode, &yard.0, NEAR_TUFTS) {
            let detailed = meshes.add(mesh);
            let distant = tile_mesh(x, z, g.rules.mode, &yard.0, FAR_TUFTS).map(|m| meshes.add(m)).unwrap_or_else(|| detailed.clone());
            commands.spawn((GrassTile { detailed: detailed.clone(), distant, high: true }, Mesh3d(detailed), MeshMaterial3d(material.clone()),
                Transform::from_xyz(x, 0.0, z), bevy::light::NotShadowCaster));
        }
    }}
}

pub fn distance_cull(
    camera: Query<&Transform, (With<EyeCamera>, Without<GrassTile>)>,
    mut tiles: Query<(&Transform, &mut Visibility, &mut Mesh3d, &mut GrassTile)>,
) {
    let Ok(eye) = camera.single() else { return; };
    for (at, mut visibility, mut mesh, mut tile) in &mut tiles {
        // Horizontal distance also keeps overview/review cameras useful.
        let delta = at.translation - eye.translation;
        let distance_sq = delta.x * delta.x + delta.z * delta.z;
        let high = distance_sq < DETAIL_DISTANCE * DETAIL_DISTANCE;
        if high != tile.high {
            mesh.0 = if high { tile.detailed.clone() } else { tile.distant.clone() };
            tile.high = high;
        }
        *visibility = if distance_sq < DRAW_DISTANCE * DRAW_DISTANCE {
            Visibility::Inherited
        } else { Visibility::Hidden };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grass_keeps_objectives_and_racing_lane_clear() {
        let yard = Yard::default();
        assert!(!on_lawn(-20.0, 10.0, GameMode::FreeForAll, &yard));
        assert!(!on_lawn(0.0, 10.0, GameMode::Smoko, &yard));
        for gate in crate::race::TRACK { assert!(!on_lawn(gate.0, gate.1, GameMode::Race, &yard)); }
        assert!(!on_lawn(0.0, -30.0, GameMode::Race, &yard));
        assert!(on_lawn(0.0, 0.0, GameMode::Race, &yard));
    }
}
