//! Host-owned race and sideline game. Throws resolve in launch order, never collision order.
use crate::{
    game::Game,
    player::{PLAYER_ID, Player, Wanted},
    yard_scene::YardRes,
};
use bbq_core::movement::{Modifiers, MoveInput, Mover};
use bbq_core::{GameMode, PlayerId};
use bbq_core::{
    hands,
    hitting::{Context, Target},
    items::ItemKind,
    itemworld::WorldEvent,
    vec::V3,
};
use std::collections::{BTreeMap, VecDeque};
pub const REPLENISH: f32 = 1.4;
pub const MAX_AIRBORNE: usize = 3;
pub const THROW_TIMEOUT: f32 = 6.0;
pub const UNLOCKS: [u32; 3] = [225, 500, 900];
#[derive(Clone, Debug)]
pub struct Attempt {
    pub item: u32,
    pub at: f32,
    pub hit: Option<bool>,
}
#[derive(Clone, Debug, Default)]
pub struct Thrower {
    pub score: u32,
    pub streak: u32,
    pub hits: u32,
    pub ready_at: f32,
    pub refill_at: f32,
    pub attempts: VecDeque<Attempt>,
    pub disconnected: bool,
}
impl Thrower {
    pub fn level(&self) -> usize {
        UNLOCKS.iter().filter(|v| self.score >= **v).count()
    }
    pub fn next(&self) -> u32 {
        50u32.saturating_add(self.streak.saturating_mul(25))
    }
    pub fn resolve(&mut self) -> u32 {
        let mut gain = 0;
        while let Some(hit) = self.attempts.front().and_then(|a| a.hit) {
            self.attempts.pop_front();
            if hit {
                let reward = self.next();
                self.score = self.score.saturating_add(reward);
                gain += reward;
                self.streak += 1;
                self.hits += 1;
            } else {
                self.streak = 0;
            }
        }
        gain
    }
}
pub fn bot_count(g: &Game, role: Role) -> usize {
    let people = usize::from(role == Role::Racer)
        + g.remotes
            .iter()
            .filter(|r| {
                g.race
                    .choices
                    .get(&r.local_id())
                    .copied()
                    .unwrap_or(Role::Racer)
                    == Role::Racer
            })
            .count();
    3usize.saturating_sub(people.min(3)) + 2
}

