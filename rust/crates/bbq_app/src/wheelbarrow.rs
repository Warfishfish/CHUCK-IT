//! Host-owned wheelbarrow. Guests request role changes and render the host's snapshot.
use bbq_core::{
    GameMode, PlayerId,
    flight::ItemState,
    items::ItemKind,
    movement::{CHAR_RADIUS, Mover},
    net_act::GuestAct,
    net_world::WheelbarrowSnap,
    yard::{self, Kind, WHEELBARROW_HOME, Yard},
};

use crate::{
    game::Game,
    player::{PLAYER_ID, Player, Wanted},
};

const REACH: f32 = 1.55;
const HANDLE_REACH: f32 = 2.05;
const SEAT_HEIGHT: f32 = 0.68;
const PUSH_STEP: f32 = 0.075;
const REFILL_REACH: f32 = 2.8;
const REFILL_SECONDS: f32 = 1.7;
const EMPTY: f32 = 0.0;
use yard::{BALLOON_STATION, WHEELBARROW_CAPACITY};

pub struct State {
    pub at: (f32, f32),
    pub pusher: Option<PlayerId>,
    pub rider: Option<PlayerId>,
    grip: (f32, f32),
    pub stock: u8,
    pub delivery: f32,
    refill: Option<u32>,
    exit: Option<PlayerId>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            at: WHEELBARROW_HOME,
            pusher: None,
            rider: None,
            grip: (0., 0.),
            stock: WHEELBARROW_CAPACITY,
            delivery: EMPTY,
            refill: None,
            exit: None,
        }
    }
}

impl State {
    pub fn snapshot(&self, net: impl Fn(u32) -> u32) -> WheelbarrowSnap {
        WheelbarrowSnap {
            at: self.at,
            pusher: self.pusher.map_or(0, &net),
            rider: self.rider.map_or(0, net),
            balloon: self.refill.unwrap_or(0),
            stock: self.stock,
            delivery: self.delivery,
        }
    }

    pub fn apply(&mut self, value: Option<WheelbarrowSnap>, local: impl Fn(u32) -> u32) {
        *self = Self::default();
        if let Some(cart) = value {
            self.at = (
                cart.at
                    .0
                    .clamp(-bbq_core::YARD_HALF_X, bbq_core::YARD_HALF_X),
                cart.at
                    .1
                    .clamp(-bbq_core::YARD_HALF_Z, bbq_core::YARD_HALF_Z),
            );
            self.pusher = (cart.pusher != 0).then(|| local(cart.pusher));
            self.rider = (cart.rider != 0).then(|| local(cart.rider));
            self.refill = (cart.balloon != 0).then_some(cart.balloon);
            self.stock = cart.stock;
            self.delivery = cart.delivery;
        }
    }

    pub fn remaining(&self) -> u8 {
        self.stock + u8::from(self.refill.is_some())
    }

    fn docked(&self) -> bool {
        (self.at.0 - BALLOON_STATION.0).hypot(self.at.1 - BALLOON_STATION.1) < REFILL_REACH
    }
}

pub fn available(mode: GameMode) -> bool {
    matches!(mode, GameMode::FreeForAll | GameMode::Teams)
}

fn eligible(g: &Game, id: PlayerId) -> bool {
    crate::smoko_mode::able(g, id)
        && !crate::smoko_mode::controlled(g, id)
        && !g.smoko.carries.contains_key(&id)
        && if id == PLAYER_ID {
            g.life.carry.is_none() && g.life.seated.is_none()
        } else {
            g.dummies
                .iter()
                .any(|d| d.id == id && d.remote.is_some() && d.seat.is_none())
        }
}

fn near(g: &Game, p: &Player, id: PlayerId) -> bool {
    let (x, z) = g.wheelbarrow.at;
    crate::smoko_mode::pos(g, p, id)
        .is_some_and(|pos| pos.y.abs() < 0.45 && (pos.x - x).hypot(pos.z - z) < REACH)
}

fn at_handles(g: &Game, p: &Player, id: PlayerId) -> bool {
    let (x, z) = g.wheelbarrow.at;
    crate::smoko_mode::pos(g, p, id).is_some_and(|pos| {
        pos.y.abs() < 0.45
            && (pos.x - x).abs() < 0.78
            && (z - pos.z >= 1.20)
            && (z - pos.z < HANDLE_REACH)
    })
}

fn can_refill(g: &Game, p: &Player, id: PlayerId) -> bool {
    g.wheelbarrow.docked()
        && (g.wheelbarrow.rider == Some(id) || g.wheelbarrow.pusher == Some(id) || near(g, p, id))
}

