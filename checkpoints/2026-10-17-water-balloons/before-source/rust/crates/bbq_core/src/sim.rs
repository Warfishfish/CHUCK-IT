//! The fixed-step simulation clock and the round state machine (spec sections 1, 9 and 15).
//!
//! Nothing here needs a window, a graphics card or a real clock, so a whole round can be run
//! in a unit test. The game feeds `FixedClock` the real frame time and runs one `Sim::step`
//! per tick it hands back.

use std::collections::BTreeMap;

use crate::PlayerId;
use crate::drinks::{DRUNK_ALL, SMOKO_SIP, SOBER_RATE};
use crate::scoring::Phase;
use crate::stun::{Body, TickEvents};

/// Simulation rate: 60 steps per second.
pub const TICK_HZ: u32 = 60;
pub const TICK_DT: f32 = 1.0 / TICK_HZ as f32;
/// Characters are frozen for this long before a round (spec: 3.2 s).
pub const COUNTDOWN: f32 = 3.2;
/// The results panel opens this long after the whistle.
pub const PANEL_DELAY: f32 = 1.1;
/// If the game stalls, run at most this many steps per frame instead of spiralling.
pub const MAX_STEPS_PER_FRAME: u32 = 8;

/// Turns uneven frame times into a whole number of fixed steps.
#[derive(Clone, Debug, Default)]
pub struct FixedClock {
    acc: f32,
}

impl FixedClock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a real frame time; returns how many `TICK_DT` steps to run now.
    pub fn advance(&mut self, real_dt: f32) -> u32 {
        self.acc += real_dt.clamp(0.0, 1.0);
        let mut steps = 0;
        while self.acc >= TICK_DT && steps < MAX_STEPS_PER_FRAME {
            self.acc -= TICK_DT;
            steps += 1;
        }
        if steps == MAX_STEPS_PER_FRAME {
            self.acc = 0.0; // drop the backlog
        }
        steps
    }

    /// How far we are between two steps (0..1), for smooth drawing.
    pub fn alpha(&self) -> f32 {
        (self.acc / TICK_DT).clamp(0.0, 1.0)
    }
}

/// Things the game should react to (sounds, banners, panels).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Countdown finished: go!
    Go,
    /// Time up: scoring stops.
    Whistle,
    /// The results panel should open.
    ShowResults,
    StunEnded(PlayerId),
    FellOver(PlayerId),
    GotUp(PlayerId),
}

/// One character's body and drunk meter.
#[derive(Clone, Debug, Default)]
pub struct Fighter {
    pub body: Body,
    /// Drunk meter, 0..=100.
    pub drunk: f32,
    pub at_smoko: bool,
}

impl Fighter {
    /// Advance by `dt`. Drunk mode holds everyone at least `DRUNK_ALL`.
    pub fn tick(&mut self, dt: f32, drunk_mode: bool) -> TickEvents {
        if self.at_smoko {
            self.drunk += SMOKO_SIP * dt;
        }
        self.drunk = (self.drunk - SOBER_RATE * dt).clamp(0.0, 100.0);
        if drunk_mode {
            self.drunk = self.drunk.max(DRUNK_ALL);
        }
        self.body.tick(dt)
    }
}

/// The round: clock, phase and everybody's bodies.
#[derive(Clone, Debug)]
pub struct Sim {
    pub phase: Phase,
    pub drunk_mode: bool,
    /// Seconds left in the current phase's timer.
    pub time_left: f32,
    pub round_len: f32,
    panel_in: f32,
    pub fighters: BTreeMap<PlayerId, Fighter>,
    pub ticks: u64,
}

impl Sim {
    pub fn new(round_len: f32, drunk_mode: bool) -> Self {
        Self {
            phase: Phase::Menu,
            drunk_mode,
            time_left: 0.0,
            round_len,
            panel_in: 0.0,
            fighters: BTreeMap::new(),
            ticks: 0,
        }
    }

    pub fn add_fighter(&mut self, id: PlayerId) {
        self.fighters.entry(id).or_default();
    }

    /// Begin a round: everyone resets, the countdown starts.
    pub fn start_round(&mut self) {
        for f in self.fighters.values_mut() {
            *f = Fighter::default();
            if self.drunk_mode {
                f.drunk = DRUNK_ALL;
            }
        }
        self.phase = Phase::Countdown;
        self.time_left = COUNTDOWN;
    }

    /// Characters can't move during the countdown or once the round is over.
    pub fn frozen(&self) -> bool {
        self.phase != Phase::Play
    }

