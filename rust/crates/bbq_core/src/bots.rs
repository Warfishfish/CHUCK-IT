//! The bot brain (spec section 11, ported from `aiUpdate`, `botSteer` and friends in the
//! browser game).
//!
//! The brain only decides. Each tick the game shows it what the bot can see (`SelfView`,
//! `WorldView`) and gets back a `Command`: which way to walk, where to face, whether to jump,
//! catch, wind up, throw or swing, and which little errand (a drink, a seat, meat, the chest) to
//! do right now. The game then does those things with the same rules as for a player.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::drinks::drunk_amt;
use crate::drunk_state::bar_spot;
use crate::flight::{ITEM_GRAV, ItemId};
use crate::items::{ItemKind, Melee};
use crate::movement::SEPARATE_DIST;
use crate::rng::Rng;
use crate::vec::V3;
use crate::yard::{BAR, Features, MEAT_TABLE, POOL_SINK, SMOKO_X, SMOKO_Z, Yard};
use crate::{GameMode, PlayerId, YARD_HALF_X, YARD_HALF_Z, smoko};

/// How good the bots are (spec 11, "Difficulty").
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Fair,
    Spicy,
}

/// The numbers behind a difficulty.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Diff {
    /// Aim error added to the throw velocity (grows with drunkenness).
    pub err: f32,
    /// Chance to react to a thrown item.
    pub react: f32,
    /// Chance to catch (as a share of `react`).
    pub catch_p: f32,
    /// Multiplier on the gap between throws.
    pub cd: f32,
    /// How much they lead a moving target.
    pub lead: f32,
    /// Throw speed multiplier.
    pub spd: f32,
    /// How much they favour real players as targets.
    pub bias: f32,
    /// Chance to dodge when they didn't "react".
    pub juke: f32,
}

impl Difficulty {
    pub fn params(self) -> Diff {
        match self {
            Difficulty::Easy => Diff {
                err: 2.2,
                react: 0.18,
                catch_p: 0.05,
                cd: 1.7,
                lead: 0.3,
                spd: 0.85,
                bias: 0.8,
                juke: 0.08,
            },
            Difficulty::Fair => Diff {
                err: 1.3,
                react: 0.42,
                catch_p: 0.14,
                cd: 1.15,
                lead: 0.75,
                spd: 0.93,
                bias: 1.0,
                juke: 0.15,
            },
            Difficulty::Spicy => Diff {
                err: 0.6,
                react: 0.7,
                catch_p: 0.28,
                cd: 0.8,
                lead: 1.0,
                spd: 1.0,
                bias: 1.25,
                juke: 0.22,
            },
        }
    }
}

/// Item gravity at 1x (the item's own factor multiplies it).
pub const GRAV: f32 = ITEM_GRAV;
/// A bot only throws at someone this close and in sight.
pub const THROW_RANGE: f32 = 21.0;
/// Close enough to swing a melee item.
pub const SWING_RANGE: f32 = 2.1;
/// Gap between melee swings.
pub const SWING_GAP: f32 = 0.9;
/// How far away a bot will walk to help a fallen teammate up.
pub const HELP_RANGE: f32 = 22.0;
/// The first errand comes after this long, and the gaps between them.
pub const ERRAND_FIRST: (f32, f32) = (6.0, 16.0);
pub const ERRAND_GAP: (f32, f32) = (14.0, 26.0);
/// Chance of "no errand, keep chucking" when one comes due.
pub const ERRAND_SKIP: f32 = 0.65;
/// An errand gives up after this long.
pub const ERRAND_TIMEOUT: f32 = 14.0;

fn wrap_pi(mut a: f32) -> f32 {
    use std::f32::consts::{PI, TAU};
    while a > PI {
        a -= TAU;
    }
    while a < -PI {
        a += TAU;
    }
    a
}

/// Ease angle `a` towards `b` by fraction `t`.
fn ang_lerp(a: f32, b: f32, t: f32) -> f32 {
    a + wrap_pi(b - a) * t
}

/// Is there a solid thing at `(x, z)` for someone at height `y`? (`blocked` in the browser game.)
pub fn blocked(yard: &Yard, x: f32, z: f32, y: f32) -> bool {
    for c in &yard.colliders {
        if c.h < 0.4 || y >= c.h - 0.3 {
            continue;
        }
        if x > c.x0 - 0.45 && x < c.x1 + 0.45 && z > c.z0 - 0.45 && z < c.z1 + 0.45 {
            return true;
        }
    }
    x.abs() > YARD_HALF_X - 0.6 || z.abs() > YARD_HALF_Z - 0.6
}

/// Clear line between two spots: no box taller than 1.6 m in the way (`los`).
pub fn line_of_sight(yard: &Yard, a: V3, b: V3) -> bool {
    for i in 1..8 {
        let t = i as f32 / 8.0;
        let (x, z) = (a.x + (b.x - a.x) * t, a.z + (b.z - a.z) * t);
        for c in &yard.colliders {
            if c.h > 1.6 && x > c.x0 && x < c.x1 && z > c.z0 && z < c.z1 {
                return false;
            }
        }
    }
    true
}

/// A throw that gets from `from` to `to` at `speed` under gravity `g`: the velocity and the time
/// it takes (the flatter of the two arcs). `None` if it can't reach.
pub fn ballistic(from: V3, to: V3, speed: f32, g: f32) -> Option<(V3, f32)> {
    let (dx, dz, dy) = (to.x - from.x, to.z - from.z, to.y - from.y);
    let h = dx.hypot(dz);
    if h < 0.1 {
        return None;
    }
    let v2 = speed * speed;
    let disc = v2 * v2 - g * (g * h * h + 2.0 * dy * v2);
    if disc < 0.0 {
        return None;
    }
    let ang = ((v2 - disc.sqrt()) / (g * h)).atan();
    let (s, c) = ang.sin_cos();
    Some((
        V3::new(dx / h * c * speed, s * speed, dz / h * c * speed),
        h / (speed * c),
    ))
}

/// Where the bot's own habits sit. Each bot keeps its own for the whole game.
#[derive(Clone, Copy, Debug)]
pub struct Personality {
    /// Hesitation after being hit, seconds.
    pub react: f32,
    /// How much its walking path wobbles.
    pub wob: f32,
    /// Chance rate of stopping to look around.
    pub pause: f32,
    /// How fast it can turn (radians per second).
    pub turn: f32,
    /// How fast its head turns.
    pub look: f32,
}

impl Personality {
    pub fn random(rng: &mut Rng) -> Self {
        Personality {
            react: rng.range(0.14, 0.32),
            wob: rng.range(0.1, 0.26),
            pause: rng.range(0.02, 0.055),
            turn: rng.range(5.5, 8.0),
            look: rng.range(6.0, 10.0),
        }
    }
}

/// What one of the bot's held items is.
#[derive(Clone, Copy, Debug)]
pub struct HeldView {
    pub id: ItemId,
    pub kind: ItemKind,
    /// Heist: which team's teddy this is.
    pub heist_team: Option<u8>,
}

/// What the bot knows about itself.
#[derive(Clone, Debug)]
pub struct SelfView {
    pub id: PlayerId,
    pub pos: V3,
    pub vel: V3,
    pub grounded: bool,
    pub in_pool: bool,
    pub stun: f32,
    pub down_t: f32,
    pub fall_t: f32,
    pub at_smoko: bool,
    /// Seconds spent in the chair so far.
    pub smoko_t: f32,
    pub carried: bool,
    pub drunk: f32,
    pub drinking: bool,
    pub catch_cd: f32,
    /// Where the throwing hand is.
    pub hand: V3,
    pub held: Vec<HeldView>,
    /// Which of `held` is in hand.
    pub selected: Option<usize>,
}

/// Someone else in the yard.
#[derive(Clone, Copy, Debug)]
pub struct OtherView {
    pub id: PlayerId,
    pub pos: V3,
    pub vel: V3,
    pub stun: f32,
    pub down_t: f32,
    pub fall_t: f32,
    pub at_smoko: bool,
    pub is_bot: bool,
    pub in_pool: bool,
    /// Same team (never the Wildcard): not a target, and can be helped up.
    pub teammate: bool,
    /// Carrying (or being carried by) someone, so steer clear.
    pub in_carry_with_me: bool,
    /// Heist: carrying a teddy that belongs to this bot's team.
    pub has_our_teddy: bool,
}

/// An item in the air.
#[derive(Clone, Copy, Debug)]
pub struct FlyingView {
    pub id: ItemId,
    pub pos: V3,
    pub vel: V3,
    pub thrower: Option<PlayerId>,
    /// Can still hit someone.
    pub live: bool,
}

/// An item lying on the ground.
#[derive(Clone, Copy, Debug)]
pub struct GroundView {
    pub id: ItemId,
    pub kind: ItemKind,
    pub pos: V3,
    pub ground_y: f32,
    /// Another bot is already heading for it.
    pub claimed_by_other: bool,
    pub heist_team: Option<u8>,
}

/// What the bot can see of the world.
#[derive(Clone, Copy)]
pub struct WorldView<'a> {
    pub time: f32,
    pub dt: f32,
    pub yard: &'a Yard,
    pub others: &'a [OtherView],
    pub flying: &'a [FlyingView],
    pub ground: &'a [GroundView],
    pub leader: Option<PlayerId>,
    pub mode: GameMode,
    pub counting_down: bool,
    pub features: Features,
    pub adult: bool,
    pub naughty: bool,
    pub chest_stock: u32,
    pub chest_at: (f32, f32),
    /// Which smoko chairs are free.
    pub free_seats: &'a [bool],
    /// The middle of the smoko pad (moved in Teddy Heist).
    pub smoko_at: (f32, f32),
    pub dazza_chasing: bool,
    pub difficulty: Difficulty,
    /// Heist: where this bot should go (raid, guard, chase, bank), if anything.
    pub heist_goal: Option<V3>,
    /// Heist: this bot is carrying a stolen teddy.
    pub carrying_stolen_teddy: bool,
}

/// A throw the bot has decided on.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Throw {
    pub item: ItemId,
    pub from: V3,
    pub vel: V3,
    /// How hard it was thrown, 0..1 (worked out from the speed; bots never power-throw).
    pub charge: f32,
}

