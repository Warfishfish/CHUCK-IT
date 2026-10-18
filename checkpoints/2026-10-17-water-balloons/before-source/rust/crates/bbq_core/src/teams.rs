//! Teams and who is on them (spec section 10).

use std::collections::BTreeMap;

use crate::PlayerId;
use crate::rng::Rng;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Team {
    Red,
    Blue,
    Green,
    Yellow,
    /// The odd one out in Teams: plays for themselves against everyone.
    Wildcard,
}

impl Team {
    pub fn name(self) -> &'static str {
        match self {
            Team::Red => "Red",
            Team::Blue => "Blue",
            Team::Green => "Green",
            Team::Yellow => "Yellow",
            Team::Wildcard => "Wildcard",
        }
    }
}

/// Which team each player is on.
#[derive(Clone, Debug, Default)]
pub struct Teams {
    map: BTreeMap<PlayerId, Team>,
}

impl Teams {
    pub fn get(&self, id: PlayerId) -> Option<Team> {
        self.map.get(&id).copied()
    }

    pub fn set(&mut self, id: PlayerId, team: Team) {
        self.map.insert(id, team);
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = (PlayerId, Team)> + '_ {
        self.map.iter().map(|(k, v)| (*k, *v))
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn count(&self, team: Team) -> usize {
        self.map.values().filter(|t| **t == team).count()
    }

    pub fn members(&self, team: Team) -> Vec<PlayerId> {
        self.map
            .iter()
            .filter(|(_, t)| **t == team)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Two players are teammates only if they share a real team. The Wildcard is nobody's teammate.
    pub fn same_team(&self, a: PlayerId, b: PlayerId) -> bool {
        if a == b {
            return false;
        }
        match (self.get(a), self.get(b)) {
            (Some(x), Some(y)) => x != Team::Wildcard && x == y,
            _ => false,
        }
    }

    /// Teams mode: even numbers split Red/Blue; an odd one out becomes the Wildcard. New players
    /// join whichever of Red/Blue has fewer. `reshuffle` wipes and redeals; `assign_only` only
    /// places people who have no team yet (used mid-round and after manual changes).
    pub fn balance(&mut self, ids: &[PlayerId], reshuffle: bool, assign_only: bool, rng: &mut Rng) {
        self.map.retain(|k, _| ids.contains(k));
        if reshuffle {
            self.map.clear();
        }
        let mut order: Vec<PlayerId> = ids
            .iter()
            .copied()
            .filter(|id| !self.map.contains_key(id))
            .collect();
        if reshuffle {
            rng.shuffle(&mut order);
        }
        if !assign_only {
            let odd = ids.len() % 2 == 1;
            if !odd {
                let wild: Vec<PlayerId> = self.members(Team::Wildcard);
                for k in wild {
                    self.map.remove(&k);
                    order.push(k);
                }
            }
            if odd && self.count(Team::Wildcard) == 0 {
                if !order.is_empty() {
                    let i = rng.index(order.len());
                    let id = order.remove(i);
                    self.map.insert(id, Team::Wildcard);
                } else {
                    let big = if self.count(Team::Red) >= self.count(Team::Blue) {
                        Team::Red
                    } else {
                        Team::Blue
                    };
                    if let Some(id) = self.members(big).last().copied() {
                        self.map.insert(id, Team::Wildcard);
                    }
                }
            }
        }
        for id in order {
            let t = if self.count(Team::Red) <= self.count(Team::Blue) {
                Team::Red
            } else {
                Team::Blue
            };
            self.map.insert(id, t);
        }
    }

    /// Heist: spread people over `keys` (the first 2 to 4 of Red, Blue, Green, Yellow), each new
    /// person going to the team with the fewest.
    pub fn assign_heist(
        &mut self,
        ids: &[PlayerId],
        keys: &[Team],
        reshuffle: bool,
        rng: &mut Rng,
    ) {
        if reshuffle || self.map.values().any(|t| !keys.contains(t)) {
            self.map.clear();
        }
        let mut order: Vec<PlayerId> = ids
            .iter()
            .copied()
            .filter(|id| !self.map.contains_key(id))
            .collect();
        if reshuffle {
            rng.shuffle(&mut order);
        }
        for id in order {
            let mut best = keys[0];
            let mut bv = self.count(best);
            for &k in keys {
                let v = self.count(k);
                if v < bv {
                    bv = v;
                    best = k;
                }
            }
            self.map.insert(id, best);
        }
    }

    /// Click-to-move in the lobby. Teams: Red, Blue, Wildcard, Red... Heist: next team along.
    pub fn cycle(&mut self, id: PlayerId, heist_keys: Option<&[Team]>) {
        match heist_keys {
            Some(keys) => {
                let idx = self
                    .get(id)
                    .and_then(|t| keys.iter().position(|k| *k == t))
                    .map(|i| i as isize)
                    .unwrap_or(-1);
                let n = keys.len() as isize;
                self.map.insert(id, keys[((idx + 1 + n) % n) as usize]);
            }
            None => {
                let next = match self.get(id).unwrap_or(Team::Red) {
                    Team::Red => Team::Blue,
                    Team::Blue => Team::Wildcard,
                    _ => Team::Red,
                };
                self.map.insert(id, next);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: u32) -> Vec<PlayerId> {
        (1..=n).collect()
    }

    #[test]
    fn even_numbers_split_red_and_blue() {
        let mut t = Teams::default();
        t.balance(&ids(4), true, false, &mut Rng::new(1));
        assert_eq!(t.count(Team::Red), 2);
        assert_eq!(t.count(Team::Blue), 2);
        assert_eq!(t.count(Team::Wildcard), 0);
    }

    #[test]
    fn an_odd_player_becomes_the_wildcard() {
        let mut t = Teams::default();
        t.balance(&ids(5), true, false, &mut Rng::new(2));
        assert_eq!(t.count(Team::Wildcard), 1);
        assert_eq!(t.count(Team::Red), 2);
        assert_eq!(t.count(Team::Blue), 2);
    }

    #[test]
    fn the_wildcard_goes_back_on_a_team_when_the_numbers_even_out() {
        let mut t = Teams::default();
        t.balance(&ids(5), true, false, &mut Rng::new(2));
        t.balance(&ids(6), false, false, &mut Rng::new(2));
        assert_eq!(t.count(Team::Wildcard), 0);
        assert_eq!(t.count(Team::Red), 3);
        assert_eq!(t.count(Team::Blue), 3);
    }

    #[test]
    fn leavers_are_removed_and_joiners_go_to_the_smaller_team() {
        let mut t = Teams::default();
        t.balance(&ids(4), true, false, &mut Rng::new(3));
        t.balance(&[1, 2, 3, 4, 5, 6, 7, 8], false, true, &mut Rng::new(3));
        assert_eq!(t.count(Team::Red), 4);
        assert_eq!(t.count(Team::Blue), 4);
        t.balance(&[1, 2], false, true, &mut Rng::new(3));
        assert_eq!(t.len(), 2);
    }

    #[test]
    fn teammates_exclude_the_wildcard_and_yourself() {
        let mut t = Teams::default();
        t.set(1, Team::Red);
        t.set(2, Team::Red);
        t.set(3, Team::Blue);
        t.set(4, Team::Wildcard);
        t.set(5, Team::Wildcard);
        assert!(t.same_team(1, 2));
        assert!(!t.same_team(1, 3));
        assert!(!t.same_team(1, 1));
        assert!(!t.same_team(4, 5), "wildcards are not on a team together");
        assert!(!t.same_team(1, 99));
    }

    #[test]
    fn heist_teams_spread_evenly() {
        let mut t = Teams::default();
        t.assign_heist(
            &ids(6),
            &[Team::Red, Team::Blue, Team::Green],
            true,
            &mut Rng::new(9),
        );
        assert_eq!(t.count(Team::Red), 2);
        assert_eq!(t.count(Team::Blue), 2);
        assert_eq!(t.count(Team::Green), 2);
        // a seventh joins the first team with the fewest
        t.assign_heist(
            &ids(7),
            &[Team::Red, Team::Blue, Team::Green],
            false,
            &mut Rng::new(9),
        );
        assert_eq!(t.len(), 7);
        assert_eq!(
            t.count(Team::Red) + t.count(Team::Blue) + t.count(Team::Green),
            7
        );
    }

    #[test]
    fn heist_wipes_teams_that_are_not_in_play() {
        let mut t = Teams::default();
        t.set(1, Team::Wildcard);
        t.set(2, Team::Green);
        t.assign_heist(&ids(2), &[Team::Red, Team::Blue], false, &mut Rng::new(1));
        assert_eq!(t.count(Team::Wildcard) + t.count(Team::Green), 0);
        assert_eq!(t.count(Team::Red), 1);
        assert_eq!(t.count(Team::Blue), 1);
    }

    #[test]
    fn click_to_move() {
        let mut t = Teams::default();
        t.set(1, Team::Red);
        t.cycle(1, None);
        assert_eq!(t.get(1), Some(Team::Blue));
        t.cycle(1, None);
        assert_eq!(t.get(1), Some(Team::Wildcard));
        t.cycle(1, None);
        assert_eq!(t.get(1), Some(Team::Red));
        let keys = [Team::Red, Team::Blue, Team::Green];
        t.cycle(1, Some(&keys));
        assert_eq!(t.get(1), Some(Team::Blue));
        t.cycle(1, Some(&keys));
        t.cycle(1, Some(&keys));
        assert_eq!(t.get(1), Some(Team::Red));
    }
}
