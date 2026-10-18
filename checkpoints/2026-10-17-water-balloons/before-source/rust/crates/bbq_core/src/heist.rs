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

/// Where the smoko pad goes in Teddy Heist: next to the pool's east end, clear of every base
/// (the default spot is right in the red base's doorway). It is the same for 2, 3 or 4 teams.
pub fn smoko_spot(_teams: usize) -> (f32, f32) {
    (-8.0, 9.0)
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

// ------------------------------------------------------------------ the arena

use crate::vec::V3;
use crate::yard::{Collider, Kind};
use crate::{YARD_HALF_X, YARD_HALF_Z};

/// How a piece of the arena is coloured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    /// A base wall, in its team's colour.
    Team(Team),
    /// Stone steps, alternating light and dark.
    Stone(bool),
    /// The crates in the middle (the big ones, then the small ones).
    Crate(bool),
}

/// One solid piece of the Heist arena and what colour to draw it.
#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub collider: Collider,
    pub tint: Tint,
}

/// Height of the stone steps.
pub const STEP_HEIGHT: f32 = 2.6;
/// Every step is this wide in z.
pub const STEP_SIZE: f32 = 1.0;
/// The half-round bay in the middle of each side wall.
pub const BAY_RADIUS: f32 = 6.0;
pub const BAY_DEPTH: f32 = 3.6;

/// The base walls (all but the doorway side), for team `team_index`.
fn base_walls(def: BaseDef, team: Team) -> Vec<Piece> {
    let (h, th) = (BASE_HALF, WALL_THICKNESS);
    let mut v = Vec::new();
    let mut wall = |cx: f32, cz: f32, w: f32, d: f32| {
        v.push(Piece {
            collider: Collider::centred(cx, cz, w, d, WALL_HEIGHT).of(Kind::HeistWall),
            tint: Tint::Team(team),
        });
    };
    if def.open != Open::ZNeg {
        wall(def.x, def.z - h, 2.0 * h, th);
    }
    if def.open != Open::ZPos {
        wall(def.x, def.z + h, 2.0 * h, th);
    }
    if def.open != Open::XNeg {
        wall(def.x - h, def.z, th, 2.0 * h);
    }
    if def.open != Open::XPos {
        wall(def.x + h, def.z, th, 2.0 * h);
    }
    v
}

/// The oval of steps (everything outside an ellipse with half-axes `ARENA_A` and `ARENA_B` is
/// filled with 1 m steps so the corners curve away) plus a half-round bay on each side.
fn oval_steps() -> Vec<Piece> {
    let mut v = Vec::new();
    let mut n = 0u32;
    let mut put = |cx: f32, cz: f32, w: f32, d: f32| {
        v.push(Piece {
            collider: Collider::centred(cx, cz, w, d, STEP_HEIGHT).of(Kind::Steps),
            tint: Tint::Stone(n.is_multiple_of(2)),
        });
        n += 1;
    };
    let mut zc = -YARD_HALF_Z + STEP_SIZE / 2.0;
    while zc < YARD_HALF_Z {
        let e = if zc.abs() >= ARENA_B {
            0.0
        } else {
            ARENA_A * (1.0 - (zc / ARENA_B) * (zc / ARENA_B)).sqrt()
        };
        let w = YARD_HALF_X - e.min(YARD_HALF_X);
        if w > 0.35 {
            put(-YARD_HALF_X + w / 2.0, zc, w, STEP_SIZE);
            put(YARD_HALF_X - w / 2.0, zc, w, STEP_SIZE);
        }
        if zc.abs() < BAY_RADIUS {
            let dep = BAY_DEPTH * (1.0 - (zc / BAY_RADIUS) * (zc / BAY_RADIUS)).sqrt();
            if dep > 0.5 {
                put(-YARD_HALF_X + dep / 2.0, zc, dep, STEP_SIZE);
                put(YARD_HALF_X - dep / 2.0, zc, dep, STEP_SIZE);
            }
        }
        zc += STEP_SIZE;
    }
    v
}

