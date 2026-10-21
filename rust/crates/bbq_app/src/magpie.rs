//! Backyard nuisance bird integration. All hit/score decisions run only on the host.
use bbq_core::{GameMode,flight::ItemState,magpie::*,vec::V3};
use crate::{game::Game,player::{Player,PLAYER_ID}};

pub fn step(g:&mut Game,p:&Player,dt:f32) {
    let enabled=g.rules.mode==GameMode::FreeForAll && g.rules.in_play() && !g.attract;
    let mut targets=Vec::new();
    if !g.me.body.is_down() && g.me.body.stun<=0.0 && !p.mover.in_pool {
        targets.push((PLAYER_ID,V3::new(p.mover.x,p.mover.y+p.mover.height-0.25,p.mover.z)));
    }
    for d in &g.dummies {
        if !d.body.is_down() && d.body.stun<=0.0 && !d.mover.in_pool {
            targets.push((d.id,V3::new(d.mover.x,d.mover.y+d.mover.height-0.25,d.mover.z)));
        }
    }
    if enabled && matches!(g.magpie.phase,1|2) {
        // Only an actual live projectile crossing the bird repels it; scenery misses do not.
        let hit=g.world.items.values().find(|it|it.state==ItemState::Flying && it.live && segment_distance(g.magpie.pos,it.pos,it.pos+it.vel*dt)<0.48).map(|it|it.id);
        let caught=(g.catcher.open() && (g.magpie.pos-V3::new(p.mover.x,p.mover.y+p.mover.height-0.2,p.mover.z)).len()<1.3)
            || g.dummies.iter().any(|d|d.bot.catcher.open() && (g.magpie.pos-V3::new(d.mover.x,d.mover.y+d.mover.height-0.2,d.mover.z)).len()<1.3);
        if hit.is_some() || caught {
            if let Some(id)=hit {if let Some(it)=g.world.items.get_mut(&id){it.live=false;it.vel=it.vel*0.35;}}
            g.magpie.repel();g.say("Magpie chased off!");
        }
    }
    let old=g.magpie.phase;
    if let Some(victim)=g.magpie.tick(dt,enabled,&targets) {
        let score=g.board.award(&g.rules,victim,PECK_POINTS);
        if score!=0 {let who=crate::bots_app::name_of(g,victim);g.say(format!("Magpie pecked {who}: -10"));
            if victim==PLAYER_ID {g.popup("MAGPIE! -10",false);}
        }
    }
    if old==0 && g.magpie.phase==1 {let who=crate::bots_app::name_of(g,g.magpie.target);g.say(format!("MAGPIE SWOOP! {who}, dodge or catch!"));}
}

#[cfg(test)] mod tests {
 use super::*;
 use bevy::prelude::*;
 fn setup()->(App,Player){let mut app=App::new();app.add_plugins(MinimalPlugins);app.add_plugins(crate::game::GamePlugin);
 let p=Player{mover:bbq_core::movement::Mover::new(0.0,0.0),prev:Vec3::ZERO,yaw:0.0,pitch:0.0,walk:0.0,shake:0.0,fov_base:85.0,fov:85.0,rng:bbq_core::rng::Rng::new(1)};
 {let mut g=app.world_mut().resource_mut::<Game>();g.world.clear();g.dummies.clear();g.magpie.phase=2;g.magpie.timer=0.0;g.magpie.target=PLAYER_ID;g.magpie.pos=V3::new(0.0,1.70,0.5);g.magpie.aim=V3::new(0.0,1.70,0.0);}
 (app,p)}
 #[test] fn only_backyard_loses_ten_and_repeat_ticks_do_not_charge_again(){
  for mode in [GameMode::FreeForAll,GameMode::Teams,GameMode::Heist,GameMode::Smoko,GameMode::Race]{
   let(mut app,p)=setup();let mut g=app.world_mut().resource_mut::<Game>();g.rules.mode=mode;
   for _ in 0..60 {step(&mut g,&p,1.0/60.0);}
   assert_eq!(g.board.score(PLAYER_ID),if mode==GameMode::FreeForAll{-10}else{0});
  }
 }
 #[test] fn only_live_projectile_intersection_repels(){
  for on_target in [false,true]{let(mut app,p)=setup();let mut g=app.world_mut().resource_mut::<Game>();
   g.magpie.pos=V3::new(0.0,2.0,4.0);g.magpie.aim=V3::new(0.0,2.0,0.0);
   let mut it=bbq_core::flight::Item::new(999,bbq_core::items::ItemKind::Teddy,V3::new(if on_target{0.0}else{3.0},2.0,4.0));
   it.state=ItemState::Flying;it.live=true;g.world.items.insert(999,it);step(&mut g,&p,1.0/60.0);
   assert_eq!(g.magpie.phase==3,on_target);assert_eq!(g.world.items[&999].live,!on_target);assert_eq!(g.board.score(PLAYER_ID),0);
  }
 }
 #[test] fn menu_countdown_results_reset_attack_without_points(){
  use bbq_core::scoring::Phase;
  for (phase,attract) in [(Phase::Countdown,false),(Phase::Results,false),(Phase::Play,true)] {
   let(mut app,p)=setup();let mut g=app.world_mut().resource_mut::<Game>();g.rules.phase=phase;g.attract=attract;
   step(&mut g,&p,1.0/60.0);assert_eq!(g.magpie,Bird::default());assert_eq!(g.board.score(PLAYER_ID),0);
  }
 }

}
