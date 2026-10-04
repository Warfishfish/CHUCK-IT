//! How a character walks, jumps, boosts, bounces and bumps into the yard (spec section 2).
//!
//! Pure maths: the game hands in what the player wants (`MoveInput`) and what is going on
//! around them (`Modifiers`), and gets back where they end up.

use crate::yard::{POOL_SINK, TRAMP_H, TRAMP_R, TRAMP_X, TRAMP_Z, Yard, in_pool_rect};
use crate::{YARD_HALF_X, YARD_HALF_Z};

/// One fixed step in seconds.
pub fn step_dt() -> f32 {
    crate::sim::TICK_DT
}

pub const CHAR_GRAVITY: f32 = 24.0;
pub const CHAR_RADIUS: f32 = 0.42;
pub const BASE_SPEED: f32 = 6.2;
pub const JUMP_V: f32 = 7.5;
pub const POOL_JUMP_V: f32 = 5.5;
pub const JUMP_BUFFER: f32 = 0.14;
pub const COYOTE: f32 = 0.10;
pub const BOOST_MULT: f32 = 1.7;
pub const BOOST_TIME: f32 = 1.6;
pub const BOOST_CD: f32 = 5.0;
pub const TRAMP_MAX: u32 = 5;
pub const TRAMP_BASE_V: f32 = 11.5;
/// Characters closer than this get pushed apart.
pub const SEPARATE_DIST: f32 = 0.9;
/// Camera eye height above the feet.
pub const EYE_HEIGHT: f32 = 1.55;
pub const PITCH_LIMIT: f32 = 1.45;
/// Mouse sensitivity, radians per pixel.
pub const MOUSE_SENS: f32 = 0.0022;
pub const FOV_DEFAULT: f32 = 85.0;
pub const FOV_MIN: f32 = 60.0;
pub const FOV_MAX: f32 = 105.0;
/// Extra FOV while boosting.
pub const SPRINT_FOV: f32 = 7.0;

/// Grip values: how fast velocity eases to the target (spec section 2 table).
pub const GRIP_GROUND: f32 = 12.0;
pub const GRIP_AIR: f32 = 6.0;
pub const GRIP_AIR_STUNNED: f32 = 2.2;
pub const GRIP_STUNNED: f32 = 3.2;
pub const GRIP_SLIDE: f32 = 0.3;
pub const GRIP_PUDDLE: f32 = 1.1;
pub const GRIP_STUMBLE: f32 = 2.4;

/// Turn yaw plus stick input into a world-space wish direction (length at most 1).
/// `mx` is +1 for right, `my` is +1 for forward.
pub fn wish_dir(yaw: f32, mx: f32, my: f32) -> (f32, f32) {
    let wx = yaw.cos() * mx - yaw.sin() * my;
    let wz = -yaw.sin() * mx - yaw.cos() * my;
    let l = wx.hypot(wz);
    if l > 1.0 { (wx / l, wz / l) } else { (wx, wz) }
}

/// Drunk steering: the wish direction is rotated by a wandering angle.
pub fn drunk_steer(wish: (f32, f32), da: f32, t: f32) -> (f32, f32) {
    if da <= 0.0 || (wish.0 == 0.0 && wish.1 == 0.0) {
        return wish;
    }
    let th = da * ((t * 0.5).sin() * 0.5 + (t * 1.37 + 1.0).sin() * 0.2);
    let (s, c) = th.sin_cos();
    (wish.0 * c - wish.1 * s, wish.0 * s + wish.1 * c)
}

/// Apply a mouse movement (in pixels) to yaw and pitch.
pub fn look(yaw: f32, pitch: f32, dx: f32, dy: f32) -> (f32, f32) {
    (
        yaw - dx * MOUSE_SENS,
        (pitch - dy * MOUSE_SENS).clamp(-PITCH_LIMIT, PITCH_LIMIT),
    )
}

/// What the player is asking for this step.
#[derive(Clone, Copy, Debug, Default)]
pub struct MoveInput {
    /// World-space wish direction from `wish_dir` (and `drunk_steer`).
    pub wish: (f32, f32),
}

