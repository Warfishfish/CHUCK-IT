//! Host-owned charge starts and overhold recovery. Animation is reconstructed from state.
use std::collections::BTreeMap;
use bbq_core::{hands,net_act::{GuestAct,HitMsg},pose::SlapKind,vec::V3};
use crate::{game::Game,player::{Player,PLAYER_ID}};
pub const FALL_SECONDS:f32=1.8;
#[derive(Default)]
pub struct State {pub starts:BTreeMap<u32,(u32,f32)>,pub falls:BTreeMap<u32,f32>}
pub fn request(g:&mut Game,id:u32,item:u32,pressed:bool) {
 if !pressed {g.charge.starts.remove(&id);if let Some(d)=g.dummies.iter_mut().find(|d|d.id==id){d.bot.winding=false;d.bot.wind_progress=0.0;}return;}
 if !g.rules.in_play() || g.charge.falls.get(&id).is_some_and(|t|*t>g.now){return;}
 let valid=if id==PLAYER_ID {g.slots.selected()==Some(item)&&!g.me.body.is_down()}else{g.dummies.iter().any(|d|d.id==id&&d.bot.slots.selected()==Some(item)&&!d.body.is_down()&&d.body.stun<=0.0&&d.dragged.is_none())};
 if valid {g.charge.starts.entry(id).or_insert((item,g.now));}
}
pub fn fall(g:&mut Game,p:&mut Player,id:u32,item:u32) {
 if g.charge.falls.get(&id).is_some_and(|t|*t>g.now){return;}
 g.charge.starts.remove(&id);g.charge.falls.insert(id,g.now+FALL_SECONDS);
 let pos=if id==PLAYER_ID {g.slots.remove(item);g.wind.cancel();g.me.body.down_t=FALL_SECONDS;g.me.body.stun=FALL_SECONDS;p.mover.vx=0.0;p.mover.vz=0.0;g.popup("WHOOPS!",false);V3::new(p.mover.x,p.mover.y+1.0,p.mover.z)}else{
  let Some(d)=g.dummies.iter_mut().find(|d|d.id==id) else{return};
  d.bot.slots.remove(item);d.bot.winding=false;d.body.down_t=FALL_SECONDS;d.body.stun=FALL_SECONDS;d.anim.start_slap(SlapKind::SentFlying,FALL_SECONDS);
  g.net_hits.push((id,HitMsg::Slap{dir:(0.0,0.0,0.0),knock:0.0,up:0.0,stun:FALL_SECONDS,dizzy:0.0,down:Some((0,FALL_SECONDS))}));
  V3::new(d.mover.x,d.mover.y+1.0,d.mover.z)
 };
 g.world.drop_item(item,pos,Some((id,g.now+hands::OVERHOLD_BLOCK)));
 g.fx(crate::fx::FxKind::Dust,V3::new(pos.x,0.1,pos.z));
 g.say("Held it too long! Back on your feet in a moment.");
}
pub fn tick(g:&mut Game,p:&mut Player) {
 if !g.rules.in_play(){g.charge=State::default();return;}
 g.charge.falls.retain(|_,until|*until>g.now);
 let entries:Vec<_>=g.charge.starts.iter().map(|(&id,&v)|(id,v)).collect();
 for (id,(item,start)) in entries {
  let valid=g.dummies.iter().find(|d|d.id==id).is_some_and(|d|d.bot.slots.selected()==Some(item)&&d.body.stun<=0.0&&!d.body.is_down());
  if !valid {g.charge.starts.remove(&id);if let Some(d)=g.dummies.iter_mut().find(|d|d.id==id){d.bot.winding=false;d.bot.wind_progress=0.0;}continue;}
  let Some(kind)=g.world.items.get(&item).map(|i|i.kind) else {g.charge.starts.remove(&id);continue};
  if g.now-start>kind.def().charge+hands::OVERHOLD {fall(g,p,id,item);}else if let Some(d)=g.dummies.iter_mut().find(|d|d.id==id){d.bot.winding=true;d.bot.wind_progress=((g.now-start)/kind.def().charge).clamp(0.0,1.0);}
 }
}
pub fn send_start(g:&mut Game){if let Some(item)=g.slots.selected(){g.net_acts.push(GuestAct::Charge{item,pressed:true});}}

#[cfg(test)] mod tests {
 use super::*;use bevy::prelude::*;
 fn game()->App{let mut a=App::new();a.add_plugins(MinimalPlugins);a.add_plugins(crate::game::GamePlugin);a}
 #[test] fn invalid_charge_and_spam_cannot_restart_clock(){let mut a=game();let mut g=a.world_mut().resource_mut::<Game>();g.rules.phase=bbq_core::scoring::Phase::Play;request(&mut g,999,88,true);assert!(g.charge.starts.is_empty());let mut rng=bbq_core::rng::Rng::new(4);let item=g.world.spawn(bbq_core::items::ItemKind::Teddy,0.0,0.0,false,&mut rng);g.world.give(item,PLAYER_ID);g.slots.add(item);request(&mut g,PLAYER_ID,item,true);let start=g.charge.starts[&PLAYER_ID].1;g.now+=0.2;request(&mut g,PLAYER_ID,item,true);assert_eq!(g.charge.starts[&PLAYER_ID].1,start);request(&mut g,PLAYER_ID,item,false);assert!(g.charge.starts.is_empty());}
}
