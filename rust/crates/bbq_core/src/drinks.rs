//! Drinks, the drunk meter, stacking it, and the Drunk-mode walk (spec section 4).

use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drink {
    /// VP (beer), left third of the bar.
    Beer,
    /// Wine, the middle.
    Wine,
    /// Rum shot, right third.
    Rum,
}

impl Drink {
    /// How much it adds to the 0..100 meter.
    pub fn amount(self) -> f32 {
        match self {
            Drink::Beer => 18.0,
            Drink::Wine => 26.0,
            Drink::Rum => 34.0,
        }
    }
    /// Seconds spent drinking it.
    pub fn time(self) -> f32 {
        match self {
            Drink::Beer => 1.4,
            Drink::Wine => 1.2,
            Drink::Rum => 0.6,
        }
    }
    /// Which drink you get standing at `dx` metres either side of the bar centre.
    pub fn at_bar(dx: f32) -> Drink {
        if dx < -0.6 {
            Drink::Beer
        } else if dx > 0.6 {
            Drink::Rum
        } else {
            Drink::Wine
        }
    }
}

/// Drunk points lost per second.
pub const SOBER_RATE: f32 = 1.6;
/// Drunk mode keeps everybody at least this drunk.
pub const DRUNK_ALL: f32 = 78.0;
/// Drunk meter gained per second sitting at smoko.
pub const SMOKO_SIP: f32 = 4.0;
/// Walking speed while drinking.
pub const DRINK_SPEED: f32 = 0.55;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tier {
    Sober,
    Tipsy,
    Drunk,
    Maggot,
    AbsolutelyMaggoted,
}

pub fn drunk_tier(d: f32) -> Tier {
    if d < 15.0 {
        Tier::Sober
    } else if d < 40.0 {
        Tier::Tipsy
    } else if d < 70.0 {
        Tier::Drunk
    } else if d < 90.0 {
        Tier::Maggot
    } else {
        Tier::AbsolutelyMaggoted
    }
}

