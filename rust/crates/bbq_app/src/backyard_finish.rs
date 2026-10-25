//! Opt-in first Backyard art pass. No collision, gameplay or network changes.
use bevy::prelude::*;
use std::sync::OnceLock;

pub const LAWN_TINT: Vec3 = Vec3::new(0.84, 0.88, 1.08);
pub const ROOT: Vec3 = Vec3::new(0.21, 0.31, 0.115);
pub const TIP: Vec3 = Vec3::new(0.37, 0.46, 0.21);
pub const SKY_HORIZON: Vec3 = Vec3::new(0.79, 0.86, 0.91);
pub const SKY_ZENITH: Vec3 = Vec3::new(0.25, 0.49, 0.78);
pub const WOOD_CONTRAST: f32 = 0.24;
pub const ROOF_CONTRAST: f32 = 0.36;
pub const PAINT_CONTRAST: f32 = 0.65;
pub const BODY_ROUGHNESS: f32 = 0.72;

pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        #[cfg(target_arch="wasm32")]
        { web_sys::window().and_then(|w|w.location().search().ok()).is_some_and(|q|q.trim_start_matches('?').split('&').any(|p|p=="backyard_finish=1")) }
        #[cfg(not(target_arch="wasm32"))]
        { std::env::args().any(|a|a=="--backyard-finish") }
    })
}

/// Trim sits against the existing house outside the playable fence; no new obstacles.
pub fn house_trim() -> Vec<bbq_core::looks::Part> {
    use bbq_core::looks::{Part,Shape,Surface};
    let matt=Surface::matt;
    let cuboid=|w,h,d,s,x,y,z|Part::new(Shape::Cuboid{w,h,d},s).at(x,y,z);
    let paint=matt(0xd8ddca);let metal=matt(0x6b8581);
    let mut parts=vec![cuboid(41.0,0.13,0.18,metal,0.0,4.52,-25.32),cuboid(41.0,0.13,0.18,metal,0.0,4.52,-33.68),cuboid(41.1,0.12,0.18,paint,0.0,6.69,-29.5)];
    for x in [-19.7,19.7] { parts.push(cuboid(0.12,4.2,0.12,metal,x,2.1,-25.29)); }
    for x in [-15.0,-9.0,-4.0,8.0,13.5] {
        parts.push(cuboid(2.05,0.08,0.27,paint,x,3.29,-25.34));
        parts.push(cuboid(2.05,0.05,0.22,matt(0x9b9c83),x,1.65,-25.36));
    }
    parts
}