/// Things around the character that change how they move.
#[derive(Clone, Copy, Debug)]
pub struct Modifiers {
    pub stunned: bool,
    /// Countdown: nobody moves.
    pub frozen: bool,
    pub charging: bool,
    pub drinking: bool,
    pub snag_buff: bool,
    pub snag_stumble: bool,
    pub carrying: bool,
    pub carried: bool,
    pub at_smoko: bool,
    pub sliding: bool,
    pub in_puddle: bool,
    /// Drunk walking-speed wobble (1.0 when sober).
    pub gait: f32,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            stunned: false,
            frozen: false,
            charging: false,
            drinking: false,
            snag_buff: false,
            snag_stumble: false,
            carrying: false,
            carried: false,
            at_smoko: false,
            sliding: false,
            in_puddle: false,
            gait: 1.0,
        }
    }
}

/// What happened during a step (for sounds and camera shake).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MoveEvents {
    pub jumped: bool,
    pub landed: bool,
    /// Landed hard enough to shake the camera (`vel.y < -10`).
    pub hard_landing: bool,
    /// Trampoline bounce number in the chain (0 = first), if one happened.
    pub bounce: Option<u32>,
    pub splash: bool,
    /// Camera shake to add (take the max with the current shake).
    pub shake: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Mover {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub prev_y: f32,
    pub grounded: bool,
    pub coyote: f32,
    pub jump_buf: f32,
    pub boost_t: f32,
    pub boost_cd: f32,
    pub tramp_n: u32,
    pub tramp_chain: bool,
    pub in_pool: bool,
    was_in_pool: bool,
    /// How far you've sunk (eased), for the camera.
    pub sink: f32,
}

impl Mover {
    pub fn new(x: f32, z: f32) -> Self {
        Self {
            x,
            y: 0.0,
            z,
            vx: 0.0,
            vy: 0.0,
            vz: 0.0,
            prev_y: 0.0,
            grounded: true,
            coyote: 0.0,
            jump_buf: 0.0,
            boost_t: 0.0,
            boost_cd: 0.0,
            tramp_n: 0,
            tramp_chain: false,
            in_pool: false,
            was_in_pool: false,
            sink: 0.0,
        }
    }

    /// Space pressed: remember it for a moment so it never eats the press.
    pub fn try_jump(&mut self, stunned: bool) {
        if !stunned {
            self.jump_buf = JUMP_BUFFER;
        }
    }

    /// Shift pressed.
    pub fn try_boost(&mut self, m: &Modifiers) {
        if m.carried
            || m.at_smoko
            || self.boost_t > 0.0
            || self.boost_cd > 0.0
            || m.stunned
            || m.frozen
        {
            return;
        }
        self.boost_t = BOOST_TIME;
    }

    pub fn sprinting(&self, m: &Modifiers) -> bool {
        self.boost_t > 0.0 && !self.in_pool && !m.stunned && !m.frozen && !m.carried
    }

    pub fn max_speed(&self, m: &Modifiers) -> f32 {
        let f = |on: bool, v: f32| if on { v } else { 1.0 };
        BASE_SPEED
            * f(self.in_pool, 0.5)
            * f(m.charging, 0.72)
            * f(self.sprinting(m), BOOST_MULT)
            * f(m.drinking, 0.55)
            * f(m.snag_buff, 1.35)
            * f(m.snag_stumble, 0.62)
            * f(m.carrying, 0.55)
            * m.gait
    }

