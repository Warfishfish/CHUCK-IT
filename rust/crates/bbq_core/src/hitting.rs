//! A thrown item meets a person: pass-through, catch or hit; knockback, stun and points
//! (spec section 3, "Hit detection" and "What a hit does").

use crate::PlayerId;
use crate::flight::Item;
use crate::movement::Mover;
use crate::scoring::{HitInput, HitOutcome, Rules, Scoreboard};
use crate::stun::{Body, HitEffect, POWER_DOWN};
use crate::teams::Teams;
use crate::vec::V3;

/// Touch distance is this plus the item's radius (horizontally).
pub const BODY_RADIUS: f32 = 0.45;
/// How tall a person is for hit purposes.
pub const BODY_HEIGHT: f32 = 1.85;
/// Catch window and cooldown (spec section 2).
pub const CATCH_WINDOW: f32 = 0.32;
pub const CATCH_COOLDOWN: f32 = 0.9;
/// A catch needs the item to be in front of you: `dot(facing, toItem) > 0.1`.
pub const CATCH_FACING: f32 = 0.1;

/// Someone an item might hit.
#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub id: PlayerId,
    /// Feet position.
    pub pos: V3,
    /// How far they've sunk in the pool.
    pub sink: f32,
    pub at_smoko: bool,
    pub catching: bool,
    pub stunned: bool,
    /// Which way they're facing (horizontal, unit length).
    pub facing: V3,
    /// Power throws knock them flat (not in grace, down or fallen).
    pub flattenable: bool,
}

/// Things that decide whether a throw can hit at all.
#[derive(Clone, Copy)]
pub struct Context<'a> {
    pub friendly_fire: bool,
    pub teams: &'a Teams,
}

/// What happened when an item reached a person.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Contact {
    /// Not touching.
    Miss,
    /// Touching but it passes straight through (smoko, teammate). Remember not to test again.
    PassThrough,
    Catch,
    Hit {
        /// Horizontal direction of the knock (unit length).
        dir: V3,
        /// A power throw that knocks them flat.
        flatten: bool,
    },
}

/// Is the item touching this person? (Horizontal distance and height window.)
pub fn touching(item: &Item, t: &Target) -> bool {
    let rr = BODY_RADIUS + item.radius();
    let (dx, dz) = (item.pos.x - t.pos.x, item.pos.z - t.pos.z);
    if dx * dx + dz * dz > rr * rr {
        return false;
    }
    let y = item.pos.y - t.pos.y + t.sink;
    !(y < -item.radius() || y > BODY_HEIGHT + item.radius())
}

/// Decide what the item does to this person. The caller adds `t.id` to `item.ignore` on
/// `PassThrough`.
pub fn contact(item: &Item, t: &Target, ctx: &Context) -> Contact {
    if item.thrower == Some(t.id) || item.ignore.contains(&t.id) {
        return Contact::Miss;
    }
    if !touching(item, t) {
        return Contact::Miss;
    }
    if t.at_smoko {
        return Contact::PassThrough;
    }
    if let Some(th) = item.thrower
        && !ctx.friendly_fire
        && ctx.teams.same_team(th, t.id)
    {
        return Contact::PassThrough;
    }
    // a catch is checked before the hit
    if t.catching && !t.stunned {
        let to = V3::new(item.pos.x - t.pos.x, 0.0, item.pos.z - t.pos.z).normalised();
        if t.facing.dot(to) > CATCH_FACING {
            return Contact::Catch;
        }
    }
    let mut dir = V3::new(item.vel.x, 0.0, item.vel.z);
    if dir.len_sq() < 1e-4 {
        dir = V3::new(0.0, 0.0, 1.0);
    }
    Contact::Hit {
        dir: dir.normalised(),
        flatten: item.power && t.flattenable,
    }
}

/// The catch window state for one player.
#[derive(Clone, Copy, Debug, Default)]
pub struct Catcher {
    pub window: f32,
    pub cooldown: f32,
}

impl Catcher {
    pub fn tick(&mut self, dt: f32) {
        self.window = (self.window - dt).max(0.0);
        self.cooldown = (self.cooldown - dt).max(0.0);
    }

    /// Right-click. `can`: not stunned, down, fallen or at smoko.
    pub fn press(&mut self, can: bool) -> bool {
        if can && self.cooldown <= 0.0 {
            self.window = CATCH_WINDOW;
            self.cooldown = CATCH_COOLDOWN;
            true
        } else {
            false
        }
    }