/// Something the bot wants done on the spot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Act {
    /// R at the bar.
    Drink,
    /// Sit in this chair.
    Sit(usize),
    /// Get up from smoko.
    StandUp,
    /// R at the meat table.
    TakeMeat(ItemKind),
    /// R at the chest.
    TakeFromChest,
    /// Pull a fallen teammate up (after holding for 1.5 s).
    HelpUp(PlayerId),
}

/// What to do about melee this tick.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Swing {
    /// Hit this person with the item in hand.
    At(PlayerId),
    /// Swing at nothing (the target is down or sitting).
    Air,
}

/// Everything the bot wants this tick.
#[derive(Clone, Debug, Default)]
pub struct Command {
    /// Which way to walk (world x, z; length up to 1).
    pub wish: (f32, f32),
    /// Which way to face, in the browser's sense: it looks along `(sin f, cos f)`.
    pub face: f32,
    pub jump: bool,
    pub catch: bool,
    /// Winding up a throw (for the animation and the slower walk).
    pub winding: bool,
    /// Wind-up progress 0..1, for the animation.
    pub wind_progress: f32,
    pub throw: Option<Throw>,
    pub swing: Option<Swing>,
    pub act: Option<Act>,
    /// The item it is heading for, so other bots leave it alone.
    pub claim: Option<ItemId>,
    /// It has decided to leave the smoko chair or stop being idle: nothing to do this tick.
    pub busy: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ErrandKind {
    Bar,
    Smoko,
    Chest,
    Meat,
}

#[derive(Clone, Copy, Debug)]
struct Errand {
    kind: ErrandKind,
    t: f32,
    ph: u8,
    x: f32,
    z: f32,
    seat: usize,
    sit: f32,
}

/// The pieces of the bot's mind that last from one tick to the next.
#[derive(Clone, Debug)]
pub struct BotBrain {
    pub pers: Personality,
    pub target: Option<PlayerId>,
    retarget: f32,
    pub range: f32,
    orbit: f32,
    orbit_s: f32,
    flip: f32,
    throw_cd: f32,
    windup: f32,
    windup_total: f32,
    errand: Option<Errand>,
    err_cd: f32,
    stuck_t: f32,
    last_pos: V3,
    wander: f32,
    wander_dir: (f32, f32),
    hes_t: f32,
    pause_t: f32,
    glance_t: f32,
    glance_at: V3,
    look_yaw: f32,
    was_stun: bool,
    /// The bot's smoothed walking direction.
    pub mv: (f32, f32),
    side_t: f32,
    side_dir: (f32, f32),
    avoid_side: f32,
    idle_goal: Option<V3>,
    idle_t: f32,
    last_goal: Option<V3>,
    seen: BTreeSet<ItemId>,
    melee_only: bool,
    last_swing: f32,
    help_t: f32,
    /// Heist guard patrol corner.
    patrol_i: Option<u32>,
    /// Which way it faces now (browser sense).
    pub face: f32,
    phase: f32,
}

impl BotBrain {
    pub fn new(rng: &mut Rng) -> Self {
        let orbit = if rng.chance(0.5) { 1.0 } else { -1.0 };
        BotBrain {
            pers: Personality::random(rng),
            target: None,
            retarget: 0.0,
            range: rng.range(8.0, 13.0),
            orbit,
            orbit_s: orbit,
            flip: rng.range(1.0, 3.0),
            throw_cd: rng.range(1.0, 2.0),
            windup: 0.0,
            windup_total: 0.0,
            errand: None,
            err_cd: rng.range(ERRAND_FIRST.0, ERRAND_FIRST.1),
            stuck_t: 1.0,
            last_pos: V3::ZERO,
            wander: 0.0,
            wander_dir: (0.0, 0.0),
            hes_t: 0.0,
            pause_t: 0.0,
            glance_t: 0.0,
            glance_at: V3::ZERO,
            look_yaw: 0.0,
            was_stun: false,
            mv: (0.0, 0.0),
            side_t: 0.0,
            side_dir: (0.0, 0.0),
            avoid_side: if rng.chance(0.5) { -1.0 } else { 1.0 },
            idle_goal: None,
            idle_t: 0.0,
            last_goal: None,
            seen: BTreeSet::new(),
            melee_only: false,
            last_swing: -9.0,
            help_t: 0.0,
            patrol_i: None,
            face: rng.range(-3.0, 3.0),
            phase: rng.range(0.0, 6.0),
        }
    }

    /// Round start (or the bot is put back in the yard): forget errands and aim.
    pub fn reset(&mut self, rng: &mut Rng) {
        self.target = None;
        self.windup = 0.0;
        self.errand = None;
        self.err_cd = rng.range(ERRAND_FIRST.0, ERRAND_FIRST.1);
        self.mv = (0.0, 0.0);
        self.seen.clear();
        self.was_stun = false;
        self.help_t = 0.0;
    }

    pub fn is_on_errand(&self) -> bool {
        self.errand.is_some()
    }

    pub fn winding_up(&self) -> bool {
        self.windup > 0.0
    }

    /// The patrol corner a Heist guard is on (0..3), choosing one at random the first time.
    pub fn patrol_corner(&mut self, rng: &mut Rng) -> u32 {
        *self.patrol_i.get_or_insert_with(|| rng.index(4) as u32)
    }

    pub fn patrol_corner_or_zero(&self) -> u32 {
        self.patrol_i.unwrap_or(0)
    }

    pub fn set_patrol_corner(&mut self, i: u32) {
        self.patrol_i = Some(i % 4);
    }

    pub fn next_patrol_corner(&mut self) {
        if let Some(i) = self.patrol_i.as_mut() {
            *i = (*i + 1) % 4;
        }
    }

    // ------------------------------------------------------------------------------ targets

    fn pick_target(&mut self, me: &SelfView, w: &WorldView, rng: &mut Rng) {
        let df = w.difficulty.params();
        let mut best: Option<(PlayerId, f32)> = None;
        for o in w.others {
            if o.id == me.id || o.teammate || o.at_smoko {
                continue;
            }
            let d = o.pos.len_between(me.pos);
            let mut s = 1.0 / (d + 3.0);
            if w.leader == Some(o.id) {
                s *= 2.8;
            }
            if !o.is_bot {
                s *= df.bias;
            }
            s *= rng.range(0.6, 1.4);
            if o.stun > 0.0 {
                s *= 0.7;
            }
            if self.melee_only && o.down_t > 0.3 {
                s *= 0.05;
            }
            if best.is_none_or(|(_, bs)| s > bs) {
                best = Some((o.id, s));
            }
        }
        self.target = best.map(|b| b.0);
        self.retarget = rng.range(3.0, 6.0);
        self.range = rng.range(7.0, 13.0);
    }

