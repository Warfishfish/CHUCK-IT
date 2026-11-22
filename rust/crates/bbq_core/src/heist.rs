//! Teddy Heist rules (spec section 13). Banks determine the winning team.

use std::collections::BTreeMap;

use crate::teams::Team;

/// Half the width of a base (bases are 6 x 6 m).
pub const BASE_HALF: f32 = 3.0;
pub const WALL_THICKNESS: f32 = 0.35;
pub const WALL_HEIGHT: f32 = 2.1;
/// Banking zone radius around a base centre.
pub const BASE_RADIUS: f32 = 2.55;
/// A teddy lying this close to its own base counts as "already home".
pub const HOME_RADIUS: f32 = 2.6;
/// Points per banked teddy (see `scoring::HEIST_BANK`).
pub const BANK_POINTS: i32 = crate::scoring::HEIST_BANK;
/// Teddies sit at these offsets from the base centre.
pub const TEDDY_OFFSETS: [(f32, f32); 4] = [(-1.3, -1.3), (1.3, -1.3), (-1.3, 1.3), (1.3, 1.3)];
/// Players spawn inside their own base at these offsets, in turn.
pub const SPAWN_OFFSETS: [(f32, f32); 25] = [
    (-2.0, -2.0),
    (-2.0, -1.0),
    (-2.0, 0.0),
    (-2.0, 1.0),
    (-2.0, 2.0),
    (-1.0, -2.0),
    (-1.0, -1.0),
    (-1.0, 0.0),
    (-1.0, 1.0),
    (-1.0, 2.0),
    (0.0, -2.0),
    (0.0, -1.0),
    (0.0, 0.0),
    (0.0, 1.0),
    (0.0, 2.0),
    (1.0, -2.0),
    (1.0, -1.0),
    (1.0, 0.0),
    (1.0, 1.0),
    (1.0, 2.0),
    (2.0, -2.0),
    (2.0, -1.0),
    (2.0, 0.0),
    (2.0, 1.0),
    (2.0, 2.0),
];
/// The oval arena: half-axes and the bay in the middle of each side.
pub const ARENA_A: f32 = 58.4;
pub const ARENA_B: f32 = 58.4;
pub const LAYOUTS_PER_TEAM_COUNT: usize = 1;

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
    layout_for(teams, 0)
}

/// Hand-authored routes for each player count. The three arrangements deliberately move
/// the approaches around the arena so players cannot memorise one straight base-to-base run.
pub fn layout_for(teams: usize, _variant: usize) -> Vec<BaseDef> {
    let corner = ARENA_A * 0.875;
    let base = |x, z, open| BaseDef { x, z, open };
    let mut bases = vec![base(-corner, -corner, Open::XPos)];
    bases.push(base(corner, if teams == 2 { corner } else { -corner }, Open::XNeg));
    if teams >= 3 { bases.push(base(-corner, corner, Open::XPos)); }
    if teams >= 4 { bases.push(base(corner, corner, Open::XNeg)); }
    bases
}

