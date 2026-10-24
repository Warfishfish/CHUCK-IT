//! Backyard nuisance bird integration. All hit/score decisions run only on the host.
use bbq_core::{GameMode,flight::ItemState,magpie::*,vec::V3};
use crate::{game::Game,player::{Player,PLAYER_ID}};

/// Normal teams rescue mates; a flock reverses the rule. Solo rescue means anyone else.
pub fn can_rescue(g:&Game,actor:u32,victim:u32,flock:bool)->bool {
    actor!=victim && if g.rules.mode==GameMode::Teams {g.teams.same_team(actor,victim)!=flock}else{true}
}
pub fn step(g:&mut Game,p:&Player,dt:f32) {
    let enabled=matches!(g.rules.mode,GameMode::FreeForAll|GameMode::Teams) && g.rules.in_play() && !g.attract;
    if !enabled {g.magpie=Default::default();g.flock.clear();g.flock_next=g.now+FLOCK_INTERVAL;g.flock_until=0.0;return;}
    let mut targets=Vec::new();
    if !g.me.body.is_down() && !p.mover.in_pool {targets.push((PLAYER_ID,V3::new(p.mover.x,p.mover.y+p.mover.height-0.25,p.mover.z)));}
    for d in &g.dummies {if !d.body.is_down() && !d.mover.in_pool {targets.push((d.id,V3::new(d.mover.x,d.mover.y+d.mover.height-0.25,d.mover.z)));}}
    if g.now>=g.flock_next && targets.len()>1 {
        g.flock_next=g.now+FLOCK_INTERVAL;g.flock_until=g.now+FLOCK_DURATION;g.magpie.repel();
        g.flock=targets.iter().take(MAX_FLOCK).enumerate().map(|(i,&(id,at))|Bird{phase:1,target:id,aim:at,timer:WARNING_SECONDS+0.08*i as f32,cling:true,..Default::default()}).collect();
        g.say("MAGPIE FLOCK! Rescue opponents: catch or hit their bird!");
    }
    if g.flock_until>0.0 && g.now>=g.flock_until {for b in &mut g.flock {if b.phase!=0{b.repel();}}g.flock_until=0.0;}
    let mut birds=std::mem::take(&mut g.flock);
    let flock=!birds.is_empty();
    if !flock {g.magpie.cling=g.rules.mode==GameMode::Teams;birds.push(g.magpie.clone());}
    // Nearby AI helpers use the same catch radius and eligibility as humans.
    let helpers:Vec<_>=g.dummies.iter().filter(|d|d.remote.is_none()&&!d.body.is_down()).filter_map(|d|{
        birds.iter().filter(|b|b.phase==4&&can_rescue(g,d.id,b.target,flock)).min_by(|a,b|{let dist=|v:&Bird|(v.pos.x-d.mover.x).hypot(v.pos.z-d.mover.z);dist(a).total_cmp(&dist(b))}).map(|b|(d.id,b.pos))
    }).collect();
    if g.options.bots_on {for (id,at) in helpers {if let Some(d)=g.dummies.iter_mut().find(|d|d.id==id){let dx=at.x-d.mover.x;let dz=at.z-d.mover.z;let dist=dx.hypot(dz);d.bot.wish=if dist>0.8{(dx/dist,dz/dist)}else{(0.0,0.0)};if dist<1.2{d.bot.catcher.press(true);}}}}
    for bird in &mut birds {
        if matches!(bird.phase,1|2|4) {
            let permitted=|actor| if !flock && g.rules.mode==GameMode::FreeForAll {true}else{can_rescue(g,actor,bird.target,flock)};
            let hit=g.world.items.values().find(|it|it.state==ItemState::Flying && it.live && it.thrower.is_some_and(&permitted) && segment_distance(bird.pos,it.pos,it.pos+it.vel*dt)<0.48).map(|it|it.id);
            let caught=(g.catcher.open() && permitted(PLAYER_ID) && (bird.pos-V3::new(p.mover.x,p.mover.y+p.mover.height-0.2,p.mover.z)).len()<1.3)
                || g.dummies.iter().any(|d|d.bot.catcher.open() && permitted(d.id) && (bird.pos-V3::new(d.mover.x,d.mover.y+d.mover.height-0.2,d.mover.z)).len()<1.3);
            if hit.is_some() || caught {if let Some(id)=hit {if let Some(it)=g.world.items.get_mut(&id){it.live=false;it.vel=it.vel*0.35;}}bird.repel();g.say("Magpie rescued! Back to the game.");}
        }
        let old=bird.phase;
        // After a flock rescue each bird stays retired for the rest of that event.
        if !(flock && bird.phase==0) {
            if let Some(victim)=bird.tick(dt,true,&targets) {
                if g.board.award(&g.rules,victim,PECK_POINTS)!=0 {let who=crate::bots_app::name_of(g,victim);g.say(format!("Magpie pecked {who}: -10"));if victim==PLAYER_ID{g.popup("MAGPIE! -10",false);}}
            }
        }
        if !flock && old==0 && bird.phase==1 {g.say("MAGPIE SWOOP! Dodge, catch, or help your mate!");}
    }
    if flock {if birds.iter().all(|b|b.phase==0){g.flock_until=0.0;}else{g.flock=birds;}}else if let Some(b)=birds.pop(){g.magpie=b;}
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
   assert_eq!(g.board.score(PLAYER_ID),if matches!(mode,GameMode::FreeForAll|GameMode::Teams){-10}else{0});
  }
 }
 #[test] fn rescue_rules_reverse_only_for_flock(){let(mut app,_)=setup();let mut g=app.world_mut().resource_mut::<Game>();g.rules.mode=GameMode::Teams;g.teams.set(1,bbq_core::teams::Team::Red);g.teams.set(2,bbq_core::teams::Team::Red);g.teams.set(3,bbq_core::teams::Team::Blue);assert!(can_rescue(&g,2,1,false));assert!(!can_rescue(&g,3,1,false));assert!(can_rescue(&g,3,1,true));assert!(!can_rescue(&g,2,1,true));assert!(!can_rescue(&g,1,1,true));}
 #[test] fn only_live_projectile_intersection_repels(){
  for on_target in [false,true]{let(mut app,p)=setup();let mut g=app.world_mut().resource_mut::<Game>();
   g.magpie.pos=V3::new(0.0,2.0,4.0);g.magpie.aim=V3::new(0.0,2.0,0.0);
   let mut it=bbq_core::flight::Item::new(999,bbq_core::items::ItemKind::Teddy,V3::new(if on_target{0.0}else{3.0},2.0,4.0));
   it.state=ItemState::Flying;it.live=true;it.thrower=Some(PLAYER_ID);g.world.items.insert(999,it);step(&mut g,&p,1.0/60.0);
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
