//! The game state that sits between the rules (`bbq_core`) and the pictures: the items, what
//! the player holds, practice dummies to hit, and the score. Runs at a fixed 60 Hz.

use bbq_core::drinks::{self, Drink};
use bbq_core::drunk_state::{self, DrunkState, Env, Event as DrunkEvent, HelpHold};
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

/// The player's own body: stuns and falls, the drunk meter, and helping mates up.
pub struct Me {
    pub body: Body,
    pub drunk: DrunkState,
    pub help: HelpHold,
}

/// Host switches that change how people fall (F5, F6 for now; the menu comes later).
pub struct Options {
    /// Drunk people can stack it.
    pub falls_on: bool,
    /// Drunk mode: everyone is held at 78 or more, and walking wobbles.
    pub drunk_mode: bool,
    /// How long a fall lasts (10 s; the host can pick 20 or 30).
    pub fall_duration: f32,
}

/// A line of big text in the middle of the screen that fades ("+18 drunk", "STACKED IT!").
pub struct Popup {
    pub text: String,
    pub big: bool,
    pub t: f32,
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
    pub me: Me,
    pub options: Options,
    pub popups: Vec<Popup>,
    /// The line at the bottom of the screen telling you what R does right now.
    pub prompt: String,
}

impl Game {
    pub fn popup(&mut self, text: impl Into<String>, big: bool) {
        self.popups.push(Popup {
            text: text.into(),
            big,
            t: 0.0,
        });
        if self.popups.len() > 4 {
            self.popups.remove(0);
        }
    }

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
        let me = Me {
            body: Body::default(),
            drunk: DrunkState::new(0.7, &mut rng),
            help: HelpHold::default(),
        };
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
            me,
            options: Options {
                falls_on: true,
                drunk_mode: false,
                fall_duration: drinks::FALL_DURATION_DEFAULT,
            },
            popups: Vec::new(),
            prompt: String::new(),
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

