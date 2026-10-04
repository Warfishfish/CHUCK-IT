//! One character's drinking, drunk meter, stacking it, getting up and being helped up
//! (spec section 4), plus the maths for the drunk and fallen camera (spec section 15).
//!
//! `drinks.rs` holds the numbers; this puts them together into the thing that is ticked every
//! step. No engine code here, so it is all tested without a window.

use crate::drinks::{
    self, DRINK_SPEED, DRUNK_ALL, Drink, DrunkModeFall, FALL_DURATION_DEFAULT, FALL_IMMUNE,
    FALL_IMMUNE_DRUNK_MODE, Gait, SOBER_RATE, Tier, drunk_amt, drunk_tier, fall_chance_per_sec,
};
use crate::rng::Rng;
use crate::stun::Body;
use crate::yard::Collider;

/// Hold R this long next to a fallen mate to pull them up.
pub const HELP_TIME: f32 = 1.5;
/// Points for helping someone up (not in Heist).
pub const HELP_PTS: i32 = 25;
/// Wait this long before you can help again.
pub const HELP_COOLDOWN: f32 = 1.0;
/// How close you must be to start helping (the host allows 2.8).
pub const HELP_REACH: f32 = 1.9;
pub const HELP_REACH_HOST: f32 = 2.8;
/// After someone helps you up you can't fall for this long.
pub const HELPED_IMMUNE: f32 = 25.0;
/// Seconds the "getting up" camera/body ramp takes.
pub const RISE_TIME: f32 = 0.45;
/// Seconds the "going down" ramp takes after stacking it.
pub const FALL_RAMP: f32 = 0.3;

/// Which drink you'd get standing at (x, y, z), or `None` if you're not at the bar.
/// You must be on the ground, within `w/2 + 0.8` of the bar's middle sideways, and from the
/// bar's middle out to `d/2 + 1.6` in front of it (towards +z).
pub fn bar_spot(bar: &Collider, x: f32, y: f32, z: f32) -> Option<Drink> {
    if y > 0.4 {
        return None;
    }
    let cx = (bar.x0 + bar.x1) / 2.0;
    let cz = (bar.z0 + bar.z1) / 2.0;
    let (w, d) = (bar.x1 - bar.x0, bar.z1 - bar.z0);
    let (dx, dz) = (x - cx, z - cz);
    if dx.abs() > w / 2.0 + 0.8 || dz < 0.0 || dz > d / 2.0 + 1.6 {
        return None;
    }
    Some(Drink::at_bar(dx))
}

/// A drink in progress.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Drinking {
    pub drink: Drink,
    pub left: f32,
    pub total: f32,
}

impl Drinking {
    /// 0 at the first sip, 1 when finished (drives the drink in your hand).
    pub fn progress(&self) -> f32 {
        1.0 - self.left / self.total
    }
}

/// What is going on around the character this tick (the game fills this in).
#[derive(Clone, Copy, Debug)]
pub struct Env {
    /// The round is being played (not countdown or break).
    pub in_play: bool,
    pub at_smoko: bool,
    pub grounded: bool,
    pub in_pool: bool,
    /// Trying to walk (used for the chance of falling).
    pub moving: bool,
    /// The host has falls switched on.
    pub falls_on: bool,
    /// Drunk mode: everyone is held at 78 or more.
    pub drunk_mode: bool,
    /// Whether this character can stack it at all (you can; bots can't).
    pub can_fall: bool,
    /// How long a fall lasts (10 s unless the host picked 20 or 30).
    pub fall_duration: f32,
}

