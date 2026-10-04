//! The game state that sits between the rules (`bbq_core`) and the pictures: the items, what
//! the player holds, practice dummies to hit, and the score. Runs at a fixed 60 Hz.

use bbq_core::flight::ItemState;
use bbq_core::hands::{self, Picker, Press, Release, Situation, Slots, Wind};
use bbq_core::hitting::{self, Catcher, Context, Target};
use bbq_core::items::{ItemKind, Melee};
use bbq_core::itemworld::{ItemWorld, WorldEvent, no_walls};
use bbq_core::movement::{EYE_HEIGHT, Modifiers, MoveInput, Mover};
use bbq_core::rng::Rng;
use bbq_core::scoring::{HitInput, Phase, Rules, Scoreboard};
use bbq_core::sim::TICK_DT;
use bbq_core::stun::Body;
use bbq_core::teams::Teams;
use bbq_core::vec::V3;
use bbq_core::{GameMode, PlayerId};
use bevy::prelude::*;

use crate::player::{PLAYER_ID, Player, Wanted};
use crate::yard_scene::YardRes;

/// A stand-in person to throw things at until real bots arrive in Phase 5.
pub struct Dummy {
    pub id: PlayerId,
    pub mover: Mover,
    pub body: Body,
    pub home: (f32, f32),
    pub anim: bbq_core::pose::Animator,
    /// Viewer settings so you can look at each pose (keys listed on screen).
    pub drunk: f32,
    pub fallen: bool,
    pub crown: bool,
    pub team: Option<bbq_core::teams::Team>,
    pub smelly: bool,
}

#[derive(Resource)]
pub struct Game {
    pub world: ItemWorld,
    pub slots: Slots,
    pub wind: Wind,
    pub catcher: Catcher,
    pub board: Scoreboard,
    pub rules: Rules,
    pub teams: Teams,
    pub rng: Rng,
    pub now: f32,
    pub dummies: Vec<Dummy>,
    /// Messages for the on-screen feed, with the time they appeared.
    pub feed: Vec<(String, f32)>,
}

impl Game {
    pub fn say(&mut self, s: impl Into<String>) {
        self.feed.push((s.into(), self.now));
        if self.feed.len() > 30 {
            self.feed.remove(0);
        }
    }
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let mut rng = Rng::new(0x5EED);
        let mut world = ItemWorld::new();
        world.initial_fill(2, &no_walls, &mut rng);
        let mut board = Scoreboard::new();
        board.ensure(PLAYER_ID);
        let spots = [(8.0, 6.0), (-6.0, 8.0), (4.0, -8.0)];
        let dummies: Vec<Dummy> = spots
            .iter()
            .enumerate()
            .map(|(i, s)| {
                board.ensure(100 + i as u32);
                Dummy {
                    id: 100 + i as u32,
                    mover: Mover::new(s.0, s.1),
                    body: Body::default(),
                    home: *s,
                    anim: bbq_core::pose::Animator::new(i as f32 * 2.1),
                    drunk: 0.0,
                    fallen: false,
                    crown: false,
                    team: None,
                    smelly: false,
                }
            })
            .collect();
        app.insert_resource(Game {
            world,
            slots: Slots::default(),
            wind: Wind::default(),
            catcher: Catcher::default(),
            board,
            rules: Rules {
                mode: GameMode::FreeForAll,
                friendly_fire: false,
                phase: Phase::Play,
            },
            teams: Teams::default(),
            rng,
            now: 0.0,
            dummies,
            feed: Vec::new(),
        })
        .add_systems(FixedUpdate, step_game);
    }
}

fn eye_of(p: &Player) -> V3 {
    V3::new(p.mover.x, p.mover.y + EYE_HEIGHT - p.mover.sink, p.mover.z)
}

/// Where the camera looks (unit vector).
pub fn aim_dir(yaw: f32, pitch: f32) -> V3 {
    V3::new(
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    )
}

/// The right hand in world space (camera-space offset turned by the camera's rotation).
pub fn hand_pos(p: &Player) -> V3 {
    let q = Quat::from_euler(EulerRot::YXZ, p.yaw, p.pitch, 0.0);
    let h = q * Vec3::new(
        hands::THROW_HAND.x,
        hands::THROW_HAND.y,
        hands::THROW_HAND.z,
    );
    eye_of(p) + V3::new(h.x, h.y, h.z)
}