    fn other<'a>(&self, w: &WorldView<'a>, id: PlayerId) -> Option<&'a OtherView> {
        w.others.iter().find(|o| o.id == id)
    }

    // ------------------------------------------------------------------------------ throwing

    /// Work out the throw at `t` with the item in hand (`botThrow`).
    fn make_throw(
        &mut self,
        me: &SelfView,
        item: HeldView,
        t: &OtherView,
        w: &WorldView,
        rng: &mut Rng,
    ) -> Throw {
        let d = item.kind.def();
        let df = w.difficulty.params();
        let from = me.hand;
        let mut aim = t.pos;
        aim.y += 1.0 - if t.in_pool { POOL_SINK * 0.6 } else { 0.0 };
        let speed = d.speed * rng.range(0.85, 1.0) * df.spd;
        let g = GRAV * d.grav;
        let mut sol = ballistic(from, aim, speed, g);
        if let Some((_, tt)) = sol {
            aim.x += t.vel.x * tt * df.lead;
            aim.z += t.vel.z * tt * df.lead;
            sol = ballistic(from, aim, speed, g).or(sol);
        }
        let mut v = match sol {
            Some((v, _)) => v,
            None => {
                // out of reach: lob it at 45 degrees
                let dir = V3::new(aim.x - from.x, 0.0, aim.z - from.z).normalised();
                V3::new(dir.x * speed * 0.707, speed * 0.707, dir.z * speed * 0.707)
            }
        };
        let e = df.err * (1.0 + drunk_amt(me.drunk) * 1.6);
        v.x += rng.range(-1.0, 1.0) * e;
        v.z += rng.range(-1.0, 1.0) * e;
        v.y += rng.range(-0.5, 0.5) * e;
        self.face = (t.pos.x - me.pos.x).atan2(t.pos.z - me.pos.z);
        self.throw_cd = rng.range(0.8, 1.9) * df.cd;
        let charge = ((v.len() / d.speed - 0.42) / 0.58).clamp(0.0, 1.0);
        Throw {
            item: item.id,
            from,
            vel: v,
            charge,
        }
    }

    // ------------------------------------------------------------------------------ errands

    fn pick_errand(&mut self, me: &SelfView, w: &WorldView, rng: &mut Rng) -> Option<Errand> {
        if w.counting_down || me.at_smoko {
            return None;
        }
        let has_melee = me.held.iter().any(|h| {
            let d = h.kind.def();
            d.melee != Melee::None && !d.throwable
        });
        let free_seats = w.free_seats.iter().filter(|f| **f).count();
        let mut opts: Vec<(ErrandKind, f32)> = Vec::new();
        if w.features.bar && me.drunk < 55.0 {
            opts.push((ErrandKind::Bar, 3.0));
        }
        if w.features.smoko && !w.naughty && free_seats > 1 {
            opts.push((ErrandKind::Smoko, 1.6));
        }
        if w.features.chest && w.adult && w.chest_stock > 0 && me.held.len() < 2 && !has_melee {
            opts.push((ErrandKind::Chest, 2.4));
        }
        if w.features.bbq && me.held.len() < 2 && !has_melee && !w.dazza_chasing {
            opts.push((ErrandKind::Meat, 1.8));
        }
        if rng.chance(ERRAND_SKIP) || opts.is_empty() {
            return None;
        }
        let total: f32 = opts.iter().map(|o| o.1).sum();
        let mut r = rng.f32() * total;
        let mut kind = opts[0].0;
        for o in &opts {
            r -= o.1;
            if r <= 0.0 {
                kind = o.0;
                break;
            }
        }
        let mut e = Errand {
            kind,
            t: ERRAND_TIMEOUT,
            ph: 0,
            x: 0.0,
            z: 0.0,
            seat: 0,
            sit: 0.0,
        };
        let bar_x = (BAR.x0 + BAR.x1) / 2.0;
        let bar_z = (BAR.z0 + BAR.z1) / 2.0;
        match kind {
            ErrandKind::Bar => {
                e.x = bar_x + [-1.3, 0.0, 1.3][rng.index(3)];
                e.z = bar_z + 1.25;
            }
            ErrandKind::Smoko => {
                let free: Vec<usize> = (0..w.free_seats.len())
                    .filter(|i| w.free_seats[*i])
                    .collect();
                e.seat = free[rng.index(free.len())];
                let (x, z) = smoko::seat_pos_at(w.smoko_at, e.seat, w.free_seats.len());
                e.x = x;
                e.z = z;
                e.sit = rng.range(5.0, 9.0);
            }
            ErrandKind::Chest => {
                e.x = w.chest_at.0;
                e.z = w.chest_at.1;
            }
            ErrandKind::Meat => {
                let sd = if rng.chance(0.5) { -1.0 } else { 1.0 };
                let (mx, mz, mw, _, _) = MEAT_TABLE;
                e.x = mx + sd * (mw / 2.0 + 0.9);
                e.z = mz;
            }
        }
        Some(e)
    }

    /// One tick of an errand. `Busy` = standing there doing it; `Go(p)` = walk to `p`;
    /// `Done` = finished or given up.
    fn run_errand(
        &mut self,
        me: &SelfView,
        w: &WorldView,
        rng: &mut Rng,
        cmd: &mut Command,
    ) -> Step {
        let Some(mut e) = self.errand else {
            return Step::Done;
        };
        e.t -= w.dt;
        let d = (e.x - me.pos.x).hypot(e.z - me.pos.z);
        let goal = V3::new(e.x, 0.0, e.z);
        let out;
        match e.kind {
            ErrandKind::Bar => {
                if e.ph == 0 {
                    if e.t <= 0.0 {
                        out = Step::Done;
                    } else if d > 0.45 || bar_spot(&BAR, me.pos.x, me.pos.y, me.pos.z).is_none() {
                        out = Step::Go(goal);
                    } else {
                        cmd.act = Some(Act::Drink);
                        e.ph = 1;
                        self.face = std::f32::consts::PI;
                        out = Step::Busy;
                    }
                } else if me.drinking {
                    out = Step::Busy;
                } else if e.ph == 1 && me.drunk < 45.0 && rng.chance(0.4) {
                    e.ph = 0;
                    e.t = 4.0;
                    out = Step::Busy;
                } else {
                    out = Step::Done;
                }
            }
            ErrandKind::Smoko => {
                let n = w.free_seats.len();
                let centre_face = (w.smoko_at.0 - me.pos.x).atan2(w.smoko_at.1 - me.pos.z);
                if e.ph == 0 {
                    if e.t <= 0.0 || e.seat >= n || !w.free_seats[e.seat] {
                        out = Step::Done;
                    } else if d > 0.7 {
                        out = Step::Go(goal);
                    } else {
                        cmd.act = Some(Act::Sit(e.seat));
                        self.windup = 0.0;
                        self.face = centre_face;
                        e.ph = 1;
                        out = Step::Busy;
                    }
                } else {
                    self.face = centre_face;
                    if me.smoko_t < e.sit {
                        out = Step::Busy;
                    } else {
                        cmd.act = Some(Act::StandUp);
                        out = Step::Done;
                    }
                }
            }
            ErrandKind::Chest => {
                if e.t <= 0.0 || w.chest_stock == 0 {
                    out = Step::Done;
                } else if d > 1.7 {
                    out = Step::Go(goal);
                } else {
                    cmd.act = Some(Act::TakeFromChest);
                    out = Step::Done;
                }
            }
            ErrandKind::Meat => {
                if e.t <= 0.0 {
                    out = Step::Done;
                } else if let Some(kind) = meat_spot(me.pos.x, me.pos.y, me.pos.z) {
                    cmd.act = Some(Act::TakeMeat(kind));
                    out = Step::Done;
                } else {
                    out = Step::Go(goal);
                }
            }
        }
        self.errand = if matches!(out, Step::Done) {
            None
        } else {
            Some(e)
        };
        out
    }

    // ------------------------------------------------------------------------------ the tick

    /// Decide what to do this tick.
    pub fn think(&mut self, me: &SelfView, w: &WorldView, rng: &mut Rng) -> Command {
        let dt = w.dt;
        let df = w.difficulty.params();
        let mut cmd = Command {
            face: self.face,
            ..Default::default()
        };

        if w.counting_down {
            if me.at_smoko {
                cmd.act = Some(Act::StandUp);
            }
            self.errand = None;
            self.mv = (0.0, 0.0);
            cmd.busy = true;
            return cmd;
        }
        if me.stun > 0.0 || me.down_t > 0.0 || me.fall_t > 0.0 || me.carried {
            // nothing to do while stunned, flat on the ground or being dragged
            self.windup = 0.0;
            self.was_stun = true;
            self.mv = (self.mv.0 * 0.5, self.mv.1 * 0.5);
            cmd.busy = true;
            return cmd;
        }
        if self.was_stun {
            self.was_stun = false;
            // shake it off for a moment after being hit
            self.hes_t = self.pers.react + rng.range(0.05, 0.2);
        }
        if let Some(r) = self.help(me, w, &mut cmd) {
            self.mv = r;
            cmd.wish = r;
            self.face_towards_help(me, w, &mut cmd);
            return cmd;
        }

        self.retarget -= dt;
        self.throw_cd -= dt;
        self.flip -= dt;
        self.stuck_t -= dt;
        self.wander -= dt;
        self.hes_t -= dt;
        self.pause_t -= dt;
        self.glance_t -= dt;
        self.idle_t -= dt;
        if self.flip <= 0.0 {
            self.orbit = -self.orbit;
            self.flip = rng.range(1.8, 4.2);
        }
        self.orbit_s += (self.orbit - self.orbit_s) * (1.0 - (-2.2 * dt).exp());

        self.dodge(me, w, rng, &mut cmd);

        let mut goal: Option<V3> = None;
        let mut face_to: Option<V3> = None;
        let near = self.nearest_free_item(me, w);
        let n_held = me.held.len();
        let hg = w.heist_goal;
        let carrying_teddy = w.mode == GameMode::Heist && w.carrying_stolen_teddy;

        // little errands
        self.err_cd -= dt;
        if self.errand.is_none() && self.err_cd <= 0.0 && w.mode != GameMode::Heist {
            self.err_cd = rng.range(ERRAND_GAP.0, ERRAND_GAP.1);
            self.errand = self.pick_errand(me, w, rng);
        }
        if self.errand.is_some() {
            match self.run_errand(me, w, rng, &mut cmd) {
                Step::Busy => {
                    self.mv = (0.0, 0.0);
                    cmd.wish = (0.0, 0.0);
                    cmd.face = self.face;
                    cmd.busy = true;
                    return cmd;
                }
                Step::Go(g) => goal = Some(g),
                Step::Done => {}
            }
        }
        let sel = me.selected.and_then(|i| me.held.get(i)).copied();
        self.melee_only = sel.is_some_and(|h| {
            let d = h.kind.def();
            d.melee != Melee::None && !d.throwable
        });
        let mut wish = (0.0f32, 0.0f32);
        if goal.is_some() || carrying_teddy {
            // already has somewhere to be
        } else if self.melee_only {
            let sel = sel.unwrap();
            let down_melee = sel.kind.def().melee == Melee::Down;
            let keep = self
                .target
                .and_then(|id| self.other(w, id))
                .is_some_and(|t| !(down_melee && t.down_t > 0.3));
            if !keep || self.retarget <= 0.0 {
                self.pick_target(me, w, rng);
            }
            if let Some(t) = self.target.and_then(|id| self.other(w, id)) {
                face_to = Some(t.pos);
                let d = (t.pos.x - me.pos.x).hypot(t.pos.z - me.pos.z).max(1e-3);
                if d > 1.6 {
                    goal = Some(t.pos);
                }
                if d < SWING_RANGE && w.time - self.last_swing > SWING_GAP {
                    self.last_swing = w.time;
                    self.face = (t.pos.x - me.pos.x).atan2(t.pos.z - me.pos.z);
                    cmd.swing = Some(if t.at_smoko || (t.down_t > 0.3 && down_melee) {
                        Swing::Air
                    } else {
                        Swing::At(t.id)
                    });
                }
            } else {
                goal = Some(self.idle_goal(me, w, rng));
            }
        } else if n_held == 0 {
            if let Some(g) = near {
                goal = Some(g.pos);
                cmd.claim = Some(g.id);
            } else {
                goal = Some(self.idle_goal(me, w, rng));
            }
        } else {
            let target_ok = self.target.and_then(|id| self.other(w, id)).is_some();
            if !target_ok || self.retarget <= 0.0 {
                self.pick_target(me, w, rng);
            }
            if let Some(t) = self.target.and_then(|id| self.other(w, id)).copied() {
                let (tx, tz) = (t.pos.x - me.pos.x, t.pos.z - me.pos.z);
                let dist = tx.hypot(tz).max(1e-3);
                face_to = Some(t.pos);
                let detour = near.filter(|g| {
                    n_held < 2
                        && (g.pos.x - me.pos.x).hypot(g.pos.z - me.pos.z) < 4.5
                        && self.windup <= 0.0
                });
                if let Some(g) = detour {
                    goal = Some(g.pos);
                    cmd.claim = Some(g.id);
                } else {
                    let (nx, nz) = (tx / dist, tz / dist);
                    let (mut mx, mut mz) = (0.0, 0.0);
                    if dist > self.range + 2.5 {
                        mx = nx;
                        mz = nz;
                    } else if dist < self.range - 3.0 {
                        mx = -nx;
                        mz = -nz;
                    }
                    mx += -nz * self.orbit_s * 0.75;
                    mz += nx * self.orbit_s * 0.75;
                    wish = (mx, mz);
                    let l = wish.0.hypot(wish.1);
                    if l > 1.0 {
                        wish = (wish.0 / l, wish.1 / l);
                    }
                }
                if self.windup > 0.0 {
                    self.windup -= dt;
                    cmd.winding = true;
                    if self.windup <= 0.0
                        && let Some(h) = sel
                    {
                        cmd.throw = Some(self.make_throw(me, h, &t, w, rng));
                        cmd.winding = false;
                    }
                } else if self.throw_cd <= 0.0
                    && dist < THROW_RANGE
                    && line_of_sight(w.yard, me.pos, t.pos)
                    && let Some(h) = sel
                {
                    self.windup_total = h.kind.def().charge * rng.range(0.55, 1.0);
                    self.windup = self.windup_total;
                }
            }
        }
        if let Some(g) = hg {
            goal = Some(g);
            face_to = face_to.or(Some(g));
        }
        let mut arrive = 99.0f32;
        if let Some(g) = goal {
            let (gx, gz) = (g.x - me.pos.x, g.z - me.pos.z);
            let gl = gx.hypot(gz).max(1e-3);
            arrive = gl;
            wish = (gx / gl, gz / gl);
            if face_to.is_none() {
                face_to = Some(g);
            }
            if let Some(lg) = self.last_goal
                && (g.x - lg.x).hypot(g.z - lg.z) > 5.0
            {
                // changed its mind: a beat before heading off
                self.hes_t = self.hes_t.max(self.pers.react * 0.6);
            }
            self.last_goal = Some(V3::new(g.x, 0.0, g.z));
        }
        // now and then stop for a second to have a look around
        if self.pause_t <= 0.0
            && self.windup <= 0.0
            && !carrying_teddy
            && self.errand.is_none()
            && hg.is_none()
            && rng.f32() < self.pers.pause * dt
        {
            self.pause_t = rng.range(0.3, 0.75);
            self.look_yaw = rng.range(-1.1, 1.1);
        }
        if self.windup > 0.0 {
            wish = (wish.0 * 0.5, wish.1 * 0.5);
        }
        let urgent = self.side_t > 0.0;
        if self.side_t > 0.0 {
            self.side_t -= dt;
            wish = self.side_dir;
        }
        if self.wander > 0.0 {
            wish = self.wander_dir;
        }
        let steered = self.steer(me, w, wish, dt, arrive, urgent, goal);
        cmd.wish = steered;
        // stuck check
        if self.stuck_t <= 0.0 {
            if me.pos.len_between(self.last_pos) < 0.4 && wish.0 * wish.0 + wish.1 * wish.1 > 0.25 {
                self.wander = 0.9;
                let a = rng.range(0.0, std::f32::consts::TAU);
                self.wander_dir = (a.cos(), a.sin());
            }
            self.last_pos = me.pos;
            self.stuck_t = 1.2;
        }
        // where to look
        let mut desired: Option<f32> = if let Some(f) = face_to {
            Some((f.x - me.pos.x).atan2(f.z - me.pos.z))
        } else if wish.0 * wish.0 + wish.1 * wish.1 > 0.01 {
            Some(wish.0.atan2(wish.1))
        } else {
            None
        };
        if self.glance_t > 0.0 && self.windup <= 0.0 {
            desired = Some((self.glance_at.x - me.pos.x).atan2(self.glance_at.z - me.pos.z));
        } else if self.pause_t > 0.0
            && self.windup <= 0.0
            && let Some(d) = desired
        {
            desired = Some(
                d + self.look_yaw * ((self.pause_t / 0.75).min(1.0) * std::f32::consts::PI).sin(),
            );
        }
        if let Some(d) = desired {
            self.face = ang_lerp(self.face, d, 1.0 - (-self.pers.look * dt).exp());
        }
        cmd.face = self.face;
        cmd.wind_progress = if self.windup > 0.0 && self.windup_total > 0.0 {
            1.0 - self.windup / self.windup_total
        } else {
            0.0
        };
        let _ = df;
        cmd
    }

    fn face_towards_help(&mut self, me: &SelfView, w: &WorldView, cmd: &mut Command) {
        if let Some(t) = self.help_target(me, w) {
            self.face = (t.pos.x - me.pos.x).atan2(t.pos.z - me.pos.z);
        }
        cmd.face = self.face;
    }

    // ------------------------------------------------------------------------------ helping up

    fn help_target<'a>(&self, me: &SelfView, w: &WorldView<'a>) -> Option<&'a OtherView> {
        if !matches!(w.mode, GameMode::Teams | GameMode::Heist) {
            return None;
        }
        let mut best: Option<(&OtherView, f32)> = None;
        for o in w.others {
            if o.id == me.id || !o.teammate || o.fall_t <= 0.0 {
                continue;
            }
            let d = o.pos.len_between(me.pos);
            if d < HELP_RANGE && best.is_none_or(|(_, bd)| d < bd) {
                best = Some((o, d));
            }
        }
        best.map(|b| b.0)
    }

    /// Walk to a fallen teammate and pull them up (`botHelp`). Returns the walking direction if
    /// it is busy with that.
    fn help(&mut self, me: &SelfView, w: &WorldView, cmd: &mut Command) -> Option<(f32, f32)> {
        let Some(t) = self.help_target(me, w) else {
            self.help_t = 0.0;
            return None;
        };
        self.windup = 0.0;
        let (dx, dz) = (t.pos.x - me.pos.x, t.pos.z - me.pos.z);
        let d = dx.hypot(dz).max(1e-3);
        if d > 1.3 {
            self.help_t = 0.0;
            Some((dx / d, dz / d))
        } else {
            self.help_t += w.dt;
            if self.help_t >= crate::drunk_state::HELP_TIME {
                self.help_t = 0.0;
                cmd.act = Some(Act::HelpUp(t.id));
            }
            Some((0.0, 0.0))
        }
    }

    // ------------------------------------------------------------------------------ dodging

    fn dodge(&mut self, me: &SelfView, w: &WorldView, rng: &mut Rng, cmd: &mut Command) {
        let df = w.difficulty.params();
        // forget items that have landed
        self.seen.retain(|id| w.flying.iter().any(|f| f.id == *id));
        for it in w.flying {
            if !it.live || it.thrower == Some(me.id) || self.seen.contains(&it.id) {
                continue;
            }
            let (rx, rz) = (me.pos.x - it.pos.x, me.pos.z - it.pos.z);
            let dist = rx.hypot(rz);
            if dist > 9.0 {
                continue;
            }
            let vh = it.vel.x.hypot(it.vel.z);
            if vh < 1.0 {
                continue;
            }
            let closing = (it.vel.x * rx + it.vel.z * rz) / dist.max(1e-3);
            if closing < vh * 0.85 || dist / closing > 0.5 {
                continue;
            }
            self.seen.insert(it.id);
            self.glance_t = 0.35;
            self.glance_at = it.pos;
            let side = |rng: &mut Rng| {
                let s = if rng.chance(0.5) { -1.0 } else { 1.0 };
                let l = it.vel.x.hypot(it.vel.z).max(1e-3);
                (-it.vel.z / l * s, it.vel.x / l * s)
            };
            if rng.chance(df.react) {
                if me.catch_cd <= 0.0 && rng.chance(df.catch_p / df.react) {
                    self.face = (-rx).atan2(-rz);
                    cmd.catch = true;
                } else if rng.chance(0.55) {
                    if me.grounded && rng.chance(0.55) {
                        cmd.jump = true;
                    } else {
                        self.side_t = 0.45;
                        self.side_dir = side(rng);
                    }
                }
            } else if rng.chance(df.juke) {
                if me.grounded && rng.chance(0.55) {
                    cmd.jump = true;
                } else {
                    self.side_t = 0.42;
                    self.side_dir = side(rng);
                }
            }
        }
    }

    // ------------------------------------------------------------------------------ items

    fn nearest_free_item<'a>(&self, me: &SelfView, w: &WorldView<'a>) -> Option<&'a GroundView> {
        let mut best: Option<(&GroundView, f32)> = None;
        for it in w.ground {
            if it.kind.def().melee != Melee::None || it.claimed_by_other {
                continue;
            }
            let d = (it.pos.x - me.pos.x).hypot(it.pos.z - me.pos.z)
                + if it.ground_y > 0.1 { 1.5 } else { 0.0 };
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((it, d));
            }
        }
        best.map(|b| b.0)
    }

    /// A spot to amble towards when there is nothing better to do; kept until reached or bored.
    fn idle_goal(&mut self, me: &SelfView, w: &WorldView, rng: &mut Rng) -> V3 {
        let reached = self
            .idle_goal
            .is_some_and(|g| (g.x - me.pos.x).hypot(g.z - me.pos.z) < 1.3);
        if self.idle_goal.is_none() || self.idle_t <= 0.0 || reached {
            self.idle_goal = None;
            for _ in 0..8 {
                let (x, z) = (rng.range(-7.0, 7.0), rng.range(-6.0, 6.0));
                if !blocked(w.yard, x, z, 0.0) {
                    self.idle_goal = Some(V3::new(x, 0.0, z));
                    break;
                }
            }
            if self.idle_goal.is_none() {
                self.idle_goal = Some(V3::new(rng.range(-3.0, 3.0), 0.0, rng.range(-3.0, 3.0)));
            }
            self.idle_t = rng.range(2.5, 4.5);
        }
        self.idle_goal.unwrap()
    }

    // ------------------------------------------------------------------------------ steering

    /// Turn "go that way" into something that looks like a person walking (`botSteer`): limited
    /// turn speed, easing in and out, a personal wobble, drunk sway, stepping round mates,
    /// slowing near the spot, and sticking to one side when going round things.
    #[allow(clippy::too_many_arguments)]
    fn steer(
        &mut self,
        me: &SelfView,
        w: &WorldView,
        wish: (f32, f32),
        dt: f32,
        arrive: f32,
        urgent: bool,
        goal: Option<V3>,
    ) -> (f32, f32) {
        let p = self.pers;
        let (mut wx, mut wz) = wish;
        let mut m = wx.hypot(wz).min(1.0);
        if m > 0.01 && !urgent {
            // give other people a bit of room (but not whoever we're heading for)
            for o in w.others {
                if o.id == me.id || Some(o.id) == self.target || o.in_carry_with_me || o.at_smoko {
                    continue;
                }
                if let Some(g) = goal
                    && (o.pos.x - g.x).hypot(o.pos.z - g.z) < 0.8
                {
                    continue;
                }
                let (dx, dz) = (me.pos.x - o.pos.x, me.pos.z - o.pos.z);
                let d = dx.hypot(dz);
                if d < 1.5 && d > 0.01 {
                    let k = (1.5 - d) / 1.5 * 0.7;
                    wx += dx / d * k;
                    wz += dz / d * k;
                }
            }
            let mut a = wz.atan2(wx);
            let t = w.time;
            let near = (arrive / 2.0).min(1.0);
            let da = drunk_amt(me.drunk);
            a += p.wob
                * ((t * 0.9 + self.phase).sin() * 0.6 + (t * 2.3 + self.phase * 1.7).sin() * 0.25)
                * near;
            if da > 0.0 {
                a += da
                    * ((t * 0.5 + self.phase).sin() * 0.5
                        + (t * 1.37 + 1.0 + self.phase).sin() * 0.22)
                    * near;
            }
            if arrive < 1.4 {
                m *= (arrive / 1.4).max(0.45);
            }
            if self.hes_t > 0.0 {
                m *= 0.35;
            }
            if self.pause_t > 0.0 {
                m *= 0.1;
            }
            wx = a.cos();
            wz = a.sin();
        } else if m > 0.01 {
            wx /= m;
            wz /= m;
        }
        // obstacles: look ahead and keep going round the same side, so they don't twitch along walls
        let mut ta: Option<f32> = None;
        if m > 0.01 {
            let base = wz.atan2(wx);
            let sd = self.avoid_side;
            let offs = [
                0.0,
                0.45 * sd,
                0.9 * sd,
                1.35 * sd,
                1.8 * sd,
                -0.45 * sd,
                -0.9 * sd,
                -1.35 * sd,
                -1.8 * sd,
                2.5 * sd,
                -2.5 * sd,
            ];
            for off in offs {
                let b = base + off;
                if !blocked(
                    w.yard,
                    me.pos.x + b.cos() * 1.1,
                    me.pos.z + b.sin() * 1.1,
                    me.pos.y,
                ) && !blocked(
                    w.yard,
                    me.pos.x + b.cos() * 0.55,
                    me.pos.z + b.sin() * 0.55,
                    me.pos.y,
                ) {
                    ta = Some(b);
                    if off * sd < 0.0 {
                        self.avoid_side = -sd;
                    }
                    break;
                }
            }
            if ta.is_none() {
                ta = Some(base);
            }
        }
        let (tx, tz) = match ta {
            Some(a) => (a.cos() * m, a.sin() * m),
            None => (0.0, 0.0),
        };
        if urgent {
            self.mv = (tx, tz);
            return self.mv;
        }
        let cur = self.mv;
        let cm = cur.0.hypot(cur.1);
        let mut ca = if cm > 0.02 {
            cur.1.atan2(cur.0)
        } else {
            ta.unwrap_or(0.0)
        };
        let mut d = 0.0;
        if let Some(t) = ta {
            d = wrap_pi(t - ca);
            let turn =
                p.turn * if arrive < 2.2 { 2.4 } else { 1.0 } * if cm < 0.3 { 3.0 } else { 1.0 };
            ca += d.clamp(-turn * dt, turn * dt);
        }
        // ease off through sharp turns
        let tm = if ta.is_none() {
            0.0
        } else {
            m * (0.55 + 0.45 * d.abs().min(std::f32::consts::PI).cos()).max(0.4)
        };
        let nm = cm + (tm - cm) * (1.0 - (-(if tm > cm { 4.5 } else { 8.0 }) * dt).exp());
        self.mv = (ca.cos() * nm, ca.sin() * nm);
        self.mv
    }
}