    // ---- the player's body: drinking, falling over, helping mates up ----
    let moving = wanted.wish.0 != 0.0 || wanted.wish.1 != 0.0;
    let env = Env {
        in_play: g.rules.in_play(),
        at_smoko: false,
        grounded: p.mover.grounded,
        in_pool: p.mover.in_pool,
        moving,
        falls_on: g.options.falls_on,
        drunk_mode: g.options.drunk_mode,
        can_fall: true,
        fall_duration: g.options.fall_duration,
    };
    let tick = g.me.body.tick(dt);
    let events =
        g.me.drunk
            .tick(dt, &env, &mut g.me.body, tick.fall_ended, &mut g.rng);
    for ev in events {
        match ev {
            DrunkEvent::DrankUp {
                added,
                tier,
                tier_changed,
                ..
            } => {
                if tier_changed {
                    g.popup(format!("{}!", tier_name(tier)), true);
                } else {
                    g.popup(format!("+{added:.0} drunk"), false);
                }
            }
            DrunkEvent::StackedIt => {
                p.mover.vx *= 0.3;
                p.mover.vz *= 0.3;
                p.shake = p.shake.max(0.15);
                g.popup("STACKED IT!", true);
            }
            DrunkEvent::GotUp => {}
        }
    }
    if g.me.body.fall_t > 0.0 || g.me.drunk.is_drinking() {
        g.wind.cancel(); // can't wind up a throw on the ground or with a drink in your hand
    }
    let can_use = env.in_play && g.me.body.stun <= 0.0;
    let help_target = if can_use {
        let mut best: Option<(usize, f32)> = None;
        for (i, d) in g.dummies.iter().enumerate() {
            if !drunk_state::can_help(false, d.body.fall_t > 0.0, false, false) {
                continue;
            }
            let dist = (d.mover.x - p.mover.x).hypot(d.mover.z - p.mover.z);
            if dist < drunk_state::HELP_REACH && best.is_none_or(|(_, b)| dist < b) {
                best = Some((i, dist));
            }
        }
        best.map(|b| b.0)
    } else {
        None
    };
    let help_done =
        g.me.help
            .tick(dt, help_target.is_some(), wanted.interact_held);
    if help_done && let Some(i) = help_target {
        g.dummies[i].body.get_up();
        let pts = g.board.award(&g.rules, PLAYER_ID, drunk_state::HELP_PTS);
        let name = crate::characters::BLOB_NAMES[i % 3];
        g.popup(format!("Helped {name} up! +{pts}"), false);
    }
    let bar_on = yard.0.features.bar;
    let spot = if can_use && bar_on {
        drunk_state::bar_spot(&bbq_core::yard::BAR, p.mover.x, p.mover.y, p.mover.z)
    } else {
        None
    };
    if std::mem::take(&mut wanted.interact_pressed)
        && help_target.is_none()
        && g.me.drunk.start_drink(spot, &g.me.body, &env)
    {
        g.wind.cancel();
    }
    g.prompt = if g.me.body.fall_t > 0.0 {
        format!(
            "Stacked it! Up in {}s (or a mate can help you up)",
            g.me.body.fall_t.ceil()
        )
    } else if let Some(i) = help_target {
        let name = crate::characters::BLOB_NAMES[i % 3];
        if g.me.help.t > 0.0 {
            format!("Helping {name} up... {:.0}%", g.me.help.progress() * 100.0)
        } else {
            format!("R: hold to help {name} up (+{})", drunk_state::HELP_PTS)
        }
    } else if g.me.drunk.is_drinking() {
        String::new()
    } else if let Some(d) = spot {
        format!("R: grab {} (+{:.0} drunk)", drink_name(d), d.amount())
    } else {
        String::new()
    };
    for pop in &mut g.popups {
        pop.t += dt;
    }
    g.popups.retain(|pop| pop.t < 1.6);

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
        can_act: g.rules.in_play() && g.me.body.fall_t <= 0.0,
        stunned_standing: g.me.body.stun > 0.0 && g.me.body.down_t <= 0.0,
        drinking: g.me.drunk.is_drinking(),
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
        let can = g.rules.in_play() && g.me.body.stun <= 0.0;
        g.catcher.press(can);
    }

    // ---- pick things up ----
    let picker = Picker {
        id: PLAYER_ID,
        pos: V3::new(p.mover.x, p.mover.y, p.mover.z),
        held: g.slots.len(),
        stunned: g.me.body.stun > 0.0,
        frozen: false,
        is_bot: false,
    };
    if let Some(id) = g.world.pickup_for(&picker, now) {
        g.world.give(id, PLAYER_ID);
        g.slots.add(id);
    }

    // ---- the dummies stand about and get knocked over ----
    let mut newly_fallen = Vec::new();
    for (i, d) in g.dummies.iter_mut().enumerate() {
        d.body.tick(dt);
        let down = d.body.fall_t > 0.0;
        if down && !d.fallen {
            newly_fallen.push(i);
        }
        d.fallen = down;
        let mods = Modifiers {
            stunned: d.body.stun > 0.0,
            ..Default::default()
        };
        d.mover.step(dt, MoveInput::default(), &mods, &yard.0);
    }

    for i in newly_fallen {
        let name = crate::characters::BLOB_NAMES[i % 3];
        let lines = [
            "{N} IS ABSOLUTELY WRECKED AND HAS FACE-PLANTED. GO HELP!",
            "MAN DOWN! {N} HAS HAD ONE TOO MANY. PICK 'EM UP!",
            "{N} JUST KISSED THE LAWN. SOMEONE GRAB 'EM!",
            "{N} TRIED TO WALK. THE GRASS WON. GO HELP!",
            "TIMBERRR! {N} IS HORIZONTAL. HOLD R TO REVIVE!",
            "{N} HAS GONE FULL STARFISH. RESCUE REQUIRED!",
            "{N} IS HAVING A LIE DOWN. NOT BY CHOICE. GO HELP!",
            "{N} HAS BECOME ONE WITH THE LAWN. GO FETCH 'EM!",
        ];
        let k = g.rng.f32() * lines.len() as f32;
        let line = lines[(k as usize).min(lines.len() - 1)].replace("{N}", &name.to_uppercase());
        g.say(line);
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
        stunned: g.me.body.stun > 0.0,
        facing: player_target_facing(p),
        flattenable: g.me.body.power_throw_flattens(),
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
                        drunk_bonus: if thrower == Some(PLAYER_ID) {
                            drinks::drunk_bonus(g.me.drunk.meter, g.options.drunk_mode)
                        } else {
                            0
                        },
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

fn tier_name(t: drinks::Tier) -> &'static str {
    use drinks::Tier::*;
    match t {
        Sober => "Sober",
        Tipsy => "Tipsy",
        Drunk => "Drunk",
        Maggot => "Maggot",
        AbsolutelyMaggoted => "Absolutely maggoted",
    }
}

