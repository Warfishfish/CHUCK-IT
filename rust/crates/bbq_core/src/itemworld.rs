//! All the items in the yard, how they spawn, and what happens as they fly, hit people and
//! get caught. This ties `flight`, `hands` and `hitting` together (spec section 3).

use std::collections::BTreeMap;

use crate::flight::{Fate, FlightEvent, Item, ItemId, ItemState, Puddle, Puddles, step_flight};
use crate::hands::{Picker, can_pick_up};
use crate::hitting::{Contact, Context, Target, contact, item_after_hit};
use crate::items::{DildoVariant, ItemKind, random_type, spawn_target};
use crate::rng::Rng;
use crate::vec::V3;
use crate::yard::{POOL_X0, POOL_X1, POOL_Z0, POOL_Z1, Yard};
use crate::{PlayerId, YARD_HALF_X, YARD_HALF_Z};

/// The 25 fixed item spawn spots (spec section 1).
pub const ITEM_SPAWNS: [(f32, f32); 25] = [
    (-9.0, 4.5),
    (6.0, 9.0),
    (12.0, -3.0),
    (-15.0, -6.0),
    (19.5, 19.5),
    (-19.5, 20.25),
    (0.0, -10.5),
    (27.0, -3.0),
    (-27.0, 6.0),
    (7.5, 19.5),
    (-13.5, -19.5),
    (16.5, -21.0),
    (-3.0, -1.5),
    (3.0, 15.0),
    (-25.0, -17.5),
    (-9.0, 12.75),
    (21.0, -7.5),
    (-17.0, 3.0),
    (20.0, 14.0),
    (-2.0, 7.0),
    (8.0, -12.0),
    (-24.0, 17.0),
    (26.0, 12.0),
    (-6.0, -12.0),
    (4.0, -19.5),
];

pub const SPAWN_EVERY: f32 = 1.3;
/// Keep this many noodles floating in the pool.
pub const NOODLES: usize = 2;
pub const NOODLE_EVERY: f32 = 4.0;
/// A new item must be at least this far from any other item that isn't being held.
pub const SPAWN_CLEAR: f32 = 2.2;

/// Things the game should react to.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WorldEvent {
    Flight {
        item: ItemId,
        ev: FlightEvent,
    },
    /// A stubby broke and left a puddle at this spot.
    Puddle {
        x: f32,
        z: f32,
    },
    /// A thrown item hit `victim`. The caller applies knockback, stun and points.
    Hit {
        item: ItemId,
        kind: ItemKind,
        victim: PlayerId,
        thrower: Option<PlayerId>,
        dir: V3,
        flatten: bool,
        charge: f32,
        start: V3,
    },
    /// `by` caught the item. It is now held by them (the caller adds it to their slots).
    Caught {
        item: ItemId,
        by: PlayerId,
        thrower: Option<PlayerId>,
    },
    /// An item was removed from the yard.
    Removed(ItemId),
    /// A new item appeared (dropped from the sky).
    Spawned(ItemId),
}

#[derive(Clone, Debug)]
pub struct ItemWorld {
    pub items: BTreeMap<ItemId, Item>,
    pub puddles: Puddles,
    next_id: ItemId,
    spawn_t: f32,
    noodle_t: f32,
}

impl Default for ItemWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemWorld {
    pub fn new() -> Self {
        Self {
            items: BTreeMap::new(),
            puddles: Puddles::default(),
            next_id: 1,
            spawn_t: SPAWN_EVERY,
            noodle_t: 0.0,
        }
    }

    /// Clear everything for a new round.
    pub fn clear(&mut self) {
        self.items.clear();
        self.puddles.0.clear();
        self.spawn_t = SPAWN_EVERY;
        self.noodle_t = NOODLE_EVERY;
    }

    /// Make an item. `drop` makes it fall from 7..10 m (harmless until it lands).
    pub fn spawn(&mut self, kind: ItemKind, x: f32, z: f32, drop: bool, rng: &mut Rng) -> ItemId {
        let id = self.next_id;
        self.next_id += 1;
        let y = if drop {
            rng.range(7.0, 10.0)
        } else {
            kind.def().radius + 0.02
        };
        let mut it = Item::new(id, kind, V3::new(x, y, z));
        it.state = ItemState::Flying;
        it.live = false;
        if kind == ItemKind::Dildo {
            it.variant = Some(crate::items::pick_dildo_variant(rng.f32()));
        }
        self.items.insert(id, it);
        id
    }

