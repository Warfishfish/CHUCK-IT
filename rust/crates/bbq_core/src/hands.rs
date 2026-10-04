//! What you hold and how you throw it: two slots, swapping, picking up, winding up,
//! holding too long, and the throw velocity (spec sections 2 and 3).

use crate::flight::{Item, ItemId, ItemState};
use crate::items::{ItemKind, Melee};
use crate::stun::POWER_CHARGE;
use crate::vec::V3;

pub const MAX_HELD: usize = 2;
pub const PICKUP_RANGE: f32 = 1.1;
pub const PICKUP_VERTICAL: f32 = 1.4;
/// An item must have been lying still this long before it can be picked up.
pub const PICKUP_REST: f32 = 0.2;
/// If charge stays full this long you drop the item.
pub const OVERHOLD: f32 = 1.5;
/// After over-holding (or being made to drop something) you can't pick it up for this long.
pub const OVERHOLD_BLOCK: f32 = 2.5;
/// Right-hand position in camera space (x right, y up, -z forward).
pub const THROW_HAND: V3 = V3::new(0.28, -0.22, -0.55);
/// Throws are aimed a little upward.
pub const AIM_LIFT: f32 = 0.07;
/// A tap shorter than this on a throwable melee item (the noodle) slaps instead.
pub const TAP_SLAP: f32 = 0.2;

/// A charge of at least this makes a power throw.
pub fn is_power(charge: f32) -> bool {
    charge >= POWER_CHARGE
}

/// Launch velocity: aim direction (lifted a touch) times `speed * (0.42 + 0.58 * charge)`,
/// plus 30% of the thrower's own horizontal velocity.
pub fn throw_velocity(aim: V3, item_speed: f32, charge: f32, thrower_vel: V3) -> V3 {
    let mut dir = aim;
    dir.y += AIM_LIFT;
    let dir = dir.normalised();
    dir * (item_speed * (0.42 + 0.58 * charge))
        + V3::new(thrower_vel.x * 0.3, 0.0, thrower_vel.z * 0.3)
}

/// Why you can't start winding up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refuse {
    HandsFull,
    CantAct,
    NothingHeld,
}

/// What's going on around the player that decides whether they can start a wind-up.
#[derive(Clone, Copy, Debug, Default)]
pub struct Situation {
    pub carrying_someone: bool,
    pub can_act: bool,
    /// Stunned and not lying flat (you may still throw while knocked flat).
    pub stunned_standing: bool,
    pub drinking: bool,
    pub at_smoko: bool,
}

/// The wind-up: hold the button to charge.
#[derive(Clone, Copy, Debug, Default)]
pub struct Wind {
    pub charging: bool,
    pub charge: f32,
    pub started_at: f32,
    pub over_t: f32,
}

/// What releasing the button does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Release {
    Throw {
        charge: f32,
    },
    /// A quick tap on a throwable melee item (noodle) slaps instead.
    Slap,
    /// Nothing was charging.
    Nothing,
}

/// What pressing the button does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Press {
    /// Wind-up started.
    Charging,
    /// A melee-only item: swing it straight away.
    SlapNow,
    Refused(Refuse),
}

impl Wind {
    /// Button pressed with `selected` in hand.
    pub fn press(&mut self, now: f32, selected: Option<ItemKind>, s: &Situation) -> Press {
        if s.carrying_someone {
            return Press::Refused(Refuse::HandsFull);
        }
        if !s.can_act || s.stunned_standing || s.drinking || s.at_smoko {
            return Press::Refused(Refuse::CantAct);
        }
        let Some(kind) = selected else {
            return Press::Refused(Refuse::NothingHeld);
        };
        let d = kind.def();
        if d.melee != Melee::None && !d.throwable {
            return Press::SlapNow;
        }
        self.charging = true;
        self.charge = 0.0;
        self.started_at = now;
        Press::Charging
    }

