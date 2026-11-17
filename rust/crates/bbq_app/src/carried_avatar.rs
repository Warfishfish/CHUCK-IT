//! A visual-only local blob for the outside carry camera. Never enters the simulation roster.
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use crate::{game::Game, menu::Settings, player::{Player,PLAYER_ID}};

pub struct CarriedAvatarPlugin;
impl Plugin for CarriedAvatarPlugin {
    fn build(&self,app:&mut App) {app.add_systems(Update,(sync_model,pose_avatar).chain());}
}
#[derive(Component)]
struct CarriedAvatar;
#[derive(Component)]
struct CarriedBelly;
fn sync_model(
    mut commands:Commands,assets:Res<AssetServer>,settings:Res<Settings>,
    old:Query<Entity,With<CarriedAvatar>>,mut shown:Local<Option<bbq_core::character::Character>>,
) {
    if *shown==Some(settings.character) {return;}
    *shown=Some(settings.character);
    for e in &old {commands.entity(e).despawn();}
    commands.spawn((
        WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(crate::style::model_path(settings.character)))),
        Transform::default(),Visibility::Hidden,CarriedAvatar,
    )).observe(|ready:On<WorldInstanceReady>,children:Query<&Children>,names:Query<&Name>,
        materials:Query<(),With<MeshMaterial3d<StandardMaterial>>>,mut commands:Commands| {
        // Reuse the local appearance resolver, without the menu's render layer or spinning rig.
        let owner=crate::face::FaceOwner::Preview;
        for node in children.iter_descendants(ready.entity) {
            let Ok(name)=names.get(node)else{continue;};
            let name=name.as_str();
            if name=="Torso" || name.starts_with("Singlet") || crate::face::shorts_kind(name).is_some() {
                commands.entity(node).insert(CarriedBelly);
            }
            if name=="Head" {commands.entity(node).insert(crate::face::HeadNode{owner,layer:Some(0)});}
            if let Some(kind)=crate::face::shorts_kind(name) {
                commands.entity(node).insert(crate::face::ShortsPiece{owner,kind});
            }
            let part=match name {
                "Torso"|"Belly"=>Some(crate::face::TintPart::Body),
                "Head"|"HandR"|"HandL"=>Some(crate::face::TintPart::Head),
                "FootL"|"FootR"=>Some(crate::face::TintPart::Foot),
                "ThongL"|"ThongR"=>Some(crate::face::TintPart::Thong),
                n if n.starts_with("Singlet")=>Some(crate::face::TintPart::Singlet),
                n if crate::face::shorts_kind(n).is_some()=>Some(crate::face::TintPart::Shorts),_=>None,
            };
            if let Some(part)=part {
                for e in std::iter::once(node).chain(children.iter_descendants(node)) {
                    if materials.contains(e) {commands.entity(e).insert(crate::face::LookTint{owner,part});}
                }
            }
        }
    });
}
fn pose_avatar(
    game:Res<Game>,player:Res<Player>,settings:Res<Settings>,
    mut avatar:Query<(&mut Transform,&mut Visibility),With<CarriedAvatar>>,
    mut bellies:Query<&mut bevy::mesh::morph::MorphWeights,With<CarriedBelly>>,
) {
    let carried=crate::smoko_mode::carried(&game,PLAYER_ID);
    let flying=crate::smoko_mode::flight_owner(&game,PLAYER_ID).is_some();
    let locked=carried || flying || (game.mirror && game.now<game.smoko.guest_until);
    for (mut tf,mut vis) in &mut avatar {
        *vis=if locked {Visibility::Visible}else{Visibility::Hidden};
        if !locked {continue;}
        let face=(-player.yaw.sin()).atan2(-player.yaw.cos());
        tf.translation=Vec3::new(player.mover.x,player.mover.y+0.32,player.mover.z);
        tf.rotation=Quat::from_euler(EulerRot::YXZ,face,-std::f32::consts::FRAC_PI_2,
            if flying {(game.now*5.0).sin()*0.18}else{(game.now*3.0).sin()*0.04});
    }
    for mut belly in &mut bellies {
        let weights=belly.weights_mut();
        if weights.len()>=2 {weights[0]=settings.belly.powf(0.8);weights[1]=(game.now*4.0).sin()*0.12;}
    }
}
