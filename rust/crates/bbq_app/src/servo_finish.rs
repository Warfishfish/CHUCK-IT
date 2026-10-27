//! Bush Servo Stage 5: scoped lighting and filtering for imported environment maps.
//! Changes only presentation; restore the exact previous camera/lights on leaving Smoko.
use bevy::prelude::*;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::render::render_resource::TextureFormat;
use crate::lighting::{self, DisplayRaw, Grade, LookMode, Rig};

// Artistic tuning. Keep shade open and the key warm without clipping player colours.
pub const SERVO_RIG: Rig = Rig {
    sky: 0xc2d7ec, ground: 0xb7b9a2, hemi: 1.00,
    sun: 0xffddb0, sun_intensity: 0.86,
    grade: Grade { saturation: 0.98, warmth: 0.003, contrast: 0.995, vignette: 0.025 },
};
pub const SUN_POSITION: Vec3 = Vec3::new(24.0, 24.0, 16.0);
pub const DISTANT_HAZE: u32 = 0xc9cfd0;
pub const TEXTURE_ANISOTROPY: u16 = 8;

#[derive(Component, Clone, Copy)]
pub enum YardLight { Sun, Fill }
#[derive(Resource, Default)]
struct SavedLook {
    active: bool,
    lights: Vec<(Entity, DirectionalLight, Transform)>,
    cameras: Vec<(Entity, AmbientLight, DistanceFog, DisplayRaw)>,
}
pub struct ServoFinishPlugin;
impl Plugin for ServoFinishPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SavedLook>()
            .add_systems(Update, (sync_lighting, filter_imported_textures));
    }
}
fn sync_lighting(
    game: Res<crate::game::Game>, look: Res<LookMode>, mut saved: ResMut<SavedLook>,
    mut lights: Query<(Entity, &YardLight, &mut DirectionalLight, &mut Transform)>,
    mut cameras: Query<(Entity, &mut AmbientLight, &mut DistanceFog, &mut DisplayRaw), With<crate::player::EyeCamera>>,
) {
    let active=game.rules.mode==bbq_core::GameMode::Smoko && *look==LookMode::Polished
        && crate::style::current()==crate::style::Style::Pop;
    if active==saved.active {return;}
    if active {
        let (ambient, fill, _)=lighting::hemisphere_from_rig(SERVO_RIG);
        for (id,kind,mut light,mut transform) in &mut lights {
            saved.lights.push((id,light.clone(),*transform));
            match kind {
                YardLight::Sun=>{
                    light.color=crate::models::hex(SERVO_RIG.sun);
                    light.illuminance=lighting::lux(SERVO_RIG.sun_intensity)*lighting::SUN_FIX;
                    *transform=Transform::from_translation(SUN_POSITION).looking_at(Vec3::ZERO,Vec3::Y);
                }
                YardLight::Fill=>{*light=fill.clone();}
            }
        }
        for (id,mut amb,mut fog,mut grade) in &mut cameras {
            saved.cameras.push((id,amb.clone(),fog.clone(),*grade));
            *amb=ambient.clone();fog.color=crate::models::hex(DISTANT_HAZE);
            *grade=DisplayRaw::with_grade(SERVO_RIG.grade);
        }
    } else {
        for (id,old_light,old_transform) in saved.lights.drain(..) {
            if let Ok((_,_,mut light,mut transform))=lights.get_mut(id) {*light=old_light;*transform=old_transform;}
        }
        for (id,old_ambient,old_fog,old_grade) in saved.cameras.drain(..) {
            if let Ok((_,mut amb,mut fog,mut grade))=cameras.get_mut(id) {*amb=old_ambient;*fog=old_fog;*grade=old_grade;}
        }
    }
    saved.active=active;
}