pub fn input(g: &mut Game, p: &Player, id: PlayerId, push: bool) {
    if !available(g.rules.mode) || !g.rules.in_play() || g.attract {
        return;
    }
    if !push && can_refill(g, p, id) && g.wheelbarrow.remaining() < WHEELBARROW_CAPACITY {
        if eligible(g, id) && g.wheelbarrow.delivery == EMPTY {
            g.wheelbarrow.delivery = f32::EPSILON;
        }
        return;
    }
    if push && g.wheelbarrow.pusher == Some(id) {
        g.wheelbarrow.pusher = None;
    } else if !push && g.wheelbarrow.rider == Some(id) {
        g.wheelbarrow.exit = Some(id);
    } else if (if push {
        at_handles(g, p, id)
    } else {
        near(g, p, id)
    }) && eligible(g, id)
    {
        let state = &mut g.wheelbarrow;
        if push && state.pusher.is_none() && state.rider != Some(id) {
            let (x, z) = if id == PLAYER_ID {
                (p.mover.x, p.mover.z)
            } else {
                let d = g.dummies.iter().find(|d| d.id == id).unwrap();
                (d.mover.x, d.mover.z)
            };
            state.pusher = Some(id);
            state.grip = (x - state.at.0, z - state.at.1);
        } else if !push && state.rider.is_none() && state.pusher != Some(id) && state.exit.is_none()
        {
            state.rider = Some(id);
        }
    }
}

pub fn read(g: &mut Game, p: &Player, wanted: &mut Wanted) {
    let push = std::mem::take(&mut wanted.wheelbarrow_push);
    if !available(g.rules.mode) || !g.rules.in_play() || g.attract {
        return;
    }
    let sit = wanted.interact_pressed
        && (can_refill(g, p, PLAYER_ID) && g.wheelbarrow.remaining() < WHEELBARROW_CAPACITY
            || g.wheelbarrow.rider == Some(PLAYER_ID)
            || g.wheelbarrow.rider.is_none()
                && g.wheelbarrow.pusher != Some(PLAYER_ID)
                && near(g, p, PLAYER_ID));
    if sit {
        wanted.interact_pressed = false;
    }
    for (pressed, push) in [(push, true), (sit, false)] {
        if !pressed {
            continue;
        }
        if g.mirror {
            g.net_acts.push(GuestAct::Wheelbarrow { push });
        } else {
            input(g, p, PLAYER_ID, push);
        }
    }
}

pub fn prompt(g: &Game, p: &Player) -> Option<String> {
    if !available(g.rules.mode) || !g.rules.in_play() || g.attract {
        return None;
    }
    let cart = &g.wheelbarrow;
    let left = cart.remaining();
    if can_refill(g, p, PLAYER_ID) && cart.delivery > EMPTY {
        Some("Wheelbarrow • Refilling at the pool".into())
    } else if can_refill(g, p, PLAYER_ID) && left < WHEELBARROW_CAPACITY {
        Some(format!(
            "Wheelbarrow • {left}/20 balloons • R: ask the attendant to refill"
        ))
    } else if cart.rider == Some(PLAYER_ID) {
        Some(format!("Wheelbarrow • {left}/20 balloons • R: hop out"))
    } else if cart.pusher == Some(PLAYER_ID) {
        Some(format!("Wheelbarrow • {left}/20 balloons • V: let go"))
    } else if near(g, p, PLAYER_ID) || at_handles(g, p, PLAYER_ID) {
        Some(format!(
            "Wheelbarrow • {left}/20 balloons • R: ride • V at the handles: push"
        ))
    } else if (p.mover.x - BALLOON_STATION.0).hypot(p.mover.z - BALLOON_STATION.1) < 3.0 {
        Some("Pool refill station • Bring the wheelbarrow here".into())
    } else {
        None
    }
}

fn overlap(a: yard::Collider, b: &yard::Collider) -> bool {
    a.x0 < b.x1 && a.x1 > b.x0 && a.z0 < b.z1 && a.z1 > b.z0
}