/// What an errand wants next.
enum Step {
    Busy,
    Go(V3),
    Done,
}

/// Which meat the table gives you here, if you are close enough (the left half gives steaks,
/// the right half fish). Same rule the player uses.
pub fn meat_spot(x: f32, y: f32, z: f32) -> Option<ItemKind> {
    if y > 0.5 {
        return None;
    }
    let (mx, mz, w, d, _) = MEAT_TABLE;
    let (dx, dz) = (x - mx, z - mz);
    let ex = (dx.abs() - w / 2.0).max(0.0);
    let ez = (dz.abs() - d / 2.0).max(0.0);
    if ex.hypot(ez) > 1.3 {
        return None;
    }
    Some(if dx < 0.0 {
        ItemKind::Steak
    } else {
        ItemKind::Fish
    })
}

/// Bots in the yard, for the game to keep: one brain per bot.
#[derive(Clone, Debug, Default)]
pub struct Crowd {
    pub brains: BTreeMap<PlayerId, BotBrain>,
    /// Which bot is heading for which item (so two bots don't go for the same one).
    pub claims: BTreeMap<ItemId, PlayerId>,
}

impl Crowd {
    pub fn add(&mut self, id: PlayerId, rng: &mut Rng) {
        self.brains.insert(id, BotBrain::new(rng));
    }

