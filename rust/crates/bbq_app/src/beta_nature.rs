//! Three-map beta scenery. Decorative roots remain outside playable footprints.
use bevy::prelude::*;
use bevy::world_serialization::WorldInstanceReady;
use bbq_core::GameMode;
#[derive(Component)]
struct NatureScope(GameMode);
#[derive(Component)]
struct Breeze { base: Quat, phase: f32, cloth: bool }
pub struct BetaNaturePlugin;
impl Plugin for BetaNaturePlugin {
    fn build(&self,app:&mut App) {app.add_systems(Startup,spawn).add_systems(Update,(visibility,breeze));}
}
// asset, x,z, scale, yaw. All trunks are beyond the arena fence/perimeter.
const BACKYARD: &[(&str,f32,f32,f32,f32)] = &[
    ("beta_tree_gum.glb",-14.0,33.5,1.1,0.3),
    ("beta_tree_citrus.glb",44.0,8.0,1.25,-0.7),
];
const SERVO: &[(&str,f32,f32,f32,f32)] = &[
    ("beta_tree_gum.glb",-44.0,-12.0,1.0,1.2),
    ("beta_tree_paperbark.glb",44.0,18.0,0.9,-0.4),
    ("beta_tree_gum.glb",-12.0,34.0,0.9,2.0),
    ("beta_windsock.glb",-44.0,-5.0,1.0,0.2),
];
const HEIST: &[(&str,f32,f32,f32,f32)] = &[
    ("beta_tree_paperbark.glb",-58.0,-14.0,1.0,0.8),
    ("beta_tree_gum.glb",-36.0,40.0,0.9,2.0),
    ("beta_tree_citrus.glb",10.0,45.0,1.2,0.2),
    ("beta_tree_paperbark.glb",57.0,17.0,1.0,-1.2),
    ("beta_tree_gum.glb",40.0,-35.0,0.95,0.8),
    ("beta_tree_paperbark.glb",-8.0,-45.0,0.85,1.4),
];
fn spawn(mut commands:Commands,assets:Res<AssetServer>) {
    for (mode,plants) in [(GameMode::FreeForAll,BACKYARD),(GameMode::Smoko,SERVO),(GameMode::Heist,HEIST)] {
        for &(asset,x,z,scale,yaw) in plants {
            root(&mut commands,&assets,mode,asset,Vec3::new(x,0.0,z),scale,yaw);
        }
    }
    root(&mut commands,&assets,GameMode::FreeForAll,"beta_kite.glb",Vec3::new(-12.3,7.1,33.6),0.85,0.3);
}
fn root(commands:&mut Commands,assets:&AssetServer,mode:GameMode,asset:&str,at:Vec3,scale:f32,yaw:f32) {
    let phase=at.x*0.11+at.z*0.17;
    commands.spawn((WorldAssetRoot(assets.load(GltfAssetLabel::Scene(0).from_asset(format!("models/{asset}")))),
        Transform::from_translation(at).with_scale(Vec3::splat(scale)).with_rotation(Quat::from_rotation_y(yaw)),
        Visibility::Hidden,NatureScope(mode),
    )).observe(move |ready:On<WorldInstanceReady>,children:Query<&Children>,nodes:Query<(&Name,&Transform)>,mut commands:Commands| {
        for e in children.iter_descendants(ready.entity) {
            let Ok((name,tf))=nodes.get(e)else{continue;};
            let cloth=name.starts_with("Breeze");
            if cloth||name.starts_with("Canopy") {commands.entity(e).insert(Breeze{base:tf.rotation,phase,cloth});}
        }
    });
}
fn visibility(screen:Res<crate::menu::Screen>,settings:Res<crate::menu::Settings>,game:Res<crate::game::Game>,mut roots:Query<(&NatureScope,&mut Visibility)>) {
    let mode=crate::menu::displayed_mode(*screen,&settings,&game);
    for (scope,mut vis) in &mut roots {
        let show=scope.0==mode || (scope.0==GameMode::FreeForAll && mode==GameMode::Teams);
        *vis=if show {Visibility::Inherited}else{Visibility::Hidden};
    }
}
fn breeze(time:Res<Time>,mut crowns:Query<(&Breeze,&mut Transform)>) {
    for (b,mut tf) in &mut crowns {
        let t=time.elapsed_secs();let wave=(t*0.7+b.phase).sin()*0.7+(t*1.3+b.phase*0.6).sin()*0.3;
        let amount=if b.cloth {0.035}else{0.009};
        tf.rotation=b.base*Quat::from_rotation_z(wave*amount)*Quat::from_rotation_x(wave*amount*0.45);
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn new_trunks_stay_outside_play_space() {
        for &(_,x,z,_,_) in BACKYARD {assert!(x.abs()>bbq_core::YARD_HALF_X+2.0 || z.abs()>bbq_core::YARD_HALF_Z+2.0);}
        for &(_,x,z,_,_) in SERVO {assert!(x.abs()>bbq_core::YARD_HALF_X+2.0 || z.abs()>bbq_core::YARD_HALF_Z+2.0);}
        for &(_,x,z,_,_) in HEIST {assert!(!bbq_core::yard::inside_heist(x,z,-2.0));}
    }
}