    /// One fixed step.
    pub fn step(&mut self) -> Vec<Event> {
        let dt = TICK_DT;
        let mut events = Vec::new();
        self.ticks += 1;

        for (&id, f) in self.fighters.iter_mut() {
            let was_fallen = f.body.fall_t > 0.0;
            let ev = f.tick(dt, self.drunk_mode);
            if ev.stun_ended {
                events.push(Event::StunEnded(id));
            }
            if ev.fall_ended {
                events.push(Event::GotUp(id));
            } else if !was_fallen && f.body.fall_t > 0.0 {
                events.push(Event::FellOver(id));
            }
        }

        match self.phase {
            Phase::Countdown => {
                self.time_left -= dt;
                if self.time_left <= 0.0 {
                    self.phase = Phase::Play;
                    self.time_left = self.round_len;
                    events.push(Event::Go);
                }
            }
            Phase::Play => {
                self.time_left -= dt;
                if self.time_left <= 0.0 {
                    self.time_left = 0.0;
                    self.phase = Phase::Results; // scoring stops at the whistle
                    self.panel_in = PANEL_DELAY;
                    events.push(Event::Whistle);
                }
            }
            Phase::Results => {
                if self.panel_in > 0.0 {
                    self.panel_in -= dt;
                    if self.panel_in <= 0.0 {
                        events.push(Event::ShowResults);
                    }
                }
            }
            Phase::Menu | Phase::Warmup => {}
        }
        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_runs_sixty_steps_a_second() {
        let mut c = FixedClock::new();
        let steps: u32 = (0..100).map(|_| c.advance(0.01)).sum();
        assert!((59..=60).contains(&steps), "got {steps}");
    }

    #[test]
    fn clock_does_not_spiral_after_a_stall() {
        let mut c = FixedClock::new();
        assert_eq!(c.advance(5.0), MAX_STEPS_PER_FRAME);
        assert_eq!(c.advance(0.0), 0);
    }

    #[test]
    fn clock_ignores_negative_time() {
        let mut c = FixedClock::new();
        assert_eq!(c.advance(-1.0), 0);
    }

    #[test]
    fn headless_round_runs_countdown_play_whistle_results() {
        let mut sim = Sim::new(10.0, false);
        sim.add_fighter(1);
        sim.add_fighter(2);
        sim.start_round();
        assert!(sim.frozen());

        let mut log = Vec::new();
        let mut ticks_to_go = None;
        for _ in 0..(60 * 20) {
            for e in sim.step() {
                if e == Event::Go {
                    ticks_to_go = Some(sim.ticks);
                }
                log.push(e);
            }
        }
        // 3.2 s at 60 Hz = 192 steps (allow one for float rounding)
        let go = ticks_to_go.expect("Go never happened");
        assert!((192..=193).contains(&go), "go at {go}");
        let order: Vec<_> = log
            .iter()
            .filter(|e| matches!(e, Event::Go | Event::Whistle | Event::ShowResults))
            .collect();
        assert_eq!(order, [&Event::Go, &Event::Whistle, &Event::ShowResults]);
        assert_eq!(sim.phase, Phase::Results);
    }

    #[test]
    fn same_inputs_same_result() {
        let run = || {
            let mut sim = Sim::new(5.0, false);
            sim.add_fighter(1);
            sim.start_round();
            sim.fighters.get_mut(&1).unwrap().drunk = 50.0;
            sim.fighters.get_mut(&1).unwrap().body.apply_hit(2.0, None);
            for _ in 0..600 {
                sim.step();
            }
            (
                sim.ticks,
                sim.fighters[&1].drunk.to_bits(),
                sim.fighters[&1].body.stun_grace.to_bits(),
            )
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn drunk_mode_holds_everyone_at_78() {
        let mut sim = Sim::new(5.0, true);
        sim.add_fighter(1);
        sim.start_round();
        for _ in 0..600 {
            sim.step();
        }
        assert!(sim.fighters[&1].drunk >= DRUNK_ALL);
    }

    #[test]
    fn sobering_rate_is_1_6_per_second() {
        let mut f = Fighter {
            drunk: 50.0,
            ..Default::default()
        };
        for _ in 0..60 {
            f.tick(TICK_DT, false);
        }
        assert!((f.drunk - 48.4).abs() < 0.01, "{}", f.drunk);
    }

    #[test]
    fn smoko_nets_plus_2_4_per_second() {
        let mut f = Fighter {
            drunk: 10.0,
            at_smoko: true,
            ..Default::default()
        };
        for _ in 0..60 {
            f.tick(TICK_DT, false);
        }
        assert!((f.drunk - 12.4).abs() < 0.01, "{}", f.drunk);
    }

    #[test]
    fn stun_gets_grace_after_it_ends() {
        let mut sim = Sim::new(5.0, false);
        sim.add_fighter(1);
        sim.start_round();
        sim.fighters.get_mut(&1).unwrap().body.apply_hit(1.0, None);
        let mut ended = false;
        for _ in 0..70 {
            ended |= sim.step().contains(&Event::StunEnded(1));
        }
        assert!(ended);
        assert!(sim.fighters[&1].body.is_stun_protected());
    }
}