/// Legacy safe-zone coordinate retained for tests. Heist draws no Smoko pad or backyard pool;
/// this point stays clear of every base across 2, 3 and 4 team layouts.
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

    pub fn from_counts(counts: [u32; 4], teams: usize) -> Self {
        Self { banked: team_keys(teams).iter().copied().zip(counts).collect() }
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

/// A deliberate charged lob clears interior walls. Level passes stay short and catchable.
pub fn pass_velocity(velocity: V3, charge: f32) -> V3 {
    let scale = (12.0 / velocity.horiz_len().max(0.001)).min(1.0);
    let lift = if charge >= 0.65 && velocity.y >= 7.0 { 12.0 } else { 3.0 };
    V3::new(velocity.x * scale, velocity.y.clamp(-8.0, lift), velocity.z * scale)
}

use crate::yard::{Collider, Kind};

/// How a piece of the arena is coloured.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tint {
    /// A base wall, in its team's colour.
    Team(Team),
    /// Stone steps, alternating light and dark.
    Stone(bool),
    /// The crates in the middle (the big ones, then the small ones).
    Crate(bool),
    /// Warm-coloured laser-tag maze walls.
    Maze(bool),
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

fn reference_walls(teams: usize) -> Vec<Piece> {
    let source = include_str!("../../../../public/heist-layouts.js");
    let cell = ARENA_A / 8.0;
    source.lines().filter(|line| line.starts_with(char::is_numeric)).filter_map(|line| {
        let mut fields = line.split_whitespace().map(|f| f.parse::<u8>().unwrap());
        if usize::from(fields.next().unwrap()) != teams { return None; }
        let mut coord = || (f32::from(fields.next().unwrap()) - 8.0) * cell;
        let (x, z, end_x, end_z) = (coord(), coord(), coord(), coord());
        Some(Piece {
            collider: Collider::centred((x+end_x)*0.5, (z+end_z)*0.5,
                (end_x-x).max(0.32), (end_z-z).max(0.32), 3.2).of(Kind::HeistCrate),
            tint: Tint::Maze(x == end_x),
        })
    }).collect()
}

/// Cell centres along two interior wall runs in each reference drawing.
pub fn vault_sites(teams: usize) -> [(f32, f32);2] {
    let cells = match teams {
        2 => [(6.5, 3.0), (9.5, 9.0)],
        3 => [(9.5, 3.0), (4.5, 8.0)],
        _ => [(5.5, 4.0), (9.5, 9.0)],
    };
    cells.map(|(x,z)| ((x-8.0)*ARENA_A/8.0,(z-8.0)*ARENA_B/8.0))
}

/// Everything solid that Heist adds to the yard, for `teams` teams.
pub fn arena(teams: usize) -> Vec<Piece> {
    arena_for(teams, 0)
}

pub fn arena_for(teams: usize, variant: usize) -> Vec<Piece> {
    let keys = team_keys(teams);
    let mut v = Vec::new();
    let bases = layout_for(teams, variant);
    for (def, team) in bases.iter().copied().zip(keys.iter()) {
        v.extend(base_walls(def, *team));
    }
    v.extend(reference_walls(teams.clamp(2,4)));
    for (x,z) in vault_sites(teams) {
        for (depth,height) in [(3.5,1.05),(1.35,2.15)] {
            v.push(Piece {collider: Collider::centred(x,z+depth,2.4,2.1,height).of(Kind::HeistCrate),
                tint: Tint::Crate(height < 2.0)});
        }
    }
    v
}

/// The base of `team` in a game of `teams` teams.
pub fn base_of(teams: usize, team: Team) -> Option<BaseDef> {
    base_of_variant(teams, 0, team)
}

pub fn base_of_variant(teams: usize, variant: usize, team: Team) -> Option<BaseDef> {
    let i = team_keys(teams).iter().position(|t| *t == team)?;
    layout_for(teams, variant).get(i).copied()
}

/// Where the `n`th person of `team` starts (inside their own base).
/// Pull-off points in the four perimeter corridors, clear of the corner bases.
pub fn outer_supply_spots() -> impl Iterator<Item = (f32, f32)> {
    let edge = ARENA_A * 15.0 / 16.0;
    let cell = ARENA_A / 8.0;
    [-4.5, -2.5, -0.5, 1.5, 3.5].into_iter().flat_map(move |offset| {
        let along = offset * cell;
        [(along, -edge), (edge, along), (along, edge), (-edge, along)]
    })
}

/// Start slots in the base and along its immediately reachable approach lanes.
pub fn spawn_slots(yard: &crate::yard::Yard, team: Team) -> Vec<(f32, f32)> {
    let Some(teams) = yard.heist else { return Vec::new(); };
    let Some(base) = base_of_variant(teams, yard.heist_variant, team) else { return Vec::new(); };
    let doorway = gate(base);
    let direction = (doorway - V3::new(base.x, 0.0, base.z)).normalised();
    let side = V3::new(-direction.z, 0.0, direction.x);
    let mut lanes = Vec::new();
    for row in 0..if teams == 2 { 4 } else { 8 } {
        for offset in [1.3, -1.3, 2.6, -2.6] {
            let near_door = doorway + side * offset;
            let step = row as f32 * 1.3;
            let spot = near_door + direction * step;
            let clear = (0..=row * 4).all(|i| {
                let at = near_door + direction * (i as f32 * 0.325);
                yard.heist_clear(at.x, at.z, 0.65)
            });
            if clear { lanes.push((spot.x, spot.z)); }
        }
    }
    let mut slots = Vec::new();
    let mut lanes = lanes.into_iter();
    for (index, (x,z)) in SPAWN_OFFSETS.into_iter().enumerate() {
        slots.push((base.x+x,base.z+z));
        if index > 0 && let Some(lane) = lanes.next() {
            slots.push(lane);
        }
    }
    slots.extend(lanes);
    slots
}

pub fn spawn_in_base(teams: usize, team: Team, n: usize) -> Option<(f32, f32)> {
    spawn_in_base_variant(teams, 0, team, n)
}

pub fn spawn_in_base_variant(teams: usize, variant: usize, team: Team, n: usize) -> Option<(f32, f32)> {
    let b = base_of_variant(teams, variant, team)?;
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

pub fn guard_post(base: BaseDef) -> V3 {
    let centre = V3::new(base.x, 0.0, base.z);
    let approach = (gate(base) - centre).normalised();
    centre + approach * 1.7 + V3::new(-approach.z, 0.0, approach.x) * 1.65
}

/// What one bot should be going for (`heistBotGoal`), in priority order: bring a stolen teddy
/// home; chase someone running off with one of ours; touch one of ours that got dropped; guard
/// (the first bot of a team of two or more); otherwise raid the nearest enemy teddy.
/// The guard watches the doorway from an offset position inside the base.
pub fn bot_goal(
    me: &Who,
    hands_free: bool,
    teams: usize,
    people: &[Who],
    teddies: &[Stray],
) -> Option<V3> {
    bot_goal_variant(me, hands_free, teams, 0, people, teddies)
}

pub fn bot_goal_variant(
    me: &Who,
    hands_free: bool,
    teams: usize,
    variant: usize,
    people: &[Who],
    teddies: &[Stray],
) -> Option<V3> {
    let bases = layout_for(teams, variant);
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
            return Some(via(me.pos, home, guard_post(home)));
        }
        // otherwise raid: the nearest enemy teddy
        let guard = || via(me.pos, home, guard_post(home));
        if !hands_free {
            return Some(guard());
        }
        let mut best: Option<(&Stray, f32)> = None;
        for t in teddies {
            if t.team == me.team || !t.on_ground {
                continue;
            }
            let sector = team_keys(teams).iter().position(|team| *team == t.team).unwrap_or(0);
            let preferred = (me.id as usize + sector) % teams == 0;
            let d = t.pos.horiz_dist(me.pos) + if preferred { 0.0 } else { 9.0 };
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((t, d));
            }
        }
        let Some((t, _)) = best else { return Some(guard()); };
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
        // the historical smoko spot near the far side of the yard remains away from all bases
        for b in layout(2) {
            assert!(((crate::yard::SMOKO_X - b.x).abs() - BASE_HALF).max((crate::yard::SMOKO_Z - b.z).abs() - BASE_HALF) > 4.5);
        }
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
        let c = ARENA_A * 0.875;
        assert_eq!((layout(2)[1].x, layout(2)[1].z), (c,c));
        assert_eq!((layout(3)[2].x, layout(3)[2].z), (-c,c));
        assert_eq!((layout(4)[3].x, layout(4)[3].z), (c,c));
    }

    #[test]
    fn touching_teddies() {
        assert_eq!(touch_teddy(Team::Red, Team::Blue, 0.5), Touch::Steal);
        assert_eq!(touch_teddy(Team::Red, Team::Red, 1.0), Touch::Nothing);
        assert_eq!(touch_teddy(Team::Red, Team::Red, 9.0), Touch::SendHome);
    }

    #[test]
    fn banking_rules() {
        assert!(can_bank(Team::Red, Team::Blue, 2.4, false, false));
        assert!(
            !can_bank(Team::Red, Team::Blue, BASE_RADIUS, false, false),
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
    fn square_reference_has_a_closed_perimeter() {
        for n in 2..=4 {
            let yard = crate::yard::Yard::heist(Default::default(), 0, n);
            for i in -58..=58 {
                for sign in [-1.0,1.0] {
                    assert!(!yard.heist_clear(sign*ARENA_A,i as f32,0.45));
                    assert!(!yard.heist_clear(i as f32,sign*ARENA_B,0.45));
                }
            }
        }
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
        let red = layout(2)[0];
        assert!(!blocked(red.x+4.2, red.z), "the doorway");
        assert!(blocked(red.x-3.0, red.z), "the back wall");
    }

    #[test]
    fn no_straight_run_between_any_pair_of_goals() {
        // Measure walkable routes on a one-metre grid, allowing diagonal travel.
        use std::cmp::Reverse;
        use std::collections::{BinaryHeap, HashMap};
        for teams in 2..=4 {
            for variant in 0..LAYOUTS_PER_TEAM_COUNT {
                let yard = crate::yard::Yard::heist_variant(Default::default(), 0, teams, variant);
                let bases = layout_for(teams, variant);
                for (index, from) in bases.iter().enumerate().take(teams - 1) {
                    let start = (from.x as i32, from.z as i32);
                    let mut distances = HashMap::from([(start, 0)]);
                    let mut queue = BinaryHeap::from([Reverse((0, start))]);
                    while let Some(Reverse((cost, (x, z)))) = queue.pop() {
                        if cost > distances[&(x, z)] { continue; }
                        for dx in -1i32..=1 {
                            for dz in -1i32..=1 {
                                if dx == 0 && dz == 0 { continue; }
                                let next = (x + dx, z + dz);
                                let diagonal = dx != 0 && dz != 0;
                                let next_cost = cost + if diagonal { 1414 } else { 1000 };
                                if next_cost >= *distances.get(&next).unwrap_or(&i32::MAX) { continue; }
                                if !yard.heist_clear(next.0 as f32, next.1 as f32, 0.65)
                                    || !yard.heist_clear(x as f32 + dx as f32 * 0.5, z as f32 + dz as f32 * 0.5, 0.65) { continue; }
                                distances.insert(next, next_cost);
                                queue.push(Reverse((next_cost, next)));
                            }
                        }
                    }
                    for to in &bases[index + 1..] {
                        let distance = distances[&(to.x as i32, to.z as i32)] as f32 / 1000.0;
                        let straight = (from.x - to.x).hypot(from.z - to.z);
                        assert!(distance >= straight + 18.0, "teams={teams} variant={variant}: {distance:.1} m walk for {straight:.1} m straight");
                        eprintln!("teams {teams}, from {index} to ({}, {}): {distance:.1} m", to.x, to.z);
                    }
                }
            }
        }
    }

    #[test]
    fn every_layout_has_thin_walls_and_connected_clear_routes() {
        let walls: Vec<_> = arena(2)
            .into_iter()
            .filter(|p| matches!(p.tint, Tint::Maze(_)))
            .collect();
        assert!(walls.len() >= 34);
        assert!(walls.iter().all(|p| p.collider.h >= 3.2));
        assert!(walls.iter().all(|p| {
            let c = p.collider;
            (c.x1-c.x0).min(c.z1-c.z0) < 0.33
        }));
        for n in 2..=4 {
          for variant in 0..LAYOUTS_PER_TEAM_COUNT {
            let yard = crate::yard::Yard::heist_variant(Default::default(), 0, n, variant);
            let walls: Vec<_> = arena_for(n, variant).into_iter().filter(|p| matches!(p.tint, Tint::Maze(_))).collect();
            let clear = |x: f32, z: f32| !walls.iter().any(|p| {
                x > p.collider.x0 - 0.65 && x < p.collider.x1 + 0.65 && z > p.collider.z0 - 0.65 && z < p.collider.z1 + 0.65
            });
            assert!(walls.len() >= 34, "teams={n} layout={variant} lost too many returns");
            let bases = layout_for(n, variant);
            // Flood a 1 m grid with 0.65 m clearance, wider than a player's collision radius.
            let cell = |base: BaseDef| (base.x.round() as i32, base.z.round() as i32);
            let start = cell(bases[0]);
            let mut reached = std::collections::HashSet::from([start]);
            let mut queue = std::collections::VecDeque::from([start]);
            while let Some((x,z)) = queue.pop_front() {
                for (dx,dz) in [(1,0),(-1,0),(0,1),(0,-1)] {
                    let next = (x+dx,z+dz);
                    if yard.heist_clear(next.0 as f32, next.1 as f32, 0.65)
                        && reached.insert(next) {
                        queue.push_back(next);
                    }
                }
            }
            for x in -58..=58 {
                for z in -58..=58 {
                    assert!(!yard.heist_clear(x as f32, z as f32, 0.65) || reached.contains(&(x,z)),
                        "sealed floor pocket at ({x}, {z}): teams={n} layout={variant}");
                }
            }
            for base in bases {
                assert!(reached.contains(&cell(base)), "disconnected base: teams={n} layout={variant} base={base:?}");
                assert!(yard.heist_clear(base.x, base.z, 0.45), "base interior blocked: teams={n} layout={variant} base=({}, {})", base.x, base.z);
                assert!(BASE_RADIUS + WALL_THICKNESS * 0.5 < BASE_HALF,
                    "the banking circle must stay inside the walls");
                let g = gate(base);
                assert!(yard.heist_clear(g.x, g.z, 0.45) && clear(g.x,g.z), "base doorway blocked: teams={n} layout={variant}");
                for &(ox, oz) in &SPAWN_OFFSETS {
                    assert!(yard.heist_clear(base.x + ox, base.z + oz, 0.4), "spawn blocked: teams={n} layout={variant}");
                }
                for p in &walls {
                    let dx = if base.x < p.collider.x0 {
                        p.collider.x0 - base.x
                    } else if base.x > p.collider.x1 {
                        base.x - p.collider.x1
                    } else {
                        0.0
                    };
                    let dz = if base.z < p.collider.z0 {
                        p.collider.z0 - base.z
                    } else if base.z > p.collider.z1 {
                        base.z - p.collider.z1
                    } else {
                        0.0
                    };
                    assert!(
                        dx.hypot(dz) > BASE_RADIUS,
                        "wall intrudes into banking zone: teams={n} layout={variant}, base=({}, {}), wall=({}, {})",
                        base.x,
                        base.z,
                        p.collider.x0,
                        p.collider.z0
                    );
                }
                for k in 0..2 {
                    let (tx, tz) = teddy_spot(base, k).unwrap();
                    assert!(yard.heist_clear(tx, tz, 0.4), "teddy spawn blocked");
                }
            }
          }
        }
    }

    #[test]
    fn spawn_slots_spread_teams_across_clear_approaches() {
        for n in 2..=4 {
            let yard = crate::yard::Yard::heist(Default::default(), 0, n);
            for &team in team_keys(n) {
                let base = base_of(n, team).unwrap();
                let slots = spawn_slots(&yard, team);
                assert!(slots.len() > SPAWN_OFFSETS.len(), "{n} flags, {team:?}");
                assert!(slots.iter().take(5).any(|&(x,z)| !inside_base(base, x, z)));
                for (i, &(x,z)) in slots.iter().enumerate() {
                    assert!(yard.heist_clear(x, z, 0.45), "blocked {n} {team:?} ({x},{z})");
                    assert!(slots[..i].iter().all(|&(a,b)| (a-x).hypot(b-z) >= 0.95));
                }
            }
        }
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
            spawn_in_base(2, Team::Red, SPAWN_OFFSETS.len() + 1),
            spawn_in_base(2, Team::Red, 1)
        );
    }

    #[test]
    fn the_doorway_is_just_outside_the_open_side() {
        let b = base_of(2, Team::Red).unwrap();
        assert_eq!(gate(b), V3::new(b.x + 4.2, 0.0, b.z));
        let b = base_of(3, Team::Green).unwrap();
        assert_eq!(gate(b), V3::new(b.x + 4.2, 0.0, b.z));
    }

    #[test]
    fn going_in_means_the_doorway_first() {
        let b = base_of(2, Team::Red).unwrap();
        let target = V3::new(red().x, 0.0, red().z);
        // from far away: the doorway
        assert_eq!(via(V3::new(0.0, 0.0, 10.0), b, target), gate(b));
        // standing at the doorway already: straight in
        assert_eq!(via(gate(b), b, target), target);
        // already inside: straight to it
        assert_eq!(via(V3::new(red().x+1.0, 0.0, red().z+1.0), b, target), target);
    }

    #[test]
    fn leaving_a_base_means_the_doorway_too() {
        let bases = layout(2);
        let inside = V3::new(red().x-1.0, 0.0, red().z+1.0);
        let out = V3::new(0.0, 0.0, 5.0);
        assert_eq!(leave(inside, out, &bases), gate(bases[0]));
        // wanting something in the same base: no detour
        assert_eq!(
            leave(inside, V3::new(red().x+1.0, 0.0, red().z), &bases),
            V3::new(red().x+1.0, 0.0, red().z)
        );
        // outside already: no detour
        assert_eq!(
            leave(out, V3::new(10.0, 0.0, 0.0), &bases),
            V3::new(10.0, 0.0, 0.0)
        );
    }

    fn red() -> BaseDef { layout(2)[0] }

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
        let g = bot_goal(&me, false, 2, std::slice::from_ref(&me), &[]).unwrap();
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
        let g = bot_goal(
            &me,
            true,
            2,
            &[me.clone(), thief.clone(), far],
            &[],
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
        let g = bot_goal(&me, true, 2, std::slice::from_ref(&me), &strays).unwrap();
        assert_eq!(g, V3::new(6.0, 0.2, 2.0));
    }

    #[test]
    fn with_a_mate_the_first_bot_guards_and_the_other_raids() {
        let a = who(1, red().x, red().z+1.0, Team::Red, true);
        let b = who(2, red().x+1.0, red().z+1.0, Team::Red, true);
        let strays = [Stray {
            team: Team::Blue,
            pos: V3::new(25.0, 0.2, 0.0),
            on_ground: true,
        }];
        let people = [a.clone(), b.clone()];
        // bot 1 holds a doorway post inside its own base
        let ga = bot_goal(&a, true, 2, &people, &strays).unwrap();
        assert!(ga.horiz_dist(V3::new(red().x, 0.0, red().z)) < 4.5, "{ga:?}");
        // bot 2 leaves by its doorway before raiding.
        let gb = bot_goal(&b, true, 2, &people, &strays).unwrap();
        assert_eq!(gb, gate(base_of(2, Team::Red).unwrap()));
    }

    #[test]
    fn a_guard_keeps_its_doorway_post_when_an_intruder_passes() {
        let guard = who(1, red().x, red().z, Team::Red, true);
        let mate = who(2, red().x - 1.0, red().z - 1.0, Team::Red, false);
        let intruder = who(3, red().x, red().z + 6.0, Team::Blue, false);
        let post = bot_goal(&guard, true, 2, &[guard.clone(), mate.clone(), intruder], &[])
            .unwrap();
        assert!(post.horiz_dist(V3::new(red().x, 0.0, red().z)) > 2.0);
        let on_post = Who { pos: post, ..guard };
        assert_eq!(bot_goal(&on_post, true, 2, &[on_post.clone(), mate], &[]), Some(post));
    }

    #[test]
    fn a_lone_bot_raids_or_heads_home_to_guard_when_its_hands_are_full() {
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
        // the nearest on-the-ground enemy teddy that is not in a base
        let g = bot_goal(&me, true, 2, std::slice::from_ref(&me), &strays).unwrap();
        assert_eq!(g, V3::new(8.0, 0.2, 0.0));
        assert_eq!(
            bot_goal(&me, false, 2, std::slice::from_ref(&me), &strays),
            Some(via(me.pos, base_of(2, Team::Red).unwrap(), guard_post(base_of(2, Team::Red).unwrap())))
        );
        // someone with no base (not a team in this game) has no goal
        let alien = who(9, 0.0, 0.0, Team::Green, true);
        assert_eq!(
            bot_goal(&alien, true, 2, std::slice::from_ref(&alien), &strays),
            None
        );
    }

    #[test]
    fn a_bot_inside_its_own_base_leaves_by_the_door_when_raiding() {
        let me = who(1, red().x, red().z+1.0, Team::Red, true);
        let strays = [Stray {
            team: Team::Blue,
            pos: V3::new(2.0, 0.2, 6.0),
            on_ground: true,
        }];
        let g = bot_goal(&me, true, 2, std::slice::from_ref(&me), &strays).unwrap();
        assert_eq!(g, gate(base_of(2, Team::Red).unwrap()));
    }
}
