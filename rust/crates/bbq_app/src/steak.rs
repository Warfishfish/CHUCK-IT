//! Shared Blender ribeye. Five joints bend the same closed mesh in the hand and in flight.
use bbq_core::items::ItemKind;
use bevy::{
    asset::RenderAssetUsages,
    camera::visibility::DynamicSkinnedMeshBounds,
    mesh::{
        Indices, PrimitiveTopology, VertexAttributeValues,
        skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    },
    prelude::*,
};

use crate::models::{Built, FloppyChain, FloppySeg};

const GRIP_X: f32 = -0.16;
const JOINT_SPACING: f32 = 0.082;
const JOINTS: usize = 5;

pub fn build(meshes: &mut Assets<Mesh>, materials: &mut Assets<StandardMaterial>) -> Built {
    let script = include_str!("../../../../public/steak-model.js");
    let data: serde_json::Value = serde_json::from_str(
        script
            .trim()
            .strip_prefix("const BBQ_RIBEYE = ")
            .unwrap()
            .trim_end_matches(';'),
    )
    .expect("Blender ribeye mesh");
    let floats = |name: &str| {
        data[name]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap() as f32)
            .collect::<Vec<_>>()
    };
    let vec3s = |name: &str| {
        floats(name)
            .chunks_exact(3)
            .map(|row| [row[0], row[1], row[2]])
            .collect::<Vec<_>>()
    };
    let positions = vec3s("positions");
    let colours = floats("colours")
        .chunks_exact(4)
        .map(|row| [row[0], row[1], row[2], row[3]])
        .collect::<Vec<_>>();
    let mut joints = Vec::with_capacity(positions.len());
    let mut weights = Vec::with_capacity(positions.len());
    for [x, _, _] in &positions {
        let u = ((*x - GRIP_X) / JOINT_SPACING).clamp(0.0, (JOINTS - 1) as f32);
        let lo = u.floor() as u16;
        let hi = (lo + 1).min(JOINTS as u16 - 1);
        joints.push([lo, hi, 0, 0]);
        weights.push([1.0 - u.fract(), u.fract(), 0.0, 0.0]);
    }
    let indices = data["indices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i.as_u64().unwrap() as u32)
        .collect();
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec3s("normals"))
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colours)
    .with_inserted_attribute(
        Mesh::ATTRIBUTE_JOINT_INDEX,
        VertexAttributeValues::Uint16x4(joints),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights)
    .with_inserted_indices(Indices::U32(indices))
    .with_generated_skinned_mesh_bounds()
    .expect("valid ribeye joint bounds");
    Built {
        mesh: meshes.add(mesh),
        material: materials.add(StandardMaterial {
            perceptual_roughness: 0.52,
            reflectance: 0.27,
            ..default()
        }),
        transform: Transform::default(),
        casts: true,
        receives: true,
        seg: None,
        outline: None,
        soft: true,
    }
}

pub fn spawn(commands: &mut Commands, root: Entity, built: &Built) {
    let mut joints = Vec::with_capacity(JOINTS);
    let mut inverse = Vec::with_capacity(JOINTS);
    let mut parent = root;
    for i in 0..JOINTS {
        let x = GRIP_X + i as f32 * JOINT_SPACING;
        let relative = if i == 0 { GRIP_X } else { JOINT_SPACING };
        let joint = commands
            .spawn((
                Transform::from_xyz(relative, 0.0, 0.0),
                Visibility::Inherited,
                FloppySeg,
            ))
            .id();
        commands.entity(parent).add_child(joint);
        parent = joint;
        joints.push(joint);
        inverse.push(Mat4::from_translation(Vec3::new(-x, 0.0, 0.0)));
    }
    let body = commands
        .spawn((
            Mesh3d(built.mesh.clone()),
            MeshMaterial3d(built.material.clone()),
            Transform::default(),
            Visibility::Inherited,
            DynamicSkinnedMeshBounds,
        ))
        .id();
    commands.entity(root).add_child(body);
    commands.entity(root).insert(FloppyChain {
        segs: joints.clone(),
        def: bbq_core::looks::floppy(ItemKind::Steak).unwrap(),
        silicone: false,
    });
    commands.queue(move |world: &mut World| {
        let inverse_bindposes = world
            .resource_mut::<Assets<SkinnedMeshInverseBindposes>>()
            .add(inverse);
        world.entity_mut(body).insert(SkinnedMesh {
            inverse_bindposes,
            joints,
        });
    });
}
