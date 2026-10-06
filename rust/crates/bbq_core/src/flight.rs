//! Items lying about, flying through the air, bouncing, smashing and floating
//! (spec section 3, "Item flight and landing", and the puddle and slide rules).

use crate::items::ItemKind;
use crate::rng::Rng;
use crate::vec::V3;
use crate::yard::{Collider, TRAMP_H, TRAMP_R, TRAMP_X, TRAMP_Z, WATER_Y, Yard, in_pool_rect};
use crate::{PlayerId, YARD_HALF_X, YARD_HALF_Z};

pub type ItemId = u32;

/// Gravity for items (each item multiplies it by its own `grav`).
pub const ITEM_GRAV: f32 = 18.0;
/// Largest distance an item moves in one sub-step, so it can't tunnel through things.
pub const MAX_SUBSTEP: f32 = 0.12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemState {
    Ground,
    Held,
    Flying,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub id: ItemId,
    pub kind: ItemKind,
    pub pos: V3,
    pub vel: V3,
    pub state: ItemState,
    /// A flying item that can still hit someone. Goes false once it lands.
    pub live: bool,
    /// Seconds it has been lying on the ground.
    pub rest_t: f32,
    pub fly_t: f32,
    /// Height of the surface it's resting on.
    pub ground_y: f32,
    pub holder: Option<PlayerId>,
    pub thrower: Option<PlayerId>,
    /// Thrown at power-throw charge.
    pub power: bool,
    /// Charge it was thrown with (for the long-shot bonus).
    pub charge: f32,
    /// Where the throw started (for the long-shot bonus).
    pub start: V3,
    /// Players this flight has already passed through.
    pub ignore: Vec<PlayerId>,
    pub uses: Option<u32>,
    pub variant: Option<crate::items::DildoVariant>,
    /// Heist: which team's teddy this is.
    pub team: Option<u8>,
    /// Someone who dropped it on purpose can't pick it up again until this time (seconds of game time).
    pub block: Option<(PlayerId, f32)>,
    /// Counts up each time it is thrown, so late hit reports can be told apart.
    pub flight_no: u32,
    /// How many times it has bounced on the trampoline since it was last thrown (teddies bounce
    /// like a ball and settle; see `collide_world`).
    pub bounces: u8,
}

impl Item {
    pub fn new(id: ItemId, kind: ItemKind, pos: V3) -> Self {
        Self {
            id,
            kind,
            pos,
            vel: V3::ZERO,
            state: ItemState::Ground,
            live: false,
            rest_t: 0.0,
            fly_t: 0.0,
            ground_y: 0.0,
            holder: None,
            thrower: None,
            power: false,
            charge: 0.0,
            start: V3::ZERO,
            ignore: Vec::new(),
            uses: kind.starting_uses(),
            variant: None,
            team: None,
            block: None,
            flight_no: 0,
            bounces: 0,
        }
    }

    pub fn radius(&self) -> f32 {
        self.kind.def().radius
    }
}

/// Things that happen to an item that the game should show or sound.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FlightEvent {
    Thud {
        pos: V3,
        gnome: bool,
    },
    Boing {
        pos: V3,
    },
    Splash {
        x: f32,
        z: f32,
    },
    /// A stubby broke. `puddle` says whether it left one.
    Smash {
        pos: V3,
        puddle: bool,
    },
    /// Left the yard. The thrower gets an "over" event. Heist teddies go home instead.
    Over {
        thrower: Option<PlayerId>,
    },
    /// Fell out of the world.
    Lost,
    /// Came to rest on a surface.
    Rested,
}

/// What happened to the item as a whole during a call.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fate {
    /// Still around (flying or resting).
    Alive,
    /// Gone (smashed, left the yard, fell out). The caller removes it.
    Removed,
}