fn player_target_facing(p: &Player) -> V3 {
    V3::new(-p.yaw.sin(), 0.0, -p.yaw.cos())
}

pub fn step_game(
    mut g: ResMut<Game>,
    mut player: ResMut<Player>,
    mut wanted: ResMut<Wanted>,
    yard: Res<YardRes>,
) {
    let dt = TICK_DT;
    g.now += dt;
    let now = g.now;
    let g = &mut *g;
    let p = &mut *player;

    // ---- the player's hands ----
    g.catcher.tick(dt);
    if wanted.swap != 0 {
        if g.slots.swap(wanted.swap) {
            g.wind.cancel();
        }
        wanted.swap = 0;
    }
    if let Some(s) = wanted.slot.take()
        && g.slots.select_slot(s)
    {
        g.wind.cancel();
    }
    let selected_kind = g
        .slots
        .selected()
        .and_then(|id| g.world.items.get(&id))
        .map(|i| i.kind);
    let situation = Situation {
        can_act: g.rules.in_play(),
        ..Default::default()
    };

    if std::mem::take(&mut wanted.throw_down) {
        match g.wind.press(now, selected_kind, &situation) {
            Press::Charging => {}
            Press::SlapNow => g.say("(melee slaps come with Dazza and bots in later phases)"),
            Press::Refused(hands::Refuse::NothingHeld) => {
                g.say("Nothing to chuck. Walk over something glowing.")
            }
            Press::Refused(_) => {}
        }
    }
    if g.wind.tick(dt, selected_kind)
        && let Some(id) = g.slots.selected()
    {
        // held it too long: drop it, and you can't pick it up again for 2.5 s
        let pos = eye_of(p);
        g.slots.remove(id);
        g.world
            .drop_item(id, pos, Some((PLAYER_ID, now + hands::OVERHOLD_BLOCK)));
        g.say("Held it too long! You dropped it.");
    }
    if std::mem::take(&mut wanted.throw_up) {
        let kind = g
            .slots
            .selected()
            .and_then(|id| g.world.items.get(&id))
            .map(|i| i.kind);
        match g.wind.release(now, kind, situation.can_act) {
            Release::Throw { charge } => {
                if let (Some(id), Some(kind)) = (g.slots.selected(), kind) {
                    let vel = hands::throw_velocity(
                        aim_dir(p.yaw, p.pitch),
                        kind.def().speed,
                        charge,
                        V3::new(p.mover.vx, 0.0, p.mover.vz),
                    );
                    let start = hand_pos(p);
                    if g.world.throw(id, PLAYER_ID, start, vel, charge, true) {
                        g.slots.remove(id);
                        g.board.count_throw(PLAYER_ID);
                    }
                }
            }
            Release::Slap | Release::Nothing => {}
        }
    }
    if std::mem::take(&mut wanted.catch) {
        let can = g.rules.in_play();
        g.catcher.press(can);
    }

    // ---- pick things up ----
    let picker = Picker {
        id: PLAYER_ID,
        pos: V3::new(p.mover.x, p.mover.y, p.mover.z),
        held: g.slots.len(),
        stunned: false,
        frozen: false,
        is_bot: false,
    };
    if let Some(id) = g.world.pickup_for(&picker, now) {
        g.world.give(id, PLAYER_ID);
        g.slots.add(id);
    }

    // ---- the dummies stand about and get knocked over ----
    for d in &mut g.dummies {
        d.body.tick(dt);
        let mods = Modifiers {
            stunned: d.body.stun > 0.0,
            ..Default::default()
        };
        d.mover.step(dt, MoveInput::default(), &mods, &yard.0);
    }

    // ---- items fly, hit and get caught ----
    let mut targets: Vec<Target> = g
        .dummies
        .iter()
        .map(|d| Target {
            id: d.id,
            pos: V3::new(d.mover.x, d.mover.y, d.mover.z),
            sink: d.mover.sink,
            at_smoko: false,
            catching: false,
            stunned: d.body.stun > 0.0,
            facing: V3::new(0.0, 0.0, 1.0),
            flattenable: d.body.power_throw_flattens(),
        })
        .collect();
    targets.push(Target {
        id: PLAYER_ID,
        pos: V3::new(p.mover.x, p.mover.y, p.mover.z),
        sink: p.mover.sink,
        at_smoko: false,
        catching: g.catcher.open(),
        stunned: false,
        facing: player_target_facing(p),
        flattenable: false, // nobody throws at you in solo
    });
    let teams = g.teams.clone();
    let ctx = Context {
        friendly_fire: false,
        teams: &teams,
    };
    let events = g.world.step(
        dt,
        &yard.0,
        &mut g.rng,
        &targets,
        &ctx,
        2 + g.dummies.len(),
        true,
        &no_walls,
    );

    for ev in events {
        match ev {
            WorldEvent::Hit {
                kind,
                victim,
                thrower,
                dir,
                flatten,
                charge,
                start,
                ..
            } => {
                let Some(di) = g.dummies.iter().position(|d| d.id == victim) else {
                    continue;
                };
                let (vx, vz) = (g.dummies[di].mover.x, g.dummies[di].mover.z);
                let leader = g.board.leader() == Some(victim);
                // knock and stun
                let item = bbq_core::flight::Item::new(0, kind, V3::ZERO);
                let d = &mut g.dummies[di];
                let res = hitting::apply_item_hit(&mut d.body, &mut d.mover, &item, dir, flatten);
                let sign = if g.rng.chance(0.5) { -1.0 } else { 1.0 };
                d.anim.tumble(dir, kind.def().knock, sign);
                let out = g.board.thrown_hit(
                    &g.rules,
                    thrower,
                    victim,
                    &HitInput {
                        charge,
                        dist: start.horiz_dist(V3::new(vx, 0.0, vz)),
                        victim_is_leader: leader,
                        drunk_bonus: 0,
                        bum_out: false,
                        same_team: false,
                    },
                );
                let mut msg = format!("HIT! +{}", out.gain);
                if out.long_shot > 0 {
                    msg += &format!(" (long shot +{})", out.long_shot);
                }
                if out.streak >= 3 {
                    msg += &format!(" streak x{}", out.streak);
                }
                if flatten && res.effect.knocked_down {
                    msg += " KNOCKED OVER!";
                }
                if !res.effect.stunned && !res.effect.knocked_down {
                    msg += " (already stunned, no new stun)";
                }
                g.say(msg);
            }
            WorldEvent::Caught { by, thrower, .. } if by == PLAYER_ID => {
                // the item is now in our hands; if full, drop the oldest
                if let Some(id) = caught_item(&g.world, PLAYER_ID, &g.slots) {
                    if g.slots.is_full()
                        && let Some(old) = g.slots.oldest()
                    {
                        g.slots.remove(old);
                        g.world.drop_item(old, eye_of(p), None);
                    }
                    g.slots.add(id);
                }
                g.board.catch(&g.rules, PLAYER_ID, thrower);
                g.say("CAUGHT! +50");
            }
            WorldEvent::Flight {
                ev: bbq_core::flight::FlightEvent::Over { thrower: Some(t) },
                ..
            } if t == PLAYER_ID => {
                g.say("Over the fence!");
            }
            _ => {}
        }
    }

    // dummies that wandered off (knocked) are put back after a bit once they've settled
    for d in &mut g.dummies {
        if !d.body.is_down() && d.body.stun <= 0.0 && d.mover.grounded && d.mover.speed() < 0.3 {
            let (hx, hz) = d.home;
            let dist = ((d.mover.x - hx).powi(2) + (d.mover.z - hz).powi(2)).sqrt();
            if dist > 0.01 {
                d.mover.x += (hx - d.mover.x) * 0.02;
                d.mover.z += (hz - d.mover.z) * 0.02;
            }
        }
    }

    // keep the feed tidy
    g.feed.retain(|(_, t)| now - *t < 6.0);
}