    pub fn open(&self) -> bool {
        self.window > 0.0
    }
}

/// What a hit did to the victim's body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitResult {
    pub effect: HitEffect,
    /// Knockback added to the victim's velocity.
    pub knock: V3,
}

/// Push a mover: `vel += dir * knock`, and lift it to at least `min_up`, making it airborne.
pub fn knock_mover(m: &mut Mover, dir: V3, knock: f32, min_up: f32) {
    m.vx += dir.x * knock;
    m.vz += dir.z * knock;
    m.vy = m.vy.max(min_up);
    m.grounded = false;
}

/// Apply a thrown-item hit to a person: knockback (always) and stun (not if already stunned or
/// in grace). A power-throw flatten knocks them down for `POWER_DOWN` seconds.
pub fn apply_item_hit(
    body: &mut Body,
    mover: &mut Mover,
    item: &Item,
    dir: V3,
    flatten: bool,
) -> HitResult {
    let d = item.kind.def();
    let min_up = d.knock * 0.38;
    knock_mover(mover, dir, d.knock, min_up);
    let effect = body.apply_hit(item.kind.hit_stun(), flatten.then_some(POWER_DOWN));
    HitResult {
        effect,
        knock: dir * d.knock,
    }
}

/// What happens to the item after it hits someone: a stubby smashes, anything else loses its
/// "live" state and bounces away at `(-vx*0.2, 3.5, -vz*0.2)`. Returns true if it smashed.
pub fn item_after_hit(item: &mut Item) -> bool {
    if item.kind.def().smash {
        return true;
    }
    item.live = false;
    item.vel = V3::new(-item.vel.x * 0.2, 3.5, -item.vel.z * 0.2);
    false
}