/// Height an item sits at when resting on a surface of height `h` (spec: "Rest heights").
pub fn rest_height(kind: ItemKind) -> f32 {
    match kind {
        ItemKind::Stubby => 0.07,
        ItemKind::Gnome => 0.28,
        ItemKind::Dildo => 0.14,
        ItemKind::Noodle => 0.08,
        ItemKind::Steak => 0.03,
        ItemKind::Fish => 0.17,
        other => other.def().radius,
    }
}

/// Number of sub-steps and the length of each for a flying item over `dt`.
pub fn substeps(speed: f32, dt: f32) -> (u32, f32) {
    let steps = ((speed * dt / MAX_SUBSTEP).ceil() as u32).max(1);
    (steps, dt / steps as f32)
}

/// Put an item to rest at height `h`. If the surface is high (above 1.3 m) and `on` is a box,
/// it gets shoved off instead. Returns false if it was shoved.
pub fn rest_item(
    it: &mut Item,
    h: f32,
    on: Option<&Collider>,
    events: &mut Vec<FlightEvent>,
) -> bool {
    if h > 1.3
        && let Some(c) = on
    {
        let (cx, cz) = ((c.x0 + c.x1) / 2.0, (c.z0 + c.z1) / 2.0);
        let (dx, dz) = (it.pos.x - cx, it.pos.z - cz);
        let l = dx.hypot(dz);
        let l = if l == 0.0 { 1.0 } else { l };
        it.vel = V3::new(dx / l * 4.0, 2.5, dz / l * 4.0);
        return false;
    }
    it.state = ItemState::Ground;
    it.vel = V3::ZERO;
    it.rest_t = 0.0;
    it.ground_y = h;
    it.live = false;
    it.thrower = None;
    it.ignore.clear();
    it.pos.y = h + rest_height(it.kind);
    events.push(FlightEvent::Rested);
    true
}

fn hit_wall(it: &mut Item, axis: Axis, events: &mut Vec<FlightEvent>) -> Fate {
    let d = it.kind.def();
    let sp = it.vel.len();
    if d.smash && sp > 4.0 {
        smash(it, events);
        return Fate::Removed;
    }
    let k = -(d.bounce + 0.15);
    match axis {
        Axis::X => it.vel.x *= k,
        Axis::Z => it.vel.z *= k,
    }
    it.vel *= 0.8;
    if sp > 5.0 {
        events.push(FlightEvent::Thud {
            pos: it.pos,
            gnome: it.kind == ItemKind::Gnome,
        });
    }
    Fate::Alive
}

#[derive(Clone, Copy)]
enum Axis {
    X,
    Z,
}

fn smash(it: &Item, events: &mut Vec<FlightEvent>) {
    let puddle = it.pos.y < 1.4 && !in_pool_rect(it.pos.x, it.pos.z);
    events.push(FlightEvent::Smash {
        pos: it.pos,
        puddle,
    });
}

/// The item landed on a surface of height `h` (0 for the ground).
fn on_surface(it: &mut Item, h: f32, on: Option<&Collider>, events: &mut Vec<FlightEvent>) -> Fate {
    let d = it.kind.def();
    let sp = it.vel.len();
    if it.pos.x.abs() > YARD_HALF_X || it.pos.z.abs() > YARD_HALF_Z {
        events.push(FlightEvent::Over {
            thrower: it.thrower,
        });
        return Fate::Removed;
    }
    if h == 0.0 && in_pool_rect(it.pos.x, it.pos.z) {
        if sp > 2.0 {
            events.push(FlightEvent::Splash {
                x: it.pos.x,
                z: it.pos.z,
            });
        }
        if d.smash {
            return Fate::Removed; // a stubby in the pool just sinks, no puddle
        }
        rest_item(it, 0.02, None, events);
        it.pos.y = WATER_Y + 0.04;
        return Fate::Alive;
    }
    if d.smash && sp > 5.0 {
        smash(it, events);
        return Fate::Removed;
    }
    if sp > 5.0 {
        events.push(FlightEvent::Thud {
            pos: V3::new(it.pos.x, h + 0.05, it.pos.z),
            gnome: it.kind == ItemKind::Gnome,
        });
    }
    it.live = false;
    it.pos.y = h + d.radius;
    if it.vel.y < -2.5 {
        it.vel.y = -it.vel.y * d.bounce;
        it.vel.x *= 0.6;
        it.vel.z *= 0.6;
    } else {
        it.vel.y = 0.0;
        it.vel.x *= 0.85;
        it.vel.z *= 0.85;
        if it.vel.horiz_len() < 1.2 {
            rest_item(it, h, on, events);
        }
    }
    Fate::Alive
}

