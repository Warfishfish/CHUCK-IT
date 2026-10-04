//! Australian BBQ game rules: pure Rust, no engine, no graphics, no clock.
//!
//! Every number here comes from `BEHAVIOUR_SPEC.md` (itself taken from the
//! JavaScript game, version 0.23.1, with the Heist scoring fix from 0.23.2).
//! The section numbers in the comments point at that file.

pub mod carry;
pub mod drinks;
pub mod flight;
pub mod hands;
pub mod heist;
pub mod hitting;
pub mod items;
pub mod itemworld;
pub mod matchflow;
pub mod movement;
pub mod rng;
pub mod scoring;
pub mod sim;
pub mod stun;
pub mod teams;
pub mod vec;
pub mod yard;

/// Who someone is. Players and bots share one id space.
pub type PlayerId = u32;

/// Yard half-sizes in metres (spec section 1).
pub const YARD_HALF_X: f32 = 33.0;
pub const YARD_HALF_Z: f32 = 24.0;

/// The game mode picked in the menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    FreeForAll,
    Teams,
    Heist,
}
