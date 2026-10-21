//! One Backyard-only nuisance bird. Host advances this state; peers only draw snapshots.
use crate::vec::V3;
pub const PECK_POINTS:i32=-10;
pub const REST_SECONDS:f32=16.0;
pub const WARNING_SECONDS:f32=1.25;
pub const SWOOP_SPEED:f32=19.0;
pub const PECK_RADIUS:f32=0.70;
pub const PERCH:V3=V3::new(14.0,crate::looks_yard::FENCE_H,crate::YARD_HALF_Z+0.05);

#[derive(Clone,Debug,PartialEq)]
pub struct Bird {
    /// 0 perched, 1 warning, 2 committed swoop, 3 retreat.
    pub phase:u8, pub pos:V3, pub target:u32, pub timer:f32,
    pub aim:V3, pub previous_target:u32,
}
impl Default for Bird {
    fn default()->Self {Self{phase:0,pos:PERCH,target:0,timer:REST_SECONDS,aim:PERCH,previous_target:0}}
}
impl Bird {
    pub fn repel(&mut self) {self.phase=3;self.timer=0.0;self.target=0;}
    /// Targets contain only standing, active participants. Lock aim after warning so dodging works.
    pub fn tick(&mut self,dt:f32,enabled:bool,targets:&[(u32,V3)])->Option<u32> {
        if !enabled {*self=Self::default();return None;}
        let dt=dt.clamp(0.0,0.05);
        if self.phase>0 && self.phase<3 && !targets.iter().any(|t|t.0==self.target) {self.repel();}
        match self.phase {
            0=>{self.timer-=dt;if self.timer<=0.0 {
                let t=targets.iter().filter(|t|t.0>self.previous_target).min_by_key(|t|t.0).or_else(||targets.iter().min_by_key(|t|t.0));
                if let Some(&(id,at))=t {self.target=id;self.aim=at;self.phase=1;self.timer=WARNING_SECONDS;}else{self.timer=2.0;}
            }},
            1=>{self.timer-=dt;if let Some(t)=targets.iter().find(|t|t.0==self.target){self.aim=t.1;}
                self.pos=PERCH+V3::new(0.0,(WARNING_SECONDS-self.timer)*0.45,0.0);
                if self.timer<=0.0 {self.phase=2;self.timer=0.0;}
            },
            2=>{let old=self.pos;let d=self.aim-self.pos;self.pos=self.pos+d.normalised()*(SWOOP_SPEED*dt).min(d.len());
                self.timer+=dt;
                let hit=targets.iter().find(|t|t.0==self.target && segment_distance(t.1,old,self.pos)<PECK_RADIUS).map(|t|t.0);
                if hit.is_some() || (self.pos-self.aim).len()<0.05 || self.timer>4.0 {self.previous_target=self.target;self.repel();}
                return hit;
            },
            _=>{let d=PERCH-self.pos;self.pos=self.pos+d.normalised()*(12.0*dt).min(d.len());
                if d.len()<0.1 {self.phase=0;self.pos=PERCH;self.timer=REST_SECONDS;}
            },
        }
        None
    }
}
pub fn segment_distance(p:V3,a:V3,b:V3)->f32 {let d=b-a;let t=((p-a).dot(d)/d.dot(d).max(0.00001)).clamp(0.0,1.0);(p-(a+d*t)).len()}

#[cfg(test)] mod tests {
 use super::*;
 #[test] fn contact_scores_once_and_retreats(){let mut b=Bird::default();b.timer=0.0;let t=[(1,V3::new(10.0,1.8,10.0))];let mut hits=0;for _ in 0..400 {hits+=b.tick(1.0/60.0,true,&t).is_some() as u32;}assert_eq!(hits,1);assert_eq!(b.target,0);}
 #[test] fn dodge_disconnect_and_disabled_never_peck(){let mut b=Bird::default();b.phase=2;b.timer=0.0;b.target=1;b.aim=V3::ZERO;b.pos=V3::new(0.0,0.0,1.0);assert_eq!(b.tick(0.05,true,&[(1,V3::new(5.0,0.0,0.0))]),None);b.target=1;b.phase=2;assert_eq!(b.tick(0.02,true,&[]),None);assert_eq!(b.phase,3);b.tick(0.02,false,&[]);assert_eq!(b,Bird::default());}
 #[test] fn repel_cancels_the_swoop(){let mut b=Bird::default();b.phase=2;b.timer=0.0;b.target=1;b.repel();for _ in 0..60{assert_eq!(b.tick(0.016,true,&[(1,PERCH)]),None);}}
}
