//! Host-owned Smoko delivery game. IDs, never vector indices, own carries and attribution.
use crate::game::Game;
use crate::player::{PLAYER_ID, Player, Wanted};
#[cfg(test)]
use bbq_core::movement::Mover;
use bbq_core::net_act::HitMsg;
use bbq_core::pose::{DraggerInfo, SlapKind};
use bbq_core::stun::Body;
use bbq_core::vec::V3;
use bbq_core::{GameMode, PlayerId};
use std::collections::BTreeMap;

pub const CENTRE: (f32, f32) = bbq_core::bush_servo::CENTRE;
pub const RADIUS: f32 = bbq_core::bush_servo::SCORE_RADIUS;
/// Minimum fresh combat knockdown in the delivery mode.
pub const KNOCKDOWN_TIME: f32 = 8.0;
pub fn knockdown_time(mode: GameMode, seconds: f32) -> f32 {
    if mode == GameMode::Smoko { seconds.max(KNOCKDOWN_TIME) } else { seconds }
}
pub fn extend_knockdown(mode: GameMode, body: &mut Body, fresh: bool) {
    if fresh {
        body.down_t = knockdown_time(mode, body.down_t);
        body.stun = body.stun.max(body.down_t);
    }
}
pub const MAX_CARRY: f32 = 8.0;
pub const THROW_WINDOW: f32 = 4.0;
pub const PROTECTION: f32 = 5.0;
const REACH: f32 = 2.2;
pub const RACE_MAX_CARRY: f32 = 3.0;
pub const RACE_RELEASE_PROTECTION: f32 = 5.0;
pub const RACE_FLIGHT_TIME: f32 = 1.2;
fn enabled(g: &Game)->bool {matches!(g.rules.mode,GameMode::Smoko|GameMode::Race)}
const SPAWNS: [(f32, f32); 4] = [(-20.0, -10.0), (20.0, -10.0), (-20.0, 18.0), (20.0, 18.0)];

#[derive(Clone, Copy)]
pub struct Carry {
    pub victim: PlayerId,
    pub since: f32,
}
#[derive(Default)]
pub struct State {
    pub carries: BTreeMap<PlayerId, Carry>,
    flights: BTreeMap<PlayerId, (PlayerId, f32)>,
    immune: BTreeMap<PlayerId, f32>,
    presses: BTreeMap<PlayerId, f32>,
    // Short host pose override for guest victims, refreshed at 15 Hz.
    pub guest_until: f32,
    pub guest_carrying: bool,
    last_sync: f32,
}