fn is_servo_asset(path: &std::path::Path) -> bool {
    matches!(path.file_name().and_then(|n|n.to_str()),
        Some("smoko_servo.glb" | "smoko_shelter.glb" | "smoko_surroundings.glb" | "smoko_landscape.glb"))
}
fn filter_imported_textures(
    mut events: MessageReader<AssetEvent<Image>>, assets: Res<AssetServer>, mut images: ResMut<Assets<Image>>,
) {
    for event in events.read() {
        // Embedded glTF images emit Added; standalone PNGs use LoadedWithDependencies.
        let id=match event {AssetEvent::Added{id}|AssetEvent::LoadedWithDependencies{id}=>*id,_=>continue};
        if !assets.get_path(id).is_some_and(|p|is_servo_asset(p.path())) {continue;}
        let Some(mut image)=images.get_mut(id) else {continue;};
        let (w,h)=(image.width(),image.height());
        let srgb=image.texture_descriptor.format==TextureFormat::Rgba8UnormSrgb;
        if image.texture_descriptor.mip_level_count>1 || (!srgb && image.texture_descriptor.format!=TextureFormat::Rgba8Unorm) {continue;}
        let Some(data)=image.data.as_ref() else {continue;};
        if data.len()!=(w*h*4) as usize || (w==1 && h==1) {continue;}
        // Colour maps are averaged in linear light; normals/roughness retain data-space values.
        let levels=if srgb {colour_mips(w,h,data)} else {crate::models::build_mips(w,h,data)};
        image.data=Some(levels);image.texture_descriptor.mip_level_count=crate::models::mip_count(w,h);
        let mut sampler=match &image.sampler {ImageSampler::Descriptor(s)=>s.clone(),_=>ImageSamplerDescriptor::linear()};
        sampler.mag_filter=ImageFilterMode::Linear;sampler.min_filter=ImageFilterMode::Linear;
        sampler.mipmap_filter=ImageFilterMode::Linear;sampler.anisotropy_clamp=TEXTURE_ANISOTROPY;
        image.sampler=ImageSampler::Descriptor(sampler);
        info!("Servo texture filtered: {:?}, {}x{}, {} levels",assets.get_path(id),w,h,image.texture_descriptor.mip_level_count);
    }
}
/// Keep highlights bright when downsampling sRGB, with alpha-weighted colour to avoid fringes.
fn colour_mips(w:u32,h:u32,base:&[u8])->Vec<u8> {
    let linear: [f32;256]=std::array::from_fn(|i|{let c=i as f32/255.;if c<=0.04045 {c/12.92}else{((c+0.055)/1.055).powf(2.4)}});
    let encode=|c:f32|{let c=if c<=0.0031308{12.92*c}else{1.055*c.powf(1./2.4)-0.055};(c.clamp(0.,1.)*255.).round() as u8};
    let mut out=base.to_vec();let mut prev=base.to_vec();let (mut w,mut h)=(w as usize,h as usize);
    while w>1 || h>1 {
        let (nw,nh)=((w/2).max(1),(h/2).max(1));let mut next=vec![0; nw*nh*4];
        for y in 0..nh {for x in 0..nw {
            let mut rgb=[0.;3];let mut alpha=0.;let mut count=0.;
            for sy in y*h/nh..(y+1)*h/nh {for sx in x*w/nw..(x+1)*w/nw {
                let i=(sy*w+sx)*4;let a=prev[i+3] as f32;
                for k in 0..3 {rgb[k]+=linear[prev[i+k] as usize]*a;}
                alpha+=a;count+=1.;
            }}
            let o=(y*nw+x)*4;
            if alpha>0. {for k in 0..3 {next[o+k]=encode(rgb[k]/alpha);}}
            next[o+3]=(alpha/count).round() as u8;
        }}
        out.extend_from_slice(&next);prev=next;w=nw;h=nh;
    }
    out
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn colour_mips_preserve_linear_brightness_and_transparent_edges() {
        let m=colour_mips(2,1,&[0,0,0,255,255,255,255,255]);assert_eq!(&m[8..],&[188,188,188,255]);
        let m=colour_mips(2,1,&[255,0,0,255,0,0,255,0]);assert_eq!(&m[8..],&[255,0,0,128]);
        let m=colour_mips(3,1,&[0,0,0,255,0,0,0,255,255,255,255,255]);assert_eq!(m.len(),16);assert_eq!(m[12],156);
    }
    #[test] fn filters_only_servo_environment_assets() {
        assert!(is_servo_asset(std::path::Path::new("models/smoko_servo.glb")));
        assert!(!is_servo_asset(std::path::Path::new("models/player.glb")));
        assert!(!is_servo_asset(std::path::Path::new("textures/lawn.png")));
    }
}