    /// Start of a round: `8 + floor(players * 1.5)` items on shuffled spots (the first three
    /// are a teddy, a stubby and a gnome), plus the noodles.
    pub fn initial_fill(
        &mut self,
        players: usize,
        heist_clear: &dyn Fn(f32, f32) -> bool,
        rng: &mut Rng,
    ) {
        self.clear();
        let mut spots: Vec<(f32, f32)> = ITEM_SPAWNS
            .iter()
            .copied()
            .filter(|s| heist_clear(s.0, s.1))
            .collect();
        rng.shuffle(&mut spots);
        let n = spots.len().min(crate::items::initial_items(players));
        for (k, s) in spots.iter().take(n).enumerate() {
            let kind = match k {
                0 => ItemKind::Teddy,
                1 => ItemKind::Stubby,
                2 => ItemKind::Gnome,
                _ => random_type(rng.f32()),
            };
            self.spawn(kind, s.0, s.1, true, rng);
        }
        for _ in 0..NOODLES {
            self.spawn_noodle(rng);
        }
    }

    fn spawn_noodle(&mut self, rng: &mut Rng) -> ItemId {
        let x = rng.range(POOL_X0 + 1.2, POOL_X1 - 1.2);
        let z = rng.range(POOL_Z0 + 0.9, POOL_Z1 - 0.9);
        let id = self.spawn(ItemKind::Noodle, x, z, false, rng);
        if let Some(it) = self.items.get_mut(&id) {
            it.pos.y = 0.6;
        }
        id
    }

    pub fn count(&self, kind: ItemKind) -> usize {
        self.items.values().filter(|i| i.kind == kind).count()
    }

    /// Put an item in someone's hands.
    pub fn give(&mut self, id: ItemId, player: PlayerId) {
        if let Some(it) = self.items.get_mut(&id) {
            it.state = ItemState::Held;
            it.holder = Some(player);
            it.thrower = None;
            it.live = false;
            it.vel = V3::ZERO;
        }
    }

    /// Drop a held item at `pos`; `block` stops that player re-grabbing it for 2.5 s.
    pub fn drop_item(&mut self, id: ItemId, pos: V3, block: Option<(PlayerId, f32)>) {
        if let Some(it) = self.items.get_mut(&id) {
            it.state = ItemState::Flying;
            it.holder = None;
            it.live = false;
            it.pos = pos;
            it.vel = V3::ZERO;
            it.block = block;
        }
    }

    /// Throw a held item.
    pub fn throw(
        &mut self,
        id: ItemId,
        by: PlayerId,
        start: V3,
        vel: V3,
        charge: f32,
        power_ok: bool,
    ) -> bool {
        let Some(it) = self.items.get_mut(&id) else {
            return false;
        };
        if it.state != ItemState::Held || it.holder != Some(by) {
            return false;
        }
        it.state = ItemState::Flying;
        it.holder = None;
        it.thrower = Some(by);
        it.live = true;
        it.fly_t = 0.0;
        it.ignore.clear();
        it.pos = start;
        it.vel = vel;
        it.start = start;
        it.charge = charge.clamp(0.0, 1.0);
        it.power = power_ok && crate::hands::is_power(charge);
        it.flight_no += 1;
        it.bounces = 0;
        true
    }

    /// The nearest item `p` could pick up right now, if any.
    pub fn pickup_for(&self, p: &Picker, now: f32) -> Option<ItemId> {
        self.items
            .values()
            .filter(|it| can_pick_up(p, it, now))
            .min_by(|a, b| a.pos.horiz_dist(p.pos).total_cmp(&b.pos.horiz_dist(p.pos)))
            .map(|it| it.id)
    }

    pub fn remove(&mut self, id: ItemId) {
        self.items.remove(&id);
    }