    /// Record the item a bot is heading for, and let go of ones it no longer wants.
    pub fn claim(&mut self, bot: PlayerId, item: Option<ItemId>) {
        self.claims.retain(|_, b| *b != bot);
        if let Some(i) = item {
            self.claims.insert(i, bot);
        }
    }

    /// Is this item claimed by a bot other than `me`?
    pub fn claimed_by_other(&self, item: ItemId, me: PlayerId) -> bool {
        self.claims.get(&item).is_some_and(|b| *b != me)
    }
}

/// How close two people stand before they push each other apart (re-exported for the bots' room).
pub const PERSONAL_SPACE: f32 = SEPARATE_DIST;

trait Between {
    fn len_between(self, o: V3) -> f32;
}

impl Between for V3 {
    fn len_between(self, o: V3) -> f32 {
        (self - o).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movement::Mover;
    use crate::yard::Yard;

    const DT: f32 = 1.0 / 60.0;

    fn me_at(x: f32, z: f32) -> SelfView {
        SelfView {
            id: 10,
            pos: V3::new(x, 0.0, z),
            vel: V3::ZERO,
            grounded: true,
            in_pool: false,
            stun: 0.0,
            down_t: 0.0,
            fall_t: 0.0,
            at_smoko: false,
            smoko_t: 0.0,
            carried: false,
            drunk: 0.0,
            drinking: false,
            catch_cd: 0.0,
            hand: V3::new(x, 1.3, z),
            held: Vec::new(),
            selected: None,
        }
    }

    fn other(id: PlayerId, x: f32, z: f32) -> OtherView {
        OtherView {
            id,
            pos: V3::new(x, 0.0, z),
            vel: V3::ZERO,
            stun: 0.0,
            down_t: 0.0,
            fall_t: 0.0,
            at_smoko: false,
            is_bot: false,
            in_pool: false,
            teammate: false,
            in_carry_with_me: false,
            has_our_teddy: false,
        }
    }

    struct Scene {
        yard: Yard,
        others: Vec<OtherView>,
        flying: Vec<FlyingView>,
        ground: Vec<GroundView>,
        seats: Vec<bool>,
    }

    impl Scene {
        fn new() -> Self {
            Scene {
                yard: Yard::default(),
                others: Vec::new(),
                flying: Vec::new(),
                ground: Vec::new(),
                seats: vec![true; 4],
            }
        }
        fn view(&self, time: f32) -> WorldView<'_> {
            WorldView {
                time,
                dt: DT,
                yard: &self.yard,
                others: &self.others,
                flying: &self.flying,
                ground: &self.ground,
                leader: None,
                mode: GameMode::FreeForAll,
                counting_down: false,
                features: Features::default(),
                adult: false,
                naughty: false,
                chest_stock: 2,
                chest_at: (-29.0, -21.0),
                free_seats: &self.seats,
                smoko_at: (SMOKO_X, SMOKO_Z),
                dazza_chasing: false,
                difficulty: Difficulty::Fair,
                heist_goal: None,
                carrying_stolen_teddy: false,
            }
        }
    }

    fn teddy(id: ItemId) -> HeldView {
        HeldView {
            id,
            kind: ItemKind::Teddy,
            heist_team: None,
        }
    }

    #[test]
    fn difficulty_table_matches_the_spec() {
        let e = Difficulty::Easy.params();
        assert_eq!((e.err, e.react, e.cd, e.spd), (2.2, 0.18, 1.7, 0.85));
        let s = Difficulty::Spicy.params();
        assert_eq!((s.err, s.react, s.catch_p, s.juke), (0.6, 0.7, 0.28, 0.22));
        assert_eq!(Difficulty::Fair.params().bias, 1.0);
        assert_eq!(Difficulty::Spicy.params().bias, 1.25);
    }

    #[test]
    fn ballistic_lands_on_the_target() {
        let from = V3::new(0.0, 1.5, 0.0);
        let to = V3::new(0.0, 1.0, 15.0);
        let (v, t) = ballistic(from, to, 24.0, GRAV).unwrap();
        // fly it by hand
        let (mut p, mut vel) = (from, v);
        let steps = (t / 0.001) as usize;
        for _ in 0..steps {
            vel.y -= GRAV * 0.001;
            p += vel * 0.001;
        }
        assert!((p.z - 15.0).abs() < 0.05, "z {}", p.z);
        assert!((p.y - 1.0).abs() < 0.05, "y {}", p.y);
        // too far to reach at this speed
        assert!(ballistic(from, V3::new(0.0, 1.0, 100.0), 10.0, GRAV).is_none());
        // right on top of you
        assert!(ballistic(from, from, 10.0, GRAV).is_none());
    }

