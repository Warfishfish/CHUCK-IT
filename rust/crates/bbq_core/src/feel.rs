//! Presentation only. Never changes velocities, collisions, charge time or scoring.
use crate::{items::ItemKind, pose::Pose, vec::{Quat, V3}};

// Animation tuning: seconds, metres and radians. Weight is an artistic 0..1 value.
pub const WINDUP_LIGHT: f32 = 0.10;
pub const WINDUP_HEAVY: f32 = 0.20;
pub const WIND_SQUASH: f32 = 0.16;
pub const RELEASE_STRETCH: f32 = 0.23;
pub const THROW_SETTLE: f32 = 0.72;
pub const THROW_FOLLOW_THROUGH:f32=0.14;
pub const HIT_STOP: f32 = 0.070;
pub const HIT_SQUASH: f32 = 0.22;
pub const HIT_SHOVE: f32 = 0.09;
pub const HIT_SETTLE: f32 = 0.65;
pub const WOBBLE_CYCLES: f32 = 2.5;
pub const SHAKE_MIN: f32 = 0.035;
pub const SHAKE_MAX: f32 = 0.10;
pub const POPUP_MIN: f32 = 26.0;
pub const POPUP_MAX: f32 = 38.0;
pub const POPUP_RISE: f32 = 4.0; // screen percentage over popup life

pub fn weight(kind: ItemKind) -> f32 {
    match kind {
        ItemKind::Noodle => 0.0,
        ItemKind::Teddy | ItemKind::Snag => 0.2,
        ItemKind::WaterBalloon | ItemKind::Steak => 0.45,
        ItemKind::Fish => 0.65,
        ItemKind::Stubby => 0.8,
        ItemKind::Dildo => 1.0,
    }
}
// Existing knock strength is the game's impact-strength proxy; no damage rules added.
pub fn strength(knock: f32) -> f32 { (knock / 10.0).clamp(0.25, 1.0) }
pub fn shake(knock: f32) -> f32 { SHAKE_MIN + (SHAKE_MAX-SHAKE_MIN)*strength(knock) }
pub fn popup_size(knock: f32) -> f32 { POPUP_MIN+(POPUP_MAX-POPUP_MIN)*strength(knock) }

#[derive(Clone, Debug, Default)]
pub struct Feel {
    wind: f32,
    pub weight: f32,
    release: Option<f32>,
    impact: Option<(f32, f32, V3)>,
}
impl Feel {
    pub fn release(&mut self) { self.release=Some(0.0); self.wind=0.0; }
    pub fn hit(&mut self, dir: V3, knock: f32) {
        self.impact=Some((0.0,strength(knock),dir.normalised()));
        self.release=None;
    }
    pub fn frozen(&self) -> bool { self.impact.is_some_and(|(t,_,_)| t < HIT_STOP) }
    pub fn tick(&mut self, dt: f32, charging: bool, weight: f32, enabled: bool, p: &mut Pose) {
        let dt=dt.clamp(0.0,0.1);
        if charging {self.weight=weight.clamp(0.0,1.0); self.wind+=dt;} else {self.wind=0.0;}
        let w=self.weight;
        if enabled {
            if charging && self.release.is_none() {
                let u=(self.wind/(WINDUP_LIGHT+(WINDUP_HEAVY-WINDUP_LIGHT)*w)).min(1.0);
                let s=WIND_SQUASH*(0.65+0.7*w)*u*u*(3.0-2.0*u);
                p.body_scale=(1.0+s*0.5,1.0-s);
                p.body_rot=p.body_rot.mul(Quat::from_euler_xyz(-s*1.3,0.0,0.0));
            }
            if let Some(t)=self.release {
                let u=t/THROW_SETTLE;
                let s=RELEASE_STRETCH*(0.65+0.7*w)*(-5.0*u).exp()*(u*std::f32::consts::TAU*WOBBLE_CYCLES).cos();
                p.body_scale=(1.0-s*0.4,1.0+s);
                p.body_rot=p.body_rot.mul(Quat::from_euler_xyz(s,0.0,0.0));
                let reach=(std::f32::consts::PI*(t/0.24).clamp(0.0,1.0)).sin()*(0.7+0.3*w);
                p.body_z+=s*0.15+reach*THROW_FOLLOW_THROUGH;
                p.hand_r.z+=reach*0.16;
            }
        }
        if let Some(t)=&mut self.release { *t+=dt;if *t>=THROW_SETTLE {self.release=None;} }
        if let Some((t,power,dir))=self.impact {
            let u=((t-HIT_STOP).max(0.0)/HIT_SETTLE).min(1.0);
            let pulse=(-5.0*u).exp()*(u*std::f32::consts::TAU*WOBBLE_CYCLES).cos();
            let s=HIT_SQUASH*power*pulse;
            let local_x=dir.x*p.yaw.cos()-dir.z*p.yaw.sin();
            let local_z=dir.x*p.yaw.sin()+dir.z*p.yaw.cos();
            if enabled {
                p.body_scale.0*=1.0-s*local_x.abs();
                p.depth_scale*=(1.0-s*local_z.abs())/(1.0-s*local_x.abs());
                p.body_scale.1*=1.0+s*0.45;
                p.body_rot=p.body_rot.mul(Quat::from_euler_xyz(s*local_z,0.0,-s*local_x));
            }
            let shove=HIT_SHOVE*power*(1.0-u).powi(2);
            p.offset.0+=dir.x*shove; p.offset.1+=dir.z*shove;
            self.impact=if t+dt>=HIT_STOP+HIT_SETTLE {None}else{Some((t+dt,power,dir))};
        }
    }
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn heavy_winds_slower_and_releases_bigger() {
        let mut light=Feel::default();let mut heavy=Feel::default();
        let mut a=Pose::default();let mut b=Pose::default();
        light.tick(0.1,true,0.0,true,&mut a);heavy.tick(0.1,true,1.0,true,&mut b);
        assert!(a.body_scale.1 < 1.0 && b.body_scale.1 < 1.0);
        light.release();heavy.release();a=Pose::default();b=Pose::default();
        light.tick(0.016,false,0.0,true,&mut a);heavy.tick(0.016,false,0.0,true,&mut b);
        assert!(b.body_scale.1>a.body_scale.1);
    }
    #[test] fn directional_impact_freezes_briefly_then_settles() {
        let mut f=Feel::default();f.hit(V3::new(1.0,0.0,0.0),8.0);
        assert!(f.frozen());let mut p=Pose::default();f.tick(0.016,false,0.0,true,&mut p);
        assert!(p.body_scale.0<1.0 && p.offset.0>0.0);
        for _ in 0..60 {p=Pose::default();f.tick(0.016,false,0.0,true,&mut p);}
        assert!(!f.frozen());assert_eq!(p.body_scale,(1.0,1.0));assert_eq!(p.offset,(0.0,0.0));
    }
    #[test] fn impact_feedback_is_bounded_and_scales() {
        assert!(shake(3.0)<shake(10.0));assert!((shake(1000.0)-SHAKE_MAX).abs()<1e-6);
        assert!(popup_size(3.0)<popup_size(10.0));
    }
}