/// A teddy's bounces on the trampoline after the first keep this share of the speed...
const TEDDY_BOUNCE: f32 = 0.8;
/// ...and it settles on the mat once it lands slower than this.
const TEDDY_SETTLE: f32 = 2.2;

/// Bump the flying item against the world after one sub-step.
pub fn collide_world(
    it: &mut Item,
    yard: &Yard,
    rng: &mut Rng,
    events: &mut Vec<FlightEvent>,
) -> Fate {
    let r = it.radius();
    // trampoline
    let (tx, tz) = (it.pos.x - TRAMP_X, it.pos.z - TRAMP_Z);
    if tx * tx + tz * tz < TRAMP_R * TRAMP_R
        && it.pos.y < TRAMP_H + r
        && it.pos.y > TRAMP_H - 0.2
        && it.vel.y < 0.0
    {
        it.pos.y = TRAMP_H + r;
        events.push(FlightEvent::Boing { pos: it.pos });
        if it.kind == ItemKind::Teddy {
            // A teddy bounces like a ball: the first bounce is the trampoline's own kick, then
            // each one is a bit lower and stays on the mat (it loses most of its sideways
            // speed), until it settles and sits on the mat.
            let incoming = -it.vel.y;
            if it.bounces > 0 && incoming < TEDDY_SETTLE {
                it.bounces = 0;
                let (x, z) = (it.pos.x, it.pos.z);
                rest_item(it, TRAMP_H, None, events);
                it.pos.x = x;
                it.pos.z = z;
                return Fate::Alive;
            }
            it.vel.y = if it.bounces == 0 { (incoming * 0.9).max(7.0) } else { incoming * TEDDY_BOUNCE };
            it.vel.x = it.vel.x * 0.5 + rng.range(-0.3, 0.3);
            it.vel.z = it.vel.z * 0.5 + rng.range(-0.3, 0.3);
            it.bounces = it.bounces.saturating_add(1);
            return Fate::Alive;
        }
        it.vel.y = (-it.vel.y * 0.9).max(7.0);
        it.vel.x += rng.range(-1.0, 1.0);
        it.vel.z += rng.range(-1.0, 1.0);
        return Fate::Alive;
    }
    // the ground (or the pool's water)
    let floor_y = r + if in_pool_rect(it.pos.x, it.pos.z) {
        WATER_Y - 0.05
    } else {
        0.0
    };
    if it.pos.y < floor_y {
        if on_surface(it, 0.0, None, events) == Fate::Removed {
            return Fate::Removed;
        }
        if it.state != ItemState::Flying {
            return Fate::Alive;
        }
    }
    // boxes
    for c in &yard.colliders {
        if it.pos.y - r > c.h {
            continue;
        }
        if it.pos.x < c.x0 - r || it.pos.x > c.x1 + r || it.pos.z < c.z0 - r || it.pos.z > c.z1 + r
        {
            continue;
        }
        let px = (it.pos.x - (c.x0 - r)).min((c.x1 + r) - it.pos.x);
        let pz = (it.pos.z - (c.z0 - r)).min((c.z1 + r) - it.pos.z);
        let py = (c.h + r) - it.pos.y;
        if py <= px && py <= pz && it.vel.y <= 0.0 {
            it.pos.y = c.h + r;
            if on_surface(it, c.h, Some(c), events) == Fate::Removed {
                return Fate::Removed;
            }
            if it.state != ItemState::Flying {
                return Fate::Alive;
            }
        } else if px < pz {
            it.pos.x = if it.pos.x < (c.x0 + c.x1) / 2.0 {
                c.x0 - r
            } else {
                c.x1 + r
            };
            if hit_wall(it, Axis::X, events) == Fate::Removed {
                return Fate::Removed;
            }
        } else {
            it.pos.z = if it.pos.z < (c.z0 + c.z1) / 2.0 {
                c.z0 - r
            } else {
                c.z1 + r
            };
            if hit_wall(it, Axis::Z, events) == Fate::Removed {
                return Fate::Removed;
            }
        }
    }
    // the fence bounces things back (below 1.8 m)
    if it.pos.x.abs() > YARD_HALF_X - r && it.pos.x.abs() < YARD_HALF_X + 0.5 && it.pos.y < 1.8 + r
    {
        it.pos.x = it.pos.x.signum() * (YARD_HALF_X - r);
        if hit_wall(it, Axis::X, events) == Fate::Removed {
            return Fate::Removed;
        }
    }
    if it.pos.z.abs() > YARD_HALF_Z - r && it.pos.z.abs() < YARD_HALF_Z + 0.5 && it.pos.y < 1.8 + r
    {
        it.pos.z = it.pos.z.signum() * (YARD_HALF_Z - r);
        if hit_wall(it, Axis::Z, events) == Fate::Removed {
            return Fate::Removed;
        }
    }
    // out of the world
    if it.pos.y < -4.0 || it.pos.x.abs() > 60.0 || it.pos.z.abs() > 60.0 {
        events.push(FlightEvent::Lost);
        return Fate::Removed;
    }
    Fate::Alive
}

