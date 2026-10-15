//! Compact Blender-built surfaces shared by every Pop character, preview and lobby face.
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::collections::HashMap;
use crate::face::{Colour, Mood};
use bbq_core::appearance::{Brows, Mouth};

const DATA: &str = include_str!("../assets/models/face_pop.json");

pub fn load(meshes: &mut Assets<Mesh>) -> HashMap<String, Handle<Mesh>> {
    let data: serde_json::Value = serde_json::from_str(DATA).expect("validated embedded face meshes");
    data.as_object().unwrap().iter().map(|(name, data)| {
        let points = |key: &str| -> Vec<[f32; 3]> {
            data[key].as_array().unwrap().iter().map(|p| std::array::from_fn(|i| p[i].as_f64().unwrap() as f32)).collect()
        };
        let indices = data["indices"].as_array().unwrap().iter().map(|i| i.as_u64().unwrap() as u32).collect();
        let positions = points("positions");
        let uvs: Vec<[f32; 2]> = positions.iter().map(|p| [p[0]+0.5, 0.5-p[1]]).collect();
        let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
            .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, points("normals"))
            .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
            .with_inserted_indices(Indices::U32(indices));
        (name.clone(), meshes.add(mesh))
    }).collect()
}

pub fn mouth(mouth: Mouth, mood: Mood) -> Vec<(&'static str, Colour)> {
    if mouth == Mouth::None { return vec![]; }
    use Colour::*;
    match mood {
        Mood::Ouch => vec![("ouch_dark",Dark),("ouch_tongue",Tongue)],
        Mood::Sip => vec![("sip_dark",Dark)],
        Mood::Grit => vec![("grit_dark",Dark),("grit_teeth",Teeth)],
        Mood::Cheer => vec![("grin_dark",Dark),("grin_teeth",Teeth),("grin_tongue",Tongue)],
        Mood::Normal => match mouth {
            Mouth::Smile => vec![("smile_dark",Dark)],
            Mouth::Smirk => vec![("smirk_dark",Dark)],
            Mouth::Grin => self::mouth(mouth,Mood::Cheer),
            Mouth::None => vec![],
        }
    }
}

pub fn brow(brows: Brows) -> Option<&'static str> {
    match brows { Brows::None => None, Brows::Flat => Some("brow_flat"), Brows::Arched => Some("brow_arched"), Brows::Bushy => Some("brow_bushy") }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbq_core::appearance::Choice;
    #[test]
    fn exported_faces_are_complete_finite_and_within_the_head() {
        let mut meshes = Assets::<Mesh>::default();
        let kit = load(&mut meshes);
        for mood in [Mood::Normal,Mood::Grit,Mood::Ouch,Mood::Sip,Mood::Cheer] {
            assert!(mouth(Mouth::None,mood).is_empty());
            for m in Mouth::ALL { for (name,_) in mouth(*m,mood) { assert!(kit.contains_key(name)); } }
        }
        assert!(brow(Brows::None).is_none());
        for b in Brows::ALL { if let Some(name) = brow(*b) { assert!(kit.contains_key(name)); } }
        for (_,handle) in kit {
            let mesh = meshes.get(&handle).unwrap();
            let bevy::mesh::VertexAttributeValues::Float32x3(p) = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else { panic!() };
            assert!(p.iter().all(|p| p.iter().all(|v| v.is_finite()) && p[0].abs()<0.4 && p[1].abs()<0.26 && p[2].abs()<0.15));
            assert!(mesh.indices().unwrap().iter().all(|i| i<p.len()));
        }
    }
}
