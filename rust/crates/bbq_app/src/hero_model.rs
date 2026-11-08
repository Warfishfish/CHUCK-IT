//! Swap a procedural prop's visuals after its Blender scene and named joints arrive.

use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;

pub fn swap<const N: usize>(
    commands: &mut Commands,
    assets: &AssetServer,
    parent: Entity,
    path: &'static str,
    fallback: Vec<Entity>,
    joints: [&'static str; N],
    bind: impl Fn(&mut Commands, [(Entity, Transform); N]) + Send + Sync + 'static,
) {
    commands
        .spawn((
            WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(path))),
            Transform::default(),
            Visibility::Hidden,
            ChildOf(parent),
        ))
        .observe(move |ready: On<WorldInstanceReady>,
                       children: Query<&Children>,
                       nodes: Query<(&Name, &Transform)>,
                       meshes: Query<(), With<Mesh3d>>,
                       mut commands: Commands| {
            let mut found = [None; N];
            let mut has_mesh = false;
            for node in children.iter_descendants(ready.entity) {
                has_mesh |= meshes.contains(node);
                if let Ok((name, transform)) = nodes.get(node)
                    && let Some(index) = joints.iter().position(|&joint| joint == name.as_str())
                {
                    found[index] = Some((node, *transform));
                }
            }
            if !has_mesh || found.iter().any(Option::is_none) {
                warn!("Incomplete hero scene {path}. Keeping the procedural model.");
                commands.entity(ready.entity).despawn();
                return;
            }
            bind(&mut commands, found.map(Option::unwrap));
            commands.entity(ready.entity).insert(Visibility::Inherited);
            for &old in &fallback {
                commands.entity(old).despawn();
            }
        });
}