    #[test]
    fn ballistic_takes_the_flatter_arc() {
        let (v, _) = ballistic(V3::ZERO, V3::new(0.0, 0.0, 10.0), 30.0, GRAV).unwrap();
        assert!(v.y < v.z, "should be a low shot, not a lob: {v:?}");
    }

    #[test]
    fn walls_block_and_the_edge_blocks() {
        let yard = Yard::default();
        assert!(!blocked(&yard, 0.0, 10.0, 0.0));
        assert!(blocked(&yard, 33.0, 0.0, 0.0), "fence");
        // the shed at (22.5, -16.5), 2.6 high
        assert!(blocked(&yard, 22.5, -16.5, 0.0));
        // on top of it you are not blocked
        assert!(!blocked(&yard, 22.5, -16.5, 2.5));
    }

    #[test]
    fn tall_things_block_the_view_but_low_ones_do_not() {
        let yard = Yard::default();
        // the shed is 2.6 high
        assert!(!line_of_sight(
            &yard,
            V3::new(15.0, 1.0, -16.5),
            V3::new(30.0, 1.0, -16.5)
        ));
        // the eskies are low
        assert!(line_of_sight(
            &yard,
            V3::new(-10.0, 1.0, -4.5),
            V3::new(-5.0, 1.0, -4.5)
        ));
    }

    #[test]
    fn nobody_does_anything_during_the_countdown() {
        let mut rng = Rng::new(1);
        let mut brain = BotBrain::new(&mut rng);
        let mut s = Scene::new();
        s.others.push(other(1, 5.0, 5.0));
        let mut w = s.view(0.0);
        w.counting_down = true;
        let c = brain.think(&me_at(0.0, 0.0), &w, &mut rng);
        assert_eq!(c.wish, (0.0, 0.0));
        assert!(c.throw.is_none() && c.busy);
    }

    #[test]
    fn a_stunned_bot_does_nothing_and_then_hesitates() {
        let mut rng = Rng::new(2);
        let mut brain = BotBrain::new(&mut rng);
        let mut s = Scene::new();
        s.others.push(other(1, 10.0, 0.0));
        let mut me = me_at(0.0, 0.0);
        me.held = vec![teddy(5)];
        me.selected = Some(0);
        me.stun = 1.0;
        let c = brain.think(&me, &s.view(0.0), &mut rng);
        assert!(c.busy && c.wish == (0.0, 0.0));
        me.stun = 0.0;
        brain.think(&me, &s.view(1.0), &mut rng);
        // it is still hesitating, so it only creeps for a moment (35% speed at most)
        assert!(brain.hes_t > 0.0);
    }

    #[test]
    fn with_nothing_in_hand_it_goes_for_the_nearest_item() {
        let mut rng = Rng::new(3);
        let mut brain = BotBrain::new(&mut rng);
        let mut s = Scene::new();
        s.others.push(other(1, -10.0, -10.0));
        s.ground.push(GroundView {
            id: 7,
            kind: ItemKind::Teddy,
            pos: V3::new(6.0, 0.2, 0.0),
            ground_y: 0.0,
            claimed_by_other: false,
            heist_team: None,
        });
        s.ground.push(GroundView {
            id: 8,
            kind: ItemKind::Gnome,
            pos: V3::new(-9.0, 0.2, 9.0),
            ground_y: 0.0,
            claimed_by_other: false,
            heist_team: None,
        });
        let me = me_at(0.0, 0.0);
        // no errands in this test
        brain.err_cd = 999.0;
        let mut c = Command::default();
        for i in 0..90 {
            c = brain.think(&me, &s.view(i as f32 * DT), &mut rng);
        }
        assert_eq!(c.claim, Some(7));
        assert!(c.wish.0 > 0.1, "should head +x: {:?}", c.wish);
    }

    #[test]
    fn it_leaves_items_another_bot_has_claimed_and_ones_on_a_box_cost_more() {
        let mut rng = Rng::new(4);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let mut s = Scene::new();
        s.ground.push(GroundView {
            id: 7,
            kind: ItemKind::Teddy,
            pos: V3::new(3.0, 0.2, 0.0),
            ground_y: 0.0,
            claimed_by_other: true,
            heist_team: None,
        });
        s.ground.push(GroundView {
            id: 8,
            kind: ItemKind::Teddy,
            pos: V3::new(-6.0, 0.2, 0.0),
            ground_y: 0.0,
            claimed_by_other: false,
            heist_team: None,
        });
        let c = brain.think(&me_at(0.0, 0.0), &s.view(0.0), &mut rng);
        assert_eq!(c.claim, Some(8));
        // melee items are never picked up off the ground by bots
        let mut s2 = Scene::new();
        s2.ground.push(GroundView {
            id: 9,
            kind: ItemKind::Steak,
            pos: V3::new(1.0, 0.2, 0.0),
            ground_y: 0.0,
            claimed_by_other: false,
            heist_team: None,
        });
        let mut b2 = BotBrain::new(&mut rng);
        b2.err_cd = 999.0;
        assert_eq!(
            b2.think(&me_at(0.0, 0.0), &s2.view(0.0), &mut rng).claim,
            None
        );
    }

    #[test]
    fn targets_skip_teammates_and_people_at_smoko_and_like_the_leader() {
        let mut rng = Rng::new(5);
        let mut s = Scene::new();
        let mut mate = other(1, 3.0, 0.0);
        mate.teammate = true;
        let mut sitting = other(2, 4.0, 0.0);
        sitting.at_smoko = true;
        s.others = vec![mate, sitting, other(3, 12.0, 0.0)];
        let me = me_at(0.0, 0.0);
        for seed in 0..20 {
            let mut r = Rng::new(seed);
            let mut b = BotBrain::new(&mut r);
            b.pick_target(&me, &s.view(0.0), &mut r);
            assert_eq!(b.target, Some(3));
        }
        // a leader at 20 m beats a plain player at 10 m most of the time
        s.others = vec![other(3, 10.0, 0.0), other(4, 14.0, 0.0)];
        let mut w = s.view(0.0);
        w.leader = Some(4);
        let mut leader_picks = 0;
        for seed in 0..400 {
            let mut r = Rng::new(seed);
            let mut b = BotBrain::new(&mut r);
            b.pick_target(&me, &w, &mut r);
            if b.target == Some(4) {
                leader_picks += 1;
            }
        }
        assert!(leader_picks > 200, "{leader_picks}");
        let _ = &mut rng;
    }

    #[test]
    fn a_bot_with_a_teddy_walks_into_range_winds_up_and_throws_at_its_target() {
        let mut rng = Rng::new(6);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        brain.pers.pause = 0.0;
        let mut s = Scene::new();
        s.others.push(other(1, 0.0, 10.0));
        let mut me = me_at(0.0, 0.0);
        me.held = vec![teddy(5)];
        me.selected = Some(0);
        let mut thrown = None;
        let mut wound = false;
        for i in 0..600 {
            let c = brain.think(&me, &s.view(i as f32 * DT), &mut rng);
            wound |= c.winding;
            if let Some(t) = c.throw {
                thrown = Some(t);
                break;
            }
        }
        let t = thrown.expect("it should throw within ten seconds");
        assert!(wound, "a wind-up comes first");
        assert_eq!(t.item, 5);
        // aimed roughly at the target: mostly +z
        assert!(t.vel.z > t.vel.x.abs() * 2.0, "{:?}", t.vel);
        assert!(
            t.charge >= 0.0 && t.charge < 1.0,
            "bots never power-throw: {}",
            t.charge
        );
        // and then it cools down before the next
        assert!(brain.throw_cd >= 0.8 * 0.8 * 1.15 / 1.15 - 0.01);
    }

    #[test]
    fn it_does_not_throw_through_the_shed() {
        let mut rng = Rng::new(7);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let mut s = Scene::new();
        // someone on the far side of the shed (22.5, -16.5)
        s.others.push(other(1, 28.0, -16.5));
        let mut me = me_at(17.0, -16.5);
        me.held = vec![teddy(5)];
        me.selected = Some(0);
        brain.range = 20.0; // it is happy to stay where it is
        brain.retarget = 99.0;
        brain.target = Some(1);
        brain.throw_cd = 0.0;
        for i in 0..120 {
            let c = brain.think(&me, &s.view(i as f32 * DT), &mut rng);
            assert!(c.throw.is_none() && !c.winding);
        }
    }

    #[test]
    fn harder_bots_aim_better() {
        let mut miss = [0.0f32; 3];
        for (k, diff) in [Difficulty::Easy, Difficulty::Fair, Difficulty::Spicy]
            .into_iter()
            .enumerate()
        {
            let mut total = 0.0;
            for seed in 0..300 {
                let mut rng = Rng::new(seed);
                let mut brain = BotBrain::new(&mut rng);
                let mut s = Scene::new();
                s.others.push(other(1, 0.0, 12.0));
                let mut w = s.view(0.0);
                w.difficulty = diff;
                let me = me_at(0.0, 0.0);
                let t = brain.make_throw(&me, teddy(5), &s.others[0], &w, &mut rng);
                // where does it come down at height 1 m? (flat enough to just use time of flight)
                let mut p = t.from;
                let mut v = t.vel;
                for _ in 0..4000 {
                    v.y -= GRAV * 0.5 * 0.001;
                    p += v * 0.001;
                    v.y -= GRAV * 0.5 * 0.001;
                    if v.y < 0.0 && p.y < 1.0 {
                        break;
                    }
                }
                total += ((p.x).powi(2) + (p.z - 12.0).powi(2)).sqrt();
            }
            miss[k] = total / 300.0;
        }
        assert!(miss[0] > miss[1] && miss[1] > miss[2], "{miss:?}");
    }

    #[test]
    fn drunk_bots_aim_worse() {
        let measure = |drunk: f32| {
            let mut total = 0.0;
            for seed in 0..300 {
                let mut rng = Rng::new(seed);
                let mut brain = BotBrain::new(&mut rng);
                let mut s = Scene::new();
                s.others.push(other(1, 0.0, 12.0));
                let mut me = me_at(0.0, 0.0);
                me.drunk = drunk;
                let t = brain.make_throw(&me, teddy(5), &s.others[0], &s.view(0.0), &mut rng);
                total += t.vel.x.abs();
            }
            total / 300.0
        };
        assert!(measure(90.0) > measure(0.0) * 1.8);
    }