    /// Advance the charge. Returns true if you've held it too long and must drop the item.
    pub fn tick(&mut self, dt: f32, selected: Option<ItemKind>) -> bool {
        if let Some(kind) = selected
            && self.charging
        {
            self.charge = (self.charge + dt / kind.def().charge).min(1.0);
        }
        if self.charging && self.charge >= 1.0 {
            self.over_t += dt;
            if self.over_t > OVERHOLD {
                self.cancel();
                return true;
            }
        } else {
            self.over_t = 0.0;
        }
        false
    }

    pub fn cancel(&mut self) {
        self.charging = false;
        self.charge = 0.0;
        self.over_t = 0.0;
    }

    /// Button released.
    pub fn release(&mut self, now: f32, selected: Option<ItemKind>, can_act: bool) -> Release {
        if !self.charging {
            return Release::Nothing;
        }
        let Some(kind) = selected.filter(|_| can_act) else {
            self.cancel();
            return Release::Nothing;
        };
        if kind.def().throwable && now - self.started_at < TAP_SLAP {
            self.cancel();
            return Release::Slap;
        }
        let charge = self.charge;
        self.cancel();
        Release::Throw { charge }
    }
}

/// The (up to two) things in your hands and which one is selected.
#[derive(Clone, Debug, Default)]
pub struct Slots {
    held: Vec<ItemId>,
    sel: usize,
}

impl Slots {
    pub fn len(&self) -> usize {
        self.held.len()
    }

    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }

    pub fn ids(&self) -> &[ItemId] {
        &self.held
    }

    pub fn selected(&self) -> Option<ItemId> {
        self.held.get(self.sel).copied()
    }

    pub fn is_full(&self) -> bool {
        self.held.len() >= MAX_HELD
    }

    /// Pick something up; it becomes the selected item. Returns false if your hands are full.
    pub fn add(&mut self, id: ItemId) -> bool {
        if self.is_full() {
            return false;
        }
        self.held.push(id);
        self.sel = self.held.len() - 1;
        true
    }

    pub fn remove(&mut self, id: ItemId) {
        if let Some(i) = self.held.iter().position(|x| *x == id) {
            self.held.remove(i);
            if self.sel >= self.held.len() {
                self.sel = self.held.len().saturating_sub(1);
            }
        }
    }

    /// Q / E / mouse wheel. Returns true if the selection changed (cancel any wind-up).
    pub fn swap(&mut self, dir: i32) -> bool {
        if self.held.len() < 2 {
            return false;
        }
        let n = self.held.len() as i32;
        self.sel = ((self.sel as i32 + dir.signum() + n) % n) as usize;
        true
    }

    /// Keys 1 and 2.
    pub fn select_slot(&mut self, slot: usize) -> bool {
        if slot < self.held.len() && slot != self.sel {
            self.sel = slot;
            true
        } else {
            false
        }
    }

    /// The oldest item (dropped first when a catch would overfill your hands).
    pub fn oldest(&self) -> Option<ItemId> {
        self.held.first().copied()
    }

    pub fn clear(&mut self) {
        self.held.clear();
        self.sel = 0;
    }
}

/// Who is trying to pick things up.
#[derive(Clone, Copy, Debug)]
pub struct Picker {
    pub id: u32,
    pub pos: V3,
    pub held: usize,
    pub stunned: bool,
    pub frozen: bool,
    pub is_bot: bool,
}

/// Can `p` pick up `it` right now? (Not bots and melee items, not blocked, in reach.)
pub fn can_pick_up(p: &Picker, it: &Item, now: f32) -> bool {
    if p.stunned || p.frozen || p.held >= MAX_HELD {
        return false;
    }
    if it.state != ItemState::Ground || it.rest_t < PICKUP_REST {
        return false;
    }
    if p.is_bot && it.kind.def().melee != Melee::None {
        return false;
    }
    if let Some((who, until)) = it.block
        && who == p.id
        && now < until
    {
        return false;
    }
    it.pos.horiz_dist(p.pos) < PICKUP_RANGE && (it.pos.y - (p.pos.y + 0.4)).abs() < PICKUP_VERTICAL
}

#[cfg(test)]
mod tests {
    use super::*;