/// A crate cross dead in the middle (so no base-to-base run is ever a straight line) and two
/// small crates beside it.
fn centrepiece() -> Vec<Piece> {
    let big = |cx, cz, w, d| Piece {
        collider: Collider::centred(cx, cz, w, d, WALL_HEIGHT).of(Kind::HeistCrate),
        tint: Tint::Crate(false),
    };
    let small = |cx, cz| Piece {
        collider: Collider::centred(cx, cz, 1.6, 1.6, WALL_HEIGHT).of(Kind::HeistCrate),
        tint: Tint::Crate(true),
    };
    vec![
        big(0.0, 0.0, 4.4, 1.3),
        big(0.0, 0.0, 1.3, 4.4),
        small(4.8, 2.8),
        small(-4.6, -3.0),
    ]
}

/// Everything solid that Heist adds to the yard, for `teams` teams.
pub fn arena(teams: usize) -> Vec<Piece> {
    let keys = team_keys(teams);
    let mut v = Vec::new();
    for (def, team) in layout(teams).into_iter().zip(keys.iter()) {
        v.extend(base_walls(def, *team));
    }
    v.extend(oval_steps());
    v.extend(centrepiece());
    v
}

/// The base of `team` in a game of `teams` teams.
pub fn base_of(teams: usize, team: Team) -> Option<BaseDef> {
    let i = team_keys(teams).iter().position(|t| *t == team)?;
    layout(teams).get(i).copied()
}

/// Where the `n`th person of `team` starts (inside their own base).
pub fn spawn_in_base(teams: usize, team: Team, n: usize) -> Option<(f32, f32)> {
    let b = base_of(teams, team)?;
    let o = SPAWN_OFFSETS[n % SPAWN_OFFSETS.len()];
    Some((b.x + o.0, b.z + o.1))
}

/// Where the `i`th teddy of a base starts (the corners, then random if there are more).
pub fn teddy_spot(b: BaseDef, i: usize) -> Option<(f32, f32)> {
    TEDDY_OFFSETS.get(i).map(|o| (b.x + o.0, b.z + o.1))
}

/// Is `(x, z)` inside the walls of this base?
pub fn inside_base(b: BaseDef, x: f32, z: f32) -> bool {
    (x - b.x).abs() < BASE_HALF + 0.1 && (z - b.z).abs() < BASE_HALF + 0.1
}

/// The doorway of a base: a point just outside the open side.
pub fn gate(b: BaseDef) -> V3 {
    let d = BASE_HALF + 1.2;
    let (dx, dz) = match b.open {
        Open::XPos => (d, 0.0),
        Open::XNeg => (-d, 0.0),
        Open::ZPos => (0.0, d),
        Open::ZNeg => (0.0, -d),
    };
    V3::new(b.x + dx, 0.0, b.z + dz)
}

/// Going into a base: walk to the doorway first, then to the target (`heistVia`).
pub fn via(me: V3, b: BaseDef, target: V3) -> V3 {
    if inside_base(b, me.x, me.z) {
        return target;
    }
    let g = gate(b);
    if (g.x - me.x).hypot(g.z - me.z) > 1.5 {
        g
    } else {
        target
    }
}

/// Someone inside a walled base who wants to be somewhere outside it has to leave by the
/// doorway, not shove at the wall (`heistLeave`).
pub fn leave(me: V3, target: V3, bases: &[BaseDef]) -> V3 {
    for b in bases {
        if inside_base(*b, me.x, me.z) && !inside_base(*b, target.x, target.z) {
            return gate(*b);
        }
    }
    target
}

// ------------------------------------------------------------------ bots playing the objective

/// A person, as the Heist bots see them.
#[derive(Clone, Debug)]
pub struct Who {
    pub id: u32,
    pub pos: V3,
    pub team: Team,
    pub is_bot: bool,
    /// Teams whose teddies this person is carrying.
    pub carrying: Vec<Team>,
}

/// A teddy, as the Heist bots see it.
#[derive(Clone, Copy, Debug)]
pub struct Stray {
    pub team: Team,
    pub pos: V3,
    pub on_ground: bool,
}