pub const LAPS: usize = 3;
pub const LIMIT: f32 = 300.0;
pub const COURSE_SCALE: f32 = 3.0;
// Oval checkpoint centres; the driving surface follows the smooth ellipse between them.
pub const TRACK: [(f32, f32); 8] = [
    (0., -24.),
    (25.455843, -16.970562),
    (36., 0.),
    (25.455843, 16.970562),
    (0., 24.),
    (-25.455843, 16.970562),
    (-36., 0.),
    (-25.455843, -16.970562),
];
pub fn oval(t: f32) -> (f32, f32) {
    (12.0 * COURSE_SCALE * t.sin(), -8.0 * COURSE_SCALE * t.cos())
}
pub const WIDTH: f32 = 4.8;
/// Grace verge, grass pace and permitted re-entry distance are metres / speed fractions.
pub const VERGE: f32 = 0.6;
pub const GRASS_SPEED: f32 = 0.55;
pub const REJOIN_RADIUS: f32 = 3.0;
pub const TUG_REACH: f32 = 1.65;
pub const TUG_SPEED: f32 = 0.65;
pub const TUG_TIME: f32 = 1.25;
pub const TUG_COOLDOWN: f32 = 5.0;
pub const TUG_IMMUNITY: f32 = 6.0;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Role {
    #[default]
    Racer,
    Thrower,
}
#[derive(Clone, Debug)]
pub struct Progress {
    pub passed: usize,
    pub finish: Option<f32>,
    pub previous: (f32, f32),
    pub disconnected: bool,
    pub protected_until: f32,
    pub return_to: Option<(f32,f32)>,
    pub tug_until: f32,
    pub tug_immune_until: f32,
}
#[derive(Default)]
pub struct State {
    pub racers: BTreeMap<PlayerId, Progress>,
    pub elapsed: f32,
    pub throwers: BTreeMap<PlayerId, Thrower>,
    pub choices: BTreeMap<PlayerId, Role>,
    pub tug_ready: BTreeMap<PlayerId,f32>,
}
fn distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    (a.0 - b.0).hypot(a.1 - b.1)
}
fn segment(p: (f32, f32), a: (f32, f32), b: (f32, f32)) -> f32 {
    let v = (b.0 - a.0, b.1 - a.1);
    let t = (((p.0 - a.0) * v.0 + (p.1 - a.1) * v.1) / (v.0 * v.0 + v.1 * v.1)).clamp(0., 1.);
    distance(p, (a.0 + v.0 * t, a.1 + v.1 * t))
}
pub fn track_distance(p: (f32, f32)) -> f32 {
    (0..64)
        .map(|i| {
            segment(
                p,
                oval(i as f32 * std::f32::consts::TAU / 64.),
                oval((i + 1) as f32 * std::f32::consts::TAU / 64.),
            )
        })
        .fold(f32::INFINITY, f32::min)
}
impl Progress {
    pub fn new(at: (f32, f32)) -> Self {
        Self {
            passed: 0,
            finish: None,
            previous: at,
            disconnected: false,
            protected_until: 0.0,
            return_to: None, tug_until: 0.0, tug_immune_until: 0.0,
        }
    }
    pub fn target(&self) -> (f32, f32) {
        TRACK[(self.passed + 1) % 8]
    }
    pub fn update(&mut self, at: (f32, f32), time: f32) -> bool {
        if self.finish.is_some() || self.disconnected {
            return false;
        }
        // Leaving the verge suspends progress until returning near the departure point.
        // No ordinary grass teleport. A shortcut, carry or throw cannot earn a gate.
        if track_distance(at) > WIDTH * 0.5 + VERGE || distance(at, self.previous) > 2.0 {
            self.return_to.get_or_insert(self.previous);
        }
        if let Some(back) = self.return_to {
            if track_distance(at) <= WIDTH * 0.5 + VERGE && distance(at,back) < REJOIN_RADIUS {
                self.return_to = None;
            }
            self.previous = at;
            return false;
        }
        let target = self.target();
        if distance(at, target) < 1.3 && distance(self.previous, target) > distance(at, target) {
            self.passed += 1;
            if self.passed == LAPS * 8 {
                self.finish = Some(time);
            }
        }
        self.previous = at;
        false
    }
}
pub fn start(g: &mut Game, p: &mut Player, y: &mut YardRes) {
    let choices = std::mem::take(&mut g.race.choices);
    g.race = State {
        choices,
        ..Default::default()
    };
    if g.rules.mode != GameMode::Race {
        return;
    }
    y.0.colliders.clear();
    y.0.race = true;
    y.0.features = bbq_core::yard::Features {
        bar: false,
        bbq: false,
        chest: false,
        smoko: false,
    };
    g.world.clear();
    g.slots.clear();
    g.round.setup.round_len = LIMIT;
    let racer = g.round.setup.race_role == Role::Racer;
    let mut ids = Vec::new();
    if racer {
        ids.push(PLAYER_ID);
    }
    for d in &g.dummies {
        if d.remote.is_some()
            && ids.len() < 3
            && g.race.choices.get(&d.id).copied().unwrap_or(Role::Racer) == Role::Racer
        {
            ids.push(d.id);
        }
    }
    for d in &g.dummies {
        if d.remote.is_none() && ids.len() < 3 {
            ids.push(d.id);
        }
    }
    let mut throwers = vec![PLAYER_ID];
    throwers.extend(g.dummies.iter().map(|d| d.id));
    for id in throwers.into_iter().filter(|id| !ids.contains(id)) {
        g.race.throwers.insert(id, Thrower::default());
    }
    for (i, id) in ids.iter().copied().enumerate() {
        let at = (-0.5 - i as f32 * 0.8, -24.0 + (i as f32 - 1.0) * 0.7);
        g.race.racers.insert(id, Progress::new(at));
        if id == PLAYER_ID {
            p.mover = Mover::new(at.0, at.1);
            p.yaw = -std::f32::consts::FRAC_PI_2;
        } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
            d.mover = Mover::new(at.0, at.1);
            d.home = at;
            d.face = std::f32::consts::FRAC_PI_2;
        }
    }
    for (i, id) in g
        .race
        .throwers
        .keys()
        .copied()
        .collect::<Vec<_>>()
        .into_iter()
        .enumerate()
    {
        let at = terrace(-0.7 + (i % 5) as f32 * 0.35);
        if id == PLAYER_ID {
            p.mover = Mover::new(at.0, at.1);
            p.yaw = std::f32::consts::PI;
            p.pitch = 0.05;
        } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
            d.mover = Mover::new(at.0, at.1);
            d.home = at;
        }
    }
    p.prev = bevy::prelude::Vec3::new(p.mover.x, 0., p.mover.z);
}
pub fn standings(g: &Game) -> Vec<PlayerId> {
    let mut ids: Vec<_> = g.race.racers.keys().copied().collect();
    ids.sort_by(|a, b| {
        let a = &g.race.racers[a];
        let b = &g.race.racers[b];
        a.disconnected
            .cmp(&b.disconnected)
            .then_with(|| match (a.finish, b.finish) {
                (Some(a), Some(b)) => a.total_cmp(&b),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => b.passed.cmp(&a.passed).then_with(|| {
                    distance(a.return_to.unwrap_or(a.previous), a.target()).total_cmp(&distance(b.return_to.unwrap_or(b.previous), b.target()))
                }),
            })
    });
    ids
}
pub fn lines(g: &Game) -> Vec<String> {
    standings(g)
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let r = &g.race.racers[id];
            let status = if r.disconnected {
                "Disconnected".into()
            } else if let Some(t) = r.finish {
                format!("{t:.2}s • FINISHED")
            } else {
                format!(
                    "Lap {}/{} • Gate {}/8",
                    (r.passed / 8 + 1).min(LAPS),
                    LAPS,
                    r.passed % 8 + 1
                )
            };
            format!("{}. {} — {}", i + 1, crate::round::name_of(g, *id), status)
        })
        .collect()
}
pub fn step(g: &mut Game, p: &mut Player, w: &mut Wanted, y: &YardRes) {
    let dt = bbq_core::sim::TICK_DT;
    if g.rules.in_play() {
        g.race.elapsed += dt;
    }
    let held: Vec<_> = g.smoko.carries.values().map(|c|c.victim).collect();
    let controlled: Vec<_> = g.dummies.iter().filter(|d|crate::smoko_mode::controlled(g,d.id)).map(|d|d.id).collect();
    for d in &mut g.dummies {
        let Some(r) = g.race.racers.get(&d.id) else {
            continue;
        };
        if held.contains(&d.id) {continue;}
        if d.remote.is_some() && !controlled.contains(&d.id) {
            if d.mover.boost_t > 0. {
                d.mover.boost_t = (d.mover.boost_t - dt).max(0.);
                if d.mover.boost_t == 0. {
                    d.mover.boost_cd = bbq_core::movement::BOOST_CD;
                }
            } else {
                d.mover.boost_cd = (d.mover.boost_cd - dt).max(0.);
            }
            continue;
        }
        let at = (d.mover.x, d.mover.z);
        // Follow the oval with a short look-ahead, never a chord across the infield.
        let sector = (r.passed % 8) as f32 * std::f32::consts::TAU / 8.;
        let mut angle = (at.0 / 36.)
            .atan2(-at.1 / 24.)
            .rem_euclid(std::f32::consts::TAU);
        if angle > sector + std::f32::consts::PI {
            angle -= std::f32::consts::TAU;
        }
        let aim = (angle + 0.10).min(sector + std::f32::consts::TAU / 8.);
        let centre = oval(aim);
        let tangent = (36. * aim.cos(), 24. * aim.sin());
        let len = tangent.0.hypot(tangent.1);
        let lane = (d.id % 3) as f32 - 1.;
        let target = r.return_to.unwrap_or((
            centre.0 + tangent.1 / len * lane * 0.75,
            centre.1 - tangent.0 / len * lane * 0.75,
        ));
        let v = (target.0 - at.0, target.1 - at.1);
        let len = v.0.hypot(v.1).max(0.001);
        let running = g.rules.in_play() && r.finish.is_none() && g.options.bots_on;
        d.bot.wish = if running {
            (v.0 / len, v.1 / len)
        } else {
            (0., 0.)
        };
        d.face = v.0.atan2(v.1);
        d.mover.step(
            dt,
            MoveInput {
                wish: d.bot.wish,
                ..Default::default()
            },
            &Modifiers {
                frozen: !running && !controlled.contains(&d.id),
                stunned: d.body.stun > 0.0,
                carrying: g.smoko.carries.contains_key(&d.id),
                gait: pace(r, at, g.now, g.race.elapsed),
                wet: d.body.wet_t > 0.0,
                ..Default::default()
            },
            &y.0,
        );
    }
    constrain_throwers(g, p);
    combat(g, p, w, y);
    if std::mem::take(&mut w.interact_pressed) { tug(g,p,PLAYER_ID); }
    crate::smoko_mode::step(g,p,w);
    let held: Vec<_> = g.race.racers.keys().copied().filter(|id|crate::smoko_mode::controlled(g,*id)).collect();
    if g.rules.in_play() {
        for (id, r) in &mut g.race.racers {
            let m = if *id == PLAYER_ID {
                &mut p.mover
            } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == *id) {
                &mut d.mover
            } else {
                r.disconnected = true;
                continue;
            };
            if held.contains(id) {r.return_to.get_or_insert(r.previous);} else {r.update((m.x,m.z),g.race.elapsed);}
            if m.y < -4.0 || m.x.abs()>51.0 || m.z.abs()>36.0 {
                let at = TRACK[r.passed % 8];
                *m = Mover::new(at.0, at.1);
                r.previous = at;
                r.return_to = None;
                r.protected_until = g.race.elapsed + 2.0;
            }
        }
        if g.race
            .racers
            .values()
            .all(|r| r.finish.is_some() || r.disconnected)
        {
            crate::round::end_round(g);
        }
    }
    g.prompt = prompt(g);
    if let Some(v)=crate::smoko_mode::pickup_target(g,p,PLAYER_ID) {g.prompt.push_str(&format!(" • F: pick up {}",crate::round::name_of(g,v)));}
    *w = Wanted {
        wish: w.wish,
        dive: w.dive,
        ..Default::default()
    };
}
pub fn terrace(angle: f32) -> (f32, f32) {
    (42. * angle.sin(), -28. * angle.cos())
}
fn constrain_throwers(g: &mut Game, p: &mut Player) {
    for id in g.race.throwers.keys() {
        let m = if *id == PLAYER_ID {
            &mut p.mover
        } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == *id) {
            &mut d.mover
        } else {
            continue;
        };
        let x = m.x / 42.;
        let z = -m.z / 28.;
        let a = x.atan2(z).clamp(-0.85, 0.85);
        let r = x.hypot(z).clamp(1., 1.12);
        m.x = 42. * r * a.sin();
        m.z = -28. * r * a.cos();
    }
}
pub fn prompt(g: &Game) -> String {
    if let Some(t) = g.race.throwers.get(&PLAYER_ID) {
        let upgrade = match t.level() {
            0 => "225: faster refill",
            1 => "500: quicker recovery",
            2 => "900: teddy unlocked",
            _ => "All upgrades unlocked • Q: change weapon",
        };
        format!(
            "THROWER • {} pts • Streak {} • Next +{} • {} • Hold/release LMB",
            t.score,
            t.streak,
            t.next(),
            upgrade
        )
    } else if crate::smoko_mode::player_locked(g) {
        "CARRIED • Escape automatically after 3 seconds; rivals can interrupt the carrier".into()
    } else if g.smoko.carries.contains_key(&PLAYER_ID) || g.smoko.guest_carrying {
        "CARRYING • Tap F: drop • Hold F, release: throw • 3-second limit".into()
    } else if let Some(r) = g.race.racers.get(&PLAYER_ID) {
        r.finish.map_or_else(
            || {
                format!(
                    "RACER • Lap {}/{} • Gate {} → • F: pick up • R: shorts tug",
                    (r.passed / 8 + 1).min(LAPS),
                    LAPS,
                    r.passed % 8 + 1
                ) + &r.return_to.map_or(String::new(), |(x,z)|format!(" • RETURN to gold ring: {:.0}m",distance(r.previous,(x,z))))
                  + if r.tug_until>g.now {" • SHORTS TUG! Slowed briefly"} else {""}
            },
            |t| format!("FINISHED in {t:.2}s • Waiting for the other racers"),
        )
    } else {
        "Race in progress • Join the next round".into()
    }
}
pub fn thrower_lines(g: &Game) -> Vec<String> {
    let mut ids: Vec<_> = g.race.throwers.iter().collect();
    ids.sort_by_key(|(id, t)| (t.disconnected, std::cmp::Reverse(t.score), **id));
    ids.into_iter()
        .enumerate()
        .map(|(i, (id, t))| {
            format!(
                "{}. {} — {} pts • streak {}{}",
                i + 1,
                crate::round::name_of(g, *id),
                t.score,
                t.streak,
                if t.disconnected { " • left" } else { "" }
            )
        })
        .collect()
}
pub fn all_lines(g: &Game) -> Vec<String> {
    let mut l = lines(g);
    l.push("SIDELINE THROWERS".into());
    l.extend(thrower_lines(g));
    l
}
pub fn thrower_winner(g: &Game) -> String {
    let best = g
        .race
        .throwers
        .values()
        .filter(|t| !t.disconnected)
        .map(|t| t.score)
        .max()
        .unwrap_or(0);
    if best == 0 {
        return "No throwing winner".into();
    }
    let names: Vec<_> = g
        .race
        .throwers
        .iter()
        .filter(|(_, t)| !t.disconnected && t.score == best)
        .map(|(id, _)| crate::round::name_of(g, *id))
        .collect();
    format!(
        "Sideline winner{}: {} ({best} pts)",
        if names.len() > 1 { "s" } else { "" },
        names.join(" / ")
    )
}
pub fn launch(g: &mut Game, id: PlayerId, item: u32, start: V3, vel: V3, charge: f32) -> bool {
    if !g.rules.in_play() {
        return false;
    }
    let Some(t) = g.race.throwers.get(&id) else {
        return false;
    };
    if t.disconnected || g.now < t.ready_at || t.attempts.len() >= MAX_AIRBORNE {
        return false;
    }
    let Some(it) = g.world.items.get(&item) else {
        return false;
    };
    if !matches!(it.kind, ItemKind::WaterBalloon | ItemKind::Teddy)
        || vel.len() > it.kind.def().speed + 3.0
        || !vel.len().is_finite()
    {
        return false;
    }
    if it.kind == ItemKind::Teddy && t.level() < 3 {
        return false;
    }
    if !g.world.throw(item, id, start, vel, charge, false) {
        return false;
    }
    let t = g.race.throwers.get_mut(&id).unwrap();
    t.ready_at = g.now + if t.level() >= 2 { 0.35 } else { 0.55 };
    t.refill_at = g.now + if t.level() >= 1 { 1.0 } else { REPLENISH };
    t.attempts.push_back(Attempt {
        item,
        at: g.now,
        hit: None,
    });
    if id == PLAYER_ID {
        g.slots.remove(item);
    } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
        d.bot.slots.remove(item);
        d.anim.feel.weight=g.world.items.get(&item).map_or(0.5, |it| bbq_core::feel::weight(it.kind));
        d.anim.start_swing();
    }
    true
}
fn combat(g: &mut Game, p: &mut Player, w: &mut Wanted, y: &YardRes) {
    let dt = bbq_core::sim::TICK_DT;
    g.me.body.tick(dt);
    for d in &mut g.dummies {
        d.body.tick(dt);
    }
    for pop in &mut g.popups {
        pop.t += dt;
    }
    g.popups.retain(|p| p.t < 1.6);
    if !g.rules.in_play() {
        g.wind.cancel();
        return;
    }
    let ids: Vec<_> = g.race.throwers.keys().copied().collect();
    for id in ids {
        let level = g.race.throwers[&id].level();
        let (held, at) = if id == PLAYER_ID {
            (g.slots.len(), (p.mover.x, p.mover.z))
        } else if let Some(d) = g.dummies.iter().find(|d| d.id == id) {
            (d.bot.slots.len(), (d.mover.x, d.mover.z))
        } else {
            g.race.throwers.get_mut(&id).unwrap().disconnected = true;
            continue;
        };
        // Refill is tied to last launch, and supplies never exceed two held items.
        let t = &g.race.throwers[&id];
        if held < if level >= 3 { 2 } else { 1 } && g.now >= t.refill_at {
            let kind = if level >= 3 && held == 1 {
                ItemKind::Teddy
            } else {
                ItemKind::WaterBalloon
            };
            let item = g.world.spawn(kind, at.0, at.1, false, &mut g.rng);
            g.world.give(item, id);
            if id == PLAYER_ID {
                g.slots.add(item);
            } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
                d.bot.slots.add(item);
            }
        }
    }
    if g.race.throwers.contains_key(&PLAYER_ID) {
        if w.swap != 0 {
            g.slots.swap(w.swap);
            g.wind.cancel();
        }
        if let Some(n) = w.slot {
            g.slots.select_slot(n);
            g.wind.cancel();
        }
        let kind = g
            .slots
            .selected()
            .and_then(|id| g.world.items.get(&id))
            .map(|it| it.kind);
        if w.throw_down {
            g.wind.press(
                g.now,
                kind,
                &hands::Situation {
                    can_act: true,
                    ..Default::default()
                },
            );
        }
        if g.wind.tick(dt, kind) {
            g.say("Charge released — hold again to throw");
        }
        if w.throw_up && g.wind.charging {
            let charge = g.wind.charge;
            g.wind.cancel();
            if let (Some(item), Some(kind)) = (g.slots.selected(), kind) {
                let vel = hands::throw_velocity(
                    crate::game::aim_dir(p.yaw, p.pitch),
                    kind.def().speed,
                    charge,
                    V3::new(p.mover.vx, 0., p.mover.vz),
                );
                launch(g, PLAYER_ID, item, crate::game::hand_pos(p), vel, charge);
            }
        }
    }
    // Bots use the same supply, cooldown, projectiles and ordered scoring as humans.
    let bots: Vec<_> = g
        .dummies
        .iter()
        .filter(|d| d.remote.is_none() && g.race.throwers.contains_key(&d.id))
        .map(|d| (d.id, d.mover.x, d.mover.z, d.bot.slots.selected()))
        .collect();
    if g.options.bots_on {
        for (id, x, z, item) in bots {
            let Some(item) = item else { continue };
            let mut targets = Vec::new();
            for (rid, r) in &g.race.racers {
                if r.finish.is_some() || r.disconnected {
                    continue;
                }
                let m = if *rid == PLAYER_ID {
                    &p.mover
                } else if let Some(d) = g.dummies.iter().find(|d| d.id == *rid) {
                    &d.mover
                } else {
                    continue;
                };
                let dist = (m.x - x).hypot(m.z - z);
                if dist < 18. {
                    targets.push((dist, m.x, m.z, m.vx, m.vz));
                }
            }
            targets.sort_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((dist, tx, tz, vx, vz)) = targets.first().copied() {
                let t = (dist / 15.).clamp(0.3, 1.0);
                let spread = g.rng.range(-0.55, 0.55);
                let speed = g.world.items[&item].kind.def().speed;
                let vel = V3::new(
                    (tx + vx * t - x + spread) / t,
                    0.5 * bbq_core::flight::ITEM_GRAV * g.world.items[&item].kind.def().grav * t,
                    (tz + vz * t - z) / t,
                );
                if vel.len() <= speed + 3. {
                    launch(g, id, item, V3::new(x, 1.2, z), vel, 1.0);
                }
            }
        }
    }
    let targets: Vec<_> = g
        .race
        .racers
        .iter()
        .filter(|(_, r)| r.finish.is_none() && !r.disconnected)
        .filter_map(|(id, _)| {
            let m = if *id == PLAYER_ID {
                &p.mover
            } else {
                &g.dummies.iter().find(|d| d.id == *id)?.mover
            };
            Some(Target {
                id: *id,
                pos: V3::new(m.x, m.y, m.z),
                sink: 0.,
                at_smoko: false,
                catching: false,
                stunned: false,
                facing: V3::ZERO,
                flattenable: false,
            })
        })
        .collect();
    let teams = g.teams.clone();
    let events = g.world.step(
        dt,
        &y.0,
        &mut g.rng,
        &targets,
        &Context {
            friendly_fire: true,
            teams: &teams,
        },
        0,
        false,
        &|_, _| true,
    );
    for ev in events {
        match ev {
            WorldEvent::Hit {
                item,
                victim,
                thrower,
                kind,
                dir,
                ..
            } => {
                let valid = g
                    .race
                    .racers
                    .get(&victim)
                    .is_some_and(|r| r.finish.is_none() && !r.disconnected);
                let mut credited = false;
                if let Some(t) = thrower.and_then(|id| g.race.throwers.get_mut(&id))
                    && let Some(a) = t
                        .attempts
                        .iter_mut()
                        .find(|a| a.item == item && a.hit.is_none())
                {
                    a.hit = Some(valid);
                    credited = valid;
                }
                g.world.remove(item);
                if credited {
                    let r = g.race.racers.get_mut(&victim).unwrap();
                    let slow = g.race.elapsed >= r.protected_until;
                    if slow {
                        r.protected_until = g.race.elapsed + 5.5;
                    }
                    let duration = if kind == ItemKind::WaterBalloon {
                        4.0
                    } else {
                        0.8
                    };
                    let at = if victim == PLAYER_ID {
                        if slow {
                            g.me.body.wet_t = duration;
                            p.shake = p.shake.max(0.06);
                        }
                        V3::new(p.mover.x, 1.2, p.mover.z)
                    } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == victim) {
                        d.anim.feel.hit(dir, kind.def().knock);
                        if slow {
                            d.body.wet_t = duration;
                        }
                        V3::new(d.mover.x, 1.2, d.mover.z)
                    } else {
                        continue;
                    };
                    g.fx(
                        if kind == ItemKind::WaterBalloon {
                            crate::fx::FxKind::BalloonBurst
                        } else {
                            crate::fx::FxKind::Hit
                        },
                        at,
                    );
                }
            }
            WorldEvent::Flight {
                ev: bbq_core::flight::FlightEvent::WaterBurst { pos },
                ..
            } => g.fx(crate::fx::FxKind::BalloonBurst, pos),
            _ => {}
        }
    }
    let mut remove = Vec::new();
    let mut gains = Vec::new();
    for (id, t) in &mut g.race.throwers {
        for a in &mut t.attempts {
            if a.hit.is_none()
                && (g.now - a.at >= THROW_TIMEOUT
                    || g.world.items.get(&a.item).is_none_or(|it| !it.live))
            {
                a.hit = Some(false);
                remove.push(a.item);
            }
        }
        let level = t.level();
        let gain = t.resolve();
        if gain > 0 {
            gains.push((*id, gain, t.level() > level));
        }
    }
    for id in remove {
        g.world.remove(id);
    }
    for (id, gain, unlocked) in gains {
        let name = crate::round::name_of(g, id);
        g.say(format!("{name} +{gain} • sideline hit streak"));
        if unlocked {
            g.say(format!("{name}: upgrade unlocked!"));
        }
        if id == PLAYER_ID {
            g.popup(
                if unlocked {
                    format!("+{gain} • UPGRADE UNLOCKED!")
                } else {
                    format!("SPLASH! +{gain}")
                },
                false,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn overlapping_throws_resolve_in_launch_order_and_reset_after_miss() {
        let mut t = Thrower::default();
        for item in 1..=3 {
            t.attempts.push_back(Attempt {
                item,
                at: 0.,
                hit: None,
            });
        }
        t.attempts[2].hit = Some(true);
        assert_eq!(t.resolve(), 0);
        t.attempts[0].hit = Some(true);
        assert_eq!(t.resolve(), 50);
        t.attempts[0].hit = Some(true);
        assert_eq!(t.resolve(), 175);
        assert_eq!((t.score, t.streak, t.next()), (225, 3, 125));
        assert_eq!(t.level(), 1);
        t.attempts.push_back(Attempt {
            item: 4,
            at: 0.,
            hit: Some(false),
        });
        assert_eq!(t.resolve(), 0);
        assert_eq!(t.next(), 50);
        assert_eq!(t.score, 225);
        assert_eq!(t.resolve(), 0, "resolved attempts cannot pay twice");
    }
    #[test]
    fn upgrades_are_threshold_based_and_scores_stay_banked() {
        let mut t = Thrower::default();
        for (score, level) in [(224, 0), (225, 1), (499, 1), (500, 2), (899, 2), (900, 3)] {
            t.score = score;
            assert_eq!(t.level(), level);
        }
    }
    #[test]
    fn grass_keeps_position_and_requires_reentry_without_shortcut_credit() {
        let mut r=Progress::new(TRACK[0]);
        for n in 1..20 {r.update((0.,-24.+n as f32*0.3),n as f32);}
        let back=r.return_to.unwrap();assert_eq!(r.passed,0);
        assert_eq!(r.previous,(0.,-18.3));
        r.update(TRACK[1],21.);assert_eq!(r.passed,0);assert!(r.return_to.is_some());
        r.update(back,22.);assert!(r.return_to.is_none());assert_eq!(r.passed,0);
    }
    #[test]
    fn grass_tug_and_edge_grace_are_bounded() {
        let mut r=Progress::new(TRACK[0]);
        assert_eq!(pace(&r,(0.,-26.8),0.,1.),1.0);
        assert_eq!(pace(&r,(0.,-29.),0.,1.),GRASS_SPEED);
        r.tug_until=1.25;
        assert_eq!(pace(&r,TRACK[0],0.,1.),TUG_SPEED);
        assert_eq!(pace(&r,(0.,-29.),0.,1.),GRASS_SPEED,"penalties do not multiply");
        assert_eq!(pace(&r,TRACK[0],1.25,1.),1.0);
        r.protected_until=3.;assert_eq!(pace(&r,(0.,-29.),2.,2.),1.0);
    }
    #[test]
    fn shortcuts_and_teleports_do_not_advance() {
        let mut r = Progress::new(TRACK[0]);
        assert!(!r.update(TRACK[1], 1.));
        assert!(r.return_to.is_some());
        assert_eq!(r.passed, 0);
        assert!(!r.update((0., 0.), 2.));
        assert_eq!(r.passed, 0);
    }
    #[test]
    fn ordered_three_laps_finish_once() {
        let mut r = Progress::new(TRACK[0]);
        let mut time = 0.;
        for i in 1..=2400 {
            time += 0.01;
            assert!(!r.update(oval(i as f32 * std::f32::consts::TAU / 800.), time));
        }
        assert_eq!(r.passed, 24);
        let finish = r.finish;
        assert!(!r.update(TRACK[0], 999.));
        assert_eq!(r.finish, finish);
    }
    #[test]
    fn backwards_crossing_cannot_finish() {
        let mut r = Progress::new(TRACK[0]);
        for i in (0..8).rev() {
            r.update(TRACK[i], 1.);
        }
        assert!(r.finish.is_none());
    }
}

/// Guests request throws; only the host consumes supplies and scores collisions.
pub fn guest_input(g: &mut Game, p: &mut Player, w: &mut Wanted, dt: f32) {
    g.prompt = prompt(g);
    if std::mem::take(&mut w.grab_pressed) {g.net_acts.push(bbq_core::net_act::GuestAct::Carry{pressed:true});}
    if std::mem::take(&mut w.grab_released) {g.net_acts.push(bbq_core::net_act::GuestAct::Carry{pressed:false});}
    if std::mem::take(&mut w.interact_pressed) {g.net_acts.push(bbq_core::net_act::GuestAct::RaceTug);}
    constrain_throwers(g, p);
    if !g.race.throwers.contains_key(&PLAYER_ID) {
        g.wind.cancel();
        return;
    }
    let before = g.slots.selected();
    if w.swap != 0 {
        g.slots.swap(w.swap);
        g.wind.cancel();
    }
    if let Some(n) = w.slot {
        g.slots.select_slot(n);
        g.wind.cancel();
    }
    if before != g.slots.selected()
        && let Some(item) = g.slots.selected()
    {
        g.net_acts
            .push(bbq_core::net_act::GuestAct::Select { item });
    }
    let kind = g
        .slots
        .selected()
        .and_then(|id| g.world.items.get(&id))
        .map(|it| it.kind);
    if w.throw_down {
        g.wind.press(
            g.now,
            kind,
            &hands::Situation {
                can_act: g.rules.in_play(),
                ..Default::default()
            },
        );
    }
    g.wind.tick(dt, kind);
    if w.throw_up && g.wind.charging {
        let charge = g.wind.charge;
        g.wind.cancel();
        if let (Some(item), Some(kind)) = (g.slots.selected(), kind) {
            let from = crate::game::hand_pos(p);
            let vel = hands::throw_velocity(
                crate::game::aim_dir(p.yaw, p.pitch),
                kind.def().speed,
                charge,
                V3::new(p.mover.vx, 0., p.mover.vz),
            );
            g.net_acts.push(bbq_core::net_act::GuestAct::Throw {
                item,
                from: (from.x, from.y, from.z),
                vel: (vel.x, vel.y, vel.z),
                charge,
            });
            g.pending_gone.push((item, g.now + 0.4));
            g.slots.remove(item);
        }
    }
}
pub fn guest_act(g: &mut Game, i: usize, act: &bbq_core::net_act::GuestAct) {
    use bbq_core::net_act::GuestAct;
    let d = &g.dummies[i];
    let id = d.id;
    if !g.race.throwers.contains_key(&id) {
        return;
    }
    match act {
        GuestAct::Select { item } => {
            if let Some(n) = d.bot.slots.ids().iter().position(|x| x == item) {
                g.dummies[i].bot.slots.select_slot(n);
            }
        }
        GuestAct::Throw {
            item,
            from,
            vel,
            charge,
        } => {
            if !d.bot.slots.ids().contains(item)
                || (from.0 - d.mover.x).hypot(from.2 - d.mover.z) > 1.5
                || !(0.5..=2.5).contains(&from.1)
            {
                return;
            }
            launch(
                g,
                id,
                *item,
                V3::new(from.0, from.1, from.2),
                V3::new(vel.0, vel.1, vel.2),
                *charge,
            );
        }
        _ => {}
    }
}

/// Shared local prediction, bot movement and host remote speed validation.
fn pace(r: &Progress, at:(f32,f32), now:f32, elapsed:f32)->f32 {
    let grass=track_distance(at)>WIDTH*0.5+VERGE && elapsed>=r.protected_until;
    (if grass {GRASS_SPEED}else{1.0_f32}).min(if now<r.tug_until {TUG_SPEED}else{1.0})
}
pub fn speed(g:&Game,id:PlayerId,at:(f32,f32))->f32 {
    if g.rules.mode!=GameMode::Race {return 1.0;}
    g.race.racers.get(&id).map_or(1.0,|r|pace(r,at,g.now,g.race.elapsed))
}
pub fn active_racer(g:&Game,id:PlayerId)->bool {
    g.race.racers.get(&id).is_some_and(|r|r.finish.is_none()&&!r.disconnected)
}
/// R reuses the interaction key. The host selects the nearby target in front, never the client.
pub fn tug(g:&mut Game,p:&mut Player,id:PlayerId) {
    if g.mirror || g.rules.mode!=GameMode::Race || !g.rules.in_play() || !active_racer(g,id)
        || !crate::smoko_mode::able(g,id) || crate::smoko_mode::carried(g,id)
        || g.smoko.carries.contains_key(&id) || g.race.tug_ready.get(&id).is_some_and(|t|*t>g.now) {return;}
    let Some(at)=crate::smoko_mode::pos(g,p,id) else{return};
    let f=crate::smoko_mode::face(g,p,id);
    let target=g.race.racers.iter().filter(|(v,r)|**v!=id && r.finish.is_none() && !r.disconnected && r.tug_immune_until<=g.now)
        .filter(|(v,_)|!crate::smoko_mode::controlled(g,**v) && !g.smoko.carries.contains_key(v))
        .filter_map(|(v,_)|crate::smoko_mode::pos(g,p,*v).map(|b|(*v,b)))
        .filter(|(_,b)|b.horiz_dist(at)<=TUG_REACH && (b.y-at.y).abs()<0.65 && (b.x-at.x)*f.0+(b.z-at.z)*f.1>0.15)
        .min_by(|a,b|a.1.horiz_dist(at).total_cmp(&b.1.horiz_dist(at))).map(|(v,_)|v);
    let Some(v)=target else{return};
    g.race.tug_ready.insert(id,g.now+TUG_COOLDOWN);
    let r=g.race.racers.get_mut(&v).unwrap();r.tug_until=g.now+TUG_TIME;r.tug_immune_until=g.now+TUG_IMMUNITY;
    if let Some(d)=g.dummies.iter_mut().find(|d|d.id==v){d.anim.feel.hit(V3::new(f.0,0.0,f.1),1.0);}
    if let Some(d)=g.dummies.iter_mut().find(|d|d.id==id){d.anim.start_swing();}
    if v==PLAYER_ID {p.shake=p.shake.max(0.035);g.popup("SHORTS TUG! Brief slowdown",false);}
    if id==PLAYER_ID {g.popup("SHORTS TUG!",true);}
    g.say(format!("{} tugged {}'s shorts!",crate::round::name_of(g,id),crate::round::name_of(g,v)));
}