/// Score a thrown hit. Returns what the thrower got.
#[allow(clippy::too_many_arguments)]
pub fn score_item_hit(
    board: &mut Scoreboard,
    rules: &Rules,
    ctx: &Context,
    item: &Item,
    victim: PlayerId,
    victim_pos: V3,
    victim_is_leader: bool,
    thrower_drunk_bonus: i32,
) -> HitOutcome {
    let same_team = item.thrower.is_some_and(|t| ctx.teams.same_team(t, victim));
    board.thrown_hit(
        rules,
        item.thrower,
        victim,
        &HitInput {
            charge: item.charge,
            dist: item.start.horiz_dist(victim_pos),
            victim_is_leader,
            drunk_bonus: thrower_drunk_bonus,
            bum_out: false,
            same_team,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GameMode;
    use crate::flight::ItemState;
    use crate::items::ItemKind;
    use crate::scoring::Phase;
    use crate::teams::Team;

    fn flying(kind: ItemKind, pos: V3, vel: V3, thrower: Option<PlayerId>) -> Item {
        let mut it = Item::new(1, kind, pos);
        it.state = ItemState::Flying;
        it.live = true;
        it.vel = vel;
        it.thrower = thrower;
        it
    }

    fn target(id: PlayerId, x: f32) -> Target {
        Target {
            id,
            pos: V3::new(x, 0.0, 0.0),
            sink: 0.0,
            at_smoko: false,
            catching: false,
            stunned: false,
            facing: V3::new(-1.0, 0.0, 0.0),
            flattenable: true,
        }
    }

    fn ctx(teams: &Teams, ff: bool) -> Context<'_> {
        Context {
            friendly_fire: ff,
            teams,
        }
    }

    #[test]
    fn touching_needs_to_be_close_and_at_body_height() {
        let t = target(2, 0.0);
        let near = flying(ItemKind::Teddy, V3::new(0.5, 1.0, 0.0), V3::ZERO, None);
        assert!(touching(&near, &t)); // 0.5 < 0.45 + 0.24
        let far = flying(ItemKind::Teddy, V3::new(0.8, 1.0, 0.0), V3::ZERO, None);
        assert!(!touching(&far, &t));
        let high = flying(ItemKind::Teddy, V3::new(0.0, 2.2, 0.0), V3::ZERO, None);
        assert!(!touching(&high, &t)); // above 1.85 + 0.24
        let low = flying(ItemKind::Teddy, V3::new(0.0, -0.3, 0.0), V3::ZERO, None);
        assert!(!touching(&low, &t));
        // sinking in the pool lets items pass over your head
        let mut swimmer = target(2, 0.0);
        swimmer.sink = 1.0;
        let over = flying(ItemKind::Teddy, V3::new(0.0, 0.4, 0.0), V3::ZERO, None);
        assert!(touching(&over, &swimmer));
    }

    #[test]
    fn you_cant_hit_yourself_or_someone_already_passed_through() {
        let teams = Teams::default();
        let mut it = flying(
            ItemKind::Teddy,
            V3::new(0.0, 1.0, 0.0),
            V3::new(0.0, 0.0, 5.0),
            Some(2),
        );
        assert_eq!(
            contact(&it, &target(2, 0.0), &ctx(&teams, false)),
            Contact::Miss
        );
        it.ignore.push(3);
        assert_eq!(
            contact(&it, &target(3, 0.0), &ctx(&teams, false)),
            Contact::Miss
        );
    }

    #[test]
    fn a_plain_hit_knocks_along_the_flight_direction() {
        let teams = Teams::default();
        let it = flying(
            ItemKind::Teddy,
            V3::new(0.0, 1.0, 0.0),
            V3::new(0.0, 4.0, -10.0),
            Some(1),
        );
        let c = contact(&it, &target(2, 0.0), &ctx(&teams, false));
        let Contact::Hit { dir, flatten } = c else {
            panic!("{c:?}")
        };
        assert!((dir.z + 1.0).abs() < 1e-5 && dir.y == 0.0);
        assert!(!flatten);
    }

    #[test]
    fn smoko_and_teammates_are_passed_through() {
        let mut teams = Teams::default();
        teams.set(1, Team::Red);
        teams.set(2, Team::Red);
        let it = flying(
            ItemKind::Teddy,
            V3::new(0.0, 1.0, 0.0),
            V3::new(0.0, 0.0, -10.0),
            Some(1),
        );
        assert_eq!(
            contact(&it, &target(2, 0.0), &ctx(&teams, false)),
            Contact::PassThrough
        );
        assert!(matches!(
            contact(&it, &target(2, 0.0), &ctx(&teams, true)),
            Contact::Hit { .. }
        ));
        let mut seated = target(3, 0.0);
        seated.at_smoko = true;
        assert_eq!(
            contact(&it, &seated, &ctx(&teams, false)),
            Contact::PassThrough
        );
    }

    #[test]
    fn catching_needs_the_window_not_stunned_and_facing_the_item() {
        let teams = Teams::default();
        // item arrives from +x; target faces -x? no: faces +x to see it
        let it = flying(
            ItemKind::Teddy,
            V3::new(0.5, 1.0, 0.0),
            V3::new(-10.0, 0.0, 0.0),
            Some(1),
        );
        let mut t = target(2, 0.0);
        t.catching = true;
        t.facing = V3::new(1.0, 0.0, 0.0);
        assert_eq!(contact(&it, &t, &ctx(&teams, false)), Contact::Catch);
        t.stunned = true;
        assert!(matches!(
            contact(&it, &t, &ctx(&teams, false)),
            Contact::Hit { .. }
        ));
        t.stunned = false;
        t.facing = V3::new(-1.0, 0.0, 0.0); // back turned
        assert!(matches!(
            contact(&it, &t, &ctx(&teams, false)),
            Contact::Hit { .. }
        ));
        t.facing = V3::new(1.0, 0.0, 0.0);
        t.catching = false;
        assert!(matches!(
            contact(&it, &t, &ctx(&teams, false)),
            Contact::Hit { .. }
        ));
    }

    #[test]
    fn a_power_throw_flattens_unless_protected() {
        let teams = Teams::default();
        let mut it = flying(
            ItemKind::Teddy,
            V3::new(0.0, 1.0, 0.0),
            V3::new(0.0, 0.0, -10.0),
            Some(1),
        );
        it.power = true;
        let Contact::Hit { flatten, .. } = contact(&it, &target(2, 0.0), &ctx(&teams, false))
        else {
            panic!()
        };
        assert!(flatten);
        let mut t = target(2, 0.0);
        t.flattenable = false;
        let Contact::Hit { flatten, .. } = contact(&it, &t, &ctx(&teams, false)) else {
            panic!()
        };
        assert!(!flatten);
    }

    #[test]
    fn catch_window_is_032_with_a_09_cooldown() {
        let mut c = Catcher::default();
        assert!(c.press(true));
        assert!(c.open());
        c.tick(0.33);
        assert!(!c.open());
        assert!(!c.press(true), "cooling down");
        c.tick(0.6);
        assert!(c.press(true));
        let mut d = Catcher::default();
        assert!(!d.press(false));
    }

    #[test]
    fn a_hit_shoves_and_stuns_but_stuns_never_stack() {
        let it = flying(
            ItemKind::Gnome,
            V3::new(0.0, 1.0, 0.0),
            V3::new(0.0, 0.0, -10.0),
            Some(1),
        );
        let dir = V3::new(0.0, 0.0, -1.0);
        let (mut b, mut m) = (Body::default(), Mover::new(0.0, 0.0));
        let r = apply_item_hit(&mut b, &mut m, &it, dir, false);
        assert!(r.effect.stunned);
        assert!((b.stun - 0.65).abs() < 1e-5);
        assert!((m.vz + 15.0).abs() < 1e-4);
        assert!((m.vy - 15.0 * 0.38).abs() < 1e-4);
        assert!(!m.grounded);
        // second hit while stunned: shoved again, no new stun
        let r2 = apply_item_hit(&mut b, &mut m, &it, dir, false);
        assert!(!r2.effect.stunned);
        assert!((m.vz + 30.0).abs() < 1e-3);
    }

    #[test]
    fn flatten_knocks_down_for_two_seconds() {
        let it = flying(
            ItemKind::Teddy,
            V3::new(0.0, 1.0, 0.0),
            V3::new(0.0, 0.0, -10.0),
            Some(1),
        );
        let (mut b, mut m) = (Body::default(), Mover::new(0.0, 0.0));
        let r = apply_item_hit(&mut b, &mut m, &it, V3::new(0.0, 0.0, -1.0), true);
        assert!(r.effect.knocked_down);
        assert_eq!(b.down_t, 2.0);
        assert!(b.is_down());
    }

    #[test]
    fn items_bounce_off_after_a_hit_and_stubbies_smash() {
        let mut it = flying(
            ItemKind::Teddy,
            V3::new(0.0, 1.0, 0.0),
            V3::new(10.0, -2.0, -5.0),
            Some(1),
        );
        assert!(!item_after_hit(&mut it));
        assert!(!it.live);
        assert_eq!(it.vel, V3::new(-2.0, 3.5, 1.0));
        let mut v = flying(ItemKind::Stubby, V3::ZERO, V3::new(1.0, 0.0, 0.0), Some(1));
        assert!(item_after_hit(&mut v));
    }

    fn board_with(ids: &[PlayerId]) -> Scoreboard {
        let mut b = Scoreboard::new();
        for i in ids {
            b.ensure(*i);
        }
        b
    }

    #[test]
    fn scoring_a_hit_uses_the_throw_charge_and_distance() {
        let teams = Teams::default();
        let rules = Rules {
            mode: GameMode::FreeForAll,
            friendly_fire: false,
            phase: Phase::Play,
        };
        let mut board = board_with(&[1, 2]);
        let mut it = flying(ItemKind::Teddy, V3::new(0.0, 1.0, 20.0), V3::ZERO, Some(1));
        it.charge = 1.0;
        it.start = V3::new(0.0, 1.5, 0.0);
        // 20 m away at full charge: (20-8)/17 * 100 / 5 -> 70 rounded to 5s = 70
        let o = score_item_hit(
            &mut board,
            &rules,
            &ctx(&teams, false),
            &it,
            2,
            V3::new(0.0, 0.0, 20.0),
            false,
            0,
        );
        assert_eq!(o.long_shot, 70);
        assert_eq!(board.score(1), 170);
        assert_eq!(board.score(2), -50);
    }

    #[test]
    fn heist_thrown_hits_score_nothing() {
        let teams = Teams::default();
        let rules = Rules {
            mode: GameMode::Heist,
            friendly_fire: false,
            phase: Phase::Play,
        };
        let mut board = board_with(&[1, 2]);
        let it = flying(ItemKind::Teddy, V3::new(0.0, 1.0, 0.0), V3::ZERO, Some(1));
        score_item_hit(
            &mut board,
            &rules,
            &ctx(&teams, false),
            &it,
            2,
            V3::ZERO,
            false,
            25,
        );
        assert_eq!(board.score(1), 0);
        assert_eq!(board.score(2), 0);
    }
}