impl Default for Env {
    fn default() -> Self {
        Env {
            in_play: true,
            at_smoko: false,
            grounded: true,
            in_pool: false,
            moving: false,
            falls_on: true,
            drunk_mode: false,
            can_fall: true,
            fall_duration: FALL_DURATION_DEFAULT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// Finished a drink. `tier_changed` is the new tier name when the drink moved you up one.
    DrankUp {
        drink: Drink,
        added: f32,
        tier: Tier,
        tier_changed: bool,
    },
    /// Just stacked it.
    StackedIt,
    /// Got back up (on your own).
    GotUp,
}

#[derive(Clone, Debug)]
pub struct DrunkState {
    /// 0..100.
    pub meter: f32,
    pub drinking: Option<Drinking>,
    /// Seconds left in which you can't stack it.
    pub fall_immune: f32,
    /// Seconds since you went down (for the camera drop).
    pub fall_age: f32,
    /// Counts down from 0.45 after getting up (for the camera and body rise).
    pub rise: f32,
    /// This character's own random wobble offset.
    pub phase: f32,
    gait: Gait,
    mode_fall: DrunkModeFall,
}

impl DrunkState {
    pub fn new(phase: f32, rng: &mut Rng) -> Self {
        DrunkState {
            meter: 0.0,
            drinking: None,
            fall_immune: 0.0,
            fall_age: 0.0,
            rise: 0.0,
            phase,
            gait: Gait::new(rng),
            mode_fall: DrunkModeFall::default(),
        }
    }

    /// Back to sober (start of a round).
    pub fn reset(&mut self) {
        self.meter = 0.0;
        self.drinking = None;
        self.fall_immune = 0.0;
        self.fall_age = 0.0;
        self.rise = 0.0;
    }

    /// How drunk you look and steer, 0 to 1.
    pub fn da(&self) -> f32 {
        drunk_amt(self.meter)
    }

    pub fn tier(&self) -> Tier {
        drunk_tier(self.meter)
    }

    pub fn is_drinking(&self) -> bool {
        self.drinking.is_some()
    }

    /// Press R at the bar. `spot` is what `bar_spot` said. Returns true if a drink started.
    pub fn start_drink(&mut self, spot: Option<Drink>, body: &Body, env: &Env) -> bool {
        let Some(drink) = spot else { return false };
        if self.drinking.is_some() || body.stun > 0.0 || env.at_smoko || !env.in_play {
            return false;
        }
        let t = drink.time();
        self.drinking = Some(Drinking {
            drink,
            left: t,
            total: t,
        });
        true
    }

    /// Anything that interrupts a drink (a hit, a fall, sitting down).
    pub fn cancel_drink(&mut self) {
        self.drinking = None;
    }

    /// The walking-speed multiplier from drinking and from the Drunk-mode wobble.
    /// (`Modifiers::drinking` already applies x0.55 for the drink, so this is only the gait.)
    pub fn gait(&mut self, dt: f32, t: f32, env: &Env, rng: &mut Rng) -> f32 {
        if !env.drunk_mode {
            return 1.0;
        }
        self.gait.multiplier(dt, self.da(), t, self.phase, rng)
    }

    /// Walking speed while drinking, for tests and the HUD.
    pub fn drink_speed(&self) -> f32 {
        if self.drinking.is_some() {
            DRINK_SPEED
        } else {
            1.0
        }
    }

    /// Add to the meter directly (smoko sips).
    pub fn add(&mut self, amount: f32) {
        self.meter = (self.meter + amount).clamp(0.0, 100.0);
    }

    /// Call once per step, after `body.tick`. `fell_ended` is `TickEvents::fall_ended` from that.
    /// `body` is changed when you stack it.
    pub fn tick(
        &mut self,
        dt: f32,
        env: &Env,
        body: &mut Body,
        fall_ended: bool,
        rng: &mut Rng,
    ) -> Vec<Event> {
        let mut out = Vec::new();

        // drinking, or sobering up
        if let Some(d) = &mut self.drinking {
            if body.stun > 0.0 {
                self.drinking = None; // getting stunned spills it
            } else {
                d.left -= dt;
                if d.left <= 0.0 {
                    let before = drunk_tier(self.meter);
                    let drink = d.drink;
                    self.meter = (self.meter + drink.amount()).min(100.0);
                    self.drinking = None;
                    let tier = drunk_tier(self.meter);
                    out.push(Event::DrankUp {
                        drink,
                        added: drink.amount(),
                        tier,
                        tier_changed: tier != before,
                    });
                }
            }
        } else if self.meter > 0.0 && !env.at_smoko {
            self.meter = (self.meter - SOBER_RATE * dt).max(0.0);
        }
        if env.drunk_mode {
            self.meter = self.meter.max(DRUNK_ALL);
        }

        // falling
        self.fall_immune = (self.fall_immune - dt).max(0.0);
        if fall_ended {
            self.fall_immune = if env.drunk_mode {
                FALL_IMMUNE_DRUNK_MODE
            } else {
                FALL_IMMUNE
            };
            out.push(Event::GotUp);
        }
        if body.fall_t > 0.0 {
            self.fall_age += dt;
            self.rise = RISE_TIME;
            return out;
        }
        if self.rise > 0.0 {
            self.rise = (self.rise - dt).max(0.0);
        }
        let can = env.can_fall
            && env.falls_on
            && env.in_play
            && body.down_t <= 0.0
            && env.grounded
            && self.fall_immune <= 0.0
            && !env.in_pool;
        if !can {
            return out;
        }
        let falls = if env.drunk_mode {
            self.mode_fall.tick(dt, env.moving, rng)
        } else {
            let p = fall_chance_per_sec(self.da(), env.moving) * dt;
            p > 0.0 && rng.f32() < p
        };
        if falls {
            self.stack_it(body, env.fall_duration);
            out.push(Event::StackedIt);
        }
        out
    }

    /// Go down for `duration` seconds (also used by the viewer key).
    pub fn stack_it(&mut self, body: &mut Body, duration: f32) {
        body.start_fall(duration);
        self.fall_age = 0.0;
        self.drinking = None;
    }

    /// Someone helped you up: up at once, and nobody trips you for 25 s.
    pub fn helped_up(&mut self, body: &mut Body) {
        body.get_up();
        self.fall_immune = self.fall_immune.max(HELPED_IMMUNE);
    }

    /// 0..1: how far down you are from stacking it (going down) or still getting up.
    /// Drives the camera drop and the body lying flat.
    pub fn fall_factor(&self, body: &Body) -> f32 {
        if body.fall_t > 0.0 {
            (self.fall_age / FALL_RAMP).min(1.0)
        } else {
            self.rise / RISE_TIME
        }
    }
}

/// Can `helper` pull `target` up? The target must be stacked, it can't be yourself, and in
/// team modes only teammates help each other.
pub fn can_help(
    helper_is_target: bool,
    target_fallen: bool,
    teams_on: bool,
    same_team: bool,
) -> bool {
    !helper_is_target && target_fallen && (!teams_on || same_team)
}

/// Hold R next to a fallen mate. Fills over 1.5 s, then they're up, and you wait 1 s before
/// helping again.
#[derive(Clone, Copy, Debug, Default)]
pub struct HelpHold {
    pub t: f32,
    pub cooldown: f32,
}

impl HelpHold {
    /// `has_target`: someone fallen is in reach. `holding`: R is down. Returns true on the
    /// tick the help is finished.
    pub fn tick(&mut self, dt: f32, has_target: bool, holding: bool) -> bool {
        self.cooldown = (self.cooldown - dt).max(0.0);
        if has_target && holding && self.cooldown <= 0.0 {
            self.t += dt;
            if self.t >= HELP_TIME {
                self.t = 0.0;
                self.cooldown = HELP_COOLDOWN;
                return true;
            }
        } else {
            self.t = 0.0;
        }
        false
    }

    /// 0..1, for the progress text.
    pub fn progress(&self) -> f32 {
        (self.t / HELP_TIME).clamp(0.0, 1.0)
    }
}

/// How the camera moves when you're drunk, stacked or flat (angles in radians, `dy` in metres).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CamFx {
    pub dy: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

/// Drunk sway plus the drop when you're down. `da` is the drunk amount, `t` the game time,
/// `fall` is `DrunkState::fall_factor`, `flat` is 0..1 for being knocked flat, and
/// `pitch_now` is where you're looking now (being flat turns your view up at the sky).
pub fn camera_fx(da: f32, t: f32, fall: f32, flat: f32, pitch_now: f32) -> CamFx {
    let yaw = da * ((t * 0.63).sin() * 0.06 + (t * 1.71).sin() * 0.018);
    let pitch = da * ((t * 0.52 + 1.3).sin() * 0.035 + (t * 1.37).sin() * 0.012);
    let roll = da * ((t * 0.8 + 0.5).sin() * 0.21 + (t * 1.9).sin() * 0.03);
    CamFx {
        dy: -flat * 1.2 - fall * 1.25,
        yaw,
        pitch: pitch + flat * (1.1 - pitch_now),
        roll: roll + flat * 0.25 + fall * 1.15,
    }
}

/// How flat you are after a knockdown: eases in over 0.22 s and out over the last 0.45 s.
pub fn flat_amount(down_t: f32, total: f32) -> f32 {
    if down_t <= 0.0 {
        return 0.0;
    }
    ((total - down_t) / 0.22).min(down_t / 0.45).clamp(0.0, 1.0)
}

/// Stops the compiler complaining that `drinks` is only used in the paths above.
#[allow(dead_code)]
fn _uses() -> f32 {
    drinks::SMOKO_SIP
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yard::BAR;

    const DT: f32 = 1.0 / 60.0;

    fn me() -> (DrunkState, Body, Rng, Env) {
        seeded(7)
    }

    fn seeded(seed: u64) -> (DrunkState, Body, Rng, Env) {
        let mut rng = Rng::new(seed);
        (
            DrunkState::new(0.0, &mut rng),
            Body::default(),
            rng,
            Env::default(),
        )
    }

    fn run(d: &mut DrunkState, b: &mut Body, env: &Env, rng: &mut Rng, secs: f32) -> Vec<Event> {
        let mut all = Vec::new();
        for _ in 0..(secs / DT) as usize {
            let ev = b.tick(DT);
            all.extend(d.tick(DT, env, b, ev.fall_ended, rng));
        }
        all
    }

    #[test]
    fn bar_spot_picks_the_drink_by_where_you_stand() {
        let z = (BAR.z0 + BAR.z1) / 2.0 + 1.0;
        assert_eq!(bar_spot(&BAR, -1.2, 0.0, z), Some(Drink::Beer));
        assert_eq!(bar_spot(&BAR, 0.0, 0.0, z), Some(Drink::Wine));
        assert_eq!(bar_spot(&BAR, 1.2, 0.0, z), Some(Drink::Rum));
        // behind the bar, too far out, too far along, or in the air: nothing
        assert_eq!(bar_spot(&BAR, 0.0, 0.0, BAR.z0 - 1.0), None);
        assert_eq!(bar_spot(&BAR, 0.0, 0.0, z + 2.0), None);
        assert_eq!(bar_spot(&BAR, 3.0, 0.0, z), None);
        assert_eq!(bar_spot(&BAR, 0.0, 1.0, z), None);
    }

    #[test]
    fn a_drink_takes_its_time_then_adds_to_the_meter() {
        let (mut d, mut b, mut rng, env) = me();
        assert!(d.start_drink(Some(Drink::Beer), &b, &env));
        assert!(!d.start_drink(Some(Drink::Rum), &b, &env), "one at a time");
        let ev = run(&mut d, &mut b, &env, &mut rng, 1.0);
        assert!(ev.is_empty() && d.is_drinking());
        let ev = run(&mut d, &mut b, &env, &mut rng, 0.6);
        assert_eq!(ev.len(), 1);
        assert!(!d.is_drinking());
        assert!((d.meter - 18.0).abs() < 1.0, "meter {}", d.meter);
        match ev[0] {
            Event::DrankUp {
                tier, tier_changed, ..
            } => {
                assert_eq!(tier, Tier::Tipsy);
                assert!(tier_changed);
            }
            e => panic!("{e:?}"),
        }
    }

    #[test]
    fn you_sober_up_slowly_but_not_at_smoko_or_while_drinking() {
        let (mut d, mut b, mut rng, mut env) = me();
        d.meter = 50.0;
        run(&mut d, &mut b, &env, &mut rng, 5.0);
        assert!((d.meter - 42.0).abs() < 0.2, "{}", d.meter);
        env.at_smoko = true;
        run(&mut d, &mut b, &env, &mut rng, 5.0);
        assert!((d.meter - 42.0).abs() < 0.2, "no sobering at smoko");
    }

    #[test]
    fn meter_stops_at_100() {
        let (mut d, mut b, mut rng, env) = me();
        d.meter = 95.0;
        d.start_drink(Some(Drink::Rum), &b, &env);
        run(&mut d, &mut b, &env, &mut rng, 1.0);
        assert!(d.meter <= 100.0 && d.meter > 98.0);
    }

    #[test]
    fn a_stun_spills_the_drink() {
        let (mut d, mut b, mut rng, env) = me();
        d.start_drink(Some(Drink::Wine), &b, &env);
        b.apply_hit(1.0, None);
        let ev = run(&mut d, &mut b, &env, &mut rng, 0.1);
        assert!(!d.is_drinking());
        assert!(ev.is_empty());
        assert_eq!(d.meter, 0.0);
    }

    #[test]
    fn cannot_drink_when_stunned_at_smoko_or_between_rounds() {
        let (mut d, mut b, _rng, mut env) = me();
        b.apply_hit(1.0, None);
        assert!(!d.start_drink(Some(Drink::Beer), &b, &env));
        b.stun = 0.0;
        env.at_smoko = true;
        assert!(!d.start_drink(Some(Drink::Beer), &b, &env));
        env.at_smoko = false;
        env.in_play = false;
        assert!(!d.start_drink(Some(Drink::Beer), &b, &env));
        env.in_play = true;
        assert!(!d.start_drink(None, &b, &env), "not at the bar");
        assert!(d.start_drink(Some(Drink::Beer), &b, &env));
    }

    #[test]
    fn sober_people_never_fall_and_very_drunk_people_do() {
        let (mut d, mut b, mut rng, mut env) = me();
        env.moving = true;
        d.meter = 14.0;
        let ev = run(&mut d, &mut b, &env, &mut rng, 120.0);
        assert!(!ev.contains(&Event::StackedIt));
        // maxed out and walking: about one fall every 36 s, so 20 minutes is plenty
        let mut falls = 0;
        for seed in 0..40 {
            let (mut d, mut b, mut rng, mut env) = seeded(seed + 100);
            env.moving = true;
            d.meter = 100.0;
            let ev = run(&mut d, &mut b, &env, &mut rng, 60.0);
            falls += ev.iter().filter(|e| **e == Event::StackedIt).count();
        }
        assert!(falls > 10, "only {falls} falls in 40 minutes");
    }

    #[test]
    fn falls_need_the_right_conditions() {
        for blocked in 0..5 {
            let (mut d, mut b, mut rng, mut env) = me();
            env.moving = true;
            d.meter = 100.0;
            match blocked {
                0 => env.falls_on = false,
                1 => env.grounded = false,
                2 => env.in_pool = true,
                3 => d.fall_immune = 9999.0,
                _ => env.can_fall = false,
            }
            let ev = run(&mut d, &mut b, &env, &mut rng, 600.0);
            assert!(!ev.contains(&Event::StackedIt), "case {blocked}");
        }
        // being knocked flat also blocks it
        let (mut d, mut b, mut rng, mut env) = me();
        env.moving = true;
        d.meter = 100.0;
        b.down_t = 5000.0;
        let ev = run(&mut d, &mut b, &env, &mut rng, 100.0);
        assert!(!ev.contains(&Event::StackedIt));
    }

    #[test]
    fn stacking_it_puts_you_down_then_gives_25_seconds_of_peace() {
        let (mut d, mut b, mut rng, env) = me();
        d.start_drink(Some(Drink::Beer), &b, &env);
        d.stack_it(&mut b, 10.0);
        assert!(b.is_down() && !d.is_drinking());
        let ev = run(&mut d, &mut b, &env, &mut rng, 9.9);
        assert!(b.is_down() && !ev.contains(&Event::GotUp));
        let ev = run(&mut d, &mut b, &env, &mut rng, 0.2);
        assert!(ev.contains(&Event::GotUp));
        assert!(!b.is_down());
        assert!((d.fall_immune - 25.0).abs() < 0.5);
        // and in Drunk mode it's only 6 s
        let (mut d, mut b, mut rng, mut env) = me();
        env.drunk_mode = true;
        d.stack_it(&mut b, 1.0);
        run(&mut d, &mut b, &env, &mut rng, 1.1);
        assert!((d.fall_immune - 6.0).abs() < 0.5);
    }

    #[test]
    fn being_helped_up_is_instant_and_gives_25_s() {
        let (mut d, mut b, mut rng, env) = me();
        d.stack_it(&mut b, 10.0);
        run(&mut d, &mut b, &env, &mut rng, 2.0);
        d.helped_up(&mut b);
        assert!(!b.is_down());
        assert!(d.fall_immune >= 25.0);
        run(&mut d, &mut b, &env, &mut rng, 0.1);
        assert!(!b.is_down());
    }

    #[test]
    fn camera_drops_while_down_and_rises_after() {
        let (mut d, mut b, mut rng, env) = me();
        d.stack_it(&mut b, 5.0);
        run(&mut d, &mut b, &env, &mut rng, 0.15);
        let half = d.fall_factor(&b);
        assert!(half > 0.4 && half < 0.6, "{half}");
        run(&mut d, &mut b, &env, &mut rng, 0.5);
        assert_eq!(d.fall_factor(&b), 1.0);
        d.helped_up(&mut b);
        d.rise = RISE_TIME;
        run(&mut d, &mut b, &env, &mut rng, 0.25);
        let r = d.fall_factor(&b);
        assert!(r > 0.3 && r < 0.7, "{r}");
        run(&mut d, &mut b, &env, &mut rng, 0.3);
        assert_eq!(d.fall_factor(&b), 0.0);
    }

    #[test]
    fn drunk_mode_holds_everyone_at_78_or_more() {
        let (mut d, mut b, mut rng, mut env) = me();
        env.drunk_mode = true;
        run(&mut d, &mut b, &env, &mut rng, 30.0);
        assert!(d.meter >= 78.0);
    }

    #[test]
    fn drunk_mode_falls_about_4_percent_of_every_2_s_of_walking() {
        let mut falls = 0;
        let runs = 200;
        for s in 0..runs {
            let mut rng = Rng::new(s + 1);
            let mut d = DrunkState::new(0.0, &mut rng);
            let mut b = Body::default();
            let env = Env {
                drunk_mode: true,
                moving: true,
                ..Env::default()
            };
            // 40 s walking, no falls allowed in the first 6 s after getting up, so count first fall
            let ev = run(&mut d, &mut b, &env, &mut rng, 40.0);
            if ev.contains(&Event::StackedIt) {
                falls += 1;
            }
        }
        // 20 rolls of 4% each: about 56% of runs have at least one fall
        let rate = falls as f32 / runs as f32;
        assert!(rate > 0.35 && rate < 0.75, "rate {rate}");
    }

    #[test]
    fn standing_still_in_drunk_mode_is_safe() {
        let (mut d, mut b, mut rng, mut env) = me();
        env.drunk_mode = true;
        env.moving = false;
        let ev = run(&mut d, &mut b, &env, &mut rng, 300.0);
        assert!(!ev.contains(&Event::StackedIt));
    }

    #[test]
    fn gait_only_wobbles_in_drunk_mode() {
        let (mut d, _b, mut rng, mut env) = me();
        d.meter = 90.0;
        assert_eq!(d.gait(DT, 1.0, &env, &mut rng), 1.0);
        env.drunk_mode = true;
        let mut lo = 9.0f32;
        let mut hi = 0.0f32;
        for i in 0..2000 {
            let g = d.gait(DT, i as f32 * DT, &env, &mut rng);
            lo = lo.min(g);
            hi = hi.max(g);
        }
        assert!(lo < 0.8 && hi > 1.1, "{lo} {hi}");
        assert!(lo >= 0.35 && hi <= 1.45);
    }

    #[test]
    fn helping_takes_1_5_s_of_holding_and_then_waits_1_s() {
        let mut h = HelpHold::default();
        let mut done = 0;
        for _ in 0..(1.4 / DT) as usize {
            done += h.tick(DT, true, true) as i32;
        }
        assert_eq!(done, 0);
        assert!(h.progress() > 0.9);
        for _ in 0..(0.2 / DT) as usize {
            done += h.tick(DT, true, true) as i32;
        }
        assert_eq!(done, 1);
        // straight after, holding does nothing for a second
        for _ in 0..(0.9 / DT) as usize {
            assert!(!h.tick(DT, true, true));
        }
        assert_eq!(h.t, 0.0);
    }

    #[test]
    fn letting_go_or_walking_away_restarts_the_help() {
        let mut h = HelpHold::default();
        for _ in 0..60 {
            h.tick(DT, true, true);
        }
        assert!(h.progress() > 0.5);
        h.tick(DT, true, false);
        assert_eq!(h.t, 0.0);
        for _ in 0..60 {
            h.tick(DT, true, true);
        }
        h.tick(DT, false, true);
        assert_eq!(h.t, 0.0);
    }

    #[test]
    fn who_can_help_whom() {
        assert!(can_help(false, true, false, false));
        assert!(!can_help(true, true, false, false), "not yourself");
        assert!(!can_help(false, false, false, false), "they're fine");
        assert!(!can_help(false, true, true, false), "teams: only teammates");
        assert!(can_help(false, true, true, true));
    }

    #[test]
    fn drunk_camera_is_still_when_sober_and_sways_when_drunk() {
        assert_eq!(camera_fx(0.0, 3.0, 0.0, 0.0, 0.0), CamFx::default());
        let mut max_roll = 0.0f32;
        for i in 0..600 {
            max_roll = max_roll.max(camera_fx(1.0, i as f32 * 0.05, 0.0, 0.0, 0.0).roll.abs());
        }
        assert!(max_roll > 0.15 && max_roll < 0.25, "{max_roll}");
    }

    #[test]
    fn camera_drops_when_fallen_or_flat() {
        let fell = camera_fx(0.0, 0.0, 1.0, 0.0, 0.0);
        assert!((fell.dy + 1.25).abs() < 1e-5);
        let flat = camera_fx(0.0, 0.0, 0.0, 1.0, 0.2);
        assert!((flat.dy + 1.2).abs() < 1e-5);
        assert!((flat.pitch - 0.9).abs() < 1e-5, "flat looks up at the sky");
    }

    #[test]
    fn flat_amount_eases_in_and_out() {
        assert_eq!(flat_amount(0.0, 3.5), 0.0);
        assert!(flat_amount(3.45, 3.5) < 0.3, "just went down");
        assert_eq!(flat_amount(2.0, 3.5), 1.0);
        assert!(flat_amount(0.2, 3.5) < 0.5, "getting up");
    }
}