fn caught_item(
    world: &ItemWorld,
    who: PlayerId,
    slots: &Slots,
) -> Option<bbq_core::flight::ItemId> {
    world
        .items
        .values()
        .find(|i| {
            i.state == ItemState::Held && i.holder == Some(who) && !slots.ids().contains(&i.id)
        })
        .map(|i| i.id)
}

/// Which kinds can't be thrown (melee-only) — used to hint in the HUD.
#[allow(dead_code)]
pub fn is_melee_only(kind: ItemKind) -> bool {
    let d = kind.def();
    d.melee != Melee::None && !d.throwable
}

#[cfg(test)]
mod tests {
    use super::*;
    use bbq_core::movement::FOV_DEFAULT;

    /// A tiny app with just the game rules in it (no window, no graphics).
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(YardRes(bbq_core::yard::Yard::default()));
        app.insert_resource(Player {
            mover: Mover::new(8.0, 0.0),
            prev: Vec3::ZERO,
            yaw: std::f32::consts::PI, // looking along +z
            pitch: 0.05,
            walk: 0.0,
            shake: 0.0,
            fov_base: FOV_DEFAULT,
            fov: FOV_DEFAULT,
            rng: Rng::new(1),
        });
        app.init_resource::<Wanted>();
        app.add_plugins(GamePlugin);
        // clear the random items so the test is predictable, keep only what we add
        app.world_mut().resource_mut::<Game>().world.clear();
        app
    }

    fn ticks(app: &mut App, n: usize) {
        for _ in 0..n {
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn give_teddy(app: &mut App) {
        let mut g = app.world_mut().resource_mut::<Game>();
        let mut rng = Rng::new(9);
        let id = g.world.spawn(ItemKind::Teddy, 8.0, 0.0, false, &mut rng);
        g.world.give(id, PLAYER_ID);
        g.slots.add(id);
    }

    #[test]
    fn walking_over_an_item_picks_it_up() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let mut rng = Rng::new(3);
            g.world.spawn(ItemKind::Gnome, 8.3, 0.2, false, &mut rng);
        }
        ticks(&mut app, 120); // it lands and rests
        let g = app.world().resource::<Game>();
        assert_eq!(g.slots.len(), 1, "should have picked the gnome up");
    }

    #[test]
    fn a_full_throw_hits_a_dummy_and_scores() {
        let mut app = app();
        give_teddy(&mut app);
        // dummy 0 is at (8, 6): in front of us
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 1);
        ticks(&mut app, 40); // wind up past full charge but under the 1.5 s over-hold
        app.world_mut().resource_mut::<Wanted>().throw_up = true;
        ticks(&mut app, 90);
        let g = app.world().resource::<Game>();
        let me = g.board.get(PLAYER_ID).unwrap();
        assert!(me.score >= 100, "score {} feed {:?}", me.score, g.feed);
        assert_eq!(me.hits, 1);
        assert_eq!(me.throws, 1);
        assert!(g.slots.is_empty());
        let d = &g.dummies[0];
        assert!(d.body.stun > 0.0 || d.body.stun_grace > 0.0 || d.body.is_down());
        assert_eq!(g.board.score(d.id), -50);
    }

    #[test]
    fn holding_too_long_drops_the_item() {
        let mut app = app();
        give_teddy(&mut app);
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 60 * 3);
        let g = app.world().resource::<Game>();
        assert!(g.slots.is_empty(), "item should have been dropped");
        assert!(g.feed.iter().any(|(s, _)| s.contains("too long")));
        assert!(!g.wind.charging);
    }

    #[test]
    fn a_missed_throw_just_lands() {
        let mut app = app();
        give_teddy(&mut app);
        app.world_mut().resource_mut::<Player>().yaw = 0.0; // looking the other way, nothing there
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 20);
        app.world_mut().resource_mut::<Wanted>().throw_up = true;
        ticks(&mut app, 180);
        let g = app.world().resource::<Game>();
        assert_eq!(g.board.get(PLAYER_ID).unwrap().hits, 0);
        assert_eq!(g.board.get(PLAYER_ID).unwrap().throws, 1);
        assert!(g.world.items.values().all(|i| !i.live));
    }

    #[test]
    fn swapping_between_two_items_selects_the_other() {
        let mut app = app();
        give_teddy(&mut app);
        give_teddy(&mut app);
        let before = app.world().resource::<Game>().slots.selected();
        app.world_mut().resource_mut::<Wanted>().swap = 1;
        ticks(&mut app, 1);
        assert_ne!(app.world().resource::<Game>().slots.selected(), before);
    }
}