/// What one bot should be going for (`heistBotGoal`), in priority order: bring a stolen teddy
/// home; chase someone running off with one of ours; touch one of ours that got dropped; guard
/// (the first bot of a team of two or more); otherwise raid the nearest enemy teddy.
/// `patrol` is the guard's current corner (0 to 3); it advances as they reach each one.
pub fn bot_goal(
    me: &Who,
    hands_free: bool,
    teams: usize,
    people: &[Who],
    teddies: &[Stray],
    patrol: &mut u32,
) -> Option<V3> {
    let bases = layout(teams);
    let base_of_team = |t: Team| -> Option<BaseDef> {
        team_keys(teams)
            .iter()
            .position(|k| *k == t)
            .and_then(|i| bases.get(i).copied())
    };
    let home = base_of_team(me.team)?;
    let home_c = V3::new(home.x, 0.0, home.z);
    let goal = (|| {
        if me.carrying.iter().any(|t| *t != me.team) {
            return Some(via(me.pos, home, home_c)); // carrying a stolen teddy: get it home
        }
        // an enemy is running off with one of ours
        let mut carrier: Option<(V3, f32)> = None;
        for c in people {
            if c.id == me.id || c.team == me.team || !c.carrying.contains(&me.team) {
                continue;
            }
            let d = c.pos.horiz_dist(me.pos);
            if d < 22.0 && carrier.is_none_or(|(_, bd)| d < bd) {
                carrier = Some((c.pos, d));
            }
        }
        if let Some((p, _)) = carrier {
            return Some(p);
        }
        // one of ours got dropped away from home: touch it to send it back
        let mut loose: Option<(V3, f32)> = None;
        for t in teddies {
            if t.team != me.team || !t.on_ground {
                continue;
            }
            if t.pos.horiz_dist(home_c) < 2.8 {
                continue;
            }
            let d = t.pos.horiz_dist(me.pos);
            if d < 20.0 && loose.is_none_or(|(_, bd)| d < bd) {
                loose = Some((t.pos, d));
            }
        }
        if let Some((p, _)) = loose {
            return Some(p);
        }
        // one bot per team of two or more guards the base
        let mates: Vec<&Who> = people.iter().filter(|c| c.team == me.team).collect();
        let first_bot = mates.iter().filter(|c| c.is_bot).map(|c| c.id).min();
        if mates.len() >= 2 && first_bot == Some(me.id) {
            for c in people {
                if c.team != me.team && c.pos.horiz_dist(home_c) < 10.0 {
                    return Some(c.pos);
                }
            }
            let ang = *patrol as f32 * std::f32::consts::FRAC_PI_2 + std::f32::consts::FRAC_PI_4;
            let wp = V3::new(home.x + ang.cos() * 1.9, 0.0, home.z + ang.sin() * 1.9);
            if wp.horiz_dist(me.pos) < 0.9 {
                *patrol = (*patrol + 1) % 4;
            }
            return Some(via(me.pos, home, wp));
        }
        // otherwise raid: the nearest enemy teddy
        if !hands_free {
            return None;
        }
        let mut best: Option<(&Stray, f32)> = None;
        for t in teddies {
            if t.team == me.team || !t.on_ground {
                continue;
            }
            let d = t.pos.horiz_dist(me.pos);
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((t, d));
            }
        }
        let (t, _) = best?;
        let eb = base_of_team(t.team)?;
        if inside_base(eb, t.pos.x, t.pos.z) {
            Some(via(me.pos, eb, t.pos))
        } else {
            Some(t.pos)
        }
    })();
    goal.map(|g| leave(me.pos, g, &bases))
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_heist_smoko_is_clear_of_every_base_and_beside_the_pool() {
        for teams in 2..=4 {
            let (sx, sz) = smoko_spot(teams);
            // the pad (radius about 4) and chairs must not touch any base's walls
            for b in layout(teams) {
                let (dx, dz) = ((sx - b.x).abs() - BASE_HALF, (sz - b.z).abs() - BASE_HALF);
                assert!(dx.max(dz) > 4.5, "teams {teams}: too close to base at {},{}", b.x, b.z);
            }
            // beside the pool: its east edge is 5 m or less away
            assert!(sx - crate::yard::POOL_X1 > -1.0 && sx - crate::yard::POOL_X1 < 6.0);
            assert!(sz > crate::yard::POOL_Z0 - 2.0 && sz < crate::yard::POOL_Z1 + 2.0);
        }
        // and the old spot is in the red base's way
        let b = layout(2)[0];
        assert!(((crate::yard::SMOKO_X - b.x).abs() - BASE_HALF).max((crate::yard::SMOKO_Z - b.z).abs() - BASE_HALF) < 4.5);
    }

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

    #[test]
    fn the_arena_fills_the_corners_but_leaves_the_oval_clear() {
        let pieces = arena(2);
        let steps: Vec<&Piece> = pieces
            .iter()
            .filter(|p| p.collider.kind == Kind::Steps)
            .collect();
        assert!(steps.len() > 40, "{}", steps.len());
        // the very middle of the yard and the middle of each base are open
        let solid = |x: f32, z: f32, y: f32| {
            pieces.iter().any(|p| {
                x > p.collider.x0
                    && x < p.collider.x1
                    && z > p.collider.z0
                    && z < p.collider.z1
                    && y < p.collider.h
            })
        };
        assert!(!solid(10.0, 10.0, 0.5));
        assert!(!solid(-20.0, 0.0, 0.5), "inside the red base");
        // the corners are filled
        assert!(solid(32.0, 22.7, 0.5));
        assert!(solid(-32.0, -22.7, 0.5));
        // the bay pushes into the middle of the side walls
        assert!(solid(31.0, 0.5, 0.5));
        assert!(!solid(31.0, 10.0, 0.5), "but only in the middle");
        // steps are 2.6 high, walls 2.1
        assert!(steps.iter().all(|p| p.collider.h == STEP_HEIGHT));
        assert!(
            pieces
                .iter()
                .filter(|p| p.collider.kind == Kind::HeistWall)
                .all(|p| p.collider.h == WALL_HEIGHT)
        );
    }

    #[test]
    fn every_base_has_three_walls_and_a_doorway() {
        for n in 2..=4 {
            let pieces = arena(n);
            let walls = pieces
                .iter()
                .filter(|p| p.collider.kind == Kind::HeistWall)
                .count();
            assert_eq!(walls, 3 * n);
            // each wall is in its team's colour
            for (i, t) in team_keys(n).iter().enumerate() {
                let mine = pieces.iter().filter(|p| p.tint == Tint::Team(*t)).count();
                assert_eq!(mine, 3, "team {i}");
            }
        }
        // red's base opens towards +x: no wall there, so you can walk in
        let pieces = arena(2);
        let blocked = |x: f32, z: f32| {
            pieces.iter().any(|p| {
                p.collider.kind == Kind::HeistWall
                    && x > p.collider.x0
                    && x < p.collider.x1
                    && z > p.collider.z0
                    && z < p.collider.z1
            })
        };
        assert!(!blocked(-17.0, 0.0), "the doorway");
        assert!(blocked(-23.0, 0.0), "the back wall");
    }

    #[test]
    fn there_is_a_crate_cross_in_the_middle() {
        let crates: Vec<_> = arena(2)
            .into_iter()
            .filter(|p| p.collider.kind == Kind::HeistCrate)
            .collect();
        assert_eq!(crates.len(), 4);
        assert!(
            crates
                .iter()
                .any(|p| (p.collider.x1 - p.collider.x0 - 4.4).abs() < 1e-5)
        );
    }

    #[test]
    fn spawns_and_teddy_spots_are_inside_the_base() {
        for n in 2..=4 {
            for t in team_keys(n) {
                let b = base_of(n, *t).unwrap();
                for k in 0..4 {
                    let (x, z) = spawn_in_base(n, *t, k).unwrap();
                    assert!(inside_base(b, x, z));
                    let (tx, tz) = teddy_spot(b, k).unwrap();
                    assert!(inside_base(b, tx, tz));
                }
                assert_eq!(teddy_spot(b, 4), None);
            }
        }
        assert_eq!(base_of(2, Team::Green), None);
        assert_eq!(
            spawn_in_base(2, Team::Red, 5),
            spawn_in_base(2, Team::Red, 1)
        );
    }

    #[test]
    fn the_doorway_is_just_outside_the_open_side() {
        let b = base_of(2, Team::Red).unwrap(); // (-20, 0) opens +x
        assert_eq!(gate(b), V3::new(-20.0 + 4.2, 0.0, 0.0));
        let b = base_of(3, Team::Green).unwrap(); // (0, 16) opens -z
        assert_eq!(gate(b), V3::new(0.0, 0.0, 16.0 - 4.2));
    }

    #[test]
    fn going_in_means_the_doorway_first() {
        let b = base_of(2, Team::Red).unwrap();
        let target = V3::new(-20.0, 0.0, 0.0);
        // from far away: the doorway
        assert_eq!(via(V3::new(0.0, 0.0, 10.0), b, target), gate(b));
        // standing at the doorway already: straight in
        assert_eq!(via(gate(b), b, target), target);
        // already inside: straight to it
        assert_eq!(via(V3::new(-19.0, 0.0, 1.0), b, target), target);
    }

    #[test]
    fn leaving_a_base_means_the_doorway_too() {
        let bases = layout(2);
        let inside = V3::new(-21.0, 0.0, 1.0);
        let out = V3::new(0.0, 0.0, 5.0);
        assert_eq!(leave(inside, out, &bases), gate(bases[0]));
        // wanting something in the same base: no detour
        assert_eq!(
            leave(inside, V3::new(-19.0, 0.0, 0.0), &bases),
            V3::new(-19.0, 0.0, 0.0)
        );
        // outside already: no detour
        assert_eq!(
            leave(out, V3::new(10.0, 0.0, 0.0), &bases),
            V3::new(10.0, 0.0, 0.0)
        );
    }

    fn who(id: u32, x: f32, z: f32, team: Team, bot: bool) -> Who {
        Who {
            id,
            pos: V3::new(x, 0.0, z),
            team,
            is_bot: bot,
            carrying: Vec::new(),
        }
    }

    #[test]
    fn a_thief_heads_home_through_the_doorway() {
        let mut me = who(1, 0.0, 10.0, Team::Red, true);
        me.carrying = vec![Team::Blue];
        let mut patrol = 0;
        let g = bot_goal(&me, false, 2, std::slice::from_ref(&me), &[], &mut patrol).unwrap();
        // red's base is at (-20, 0), door at (-15.8, 0)
        assert_eq!(g, gate(base_of(2, Team::Red).unwrap()));
    }

    #[test]
    fn it_chases_someone_running_off_with_its_teddy() {
        let me = who(1, 0.0, 0.0, Team::Red, true);
        let mut thief = who(2, 8.0, 3.0, Team::Blue, false);
        thief.carrying = vec![Team::Red];
        let far = {
            let mut f = who(3, 50.0, 0.0, Team::Blue, false);
            f.carrying = vec![Team::Red];
            f
        };
        let mut patrol = 0;
        let g = bot_goal(
            &me,
            true,
            2,
            &[me.clone(), thief.clone(), far],
            &[],
            &mut patrol,
        )
        .unwrap();
        assert_eq!(g, thief.pos);
    }

    #[test]
    fn it_touches_its_own_teddy_when_dropped_away_from_home() {
        let me = who(1, 0.0, 0.0, Team::Red, true);
        let strays = [
            Stray {
                team: Team::Red,
                pos: V3::new(-19.0, 0.2, 0.0),
                on_ground: true,
            }, // at home
            Stray {
                team: Team::Red,
                pos: V3::new(6.0, 0.2, 2.0),
                on_ground: true,
            }, // dropped
        ];
        let mut patrol = 0;
        let g = bot_goal(&me, true, 2, std::slice::from_ref(&me), &strays, &mut patrol).unwrap();
        assert_eq!(g, V3::new(6.0, 0.2, 2.0));
    }

    #[test]
    fn with_a_mate_the_first_bot_guards_and_the_other_raids() {
        let a = who(1, -16.0, 1.0, Team::Red, true);
        let b = who(2, -10.0, 4.0, Team::Red, true);
        let strays = [Stray {
            team: Team::Blue,
            pos: V3::new(19.0, 0.2, 0.0),
            on_ground: true,
        }];
        let people = [a.clone(), b.clone()];
        let mut pa = 0;
        let mut pb = 0;
        // bot 1 patrols a corner of its own base
        let ga = bot_goal(&a, true, 2, &people, &strays, &mut pa).unwrap();
        assert!(ga.horiz_dist(V3::new(-20.0, 0.0, 0.0)) < 4.5, "{ga:?}");
        // bot 2 goes raiding, via the enemy doorway (blue base at (20, 0) opens -x)
        let gb = bot_goal(&b, true, 2, &people, &strays, &mut pb).unwrap();
        assert_eq!(gb, gate(base_of(2, Team::Blue).unwrap()));
    }

    #[test]
    fn a_guard_goes_for_an_intruder_near_home_and_otherwise_walks_its_patrol() {
        let a = who(1, -14.0, 3.0, Team::Red, true); // just outside the door
        let mate = who(2, -19.0, -1.0, Team::Red, false);
        let intruder = who(3, -17.0, 6.0, Team::Blue, false);
        let mut patrol = 0;
        let g = bot_goal(
            &a,
            true,
            2,
            &[a.clone(), mate.clone(), intruder.clone()],
            &[],
            &mut patrol,
        )
        .unwrap();
        assert_eq!(g, intruder.pos);
        // at a patrol corner it moves on to the next
        let ang = std::f32::consts::FRAC_PI_4;
        let corner = who(1, -20.0 + ang.cos() * 1.9, ang.sin() * 1.9, Team::Red, true);
        let mut patrol = 0;
        bot_goal(&corner, true, 2, &[corner.clone(), mate], &[], &mut patrol);
        assert_eq!(patrol, 1);
    }

    #[test]
    fn a_lone_bot_raids_and_full_hands_means_nothing_to_do() {
        let me = who(1, 0.0, 0.0, Team::Red, true);
        let strays = [
            Stray {
                team: Team::Blue,
                pos: V3::new(8.0, 0.2, 0.0),
                on_ground: true,
            },
            Stray {
                team: Team::Blue,
                pos: V3::new(15.0, 0.2, 0.0),
                on_ground: true,
            },
            Stray {
                team: Team::Blue,
                pos: V3::new(-3.0, 0.2, 0.0),
                on_ground: false,
            },
        ];
        let mut patrol = 0;
        // the nearest on-the-ground enemy teddy that is not in a base
        let g = bot_goal(&me, true, 2, std::slice::from_ref(&me), &strays, &mut patrol).unwrap();
        assert_eq!(g, V3::new(8.0, 0.2, 0.0));
        assert_eq!(
            bot_goal(&me, false, 2, std::slice::from_ref(&me), &strays, &mut patrol),
            None
        );
        // someone with no base (not a team in this game) has no goal
        let alien = who(9, 0.0, 0.0, Team::Green, true);
        assert_eq!(
            bot_goal(&alien, true, 2, std::slice::from_ref(&alien), &strays, &mut patrol),
            None
        );
    }

    #[test]
    fn a_bot_inside_its_own_base_leaves_by_the_door_when_raiding() {
        let me = who(1, -21.0, 1.0, Team::Red, true);
        let strays = [Stray {
            team: Team::Blue,
            pos: V3::new(2.0, 0.2, 6.0),
            on_ground: true,
        }];
        let mut patrol = 0;
        let g = bot_goal(&me, true, 2, std::slice::from_ref(&me), &strays, &mut patrol).unwrap();
        assert_eq!(g, gate(base_of(2, Team::Red).unwrap()));
    }
}
