//! Scores, streaks and who is winning (spec sections 3 and 9).
//!
//! **Heist rule (decided by Marcus, 4 Oct 2026):** in Heist only banking a teddy scores.
//! Thrown hits, slaps, catches and every bonus pay nothing and cost nothing.

use std::collections::BTreeMap;

use crate::items::{CRIT_POINTS, DildoVariant};
use crate::teams::{Team, Teams};
use crate::{GameMode, PlayerId};

/// Where the game is in a round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Menu,
    /// Online free play before a round: nothing scores.
    Warmup,
    Countdown,
    Play,
    Results,
}

/// The things scoring depends on.
#[derive(Clone, Copy, Debug)]
pub struct Rules {
    pub mode: GameMode,
    pub friendly_fire: bool,
    pub phase: Phase,
}

impl Rules {
    /// Points count at all (round in play).
    pub fn in_play(&self) -> bool {
        self.phase == Phase::Play
    }
    /// Normal scoring: in play, and not Heist.
    pub fn scoring(&self) -> bool {
        self.in_play() && self.mode != GameMode::Heist
    }
}

// --- point values ---
pub const HIT: i32 = 100;
pub const BOUNTY: i32 = 50;
pub const VICTIM_PENALTY: i32 = 50;
pub const CATCH: i32 = 50;
pub const SILLY: i32 = 40;
pub const BUM_OUT: i32 = 15;
pub const CANNONBALL: i32 = 100;
pub const POOL_OR_TRAMP: i32 = 50;
pub const NAUGHTY_CORNER: i32 = 100;
pub const HELP_UP: i32 = 25;
pub const HEIST_BANK: i32 = 150;
pub const DAZZA_STUN: i32 = 20;
/// KO, berserk, flip the barbie.
pub const DAZZA_SLAP: [i32; 3] = [75, 25, 50];
pub const LONG_MIN: f32 = 8.0;
pub const LONG_MAX: f32 = 25.0;
pub const LONG_PTS: f32 = 100.0;

/// x1.5 at 3 hits in a row, x2 at 5.
pub fn streak_mult(streak: u32) -> f32 {
    if streak >= 5 {
        2.0
    } else if streak >= 3 {
        1.5
    } else {
        1.0
    }
}

