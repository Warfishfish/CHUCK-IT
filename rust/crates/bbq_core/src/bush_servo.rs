//! Bush Servo: Smoko's separate environment. Shared solid dimensions keep host and guest aligned.
use crate::looks::{Part, Shape, Surface, Tex};
use crate::vec::V3;
use crate::yard::{Collider, Kind};
use std::f32::consts::{TAU, FRAC_PI_2};

pub const CENTRE: (f32, f32) = (0., 10.);
pub const SCORE_RADIUS: f32 = 3.5;
pub const ROAD_SEGMENTS: usize = 96;
// Ground tuning: the baked colour maps carry the palette.
const DIRT: u32 = 0xffffff;
const ROAD_Z_SCALE: f32 = 0.65;
const GROUND_STONES: usize = 96;
const TIMBER: u32 = 0x956b43;
const CREAM: u32 = 0xe7d3ac;
const TEAL: u32 = 0x4d7770;
const RUST: u32 = 0xa95a43;
// x,z,width,depth,height. All ground-based solid props have matching meshes below.
pub const SOLIDS: [(f32,f32,f32,f32,f32); 7] = [
    (-4.,6.,0.24,0.24,3.8),(4.,6.,0.24,0.24,3.8),
    (-4.,14.,0.24,0.24,3.8),(4.,14.,0.24,0.24,3.8),
    (-2.3,15.2,3.2,0.5,0.46),(2.3,15.2,3.2,0.5,0.46),
    (5.6,15.9,1.3,0.75,0.72),
];
pub fn colliders()->Vec<Collider> {
    SOLIDS.iter().map(|&(x,z,w,d,h)|Collider::centred(x,z,w,d,h).of(Kind::Plain)).collect()
}
fn block(w:f32,h:f32,d:f32,x:f32,y:f32,z:f32,c:u32)->Part {
    Part::new(Shape::Cuboid{w,h,d},Surface::matt(c)).at(x,y,z)
}
fn timber(w:f32,h:f32,d:f32,x:f32,y:f32,z:f32)->Part {
    let mut p=block(w,h,d,x,y,z,TIMBER);p.surface=p.surface.textured(Tex::Wood);p
}
pub fn parts()->Vec<Part> { parts_with_shelter(true) }
/// Imported Blender shelter replaces only the shelter visuals; physics stays shared.
pub fn parts_with_shelter(procedural_shelter: bool)->Vec<Part> {
    let mut p=vec![Part::new(Shape::Quad{w:260.,h:260.},Surface::matt(DIRT).textured(Tex::ServoDirt).repeating(32.,32.).no_shadow()).at(0.,-0.018,0.).turn(-FRAC_PI_2,0.,0.)];
    // One opaque irregular annulus: worn asphalt receives normal ground shadows.
    // Purely visual; movement and projectile collision remain the shared flat ground.
    let mut road=Part::new(Shape::Ring{inner:18.,outer:26.,seg:ROAD_SEGMENTS as u32},
        Surface::matt(0xffffff).textured(Tex::ServoRoad).no_shadow())
        .at(0.,0.008,6.).turn(-FRAC_PI_2,0.,0.);
    road.scale=V3::new(1.,ROAD_Z_SCALE,1.);p.push(road);
    // Low embedded stones at shoulders: below boot height, never an implied obstacle.
    for i in 0..GROUND_STONES {
        let a=i as f32*2.399963;
        let r=if i%2==0 {26.4+(i%7) as f32*0.17} else {17.4-(i%5) as f32*0.13};
        let x=r*a.cos();let z=6.+r*a.sin()*ROAD_Z_SCALE;
        if x*x+(z-10.)*(z-10.)<45. {continue;}
        let size=0.045+(i%4) as f32*0.014;
        let mut stone=Part::new(Shape::Ico{r:size,detail:0},Surface::matt(if i%3==0{0xa97750}else{0xc19362}))
            .at(x,0.008,z).turn(0.,a,0.);
        stone.scale=V3::new(1.5,0.45,1.);p.push(stone);
    }
    for i in 0..ROAD_SEGMENTS {
        if i%3!=0 {continue;}
        let a=i as f32*TAU/ROAD_SEGMENTS as f32;
        let x=18.5*a.sin();let z=6.+12.025*a.cos();
        let yaw=(18.5*a.cos()).atan2(-12.025*a.sin());
        p.push(block(0.10,0.008,0.75,x,0.016,z,0x9f9179).turn(0.,yaw,0.));
    }
    // Level delivery pad: visual ground only, never obstructs throws or carried victims.
    // Existing scoring ring remains legible; earth continues naturally beneath the shelter.
    p.push(Part::new(Shape::Ring{inner:SCORE_RADIUS-0.12,outer:SCORE_RADIUS,seg:96},Surface::matt(0xf5ce69).no_shadow()).at(0.,0.035,10.).turn(-FRAC_PI_2,0.,0.));
    if procedural_shelter {
    for (i,&(x,z,w,d,h)) in SOLIDS.iter().enumerate() {
        p.push(if i==6 {block(w,h,d,x,h/2.,z,TEAL)}else{timber(w,h,d,x,h/2.,z)});
        if i<4 {p.push(block(0.28,0.10,0.28,x,0.05,z,0xaca492));}
    }
    p.push(block(1.34,0.08,0.79,5.6,0.76,15.9,CREAM));
    // Open-sided shelter, wide clear approaches. High roof leaves tall avatars clearance.
    for z in [6.,14.] {p.push(timber(8.5,0.22,0.18,0.,3.65,z));}
    for x in [-4.,4.] {p.push(timber(0.18,0.22,8.5,x,3.65,10.));}
    for (z,tilt) in [(7.75,-0.12),(12.25,0.12)] {
        let mut roof=block(9.3,0.10,4.6,0.,4.02,z,RUST).turn(tilt,0.,0.);
        roof.surface=roof.surface.textured(Tex::Corrugated).repeating(3.,1.);p.push(roof);
    }
    p.push(block(9.4,0.12,0.2,0.,4.31,10.,0x805549));
    for z in [5.45,14.55] {p.push(block(9.4,0.12,0.18,0.,3.73,z,CREAM));}
    // Existing legible Smoko artwork, on both approaches, grounded within a real signboard.
    for (z,yaw) in [(5.86, std::f32::consts::PI),(14.14,0.)] {
        p.push(block(3.5,0.95,0.10,0.,3.05,z,TEAL));
        p.push(Part::new(Shape::Quad{w:3.2,h:0.8},Surface::matt(0xffffff).textured(Tex::SignSmoko).no_shadow()).at(0.,3.05,z+if yaw==0. {0.06}else{-0.06}).turn(0.,yaw,0.));
    }
    } else {
        // Keep the existing solid esky and lid; Blender supplies posts and rear benches.
        p.push(block(1.3,0.72,0.75,5.6,0.36,15.9,TEAL));
        p.push(block(1.34,0.08,0.79,5.6,0.76,15.9,CREAM));
    }
    // Servo facade beyond the northern arena fence: no unreachable fake interaction points.
    p.push(block(23.,4.6,7.,-8.,2.3,-29.,CREAM));
    let mut roof=block(25.,0.22,9.,-8.,4.7,-29.,RUST).turn(0.04,0.,0.);roof.surface=roof.surface.textured(Tex::Corrugated).repeating(5.,2.);p.push(roof);
    for x in [-16.,-10.,-3.] {
        p.push(block(4.6,2.0,0.08,x,2.35,-25.45,TEAL));
        p.push(block(4.25,1.7,0.04,x,2.35,-25.39,0x789ba1));
        for xx in [x-2.24,x,x+2.24] {p.push(block(0.10,2.,0.09,xx,2.35,-25.32,CREAM));}
        p.push(block(4.6,0.12,0.16,x,1.34,-25.30,CREAM));
    }
    p.push(block(24.,0.55,0.22,-8.,4.12,-25.25,TEAL));
    p.push(Part::new(Shape::Quad{w:11.,h:0.9},Surface::matt(0xffffff).textured(Tex::SignServo)).at(-8.,4.03,-25.10));
    // Shaded porch, gutter, window transoms and grounded forecourt details.
    let mut awning=block(24.,0.14,2.,-8.,3.6,-24.9,RUST).turn(0.06,0.,0.);
    awning.surface=awning.surface.textured(Tex::Corrugated).repeating(5.,1.);p.push(awning);
    p.push(block(24.,0.12,0.14,-8.,3.5,-23.94,CREAM));
    for x in [-19.,3.] {p.push(block(0.15,3.5,0.15,x,1.75,-24.25,TEAL));}
    for x in [-16.,-10.,-3.] {p.push(block(4.5,0.08,0.12,x,2.6,-25.29,CREAM));}
    // Two vintage pumps set beyond the fence, not interactive supplies.
    for x in [8.,11.] {
        p.push(block(1.,0.18,1.1,x,0.09,-26.,0xb8ac95));
        p.push(block(0.7,0.9,0.6,x,0.63,-26.,RUST));
        p.push(block(0.8,0.85,0.64,x,1.5,-26.,CREAM));
        p.push(block(0.55,0.3,0.02,x,1.65,-25.67,0x384e4d));
        p.push(block(0.36,0.08,0.03,x,1.65,-25.65,0xc8d4bc));
        p.push(block(0.07,1.,0.07,x+0.48,1.1,-26.,0x45483f));
    }
    // Continuous stock fence marks the actual rectangular play limits; curved route is inside.
    for z in [-24.,24.] {for i in 0..23 {let x=-33.+i as f32*3.;p.push(timber(0.16,1.8,0.16,x,0.9,z));}
        for y in [0.55,1.25] {p.push(timber(66.,0.12,0.12,0.,y,z));}}
    for x in [-33.,33.] {for i in 0..17 {let z=-24.+i as f32*3.;p.push(timber(0.16,1.8,0.16,x,0.9,z));}
        for y in [0.55,1.25] {p.push(timber(0.12,0.12,48.,x,y,0.));}}
    // Branching gums, patchy low scrub and distant ochre ridges outside the play boundary.
    for i in 0..24 {
        let a=i as f32*TAU/24.;let x=43.*a.sin();let z=36.*a.cos();let h=4.5+(i%4) as f32*0.6;
        p.push(Part::new(Shape::Cylinder{top:0.12,bottom:0.32,h,seg:8,caps:true},Surface::matt(0xa9977b)).at(x,h/2.,z).turn(0.06,0.,0.05));
        for j in 0..5 {
            let b=j as f32*2.4+i as f32;let dx=b.sin()*2.4;let dz=b.cos()*2.4;
            let at=V3::new(x+dx,h+0.7+(j%2) as f32*0.5,z+dz);
            p.push(crate::looks_yard::rod(V3::new(x,h*0.6,z),at,0.13,Surface::matt(0xa9977b)));
            let mut leaf=Part::new(Shape::Sphere{r:1.75,ws:10,hs:7},Surface::matt(if j%2==0{0x6f8054}else{0x87915a})).at(at.x,at.y,at.z);
            leaf.scale=V3::new(1.3,0.48,1.);p.push(leaf);
        }
        for j in 0..3 {
            let dx=(j as f32-1.)*1.2;
            let mut shrub=Part::new(Shape::Ico{r:0.7,detail:1},Surface::matt(if j==1{0x8a8b54}else{0x79815b})).at(x+dx,0.5,z+2.);
            shrub.scale=V3::new(1.1,0.8,1.);p.push(shrub);
        }
    }
    for i in 0..12 {
        let a=i as f32*TAU/12.;let mut hill=Part::new(Shape::Ico{r:1.,detail:2},Surface::matt(if i%2==0{0xb9997e}else{0xc5a485})).at(90.*a.sin(),-2.,90.*a.cos());
        hill.scale=V3::new(22.,7.+(i%3) as f32*2.,18.);p.push(hill);
    }
    p
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn delivery_centre_and_lanes_are_clear(){
  for (x,z) in [(0.,10.),(0.,4.),(0.,16.),(-6.,10.),(6.,10.)] {
   assert!(!colliders().iter().any(|c|x>c.x0-0.45&&x<c.x1+0.45&&z>c.z0-0.45&&z<c.z1+0.45));
  }
 }
 #[test] fn solid_dimensions_match_visuals(){let p=parts();for c in colliders(){assert!(p.iter().any(|p|matches!(p.shape,Shape::Cuboid{w,h,d} if (w-(c.x1-c.x0)).abs()<0.001&&(d-(c.z1-c.z0)).abs()<0.001&&(h-c.h).abs()<0.001)&&(p.pos.x-(c.x0+c.x1)/2.).abs()<0.001&&(p.pos.z-(c.z0+c.z1)/2.).abs()<0.001));}}
}
