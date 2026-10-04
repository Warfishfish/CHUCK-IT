//! Rounds and best-of-N matches (spec section 10).

use std::collections::BTreeMap;

use crate::PlayerId;
use crate::heist::Bank;
use crate::teams::{Team, Teams};

/// Who can win a round: a whole team, or one player.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Team(Team),
    Player(PlayerId),
}

/// Free for all: the top score wins; a tie for first is a draw.
pub fn round_winner_ffa(rows: &[(PlayerId, i32)]) -> Option<Key> {
    let mut r = rows.to_vec();
    r.sort_by_key(|x| std::cmp::Reverse(x.1));
    match r.as_slice() {
        [] => None,
        [(id, _)] => Some(Key::Player(*id)),
        [(id, a), (_, b), ..] => (a != b).then_some(Key::Player(*id)),
    }
}

/// Teams: Red and Blue totals decide it, unless the Wildcard beats both on their own.
pub fn round_winner_teams(rows: &[(PlayerId, i32)], teams: &Teams) -> Option<Key> {
    let (mut red, mut blue) = (0, 0);
    let mut wild: Option<(PlayerId, i32)> = None;
    for &(id, score) in rows {
        match teams.get(id) {
            Some(Team::Wildcard) => wild = Some((id, score)),
            Some(Team::Red) => red += score,
            Some(Team::Blue) => blue += score,
            _ => {}
        }
    }
    if let Some((id, score)) = wild
        && score > red.max(blue)
    {
        return Some(Key::Player(id));
    }
    if red == blue {
        None
    } else if red > blue {
        Some(Key::Team(Team::Red))
    } else {
        Some(Key::Team(Team::Blue))
    }
}

/// Heist: the most teddies banked.
pub fn round_winner_heist(bank: &Bank) -> Option<Key> {
    bank.winner().map(Key::Team)
}

/// A best-of-N match.
#[derive(Clone, Debug)]
pub struct Match {
    /// Rounds in the match: 1, 3 or 5.
    pub len: u32,
    /// The round number now (or last) being played; 0 before the first.
    pub no: u32,
    pub wins: BTreeMap<Key, u32>,
    pub over: bool,
}

impl Match {
    pub fn new(len: u32) -> Self {
        Match {
            len: len.max(1),
            no: 0,
            wins: BTreeMap::new(),
            over: true,
        }
    }

    /// Round wins needed to take the match.
    pub fn need(&self) -> u32 {
        self.len / 2 + 1
    }

    /// Start the next round (and a fresh match if the last one finished).
    pub fn start_round(&mut self) {
        if self.over || self.no == 0 {
            self.no = 0;
            self.wins.clear();
            self.over = false;
        }
        self.no += 1;
    }

    /// Record the winner of the round (`None` for a draw). The match ends when someone has
    /// enough wins or the rounds run out.
    pub fn end_round(&mut self, winner: Option<Key>) {
        if let Some(w) = winner {
            *self.wins.entry(w).or_insert(0) += 1;
        }
        let top = self.wins.values().copied().max().unwrap_or(0);
        if top >= self.need() || self.no >= self.len {
            self.over = true;
        }
    }

    /// Who is ahead on round wins. Ties have no leader.
    pub fn leader(&self) -> Option<Key> {
        let mut best = 0;
        let mut who = None;
        let mut tie = false;
        for (k, v) in &self.wins {
            if *v > best {
                best = *v;
                who = Some(*k);
                tie = false;
            } else if *v == best {
                tie = true;
            }
        }
        if tie { None } else { who }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ffa_winner_and_ties() {
        assert_eq!(round_winner_ffa(&[(1, 100), (2, 50)]), Some(Key::Player(1)));
        assert_eq!(round_winner_ffa(&[(1, 50), (2, 50), (3, 10)]), None);
        assert_eq!(round_winner_ffa(&[(1, 0), (2, 0)]), None);
        assert_eq!(round_winner_ffa(&[(7, 3)]), Some(Key::Player(7)));
        assert_eq!(round_winner_ffa(&[]), None);
    }

    #[test]
    fn team_winner_sums_each_side() {
        let mut t = Teams::default();
        t.set(1, Team::Red);
        t.set(2, Team::Red);
        t.set(3, Team::Blue);
        t.set(4, Team::Blue);
        let rows = [(1, 100), (2, 100), (3, 150), (4, 20)];
        assert_eq!(round_winner_teams(&rows, &t), Some(Key::Team(Team::Red)));
        let tied = [(1, 100), (2, 100), (3, 150), (4, 50)];
        assert_eq!(round_winner_teams(&tied, &t), None);
    }

    #[test]
    fn the_wildcard_can_beat_both_teams() {
        let mut t = Teams::default();
        t.set(1, Team::Red);
        t.set(2, Team::Blue);
        t.set(3, Team::Wildcard);
        assert_eq!(
            round_winner_teams(&[(1, 100), (2, 80), (3, 101)], &t),
            Some(Key::Player(3))
        );
        assert_eq!(
            round_winner_teams(&[(1, 100), (2, 80), (3, 100)], &t),
            Some(Key::Team(Team::Red))
        );
    }

    #[test]
    fn heist_winner_is_the_team_with_most_banked() {
        let mut b = Bank::new(&[Team::Red, Team::Blue]);
        assert_eq!(round_winner_heist(&b), None);
        b.add(Team::Blue);
        assert_eq!(round_winner_heist(&b), Some(Key::Team(Team::Blue)));
    }

    #[test]
    fn best_of_three_ends_at_two_wins() {
        let mut m = Match::new(3);
        assert_eq!(m.need(), 2);
        m.start_round();
        assert_eq!(m.no, 1);
        m.end_round(Some(Key::Player(1)));
        assert!(!m.over);
        m.start_round();
        m.end_round(Some(Key::Player(1)));
        assert!(m.over, "two wins out of three takes the match");
        assert_eq!(m.leader(), Some(Key::Player(1)));
    }

    #[test]
    fn a_match_also_ends_when_the_rounds_run_out() {
        let mut m = Match::new(3);
        for w in [Some(Key::Player(1)), Some(Key::Player(2)), None] {
            m.start_round();
            m.end_round(w);
        }
        assert!(m.over);
        assert_eq!(m.leader(), None, "1-1 has no leader");
    }

    #[test]
    fn a_single_round_match_is_over_after_one_round() {
        let mut m = Match::new(1);
        m.start_round();
        m.end_round(None);
        assert!(m.over);
    }

    #[test]
    fn starting_after_a_finished_match_starts_a_new_one() {
        let mut m = Match::new(3);
        m.start_round();
        m.end_round(Some(Key::Player(1)));
        m.start_round();
        m.end_round(Some(Key::Player(1)));
        assert!(m.over);
        m.start_round();
        assert_eq!(m.no, 1);
        assert!(m.wins.is_empty());
        assert!(!m.over);
    }
}