/// Bonus for a long throw: scales with how far it went (8 to 25 m) and how hard it was
/// wound up, in steps of 5, up to 100.
pub fn long_shot(charge: f32, dist: f32) -> i32 {
    let f = ((dist - LONG_MIN) / (LONG_MAX - LONG_MIN)).clamp(0.0, 1.0);
    ((charge * f * LONG_PTS / 5.0).round() as i32) * 5
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerStats {
    pub score: i32,
    pub hits: u32,
    pub taken: u32,
    pub catches: u32,
    pub throws: u32,
    pub streak: u32,
    /// Teddies this player has banked (Heist).
    pub banked: u32,
}

/// What a thrown hit was worth.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HitInput {
    /// Wind-up charge of the throw, 0..1.
    pub charge: f32,
    /// Distance from where the throw started to the victim.
    pub dist: f32,
    pub victim_is_leader: bool,
    pub drunk_bonus: i32,
    pub bum_out: bool,
    pub same_team: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HitOutcome {
    /// Points the thrower got.
    pub gain: i32,
    /// Thrower's streak after this hit.
    pub streak: u32,
    pub long_shot: i32,
    /// Points taken off the victim (0 or 50).
    pub victim_loss: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct SlapInput {
    pub victim_is_leader: bool,
    pub drunk_bonus: i32,
    pub variant: DildoVariant,
    pub crit: bool,
    pub same_team: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Scoreboard {
    stats: BTreeMap<PlayerId, PlayerStats>,
}

impl Scoreboard {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ensure(&mut self, id: PlayerId) {
        self.stats.entry(id).or_default();
    }

    pub fn remove(&mut self, id: PlayerId) {
        self.stats.remove(&id);
    }

    pub fn get(&self, id: PlayerId) -> Option<&PlayerStats> {
        self.stats.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (PlayerId, &PlayerStats)> {
        self.stats.iter().map(|(k, v)| (*k, v))
    }

    pub fn score(&self, id: PlayerId) -> i32 {
        self.stats.get(&id).map_or(0, |s| s.score)
    }

    /// A thrown item landed on `victim`. Teammate hits when friendly fire is off must be
    /// filtered out before calling this (the item just passes through).
    pub fn thrown_hit(
        &mut self,
        rules: &Rules,
        thrower: Option<PlayerId>,
        victim: PlayerId,
        hit: &HitInput,
    ) -> HitOutcome {
        let mut out = HitOutcome::default();
        if !rules.scoring() {
            return out;
        }
        if let Some(t) = thrower
            && t != victim
            && !hit.same_team
            && let Some(s) = self.stats.get_mut(&t)
        {
            s.streak += 1;
            out.streak = s.streak;
            let bounty = if hit.victim_is_leader { BOUNTY } else { 0 };
            out.long_shot = long_shot(hit.charge, hit.dist);
            out.gain = ((HIT + bounty) as f32 * streak_mult(s.streak)).round() as i32
                + hit.drunk_bonus
                + if hit.bum_out { BUM_OUT } else { 0 }
                + out.long_shot;
            s.score += out.gain;
            s.hits += 1;
        }
        if let Some(v) = self.stats.get_mut(&victim) {
            v.score -= VICTIM_PENALTY;
            v.taken += 1;
            v.streak = 0;
            out.victim_loss = VICTIM_PENALTY;
        }
        out
    }

    /// A thrown item was caught: +50 to the catcher; the thrower's streak resets.
    pub fn catch(&mut self, rules: &Rules, catcher: PlayerId, thrower: Option<PlayerId>) {
        if !rules.in_play() {
            return;
        }
        if rules.mode != GameMode::Heist
            && let Some(s) = self.stats.get_mut(&catcher)
        {
            s.score += CATCH;
            s.catches += 1;
        }
        if let Some(t) = thrower
            && let Some(s) = self.stats.get_mut(&t)
        {
            s.streak = 0;
        }
    }

    /// Steak, fish or noodle slap: points to the attacker, no penalty to the victim.
    /// Returns the points gained.
    pub fn stun_slap(
        &mut self,
        rules: &Rules,
        attacker: PlayerId,
        victim: PlayerId,
        base_pts: i32,
        drunk_bonus: i32,
        same_team: bool,
    ) -> i32 {
        if !rules.scoring() {
            return 0;
        }
        let mut gain = 0;
        if !same_team && let Some(s) = self.stats.get_mut(&attacker) {
            gain = base_pts + drunk_bonus;
            s.score += gain;
            s.hits += 1;
            s.throws += 1;
        }
        if let Some(v) = self.stats.get_mut(&victim) {
            v.taken += 1;
        }
        gain
    }

    /// Dildo slap. Scores like a thrown hit, plus the size and critical bonuses.
    pub fn dildo_slap(
        &mut self,
        rules: &Rules,
        attacker: PlayerId,
        victim: PlayerId,
        s: &SlapInput,
    ) -> HitOutcome {
        let mut out = HitOutcome::default();
        if !rules.scoring() {
            return out;
        }
        if !s.same_team
            && let Some(a) = self.stats.get_mut(&attacker)
        {
            a.streak += 1;
            out.streak = a.streak;
            let bounty = if s.victim_is_leader { BOUNTY } else { 0 };
            out.gain = ((HIT + bounty) as f32 * streak_mult(a.streak)).round() as i32
                + s.drunk_bonus
                + s.variant.def().points
                + if s.crit { CRIT_POINTS } else { 0 };
            a.score += out.gain;
            a.hits += 1;
            a.throws += 1;
        }
        if let Some(v) = self.stats.get_mut(&victim) {
            v.score -= VICTIM_PENALTY;
            v.taken += 1;
            v.streak = 0;
            out.victim_loss = VICTIM_PENALTY;
        }
        out
    }

    /// Bare-handed slap (Cheeky mode): +40 and the drunk bonus.
    pub fn silly_slap(
        &mut self,
        rules: &Rules,
        attacker: PlayerId,
        victim: PlayerId,
        drunk_bonus: i32,
        same_team: bool,
    ) -> i32 {
        if !rules.scoring() {
            return 0;
        }
        let mut gain = 0;
        if !same_team && let Some(a) = self.stats.get_mut(&attacker) {
            gain = SILLY + drunk_bonus;
            a.score += gain;
            a.hits += 1;
        }
        if let Some(v) = self.stats.get_mut(&victim) {
            v.taken += 1;
        }
        gain
    }

    /// Human cannonball: +100 to the thrower (not for teammates), the victim counts a hit taken.
    pub fn cannonball(
        &mut self,
        rules: &Rules,
        thrower: PlayerId,
        victim: PlayerId,
        same_team: bool,
    ) -> i32 {
        if !rules.scoring() {
            return 0;
        }
        let mut pts = 0;
        if !same_team
            && thrower != victim
            && let Some(a) = self.stats.get_mut(&thrower)
        {
            pts = CANNONBALL;
            a.score += pts;
            a.hits += 1;
        }
        if let Some(v) = self.stats.get_mut(&victim) {
            v.taken += 1;
        }
        pts
    }

    /// A flat bonus that pays in normal scoring only (not Heist): pool/tramp, Naughty Corner,
    /// help-up, Dazza stun. Returns the points given.
    pub fn award(&mut self, rules: &Rules, id: PlayerId, pts: i32) -> i32 {
        if !rules.scoring() {
            return 0;
        }
        match self.stats.get_mut(&id) {
            Some(s) => {
                s.score += pts;
                pts
            }
            None => 0,
        }
    }

    /// Dazza KO / berserk / flip points: paid in any mode, but only while in play.
    pub fn award_dazza_slap(&mut self, rules: &Rules, id: PlayerId, outcome: usize) -> i32 {
        if !rules.in_play() {
            return 0;
        }
        let pts = DAZZA_SLAP[outcome.min(2)];
        match self.stats.get_mut(&id) {
            Some(s) => {
                s.score += pts;
                pts
            }
            None => 0,
        }
    }

    /// Bank a stolen teddy: +150 and one more teddy on this player's count. Returns the points.
    pub fn bank(&mut self, rules: &Rules, id: PlayerId) -> i32 {
        if !rules.in_play() {
            return 0;
        }
        match self.stats.get_mut(&id) {
            Some(s) => {
                s.score += HEIST_BANK;
                s.banked += 1;
                HEIST_BANK
            }
            None => 0,
        }
    }

    /// The leader: top score if it's above 0 and strictly ahead of second place.
    pub fn leader(&self) -> Option<PlayerId> {
        let mut rows: Vec<(PlayerId, i32)> =
            self.stats.iter().map(|(k, v)| (*k, v.score)).collect();
        rows.sort_by_key(|x| std::cmp::Reverse(x.1));
        match rows.as_slice() {
            [] => None,
            [(id, s)] => (*s > 0).then_some(*id),
            [(id, s), (_, second), ..] => (*s > 0 && s > second).then_some(*id),
        }
    }

    /// Total score per team (Wildcard excluded: they play for themselves).
    pub fn team_totals(&self, teams: &Teams) -> BTreeMap<Team, i32> {
        let mut m = BTreeMap::new();
        for (id, s) in &self.stats {
            if let Some(t) = teams.get(*id)
                && t != Team::Wildcard
            {
                *m.entry(t).or_insert(0) += s.score;
            }
        }
        m
    }

    /// The player who has banked the most in a team, with their count (ties return everyone).
    pub fn top_bankers(&self, teams: &Teams, team: Team) -> (u32, Vec<PlayerId>) {
        let mut best = 0;
        let mut who = vec![];
        for (id, s) in &self.stats {
            if teams.get(*id) != Some(team) || s.banked == 0 {
                continue;
            }
            if s.banked > best {
                best = s.banked;
                who = vec![*id];
            } else if s.banked == best {
                who.push(*id);
            }
        }
        (best, who)
    }

    pub fn reset(&mut self) {
        for s in self.stats.values_mut() {
            *s = PlayerStats::default();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(mode: GameMode) -> Rules {
        Rules {
            mode,
            friendly_fire: false,
            phase: Phase::Play,
        }
    }

    fn board() -> Scoreboard {
        let mut b = Scoreboard::new();
        for id in 1..=4 {
            b.ensure(id);
        }
        b
    }

    #[test]
    fn streak_multiplier() {
        assert_eq!(streak_mult(1), 1.0);
        assert_eq!(streak_mult(2), 1.0);
        assert_eq!(streak_mult(3), 1.5);
        assert_eq!(streak_mult(4), 1.5);
        assert_eq!(streak_mult(5), 2.0);
        assert_eq!(streak_mult(10), 2.0);
    }

    #[test]
    fn long_shot_scales_with_distance_and_charge() {
        assert_eq!(long_shot(1.0, 5.0), 0);
        assert_eq!(long_shot(1.0, 8.0), 0);
        assert_eq!(long_shot(1.0, 25.0), 100);
        assert_eq!(long_shot(1.0, 40.0), 100);
        assert_eq!(long_shot(0.5, 25.0), 50);
        // 16.5 m is exactly halfway between 8 and 25
        assert_eq!(long_shot(1.0, 16.5), 50);
        assert_eq!(long_shot(1.0, 16.5) % 5, 0);
    }

    #[test]
    fn a_plain_hit_is_100_and_costs_the_victim_50() {
        let mut b = board();
        let o = b.thrown_hit(
            &rules(GameMode::FreeForAll),
            Some(1),
            2,
            &HitInput::default(),
        );
        assert_eq!(o.gain, 100);
        assert_eq!(o.streak, 1);
        assert_eq!(b.score(1), 100);
        assert_eq!(b.score(2), -50);
        assert_eq!(b.get(1).unwrap().hits, 1);
        assert_eq!(b.get(2).unwrap().taken, 1);
    }

    #[test]
    fn leader_bounty_drunk_bonus_bum_out_and_long_shot_all_add() {
        let mut b = board();
        let hit = HitInput {
            charge: 1.0,
            dist: 25.0,
            victim_is_leader: true,
            drunk_bonus: 50,
            bum_out: true,
            same_team: false,
        };
        let o = b.thrown_hit(&rules(GameMode::FreeForAll), Some(1), 2, &hit);
        // (100+50)*1 + 50 + 15 + 100
        assert_eq!(o.gain, 315);
    }

    #[test]
    fn streaks_build_and_reset_when_hit() {
        let mut b = board();
        let r = rules(GameMode::FreeForAll);
        let gains: Vec<i32> = (0..5)
            .map(|_| b.thrown_hit(&r, Some(1), 2, &HitInput::default()).gain)
            .collect();
        assert_eq!(gains, vec![100, 100, 150, 150, 200]);
        // being hit resets your streak
        b.thrown_hit(&r, Some(2), 1, &HitInput::default());
        assert_eq!(b.get(1).unwrap().streak, 0);
        assert_eq!(b.thrown_hit(&r, Some(1), 2, &HitInput::default()).gain, 100);
    }

    #[test]
    fn a_catch_resets_the_throwers_streak() {
        let mut b = board();
        let r = rules(GameMode::FreeForAll);
        b.thrown_hit(&r, Some(1), 2, &HitInput::default());
        b.catch(&r, 3, Some(1));
        assert_eq!(b.get(1).unwrap().streak, 0);
        assert_eq!(b.score(3), 50);
        assert_eq!(b.get(3).unwrap().catches, 1);
    }

    #[test]
    fn friendly_fire_pays_the_thrower_nothing_but_still_costs_the_victim() {
        let mut b = board();
        let hit = HitInput {
            same_team: true,
            ..Default::default()
        };
        let o = b.thrown_hit(&rules(GameMode::Teams), Some(1), 2, &hit);
        assert_eq!(o.gain, 0);
        assert_eq!(b.score(1), 0);
        assert_eq!(b.score(2), -50);
    }

    #[test]
    fn nothing_scores_outside_play() {
        let mut b = board();
        for phase in [Phase::Warmup, Phase::Countdown, Phase::Results, Phase::Menu] {
            let r = Rules {
                mode: GameMode::FreeForAll,
                friendly_fire: false,
                phase,
            };
            b.thrown_hit(&r, Some(1), 2, &HitInput::default());
            b.catch(&r, 1, None);
            assert_eq!(b.bank(&r, 1), 0);
            assert_eq!(b.award(&r, 1, 25), 0);
        }
        assert_eq!(b.score(1), 0);
        assert_eq!(b.score(2), 0);
    }

    #[test]
    fn heist_only_banking_scores() {
        let mut b = board();
        let r = rules(GameMode::Heist);
        // thrown hit: nothing for the thrower, nothing off the victim
        let o = b.thrown_hit(&r, Some(1), 2, &HitInput::default());
        assert_eq!(o, HitOutcome::default());
        assert_eq!(b.score(1), 0);
        assert_eq!(b.score(2), 0);
        assert_eq!(b.get(2).unwrap().taken, 0);
        // catch, slaps, cannonball, bonuses: all nothing
        b.catch(&r, 3, Some(1));
        assert_eq!(b.stun_slap(&r, 1, 2, 50, 25, false), 0);
        let slap = SlapInput {
            victim_is_leader: false,
            drunk_bonus: 0,
            variant: DildoVariant::Gold,
            crit: true,
            same_team: false,
        };
        assert_eq!(b.dildo_slap(&r, 1, 2, &slap), HitOutcome::default());
        assert_eq!(b.silly_slap(&r, 1, 2, 0, false), 0);
        assert_eq!(b.cannonball(&r, 1, 2, false), 0);
        assert_eq!(b.award(&r, 1, 50), 0);
        assert_eq!(b.award(&r, 1, 20), 0);
        for id in 1..=4 {
            assert_eq!(b.score(id), 0, "player {id} should still be on 0");
        }
        // banking is the only thing that scores
        assert_eq!(b.bank(&r, 1), 150);
        assert_eq!(b.score(1), 150);
        assert_eq!(b.get(1).unwrap().banked, 1);
        b.bank(&r, 1);
        assert_eq!(b.score(1), 300);
        assert_eq!(b.get(1).unwrap().banked, 2);
    }

    #[test]
    fn dazza_slap_points_still_pay_in_heist_but_the_stun_does_not() {
        let mut b = board();
        let r = rules(GameMode::Heist);
        assert_eq!(b.award_dazza_slap(&r, 1, 0), 75);
        assert_eq!(b.award_dazza_slap(&r, 1, 1), 25);
        assert_eq!(b.award_dazza_slap(&r, 1, 2), 50);
        assert_eq!(b.award(&r, 1, DAZZA_STUN), 0);
        assert_eq!(b.score(1), 150);
    }

    #[test]
    fn dildo_slap_scoring() {
        let mut b = board();
        let r = rules(GameMode::FreeForAll);
        let s = SlapInput {
            victim_is_leader: false,
            drunk_bonus: 0,
            variant: DildoVariant::Jumbo,
            crit: true,
            same_team: false,
        };
        let o = b.dildo_slap(&r, 1, 2, &s);
        assert_eq!(o.gain, 100 + 50 + 60);
        assert_eq!(b.score(2), -50);
        let mini = SlapInput {
            variant: DildoVariant::Mini,
            crit: false,
            ..s
        };
        let o2 = b.dildo_slap(&r, 3, 4, &mini);
        assert_eq!(o2.gain, 100 - 25);
    }

    #[test]
    fn stun_slap_pays_and_does_not_penalise() {
        let mut b = board();
        let r = rules(GameMode::FreeForAll);
        assert_eq!(b.stun_slap(&r, 1, 2, 50, 25, false), 75);
        assert_eq!(b.score(2), 0);
        assert_eq!(b.get(2).unwrap().taken, 1);
        assert_eq!(b.stun_slap(&r, 1, 2, 50, 25, true), 0);
    }

    #[test]
    fn leader_needs_a_clear_lead_above_zero() {
        let mut b = board();
        assert_eq!(b.leader(), None);
        let r = rules(GameMode::FreeForAll);
        b.award(&r, 1, 100);
        assert_eq!(b.leader(), Some(1));
        b.award(&r, 2, 100);
        assert_eq!(b.leader(), None, "a tie has no leader");
        b.award(&r, 2, 1);
        assert_eq!(b.leader(), Some(2));
    }

    #[test]
    fn top_banker_per_team() {
        let mut b = board();
        let r = rules(GameMode::Heist);
        let mut teams = Teams::default();
        teams.set(1, Team::Red);
        teams.set(2, Team::Red);
        teams.set(3, Team::Blue);
        teams.set(4, Team::Blue);
        b.bank(&r, 1);
        b.bank(&r, 2);
        b.bank(&r, 2);
        b.bank(&r, 3);
        b.bank(&r, 4);
        assert_eq!(b.top_bankers(&teams, Team::Red), (2, vec![2]));
        assert_eq!(b.top_bankers(&teams, Team::Blue), (1, vec![3, 4]));
        assert_eq!(b.top_bankers(&teams, Team::Green), (0, vec![]));
    }

    #[test]
    fn team_totals_skip_the_wildcard() {
        let mut b = board();
        let r = rules(GameMode::Teams);
        let mut teams = Teams::default();
        teams.set(1, Team::Red);
        teams.set(2, Team::Blue);
        teams.set(3, Team::Wildcard);
        b.award(&r, 1, 100);
        b.award(&r, 2, 40);
        b.award(&r, 3, 500);
        let t = b.team_totals(&teams);
        assert_eq!(t.get(&Team::Red), Some(&100));
        assert_eq!(t.get(&Team::Blue), Some(&40));
        assert_eq!(t.get(&Team::Wildcard), None);
    }
}