    fn can() -> Situation {
        Situation {
            can_act: true,
            ..Default::default()
        }
    }

    fn ground_item(kind: ItemKind, x: f32, z: f32) -> Item {
        let mut it = Item::new(1, kind, V3::new(x, 0.24, z));
        it.rest_t = 1.0;
        it
    }

    fn picker() -> Picker {
        Picker {
            id: 1,
            pos: V3::ZERO,
            held: 0,
            stunned: false,
            frozen: false,
            is_bot: false,
        }
    }

    #[test]
    fn charge_fills_at_one_over_item_charge_time() {
        let mut w = Wind::default();
        assert_eq!(w.press(0.0, Some(ItemKind::Teddy), &can()), Press::Charging);
        for _ in 0..30 {
            w.tick(0.01, Some(ItemKind::Teddy)); // 0.3 s of a 0.55 s teddy
        }
        assert!((w.charge - 0.3 / 0.55).abs() < 1e-3);
        for _ in 0..100 {
            w.tick(0.01, Some(ItemKind::Teddy));
        }
        assert_eq!(w.charge, 1.0);
    }

    #[test]
    fn holding_at_full_charge_for_1_5_s_drops_it() {
        let mut w = Wind::default();
        w.press(0.0, Some(ItemKind::Teddy), &can());
        let mut dropped_at = None;
        let mut t = 0.0_f32;
        for _ in 0..400 {
            t += 0.01;
            if w.tick(0.01, Some(ItemKind::Teddy)) {
                dropped_at = Some(t);
                break;
            }
        }
        // 0.55 s to fill, then more than 1.5 s
        let t = dropped_at.expect("never dropped");
        assert!((t - (0.55 + 1.5)).abs() < 0.05, "{t}");
        assert!(!w.charging);
    }

    #[test]
    fn releasing_throws_with_the_charge() {
        let mut w = Wind::default();
        w.press(0.0, Some(ItemKind::Gnome), &can());
        for _ in 0..50 {
            w.tick(0.01, Some(ItemKind::Gnome));
        }
        let Release::Throw { charge } = w.release(0.5, Some(ItemKind::Gnome), true) else {
            panic!()
        };
        assert!((charge - 0.5).abs() < 1e-4);
        assert!(!w.charging);
        assert_eq!(
            w.release(0.6, Some(ItemKind::Gnome), true),
            Release::Nothing
        );
    }

    #[test]
    fn a_tap_on_the_noodle_slaps_but_a_hold_throws_it() {
        let mut w = Wind::default();
        w.press(0.0, Some(ItemKind::Noodle), &can());
        assert_eq!(w.release(0.1, Some(ItemKind::Noodle), true), Release::Slap);
        w.press(1.0, Some(ItemKind::Noodle), &can());
        for _ in 0..30 {
            w.tick(0.01, Some(ItemKind::Noodle));
        }
        assert!(matches!(
            w.release(1.3, Some(ItemKind::Noodle), true),
            Release::Throw { .. }
        ));
    }

    #[test]
    fn melee_only_items_swing_at_once() {
        for k in [ItemKind::Steak, ItemKind::Fish, ItemKind::Dildo] {
            assert_eq!(Wind::default().press(0.0, Some(k), &can()), Press::SlapNow);
        }
    }

    #[test]
    fn cant_wind_up_when_busy() {
        let mut w = Wind::default();
        let kind = Some(ItemKind::Teddy);
        let s = Situation {
            carrying_someone: true,
            can_act: true,
            ..Default::default()
        };
        assert_eq!(w.press(0.0, kind, &s), Press::Refused(Refuse::HandsFull));
        for s in [
            Situation {
                can_act: false,
                ..Default::default()
            },
            Situation {
                stunned_standing: true,
                can_act: true,
                ..Default::default()
            },
            Situation {
                drinking: true,
                can_act: true,
                ..Default::default()
            },
            Situation {
                at_smoko: true,
                can_act: true,
                ..Default::default()
            },
        ] {
            assert_eq!(w.press(0.0, kind, &s), Press::Refused(Refuse::CantAct));
        }
        assert_eq!(
            w.press(0.0, None, &can()),
            Press::Refused(Refuse::NothingHeld)
        );
        assert!(!w.charging);
    }