/// 0 below 15 on the meter, easing to 1 at 85 (a smoothstep). Drives steering twist,
/// sway, bot aim error and so on.
pub fn drunk_amt(d: f32) -> f32 {
    let x = ((d - 15.0) / 70.0).clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Bonus points for hitting someone while drunk. Always 0 in Drunk mode.
pub fn drunk_bonus(drunk: f32, drunk_mode: bool) -> i32 {
    if drunk_mode {
        0
    } else if drunk >= 40.0 {
        50
    } else if drunk >= 15.0 {
        25
    } else {
        0
    }
}

/// Chance **per second** of stacking it in normal play (the player must be drunk enough:
/// `drunk_amt >= 0.35`). Only checked for a grounded, upright player.
pub fn fall_chance_per_sec(da: f32, moving: bool) -> f32 {
    if da < 0.35 {
        0.0
    } else {
        da.powf(1.5) * if moving { 0.028 } else { 0.008 }
    }
}

/// In Drunk mode: every full 2 seconds of walking there is a 4% chance of stacking it.
pub const DRUNK_MODE_FALL_EVERY: f32 = 2.0;
pub const DRUNK_MODE_FALL_CHANCE: f32 = 0.04;
/// How long you stay down after stacking it (the host can pick 20 or 30).
pub const FALL_DURATION_DEFAULT: f32 = 10.0;
/// Immunity from falling after you get up (6 s in Drunk mode).
pub const FALL_IMMUNE: f32 = 25.0;
pub const FALL_IMMUNE_DRUNK_MODE: f32 = 6.0;

/// Tracks the "every 2 seconds of walking" Drunk-mode fall roll.
#[derive(Clone, Debug, Default)]
pub struct DrunkModeFall {
    walked: f32,
}

impl DrunkModeFall {
    /// Call every tick. Returns true on the tick that the player stacks it.
    pub fn tick(&mut self, dt: f32, walking: bool, rng: &mut Rng) -> bool {
        if !walking {
            return false;
        }
        self.walked += dt;
        if self.walked >= DRUNK_MODE_FALL_EVERY {
            self.walked = 0.0;
            return rng.chance(DRUNK_MODE_FALL_CHANCE);
        }
        false
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GaitState {
    Steady,
    Lurch,
    Slow,
}

/// The uneven Drunk-mode walking speed: steady, then a lurch forward (up to x1.32)
/// or a slow-down to get balance back (x0.5), scaled by how drunk you are.
#[derive(Clone, Debug)]
pub struct Gait {
    state: GaitState,
    time_left: f32,
    k: f32,
}

impl Gait {
    pub fn new(rng: &mut Rng) -> Self {
        Gait {
            state: GaitState::Steady,
            time_left: rng.range(1.0, 3.0),
            k: 1.0,
        }
    }

    /// Speed multiplier for this tick. `t` is the game time and `phase` the character's
    /// own random phase (so two drunks don't wobble in step).
    pub fn multiplier(&mut self, dt: f32, da: f32, t: f32, phase: f32, rng: &mut Rng) -> f32 {
        if da <= 0.0 {
            return 1.0;
        }
        self.time_left -= dt;
        if self.time_left <= 0.0 {
            if self.state != GaitState::Steady {
                self.state = GaitState::Steady;
                self.time_left = rng.range(1.0, 3.0);
            } else {
                let r = rng.f32();
                if r < 0.42 {
                    self.state = GaitState::Lurch;
                    self.time_left = rng.range(0.3, 0.7);
                } else if r < 0.8 {
                    self.state = GaitState::Slow;
                    self.time_left = rng.range(0.4, 1.1);
                } else {
                    self.time_left = rng.range(0.6, 1.8);
                }
            }
        }
        let wave = 1.0 + 0.12 * (t * 2.1 + phase).sin() + 0.07 * (t * 5.3 + phase * 2.0).sin();
        let target = match self.state {
            GaitState::Lurch => 1.32,
            GaitState::Slow => 0.5,
            GaitState::Steady => 0.88,
        } * wave;
        let rate = if self.state == GaitState::Lurch {
            10.0
        } else {
            5.0
        };
        self.k += (target - self.k) * (1.0 - (-rate * dt).exp());
        (1.0 + (self.k - 1.0) * da).clamp(0.35, 1.45)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiers() {
        assert_eq!(drunk_tier(0.0), Tier::Sober);
        assert_eq!(drunk_tier(14.9), Tier::Sober);
        assert_eq!(drunk_tier(15.0), Tier::Tipsy);
        assert_eq!(drunk_tier(40.0), Tier::Drunk);
        assert_eq!(drunk_tier(70.0), Tier::Maggot);
        assert_eq!(drunk_tier(90.0), Tier::AbsolutelyMaggoted);
    }

    #[test]
    fn drunk_amt_is_a_smoothstep_from_15_to_85() {
        assert_eq!(drunk_amt(0.0), 0.0);
        assert_eq!(drunk_amt(15.0), 0.0);
        assert!((drunk_amt(50.0) - 0.5).abs() < 1e-5);
        assert_eq!(drunk_amt(85.0), 1.0);
        assert_eq!(drunk_amt(100.0), 1.0);
    }

    #[test]
    fn drunk_bonus_thresholds_and_drunk_mode() {
        assert_eq!(drunk_bonus(10.0, false), 0);
        assert_eq!(drunk_bonus(15.0, false), 25);
        assert_eq!(drunk_bonus(39.9, false), 25);
        assert_eq!(drunk_bonus(40.0, false), 50);
        assert_eq!(drunk_bonus(90.0, true), 0);
    }

    #[test]
    fn drinks() {
        assert_eq!(Drink::at_bar(-1.2), Drink::Beer);
        assert_eq!(Drink::at_bar(0.0), Drink::Wine);
        assert_eq!(Drink::at_bar(1.2), Drink::Rum);
        assert_eq!(Drink::Rum.amount(), 34.0);
        assert_eq!(Drink::Rum.time(), 0.6);
    }

    #[test]
    fn falls_need_to_be_drunk_enough() {
        assert_eq!(fall_chance_per_sec(0.3, true), 0.0);
        assert!((fall_chance_per_sec(1.0, true) - 0.028).abs() < 1e-6);
        assert!((fall_chance_per_sec(1.0, false) - 0.008).abs() < 1e-6);
    }

    #[test]
    fn drunk_mode_falls_roughly_4_percent_per_2_seconds() {
        let mut rng = Rng::new(42);
        let mut fall = DrunkModeFall::default();
        let (mut rolls, mut falls) = (0, 0);
        for _ in 0..200_000 {
            // 2 seconds at 60 Hz per roll
            for _ in 0..120 {
                if fall.tick(1.0 / 60.0, true, &mut rng) {
                    falls += 1;
                }
            }
            rolls += 1;
            if rolls >= 2000 {
                break;
            }
        }
        let rate = falls as f32 / rolls as f32;
        assert!((0.025..0.06).contains(&rate), "rate was {rate}");
        // standing still never rolls
        let mut still = DrunkModeFall::default();
        for _ in 0..10_000 {
            assert!(!still.tick(1.0 / 60.0, false, &mut rng));
        }
    }

    #[test]
    fn gait_stays_inside_its_limits_and_is_slower_than_sober_on_average() {
        let mut rng = Rng::new(3);
        let mut gait = Gait::new(&mut rng);
        let (mut sum, mut n) = (0.0, 0);
        for i in 0..60 * 600 {
            let t = i as f32 / 60.0;
            let m = gait.multiplier(1.0 / 60.0, 1.0, t, 1.0, &mut rng);
            assert!((0.35..=1.45).contains(&m));
            sum += m;
            n += 1;
        }
        let avg = sum / n as f32;
        assert!(avg < 1.0, "average was {avg}");
        // sober: always exactly 1
        assert_eq!(gait.multiplier(1.0 / 60.0, 0.0, 0.0, 0.0, &mut rng), 1.0);
    }
}
