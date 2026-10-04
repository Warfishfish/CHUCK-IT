//! The mystery chest (Cheeky mode only, spec section 8): holds up to 3 dildos, refills slowly,
//! and each one you take is a random size.

use crate::items::{DildoVariant, pick_dildo_variant};
use crate::rng::Rng;

pub const MAX_STOCK: u32 = 3;
pub const START_STOCK: u32 = 2;
/// A new one every 12 s, only while it holds fewer than 3.
pub const REFILL_EVERY: f32 = 12.0;
/// You can open it within this distance (the host allows 3 m).
pub const REACH: f32 = 2.0;
pub const HOST_REACH: f32 = 3.0;
/// You need free hands: fewer than this many things held.
pub const HANDS_NEEDED: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refused {
    Empty,
    HandsFull,
    TooFar,
}

#[derive(Clone, Copy, Debug)]
pub struct Chest {
    pub stock: u32,
    refill_t: f32,
    /// Which of the 9 spots it is at this round.
    pub spot: usize,
}

impl Default for Chest {
    fn default() -> Self {
        Chest {
            stock: START_STOCK,
            refill_t: 0.0,
            spot: 0,
        }
    }
}

impl Chest {
    /// New round: back to 2, somewhere new.
    pub fn new_round(&mut self, spot: usize) {
        self.stock = START_STOCK;
        self.refill_t = 0.0;
        self.spot = spot;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.stock >= MAX_STOCK {
            self.refill_t = 0.0;
            return;
        }
        self.refill_t += dt;
        if self.refill_t >= REFILL_EVERY {
            self.refill_t = 0.0;
            self.stock += 1;
        }
    }

    /// Take one: a random size of dildo.
    pub fn take(
        &mut self,
        distance: f32,
        held: usize,
        rng: &mut Rng,
    ) -> Result<DildoVariant, Refused> {
        if distance > REACH {
            return Err(Refused::TooFar);
        }
        if held >= HANDS_NEEDED {
            return Err(Refused::HandsFull);
        }
        if self.stock == 0 {
            return Err(Refused::Empty);
        }
        self.stock -= 1;
        Ok(pick_dildo_variant(rng.f32()))
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_two_and_refills_to_three_then_stops() {
        let mut c = Chest::default();
        assert_eq!(c.stock, 2);
        for _ in 0..(11.9 * 60.0) as usize {
            c.tick(1.0 / 60.0);
        }
        assert_eq!(c.stock, 2);
        for _ in 0..60 {
            c.tick(1.0 / 60.0);
        }
        assert_eq!(c.stock, 3);
        for _ in 0..(60.0 * 60.0) as usize {
            c.tick(1.0 / 60.0);
        }
        assert_eq!(c.stock, 3, "never over 3");
    }

    #[test]
    fn taking_needs_to_be_close_with_a_free_hand_and_stock() {
        let mut rng = Rng::new(1);
        let mut c = Chest::default();
        assert_eq!(c.take(2.5, 0, &mut rng), Err(Refused::TooFar));
        assert_eq!(c.take(1.0, 2, &mut rng), Err(Refused::HandsFull));
        assert!(c.take(1.0, 1, &mut rng).is_ok());
        assert!(c.take(1.0, 0, &mut rng).is_ok());
        assert_eq!(c.stock, 0);
        assert_eq!(c.take(1.0, 0, &mut rng), Err(Refused::Empty));
    }

    #[test]
    fn sizes_follow_the_46_28_18_4_weights() {
        let mut rng = Rng::new(9);
        let mut counts = [0usize; 4];
        let n = 20000;
        for _ in 0..n {
            let mut c = Chest::default();
            let v = c.take(0.5, 0, &mut rng).unwrap();
            let i = DildoVariant::ALL.iter().position(|x| *x == v).unwrap();
            counts[i] += 1;
        }
        let want = [0.479, 0.292, 0.1875, 0.0417];
        for i in 0..4 {
            let got = counts[i] as f32 / n as f32;
            assert!((got - want[i]).abs() < 0.015, "{i}: {got} vs {}", want[i]);
        }
    }

    #[test]
    fn a_new_round_restocks_and_moves_it() {
        let mut c = Chest::default();
        c.stock = 0;
        c.new_round(5);
        assert_eq!((c.stock, c.spot), (2, 5));
    }
}
