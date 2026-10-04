//! Teddy Heist rules (spec section 13). Only banking a teddy scores.

use std::collections::BTreeMap;

use crate::teams::Team;

/// Half the width of a base (bases are 6 x 6 m).
pub const BASE_HALF: f32 = 3.0;
pub const WALL_THICKNESS: f32 = 0.35;
pub const WALL_HEIGHT: f32 = 2.1;
/// Banking zone radius around a base centre.
pub const BASE_RADIUS: f32 = 4.3;
/// A teddy lying this close to its own base counts as "already home".
pub const HOME_RADIUS: f32 = 2.6;
/// Points per banked teddy (see `scoring::HEIST_BANK`).
pub const BANK_POINTS: i32 = crate::scoring::HEIST_BANK;
/// Teddies sit at these offsets from the base centre.
pub const TEDDY_OFFSETS: [(f32, f32); 4] = [(-1.3, -1.3), (1.3, -1.3), (-1.3, 1.3), (1.3, 1.3)];
/// Players spawn inside their own base at these offsets, in turn.
pub const SPAWN_OFFSETS: [(f32, f32); 4] = [(0.0, 2.1), (0.0, -2.1), (2.1, 0.0), (-2.1, 0.0)];
/// The oval arena: half-axes and the bay in the middle of each side.
pub const ARENA_A: f32 = 40.0;
pub const ARENA_B: f32 = 29.0;

pub const TEAM_KEYS: [Team; 4] = [Team::Red, Team::Blue, Team::Green, Team::Yellow];

/// The first `n` (2 to 4) Heist teams.
pub fn team_keys(n: usize) -> &'static [Team] {
    &TEAM_KEYS[..n.clamp(2, 4)]
}

/// Which side of a base has the doorway.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Open {
    XPos,
    XNeg,
    ZPos,
    ZNeg,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BaseDef {
    pub x: f32,
    pub z: f32,
    pub open: Open,
}

/// Base positions for 2, 3 or 4 teams, in the order of `team_keys`.
pub fn layout(teams: usize) -> Vec<BaseDef> {
    use Open::*;
    let b = |x, z, open| BaseDef { x, z, open };
    match teams.clamp(2, 4) {
        2 => vec![b(-20.0, 0.0, XPos), b(20.0, 0.0, XNeg)],
        3 => vec![
            b(-21.0, -10.0, XPos),
            b(21.0, -10.0, XNeg),
            b(0.0, 16.0, ZNeg),
        ],
        _ => vec![
            b(-21.0, -10.0, XPos),
            b(21.0, -10.0, XNeg),
            b(-20.0, 18.0, XPos),
            b(21.0, 18.0, XNeg),
        ],
    }
}

/// Teddies in each base: 2 to start, plus 1 for every couple of players sharing a team, max 4.
pub fn teddy_count(players: usize, teams: usize) -> usize {
    (2 + (players / teams.max(1)) / 2).clamp(2, 4)
}

/// What happens when a player walks over a teddy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Touch {
    /// An enemy teddy: pick it up.
    Steal,
    /// Your own teddy, away from home: it jumps back to its base.
    SendHome,
    /// Your own teddy, already at home: nothing happens.
    Nothing,
}

pub fn touch_teddy(player: Team, teddy: Team, teddy_dist_to_home: f32) -> Touch {
    if player != teddy {
        Touch::Steal
    } else if teddy_dist_to_home < HOME_RADIUS {
        Touch::Nothing
    } else {
        Touch::SendHome
    }
}

/// Can this player bank this teddy right now? They must carry an enemy teddy into their own
/// base zone, and not be stunned or down.
pub fn can_bank(
    player: Team,
    teddy: Team,
    dist_to_own_base: f32,
    stunned: bool,
    down: bool,
) -> bool {
    player != teddy && dist_to_own_base < BASE_RADIUS && !stunned && !down
}

/// Teddies banked per team.
#[derive(Clone, Debug, Default)]
pub struct Bank {
    banked: BTreeMap<Team, u32>,
}