/// Move a flying item by `dt`. `hit_check` runs after each sub-step while the item is live and
/// returns true if the item was used up by a hit (the caller has changed its state).
pub fn step_flight(
    it: &mut Item,
    dt: f32,
    yard: &Yard,
    rng: &mut Rng,
    events: &mut Vec<FlightEvent>,
    hit_check: &mut dyn FnMut(&mut Item) -> bool,
) -> Fate {
    it.fly_t += dt;
    let (steps, h) = substeps(it.vel.len(), dt);
    for _ in 0..steps {
        if it.state != ItemState::Flying {
            break;
        }
        it.vel.y -= ITEM_GRAV * it.kind.def().grav * h;
        it.pos += it.vel * h;
        if it.live {
            hit_check(it);
        }
        if it.state != ItemState::Flying {
            break;
        }
        if collide_world(it, yard, rng, events) == Fate::Removed {
            return Fate::Removed;
        }
    }
    Fate::Alive
}

/// Spin speed for a flying item, radians per second (spec: `min(18, speed*0.6)`, gnome x0.6).
pub fn spin_rate(kind: ItemKind, speed: f32) -> f32 {
    speed.mul_add(0.6, 0.0).min(18.0) * if kind == ItemKind::Gnome { 0.6 } else { 1.0 }
}

// --- puddles and sliding ---------------------------------------------------------------

/// A spill left by a smashed stubby.
#[derive(Clone, Copy, Debug)]
pub struct Puddle {
    pub x: f32,
    pub z: f32,
    pub r: f32,
    pub life: f32,
    pub grow: f32,
}

pub const PUDDLE_LIFE: f32 = 14.0;
pub const SLIDE_T: f32 = 1.1;

impl Puddle {
    pub fn new(x: f32, z: f32, rng: &mut Rng) -> Self {
        Self {
            x,
            z,
            r: rng.range(1.1, 1.5),
            life: PUDDLE_LIFE,
            grow: 0.0,
        }
    }

    /// Current radius (it grows from 20% to full in a quarter of a second).
    pub fn size(&self) -> f32 {
        self.r * (0.2 + 0.8 * self.grow)
    }

    /// 0..1: fades during the last 2 seconds.
    pub fn opacity(&self) -> f32 {
        (self.life / 2.0).clamp(0.0, 1.0)
    }