    /// One fixed step: spawn, fly, land, hit, catch, clean up puddles.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        dt: f32,
        yard: &Yard,
        rng: &mut Rng,
        targets: &[Target],
        ctx: &Context,
        players: usize,
        spawning: bool,
        heist_clear: &dyn Fn(f32, f32) -> bool,
    ) -> Vec<WorldEvent> {
        let mut out = Vec::new();
        self.puddles.tick(dt);

        if spawning {
            self.noodle_t -= dt;
            if self.noodle_t <= 0.0 {
                self.noodle_t = NOODLE_EVERY;
                if self.count(ItemKind::Noodle) < NOODLES {
                    let id = self.spawn_noodle(rng);
                    out.push(WorldEvent::Spawned(id));
                }
            }
            self.spawn_t -= dt;
            if self.spawn_t <= 0.0 {
                self.spawn_t = SPAWN_EVERY;
                if (self.items.len() as f32)
                    < spawn_target(players) as f32 + spawn_target_fraction(players)
                {
                    let free: Vec<(f32, f32)> = ITEM_SPAWNS
                        .iter()
                        .copied()
                        .filter(|s| {
                            heist_clear(s.0, s.1)
                                && !self.items.values().any(|it| {
                                    it.state != ItemState::Held
                                        && it.pos.horiz_dist(V3::new(s.0, 0.0, s.1)) < SPAWN_CLEAR
                                })
                        })
                        .collect();
                    if !free.is_empty() {
                        let s = free[rng.index(free.len())];
                        let kind = random_type(rng.f32());
                        let id = self.spawn(
                            kind,
                            s.0 + rng.range(-0.4, 0.4),
                            s.1 + rng.range(-0.4, 0.4),
                            true,
                            rng,
                        );
                        out.push(WorldEvent::Spawned(id));
                    }
                }
            }
        }

        let ids: Vec<ItemId> = self.items.keys().copied().collect();
        for id in ids {
            let Some(mut it) = self.items.remove(&id) else {
                continue;
            };
            match it.state {
                ItemState::Ground => {
                    it.rest_t += dt;
                    self.items.insert(id, it);
                }
                ItemState::Held => {
                    self.items.insert(id, it);
                }
                ItemState::Flying => {
                    let mut fl = Vec::new();
                    let mut contacts: Vec<WorldEvent> = Vec::new();
                    let mut smashed_by_hit = false;
                    let fate = step_flight(&mut it, dt, yard, rng, &mut fl, &mut |item| {
                        for t in targets {
                            match contact(item, t, ctx) {
                                Contact::Miss => {}
                                Contact::PassThrough => item.ignore.push(t.id),
                                Contact::Catch => {
                                    let thrower = item.thrower;
                                    item.state = ItemState::Held;
                                    item.holder = Some(t.id);
                                    item.live = false;
                                    item.vel = V3::ZERO;
                                    item.thrower = None;
                                    contacts.push(WorldEvent::Caught {
                                        item: item.id,
                                        by: t.id,
                                        thrower,
                                    });
                                    return true;
                                }
                                Contact::Hit { dir, flatten } => {
                                    contacts.push(WorldEvent::Hit {
                                        item: item.id,
                                        kind: item.kind,
                                        victim: t.id,
                                        thrower: item.thrower,
                                        dir,
                                        flatten,
                                        charge: item.charge,
                                        start: item.start,
                                    });
                                    if item_after_hit(item) {
                                        smashed_by_hit = true;
                                        item.state = ItemState::Ground; // stops the flight loop
                                    }
                                    return true;
                                }
                            }
                        }
                        false
                    });
                    out.extend(contacts);
                    let mut removed = fate == Fate::Removed;
                    if smashed_by_hit {
                        let puddle =
                            it.pos.y < 1.4 && !crate::yard::in_pool_rect(it.pos.x, it.pos.z);
                        fl.push(FlightEvent::Smash {
                            pos: it.pos,
                            puddle,
                        });
                        removed = true;
                    }
                    for ev in &fl {
                        if let FlightEvent::Smash { pos, puddle: true } = ev {
                            self.puddles.0.push(Puddle::new(pos.x, pos.z, rng));
                            out.push(WorldEvent::Puddle { x: pos.x, z: pos.z });
                        }
                        out.push(WorldEvent::Flight { item: id, ev: *ev });
                    }
                    if removed {
                        out.push(WorldEvent::Removed(id));
                    } else {
                        self.items.insert(id, it);
                    }
                }
            }
        }
        out
    }
}

/// `spawn_target` gives a whole number; the real target is `min(22, 6 + players * 1.6)`.
/// Items spawn while the count is below the real (fractional) target, which the whole number
/// rounds down, so this adds back the fraction.
fn spawn_target_fraction(players: usize) -> f32 {
    let exact = (6.0 + players as f32 * 1.6).min(22.0);
    exact - spawn_target(players) as f32
}

/// Convenience for tests and the game: no Heist walls.
pub fn no_walls(_x: f32, _z: f32) -> bool {
    true
}

/// Is `(x, z)` inside the yard?
pub fn in_yard(x: f32, z: f32) -> bool {
    x.abs() <= YARD_HALF_X && z.abs() <= YARD_HALF_Z
}

