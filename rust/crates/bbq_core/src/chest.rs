//! The chest (Cheeky mode, spec section 8): a stock of up to 3 dildos that refills slowly and
//! moves to a new spot every round.

use crate::items::{DildoVariant, pick_dildo_variant};
use crate::rng::Rng;
use crate::yard::CHEST_SPOTS;

pub const MAX_STOCK: u32 = 3;
pub const START_STOCK: u32 = 2;
/// Seconds to refill one.
pub const REFILL: f32 = 12.0;
/// How close you must be to press R (your own computer's check).
pub const REACH: f32 = 2.0;
/// How close the host allows.
pub const HOST_REACH: f32 = 3.0;
/// You can't use it while higher than this.
pub const USE_HEIGHT: f32 = 0.5;

#[derive(Clone, Debug)]
pub struct Chest {
    pub stock: u32,
    timer: f32,
    pub spot: usize,
}

impl Default for Chest {
    fn default() -> Self {
        Self::new()
    }
}

impl Chest {
    pub fn new() -> Self {
        Chest {
            stock: START_STOCK,
            timer: REFILL,
            spot: 0,
        }
    }

    pub fn position(&self) -> (f32, f32) {
        let (x, z, _) = CHEST_SPOTS[self.spot % CHEST_SPOTS.len()];
        (x, z)
    }

    /// Start of a round: 2 in stock, timer reset, and a different spot.
    pub fn new_round(&mut self, rng: &mut Rng) {
        self.stock = START_STOCK;
        self.timer = REFILL;
        let n = CHEST_SPOTS.len();
        // any spot but the one it's on now
        let next = (self.spot + 1 + rng.index(n - 1)) % n;
        self.spot = next;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.stock < MAX_STOCK {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.stock += 1;
                self.timer = REFILL;
            }
        }
    }

    /// Seconds until the next dildo appears (for the "restocking" hint).
    pub fn restock_in(&self) -> Option<f32> {
        (self.stock < MAX_STOCK).then_some(self.timer.max(0.0))
    }

    /// Is `(x, z, y)` close enough to use it?
    pub fn near(&self, x: f32, z: f32, y: f32, reach: f32) -> bool {
        let (cx, cz) = self.position();
        y < USE_HEIGHT && (x - cx).hypot(z - cz) < reach
    }

    /// Take one. Returns the size you got, or `None` if it's empty.
    pub fn take(&mut self, rng: &mut Rng) -> Option<DildoVariant> {
        if self.stock == 0 {
            return None;
        }
        self.stock -= 1;
        if self.stock < MAX_STOCK && self.timer <= 0.0 {
            self.timer = REFILL;
        }
        Some(pick_dildo_variant(rng.f32()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_two_and_refills_to_three_in_12_seconds() {
        let mut c = Chest::new();
        assert_eq!(c.stock, 2);
        for _ in 0..(11 * 60) {
            c.tick(1.0 / 60.0);
        }
        assert_eq!(c.stock, 2);
        for _ in 0..90 {
            c.tick(1.0 / 60.0);
        }
        assert_eq!(c.stock, 3);
        // full: no more, and the timer is paused
        for _ in 0..(30 * 60) {
            c.tick(1.0 / 60.0);
        }
        assert_eq!(c.stock, 3);
        assert_eq!(c.restock_in(), None);
    }

    #[test]
    fn taking_empties_it_and_then_says_no() {
        let mut c = Chest::new();
        let mut rng = Rng::new(1);
        assert!(c.take(&mut rng).is_some());
        assert!(c.take(&mut rng).is_some());
        assert_eq!(c.stock, 0);
        assert!(c.take(&mut rng).is_none());
        assert!(c.restock_in().is_some());
    }

    #[test]
    fn the_chest_moves_each_round() {
        let mut c = Chest::new();
        let mut rng = Rng::new(7);
        for _ in 0..50 {
            let before = c.spot;
            c.new_round(&mut rng);
            assert_ne!(c.spot, before);
            assert!(c.spot < CHEST_SPOTS.len());
            assert_eq!(c.stock, START_STOCK);
        }
    }

    #[test]
    fn it_has_to_be_close_and_low() {
        let c = Chest::new();
        let (x, z) = c.position();
        assert!(c.near(x + 1.0, z, 0.0, REACH));
        assert!(!c.near(x + 2.5, z, 0.0, REACH));
        assert!(c.near(x + 2.5, z, 0.0, HOST_REACH));
        assert!(!c.near(x, z, 1.0, REACH));
    }

    #[test]
    fn variants_follow_the_weights_roughly() {
        let mut rng = Rng::new(3);
        let mut gold = 0;
        for _ in 0..4000 {
            let mut c = Chest::new();
            if c.take(&mut rng) == Some(DildoVariant::Gold) {
                gold += 1;
            }
        }
        // weight 4 out of 96
        assert!((80..260).contains(&gold), "gold {gold}");
    }
}