    /// One step of `dt` seconds.
    pub fn step(&mut self, dt: f32, input: MoveInput, m: &Modifiers, yard: &Yard) -> MoveEvents {
        let mut ev = MoveEvents::default();

        // pool
        self.in_pool = in_pool_rect(self.x, self.z) && self.y < 0.1;
        if self.in_pool && !self.was_in_pool {
            ev.splash = true;
        }
        self.was_in_pool = self.in_pool;

        // boost timer
        if self.boost_t > 0.0 {
            self.boost_t = (self.boost_t - dt).max(0.0);
            if self.boost_t <= 0.0 {
                self.boost_cd = BOOST_CD;
            }
        } else {
            self.boost_cd = (self.boost_cd - dt).max(0.0);
        }

        // velocity eases toward the target speed
        let max_sp = self.max_speed(m);
        let (mut tx, mut tz) = (input.wish.0 * max_sp, input.wish.1 * max_sp);
        if m.stunned || m.frozen {
            tx = 0.0;
            tz = 0.0;
        }
        let grip = if !self.grounded {
            if m.stunned {
                GRIP_AIR_STUNNED
            } else {
                GRIP_AIR
            }
        } else if m.stunned {
            GRIP_STUNNED
        } else if m.sliding {
            GRIP_SLIDE
        } else if m.in_puddle {
            GRIP_PUDDLE
        } else if m.snag_stumble {
            GRIP_STUMBLE
        } else {
            GRIP_GROUND
        };
        let k = 1.0 - (-grip * dt).exp();
        self.vx += (tx - self.vx) * k;
        self.vz += (tz - self.vz) * k;

        // jump, with a buffer and coyote time
        self.coyote = if self.grounded {
            COYOTE
        } else {
            (self.coyote - dt).max(0.0)
        };
        self.jump_buf = (self.jump_buf - dt).max(0.0);
        if self.jump_buf > 0.0 && self.coyote > 0.0 && !m.stunned && !m.frozen {
            self.vy = if self.in_pool { POOL_JUMP_V } else { JUMP_V };
            self.grounded = false;
            self.coyote = 0.0;
            self.jump_buf = 0.0;
            ev.jumped = true;
        }

        // move
        let fall_v = self.vy;
        self.prev_y = self.y;
        self.vy -= CHAR_GRAVITY * dt;
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.z += self.vz * dt;

        // push out of boxes (unless we're above them: then we land on top)
        let r = CHAR_RADIUS;
        for c in &yard.colliders {
            let skip = if c.no_top {
                self.y >= c.h + 1.5
            } else {
                self.y >= c.h - 0.05 || self.prev_y >= c.h - 0.05
            };
            if skip {
                continue;
            }
            let cx = self.x.clamp(c.x0, c.x1);
            let cz = self.z.clamp(c.z0, c.z1);
            let (dx, dz) = (self.x - cx, self.z - cz);
            let d2 = dx * dx + dz * dz;
            if d2 >= r * r {
                continue;
            }
            if d2 > 1e-6 {
                let d = d2.sqrt();
                let (nx, nz) = (dx / d, dz / d);
                self.x += nx * (r - d);
                self.z += nz * (r - d);
                let vn = self.vx * nx + self.vz * nz;
                if vn < 0.0 {
                    self.vx -= vn * nx;
                    self.vz -= vn * nz;
                }
            } else {
                // centre is inside the box: leave by the shortest way
                let (l, rt, b, f) = (self.x - c.x0, c.x1 - self.x, self.z - c.z0, c.z1 - self.z);
                let mn = l.min(rt).min(b).min(f);
                if mn == l {
                    self.x = c.x0 - r;
                } else if mn == rt {
                    self.x = c.x1 + r;
                } else if mn == b {
                    self.z = c.z0 - r;
                } else {
                    self.z = c.z1 + r;
                }
            }
        }

        // yard edge
        self.x = self.x.clamp(-YARD_HALF_X + 0.45, YARD_HALF_X - 0.45);
        self.z = self.z.clamp(-YARD_HALF_Z + 0.45, YARD_HALF_Z - 0.45);

        // floor, trampoline and landing
        let floor = yard.floor_at(self.x, self.z, self.y, self.prev_y);
        let (tdx, tdz) = (self.x - TRAMP_X, self.z - TRAMP_Z);
        if tdx * tdx + tdz * tdz < TRAMP_R * TRAMP_R && self.y <= TRAMP_H + 0.02 && self.vy <= 0.0 {
            let n = if self.tramp_chain {
                (self.tramp_n + 1).min(TRAMP_MAX)
            } else {
                0
            };
            self.tramp_n = n;
            self.tramp_chain = true;
            self.y = TRAMP_H;
            self.vy = TRAMP_BASE_V * (1.0 + 0.2 * n as f32);
            self.grounded = false;
            ev.bounce = Some(n);
            ev.shake = 0.03 + 0.015 * n as f32;
        } else if self.y <= floor {
            self.tramp_chain = false;
            self.tramp_n = 0;
            if self.vy < -10.0 {
                ev.hard_landing = true;
                ev.shake = ev.shake.max(0.05);
            }
            if !self.grounded && fall_v < -4.0 {
                ev.landed = true;
            }
            self.y = floor;
            self.vy = 0.0;
            self.grounded = true;
        } else {
            self.grounded = false;
        }

        // sink in the pool (eased)
        let target = if self.in_pool { POOL_SINK } else { 0.0 };
        self.sink += (target - self.sink) * (1.0 - (-7.0 * dt).exp());
        ev
    }