/// Give a chest-made dildo its variant (so it keeps it).
pub fn set_variant(it: &mut Item, v: DildoVariant) {
    it.variant = Some(v);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::teams::Teams;

    const DT: f32 = 1.0 / 60.0;

    fn run(
        w: &mut ItemWorld,
        secs: f32,
        targets: &[Target],
        rng: &mut Rng,
        spawning: bool,
    ) -> Vec<WorldEvent> {
        let yard = Yard::default();
        let teams = Teams::default();
        let ctx = Context {
            friendly_fire: false,
            teams: &teams,
        };
        let mut all = Vec::new();
        for _ in 0..(secs / DT) as usize {
            all.extend(w.step(DT, &yard, rng, targets, &ctx, 4, spawning, &no_walls));
        }
        all
    }

    fn victim(id: PlayerId, x: f32, z: f32) -> Target {
        Target {
            id,
            pos: V3::new(x, 0.0, z),
            sink: 0.0,
            at_smoko: false,
            catching: false,
            stunned: false,
            facing: V3::new(0.0, 0.0, 1.0),
            flattenable: true,
        }
    }

    #[test]
    fn start_of_round_places_8_plus_one_and_a_half_per_player_plus_two_noodles() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(1);
        w.initial_fill(4, &no_walls, &mut rng);
        assert_eq!(w.items.len(), 14 + 2);
        assert_eq!(w.count(ItemKind::Noodle), 2);
        // the first three non-noodle items are a teddy, a stubby and a gnome
        for k in [ItemKind::Teddy, ItemKind::Stubby, ItemKind::Gnome] {
            assert!(w.count(k) >= 1);
        }
        // all start in the air except noodles
        assert!(
            w.items
                .values()
                .filter(|i| i.kind != ItemKind::Noodle)
                .all(|i| i.pos.y >= 7.0 && !i.live)
        );
    }

    #[test]
    fn dropped_items_land_and_can_then_be_picked_up() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(2);
        w.initial_fill(2, &no_walls, &mut rng);
        run(&mut w, 6.0, &[], &mut rng, false);
        let grounded = w
            .items
            .values()
            .filter(|i| i.state == ItemState::Ground)
            .count();
        assert!(
            grounded as f32 >= w.items.len() as f32 * 0.8,
            "{grounded} of {}",
            w.items.len()
        );
        let it = w
            .items
            .values()
            .find(|i| i.state == ItemState::Ground)
            .unwrap()
            .clone();
        let p = Picker {
            id: 1,
            pos: V3::new(it.pos.x, 0.0, it.pos.z),
            held: 0,
            stunned: false,
            frozen: false,
            is_bot: false,
        };
        assert!(w.pickup_for(&p, 10.0).is_some());
    }

    #[test]
    fn the_spawner_tops_up_to_the_target_and_stops() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(3);
        run(&mut w, 60.0, &[], &mut rng, true);
        // 4 players: target min(22, 6 + 6.4) = 12.4. Everything counts (noodles too), and it
        // keeps spawning while the count is below the target, so it stops at 13.
        assert_eq!(w.items.len(), 13);
        assert_eq!(w.count(ItemKind::Noodle), 2);
    }

    #[test]
    fn a_thrown_item_hits_a_person_and_reports_it() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(4);
        let id = w.spawn(ItemKind::Teddy, 0.0, 0.0, false, &mut rng);
        w.give(id, 1);
        assert!(w.throw(
            id,
            1,
            V3::new(2.0, 1.2, 0.0),
            V3::new(0.0, 0.0, 20.0),
            0.5,
            true
        ));
        let ev = run(&mut w, 1.0, &[victim(2, 2.0, 6.0)], &mut rng, false);
        let hit = ev.iter().find_map(|e| match e {
            WorldEvent::Hit {
                victim, thrower, ..
            } => Some((*victim, *thrower)),
            _ => None,
        });
        assert_eq!(hit, Some((2, Some(1))));
        // the item bounced off and is no longer live
        assert!(!w.items[&id].live);
    }

    #[test]
    fn a_hit_registers_only_once_per_throw() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(4);
        let id = w.spawn(ItemKind::Gnome, 0.0, 0.0, false, &mut rng);
        w.give(id, 1);
        w.throw(
            id,
            1,
            V3::new(2.0, 1.2, 0.0),
            V3::new(0.0, 0.0, 15.0),
            1.0,
            true,
        );
        let ev = run(&mut w, 2.0, &[victim(2, 2.0, 3.0)], &mut rng, false);
        assert_eq!(
            ev.iter()
                .filter(|e| matches!(e, WorldEvent::Hit { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn a_stubby_that_hits_someone_smashes_and_leaves_a_puddle() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(5);
        let id = w.spawn(ItemKind::Stubby, 0.0, 0.0, false, &mut rng);
        w.give(id, 1);
        w.throw(
            id,
            1,
            V3::new(2.0, 1.2, 0.0),
            V3::new(0.0, 0.0, 20.0),
            1.0,
            true,
        );
        let ev = run(&mut w, 1.0, &[victim(2, 2.0, 3.0)], &mut rng, false);
        assert!(ev.iter().any(|e| matches!(e, WorldEvent::Hit { .. })));
        assert!(ev.iter().any(|e| matches!(e, WorldEvent::Puddle { .. })));
        assert!(!w.items.contains_key(&id));
        assert_eq!(w.puddles.0.len(), 1);
    }

    #[test]
    fn a_catch_puts_it_in_the_catchers_hands() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(6);
        let id = w.spawn(ItemKind::Teddy, 0.0, 0.0, false, &mut rng);
        w.give(id, 1);
        w.throw(
            id,
            1,
            V3::new(2.0, 1.2, 0.0),
            V3::new(0.0, 0.0, 20.0),
            0.5,
            true,
        );
        let mut t = victim(2, 2.0, 6.0);
        t.catching = true;
        t.facing = V3::new(0.0, 0.0, -1.0);
        let ev = run(&mut w, 1.0, &[t], &mut rng, false);
        assert!(ev.iter().any(|e| matches!(
            e,
            WorldEvent::Caught {
                by: 2,
                thrower: Some(1),
                ..
            }
        )));
        assert_eq!(w.items[&id].state, ItemState::Held);
        assert_eq!(w.items[&id].holder, Some(2));
        assert!(!ev.iter().any(|e| matches!(e, WorldEvent::Hit { .. })));
    }

    #[test]
    fn the_thrower_is_never_hit_by_their_own_throw() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(7);
        let id = w.spawn(ItemKind::Teddy, 0.0, 0.0, false, &mut rng);
        w.give(id, 1);
        w.throw(
            id,
            1,
            V3::new(0.0, 1.2, 0.3),
            V3::new(0.0, 0.0, 20.0),
            0.5,
            true,
        );
        let ev = run(&mut w, 0.5, &[victim(1, 0.0, 0.0)], &mut rng, false);
        assert!(!ev.iter().any(|e| matches!(e, WorldEvent::Hit { .. })));
    }

    #[test]
    fn a_power_throw_is_flagged_only_when_allowed() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(8);
        let a = w.spawn(ItemKind::Teddy, 0.0, 0.0, false, &mut rng);
        let b = w.spawn(ItemKind::Teddy, 1.0, 0.0, false, &mut rng);
        w.give(a, 1);
        w.give(b, 1);
        w.throw(a, 1, V3::ZERO, V3::new(0.0, 0.0, 10.0), 1.0, true);
        w.throw(b, 1, V3::ZERO, V3::new(0.0, 0.0, 10.0), 1.0, false); // a bot
        assert!(w.items[&a].power);
        assert!(!w.items[&b].power, "bots never power-throw");
    }

    #[test]
    fn only_the_holder_can_throw_and_drop_blocks_a_regrab() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(9);
        let id = w.spawn(ItemKind::Teddy, 0.0, 0.0, false, &mut rng);
        w.give(id, 1);
        assert!(!w.throw(id, 2, V3::ZERO, V3::ZERO, 0.5, true));
        w.drop_item(id, V3::new(0.0, 1.0, 0.0), Some((1, 99.0)));
        run(&mut w, 3.0, &[], &mut rng, false);
        let p = Picker {
            id: 1,
            pos: V3::new(0.0, 0.0, 0.0),
            held: 0,
            stunned: false,
            frozen: false,
            is_bot: false,
        };
        assert_eq!(w.pickup_for(&p, 5.0), None, "blocked");
        let p2 = Picker { id: 2, ..p };
        assert_eq!(w.pickup_for(&p2, 5.0), Some(id));
    }

    #[test]
    fn noodles_are_topped_up_every_four_seconds() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(10);
        w.initial_fill(2, &no_walls, &mut rng);
        let noodle = w
            .items
            .values()
            .find(|i| i.kind == ItemKind::Noodle)
            .unwrap()
            .id;
        w.remove(noodle);
        assert_eq!(w.count(ItemKind::Noodle), 1);
        run(&mut w, 4.5, &[], &mut rng, true);
        assert_eq!(w.count(ItemKind::Noodle), 2);
    }

    #[test]
    fn clearing_removes_items_and_puddles() {
        let mut w = ItemWorld::new();
        let mut rng = Rng::new(11);
        w.initial_fill(4, &no_walls, &mut rng);
        w.puddles.0.push(Puddle::new(0.0, 0.0, &mut rng));
        w.clear();
        assert!(w.items.is_empty() && w.puddles.0.is_empty());
    }
}