pub fn drink_name(d: Drink) -> &'static str {
    match d {
        Drink::Beer => "a cold VP",
        Drink::Wine => "a glass of wine",
        Drink::Rum => "a rum shot",
    }
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

    fn put_player(app: &mut App, x: f32, z: f32) {
        let mut p = app.world_mut().resource_mut::<Player>();
        p.mover = Mover::new(x, z);
    }

    fn press_r(app: &mut App) {
        app.world_mut().resource_mut::<Wanted>().interact_pressed = true;
    }

    #[test]
    fn r_at_the_bar_pours_a_drink_that_adds_to_the_meter() {
        let mut app = app();
        put_player(&mut app, -1.2, -20.5); // left third: a VP
        ticks(&mut app, 30);
        assert!(app.world().resource::<Game>().prompt.contains("VP"));
        press_r(&mut app);
        ticks(&mut app, 1);
        assert!(app.world().resource::<Game>().me.drunk.is_drinking());
        ticks(&mut app, 60 * 2);
        let g = app.world().resource::<Game>();
        assert!(!g.me.drunk.is_drinking());
        assert!(
            (g.me.drunk.meter - 18.0).abs() < 2.0,
            "{}",
            g.me.drunk.meter
        );
        assert!(
            g.popups
                .iter()
                .any(|p| p.text.contains("Tipsy") || p.text.contains("drunk"))
        );
    }

    #[test]
    fn r_away_from_the_bar_does_nothing() {
        let mut app = app();
        put_player(&mut app, 8.0, 0.0);
        ticks(&mut app, 30);
        press_r(&mut app);
        ticks(&mut app, 10);
        assert!(!app.world().resource::<Game>().me.drunk.is_drinking());
    }

    #[test]
    fn no_drinks_when_the_bar_is_switched_off() {
        let mut app = app();
        app.world_mut().resource_mut::<YardRes>().0.features.bar = false;
        put_player(&mut app, -1.2, -20.5);
        ticks(&mut app, 30);
        press_r(&mut app);
        ticks(&mut app, 10);
        assert!(!app.world().resource::<Game>().me.drunk.is_drinking());
    }

    #[test]
    fn a_drink_in_your_hand_slows_you_down() {
        let mut app = app();
        put_player(&mut app, -1.2, -20.5);
        ticks(&mut app, 30);
        press_r(&mut app);
        ticks(&mut app, 1);
        app.world_mut().resource_mut::<Wanted>().wish = (1.0, 0.0);
        ticks(&mut app, 30);
        let slow = app.world().resource::<Player>().mover.speed();
        assert!(slow < 6.2 * 0.6, "speed while drinking {slow}");
    }

    #[test]
    fn holding_r_for_1_5_s_next_to_a_fallen_mate_gets_them_up_for_25_points() {
        let mut app = app();
        put_player(&mut app, 8.0, 4.5); // dummy 0 is at (8, 6)
        app.world_mut().resource_mut::<Game>().dummies[0]
            .body
            .start_fall(10.0);
        ticks(&mut app, 5);
        app.world_mut().resource_mut::<Wanted>().interact_held = true;
        ticks(&mut app, 60); // 1 s: not yet
        assert!(app.world().resource::<Game>().dummies[0].body.fall_t > 0.0);
        ticks(&mut app, 45);
        let g = app.world().resource::<Game>();
        assert_eq!(g.dummies[0].body.fall_t, 0.0);
        assert_eq!(g.board.score(PLAYER_ID), 25);
    }

    #[test]
    fn letting_go_of_r_stops_the_help() {
        let mut app = app();
        put_player(&mut app, 8.0, 4.5);
        app.world_mut().resource_mut::<Game>().dummies[0]
            .body
            .start_fall(10.0);
        ticks(&mut app, 5);
        app.world_mut().resource_mut::<Wanted>().interact_held = true;
        ticks(&mut app, 60);
        app.world_mut().resource_mut::<Wanted>().interact_held = false;
        ticks(&mut app, 5);
        app.world_mut().resource_mut::<Wanted>().interact_held = true;
        ticks(&mut app, 60);
        assert!(app.world().resource::<Game>().dummies[0].body.fall_t > 0.0);
    }

    #[test]
    fn nobody_helps_from_too_far_away() {
        let mut app = app();
        put_player(&mut app, 8.0, 0.0); // 6 m from the dummy
        app.world_mut().resource_mut::<Game>().dummies[0]
            .body
            .start_fall(10.0);
        ticks(&mut app, 5);
        app.world_mut().resource_mut::<Wanted>().interact_held = true;
        ticks(&mut app, 200);
        assert!(app.world().resource::<Game>().dummies[0].body.fall_t > 0.0);
    }

    #[test]
    fn a_dummy_stacking_it_tells_you_to_go_and_help() {
        let mut app = app();
        ticks(&mut app, 2);
        app.world_mut().resource_mut::<Game>().dummies[1]
            .body
            .start_fall(10.0);
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert!(
            g.feed.iter().any(|(s, _)| s.contains("SHEILA")),
            "{:?}",
            g.feed
        );
    }

    #[test]
    fn a_very_drunk_walker_eventually_stacks_it_and_cannot_throw() {
        let mut app = app();
        give_teddy(&mut app);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.me.drunk.meter = 100.0;
        }
        app.world_mut().resource_mut::<Wanted>().wish = (0.0, 0.0);
        // stand still so the yard walls don't matter; force it the quick way
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let g = &mut *g;
            g.me.drunk.stack_it(&mut g.me.body, 10.0);
        }
        ticks(&mut app, 5);
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 30);
        let g = app.world().resource::<Game>();
        assert!(!g.wind.charging, "can't wind up while down");
        assert!(g.me.body.fall_t > 0.0);
        assert!(g.prompt.contains("Stacked it"), "{}", g.prompt);
    }

    #[test]
    fn falling_over_is_less_than_certain_but_happens_to_a_maggot_who_keeps_walking() {
        let mut fell = 0;
        for seed in 0..12u64 {
            let mut app = app();
            {
                let mut g = app.world_mut().resource_mut::<Game>();
                g.rng = Rng::new(seed + 50);
                g.me.drunk.meter = 100.0;
                g.options.drunk_mode = true; // 4% per 2 s of walking, quick to test
            }
            put_player(&mut app, 2.0, 8.0);
            for _ in 0..40 {
                // walk back and forth in open grass
                let dir = if (app.world().resource::<Game>().now as i32 / 3) % 2 == 0 {
                    1.0
                } else {
                    -1.0
                };
                app.world_mut().resource_mut::<Wanted>().wish = (dir, 0.0);
                ticks(&mut app, 30);
                if app.world().resource::<Game>().me.body.fall_t > 0.0 {
                    fell += 1;
                    break;
                }
            }
        }
        assert!(fell >= 2, "only {fell} of 12 fell");
    }

    #[test]
    fn falls_can_be_switched_off() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.me.drunk.meter = 100.0;
            g.options.drunk_mode = true;
            g.options.falls_on = false;
        }
        put_player(&mut app, 2.0, 8.0);
        for i in 0..300 {
            let dir = if (i / 90) % 2 == 0 { 1.0 } else { -1.0 };
            app.world_mut().resource_mut::<Wanted>().wish = (dir, 0.0);
            ticks(&mut app, 20);
        }
        assert_eq!(app.world().resource::<Game>().me.body.fall_t, 0.0);
    }

    #[test]
    fn drunk_throwers_earn_a_bonus_on_hits() {
        let mut app = app();
        give_teddy(&mut app);
        app.world_mut().resource_mut::<Game>().me.drunk.meter = 60.0;
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 41);
        app.world_mut().resource_mut::<Wanted>().throw_up = true;
        ticks(&mut app, 90);
        let g = app.world().resource::<Game>();
        let me = g.board.get(PLAYER_ID).unwrap();
        assert!(
            me.score >= 150,
            "a hit while drunk should pay the +50 bonus: {}",
            me.score
        );
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
