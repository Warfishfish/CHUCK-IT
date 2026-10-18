//! Continuous Blender-built silicone surfaces, deformed by the existing seven-link rig.
//! Shared GPU meshes avoid per-frame vertex uploads or a soft-body solver.
use bevy::{asset::RenderAssetUsages, camera::visibility::DynamicSkinnedMeshBounds,
    mesh::{Indices, PrimitiveTopology, VertexAttributeValues, skinning::{SkinnedMesh, SkinnedMeshInverseBindposes}}, prelude::*};
use bbq_core::{items::{DildoVariant, ItemKind}, looks::DildoSkin};
use crate::models::{Built, FloppyChain, FloppySeg};

fn data() -> &'static serde_json::Value {
    static DATA: std::sync::OnceLock<serde_json::Value> = std::sync::OnceLock::new();
    DATA.get_or_init(|| serde_json::from_str(include_str!("../assets/models/silicone.json")).expect("validated silicone export"))
}

pub fn build(variant: DildoVariant, skin: DildoSkin, meshes: &mut Assets<Mesh>, mats: &mut Assets<StandardMaterial>) -> Built {
    let source = &data()[format!("{skin:?}")];
    let points = |name: &str| -> Vec<[f32;3]> { source[name].as_array().unwrap().iter().map(|v| std::array::from_fn(|i| v[i].as_f64().unwrap() as f32)).collect() };
    let scale = variant.def().scale;
    let original = points("positions");
    let mut joints = Vec::new(); let mut weights = Vec::new(); let mut colors = Vec::new();
    for p in &original {
        let u = ((p[1] + 0.145) / 0.155).clamp(0.0, 6.0);
        let lo = u.floor() as u16; let hi = (lo+1).min(6); let t = u - lo as f32;
        joints.push([lo, hi, 0, 0]); weights.push([1.0-t, t, 0.0, 0.0]);
        let tone = if p[1] < -0.07 { 0.84 } else { 1.0 };
        colors.push([tone,tone,tone,1.0]);
    }
    let positions: Vec<[f32;3]> = original.iter().map(|p| p.map(|v| v * scale)).collect();
    let uv: Vec<[f32;2]> = original.iter().map(|p| [p[2].atan2(p[0])/std::f32::consts::TAU+0.5,(p[1]+0.205)/1.325]).collect();
    let indices: Vec<u32> = source["indices"].as_array().unwrap().iter().map(|x| x.as_u64().unwrap() as u32).collect();
    let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, points("normals"))
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_INDEX, VertexAttributeValues::Uint16x4(joints))
        .with_inserted_attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT, weights)
        .with_inserted_indices(Indices::U32(indices))
        .with_generated_skinned_mesh_bounds().expect("valid seven-joint bounds");
    let color = crate::models::hex(bbq_core::looks::dildo_colour(variant));
    let emissive = if variant == DildoVariant::NeonLucky { color.to_linear() * 0.18 } else { LinearRgba::BLACK };
    Built { mesh: meshes.add(mesh), material: mats.add(StandardMaterial {
        base_color: color, perceptual_roughness: 0.56, reflectance: 0.30, metallic: 0.0, emissive, ..default()
    }), transform: Transform::default(), casts: true, receives: true, seg: None, outline: None, soft: true }
}

pub fn spawn(commands: &mut Commands, root: Entity, built: &Built, variant: DildoVariant) {
    let scale = variant.def().scale;
    let mut joints = Vec::new();
    let mut inverse = Vec::new();
    for i in 0..7 {
        let y = (-0.145 + i as f32 * 0.155) * scale;
        let rel = if i == 0 { y } else { 0.155 * scale };
        let e = commands.spawn((Transform::from_xyz(0.0,rel,0.0), Visibility::Inherited, FloppySeg)).id();
        commands.entity(if i == 0 { root } else { joints[i-1] }).add_child(e);
        joints.push(e); inverse.push(Mat4::from_translation(Vec3::new(0.0,-y,0.0)));
    }
    let body = commands.spawn((Mesh3d(built.mesh.clone()), MeshMaterial3d(built.material.clone()), Transform::default(), Visibility::Inherited, DynamicSkinnedMeshBounds)).id();
    commands.entity(root).add_child(body);
    commands.entity(root).insert(FloppyChain { segs: joints.clone(), def: bbq_core::looks::floppy(ItemKind::Dildo).unwrap(), silicone: true });
    // Only seven matrices per instance; the geometry/material stay cached across instances.
    commands.queue(move |world: &mut World| {
        let inverse_bindposes = world.resource_mut::<Assets<SkinnedMeshInverseBindposes>>().add(inverse);
        world.entity_mut(body).insert(SkinnedMesh { inverse_bindposes, joints });
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_silicone_skins_have_normalized_weights_finite_geometry_and_bounded_cost() {
        for skin in DildoSkin::ALL {
            for variant in [DildoVariant::Mini,DildoVariant::Classic,DildoVariant::Jumbo] {
                let mut meshes=Assets::<Mesh>::default(); let mut mats=Assets::<StandardMaterial>::default();
                let b=build(variant,skin,&mut meshes,&mut mats);let mesh=meshes.get(&b.mesh).unwrap();
                let VertexAttributeValues::Float32x3(p)=mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else {panic!()};
                assert!(p.len()<3000);
                assert!(p.iter().flatten().all(|v| v.is_finite()));
                assert!(p.iter().all(|v|v[0].abs()<=0.20*variant.def().scale && v[2].abs()<=0.20*variant.def().scale));
                let VertexAttributeValues::Float32x4(w)=mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT).unwrap() else {panic!()};
                assert!(w.iter().all(|w|(w.iter().sum::<f32>()-1.0).abs()<1e-6 && w.iter().all(|v|*v>=0.0)));
                let VertexAttributeValues::Uint16x4(j)=mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX).unwrap() else {panic!()};
                assert!(j.iter().flatten().all(|v|*v<7));
                let material=mats.get(&b.material).unwrap();assert!(material.perceptual_roughness>=0.5 && material.metallic==0.0);
            }
        }
    }
}