fn solid_at(yard: &Yard, at: (f32, f32), at_body: Option<(f32, f32)>) -> bool {
    let body =
        at_body.map(|(x, z)| yard::Collider::centred(x, z, CHAR_RADIUS * 2., CHAR_RADIUS * 2., 1.));
    let [tray, handles] = yard::wheelbarrow_colliders(at);
    [Some(tray), Some(handles), body]
        .into_iter()
        .flatten()
        .any(|probe| {
            probe.x0 < -bbq_core::YARD_HALF_X + 0.45
                || probe.x1 > bbq_core::YARD_HALF_X - 0.45
                || probe.z0 < -bbq_core::YARD_HALF_Z + 0.45
                || probe.z1 > bbq_core::YARD_HALF_Z - 0.45
                || (probe.x0 < yard::POOL_X1
                    && probe.x1 > yard::POOL_X0
                    && probe.z0 < yard::POOL_Z1
                    && probe.z1 > yard::POOL_Z0)
                || overlap(
                    probe,
                    &yard::Collider::centred(yard::TRAMP_X, yard::TRAMP_Z, 5., 5., 1.),
                )
                || yard.colliders.iter().any(|c| {
                    !matches!(
                        c.kind,
                        Kind::Wheelbarrow | Kind::VerandaRoof | Kind::HouseRoof
                    ) && overlap(probe, c)
                })
        })
}

fn place(m: &mut Mover, (x, y, z): (f32, f32, f32)) {
    m.x = x;
    m.y = y;
    m.z = z;
    m.vx = 0.;
    m.vy = 0.;
    m.vz = 0.;
    m.grounded = true;
}

pub fn pin_local(g: &Game, p: &mut Player) -> bool {
    if available(g.rules.mode) && g.wheelbarrow.rider == Some(PLAYER_ID) {
        place(
            &mut p.mover,
            (g.wheelbarrow.at.0, SEAT_HEIGHT, g.wheelbarrow.at.1),
        );
        return true;
    }
    false
}

pub fn pin_peer(g: &mut Game) {
    let (x, z) = g.wheelbarrow.at;
    if let Some(d) = g
        .dummies
        .iter_mut()
        .find(|d| Some(d.id) == g.wheelbarrow.rider)
    {
        place(&mut d.mover, (x, SEAT_HEIGHT, z));
    }
}

pub fn issued(g: &Game, id: u32) -> bool {
    g.wheelbarrow.refill == Some(id)
}

fn supply(g: &mut Game, id: PlayerId) {
    if let Some(item) = g.wheelbarrow.refill.take() {
        if g.world
            .items
            .get(&item)
            .is_some_and(|it| it.state == ItemState::Held && it.holder == Some(id))
        {
            if g.wheelbarrow.exit.is_none() {
                g.wheelbarrow.refill = Some(item);
                return;
            }
            g.world.remove(item);
            g.wheelbarrow.stock = (g.wheelbarrow.stock + 1).min(WHEELBARROW_CAPACITY);
            if id == PLAYER_ID {
                g.slots.remove(item);
                g.wind.cancel();
            } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
                d.bot.slots.remove(item);
            }
        }
    }
    if g.wheelbarrow.exit.is_some() || g.wheelbarrow.stock == 0 {
        return;
    }
    let slots = if id == PLAYER_ID {
        &mut g.slots
    } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
        &mut d.bot.slots
    } else {
        return;
    };
    if slots.is_full() {
        return;
    }
    let (x, z) = g.wheelbarrow.at;
    let item = g
        .world
        .spawn(ItemKind::WaterBalloon, x, z, false, &mut g.rng);
    g.world.give(item, id);
    slots.add(item);
    g.wheelbarrow.refill = Some(item);
    g.wheelbarrow.stock -= 1;
}