    #[test]
    fn throw_velocity_scales_with_charge_and_adds_a_bit_of_your_own_motion() {
        let aim = V3::new(0.0, 0.0, -1.0);
        let v0 = throw_velocity(aim, 24.0, 0.0, V3::ZERO);
        let v1 = throw_velocity(aim, 24.0, 1.0, V3::ZERO);
        assert!((v0.len() - 24.0 * 0.42).abs() < 1e-3);
        assert!((v1.len() - 24.0).abs() < 1e-3);
        assert!(v1.y > 0.0, "lifted a touch");
        let running = throw_velocity(aim, 24.0, 1.0, V3::new(0.0, 0.0, -10.0));
        assert!((running.z - (v1.z - 3.0)).abs() < 1e-3);
    }

    #[test]
    fn power_throw_needs_90_percent() {
        assert!(is_power(0.9) && is_power(1.0));
        assert!(!is_power(0.89));
    }

    #[test]
    fn two_slots_swap_and_select() {
        let mut s = Slots::default();
        assert!(s.add(10));
        assert!(s.add(11));
        assert!(!s.add(12), "hands are full");
        assert_eq!(s.selected(), Some(11)); // newest is selected
        assert!(s.swap(1));
        assert_eq!(s.selected(), Some(10));
        assert!(s.swap(-1));
        assert_eq!(s.selected(), Some(11));
        assert!(s.select_slot(0));
        assert_eq!(s.selected(), Some(10));
        s.remove(10);
        assert_eq!(s.selected(), Some(11));
        assert!(!s.swap(1), "nothing to swap with one item");
        assert_eq!(s.oldest(), Some(11));
    }

    #[test]
    fn pickup_rules() {
        let it = ground_item(ItemKind::Teddy, 0.5, 0.0);
        assert!(can_pick_up(&picker(), &it, 0.0));
        // too far
        assert!(!can_pick_up(
            &picker(),
            &ground_item(ItemKind::Teddy, 1.2, 0.0),
            0.0
        ));
        // too high
        let mut high = ground_item(ItemKind::Teddy, 0.5, 0.0);
        high.pos.y = 2.0;
        assert!(!can_pick_up(&picker(), &high, 0.0));
        // not rested yet
        let mut fresh = ground_item(ItemKind::Teddy, 0.5, 0.0);
        fresh.rest_t = 0.1;
        assert!(!can_pick_up(&picker(), &fresh, 0.0));
        // hands full / stunned / frozen
        assert!(!can_pick_up(
            &Picker {
                held: 2,
                ..picker()
            },
            &it,
            0.0
        ));
        assert!(!can_pick_up(
            &Picker {
                stunned: true,
                ..picker()
            },
            &it,
            0.0
        ));
        assert!(!can_pick_up(
            &Picker {
                frozen: true,
                ..picker()
            },
            &it,
            0.0
        ));
        // bots skip melee items
        let steak = ground_item(ItemKind::Steak, 0.5, 0.0);
        assert!(can_pick_up(&picker(), &steak, 0.0));
        assert!(!can_pick_up(
            &Picker {
                is_bot: true,
                ..picker()
            },
            &steak,
            0.0
        ));
        // flying items can't be picked up
        let mut flying = ground_item(ItemKind::Teddy, 0.5, 0.0);
        flying.state = ItemState::Flying;
        assert!(!can_pick_up(&picker(), &flying, 0.0));
    }

    #[test]
    fn you_cant_grab_back_what_you_were_forced_to_drop_for_2_5_s() {
        let mut it = ground_item(ItemKind::Teddy, 0.5, 0.0);
        it.block = Some((1, 2.5));
        assert!(!can_pick_up(&picker(), &it, 1.0));
        assert!(can_pick_up(&picker(), &it, 2.6));
        assert!(
            can_pick_up(&Picker { id: 2, ..picker() }, &it, 1.0),
            "others can"
        );
    }
}