    /// Is a character standing at (x, z) on the grass in it? Uses `r * grow`, as the JS does.
    pub fn covers(&self, x: f32, z: f32) -> bool {
        (x - self.x).hypot(z - self.z) < self.r * self.grow
    }
}

#[derive(Clone, Debug, Default)]
pub struct Puddles(pub Vec<Puddle>);

impl Puddles {
    pub fn tick(&mut self, dt: f32) {
        for p in &mut self.0 {
            p.life -= dt;
            p.grow = (p.grow + dt * 4.0).min(1.0);
        }
        self.0.retain(|p| p.life > 0.0);
    }

    pub fn slip_at(&self, x: f32, z: f32, y: f32) -> bool {
        y < 0.1 && self.0.iter().any(|p| p.covers(x, z))
    }
}

/// Tracks slipping on puddles for one character.
#[derive(Clone, Copy, Debug, Default)]
pub struct Slider {
    pub slide_t: f32,
    pub was_slip: bool,
    /// Spin direction and speed for the animation (rad/s), set when a slide starts.
    pub spin: f32,
}

/// What a slide check decided.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlideStart {
    /// Multiply horizontal velocity by this.
    pub speed_mult: f32,
}

impl Slider {
    /// Call once per step. `slip` = standing in a puddle now. Returns `Some` when a slide starts
    /// (also cancel any wind-up when it does).
    #[allow(clippy::too_many_arguments)]
    pub fn tick(
        &mut self,
        dt: f32,
        slip: bool,
        speed: f32,
        grounded: bool,
        seated: bool,
        down: bool,
        rng: &mut Rng,
    ) -> Option<SlideStart> {
        self.slide_t = (self.slide_t - dt).max(0.0);
        let mut out = None;
        if slip
            && !self.was_slip
            && grounded
            && speed > 2.4
            && self.slide_t <= 0.0
            && !seated
            && !down
        {
            self.slide_t = SLIDE_T;
            self.spin = if rng.chance(0.5) { -1.0 } else { 1.0 } * rng.range(7.0, 11.0);
            out = Some(SlideStart {
                speed_mult: (11.0f32).min(speed * 1.35) / speed,
            });
        }
        self.was_slip = slip;
        out
    }