    pub fn speed(&self) -> f32 {
        self.vx.hypot(self.vz)
    }
}

/// Push two characters apart if they overlap (spec: 0.9 m, under 1.5 m apart vertically).
/// `a_moves` / `b_moves` say who may be moved (remote players never are).
pub fn separate(a: &mut Mover, b: &mut Mover, a_moves: bool, b_moves: bool) {
    if !a_moves && !b_moves {
        return;
    }
    let (dx, dz) = (b.x - a.x, b.z - a.z);
    let d = dx.hypot(dz);
    if d < SEPARATE_DIST && d > 1e-4 && (a.y - b.y).abs() < 1.5 {
        let p = (SEPARATE_DIST - d) / if a_moves && b_moves { 2.0 } else { 1.0 };
        let (nx, nz) = (dx / d, dz / d);
        if a_moves {
            a.x -= nx * p;
            a.z -= nz * p;
        }
        if b_moves {
            b.x += nx * p;
            b.z += nz * p;
        }
    }
}

/// Smoothly move the camera FOV toward its target (eased at `1 - exp(-8 dt)`).
pub fn ease_fov(current: f32, base: f32, sprinting: bool, dt: f32) -> f32 {
    let target = base.clamp(FOV_MIN, FOV_MAX) + if sprinting { SPRINT_FOV } else { 0.0 };
    current + (target - current) * (1.0 - (-8.0 * dt).exp())
}

/// Camera head-bob height while on the ground.
pub fn head_bob(walk_phase: f32, grounded: bool) -> f32 {
    if grounded {
        (walk_phase * 2.0).sin() * 0.035
    } else {
        0.0
    }
}

/// Advance the walk phase (used for bob and footsteps): `speed * dt * 2.4` while grounded and moving.
pub fn advance_walk(phase: f32, speed: f32, grounded: bool, dt: f32) -> f32 {
    if grounded && speed > 0.5 {
        phase + speed * dt * 2.4
    } else {
        phase
    }
}