    #[test]
    fn throws_at_someone_in_the_pool_aim_lower() {
        let mut rng = Rng::new(8);
        let mut brain = BotBrain::new(&mut rng);
        let mut s = Scene::new();
        let mut t = other(1, 0.0, 10.0);
        let dry = brain.make_throw(
            &me_at(0.0, 0.0),
            teddy(5),
            &t,
            &s.view(0.0),
            &mut Rng::new(1),
        );
        t.in_pool = true;
        let wet = brain.make_throw(
            &me_at(0.0, 0.0),
            teddy(5),
            &t,
            &s.view(0.0),
            &mut Rng::new(1),
        );
        assert!(wet.vel.y < dry.vel.y);
        s.others.clear();
    }

    #[test]
    fn it_leads_a_moving_target() {
        let mut rng = Rng::new(9);
        let mut brain = BotBrain::new(&mut rng);
        let s = Scene::new();
        let mut t = other(1, 0.0, 12.0);
        t.vel = V3::new(6.0, 0.0, 0.0);
        let mut w = s.view(0.0);
        w.difficulty = Difficulty::Spicy;
        // average over the random error
        let mut vx = 0.0;
        for seed in 0..200 {
            let mut r = Rng::new(seed);
            vx += brain
                .make_throw(&me_at(0.0, 0.0), teddy(5), &t, &w, &mut r)
                .vel
                .x;
        }
        assert!(
            vx / 200.0 > 1.0,
            "should aim ahead of a runner: {}",
            vx / 200.0
        );
    }

    #[test]
    fn an_incoming_item_makes_it_catch_dodge_or_hop() {
        let mut caught = 0;
        let mut dodged = 0;
        let mut nothing = 0;
        for seed in 0..400 {
            let mut rng = Rng::new(seed);
            let mut brain = BotBrain::new(&mut rng);
            brain.err_cd = 999.0;
            let mut s = Scene::new();
            // heading straight at us from 4 m away at 20 m/s: 0.2 s to impact
            s.flying.push(FlyingView {
                id: 1,
                pos: V3::new(0.0, 1.2, 4.0),
                vel: V3::new(0.0, 0.0, -20.0),
                thrower: Some(2),
                live: true,
            });
            let mut w = s.view(0.0);
            w.difficulty = Difficulty::Spicy;
            let c = brain.think(&me_at(0.0, 0.0), &w, &mut rng);
            if c.catch {
                caught += 1;
            } else if c.jump || brain.side_t > 0.0 {
                dodged += 1;
            } else {
                nothing += 1;
            }
            // it only reacts once per item
            let c2 = brain.think(&me_at(0.0, 0.0), &w, &mut rng);
            assert!(!c2.catch || c.catch, "no second reaction");
        }
        assert!(
            caught > 20 && dodged > 100 && nothing > 20,
            "{caught} {dodged} {nothing}"
        );
    }

    #[test]
    fn it_ignores_its_own_throws_and_things_flying_away() {
        let mut rng = Rng::new(10);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let mut s = Scene::new();
        s.flying.push(FlyingView {
            id: 1,
            pos: V3::new(0.0, 1.2, 4.0),
            vel: V3::new(0.0, 0.0, -20.0),
            thrower: Some(10), // me
            live: true,
        });
        s.flying.push(FlyingView {
            id: 2,
            pos: V3::new(0.0, 1.2, 4.0),
            vel: V3::new(0.0, 0.0, 20.0), // going away
            thrower: Some(3),
            live: true,
        });
        s.flying.push(FlyingView {
            id: 3,
            pos: V3::new(0.0, 1.2, 4.0),
            vel: V3::new(0.0, 0.0, -20.0),
            thrower: Some(3),
            live: false, // already landed
        });
        let mut w = s.view(0.0);
        w.difficulty = Difficulty::Spicy;
        let c = brain.think(&me_at(0.0, 0.0), &w, &mut rng);
        assert!(!c.catch && !c.jump && brain.side_t <= 0.0 && brain.glance_t <= 0.0);
    }

    #[test]
    fn a_melee_bot_walks_up_and_swings_every_point_nine_seconds() {
        let mut rng = Rng::new(11);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let mut s = Scene::new();
        s.others.push(other(1, 0.0, 1.5));
        let mut me = me_at(0.0, 0.0);
        me.held = vec![HeldView {
            id: 5,
            kind: ItemKind::Steak,
            heist_team: None,
        }];
        me.selected = Some(0);
        let mut swings = Vec::new();
        for i in 0..200 {
            let c = brain.think(&me, &s.view(i as f32 * DT), &mut rng);
            if let Some(sw) = c.swing {
                swings.push((i as f32 * DT, sw));
            }
        }
        assert_eq!(swings.len(), 4, "{swings:?}");
        assert!(swings.iter().all(|(_, s)| *s == Swing::At(1)));
        assert!((swings[1].0 - swings[0].0 - 0.9).abs() < 0.03);
    }

    #[test]
    fn a_dildo_bot_swings_at_air_when_the_target_is_already_down() {
        let mut rng = Rng::new(12);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let mut s = Scene::new();
        let mut t = other(1, 0.0, 1.5);
        t.down_t = 2.0;
        s.others.push(t);
        let mut me = me_at(0.0, 0.0);
        me.held = vec![HeldView {
            id: 5,
            kind: ItemKind::Dildo,
            heist_team: None,
        }];
        me.selected = Some(0);
        brain.target = Some(1);
        brain.retarget = 99.0;
        // it would rather pick someone else; with nobody else it just idles
        let c = brain.think(&me, &s.view(0.0), &mut rng);
        assert!(!matches!(c.swing, Some(Swing::At(_))));
    }

    #[test]
    fn a_free_bot_helps_a_fallen_teammate_in_team_modes_only() {
        let mut rng = Rng::new(13);
        let mut s = Scene::new();
        let mut mate = other(1, 3.0, 0.0);
        mate.teammate = true;
        mate.fall_t = 8.0;
        s.others.push(mate);
        let mut ffa = BotBrain::new(&mut rng);
        ffa.err_cd = 999.0;
        // in free for all it does not care
        let c = ffa.think(&me_at(0.0, 0.0), &s.view(0.0), &mut rng);
        assert!(c.act.is_none());
        // in teams it walks over...
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let mut w = s.view(0.0);
        w.mode = GameMode::Teams;
        let c = brain.think(&me_at(0.0, 0.0), &w, &mut rng);
        assert!(c.wish.0 > 0.9, "{:?}", c.wish);
        // ...and once there, holds for 1.5 s and then pulls them up
        let mut helped = None;
        let me = me_at(2.0, 0.0);
        for i in 0..120 {
            let c = brain.think(&me, &w, &mut rng);
            if let Some(Act::HelpUp(id)) = c.act {
                helped = Some((i, id));
                break;
            }
            assert_eq!(c.wish, (0.0, 0.0));
        }
        let (i, id) = helped.unwrap();
        assert_eq!(id, 1);
        assert!((i as f32 * DT - 1.5).abs() < 0.05, "took {}", i as f32 * DT);
    }

    #[test]
    fn it_will_not_help_someone_far_away_or_someone_not_down() {
        let mut rng = Rng::new(14);
        let mut s = Scene::new();
        let mut far = other(1, 30.0, 0.0);
        far.teammate = true;
        far.fall_t = 5.0;
        let mut up = other(2, 3.0, 0.0);
        up.teammate = true;
        s.others = vec![far, up];
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let mut w = s.view(0.0);
        w.mode = GameMode::Teams;
        assert!(brain.help_target(&me_at(0.0, 0.0), &w).is_none());
    }

    #[test]
    fn a_bar_errand_walks_to_the_bar_and_has_a_drink() {
        let mut rng = Rng::new(15);
        let mut brain = BotBrain::new(&mut rng);
        brain.errand = Some(Errand {
            kind: ErrandKind::Bar,
            t: ERRAND_TIMEOUT,
            ph: 0,
            x: 0.0,
            z: -20.25,
            seat: 0,
            sit: 0.0,
        });
        let s = Scene::new();
        let mut me = me_at(0.0, -10.0);
        // far away: it walks (it eases into a walk rather than starting at full speed)
        let mut c = brain.think(&me, &s.view(0.0), &mut rng);
        for i in 1..40 {
            c = brain.think(&me, &s.view(i as f32 * DT), &mut rng);
        }
        assert!(c.wish.1 < -0.3, "{:?}", c.wish);
        assert!(c.act.is_none());
        // at the bar: it drinks and faces the bar
        me.pos = V3::new(0.0, 0.0, -20.25);
        let c = brain.think(&me, &s.view(1.0), &mut rng);
        assert_eq!(c.act, Some(Act::Drink));
        assert!(c.busy && c.wish == (0.0, 0.0));
        // while the drink is going it stands there
        me.drinking = true;
        let c = brain.think(&me, &s.view(1.5), &mut rng);
        assert!(c.busy && c.act.is_none());
        // afterwards, at a low meter it sometimes goes back for another, otherwise it is done
        me.drinking = false;
        me.drunk = 100.0;
        brain.think(&me, &s.view(2.5), &mut rng);
        assert!(!brain.is_on_errand());
    }

    #[test]
    fn a_smoko_errand_sits_stays_5_to_9_seconds_and_gets_up() {
        let mut rng = Rng::new(16);
        let mut brain = BotBrain::new(&mut rng);
        let (sx, sz) = smoko::seat_pos(2, 4);
        brain.errand = Some(Errand {
            kind: ErrandKind::Smoko,
            t: ERRAND_TIMEOUT,
            ph: 0,
            x: sx,
            z: sz,
            seat: 2,
            sit: 6.0,
        });
        let s = Scene::new();
        let mut me = me_at(sx - 0.2, sz);
        let c = brain.think(&me, &s.view(0.0), &mut rng);
        assert_eq!(c.act, Some(Act::Sit(2)));
        me.at_smoko = true;
        me.smoko_t = 3.0;
        let c = brain.think(&me, &s.view(3.0), &mut rng);
        assert!(c.busy && c.act.is_none());
        me.smoko_t = 6.1;
        let c = brain.think(&me, &s.view(6.1), &mut rng);
        assert_eq!(c.act, Some(Act::StandUp));
        assert!(!brain.is_on_errand());
    }