    pub fn sliding(&self) -> bool {
        self.slide_t > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn thrown(kind: ItemKind, pos: V3, vel: V3) -> Item {
        let mut it = Item::new(1, kind, pos);
        it.state = ItemState::Flying;
        it.live = true;
        it.vel = vel;
        it
    }

    fn fly(it: &mut Item, yard: &Yard, secs: f32) -> (Fate, Vec<FlightEvent>) {
        let mut rng = Rng::new(7);
        let mut ev = Vec::new();
        let mut fate = Fate::Alive;
        for _ in 0..(secs / DT) as usize {
            fate = step_flight(it, DT, yard, &mut rng, &mut ev, &mut |_| false);
            if fate == Fate::Removed || it.state == ItemState::Ground {
                break;
            }
        }
        (fate, ev)
    }

    #[test]
    fn a_fast_item_takes_more_substeps() {
        assert_eq!(substeps(1.0, DT).0, 1);
        let (n, h) = substeps(27.0, DT);
        assert_eq!(n, 4); // 27/60 = 0.45 m -> 4 steps of <= 0.12
        assert!((h * n as f32 - DT).abs() < 1e-6);
    }

    #[test]
    fn a_teddy_falls_lands_and_comes_to_rest() {
        let yard = Yard::default();
        let mut it = thrown(
            ItemKind::Teddy,
            V3::new(0.0, 2.0, 10.0),
            V3::new(3.0, 0.0, 0.0),
        );
        let (fate, ev) = fly(&mut it, &yard, 10.0);
        assert_eq!(fate, Fate::Alive);
        assert_eq!(it.state, ItemState::Ground);
        assert!(!it.live);
        assert!((it.pos.y - 0.24).abs() < 1e-4); // teddy rests at its radius
        assert!(ev.contains(&FlightEvent::Rested));
    }

    #[test]
    fn landing_makes_it_harmless() {
        let yard = Yard::default();
        let mut it = thrown(ItemKind::Gnome, V3::new(0.0, 0.5, 10.0), V3::ZERO);
        let mut rng = Rng::new(1);
        let mut ev = Vec::new();
        for _ in 0..30 {
            step_flight(&mut it, DT, &yard, &mut rng, &mut ev, &mut |_| false);
        }
        assert!(!it.live);
    }

    #[test]
    fn a_fast_stubby_smashes_on_the_ground_and_leaves_a_puddle() {
        let yard = Yard::default();
        let mut it = thrown(
            ItemKind::Stubby,
            V3::new(0.0, 1.0, 10.0),
            V3::new(0.0, -8.0, 0.0),
        );
        let (fate, ev) = fly(&mut it, &yard, 2.0);
        assert_eq!(fate, Fate::Removed);
        assert!(
            ev.iter()
                .any(|e| matches!(e, FlightEvent::Smash { puddle: true, .. }))
        );
    }

    #[test]
    fn a_stubby_that_lands_gently_does_not_smash() {
        let yard = Yard::default();
        let mut it = thrown(ItemKind::Stubby, V3::new(0.0, 0.3, 10.0), V3::ZERO);
        let (fate, ev) = fly(&mut it, &yard, 3.0);
        assert_eq!(fate, Fate::Alive);
        assert!(!ev.iter().any(|e| matches!(e, FlightEvent::Smash { .. })));
        assert_eq!(it.state, ItemState::Ground);
    }

    #[test]
    fn a_stubby_in_the_pool_is_removed_without_a_puddle() {
        let yard = Yard::default();
        let mut it = thrown(
            ItemKind::Stubby,
            V3::new(-20.0, 1.0, 10.0),
            V3::new(0.0, -10.0, 0.0),
        );
        let (fate, ev) = fly(&mut it, &yard, 2.0);
        assert_eq!(fate, Fate::Removed);
        assert!(!ev.iter().any(|e| matches!(e, FlightEvent::Smash { .. })));
    }

    #[test]
    fn a_teddy_in_the_pool_floats_at_the_surface() {
        let yard = Yard::default();
        let mut it = thrown(ItemKind::Teddy, V3::new(-20.0, 1.0, 10.0), V3::ZERO);
        let (fate, _) = fly(&mut it, &yard, 3.0);
        assert_eq!(fate, Fate::Alive);
        assert_eq!(it.state, ItemState::Ground);
        assert!((it.pos.y - (WATER_Y + 0.04)).abs() < 1e-5);
    }

    #[test]
    fn a_teddy_bounces_on_the_trampoline_a_few_times_and_settles_on_the_mat() {
        let yard = Yard::default();
        let mut it = thrown(ItemKind::Teddy, V3::new(TRAMP_X + 0.4, 1.8, TRAMP_Z - 0.3), V3::new(0.5, 0.0, 0.2));
        let mut rng = Rng::new(5);
        let mut ev = Vec::new();
        let mut tops: Vec<f32> = Vec::new();
        let mut last_vy = 0.0f32;
        for _ in 0..(12.0 / DT) as usize {
            if it.state != ItemState::Flying {
                break;
            }
            step_flight(&mut it, DT, &yard, &mut rng, &mut ev, &mut |_| false);
            if last_vy > 0.0 && it.vel.y <= 0.0 {
                tops.push(it.pos.y);
            }
            last_vy = it.vel.y;
        }
        let boings = ev.iter().filter(|e| matches!(e, FlightEvent::Boing { .. })).count();
        assert!((4..=9).contains(&boings), "a few bounces, not one and not forever ({boings})");
        assert!(tops.windows(2).all(|w| w[1] < w[0] + 0.01), "each bounce is lower: {tops:?}");
        assert_eq!(it.state, ItemState::Ground, "and then it sits still");
        assert!((it.ground_y - TRAMP_H).abs() < 1e-4, "on the mat, not under it");
        assert!((it.pos.x - TRAMP_X).hypot(it.pos.z - TRAMP_Z) < TRAMP_R, "still on the mat");
    }

    #[test]
    fn other_things_keep_the_trampolines_original_kick() {
        let yard = Yard::default();
        let mut it = thrown(ItemKind::Steak, V3::new(TRAMP_X, 1.5, TRAMP_Z), V3::ZERO);
        let mut rng = Rng::new(5);
        let mut ev = Vec::new();
        let mut best = 0.0f32;
        for _ in 0..(1.5 / DT) as usize {
            step_flight(&mut it, DT, &yard, &mut rng, &mut ev, &mut |_| false);
            best = best.max(it.vel.y);
        }
        assert!(best >= 7.0, "the spec's max(7, ...) kick ({best})");
        assert_eq!(it.bounces, 0);
    }

    #[test]
    fn the_trampoline_kicks_an_item_back_up() {
        let yard = Yard::default();
        let mut it = thrown(ItemKind::Teddy, V3::new(TRAMP_X, 1.5, TRAMP_Z), V3::ZERO);
        let (_, ev) = fly(&mut it, &yard, 1.0);
        assert!(ev.iter().any(|e| matches!(e, FlightEvent::Boing { .. })));
    }

    #[test]
    fn a_hard_throw_into_a_wall_bounces_back_and_slows() {
        let yard = Yard::default();
        // brick wall (20,-4), 4.5 wide: x 17.75..22.25, z -4.175..-3.825, h 1.1
        let mut it = thrown(
            ItemKind::Teddy,
            V3::new(20.0, 0.7, 0.0),
            V3::new(0.0, 0.0, -20.0),
        );
        let mut rng = Rng::new(3);
        let mut ev = Vec::new();
        let mut bounced = false;
        for _ in 0..60 {
            step_flight(&mut it, DT, &yard, &mut rng, &mut ev, &mut |_| false);
            if it.vel.z > 0.0 {
                bounced = true;
                break;
            }
        }
        assert!(bounced);
        assert!(it.vel.z < 20.0 * 0.8 * 0.65 + 0.5);
        assert!(ev.iter().any(|e| matches!(e, FlightEvent::Thud { .. })));
    }

    #[test]
    fn landing_on_a_high_box_shoves_the_item_off() {
        let yard = Yard::default();
        // shed top at 2.6 centred (22.5, -16.5)
        let mut it = thrown(ItemKind::Teddy, V3::new(22.9, 3.0, -16.5), V3::ZERO);
        let mut rng = Rng::new(5);
        let mut ev = Vec::new();
        let mut shoved = false;
        for _ in 0..60 {
            step_flight(&mut it, DT, &yard, &mut rng, &mut ev, &mut |_| false);
            if it.vel.x > 3.0 && it.state == ItemState::Flying {
                shoved = true;
                break;
            }
        }
        assert!(shoved, "vel {:?}", it.vel);
    }

    #[test]
    fn over_the_fence_is_an_over_event() {
        let yard = Yard::default();
        let mut it = thrown(
            ItemKind::Teddy,
            V3::new(32.0, 4.0, 0.0),
            V3::new(20.0, 3.0, 0.0),
        );
        it.thrower = Some(9);
        let (fate, ev) = fly(&mut it, &yard, 3.0);
        assert_eq!(fate, Fate::Removed);
        assert!(ev.contains(&FlightEvent::Over { thrower: Some(9) }));
    }

    #[test]
    fn the_fence_bounces_a_low_item_back() {
        let yard = Yard::default();
        let mut it = thrown(
            ItemKind::Teddy,
            V3::new(31.0, 1.0, 0.0),
            V3::new(20.0, 2.0, 0.0),
        );
        let mut rng = Rng::new(2);
        let mut ev = Vec::new();
        for _ in 0..30 {
            step_flight(&mut it, DT, &yard, &mut rng, &mut ev, &mut |_| false);
            if it.vel.x < 0.0 {
                return;
            }
        }
        panic!("never bounced");
    }

    #[test]
    fn rest_heights_match_the_spec() {
        assert_eq!(rest_height(ItemKind::Stubby), 0.07);
        assert_eq!(rest_height(ItemKind::Gnome), 0.28);
        assert_eq!(rest_height(ItemKind::Fish), 0.17);
        assert_eq!(rest_height(ItemKind::Teddy), 0.24);
    }

    #[test]
    fn the_hit_check_can_stop_the_flight() {
        let yard = Yard::default();
        let mut it = thrown(
            ItemKind::Teddy,
            V3::new(0.0, 1.5, 0.0),
            V3::new(5.0, 0.0, 0.0),
        );
        let mut rng = Rng::new(1);
        let mut ev = Vec::new();
        step_flight(&mut it, DT, &yard, &mut rng, &mut ev, &mut |i| {
            i.state = ItemState::Ground;
            true
        });
        assert_eq!(it.state, ItemState::Ground);
    }

    #[test]
    fn puddles_grow_fade_and_go() {
        let mut rng = Rng::new(1);
        let mut ps = Puddles(vec![Puddle::new(0.0, 0.0, &mut rng)]);
        assert!(ps.0[0].r >= 1.1 && ps.0[0].r <= 1.5);
        assert!(!ps.slip_at(0.5, 0.0, 0.0), "not grown yet");
        ps.tick(0.25);
        assert!(ps.slip_at(0.5, 0.0, 0.0));
        assert!(!ps.slip_at(0.5, 0.0, 1.0), "not in the air");
        assert!(!ps.slip_at(5.0, 0.0, 0.0));
        ps.tick(11.0);
        assert!(ps.0[0].opacity() > 0.9);
        ps.tick(2.0);
        assert!(ps.0[0].opacity() < 0.6);
        ps.tick(1.0);
        assert!(ps.0.is_empty());
    }

    #[test]
    fn stepping_into_a_puddle_at_speed_starts_a_slide() {
        let mut rng = Rng::new(4);
        let mut s = Slider::default();
        let start = s
            .tick(DT, true, 6.2, true, false, false, &mut rng)
            .expect("slide");
        assert!((start.speed_mult - 8.37 / 6.2).abs() < 0.01);
        assert!(s.sliding());
        assert!(s.spin.abs() >= 7.0 && s.spin.abs() <= 11.0);
        // still in the same puddle: no new slide
        assert!(
            s.tick(DT, true, 6.2, true, false, false, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn slow_walkers_and_seated_people_do_not_slide() {
        let mut rng = Rng::new(4);
        assert!(
            Slider::default()
                .tick(DT, true, 2.0, true, false, false, &mut rng)
                .is_none()
        );
        assert!(
            Slider::default()
                .tick(DT, true, 6.0, true, true, false, &mut rng)
                .is_none()
        );
        assert!(
            Slider::default()
                .tick(DT, true, 6.0, true, false, true, &mut rng)
                .is_none()
        );
        assert!(
            Slider::default()
                .tick(DT, true, 6.0, false, false, false, &mut rng)
                .is_none()
        );
    }

    #[test]
    fn slide_speed_is_capped_at_11() {
        let mut rng = Rng::new(4);
        let mut s = Slider::default();
        let st = s
            .tick(DT, true, 12.0, true, false, false, &mut rng)
            .unwrap();
        assert!((12.0 * st.speed_mult - 11.0).abs() < 1e-4);
    }

    #[test]
    fn spin_rate_caps_and_gnome_is_slower() {
        assert_eq!(spin_rate(ItemKind::Teddy, 100.0), 18.0);
        assert!((spin_rate(ItemKind::Gnome, 100.0) - 10.8).abs() < 1e-4);
    }
}