pub fn pos(g: &Game, p: &Player, id: PlayerId) -> Option<V3> {
    let m = if id == PLAYER_ID {
        &p.mover
    } else {
        &g.dummies.iter().find(|d| d.id == id)?.mover
    };
    Some(V3::new(m.x, m.y, m.z))
}
fn body(g: &Game, id: PlayerId) -> Option<&Body> {
    if id == PLAYER_ID {
        Some(&g.me.body)
    } else {
        g.dummies.iter().find(|d| d.id == id).map(|d| &d.body)
    }
}
fn down(g: &Game, id: PlayerId) -> bool {
    body(g, id).is_some_and(|b| b.is_down() || b.stun > 0.6)
}
pub fn able(g: &Game, id: PlayerId) -> bool {
    body(g, id).is_some_and(|b| !b.is_down() && b.stun <= 0.0)
}
pub fn carried(g: &Game, id: PlayerId) -> bool {
    g.smoko.carries.values().any(|c| c.victim == id)
}
pub fn controlled(g: &Game, id: PlayerId) -> bool {
    enabled(g)
        && (carried(g, id)
            || g.smoko.flights.contains_key(&id)
            || g.smoko
                .immune
                .get(&id)
                .is_some_and(|t| *t - g.now > PROTECTION - 0.35))
}
pub fn player_locked(g: &Game) -> bool {
    enabled(g)
        && (carried(g, PLAYER_ID) || (g.mirror && g.now < g.smoko.guest_until))
}
fn eligible(g: &Game, p: &Player, carrier: PlayerId, victim: PlayerId) -> bool {
    carrier != victim
        && (if g.rules.mode==GameMode::Race {
            crate::race::active_racer(g,carrier) && crate::race::active_racer(g,victim)
                && !controlled(g,carrier) && !controlled(g,victim)
                && pos(g,p,carrier).is_some_and(|v|v.y<0.6)
        } else {down(g,victim)})
        && !carried(g, victim)
        && !g.smoko.carries.contains_key(&victim)
        && !g.smoko.immune.get(&victim).is_some_and(|t| *t > g.now)
        && pos(g, p, victim).is_some_and(|v| v.y < 0.6)
}
fn nearest(g: &Game, p: &Player, id: PlayerId) -> Option<PlayerId> {
    let at = pos(g, p, id)?;
    std::iter::once(PLAYER_ID)
        .chain(g.dummies.iter().map(|d| d.id))
        .filter(|v| eligible(g, p, id, *v))
        .min_by(|a, b| {
            pos(g, p, *a)
                .unwrap()
                .horiz_dist(at)
                .total_cmp(&pos(g, p, *b).unwrap().horiz_dist(at))
        })
}
pub fn face(g: &Game, p: &Player, id: PlayerId) -> (f32, f32) {
    if id == PLAYER_ID {
        (-p.yaw.sin(), -p.yaw.cos())
    } else {
        g.dummies
            .iter()
            .find(|d| d.id == id)
            .map_or((0.0, 1.0), |d| (d.face.sin(), d.face.cos()))
    }
}
fn set_pose(g: &mut Game, p: &mut Player, id: PlayerId, at: V3, vel: V3, held: bool) {
    let (m, b) = if id == PLAYER_ID {
        (&mut p.mover, &mut g.me.body)
    } else {
        let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) else {
            return;
        };
        d.dragged = held.then_some(DraggerInfo {
            face: 0.0,
            speed: 2.0,
            walk: g.now * 5.0,
            grounded: true,
        });
        (&mut d.mover, &mut d.body)
    };
    m.x = at.x;
    m.y = at.y;
    m.z = at.z;
    m.vx = vel.x;
    m.vy = vel.y;
    m.vz = vel.z;
    m.grounded = at.y <= 0.01;
    b.stun = b.stun.max(0.5);
    if held {
        b.down_t = b.down_t.max(0.5);
    }
}
fn sync_person(g: &mut Game, p: &Player, id: PlayerId, lock: bool) {
    let Some(d) = g.dummies.iter().find(|d| d.id == id && d.remote.is_some()) else {
        return;
    };
    let m = &d.mover;
    let msg = HitMsg::Smoko {
        pos: (m.x, m.y, m.z),
        vel: (m.vx, m.vy, m.vz),
        locked: lock,
        carrying: g.smoko.carries.contains_key(&id),
    };
    g.net_hits.push((id, msg));
    let _ = p;
}
pub fn pickup_target(g:&Game,p:&Player,id:PlayerId)->Option<PlayerId> {
    if !enabled(g) || !g.rules.in_play() || !able(g,id) || carried(g,id) || g.smoko.carries.contains_key(&id) {return None;}
    let v=nearest(g,p,id)?;
    (pos(g,p,v)?.horiz_dist(pos(g,p,id)?)<=REACH).then_some(v)
}
fn grab(g: &mut Game, p: &mut Player, id: PlayerId) {
    if !able(g, id) || carried(g, id) || g.smoko.carries.contains_key(&id) {
        return;
    }
    let Some(v) = pickup_target(g, p, id) else { return };
    if pos(g, p, v).unwrap().horiz_dist(pos(g, p, id).unwrap()) > REACH {
        return;
    }
    if let Some(r)=g.race.racers.get_mut(&v) {r.return_to.get_or_insert(r.previous);}
    g.smoko.flights.remove(&v);
    g.smoko.carries.insert(
        id,
        Carry {
            victim: v,
            since: g.now,
        },
    );
    g.smoko.presses.remove(&id);
    if id == PLAYER_ID {
        g.wind.cancel();
    }
    sync_person(g, p, id, false);
}
fn release(g: &mut Game, p: &mut Player, id: PlayerId, throw: bool, credit: bool) {
    let Some(c) = g.smoko.carries.remove(&id) else {
        return;
    };
    g.smoko.presses.remove(&id);
    if let Some(at) = pos(g, p, c.victim) {
        let f = face(g, p, id);
        let vel = if throw {
            V3::new(f.0 * bbq_core::carry::THROW_HV, bbq_core::carry::THROW_VY, f.1 * bbq_core::carry::THROW_HV)
        } else {
            V3::ZERO
        };
        set_pose(g, p, c.victim, at, vel, false);
        if g.rules.mode==GameMode::Race {
            g.smoko.immune.insert(c.victim,g.now+RACE_RELEASE_PROTECTION);
            g.smoko.flights.insert(c.victim,(id,g.now+RACE_FLIGHT_TIME));
            let b=if c.victim==PLAYER_ID {&mut g.me.body}else{&mut g.dummies.iter_mut().find(|d|d.id==c.victim).unwrap().body};
            b.down_t=0.55;b.stun=0.55;
        } else if credit {
            g.smoko.flights.insert(
                c.victim,
                (id, g.now + if throw { THROW_WINDOW } else { 0.8 }),
            );
            if c.victim == PLAYER_ID {
                g.me.body.down_t = THROW_WINDOW;
                g.me.body.stun = THROW_WINDOW;
            } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == c.victim) {
                d.body.down_t = THROW_WINDOW;
                d.body.stun = THROW_WINDOW;
                if throw { d.anim.start_slap(SlapKind::SentFlying, THROW_WINDOW); }
            }
        } else {
            g.smoko.immune.insert(c.victim, g.now + 1.0);
        }
    }
    sync_person(g, p, id, false);
    sync_person(g, p, c.victim, credit);
}
/// Local and network inputs both enter this host-only validator. No client-supplied victim/score.
pub fn input(g: &mut Game, p: &mut Player, id: PlayerId, pressed: bool) {
    if g.mirror || !enabled(g) || !g.rules.in_play() {
        return;
    }
    if pressed {
        if g.smoko.carries.contains_key(&id) {
            g.smoko.presses.entry(id).or_insert(g.now);
        } else {
            grab(g, p, id);
        }
    } else if let Some(since) = g.smoko.presses.remove(&id) {
        release(g, p, id, g.now - since >= 0.25, true);
    }
}
fn in_pad(v: V3) -> bool {
    (v.x - CENTRE.0).hypot(v.z - CENTRE.1) < RADIUS && (-0.1..=1.4).contains(&v.y)
}
fn deliver(g: &mut Game, p: &mut Player, owner: PlayerId, victim: PlayerId) {
    if body(g, owner).is_none() || g.smoko.immune.get(&victim).is_some_and(|t| *t > g.now) {
        return;
    }
    g.smoko.carries.retain(|_, c| c.victim != victim);
    g.smoko.flights.remove(&victim);
    let points = g.board.deliver(&g.rules, owner);
    if points == 0 {
        return;
    }
    g.smoko.immune.insert(victim, g.now + PROTECTION);
    let (x, z) = SPAWNS[(victim as usize + g.board.score(owner) as usize / 100) % SPAWNS.len()];
    set_pose(g, p, victim, V3::new(x, 0.0, z), V3::ZERO, false);
    if victim == PLAYER_ID {
        g.me.body = Body::default();
        g.me.drunk.reset();
        g.life.seated = None;
    } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == victim) {
        d.body = Body::default();
        d.home = (x,z);
        d.seat = None;
        d.naughty_t = 0.0;
        d.bot.drunk.reset();
        d.thrown_at = None;
    }
    let who = crate::round::name_of(g, owner);
    g.say(format!("{who} delivered a mate to SMOKO! +{points}"));
    if owner == PLAYER_ID {
        g.popup(format!("SMOKO DELIVERY! +{points}"), true);
    }
    if victim == PLAYER_ID {
        g.popup("Delivered! Back in play — 5s pickup protection", false);
    }
    g.fx(crate::fx::FxKind::Hit, V3::new(CENTRE.0, 1.0, CENTRE.1));
    sync_person(g, p, victim, false);
    sync_person(g, p, owner, false);
}
/// Goal plugs into the existing obstacle-aware bot steering; no Heist objective is used.
pub fn bot_goal(g: &Game, p: &Player, id: PlayerId) -> Option<V3> {
    if g.rules.mode != GameMode::Smoko || !able(g, id) {
        return None;
    }
    if g.smoko.carries.contains_key(&id) {
        return Some(V3::new(CENTRE.0, 0.0, CENTRE.1));
    }
    nearest(g, p, id).and_then(|v| pos(g, p, v))
}
pub fn step(g: &mut Game, p: &mut Player, w: &mut Wanted) {
    if !enabled(g) { return; }
    let race=g.rules.mode==GameMode::Race;
    if !g.rules.in_play() {
        let ids: Vec<_> = g.smoko.carries.keys().copied().collect();
        for id in ids {
            release(g, p, id, false, false);
        }
        g.smoko.flights.clear();
        g.smoko.presses.clear();
        return;
    }
    if std::mem::take(&mut w.grab_pressed) {
        input(g, p, PLAYER_ID, true);
    }
    if std::mem::take(&mut w.grab_released) {
        input(g, p, PLAYER_ID, false);
    }
    // Bots opportunistically collect downed opponents, then use steering to reach the pad.
    let bots: Vec<_> = g
        .dummies
        .iter()
        .filter(|d| d.remote.is_none())
        .map(|d| d.id)
        .collect();
    if g.options.bots_on && !race {
        for id in bots {
            grab(g, p, id);
        }
    }
    let carries: Vec<_> = g.smoko.carries.iter().map(|(a, c)| (*a, *c)).collect();
    for (owner, c) in carries {
        if !able(g, owner) || body(g, c.victim).is_none() || g.now - c.since > if race {RACE_MAX_CARRY}else{MAX_CARRY} {
            release(g, p, owner, false, false);
            continue;
        }
        let Some(at) = pos(g, p, owner) else {
            release(g, p, owner, false, false);
            continue;
        };
        let f = face(g, p, owner);
        // Carry low at the right hip so the destination and crosshair remain visible.
        let held = if race {
            // The drag pose extends forward from its feet: tuck those behind the carrier
            // so the victim rests at the hip instead of floating a full body-length away.
            V3::new(at.x-f.0*0.4-f.1*0.55,at.y+0.65,at.z-f.1*0.4+f.0*0.55)
        } else { V3::new(at.x + f.0 * 1.2 - f.1 * 1.3, at.y + 0.1, at.z + f.1 * 1.2 + f.0 * 1.3) };
        set_pose(g, p, c.victim, held, V3::ZERO, true);
        if let Some(d) = g.dummies.iter_mut().find(|d| d.id == c.victim) {
            if let Some(dragger) = &mut d.dragged {
                dragger.face = f.0.atan2(f.1);
            }
        }
        if !race && in_pad(held) {
            deliver(g, p, owner, c.victim);
        }
    }
    let flights: Vec<_> = g.smoko.flights.iter().map(|(v, o)| (*v, *o)).collect();
    for (victim, (owner, until)) in flights {
        let Some(at) = pos(g, p, victim) else {
            g.smoko.flights.remove(&victim);
            continue;
        };
        if g.now > until || (!race && !down(g, victim)) || body(g, owner).is_none() {
            g.smoko.flights.remove(&victim);
            sync_person(g, p, victim, false);
            continue;
        }
        if !race && in_pad(at) {
            deliver(g, p, owner, victim);
        }
    }
    // Resolve disconnected carriers, expired records and out-of-bounds victims without scoring.
    g.smoko.immune.retain(|_, t| *t > g.now);
    let victims: Vec<_> = g.smoko.flights.keys().copied().collect();
    for v in victims {
        if !race && pos(g, p, v).is_some_and(|a| a.y < -4.0 || a.x.abs() > 34.0 || a.z.abs() > 25.0) {
            g.smoko.flights.remove(&v);
            set_pose(g, p, v, V3::new(20.0, 0.0, -10.0), V3::ZERO, false);
            g.smoko.immune.insert(v, g.now + PROTECTION);
            sync_person(g, p, v, false);
        }
    }
    if g.now - g.smoko.last_sync >= 1.0 / 15.0 {
        g.smoko.last_sync = g.now;
        let ids: Vec<_> = g
            .dummies
            .iter()
            .filter(|d| d.remote.is_some())
            .map(|d| d.id)
            .collect();
        for id in ids {
            if controlled(g, id) {
                sync_person(g, p, id, true);
            }
        }
    }
    if race {return;}
    g.prompt = if player_locked(g) {
        "Being carried! A mate can interrupt the carrier; you escape after 8 seconds.".into()
    } else if g.smoko.carries.contains_key(&PLAYER_ID) {
        "Take them to SMOKO • Tap F: drop • Hold F, release: throw".into()
    } else {
        "SMOKO: Knock someone down → F to pick up → deliver to the marked pad (+100)".into()
    };
}

