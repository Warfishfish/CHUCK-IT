//! Local shelter/picnic bounce and ground contact. No gameplay or distant-light changes.
use crate::{game::Game, lighting::LookMode};
use bevy::asset::RenderAssetUsages;
use bevy::light::{NotShadowCaster, NotShadowReceiver};
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

// Small, shadowless bounce lights lift canopy shade without another shadow pass.
pub const BOUNCE_LUMENS: f32 = 18000.0;
pub const BOUNCE_RANGE: f32 = 7.0;
pub const CONTACT_ALPHA: f32 = 0.22;
pub const CHARACTER_CONTACT_ALPHA: f32 = 0.17;
#[derive(Component)]
struct DepthRoot;
#[derive(Component)]
struct ContactActor(usize);
#[derive(Resource)]
struct Kit {
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    actor: Handle<StandardMaterial>,
}
pub struct ArenaDepthPlugin;
impl Plugin for ArenaDepthPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync, actors).chain());
    }
}
fn contact_mesh() -> Mesh {
    let mut p = vec![[0., 0., 0.]];
    let mut c = vec![[1., 1., 1., 1.]];
    let mut ix = Vec::new();
    for ring in 1..=4 {
        let r = ring as f32 / 4.;
        for j in 0..32 {
            let a = j as f32 * std::f32::consts::TAU / 32.;
            p.push([r * a.cos(), 0., r * a.sin()]);
            c.push([1., 1., 1., (1. - r * r).powi(2)]);
        }
    }
    for j in 0..32 {
        ix.extend([0, 1 + (j + 1) % 32, 1 + j]);
    }
    for ring in 0..3 {
        for j in 0..32 {
            let a = 1 + ring * 32 + j;
            let b = 1 + ring * 32 + (j + 1) % 32;
            ix.extend([a, b, b + 32, a, b + 32, a + 32]);
        }
    }
    let n = vec![[0., 1., 0.]; p.len()];
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, p)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, n)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, c)
    .with_inserted_indices(Indices::U32(ix))
}
fn material(alpha: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::srgba(0.12, 0.095, 0.065, alpha),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        cull_mode: None,
        ..default()
    }
}
fn sync(
    mut commands: Commands,
    game: Res<Game>,
    screen: Res<crate::menu::Screen>, settings: Res<crate::menu::Settings>,
    look: Res<LookMode>,
    roots: Query<Entity, With<DepthRoot>>,
    kit: Option<Res<Kit>>,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
) {
    let active = crate::menu::displayed_mode(*screen, &settings, &game) == bbq_core::GameMode::Smoko
        && *look == LookMode::Polished
        && crate::style::current() == crate::style::Style::Pop;
    if !active {
        for e in &roots {
            commands.entity(e).despawn();
        }
        return;
    }
    if !roots.is_empty() {
        return;
    }
    let root = commands
        .spawn((DepthRoot, Transform::default(), Visibility::default()))
        .id();
    // The bake in picnic_canopy.py covers x [-12,16], z [-12,16].
    // Plane3d UV (0,0) is its +X,+Z corner in Bevy; set the UVs explicitly.
    let mut shade_plane = Plane3d::default().mesh().size(28., 28.).build();
    if let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) =
        shade_plane.attribute(Mesh::ATTRIBUTE_POSITION)
    {
        let uv: Vec<[f32; 2]> = points.iter().map(|p| [0.5 + p[0]/28., 0.5 + p[2]/28.]).collect();
        shade_plane.insert_attribute(Mesh::ATTRIBUTE_UV_0, uv);
    }
    commands.spawn((
        Mesh3d(meshes.add(shade_plane)),
        MeshMaterial3d(mats.add(StandardMaterial {
            base_color_texture: Some(assets.load("textures/smoko_picnic_shade.png")),
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            // Draw after intersecting ground wear and before the player's effects.
            depth_bias: 24.,
            ..default()
        })),
        Transform::from_xyz(2., 0.024, 2.),
        NotShadowCaster, NotShadowReceiver, ChildOf(root),
    ));
    let (mesh, mat) = if let Some(k) = kit {
        (k.mesh.clone(), k.material.clone())
    } else {
        let mesh = meshes.add(contact_mesh());
        let mat = mats.add(material(CONTACT_ALPHA));
        commands.insert_resource(Kit {
            mesh: mesh.clone(),
            material: mat.clone(),
            actor: mats.add(material(CHARACTER_CONTACT_ALPHA)),
        });
        (mesh, mat)
    };
    for (x, y, z) in [(0., 2.8, 10.), (8.5, 3., -2.5), (-8.5, 3., -2.5)] {
        commands.spawn((
            PointLight {
                color: Color::srgb(1., 0.91, 0.79),
                intensity: BOUNCE_LUMENS,
                range: BOUNCE_RANGE,
                radius: 2.,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(x, y, z),
            ChildOf(root),
        ));
    }
    let mut patches = vec![
        (8.5, -2.5, 1.55, 0.85),
        (-8.5, -2.5, 1.55, 0.85),
        (11., -1.5, 0.8, 0.5),
        (-11., -1.5, 0.8, 0.5),
        (5.6, 15.9, 0.8, 0.5),
    ];
    for x in [-4., 4.] {
        for z in [6., 14.] {
            patches.push((x, z, 0.45, 0.45));
        }
    }
    for (x, z, w, d) in patches {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat.clone()),
            Transform::from_xyz(x, 0.012, z).with_scale(Vec3::new(w, 1., d)),
            NotShadowCaster,
            NotShadowReceiver,
            ChildOf(root),
        ));
    }
}
fn in_slice(x: f32, z: f32) -> bool {
    (x.abs() < 6. && (4.5..16.).contains(&z))
        || ((x.abs() - 8.5).abs() < 4. && (-5.5..1.).contains(&z))
}
fn actors(
    mut commands: Commands,
    game: Res<Game>,
    root: Query<Entity, With<DepthRoot>>,
    kit: Option<Res<Kit>>,
    mut patches: Query<(Entity, &ContactActor, &mut Transform, &mut Visibility)>,
) {
    let (Some(root), Some(kit)) = (root.iter().next(), kit) else {
        return;
    };
    for (e, actor, mut t, mut v) in &mut patches {
        let Some(d) = game.dummies.get(actor.0) else {
            commands.entity(e).despawn();
            continue;
        };
        let m = &d.mover;
        // Do not paint a floating shadow on a table, carried body or airborne character.
        *v = if in_slice(m.x, m.z) && m.y.abs() < 0.12 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        t.translation = Vec3::new(m.x, 0.018, m.z);
    }
    for i in 0..game.dummies.len() {
        if patches.iter().any(|(_, a, _, _)| a.0 == i) {
            continue;
        }
        commands.spawn((
            ContactActor(i),
            Mesh3d(kit.mesh.clone()),
            MeshMaterial3d(kit.actor.clone()),
            Transform::from_scale(Vec3::new(0.65, 1., 0.5)),
            Visibility::Hidden,
            NotShadowCaster,
            NotShadowReceiver,
            ChildOf(root),
        ));
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contact_is_local_and_soft() {
        assert!(in_slice(0., 10.));
        assert!(in_slice(8.5, -2.5));
        for (x, z) in [(0., 30.), (25., 0.), (0., -20.)] {
            assert!(!in_slice(x, z));
        }
        let m = contact_mesh();
        assert_eq!(m.count_vertices(), 129);
        assert_eq!(m.indices().unwrap().len(), 672);
    }
}