/// Where the camera sits for a character (feet position, eye height, sink, bob).
pub fn eye_y(feet_y: f32, sink: f32, bob: f32) -> f32 {
    feet_y + EYE_HEIGHT - sink + bob
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::yard::Features;

    const DT: f32 = 1.0 / 60.0;

    fn settle(m: &mut Mover, yard: &Yard) {
        for _ in 0..30 {
            m.step(DT, MoveInput::default(), &Modifiers::default(), yard);
        }
    }

    fn walk(m: &mut Mover, yard: &Yard, wish: (f32, f32), secs: f32, mods: &Modifiers) {
        for _ in 0..(secs / DT) as usize {
            m.step(DT, MoveInput { wish }, mods, yard);
        }
    }

    #[test]
    fn wish_is_relative_to_where_you_look() {
        let (x, z) = wish_dir(0.0, 0.0, 1.0);
        assert!(x.abs() < 1e-6 && (z + 1.0).abs() < 1e-6); // forward is -z at yaw 0
        let (x, z) = wish_dir(0.0, 1.0, 0.0);
        assert!((x - 1.0).abs() < 1e-6 && z.abs() < 1e-6); // right is +x
        let (x, z) = wish_dir(0.0, 1.0, 1.0);
        assert!((x.hypot(z) - 1.0).abs() < 1e-5); // diagonal isn't faster
    }

    #[test]
    fn sober_steering_is_unchanged_and_drunk_steering_rotates() {
        assert_eq!(drunk_steer((1.0, 0.0), 0.0, 3.0), (1.0, 0.0));
        let (x, z) = drunk_steer((0.0, -1.0), 1.0, 2.0);
        assert!(x.abs() > 0.01);
        assert!((x.hypot(z) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn looking_clamps_pitch() {
        let (_, p) = look(0.0, 0.0, 0.0, -10_000.0);
        assert_eq!(p, PITCH_LIMIT);
        let (y, _) = look(0.0, 0.0, 100.0, 0.0);
        assert!((y + 0.22).abs() < 1e-5);
    }

    #[test]
    fn walking_reaches_about_6_2() {
        let yard = Yard::new(Features::default(), 0);
        let mut m = Mover::new(0.0, 0.0);
        settle(&mut m, &yard);
        walk(&mut m, &yard, (1.0, 0.0), 1.0, &Modifiers::default());
        assert!((m.speed() - 6.2).abs() < 0.05, "{}", m.speed());
    }

    #[test]
    fn modifiers_multiply_the_speed() {
        let m = Mover::new(0.0, 0.0);
        let mods = Modifiers {
            charging: true,
            drinking: true,
            ..Default::default()
        };
        assert!((m.max_speed(&mods) - 6.2 * 0.72 * 0.55).abs() < 1e-4);
        let mut b = Mover::new(0.0, 0.0);
        b.boost_t = 1.0;
        assert!((b.max_speed(&Modifiers::default()) - 6.2 * 1.7).abs() < 1e-4);
    }

    #[test]
    fn stunned_or_frozen_you_cannot_walk() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 0.0);
        settle(&mut m, &yard);
        walk(
            &mut m,
            &yard,
            (1.0, 0.0),
            1.0,
            &Modifiers {
                frozen: true,
                ..Default::default()
            },
        );
        assert!(m.speed() < 0.01);
    }

    #[test]
    fn jump_goes_up_and_lands() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 0.0);
        settle(&mut m, &yard);
        m.try_jump(false);
        let mut peak = 0.0f32;
        let mut jumped = false;
        for _ in 0..120 {
            let ev = m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
            jumped |= ev.jumped;
            peak = peak.max(m.y);
        }
        assert!(jumped);
        // v^2 / 2g = 1.17 m on paper; stepping gravity before moving at 60 Hz (as the
        // JavaScript game does) gives about 1.11 m.
        assert!((peak - 1.11).abs() < 0.03, "{peak}");
        assert!(m.grounded && m.y == 0.0);
    }

    #[test]
    fn jump_buffer_catches_an_early_press() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 0.0);
        settle(&mut m, &yard);
        m.try_jump(false);
        m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
        // in the air; press again just before landing
        for _ in 0..200 {
            if m.vy < 0.0 && m.y < 0.15 {
                m.try_jump(false);
            }
            let ev = m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
            if ev.jumped && m.vy > 0.0 {
                return;
            }
        }
        panic!("buffered jump never fired");
    }

    #[test]
    fn coyote_time_allows_a_jump_just_after_leaving_the_ground() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 0.0);
        settle(&mut m, &yard);
        m.grounded = false; // just stepped off an edge
        m.coyote = 0.08;
        m.y = 0.5;
        m.try_jump(false);
        let ev = m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
        assert!(ev.jumped);
    }

    #[test]
    fn no_jumping_when_stunned() {
        let mut m = Mover::new(0.0, 0.0);
        m.try_jump(true);
        assert_eq!(m.jump_buf, 0.0);
    }

    #[test]
    fn boost_lasts_1_6_then_5_s_cooldown() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 0.0);
        settle(&mut m, &yard);
        let mods = Modifiers::default();
        m.try_boost(&mods);
        assert!(m.boost_t > 0.0);
        let mut t = 0.0;
        while m.boost_t > 0.0 {
            m.step(DT, MoveInput::default(), &mods, &yard);
            t += DT;
        }
        assert!((t - 1.6).abs() < 0.05, "{t}");
        m.try_boost(&mods);
        assert_eq!(m.boost_t, 0.0, "cooldown blocks the next boost");
        assert!(m.boost_cd > 4.9);
        for _ in 0..(5.1 / DT) as usize {
            m.step(DT, MoveInput::default(), &mods, &yard);
        }
        m.try_boost(&mods);
        assert!(m.boost_t > 0.0);
    }

    #[test]
    fn boost_is_blocked_when_stunned_carried_smoko_or_frozen() {
        for mods in [
            Modifiers {
                stunned: true,
                ..Default::default()
            },
            Modifiers {
                carried: true,
                ..Default::default()
            },
            Modifiers {
                at_smoko: true,
                ..Default::default()
            },
            Modifiers {
                frozen: true,
                ..Default::default()
            },
        ] {
            let mut m = Mover::new(0.0, 0.0);
            m.try_boost(&mods);
            assert_eq!(m.boost_t, 0.0);
        }
    }

    #[test]
    fn yard_edge_stops_you() {
        let yard = Yard::default();
        let mut m = Mover::new(30.0, 0.0);
        settle(&mut m, &yard);
        walk(&mut m, &yard, (1.0, 0.0), 3.0, &Modifiers::default());
        assert!((m.x - (YARD_HALF_X - 0.45)).abs() < 1e-4);
    }

    #[test]
    fn walls_push_you_out_and_kill_the_velocity_into_them() {
        let yard = Yard::default();
        // brick wall at (20,-4), 4.5 x 0.35: walk into it from the south (+z side is z1 = -3.825)
        let mut m = Mover::new(20.0, 0.0);
        settle(&mut m, &yard);
        walk(&mut m, &yard, (0.0, -1.0), 3.0, &Modifiers::default());
        assert!(m.z >= -3.825 + 0.4, "z = {}", m.z);
        assert!(m.vz.abs() < 0.5);
    }

    #[test]
    fn you_can_jump_onto_a_low_box_and_stand_on_it() {
        let yard = Yard::default();
        // table at (6,-16.5), top 0.8: jump up while walking onto it
        let mut m = Mover::new(6.0, -14.3);
        settle(&mut m, &yard);
        let mods = Modifiers::default();
        for i in 0..240 {
            if i == 0 {
                m.try_jump(false);
            }
            m.step(DT, MoveInput { wish: (0.0, -1.0) }, &mods, &yard);
            if m.grounded && (m.y - 0.8).abs() < 1e-4 {
                return; // landed on top of the table
            }
        }
        panic!("never got on the table: y={} z={}", m.y, m.z);
    }

    #[test]
    fn the_pole_cannot_be_stood_on() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 3.0);
        m.y = 2.5;
        m.prev_y = 2.5;
        m.grounded = false;
        for _ in 0..120 {
            m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
        }
        assert_eq!(m.y, 0.0);
    }

    #[test]
    fn trampoline_bounces_get_higher_up_to_the_cap() {
        let yard = Yard::default();
        let mut m = Mover::new(TRAMP_X, TRAMP_Z);
        m.y = 1.0;
        m.prev_y = 1.0;
        m.grounded = false;
        let mut bounces = Vec::new();
        for _ in 0..(60 * 20) {
            let ev = m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
            if let Some(n) = ev.bounce {
                bounces.push((n, m.vy));
            }
        }
        assert!(bounces.len() >= 6);
        assert_eq!(bounces[0].0, 0);
        assert!((bounces[0].1 - 11.5).abs() < 1e-4);
        assert!((bounces[1].1 - 13.8).abs() < 1e-4);
        let last = bounces.last().unwrap();
        assert_eq!(last.0, TRAMP_MAX);
        assert!((last.1 - 23.0).abs() < 1e-4);
    }

    #[test]
    fn touching_the_grass_resets_the_trampoline_chain() {
        let yard = Yard::default();
        let mut m = Mover::new(TRAMP_X, TRAMP_Z);
        m.tramp_chain = true;
        m.tramp_n = 4;
        m.x = 0.0;
        m.z = 0.0;
        m.y = 0.5;
        m.grounded = false;
        for _ in 0..60 {
            m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
        }
        assert!(!m.tramp_chain && m.tramp_n == 0);
    }

    #[test]
    fn pool_halves_speed_sinks_and_lowers_the_jump() {
        let yard = Yard::default();
        let mut m = Mover::new(-20.0, 10.0);
        settle(&mut m, &yard);
        assert!(m.in_pool);
        walk(&mut m, &yard, (1.0, 0.0), 1.0, &Modifiers::default());
        assert!((m.speed() - 3.1).abs() < 0.05, "{}", m.speed());
        assert!((m.sink - POOL_SINK).abs() < 0.05);
        m.try_jump(false);
        m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
        assert!((m.vy - (POOL_JUMP_V - CHAR_GRAVITY * DT)).abs() < 1e-4);
    }

    #[test]
    fn grip_is_loose_on_a_slide() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 0.0);
        settle(&mut m, &yard);
        m.vx = 8.0;
        let mods = Modifiers {
            sliding: true,
            ..Default::default()
        };
        walk(&mut m, &yard, (0.0, 0.0), 0.5, &mods);
        assert!(m.vx > 6.5, "slid only to {}", m.vx);
    }

    #[test]
    fn characters_push_each_other_apart() {
        let mut a = Mover::new(0.0, 0.0);
        let mut b = Mover::new(0.5, 0.0);
        separate(&mut a, &mut b, true, true);
        assert!((b.x - a.x - 0.9).abs() < 1e-5);
        // remote players are not moved
        let mut c = Mover::new(0.0, 0.0);
        let mut d = Mover::new(0.5, 0.0);
        separate(&mut c, &mut d, true, false);
        assert_eq!(d.x, 0.5);
        assert!((c.x - (-0.4)).abs() < 1e-5);
        // far apart vertically: no push
        let mut e = Mover::new(0.0, 0.0);
        let mut f = Mover::new(0.5, 0.0);
        f.y = 2.0;
        separate(&mut e, &mut f, true, true);
        assert_eq!(f.x, 0.5);
    }

    #[test]
    fn fov_widens_when_boosting_and_eases() {
        let f1 = ease_fov(85.0, 85.0, true, DT);
        assert!(f1 > 85.0 && f1 < 92.0);
        let mut f = 85.0;
        for _ in 0..120 {
            f = ease_fov(f, 85.0, true, DT);
        }
        assert!((f - 92.0).abs() < 0.01);
        assert_eq!(ease_fov(85.0, 200.0, false, 10.0).round(), 105.0);
    }

    #[test]
    fn bob_and_eye_height() {
        assert_eq!(head_bob(1.0, false), 0.0);
        assert!(head_bob(0.4, true).abs() <= 0.035);
        assert!((eye_y(0.0, 0.0, 0.0) - 1.55).abs() < 1e-6);
        assert!((eye_y(0.0, 1.0, 0.0) - 0.55).abs() < 1e-6);
    }

    #[test]
    fn hard_landing_shakes_the_camera() {
        let yard = Yard::default();
        let mut m = Mover::new(0.0, 0.0);
        m.y = 20.0;
        m.prev_y = 20.0;
        m.grounded = false;
        let mut shook = false;
        for _ in 0..200 {
            let ev = m.step(DT, MoveInput::default(), &Modifiers::default(), &yard);
            shook |= ev.hard_landing && ev.shake >= 0.05;
        }
        assert!(shook);
    }
}