/// Presentation only. Guests use replicated carry state; never predict points or recovery timers.
pub fn hud_prompt(g: &Game, p: &Player) -> String {
    if player_locked(g) {
        return "Being carried — another player can knock your carrier down to free you".into();
    }
    let carry = g.smoko.carries.get(&PLAYER_ID);
    if carry.is_some() || (g.mirror && g.smoko.guest_carrying) {
        let distance = (p.mover.x - CENTRE.0).hypot(p.mover.z - CENTRE.1);
        let remaining = carry.filter(|_| !g.mirror)
            .map(|c| format!(" • {:.0}s left", (MAX_CARRY - (g.now - c.since)).max(0.0).ceil()))
            .unwrap_or_default();
        return format!("SMOKO shelter: {:.0}m{} • Tap F: drop • Hold F, release: throw", distance.ceil(), remaining);
    }
    if !g.mirror && let Some(until) = g.smoko.immune.get(&PLAYER_ID).filter(|t| **t > g.now) {
        return format!("Back in play • Pickup protection: {:.0}s • Deliver opponents to SMOKO (+100)", (until-g.now).ceil());
    }
    if let Some(victim) = pickup_target(g, p, PLAYER_ID) {
        return format!("F: pick up {} • Carry or throw them into SMOKO (+100)", crate::round::name_of(g, victim));
    }
    "SMOKO: knock someone down → move close and press F → deliver to the shelter (+100)".into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;
    fn fixture() -> (App, Player) {
        let mut a = App::new();
        a.add_plugins(MinimalPlugins);
        a.add_plugins(crate::game::GamePlugin);
        let mut g = a.world_mut().resource_mut::<Game>();
        g.rules.mode = GameMode::Smoko;
        g.options.bots_on = false;
        g.dummies[0].mover = Mover::new(0.0, 5.5);
        g.dummies[0].body.down_t = 4.0;
        let p = Player {
            mover: Mover::new(0.0, 4.0),
            prev: Vec3::ZERO,
            yaw: std::f32::consts::PI,
            pitch: 0.0,
            walk: 0.0,
            shake: 0.0,
            fov_base: 85.0,
            fov: 85.0,
            rng: bbq_core::rng::Rng::new(1),
        };
        (a, p)
    }
    #[test]
    fn contextual_hints_follow_eligibility_carry_and_guest_authority() {
        let (mut app, mut p) = fixture();
        let mut g = app.world_mut().resource_mut::<Game>();
        g.rules.phase = bbq_core::scoring::Phase::Play;
        assert!(hud_prompt(&g, &p).starts_with("F: pick up"));
        p.mover.x = 20.0;
        assert!(!hud_prompt(&g, &p).starts_with("F: pick up"));
        p.mover.x = 0.0;
        grab(&mut g, &mut p, PLAYER_ID);
        assert!(hud_prompt(&g, &p).contains("8s left"));
        g.now += 3.2;
        assert!(hud_prompt(&g, &p).contains("5s left"));
        g.mirror = true;
        g.smoko.carries.clear();
        g.smoko.guest_carrying = true;
        let hint = hud_prompt(&g, &p);
        assert!(hint.contains("SMOKO shelter:"));
        assert!(!hint.contains("s left"));
        g.smoko.guest_carrying = false;
        g.smoko.guest_until = g.now + 1.0;
        assert!(hud_prompt(&g, &p).starts_with("Being carried"));
    }

    fn race_fixture()->(App,Player) {
        let (mut a,p)=fixture();
        {let mut g=a.world_mut().resource_mut::<Game>();g.rules.mode=GameMode::Race;g.dummies[0].body=Body::default();
            g.race.racers.insert(PLAYER_ID,crate::race::Progress::new((p.mover.x,p.mover.z)));
            g.race.racers.insert(100,crate::race::Progress::new((0.,5.5)));}
        (a,p)
    }
    #[test]
    fn race_pickup_drop_throw_protection_and_no_delivery_points() {
        let (mut a,mut p)=race_fixture();let mut g=a.world_mut().resource_mut::<Game>();
        input(&mut g,&mut p,PLAYER_ID,true);assert_eq!(g.smoko.carries[&PLAYER_ID].victim,100);
        input(&mut g,&mut p,100,true);assert_eq!(g.smoko.carries.len(),1);
        step(&mut g,&mut p,&mut Wanted::default());
        input(&mut g,&mut p,PLAYER_ID,true);g.now+=0.4;input(&mut g,&mut p,PLAYER_ID,false);
        assert!(g.smoko.carries.is_empty());assert!(g.dummies[0].mover.vz>9.0);
        assert!(g.race.racers[&100].return_to.is_some());
        g.dummies[0].mover=Mover::new(0.,5.5);g.dummies[0].body=Body::default();
        input(&mut g,&mut p,PLAYER_ID,true);assert!(g.smoko.carries.is_empty());
        g.now+=RACE_RELEASE_PROTECTION+0.1;step(&mut g,&mut p,&mut Wanted::default());
        input(&mut g,&mut p,PLAYER_ID,true);assert_eq!(g.smoko.carries.len(),1);
        p.mover=Mover::new(0.,9.);step(&mut g,&mut p,&mut Wanted::default());
        assert_eq!(g.board.score(PLAYER_ID),0,"Smoko pad must not score in Race");
        g.now+=RACE_MAX_CARRY+0.1;step(&mut g,&mut p,&mut Wanted::default());
        assert!(g.smoko.carries.is_empty());assert!(g.dummies[0].dragged.is_none());
    }
    #[test]
    fn race_pickup_rejects_throwers_finished_airborne_and_remote_mirrors() {
        for invalid in 0..4 {let (mut a,mut p)=race_fixture();let mut g=a.world_mut().resource_mut::<Game>();
            match invalid {0=>{g.race.racers.remove(&100);},1=>g.race.racers.get_mut(&100).unwrap().finish=Some(1.0),2=>g.dummies[0].mover.y=2.0,_=>g.mirror=true}
            input(&mut g,&mut p,PLAYER_ID,true);assert!(g.smoko.carries.is_empty());
        }
    }
    #[test]
    fn race_disconnect_and_round_end_release_everyone() {
        for end in [false,true] {let (mut a,mut p)=race_fixture();let mut g=a.world_mut().resource_mut::<Game>();
            let now=g.now;g.smoko.carries.insert(100,Carry{victim:PLAYER_ID,since:now});
            if end {g.rules.phase=bbq_core::scoring::Phase::Results;}else{g.dummies.retain(|d|d.id!=100);}
            step(&mut g,&mut p,&mut Wanted::default());assert!(g.smoko.carries.is_empty());assert!(!player_locked(&g));
        }
    }
    #[test]
    fn longer_knockdown_allows_late_pickup_without_repeat_hit_refresh() {
        let (mut a, mut p) = fixture();
        let mut g = a.world_mut().resource_mut::<Game>();
        let b = &mut g.dummies[0].body;
        *b = Body::default();
        let hit = b.apply_hit(2.0, Some(2.0));
        extend_knockdown(GameMode::Smoko, b, hit.knocked_down);
        b.tick(5.0);
        let repeat = b.apply_hit(2.0, Some(2.0));
        extend_knockdown(GameMode::Smoko, b, repeat.knocked_down);
        assert_eq!(b.down_t, 3.0);
        input(&mut g, &mut p, PLAYER_ID, true);
        assert_eq!(g.smoko.carries[&PLAYER_ID].victim, 100);
        p.mover = Mover::new(0.0, 9.0);
        step(&mut g, &mut p, &mut Wanted::default());
        assert_eq!(g.board.score(PLAYER_ID), 100);
        let mut b = Body::default();
        b.apply_hit(8.0, Some(knockdown_time(GameMode::Smoko, 3.5)));
        b.tick(8.0);
        assert!(!b.is_down());
        assert_eq!(knockdown_time(GameMode::FreeForAll, 3.5), 3.5);
    }
    #[test]
    fn walk_in_is_not_a_delivery_and_hits_do_not_score() {
        let (mut a, mut p) = fixture();
        let mut g = a.world_mut().resource_mut::<Game>();
        p.mover = Mover::new(CENTRE.0, CENTRE.1);
        g.dummies[0].mover = Mover::new(CENTRE.0, CENTRE.1);
        step(&mut g, &mut p, &mut Wanted::default());
        assert_eq!(g.board.score(PLAYER_ID), 0);
        let r = g.rules;
        assert_eq!(g.board.award(&r, PLAYER_ID, 100), 0);
        assert_eq!(g.board.award_dazza_slap(&r, PLAYER_ID, 0), 0);
    }
    #[test]
    fn carried_delivery_scores_once_respawns_and_protects() {
        let (mut a, mut p) = fixture();
        let mut g = a.world_mut().resource_mut::<Game>();
        g.dummies[0].body.wet_t = 4.0;
        input(&mut g, &mut p, PLAYER_ID, true);
        assert_eq!(g.smoko.carries[&PLAYER_ID].victim, 100);
        p.mover = Mover::new(0.0, 9.0);
        step(&mut g, &mut p, &mut Wanted::default());
        assert_eq!(g.board.score(PLAYER_ID), 100);
        assert!(g.smoko.carries.is_empty());
        assert_eq!(g.dummies[0].body.wet_t,0.0);
        assert!(!in_pad(pos(&g, &p, 100).unwrap()));
        for _ in 0..20 {
            step(&mut g, &mut p, &mut Wanted::default());
        }
        assert_eq!(g.board.score(PLAYER_ID), 100);
        g.dummies[0].mover = Mover::new(0.0, 10.0);
        g.dummies[0].body.down_t = 4.0;
        input(&mut g, &mut p, PLAYER_ID, true);
        assert!(g.smoko.carries.is_empty());
    }
    #[test]
    fn thrown_delivery_has_one_owner_and_expired_or_recovered_throws_do_not_pay() {
        for expires in [false, true] {
            let (mut a, mut p) = fixture();
            let mut g = a.world_mut().resource_mut::<Game>();
            input(&mut g, &mut p, PLAYER_ID, true);
            step(&mut g, &mut p, &mut Wanted::default());
            input(&mut g, &mut p, PLAYER_ID, true);
            g.now += 0.4;
            input(&mut g, &mut p, PLAYER_ID, false);
            assert!(g.dummies[0].mover.vz > 9.0);
            if expires {
                g.now += THROW_WINDOW + 0.1;
            }
            g.dummies[0].mover = Mover::new(0.0, 10.0);
            step(&mut g, &mut p, &mut Wanted::default());
            assert_eq!(g.board.score(PLAYER_ID), if expires { 0 } else { 100 });
        }
        let (mut a, mut p) = fixture();
        let mut g = a.world_mut().resource_mut::<Game>();
        let until = g.now + 4.0;
        g.smoko.flights.insert(100, (PLAYER_ID, until));
        g.dummies[0].body = Body::default();
        g.dummies[0].mover = Mover::new(0.0, 10.0);
        step(&mut g, &mut p, &mut Wanted::default());
        assert_eq!(g.board.score(PLAYER_ID), 0);
    }
    #[test]
    fn interrupted_carrier_recovery_and_round_end_drop_without_scoring() {
        for end in [false, true] {
            let (mut a, mut p) = fixture();
            let mut g = a.world_mut().resource_mut::<Game>();
            input(&mut g, &mut p, PLAYER_ID, true);
            if end {
                g.rules.phase = bbq_core::scoring::Phase::Results;
            } else {
                g.me.body.stun = 1.0;
            }
            step(&mut g, &mut p, &mut Wanted::default());
            assert!(g.smoko.carries.is_empty() && g.smoko.flights.is_empty());
            assert_eq!(g.board.score(PLAYER_ID), 0);
            assert!(g.dummies[0].dragged.is_none());
        }
    }
    #[test]
    fn bot_can_deliver_the_human_and_gets_the_points() {
        let (mut a, mut p) = fixture();
        let mut g = a.world_mut().resource_mut::<Game>();
        g.dummies[0].body = Body::default();
        g.me.body.down_t = 4.0;
        input(&mut g, &mut p, 100, true);
        assert_eq!(g.smoko.carries[&100].victim, PLAYER_ID);
        assert_eq!(bot_goal(&g, &p, 100), Some(V3::new(0.0, 0.0, 10.0)));
        g.dummies[0].mover = Mover::new(0.0, 9.0);
        g.dummies[0].face = 0.0;
        step(&mut g, &mut p, &mut Wanted::default());
        assert_eq!(g.board.score(100), 100);
        assert_eq!(g.board.score(PLAYER_ID), 0);
        assert!(!in_pad(pos(&g, &p, PLAYER_ID).unwrap()));
    }
    #[test]
    fn disconnected_owner_cannot_score_and_victim_is_released() {
        let (mut a, mut p) = fixture();
        let mut g = a.world_mut().resource_mut::<Game>();
        g.smoko.carries.insert(
            999,
            Carry {
                victim: 100,
                since: 0.0,
            },
        );
        step(&mut g, &mut p, &mut Wanted::default());
        assert!(g.smoko.carries.is_empty());
        assert_eq!(g.board.score(999), 0);
    }
}
