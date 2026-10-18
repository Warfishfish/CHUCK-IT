//! Race foundation. Solo host-owned progress; throwing and online race sync are later milestones.
use crate::{
    game::Game,
    player::{PLAYER_ID, Player, Wanted},
    yard_scene::YardRes,
};
use bbq_core::movement::{Modifiers, MoveInput, Mover};
use bbq_core::{GameMode, PlayerId};
use std::collections::BTreeMap;
pub const LAPS: usize = 3;
pub const LIMIT: f32 = 300.0;
pub const COURSE_SCALE: f32 = 3.0;
// Oval checkpoint centres; the driving surface follows the smooth ellipse between them.
pub const TRACK: [(f32, f32); 8] = [
 (0.,-24.),(25.455843,-16.970562),(36.,0.),(25.455843,16.970562),
 (0.,24.),(-25.455843,16.970562),(-36.,0.),(-25.455843,-16.970562)
];
pub fn oval(t:f32)->(f32,f32) {(12.0*COURSE_SCALE*t.sin(),-8.0*COURSE_SCALE*t.cos())}
pub const WIDTH: f32 = 4.8;
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
}
#[derive(Default)]
pub struct State {
    pub racers: BTreeMap<PlayerId, Progress>,
    pub elapsed: f32,
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
        .map(|i|segment(p,oval(i as f32*std::f32::consts::TAU/64.),oval((i+1) as f32*std::f32::consts::TAU/64.)))
        .fold(f32::INFINITY,f32::min)
}
impl Progress {
    pub fn new(at: (f32, f32)) -> Self {
        Self {
            passed: 0,
            finish: None,
            previous: at,
            disconnected: false,
        }
    }
    pub fn target(&self) -> (f32, f32) {
        TRACK[(self.passed + 1) % 8]
    }
    pub fn update(&mut self, at: (f32, f32), time: f32) -> bool {
        if self.finish.is_some() || self.disconnected {
            return false;
        }
        // Out-of-lane movement or teleporting must never advance race progress.
        if track_distance(at) > WIDTH * 0.5 + 0.35 || distance(at, self.previous) > 2.0 {
            return true;
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
    g.race = State::default();
    if g.rules.mode != GameMode::Race {
        return;
    }
    y.0.colliders.clear();
    y.0.race=true;
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
    g.round.setup.bots=if racer {2}else{3};
    let mut ids = Vec::new();
    if racer {
        ids.push(PLAYER_ID);
    }
    ids.extend(
        g.dummies
            .iter()
            .filter(|d| d.remote.is_none())
            .map(|d| d.id)
            .take(if racer { 2 } else { 3 }),
    );
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
    if !racer {
        p.mover = Mover::new(0., -30.);
        p.yaw = std::f32::consts::PI;
        p.pitch = -0.1;
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
                    distance(a.previous, a.target()).total_cmp(&distance(b.previous, b.target()))
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
    for d in &mut g.dummies {
        let Some(r) = g.race.racers.get(&d.id) else {
            continue;
        };
        let at = (d.mover.x, d.mover.z);
        // Follow the oval with a short look-ahead, never a chord across the infield.
        let sector=(r.passed%8) as f32*std::f32::consts::TAU/8.;
        let mut angle=(at.0/36.).atan2(-at.1/24.).rem_euclid(std::f32::consts::TAU);
        if angle>sector+std::f32::consts::PI {angle-=std::f32::consts::TAU;}
        let aim=(angle+0.10).min(sector+std::f32::consts::TAU/8.);
        let centre=oval(aim);let tangent=(36.*aim.cos(),24.*aim.sin());let len=tangent.0.hypot(tangent.1);
        let lane=(d.id%3) as f32-1.;
        let target=(centre.0+tangent.1/len*lane*0.75,centre.1-tangent.0/len*lane*0.75);
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
                frozen: !running,
                ..Default::default()
            },
            &y.0,
        );
    }
    if !g.race.racers.contains_key(&PLAYER_ID) {
        let x=p.mover.x/42.;let z=-p.mover.z/28.;
        let angle=x.atan2(z).clamp(-0.85,0.85);
        let radius=x.hypot(z).clamp(1.0,1.12);
        p.mover.x=42.*radius*angle.sin();p.mover.z=-28.*radius*angle.cos();
    }
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
            if r.update((m.x, m.z), g.race.elapsed) || m.y < -1.0 {
                let at = TRACK[r.passed % 8];
                *m = Mover::new(at.0, at.1);
                r.previous = at;
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
    g.prompt = if let Some(r) = g.race.racers.get(&PLAYER_ID) {
        if let Some(t) = r.finish {
            format!("FINISHED in {t:.2}s • Waiting for the other racers")
        } else {
            format!(
                "RACER • Lap {}/{} • Follow gate {} → • Stay inside the white edges",
                (r.passed / 8 + 1).min(LAPS),
                LAPS,
                r.passed % 8 + 1
            )
        }
    } else {
        "SIDELINE THROWER • Race preview • Throwing and streaks arrive in milestone 2".into()
    };
    // Consume action edges: no combat or legacy errands in the race foundation.
    w.throw_down = false;
    w.throw_up = false;
    w.grab_pressed = false;
    w.grab_released = false;
    w.interact_pressed = false;
    w.jump_pressed = false;
    w.emote = None;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcuts_and_teleports_do_not_advance() {
        let mut r = Progress::new(TRACK[0]);
        assert!(r.update(TRACK[1], 1.));
        assert_eq!(r.passed, 0);
        assert!(r.update((0., 0.), 2.));
        assert_eq!(r.passed, 0);
    }
    #[test]
    fn ordered_three_laps_finish_once() {
        let mut r = Progress::new(TRACK[0]);
        let mut time = 0.;
        for i in 1..=2400 {time+=0.01;assert!(!r.update(oval(i as f32*std::f32::consts::TAU/800.),time));}
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