pub fn step(g: &mut Game, p: &mut Player, yard: &mut Yard) {
    if !available(g.rules.mode) || !g.rules.in_play() || g.attract {
        return;
    }
    let (x, z) = g.wheelbarrow.at;
    if let Some(id) = g.wheelbarrow.pusher {
        if let Some(pos) = crate::smoko_mode::pos(g, p, id).filter(|_| eligible(g, id)) {
            let (gx, gz) = g.wheelbarrow.grip;
            let (dx, dz) = (pos.x - x - gx, pos.z - z - gz);
            let distance = dx.hypot(dz);
            if distance > 0.0001 {
                let k = (PUSH_STEP / distance).min(1.);
                let next = (x + dx * k, z + dz * k);
                if !solid_at(yard, next, Some((next.0 + gx, next.1 + gz))) {
                    g.wheelbarrow.at = next;
                }
            }
            let (x, z) = g.wheelbarrow.at;
            if id == PLAYER_ID {
                p.mover.x = x + gx;
                p.mover.y = 0.;
                p.mover.z = z + gz;
                p.mover.vy = 0.;
            } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
                d.mover.x = x + gx;
                d.mover.y = 0.;
                d.mover.z = z + gz;
                d.mover.vy = 0.;
            }
        } else {
            g.wheelbarrow.pusher = None;
        }
    }
    yard.set_wheelbarrow(g.wheelbarrow.at);
    let cart = &mut g.wheelbarrow;
    if cart.delivery > EMPTY {
        if !cart.docked() {
            cart.delivery = EMPTY;
        } else {
            cart.delivery += bbq_core::sim::TICK_DT / REFILL_SECONDS;
            if cart.delivery >= 1.0 {
                cart.delivery = EMPTY;
                cart.stock = WHEELBARROW_CAPACITY
                    - u8::from(cart.refill.is_some_and(|item| {
                        g.world
                            .items
                            .get(&item)
                            .is_some_and(|i| i.state == ItemState::Held)
                    }));
            }
        }
    }
    if let Some(id) = g.wheelbarrow.rider {
        if !eligible(g, id) {
            g.wheelbarrow.exit = Some(id);
        }
        if g.wheelbarrow.exit.is_some() {
            let (x, z) = g.wheelbarrow.at;
            let candidates = [
                (1.10, 0.),
                (-1.10, 0.),
                (0., 1.35),
                (0., -2.0),
                (1.25, 1.1),
                (-1.25, 1.1),
                (1.25, -1.1),
                (-1.25, -1.1),
            ];
            let landing = candidates
                .into_iter()
                .map(|(dx, dz)| (x + dx, z + dz))
                .find(|&(x, z)| {
                    let probe =
                        yard::Collider::centred(x, z, CHAR_RADIUS * 2., CHAR_RADIUS * 2., 1.);
                    x.abs() < bbq_core::YARD_HALF_X - 0.45
                        && z.abs() < bbq_core::YARD_HALF_Z - 0.45
                        && !yard::in_pool_rect(x, z)
                        && !yard.colliders.iter().any(|c| overlap(probe, c))
                });
            let present = crate::smoko_mode::pos(g, p, id).is_some();
            supply(g, id);
            if landing.is_some() || !present {
                g.wheelbarrow.exit = None;
                g.wheelbarrow.rider = None;
            }
            if let Some((x, z)) = landing {
                if id == PLAYER_ID {
                    place(&mut p.mover, (x, 0., z));
                } else if let Some(d) = g.dummies.iter_mut().find(|d| d.id == id) {
                    place(&mut d.mover, (x, 0., z));
                    g.net_hits.push((
                        id,
                        bbq_core::net_act::HitMsg::Smoko {
                            pos: (x, 0., z),
                            vel: (0., 0., 0.),
                            locked: false,
                            carrying: false,
                        },
                    ));
                }
            }
        } else {
            supply(g, id);
        }
    }
    pin_local(g, p);
    pin_peer(g);
}

/// Pool attendant and tap in station-local space, facing the yard.
pub fn station_parts() -> Vec<bbq_core::looks::Part> {
    use bbq_core::looks::{Part, Shape, Surface};
    let sphere = |r, c| Part::new(Shape::Sphere { r, ws: 12, hs: 8 }, Surface::matt(c));
    let board = |w, h, d, c| Part::new(Shape::Cuboid { w, h, d }, Surface::matt(c)).bevel(0.035);
    let shorts = 0x29455e;
    let skin = 0xd49964;
    let shirt = 0x319588;
    let enamel = 0x2d7aa2;
    let tap = 0xbac5ca;
    let mut parts = vec![
        sphere(0.29, shirt)
            .stretch(0.91, 1.14, 1.0)
            .at(-0.33, 0.77, 0.0),
        sphere(0.23, skin).at(-0.31, 1.24, 0.0),
        sphere(0.24, shorts)
            .stretch(1.04, 0.39, 1.0)
            .at(-0.33, 1.43, 0.0),
        board(0.16, 0.035, 0.32, shorts).at(-0.12, 1.40, 0.0),
        sphere(0.072, skin).at(-0.085, 1.22, 0.0),
        board(0.28, 1.57, 0.34, enamel).at(0.49, 0.785, 0.0),
        board(0.27, 0.09, 0.29, 0xf2e7c1).at(0.49, 1.40, 0.005),
        board(0.14, 0.06, 0.08, tap).at(0.64, 1.08, 0.0),
        board(0.055, 0.10, 0.065, tap).at(0.69, 1.025, 0.0),
        board(0.08, 0.035, 0.20, 0xd36548).at(0.60, 1.15, 0.0),
    ];
    for z in [-0.14, 0.14] {
        parts.push(
            sphere(0.14, shorts)
                .stretch(1.08, 1.4, 0.85)
                .at(-0.33, 0.28, z),
        );
        parts.push(
            sphere(0.13, shorts)
                .stretch(1.32, 0.65, 0.96)
                .at(-0.28, 0.09, z),
        );
        parts.push(sphere(0.027, 0x252834).at(-0.123, 1.27, z * 0.52));
    }
    parts
}