    #[test]
    fn a_smoko_errand_is_dropped_if_somebody_else_takes_the_chair() {
        let mut rng = Rng::new(17);
        let mut brain = BotBrain::new(&mut rng);
        let (sx, sz) = smoko::seat_pos(1, 4);
        brain.errand = Some(Errand {
            kind: ErrandKind::Smoko,
            t: ERRAND_TIMEOUT,
            ph: 0,
            x: sx,
            z: sz,
            seat: 1,
            sit: 6.0,
        });
        let mut s = Scene::new();
        s.seats[1] = false;
        let c = brain.think(&me_at(0.0, 0.0), &s.view(0.0), &mut rng);
        assert!(!brain.is_on_errand());
        assert!(c.act.is_none());
    }

    #[test]
    fn meat_and_chest_errands() {
        let mut rng = Rng::new(18);
        let s = Scene::new();
        let mut brain = BotBrain::new(&mut rng);
        brain.errand = Some(Errand {
            kind: ErrandKind::Meat,
            t: ERRAND_TIMEOUT,
            ph: 0,
            x: -9.9,
            z: -18.2,
            seat: 0,
            sit: 0.0,
        });
        // at the left of the table: steak
        let c = brain.think(&me_at(-9.9, -18.2), &s.view(0.0), &mut rng);
        assert_eq!(c.act, Some(Act::TakeMeat(ItemKind::Steak)));
        assert!(!brain.is_on_errand());
        let mut b2 = BotBrain::new(&mut rng);
        b2.errand = Some(Errand {
            kind: ErrandKind::Chest,
            t: ERRAND_TIMEOUT,
            ph: 0,
            x: -29.0,
            z: -21.0,
            seat: 0,
            sit: 0.0,
        });
        let c = b2.think(&me_at(-28.0, -20.0), &s.view(0.0), &mut rng);
        assert_eq!(c.act, Some(Act::TakeFromChest));
        // an empty chest ends the errand
        let mut b3 = BotBrain::new(&mut rng);
        b3.errand = b2.errand;
        b3.errand = Some(Errand {
            kind: ErrandKind::Chest,
            t: ERRAND_TIMEOUT,
            ph: 0,
            x: -29.0,
            z: -21.0,
            seat: 0,
            sit: 0.0,
        });
        let mut w = s.view(0.0);
        w.chest_stock = 0;
        let c = b3.think(&me_at(0.0, 0.0), &w, &mut rng);
        assert!(!b3.is_on_errand() && c.act.is_none());
    }

    #[test]
    fn errands_time_out() {
        let mut rng = Rng::new(19);
        let s = Scene::new();
        let mut brain = BotBrain::new(&mut rng);
        brain.errand = Some(Errand {
            kind: ErrandKind::Bar,
            t: 0.01,
            ph: 0,
            x: 0.0,
            z: -20.0,
            seat: 0,
            sit: 0.0,
        });
        brain.think(&me_at(10.0, 10.0), &s.view(0.0), &mut rng);
        brain.think(&me_at(10.0, 10.0), &s.view(0.1), &mut rng);
        assert!(!brain.is_on_errand());
    }

    #[test]
    fn errands_are_picked_from_what_is_on_offer() {
        let s = Scene::new();
        let mut kinds = BTreeMap::new();
        let mut none = 0;
        for seed in 0..2000 {
            let mut rng = Rng::new(seed);
            let mut brain = BotBrain::new(&mut rng);
            match brain.pick_errand(&me_at(0.0, 0.0), &s.view(0.0), &mut rng) {
                Some(e) => *kinds.entry(format!("{:?}", e.kind)).or_insert(0) += 1,
                None => none += 1,
            }
        }
        // 65% nothing; the rest by weight bar 3 : smoko 1.6 : meat 1.8 (no chest without Cheeky mode)
        assert!((none as f32 / 2000.0 - 0.65).abs() < 0.04, "{none}");
        assert!(!kinds.contains_key("Chest"));
        assert!(kinds["Bar"] > kinds["Meat"] && kinds["Meat"] > kinds["Smoko"] * 9 / 10);
        // a drunk bot skips the bar
        let mut drunk = me_at(0.0, 0.0);
        drunk.drunk = 60.0;
        for seed in 0..500 {
            let mut rng = Rng::new(seed);
            let mut brain = BotBrain::new(&mut rng);
            if let Some(e) = brain.pick_errand(&drunk, &s.view(0.0), &mut rng) {
                assert_ne!(e.kind, ErrandKind::Bar);
            }
        }
        // someone holding a slapper does not want another one; Dazza chasing puts it off the meat
        let mut armed = me_at(0.0, 0.0);
        armed.held = vec![HeldView {
            id: 1,
            kind: ItemKind::Steak,
            heist_team: None,
        }];
        let mut w = s.view(0.0);
        w.dazza_chasing = true;
        for seed in 0..500 {
            let mut rng = Rng::new(seed);
            let mut brain = BotBrain::new(&mut rng);
            if let Some(e) = brain.pick_errand(&armed, &w, &mut rng) {
                assert!(!matches!(e.kind, ErrandKind::Meat | ErrandKind::Chest));
            }
        }
    }

    #[test]
    fn naughty_corner_mode_and_a_full_pad_rule_out_smoko() {
        let mut s = Scene::new();
        s.seats = vec![true, false, false, false]; // only one free chair
        let mut seen_smoko = false;
        for seed in 0..1000 {
            let mut rng = Rng::new(seed);
            let mut brain = BotBrain::new(&mut rng);
            if let Some(e) = brain.pick_errand(&me_at(0.0, 0.0), &s.view(0.0), &mut rng) {
                seen_smoko |= e.kind == ErrandKind::Smoko;
            }
        }
        assert!(!seen_smoko, "needs more than one free chair");
        let s = Scene::new();
        let mut w = s.view(0.0);
        w.naughty = true;
        for seed in 0..1000 {
            let mut rng = Rng::new(seed);
            let mut brain = BotBrain::new(&mut rng);
            if let Some(e) = brain.pick_errand(&me_at(0.0, 0.0), &w, &mut rng) {
                assert_ne!(e.kind, ErrandKind::Smoko);
            }
        }
    }

    #[test]
    fn heist_bots_do_not_run_errands_and_follow_their_heist_goal() {
        let mut rng = Rng::new(20);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 0.0;
        let s = Scene::new();
        let mut w = s.view(0.0);
        w.mode = GameMode::Heist;
        w.heist_goal = Some(V3::new(10.0, 0.0, 0.0));
        let mut c = Command::default();
        for i in 0..60 {
            let mut ww = w;
            ww.time = i as f32 * DT;
            c = brain.think(&me_at(0.0, 0.0), &ww, &mut rng);
        }
        assert!(!brain.is_on_errand());
        assert!(c.wish.0 > 0.3, "{:?}", c.wish);
    }

    // ---- walking about in a real yard ----

    /// Walk a bot for a while with the real movement rules, and report the longest time it
    /// spent stuck (barely moving while trying to walk) and how often its walking direction flipped.
    fn walk(seed: u64, secs: f32, start: (f32, f32)) -> (f32, usize, f32) {
        let mut rng = Rng::new(seed);
        let mut brain = BotBrain::new(&mut rng);
        brain.err_cd = 999.0;
        let yard = Yard::default();
        let mut mover = Mover::new(start.0, start.1);
        let mut s = Scene::new();
        s.yard = yard.clone();
        let mut longest_stuck = 0.0f32;
        let mut stuck = 0.0f32;
        let mut flips = 0usize;
        let mut last_dir: Option<f32> = None;
        let mut travelled = 0.0f32;
        for i in 0..(secs / DT) as usize {
            let me = SelfView {
                id: 10,
                pos: V3::new(mover.x, mover.y, mover.z),
                vel: V3::new(mover.vx, mover.vy, mover.vz),
                grounded: mover.grounded,
                in_pool: mover.in_pool,
                hand: V3::new(mover.x, mover.y + 1.3, mover.z),
                ..me_at(0.0, 0.0)
            };
            let c = brain.think(&me, &s.view(i as f32 * DT), &mut rng);
            let (ox, oz) = (mover.x, mover.z);
            mover.step(
                DT,
                crate::movement::MoveInput { wish: c.wish, ..Default::default() },
                &crate::movement::Modifiers::default(),
                &yard,
            );
            let moved = (mover.x - ox).hypot(mover.z - oz);
            travelled += moved;
            if c.wish.0.hypot(c.wish.1) > 0.5 && moved < 0.02 {
                stuck += DT;
                longest_stuck = longest_stuck.max(stuck);
            } else {
                stuck = 0.0;
            }
            if moved > 0.01 {
                let dir = (mover.z - oz).atan2(mover.x - ox);
                if let Some(l) = last_dir
                    && wrap_pi(dir - l).abs() > 2.5
                {
                    flips += 1;
                }
                last_dir = Some(dir);
            }
        }
        (longest_stuck, flips, travelled)
    }

    #[test]
    fn an_idle_bot_wanders_without_getting_stuck_or_twitching() {
        for seed in 0..6 {
            let (stuck, flips, travelled) = walk(seed, 90.0, (0.0, 0.0));
            assert!(stuck < 2.0, "seed {seed} was stuck for {stuck}s");
            assert!(flips < 8, "seed {seed} turned right round {flips} times");
            assert!(travelled > 30.0, "seed {seed} only walked {travelled} m");
        }
    }

    #[test]
    fn a_bot_beside_the_shed_still_gets_away() {
        for seed in 10..14 {
            let (stuck, _, travelled) = walk(seed, 60.0, (20.0, -14.0));
            assert!(stuck < 2.5, "seed {seed} stuck {stuck}");
            assert!(travelled > 20.0);
        }
    }
}