impl Bank {
    pub fn new(teams: &[Team]) -> Self {
        Bank {
            banked: teams.iter().map(|t| (*t, 0)).collect(),
        }
    }

    pub fn add(&mut self, team: Team) -> u32 {
        let c = self.banked.entry(team).or_insert(0);
        *c += 1;
        *c
    }

    pub fn get(&self, team: Team) -> u32 {
        self.banked.get(&team).copied().unwrap_or(0)
    }

    pub fn points(&self, team: Team) -> i32 {
        self.get(team) as i32 * BANK_POINTS
    }

    /// The team with the most teddies. A tie, or nobody banking anything, is a draw (`None`).
    pub fn winner(&self) -> Option<Team> {
        let mut best = 0;
        let mut who = None;
        let mut tie = false;
        for (t, v) in &self.banked {
            if *v > best {
                best = *v;
                who = Some(*t);
                tie = false;
            } else if *v == best {
                tie = true;
            }
        }
        if best == 0 || tie { None } else { who }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn teddy_counts() {
        assert_eq!(teddy_count(2, 2), 2);
        assert_eq!(teddy_count(4, 2), 3); // 2 per team: 2 + 1
        assert_eq!(teddy_count(8, 2), 4); // 4 per team: 2 + 2
        assert_eq!(teddy_count(16, 2), 4); // capped
        assert_eq!(teddy_count(3, 3), 2);
    }

    #[test]
    fn layouts_have_one_base_per_team() {
        for n in 2..=4 {
            assert_eq!(layout(n).len(), n);
            assert_eq!(team_keys(n).len(), n);
        }
        assert_eq!(
            layout(2)[0],
            BaseDef {
                x: -20.0,
                z: 0.0,
                open: Open::XPos
            }
        );
        assert_eq!(
            layout(3)[2],
            BaseDef {
                x: 0.0,
                z: 16.0,
                open: Open::ZNeg
            }
        );
        assert_eq!(
            layout(4)[3],
            BaseDef {
                x: 21.0,
                z: 18.0,
                open: Open::XNeg
            }
        );
        // every base sits inside the oval arena with room to spare
        for n in 2..=4 {
            for b in layout(n) {
                let e = (b.x / ARENA_A).powi(2) + (b.z / ARENA_B).powi(2);
                assert!(e < 1.0, "base at ({}, {}) is outside the oval", b.x, b.z);
            }
        }
    }

    #[test]
    fn touching_teddies() {
        assert_eq!(touch_teddy(Team::Red, Team::Blue, 0.5), Touch::Steal);
        assert_eq!(touch_teddy(Team::Red, Team::Red, 1.0), Touch::Nothing);
        assert_eq!(touch_teddy(Team::Red, Team::Red, 9.0), Touch::SendHome);
    }

    #[test]
    fn banking_rules() {
        assert!(can_bank(Team::Red, Team::Blue, 4.0, false, false));
        assert!(
            !can_bank(Team::Red, Team::Blue, 4.3, false, false),
            "must be inside the zone"
        );
        assert!(
            !can_bank(Team::Red, Team::Red, 1.0, false, false),
            "can't bank your own teddy"
        );
        assert!(
            !can_bank(Team::Red, Team::Blue, 1.0, true, false),
            "not while stunned"
        );
        assert!(
            !can_bank(Team::Red, Team::Blue, 1.0, false, true),
            "not while down"
        );
    }

    #[test]
    fn winner_is_most_banked_and_ties_are_draws() {
        let mut b = Bank::new(&[Team::Red, Team::Blue]);
        assert_eq!(b.winner(), None, "0-0 is a draw");
        b.add(Team::Red);
        assert_eq!(b.winner(), Some(Team::Red));
        b.add(Team::Blue);
        assert_eq!(b.winner(), None);
        b.add(Team::Blue);
        assert_eq!(b.winner(), Some(Team::Blue));
        assert_eq!(b.points(Team::Blue), 300);
    }
}
