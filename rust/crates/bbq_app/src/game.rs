//! The game state that sits between the rules (`bbq_core`) and the pictures: the items, what
//! the player holds, practice dummies to hit, and the score. Runs at a fixed 60 Hz.

use bbq_core::drinks::{self, Drink};
use bbq_core::drunk_state::{self, DrunkState, Env, Event as DrunkEvent, HelpHold};
use bbq_core::emotes::Place;
use bbq_core::flight::ItemState;
use bbq_core::hands::{self, Press, Release, Situation, Slots, Wind};
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

use crate::bots_app::BotBody;
use crate::player::{PLAYER_ID, Player, Wanted};
use crate::yard_scene::YardRes;

/// A stand-in person to throw things at until real bots arrive in Phase 5.
/// Another human in an online yard (their blob is a puppet that copies what they say they do).
#[derive(Clone, Debug, PartialEq)]
pub struct RemoteInfo {
    /// Their number in the yard (the host is 1).
    pub net_id: u32,
    pub name: String,
    pub character: bbq_core::character::Character,
}

impl RemoteInfo {
    /// The id their puppet has in this game (yours is `PLAYER_ID`; bots are 100 up).
    pub fn local_id(&self) -> PlayerId {
        REMOTE_ID_BASE + self.net_id
    }
}

/// Puppets for people online have ids from here up.
pub const REMOTE_ID_BASE: PlayerId = 200;

pub struct Dummy {
    pub id: PlayerId,
    /// Set for a person playing online: the bot brain leaves them alone.
    pub remote: Option<RemoteInfo>,
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
    /// Seconds of fish pong left.
    pub smell_t: f32,
    /// Sitting on a smoko chair (the Naughty Corner puts people there).
    pub seat: Option<usize>,
    /// Seconds left in the Naughty Corner.
    pub naughty_t: f32,
    /// Being dragged by the player (their facing and walk phase feed the pose).
    pub dragged: Option<bbq_core::pose::DraggerInfo>,
    /// Who last hit them and when (for the pool / trampoline bonus).
    pub last_hit: Option<(PlayerId, f32)>,
    /// When they were thrown (for the human cannonball).
    pub thrown_at: Option<f32>,
    /// Hit someone with this throw already.
    pub thrown_hit: bool,
    /// What the bot carries and does (its hands, drink and walking).
    pub bot: BotBody,
    /// Which way it faces, as the browser game does it: it looks along `(sin f, cos f)`.
    pub face: f32,
    /// Sitting at smoko because its own brain chose to (not sent to the Naughty Corner).
    pub sat_by_choice: bool,
    /// How big this bot's beer belly is (looks only; 1 = normal).
    pub belly: f32,
}

impl Dummy {
    /// A fresh bot (or practice blob) standing at `(x, z)`. `i` is its place in the yard.
    pub fn new(id: PlayerId, x: f32, z: f32, i: usize, rng: &mut Rng) -> Self {
        Dummy {
            id,
            remote: None,
            mover: Mover::new(x, z),
            body: Body::default(),
            home: (x, z),
            anim: bbq_core::pose::Animator::new(i as f32 * 2.1),
            drunk: 0.0,
            fallen: false,
            crown: false,
            team: None,
            smelly: false,
            smell_t: 0.0,
            seat: None,
            naughty_t: 0.0,
            dragged: None,
            last_hit: None,
            thrown_at: None,
            thrown_hit: false,
            bot: BotBody::new(i as f32 * 1.9, rng),
            face: 0.0,
            sat_by_choice: false,
            belly: 0.5 + rng.f32() * 1.1,
        }
    }
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
    /// Cheeky mode: bare-handed slaps, the chest, rude Dazza lines, gnomes in Dazza's bum.
    pub adult: bool,
    /// The Naughty Corner (needs smoko).
    pub naughty: bool,
    /// Smoko is switched on in the yard (kept in step with the F4 toggle).
    pub smoko_on: bool,
    /// The bots play (F9 freezes them into practice dummies).
    pub bots_on: bool,
    /// How good the bots are (F10).
    pub bot_difficulty: bbq_core::bots::Difficulty,
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
    pub life: crate::life::Life,
    /// The bots' minds.
    pub crowd: bbq_core::bots::Crowd,
    /// Rounds, matches and the game mode.
    pub round: crate::round::RoundCtl,
    /// Teddy Heist, when that is the mode.
    pub heist: Option<crate::heist_app::HeistState>,
    /// The yard behind the menu: the bots wander and ignore you.
    pub attract: bool,
    /// Things that just happened, for the particle effects to show (see `fx.rs`).
    pub fx: Vec<crate::fx::FxEvent>,
    /// The other people in an online yard (empty when playing alone).
    pub remotes: Vec<RemoteInfo>,
    /// Goes up whenever the people in the yard are rebuilt, so the blobs are rebuilt too.
    pub dummies_version: u32,
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

    /// Show a particle effect at `at`.
    pub fn fx(&mut self, kind: crate::fx::FxKind, at: V3) {
        if self.fx.len() < 24 {
            self.fx.push(crate::fx::FxEvent { kind, at });
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
        let mut dummy_rng = Rng::new(0xB07);
        let mut crowd = bbq_core::bots::Crowd::default();
        let dummies: Vec<Dummy> = spots
            .iter()
            .enumerate()
            .map(|(i, s)| {
                board.ensure(100 + i as u32);
                Dummy::new(100 + i as u32, s.0, s.1, i, &mut dummy_rng)
            })
            .collect();
        for d in &dummies {
            crowd.add(d.id, &mut dummy_rng);
        }
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
                adult: false,
                naughty: true,
                smoko_on: true,
                bots_on: true,
                bot_difficulty: bbq_core::bots::Difficulty::Fair,
            },
            popups: Vec::new(),
            prompt: String::new(),
            life: crate::life::Life::new(),
            crowd,
            round: crate::round::RoundCtl::new(),
            heist: None,
            attract: false,
            fx: Vec::new(),
            remotes: Vec::new(),
            dummies_version: 0,
        })
        .add_systems(FixedUpdate, step_game.run_if(crate::menu::world_runs));
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

    // ---- the round clock, and Heist banking ----
    crate::round::step(g);
    crate::heist_app::step(g, p);

    // ---- the player's body: drinking, falling over, helping mates up ----
    g.options.smoko_on = yard.0.features.smoko;
    if !yard.0.features.smoko && g.life.seated.is_some() {
        g.life.seated = None; // smoko switched off while you were sitting
    }
    let seated = g.life.seated.is_some();
    let moving = wanted.wish.0 != 0.0 || wanted.wish.1 != 0.0;
    let env = Env {
        in_play: g.rules.in_play(),
        at_smoko: seated,
        grounded: p.mover.grounded,
        in_pool: p.mover.in_pool,
        moving,
        falls_on: g.options.falls_on,
        drunk_mode: g.options.drunk_mode,
        can_fall: !seated,
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
        let name = crate::characters::BLOB_NAMES[i % crate::characters::BLOB_NAMES.len()];
        g.popup(format!("Helped {name} up! +{pts}"), false);
    }
    let bar_on = yard.0.features.bar;
    let spot = if can_use && bar_on {
        drunk_state::bar_spot(&bbq_core::yard::BAR, p.mover.x, p.mover.y, p.mover.z)
    } else {
        None
    };
    if std::mem::take(&mut wanted.interact_pressed) && help_target.is_none() {
        let features = yard.0.features;
        if !crate::life::interact(g, p, &features, yard.0.chest_spot)
            && g.me.drunk.start_drink(spot, &g.me.body, &env)
        {
            g.wind.cancel();
        }
    }
    g.prompt = if g.me.body.fall_t > 0.0 {
        format!(
            "Stacked it! Up in {}s (or a mate can help you up)",
            g.me.body.fall_t.ceil()
        )
    } else if let Some(i) = help_target {
        let name = crate::characters::BLOB_NAMES[i % crate::characters::BLOB_NAMES.len()];
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
        crate::life::prompt(g, p, &yard.0.features, yard.0.chest_spot)
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
    // G: throw away what you are holding. It drops a little in front of you and you cannot pick
    // it straight back up (2.5 s), as in the browser game's over-held drop.
    if std::mem::take(&mut wanted.drop_held)
        && g.rules.phase != Phase::Countdown
        && g.me.body.stun <= 0.0
        && g.life.seated.is_none()
        && g.life.carry.is_none()
        && let Some(id) = g.slots.selected()
    {
        g.slots.remove(id);
        g.wind.cancel();
        let aim = aim_dir(p.yaw, 0.0);
        let at = eye_of(p) + aim * 0.7 + V3::new(0.0, -0.4, 0.0);
        let until = g.now + 2.5;
        g.world.drop_item(id, at, Some((PLAYER_ID, until)));
        if let Some(it) = g.world.items.get_mut(&id) {
            it.vel = aim * 2.2 + V3::new(0.0, 1.0, 0.0);
        }
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
        at_smoko: seated,
        carrying_someone: g.life.carry.is_some(),
    };

    if std::mem::take(&mut wanted.throw_down) {
        match g.wind.press(now, selected_kind, &situation) {
            Press::Charging => {}
            Press::SlapNow => crate::life::player_slap(g, p),
            Press::Refused(hands::Refuse::NothingHeld) => {
                if g.options.adult {
                    crate::life::bare_slap(g, p);
                } else {
                    g.say("Nothing to chuck. Walk over something glowing.");
                }
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
            Release::Slap => crate::life::player_slap(g, p),
            Release::Nothing => {}
        }
    }
    if std::mem::take(&mut wanted.catch) {
        let can = g.rules.in_play() && g.me.body.stun <= 0.0;
        g.catcher.press(can);
    }

    // ---- pick things up ----
    if g.rules.phase != Phase::Countdown
        && g.me.body.stun <= 0.0
        && let Some(id) = crate::heist_app::try_pickup(
            g,
            PLAYER_ID,
            V3::new(p.mover.x, p.mover.y, p.mover.z),
            g.slots.len(),
            false,
            false,
        )
    {
        g.world.give(id, PLAYER_ID);
        g.slots.add(id);
    }

    // ---- the bots decide what to do ----
    crate::bots_app::step(g, p, &yard.0);

    // ---- the dummies stand about and get knocked over ----
    let countdown = g.rules.phase == Phase::Countdown;
    let mut newly_fallen = Vec::new();
    let mut env_landings: Vec<(usize, Place)> = Vec::new();
    let mut splashes: Vec<V3> = Vec::new();
    for (i, d) in g.dummies.iter_mut().enumerate() {
        d.body.tick(dt);
        let down = d.body.fall_t > 0.0;
        if down && !d.fallen {
            newly_fallen.push(i);
        }
        d.fallen = down;
        if d.seat.is_some() || d.dragged.is_some() {
            continue; // sitting in a smoko chair, or being dragged: placed by hand
        }
        let mods = Modifiers {
            stunned: d.body.stun > 0.0,
            charging: d.bot.winding,
            drinking: d.bot.drunk.is_drinking(),
            frozen: countdown,
            ..Default::default()
        };
        let ev = d
            .mover
            .step(dt, MoveInput { wish: d.bot.wish }, &mods, &yard.0);
        if ev.splash {
            splashes.push(V3::new(d.mover.x, bbq_core::yard::WATER_Y, d.mover.z));
            env_landings.push((i, Place::Pool));
        } else if ev.bounce.is_some() {
            env_landings.push((i, Place::Tramp));
        }
    }
    for (i, place) in env_landings {
        crate::life::env_bonus(g, i, place);
    }
    for at in splashes {
        g.fx(crate::fx::FxKind::Splash, at);
    }

    for i in newly_fallen {
        let at = V3::new(g.dummies[i].mover.x, 0.15, g.dummies[i].mover.z);
        g.fx(crate::fx::FxKind::Dust, at);
        let name = crate::characters::BLOB_NAMES[i % crate::characters::BLOB_NAMES.len()];
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
            at_smoko: d.seat.is_some(),
            catching: d.bot.catcher.open(),
            stunned: d.body.stun > 0.0,
            facing: V3::new(d.face.sin(), 0.0, d.face.cos()),
            flattenable: d.body.power_throw_flattens(),
        })
        .collect();
    targets.push(Target {
        id: PLAYER_ID,
        pos: V3::new(p.mover.x, p.mover.y, p.mover.z),
        sink: p.mover.sink,
        at_smoko: seated,
        catching: g.catcher.open(),
        stunned: g.me.body.stun > 0.0,
        facing: player_target_facing(p),
        flattenable: g.me.body.power_throw_flattens(),
    });
    let teams = g.teams.clone();
    let ctx = Context {
        friendly_fire: g.rules.friendly_fire,
        teams: &teams,
    };
    let stray_teddies = crate::heist_app::team_teddies(g);
    let events = g.world.step(
        dt,
        &yard.0,
        &mut g.rng,
        &targets,
        &ctx,
        2 + g.dummies.len(),
        true,
        &|x, z| yard.0.heist_clear(x, z, 1.2),
    );
    let removed: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            WorldEvent::Removed(id) => Some(*id),
            _ => None,
        })
        .collect();
    if !stray_teddies.is_empty() && !removed.is_empty() {
        crate::heist_app::teddies_left(g, &stray_teddies, &removed);
    }

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
                if victim == PLAYER_ID {
                    crate::bots_app::hit_player(g, p, kind, thrower, dir, flatten, charge, start);
                    continue;
                }
                let Some(di) = g.dummies.iter().position(|d| d.id == victim) else {
                    continue;
                };
                let (vx, vz) = (g.dummies[di].mover.x, g.dummies[di].mover.z);
                g.fx(
                    if kind == ItemKind::Stubby { crate::fx::FxKind::Smash } else { crate::fx::FxKind::Hit },
                    V3::new(vx, 1.2, vz),
                );
                let leader = g.board.leader() == Some(victim);
                // knock and stun
                let item = bbq_core::flight::Item::new(0, kind, V3::ZERO);
                let d = &mut g.dummies[di];
                if let Some(t) = thrower {
                    d.last_hit = Some((t, now));
                }
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
                        drunk_bonus: thrower.map_or(0, |t| crate::bots_app::drunk_bonus_of(g, t)),
                        bum_out: false,
                        same_team: thrower.is_some_and(|t| g.teams.same_team(t, victim)),
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
                if thrower == Some(PLAYER_ID) {
                    g.say(msg);
                } else {
                    let who =
                        thrower.map_or("Somebody".to_string(), |t| crate::bots_app::name_of(g, t));
                    let to = crate::bots_app::name_of(g, victim);
                    g.say(format!("{who} hit {to}"));
                }
            }
            WorldEvent::Caught { by, thrower, .. } if by != PLAYER_ID => {
                if let Some(i) = g.dummies.iter().position(|d| d.id == by)
                    && let Some(id) = caught_item(&g.world, by, &g.dummies[i].bot.slots)
                {
                    let slots = &mut g.dummies[i].bot.slots;
                    if slots.is_full()
                        && let Some(old) = slots.oldest()
                    {
                        slots.remove(old);
                        let at = V3::new(g.dummies[i].mover.x, 1.2, g.dummies[i].mover.z);
                        g.world.drop_item(old, at, None);
                    }
                    g.dummies[i].bot.slots.add(id);
                    g.board.catch(&g.rules, by, thrower);
                    let who = crate::bots_app::name_of(g, by);
                    g.say(format!("{who} caught it!"));
                }
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
    for d in g.dummies.iter_mut().filter(|_| !g.options.bots_on) {
        if !d.body.is_down() && d.body.stun <= 0.0 && d.mover.grounded && d.mover.speed() < 0.3 {
            let (hx, hz) = d.home;
            let dist = ((d.mover.x - hx).powi(2) + (d.mover.z - hz).powi(2)).sqrt();
            if dist > 0.01 {
                d.mover.x += (hx - d.mover.x) * 0.02;
                d.mover.z += (hz - d.mover.z) * 0.02;
            }
        }
    }

    // Dazza, smoko, dragging, emotes
    crate::life::step(g, p, &mut wanted, &yard.0);

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
    use bbq_core::teams::Team;

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
        // clear the random items so the test is predictable, keep only what we add; the bots
        // stand still (the tests are about the player's own moves)
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.world.clear();
            g.options.bots_on = false;
        }
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
            g.feed.iter().any(|(s, _)| s.contains("DAVO")),
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

    // ---------------------------------------------------------------- Phase 5B

    fn give_kind(app: &mut App, kind: ItemKind) {
        let mut g = app.world_mut().resource_mut::<Game>();
        let mut rng = Rng::new(11);
        let id = g.world.spawn(kind, 8.0, 0.0, false, &mut rng);
        if let Some(it) = g.world.items.get_mut(&id) {
            it.uses = kind.starting_uses();
            if kind == ItemKind::Dildo {
                it.variant = Some(bbq_core::items::DildoVariant::Classic);
            }
        }
        g.world.give(id, PLAYER_ID);
        g.slots.add(id);
    }

    fn click(app: &mut App) {
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(app, 1);
        app.world_mut().resource_mut::<Wanted>().throw_up = true;
        ticks(app, 2);
    }

    fn near_dummy0(app: &mut App) {
        put_player(app, 8.0, 4.5); // dummy 0 stands at (8, 6), in front of us
        ticks(app, 2);
    }

    #[test]
    fn a_steak_slap_stuns_a_dummy_pays_50_and_uses_up_a_charge() {
        let mut app = app();
        near_dummy0(&mut app);
        give_kind(&mut app, ItemKind::Steak);
        click(&mut app);
        let g = app.world().resource::<Game>();
        assert!(
            g.dummies[0].body.stun > 1.0,
            "stun {}",
            g.dummies[0].body.stun
        );
        assert_eq!(g.board.score(PLAYER_ID), 50);
        let id = g.slots.selected().unwrap();
        assert_eq!(g.world.items[&id].uses, Some(2));
        assert_eq!(
            g.board.score(g.dummies[0].id),
            0,
            "no penalty for the victim"
        );
    }

    #[test]
    fn a_steak_wears_out_after_three_slaps() {
        let mut app = app();
        near_dummy0(&mut app);
        give_kind(&mut app, ItemKind::Steak);
        for _ in 0..3 {
            click(&mut app);
            ticks(&mut app, 60);
            put_player(&mut app, 8.0, 4.5);
            let mut g = app.world_mut().resource_mut::<Game>();
            g.dummies[0].mover.x = 8.0;
            g.dummies[0].mover.z = 6.0;
        }
        let g = app.world().resource::<Game>();
        assert!(g.slots.is_empty(), "the steak should have fallen apart");
    }

    #[test]
    fn a_dildo_slap_knocks_flat_and_scores_like_a_hit() {
        let mut app = app();
        near_dummy0(&mut app);
        give_kind(&mut app, ItemKind::Dildo);
        click(&mut app);
        let g = app.world().resource::<Game>();
        assert!(g.dummies[0].body.is_down());
        assert!(
            g.board.score(PLAYER_ID) >= 100,
            "{}",
            g.board.score(PLAYER_ID)
        );
        assert_eq!(g.board.score(g.dummies[0].id), -50);
    }

    fn set_held_variant(app: &mut App, v: bbq_core::items::DildoVariant) {
        let mut g = app.world_mut().resource_mut::<Game>();
        let id = g.slots.selected().unwrap();
        g.world.items.get_mut(&id).unwrap().variant = Some(v);
    }

    #[test]
    fn a_long_dildo_reaches_a_target_a_normal_one_cannot() {
        use bbq_core::items::DildoVariant;
        // the dummy is 2.8 m away: past the normal 2.6 m reach, inside Long John's 3.6 m
        for (variant, hits) in [(DildoVariant::Classic, false), (DildoVariant::LongJohn, true)] {
            let mut app = app();
            put_player(&mut app, 8.0, 3.2); // dummy 0 stands at (8, 6)
            ticks(&mut app, 2);
            give_kind(&mut app, ItemKind::Dildo);
            set_held_variant(&mut app, variant);
            click(&mut app);
            let g = app.world().resource::<Game>();
            assert_eq!(g.dummies[0].body.is_down(), hits, "{variant:?}");
        }
    }

    #[test]
    fn a_slap_out_of_reach_misses_and_keeps_the_charge() {
        let mut app = app();
        put_player(&mut app, 8.0, 0.0); // 6 m away
        give_kind(&mut app, ItemKind::Steak);
        click(&mut app);
        let g = app.world().resource::<Game>();
        assert_eq!(g.board.score(PLAYER_ID), 0);
        let id = g.slots.selected().unwrap();
        assert_eq!(g.world.items[&id].uses, Some(3));
    }

    #[test]
    fn slaps_have_a_cooldown() {
        let mut app = app();
        near_dummy0(&mut app);
        give_kind(&mut app, ItemKind::Steak);
        click(&mut app);
        click(&mut app); // straight away: too soon
        assert_eq!(app.world().resource::<Game>().board.score(PLAYER_ID), 50);
    }

    #[test]
    fn bare_hands_only_slap_in_cheeky_mode() {
        let mut app = app();
        near_dummy0(&mut app);
        click(&mut app);
        assert_eq!(app.world().resource::<Game>().board.score(PLAYER_ID), 0);
        app.world_mut().resource_mut::<Game>().options.adult = true;
        ticks(&mut app, 60);
        click(&mut app);
        let g = app.world().resource::<Game>();
        assert_eq!(g.board.score(PLAYER_ID), 40);
        assert!(g.dummies[0].body.stun > 0.0);
    }

    fn near_dazza(app: &mut App) {
        put_player(app, -6.0, -17.0);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.life.dazza.pos = bbq_core::dazza::HOME;
            g.life.dazza.face = 0.0;
        }
        app.world_mut().resource_mut::<Player>().yaw = 0.0; // looking along -z, at the grill
        ticks(app, 2);
    }

    #[test]
    fn a_steak_slap_on_dazza_stuns_him_for_20_points() {
        let mut app = app();
        near_dazza(&mut app);
        give_kind(&mut app, ItemKind::Steak);
        click(&mut app);
        let g = app.world().resource::<Game>();
        assert_eq!(g.life.dazza.state, bbq_core::dazza::DazzaState::Stunned);
        assert_eq!(g.board.score(PLAYER_ID), 20);
        assert!(!g.life.say_text.is_empty());
    }

    #[test]
    fn dildo_slaps_on_dazza_do_one_of_three_things_and_pay() {
        let mut seen = std::collections::BTreeSet::new();
        for seed in 0..30u64 {
            let mut app = app();
            app.world_mut().resource_mut::<Game>().rng = Rng::new(seed + 1);
            near_dazza(&mut app);
            give_kind(&mut app, ItemKind::Dildo);
            click(&mut app);
            let g = app.world().resource::<Game>();
            let s = g.board.score(PLAYER_ID);
            assert!([75, 25, 50].contains(&s), "{s}");
            seen.insert(s);
        }
        assert_eq!(
            seen.len(),
            3,
            "KO, berserk and flip should all turn up: {seen:?}"
        );
    }

    /// R at the meat table, then bin the meat so the next grab isn't refused for full hands.
    fn take_meat(app: &mut App) {
        press_r(app);
        ticks(app, 50);
        let mut g = app.world_mut().resource_mut::<Game>();
        let ids: Vec<_> = g.slots.ids().to_vec();
        for id in ids {
            g.slots.remove(id);
            g.world.remove(id);
        }
    }

    #[test]
    fn taking_meat_makes_dazza_angry_and_three_times_makes_him_chase() {
        let mut app = app();
        put_player(&mut app, -9.5, -17.0); // left half of the meat table: steak
        ticks(&mut app, 2);
        press_r(&mut app);
        ticks(&mut app, 2);
        {
            let g = app.world().resource::<Game>();
            assert_eq!(g.slots.len(), 1);
            let id = g.slots.selected().unwrap();
            assert_eq!(g.world.items[&id].kind, ItemKind::Steak);
            assert_eq!(g.life.dazza.state, bbq_core::dazza::DazzaState::Angry);
        }
        ticks(&mut app, 60);
        // a second grab with a full hand is fine (two allowed), a third is refused
        press_r(&mut app);
        ticks(&mut app, 60);
        press_r(&mut app);
        ticks(&mut app, 60);
        let g = app.world().resource::<Game>();
        assert_eq!(g.slots.len(), 2, "max two things in your hands");
    }

    #[test]
    fn dazza_chases_and_spatulas_a_thief() {
        let mut app = app();
        put_player(&mut app, -9.5, -17.0);
        ticks(&mut app, 2);
        // he has already had it in for us (two earlier raids); the next one sends him over
        app.world_mut()
            .resource_mut::<Game>()
            .life
            .dazza
            .grudges
            .insert(PLAYER_ID, 1.6);
        take_meat(&mut app);
        // stand still where he can reach us
        let mut hit = app.world().resource::<Game>().me.body.stun > 0.0
            || app
                .world()
                .resource::<Game>()
                .popups
                .iter()
                .any(|p| p.text.contains("SPATULA"));
        for _ in 0..60 * 8 {
            if hit {
                break;
            }
            ticks(&mut app, 1);
            let g = app.world().resource::<Game>();
            if g.me.body.stun > 0.0 || g.popups.iter().any(|p| p.text.contains("SPATULA")) {
                hit = true;
                break;
            }
        }
        assert!(
            hit,
            "state {:?}",
            app.world().resource::<Game>().life.dazza.state
        );
    }

    #[test]
    fn dazza_stays_home_when_the_bbq_is_switched_off() {
        let mut app = app();
        app.world_mut().resource_mut::<YardRes>().0.features.bbq = false;
        ticks(&mut app, 120);
        let g = app.world().resource::<Game>();
        assert!((g.life.dazza.pos.x - bbq_core::dazza::HOME.x).abs() < 0.01);
    }

    fn put_in_smoko(app: &mut App) {
        put_player(app, yard::SMOKO_X + 1.0, yard::SMOKO_Z);
        ticks(app, 2);
    }
    use bbq_core::yard;

    #[test]
    fn r_in_the_smoko_zone_sits_you_down_and_the_drinks_keep_coming() {
        let mut app = app();
        app.world_mut().resource_mut::<Game>().options.naughty = false;
        put_in_smoko(&mut app);
        press_r(&mut app);
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().life.seated.is_some());
        ticks(&mut app, 60 * 5);
        let g = app.world().resource::<Game>();
        assert!(g.me.drunk.meter > 15.0, "meter {}", g.me.drunk.meter);
        // r stands you up again
        press_r(&mut app);
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().life.seated.is_none());
    }

    #[test]
    fn smoko_runs_out_after_20_seconds() {
        let mut app = app();
        app.world_mut().resource_mut::<Game>().options.naughty = false;
        put_in_smoko(&mut app);
        press_r(&mut app);
        ticks(&mut app, 60 * 21);
        let g = app.world().resource::<Game>();
        assert!(g.life.seated.is_none());
        assert!(
            g.popups.iter().any(|p| p.text.contains("Smoko's over")) || g.me.drunk.meter >= 99.0
        );
    }

    #[test]
    fn nobody_can_hit_you_while_you_sit_at_smoko() {
        let mut app = app();
        app.world_mut().resource_mut::<Game>().options.naughty = false;
        put_in_smoko(&mut app);
        press_r(&mut app);
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert!(g.life.seated.is_some());
        let p = app.world().resource::<Player>();
        let (px, pz) = (p.mover.x, p.mover.z);
        // a thrown teddy aimed straight at the seat passes through
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let mut rng = Rng::new(2);
            let id = g
                .world
                .spawn(ItemKind::Teddy, px, pz - 4.0, false, &mut rng);
            g.world.give(id, 100);
            g.world.throw(
                id,
                100,
                V3::new(px, 1.2, pz - 4.0),
                V3::new(0.0, 0.0, 20.0),
                1.0,
                true,
            );
        }
        ticks(&mut app, 60);
        assert_eq!(app.world().resource::<Game>().me.body.stun, 0.0);
    }

    #[test]
    fn with_the_naughty_corner_on_the_zone_is_not_a_safe_break() {
        let mut app = app();
        put_in_smoko(&mut app);
        press_r(&mut app);
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().life.seated.is_none());
    }

    fn fell(app: &mut App) {
        app.world_mut().resource_mut::<Game>().dummies[0]
            .body
            .start_fall(10.0);
        ticks(app, 2);
    }

    #[test]
    fn f_grabs_someone_who_is_down_and_a_tap_puts_them_down() {
        let mut app = app();
        near_dummy0(&mut app);
        fell(&mut app);
        app.world_mut().resource_mut::<Wanted>().grab_pressed = true;
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().life.carry.is_some());
        ticks(&mut app, 30);
        {
            let g = app.world().resource::<Game>();
            let d = &g.dummies[0];
            let p = app.world().resource::<Player>();
            assert!(
                (d.mover.z - (p.mover.z - bbq_core::carry::DRAG_DISTANCE)).abs() < 0.2
                    || (d.mover.z - p.mover.z).abs() < 2.0
            );
            assert!(d.dragged.is_some());
        }
        app.world_mut().resource_mut::<Wanted>().grab_pressed = true;
        ticks(&mut app, 2);
        app.world_mut().resource_mut::<Wanted>().grab_released = true;
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert!(g.life.carry.is_none());
        assert!(g.dummies[0].dragged.is_none());
        assert!(
            g.dummies[0].thrown_at.is_none(),
            "a tap puts down, it doesn't throw"
        );
        assert!(g.dummies[0].body.stun > 0.0);
    }

    #[test]
    fn you_cannot_grab_someone_who_is_only_stunned() {
        let mut app = app();
        near_dummy0(&mut app);
        app.world_mut().resource_mut::<Game>().dummies[0]
            .body
            .apply_hit(2.0, None);
        app.world_mut().resource_mut::<Wanted>().grab_pressed = true;
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().life.carry.is_none());
    }

    #[test]
    fn holding_f_then_letting_go_chucks_them_and_they_fly() {
        let mut app = app();
        near_dummy0(&mut app);
        fell(&mut app);
        app.world_mut().resource_mut::<Wanted>().grab_pressed = true;
        ticks(&mut app, 2);
        app.world_mut().resource_mut::<Wanted>().grab_pressed = true; // F down to throw
        ticks(&mut app, 30); // held for half a second
        app.world_mut().resource_mut::<Wanted>().grab_released = true;
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert!(g.dummies[0].thrown_at.is_some());
        assert!(
            g.dummies[0].mover.vz > 5.0 || g.dummies[0].mover.z > 7.0,
            "flying away"
        );
    }

    #[test]
    fn a_thrown_person_flattens_whoever_they_hit_for_100() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let now = g.now;
            g.dummies[0].thrown_at = Some(now - 0.3);
            g.dummies[0].mover.x = g.dummies[1].mover.x + 0.5;
            g.dummies[0].mover.z = g.dummies[1].mover.z;
            g.dummies[0].mover.y = 1.0;
            g.dummies[0].mover.vy = 0.0;
        }
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert!(g.dummies[1].body.is_down());
        assert_eq!(g.board.score(PLAYER_ID), 100);
    }

    #[test]
    fn setting_someone_down_in_the_smoko_zone_sends_them_to_the_naughty_corner() {
        let mut app = app();
        near_dummy0(&mut app);
        fell(&mut app);
        app.world_mut().resource_mut::<Wanted>().grab_pressed = true;
        ticks(&mut app, 2);
        // walk into the smoko zone with them in tow
        put_player(&mut app, yard::SMOKO_X + 1.5, yard::SMOKO_Z);
        app.world_mut().resource_mut::<Wanted>().grab_pressed = true;
        ticks(&mut app, 3);
        app.world_mut().resource_mut::<Wanted>().grab_released = true;
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert!(
            g.dummies[0].naughty_t > 4.0,
            "naughty {}",
            g.dummies[0].naughty_t
        );
        assert!(g.dummies[0].seat.is_some());
        assert_eq!(g.board.score(PLAYER_ID), 100);
        ticks(&mut app, 60 * 6);
        let g = app.world().resource::<Game>();
        assert!(g.dummies[0].seat.is_none(), "up again after five seconds");
    }

    #[test]
    fn knocking_someone_into_the_pool_soon_after_hitting_them_pays_50() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let now = g.now;
            g.dummies[2].last_hit = Some((PLAYER_ID, now - 1.0));
            g.dummies[2].mover.x = -20.0;
            g.dummies[2].mover.z = 10.0;
            g.dummies[2].mover.y = 0.0;
        }
        ticks(&mut app, 3);
        let g = app.world().resource::<Game>();
        assert_eq!(g.board.score(PLAYER_ID), 50, "{:?}", g.feed);
        assert!(g.popups.iter().any(|p| p.text.contains("SPLASHDOWN")));
    }

    #[test]
    fn the_pool_pays_nothing_if_you_hit_them_ages_ago() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let now = g.now;
            g.dummies[2].last_hit = Some((PLAYER_ID, now - 10.0));
            g.dummies[2].mover.x = -20.0;
            g.dummies[2].mover.z = 10.0;
        }
        ticks(&mut app, 3);
        assert_eq!(app.world().resource::<Game>().board.score(PLAYER_ID), 0);
    }

    #[test]
    fn emotes_have_a_two_second_cooldown() {
        let mut app = app();
        ticks(&mut app, 2);
        app.world_mut().resource_mut::<Wanted>().emote = Some(bbq_core::pose::Emote::Taunt);
        ticks(&mut app, 2);
        let n1 = app.world().resource::<Game>().popups.len();
        assert_eq!(n1, 1);
        app.world_mut().resource_mut::<Wanted>().emote = Some(bbq_core::pose::Emote::Dance);
        ticks(&mut app, 2);
        assert_eq!(app.world().resource::<Game>().popups.len(), 1, "too soon");
        ticks(&mut app, 130);
        app.world_mut().resource_mut::<Wanted>().emote = Some(bbq_core::pose::Emote::Laugh);
        ticks(&mut app, 2);
        assert!(!app.world().resource::<Game>().popups.is_empty());
    }

    #[test]
    fn you_cannot_emote_when_stunned() {
        let mut app = app();
        app.world_mut()
            .resource_mut::<Game>()
            .me
            .body
            .apply_hit(2.0, None);
        app.world_mut().resource_mut::<Wanted>().emote = Some(bbq_core::pose::Emote::Taunt);
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().popups.is_empty());
    }

    fn chest_spot_pos(app: &App) -> (f32, f32) {
        let y = app.world().resource::<YardRes>();
        let (x, z, _) = bbq_core::yard::CHEST_SPOTS[y.0.chest_spot % 9];
        (x, z)
    }

    #[test]
    fn the_chest_gives_a_dildo_in_cheeky_mode_only() {
        let mut app = app();
        let (cx, cz) = chest_spot_pos(&app);
        put_player(&mut app, cx + 1.4, cz);
        ticks(&mut app, 2);
        press_r(&mut app);
        ticks(&mut app, 2);
        assert!(
            app.world().resource::<Game>().slots.is_empty(),
            "not in Cheeky mode"
        );
        app.world_mut().resource_mut::<Game>().options.adult = true;
        press_r(&mut app);
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert_eq!(g.slots.len(), 1);
        assert_eq!(g.life.chest.stock, 1);
        let id = g.slots.selected().unwrap();
        assert_eq!(g.world.items[&id].kind, ItemKind::Dildo);
    }

    #[test]
    fn the_chest_runs_out() {
        let mut app = app();
        let (cx, cz) = chest_spot_pos(&app);
        put_player(&mut app, cx + 1.4, cz);
        app.world_mut().resource_mut::<Game>().options.adult = true;
        app.world_mut().resource_mut::<Game>().life.chest.stock = 0;
        ticks(&mut app, 2);
        press_r(&mut app);
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().slots.is_empty());
    }

    // ---------------------------------------------------------------- Phase 6: the bots

    fn bots_on(app: &mut App) {
        app.world_mut().resource_mut::<Game>().options.bots_on = true;
    }

    fn give_bot(app: &mut App, i: usize, kind: ItemKind) -> bbq_core::flight::ItemId {
        let mut g = app.world_mut().resource_mut::<Game>();
        let mut rng = Rng::new(21);
        let (x, z) = (g.dummies[i].mover.x, g.dummies[i].mover.z);
        let id = g.world.spawn(kind, x, z, false, &mut rng);
        let owner = g.dummies[i].id;
        g.world.give(id, owner);
        g.dummies[i].bot.slots.add(id);
        id
    }

    fn bot_throws(g: &Game) -> u32 {
        g.dummies
            .iter()
            .map(|d| g.board.get(d.id).map_or(0, |s| s.throws))
            .sum()
    }

    #[test]
    fn frozen_bots_stand_still_like_practice_dummies() {
        let mut app = app();
        ticks(&mut app, 600);
        let g = app.world().resource::<Game>();
        for d in &g.dummies {
            assert!((d.mover.x - d.home.0).abs() < 0.01 && (d.mover.z - d.home.1).abs() < 0.01);
        }
    }

    #[test]
    fn bots_wander_about_and_pick_things_up_and_throw_them() {
        let mut app = app();
        bots_on(&mut app);
        // plenty to pick up around the bots
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let mut rng = Rng::new(5);
            for k in 0..12 {
                let (x, z) = (-8.0 + k as f32 * 1.5, -4.0 + (k % 3) as f32 * 3.0);
                g.world.spawn(ItemKind::Teddy, x, z, false, &mut rng);
            }
        }
        put_player(&mut app, 20.0, 15.0);
        ticks(&mut app, 60 * 40);
        let g = app.world().resource::<Game>();
        let held: usize = g.dummies.iter().map(|d| d.bot.slots.len()).sum();
        assert!(
            held + bot_throws(g) as usize > 0,
            "bots should have picked something up"
        );
        assert!(bot_throws(g) >= 1, "and thrown it; feed {:?}", g.feed);
        for d in &g.dummies {
            let moved = (d.mover.x - d.home.0).hypot(d.mover.z - d.home.1);
            assert!(
                moved > 2.0 || d.seat.is_some(),
                "bot {} never left home",
                d.id
            );
        }
    }

    #[test]
    fn bots_hit_the_player_when_they_stand_in_the_open() {
        let mut app = app();
        bots_on(&mut app);
        put_player(&mut app, 0.0, 6.0);
        for i in 0..3 {
            for _ in 0..2 {
                give_bot(&mut app, i, ItemKind::Teddy);
            }
        }
        ticks(&mut app, 60 * 45);
        let g = app.world().resource::<Game>();
        let me = g.board.get(PLAYER_ID).unwrap();
        assert!(
            me.taken >= 1,
            "someone should have hit the player; feed {:?}",
            g.feed
        );
        assert!(me.score <= -50);
        let scored: i32 = g.dummies.iter().map(|d| g.board.score(d.id)).sum();
        assert!(scored > 0, "the thrower should have been paid");
        assert!(g.feed.iter().any(|(m, _)| m.contains("hit you")) || me.taken >= 1);
    }

    #[test]
    fn a_hit_on_the_player_knocks_them_and_stuns_them_once() {
        let mut app = app();
        put_player(&mut app, 2.0, 0.0); // off to one side of the clothesline pole
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let mut rng = Rng::new(3);
            // a teddy coming straight at the player from the front (+z side), thrown by bot 100
            let id = g.world.spawn(ItemKind::Teddy, 2.0, 6.0, false, &mut rng);
            g.world.give(id, 100);
            g.world.throw(
                id,
                100,
                V3::new(2.0, 1.2, 6.0),
                V3::new(0.0, 0.5, -18.0),
                0.8,
                false,
            );
        }
        ticks(&mut app, 60);
        let g = app.world().resource::<Game>();
        assert!(g.me.body.stun > 0.0 || g.me.body.stun_grace > 0.0);
        assert_eq!(g.board.get(PLAYER_ID).unwrap().taken, 1);
        assert_eq!(g.board.score(PLAYER_ID), -50);
        assert!(g.board.score(100) >= 100);
        assert!(g.popups.iter().any(|p| p.text.contains("got you")));
    }

    #[test]
    fn a_bot_with_a_steak_walks_up_and_stuns_the_player() {
        let mut app = app();
        bots_on(&mut app);
        put_player(&mut app, 8.0, 3.0);
        give_bot(&mut app, 0, ItemKind::Steak);
        {
            // the other two have nothing to do with it
            let mut g = app.world_mut().resource_mut::<Game>();
            g.dummies[1].mover.x = -30.0;
            g.dummies[2].mover.x = 30.0;
        }
        ticks(&mut app, 60 * 25);
        let g = app.world().resource::<Game>();
        let steaked = g.feed.iter().any(|(m, _)| m.contains("steaked you"));
        assert!(
            steaked || g.board.score(100) >= 50,
            "feed {:?} score {}",
            g.feed,
            g.board.score(100)
        );
    }

    #[test]
    fn bots_never_stack_it_however_drunk() {
        let mut app = app();
        bots_on(&mut app);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            for d in &mut g.dummies {
                d.bot.drunk.add(100.0);
            }
        }
        ticks(&mut app, 60 * 60);
        let g = app.world().resource::<Game>();
        assert!(g.dummies.iter().all(|d| d.body.fall_t <= 0.0));
    }

    #[test]
    fn bots_run_little_errands_sooner_or_later() {
        let mut app = app();
        bots_on(&mut app);
        put_player(&mut app, 30.0, 20.0);
        let mut did_something = false;
        for _ in 0..60 {
            ticks(&mut app, 60 * 5);
            let g = app.world().resource::<Game>();
            did_something |= g
                .dummies
                .iter()
                .any(|d| d.bot.drunk.meter > 1.0 || d.seat.is_some())
                || g.life.dazza.state != bbq_core::dazza::DazzaState::Cook;
            if did_something {
                break;
            }
        }
        assert!(did_something, "five minutes and no bar, smoko or meat run");
    }

    #[test]
    fn bots_start_fair_and_the_test_fixture_freezes_them() {
        let app = app();
        assert_eq!(
            app.world().resource::<Game>().options.bot_difficulty,
            bbq_core::bots::Difficulty::Fair
        );
        assert!(
            !app.world().resource::<Game>().options.bots_on,
            "the test fixture freezes them"
        );
    }

    #[test]
    fn the_leader_wears_the_crown() {
        let mut app = app();
        bots_on(&mut app);
        put_player(&mut app, 30.0, 20.0);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let id = g.dummies[1].id;
            g.board.award(
                &Rules {
                    mode: GameMode::FreeForAll,
                    friendly_fire: false,
                    phase: Phase::Play,
                },
                id,
                300,
            );
        }
        ticks(&mut app, 5);
        let g = app.world().resource::<Game>();
        assert!(g.dummies[1].crown && !g.dummies[0].crown && !g.dummies[2].crown);
    }

    #[test]
    fn nobody_gets_stuck_for_long_in_a_real_game() {
        let mut app = app();
        bots_on(&mut app);
        put_player(&mut app, 30.0, 20.0);
        let mut last = [(0.0f32, 0.0f32); 3];
        let mut still = [0.0f32; 3];
        let mut worst = 0.0f32;
        for sec in 0..120 {
            ticks(&mut app, 60);
            let g = app.world().resource::<Game>();
            for (i, d) in g.dummies.iter().enumerate() {
                let moved = (d.mover.x - last[i].0).hypot(d.mover.z - last[i].1);
                let parked = d.seat.is_some()
                    || d.bot.drunk.is_drinking()
                    || d.bot.winding
                    || d.body.stun > 0.0;
                if sec > 0 && moved < 0.3 && !parked {
                    still[i] += 1.0;
                    worst = worst.max(still[i]);
                } else {
                    still[i] = 0.0;
                }
                last[i] = (d.mover.x, d.mover.z);
            }
        }
        assert!(worst < 12.0, "a bot stood still for {worst} s");
    }

    // ---------------------------------------------------------------- Phase 7: rounds and modes

    fn start_with(app: &mut App, setup: crate::round::Setup) {
        app.world_mut().resource_mut::<Game>().round.setup = setup;
        app.world_mut().resource_scope(|w, mut g: Mut<Game>| {
            w.resource_scope(|w, mut p: Mut<Player>| {
                let mut y = w.resource_mut::<YardRes>();
                crate::round::start_round(&mut g, &mut p, &mut y);
            });
        });
        // the real game runs bots; the fixture froze them
        app.world_mut().resource_mut::<Game>().options.bots_on = true;
    }

    fn setup(mode: GameMode, bots: usize) -> crate::round::Setup {
        crate::round::Setup {
            mode,
            bots,
            round_len: 60.0,
            ..crate::round::Setup::default()
        }
    }

    #[test]
    fn a_round_counts_down_plays_and_blows_the_whistle() {
        let mut app = app();
        start_with(&mut app, setup(GameMode::FreeForAll, 3));
        assert_eq!(app.world().resource::<Game>().rules.phase, Phase::Countdown);
        // nobody moves in the countdown
        let before = {
            let g = app.world().resource::<Game>();
            g.dummies
                .iter()
                .map(|d| (d.mover.x, d.mover.z))
                .collect::<Vec<_>>()
        };
        app.world_mut().resource_mut::<Wanted>().wish = (1.0, 0.0);
        ticks(&mut app, 120);
        {
            let g = app.world().resource::<Game>();
            assert_eq!(g.rules.phase, Phase::Countdown);
            for (d, b) in g.dummies.iter().zip(&before) {
                assert!(
                    (d.mover.x - b.0).abs() < 0.2 && (d.mover.z - b.1).abs() < 0.2,
                    "a bot moved in the countdown"
                );
            }
            assert!(g.round.banner.is_some());
        }
        ticks(&mut app, 90); // 3.2 s is 192 ticks
        assert_eq!(app.world().resource::<Game>().rules.phase, Phase::Play);
        assert!((app.world().resource::<Game>().round.time_left - 60.0).abs() < 2.0);
        ticks(&mut app, 60 * 61);
        let g = app.world().resource::<Game>();
        assert_eq!(g.rules.phase, Phase::Results);
        assert!(g.round.results.is_some());
        assert!(g.round.mtch.over, "a single round is a whole match");
    }

    #[test]
    fn nothing_scores_after_the_whistle() {
        let mut app = app();
        start_with(&mut app, setup(GameMode::FreeForAll, 1));
        ticks(&mut app, 200 + 60 * 61);
        let scores: Vec<i32> = {
            let g = app.world().resource::<Game>();
            g.board.iter().map(|(_, s)| s.score).collect()
        };
        ticks(&mut app, 600);
        let g = app.world().resource::<Game>();
        let after: Vec<i32> = g.board.iter().map(|(_, s)| s.score).collect();
        assert_eq!(scores, after);
    }

    #[test]
    fn the_bot_count_is_synced_each_round() {
        let mut app = app();
        start_with(&mut app, setup(GameMode::FreeForAll, 7));
        {
            let g = app.world().resource::<Game>();
            assert_eq!(g.dummies.len(), 7);
            assert_eq!(g.board.iter().count(), 8);
            assert_eq!(g.crowd.brains.len(), 7);
        }
        start_with(&mut app, setup(GameMode::FreeForAll, 0));
        let g = app.world().resource::<Game>();
        assert!(g.dummies.is_empty(), "1v1 with no bots must always work");
        assert_eq!(g.board.iter().count(), 1);
        assert!(g.crowd.brains.is_empty());
    }

    #[test]
    fn a_round_with_no_bots_runs_and_ends() {
        let mut app = app();
        start_with(&mut app, setup(GameMode::FreeForAll, 0));
        ticks(&mut app, 200 + 60 * 61);
        let g = app.world().resource::<Game>();
        assert_eq!(g.rules.phase, Phase::Results);
        assert!(g.round.results.as_ref().unwrap().lines.len() >= 2);
    }

    #[test]
    fn everyone_starts_a_round_sober_with_empty_hands_and_a_scoreboard_of_zero() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.me.drunk.add(80.0);
            g.board.award(
                &Rules {
                    mode: GameMode::FreeForAll,
                    friendly_fire: false,
                    phase: Phase::Play,
                },
                PLAYER_ID,
                500,
            );
        }
        give_teddy(&mut app);
        start_with(&mut app, setup(GameMode::FreeForAll, 3));
        let g = app.world().resource::<Game>();
        assert_eq!(g.me.drunk.meter, 0.0);
        assert!(g.slots.is_empty());
        assert!(g.board.iter().all(|(_, s)| s.score == 0));
        assert!(g.world.items.len() >= 8, "the yard is stocked");
    }

    #[test]
    fn teams_are_dealt_and_wear_sashes_and_the_odd_one_out_is_the_wildcard() {
        let mut app = app();
        start_with(&mut app, setup(GameMode::Teams, 3)); // four people: 2 v 2
        {
            let g = app.world().resource::<Game>();
            let red = g.teams.count(Team::Red);
            let blue = g.teams.count(Team::Blue);
            assert_eq!((red, blue), (2, 2));
            assert!(g.dummies.iter().all(|d| d.team.is_some()));
        }
        start_with(&mut app, setup(GameMode::Teams, 2)); // three people: a Wildcard
        let g = app.world().resource::<Game>();
        assert_eq!(g.teams.count(Team::Wildcard), 1);
    }

    #[test]
    fn bots_do_not_target_their_own_team() {
        let mut app = app();
        start_with(&mut app, setup(GameMode::Teams, 3));
        ticks(&mut app, 200 + 60 * 40);
        let g = app.world().resource::<Game>();
        for (id, brain) in &g.crowd.brains {
            if let Some(t) = brain.target {
                assert!(!g.teams.same_team(*id, t), "bot {id} targeted teammate {t}");
            }
        }
    }

    #[test]
    fn teammates_pass_through_each_other_unless_friendly_fire_is_on() {
        for friendly in [false, true] {
            let mut app = app();
            let mut s = setup(GameMode::Teams, 3);
            s.friendly_fire = friendly;
            start_with(&mut app, s);
            {
                let mut g = app.world_mut().resource_mut::<Game>();
                g.options.bots_on = false;
                g.rules.phase = Phase::Play;
                // put the player and bot 0 on the same team, bot 1 far away
                g.teams.set(PLAYER_ID, Team::Red);
                let b0 = g.dummies[0].id;
                g.teams.set(b0, Team::Red);
                g.dummies[0].mover.x = 2.0;
                g.dummies[0].mover.z = 6.0;
                g.dummies[0].home = (2.0, 6.0);
                g.dummies[1].mover.x = -30.0;
                g.dummies[2].mover.x = 30.0;
                g.world.clear();
                for d in g.dummies.iter_mut() {
                    d.bot.wish = (0.0, 0.0);
                }
            }
            put_player(&mut app, 2.0, 0.0);
            give_teddy_at(&mut app, 2.0, 0.0);
            app.world_mut().resource_mut::<Player>().yaw = std::f32::consts::PI; // looking along +z, at bot 0
            app.world_mut().resource_mut::<Player>().pitch = 0.05;
            app.world_mut().resource_mut::<Wanted>().throw_down = true;
            ticks(&mut app, 40);
            app.world_mut().resource_mut::<Wanted>().throw_up = true;
            ticks(&mut app, 90);
            let g = app.world().resource::<Game>();
            let hit = g.board.get(g.dummies[0].id).unwrap().taken > 0;
            assert_eq!(
                hit,
                friendly,
                "friendly fire {friendly}: taken {}",
                g.board.get(g.dummies[0].id).unwrap().taken
            );
        }
    }

    fn give_teddy_at(app: &mut App, x: f32, z: f32) {
        let mut g = app.world_mut().resource_mut::<Game>();
        let mut rng = Rng::new(9);
        let id = g.world.spawn(ItemKind::Teddy, x, z, false, &mut rng);
        g.world.give(id, PLAYER_ID);
        g.slots.add(id);
    }

    #[test]
    fn the_team_with_more_points_wins_a_teams_round() {
        let mut app = app();
        start_with(&mut app, setup(GameMode::Teams, 3));
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.options.bots_on = false;
            let rules = Rules {
                mode: GameMode::Teams,
                friendly_fire: false,
                phase: Phase::Play,
            };
            let ids: Vec<_> = crate::round::everyone(&g);
            for id in ids {
                let team = g.teams.get(id).unwrap();
                g.board
                    .award(&rules, id, if team == Team::Blue { 200 } else { 50 });
            }
        }
        ticks(&mut app, 200 + 60 * 61);
        let g = app.world().resource::<Game>();
        assert_eq!(
            g.round.results.as_ref().unwrap().winner,
            Some(bbq_core::matchflow::Key::Team(Team::Blue))
        );
    }

    #[test]
    fn a_best_of_three_goes_on_until_someone_has_two_round_wins() {
        let mut app = app();
        let mut s = setup(GameMode::FreeForAll, 1);
        s.match_len = 3;
        s.round_len = 5.0;
        start_with(&mut app, s.clone());
        let mut rounds = 0;
        while !app.world().resource::<Game>().round.mtch.over && rounds < 3 {
            // you win each round: 100 points
            ticks(&mut app, 200);
            {
                let mut g = app.world_mut().resource_mut::<Game>();
                g.options.bots_on = false;
                g.board.award(
                    &Rules {
                        mode: GameMode::FreeForAll,
                        friendly_fire: false,
                        phase: Phase::Play,
                    },
                    PLAYER_ID,
                    100,
                );
            }
            ticks(&mut app, 60 * 6);
            rounds += 1;
            if !app.world().resource::<Game>().round.mtch.over {
                start_with(&mut app, s.clone());
            }
        }
        let g = app.world().resource::<Game>();
        assert_eq!(rounds, 2, "two straight wins take a best of three");
        assert!(g.round.mtch.over);
        assert_eq!(
            g.round.mtch.leader(),
            Some(bbq_core::matchflow::Key::Player(PLAYER_ID))
        );
    }

    // ---- Teddy Heist ----

    fn heist_app(teams: usize, bots: usize) -> App {
        let mut app = app();
        let mut s = setup(GameMode::Heist, bots);
        s.heist_teams = teams;
        start_with(&mut app, s);
        {
            // skip the countdown and freeze the bots so each test can set the scene
            let mut g = app.world_mut().resource_mut::<Game>();
            g.rules.phase = Phase::Play;
            g.round.time_left = 600.0;
            g.options.bots_on = false;
        }
        ticks(&mut app, 240); // the teddies are tossed in and settle
        app
    }

    fn base_of(app: &App, team: Team) -> bbq_core::heist::BaseDef {
        let g = app.world().resource::<Game>();
        bbq_core::heist::base_of(g.heist.as_ref().unwrap().teams, team).unwrap()
    }

    fn teddies_of(app: &App, team: Team) -> Vec<bbq_core::flight::ItemId> {
        let g = app.world().resource::<Game>();
        let ti = crate::heist_app::team_index(team).unwrap();
        g.world
            .items
            .values()
            .filter(|i| i.team == Some(ti))
            .map(|i| i.id)
            .collect()
    }

    #[test]
    fn heist_puts_the_player_in_their_base_with_teddies_in_every_base() {
        let app = heist_app(2, 3);
        let g = app.world().resource::<Game>();
        let p = app.world().resource::<Player>();
        let team = g.teams.get(PLAYER_ID).unwrap();
        let b = base_of(&app, team);
        assert!(
            bbq_core::heist::inside_base(b, p.mover.x, p.mover.z),
            "spawned outside the base"
        );
        // 4 players, 2 teams: 2 per team = 2 + 1 teddies each
        for t in [Team::Red, Team::Blue] {
            assert_eq!(teddies_of(&app, t).len(), 3, "{t:?}");
        }
        assert!(app.world().resource::<YardRes>().0.heist.is_some());
    }

    #[test]
    fn walking_over_an_enemy_teddy_steals_it() {
        let mut app = heist_app(2, 1);
        let me = app.world().resource::<Game>().teams.get(PLAYER_ID).unwrap();
        let enemy = if me == Team::Red {
            Team::Blue
        } else {
            Team::Red
        };
        let t = teddies_of(&app, enemy)[0];
        let at = app.world().resource::<Game>().world.items[&t].pos;
        let at2 = app.world().resource::<Game>().world.items[&t].pos;
        let _ = at;
        put_player(&mut app, at2.x, at2.z);
        ticks(&mut app, 5);
        let g = app.world().resource::<Game>();
        let pp = app.world().resource::<Player>();
        assert!(
            g.slots.ids().contains(&t),
            "should have picked up the enemy teddy: item {:?} player ({},{},{}) me {:?}",
            g.world
                .items
                .get(&t)
                .map(|i| (i.pos, i.state, i.rest_t, i.team)),
            pp.mover.x,
            pp.mover.y,
            pp.mover.z,
            g.teams.get(PLAYER_ID)
        );
    }

    #[test]
    fn walking_over_your_own_teddy_does_nothing_at_home_but_sends_a_stray_home() {
        let mut app = heist_app(2, 1);
        let me = app.world().resource::<Game>().teams.get(PLAYER_ID).unwrap();
        ticks(&mut app, 120);
        let mine = teddies_of(&app, me)[0];
        let at = app.world().resource::<Game>().world.items[&mine].pos;
        put_player(&mut app, at.x, at.z);
        ticks(&mut app, 5);
        assert!(
            !app.world().resource::<Game>().slots.ids().contains(&mine),
            "a teddy at home is left alone"
        );
        // now it is somewhere else in the yard
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let it = g.world.items.get_mut(&mine).unwrap();
            it.pos = V3::new(10.0, 0.3, 10.0);
            it.state = ItemState::Ground;
            it.rest_t = 1.0;
        }
        put_player(&mut app, 10.0, 10.0);
        ticks(&mut app, 3);
        let g = app.world().resource::<Game>();
        assert!(!g.slots.ids().contains(&mine));
        assert!(
            g.feed.iter().any(|(m, _)| m.contains("scurried back")),
            "{:?}",
            g.feed
        );
        ticks(&mut app, 120);
        let g = app.world().resource::<Game>();
        let b = bbq_core::heist::base_of(2, me).unwrap();
        let it = &g.world.items[&mine];
        assert!(
            it.pos.horiz_dist(V3::new(b.x, 0.0, b.z)) < 3.0,
            "went home to {:?}",
            it.pos
        );
    }

    #[test]
    fn carrying_an_enemy_teddy_into_your_base_banks_it_for_150() {
        let mut app = heist_app(2, 1);
        let me = app.world().resource::<Game>().teams.get(PLAYER_ID).unwrap();
        let enemy = if me == Team::Red {
            Team::Blue
        } else {
            Team::Red
        };
        ticks(&mut app, 120);
        let t = teddies_of(&app, enemy)[0];
        let at = app.world().resource::<Game>().world.items[&t].pos;
        put_player(&mut app, at.x, at.z);
        ticks(&mut app, 5);
        assert!(app.world().resource::<Game>().slots.ids().contains(&t));
        // not yet: outside the banking zone
        assert_eq!(
            app.world()
                .resource::<Game>()
                .heist
                .as_ref()
                .unwrap()
                .bank
                .get(me),
            0
        );
        let home = base_of(&app, me);
        put_player(&mut app, home.x, home.z);
        ticks(&mut app, 3);
        let g = app.world().resource::<Game>();
        assert_eq!(g.heist.as_ref().unwrap().bank.get(me), 1);
        assert_eq!(g.board.score(PLAYER_ID), 150, "only banking scores");
        assert_eq!(g.board.get(PLAYER_ID).unwrap().banked, 1);
        assert!(!g.world.items.contains_key(&t), "the banked teddy is gone");
        assert!(g.slots.is_empty());
    }

    #[test]
    fn a_stunned_thief_drops_the_teddy_and_cannot_bank() {
        let mut app = heist_app(2, 1);
        let me = app.world().resource::<Game>().teams.get(PLAYER_ID).unwrap();
        let enemy = if me == Team::Red {
            Team::Blue
        } else {
            Team::Red
        };
        ticks(&mut app, 120);
        let t = teddies_of(&app, enemy)[0];
        let at = app.world().resource::<Game>().world.items[&t].pos;
        put_player(&mut app, at.x, at.z);
        ticks(&mut app, 5);
        assert!(app.world().resource::<Game>().slots.ids().contains(&t));
        app.world_mut()
            .resource_mut::<Game>()
            .me
            .body
            .apply_hit(1.0, None);
        ticks(&mut app, 2);
        let g = app.world().resource::<Game>();
        assert!(!g.slots.ids().contains(&t), "the stun made them drop it");
        assert!(g.world.items.contains_key(&t));
    }

    #[test]
    fn a_stolen_teddy_thrown_over_the_fence_comes_home() {
        let mut app = heist_app(2, 1);
        let me = app.world().resource::<Game>().teams.get(PLAYER_ID).unwrap();
        let enemy = if me == Team::Red {
            Team::Blue
        } else {
            Team::Red
        };
        ticks(&mut app, 120);
        let t = teddies_of(&app, enemy)[0];
        let before = app
            .world()
            .resource::<Game>()
            .world
            .items
            .values()
            .filter(|i| i.team.is_some())
            .count();
        {
            // pretend it was thrown out of the yard
            let mut g = app.world_mut().resource_mut::<Game>();
            let it = g.world.items.get_mut(&t).unwrap();
            it.pos = V3::new(32.0, 3.0, 0.0);
            it.vel = V3::new(30.0, 4.0, 0.0);
            it.state = ItemState::Flying;
            it.live = true;
            it.thrower = Some(PLAYER_ID);
        }
        ticks(&mut app, 60);
        let g = app.world().resource::<Game>();
        let count = g.world.items.values().filter(|i| i.team.is_some()).count();
        assert_eq!(count, before, "the teddy count stays the same");
        assert!(
            g.feed.iter().any(|(m, _)| m.contains("came back")),
            "{:?}",
            g.feed
        );
    }

    #[test]
    fn thrown_hits_pay_nothing_in_heist_but_still_knock_people_over() {
        let mut app = heist_app(2, 3);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.world.clear();
            let b0 = g.dummies[0].id;
            g.teams.set(PLAYER_ID, Team::Red);
            g.teams.set(b0, Team::Blue);
            g.dummies[0].mover.x = 8.0;
            g.dummies[0].mover.z = 6.0;
            g.dummies[0].home = (8.0, 6.0);
            g.dummies[0].mover.y = 0.0;
            for d in g.dummies.iter_mut() {
                d.bot.wish = (0.0, 0.0);
            }
        }
        put_player(&mut app, 8.0, 0.0);
        give_teddy_at(&mut app, 8.0, 0.0);
        app.world_mut().resource_mut::<Player>().yaw = std::f32::consts::PI;
        app.world_mut().resource_mut::<Player>().pitch = 0.05;
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 40);
        app.world_mut().resource_mut::<Wanted>().throw_up = true;
        let mut hit = false;
        for _ in 0..90 {
            ticks(&mut app, 1);
            let d = &app.world().resource::<Game>().dummies[0];
            hit |= d.body.stun > 0.0 || d.body.is_down();
        }
        let g = app.world().resource::<Game>();
        let d = &g.dummies[0];
        assert!(hit, "it should have hit");
        assert_eq!(g.board.score(PLAYER_ID), 0, "no points for a hit in Heist");
        assert_eq!(g.board.score(d.id), 0, "and nothing lost");
    }

    #[test]
    fn the_heist_winner_is_the_team_with_most_banked() {
        let mut app = heist_app(2, 3);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let h = g.heist.as_mut().unwrap();
            h.bank.add(Team::Blue);
            h.bank.add(Team::Blue);
            h.bank.add(Team::Red);
            g.round.time_left = 0.01;
        }
        ticks(&mut app, 5);
        let g = app.world().resource::<Game>();
        assert_eq!(
            g.round.results.as_ref().unwrap().winner,
            Some(bbq_core::matchflow::Key::Team(Team::Blue))
        );
    }

    #[test]
    fn heist_bots_play_for_a_while_and_nobody_gets_stuck() {
        let mut app = heist_app(2, 3);
        app.world_mut().resource_mut::<Game>().options.bots_on = true;
        put_player(&mut app, 0.0, 22.0); // the player sits out in a corner
        let mut last = [(0.0f32, 0.0f32); 3];
        let mut still = [0.0f32; 3];
        let mut worst = 0.0f32;
        let mut moved_far = [0.0f32; 3];
        for sec in 0..150 {
            ticks(&mut app, 60);
            let g = app.world().resource::<Game>();
            for (i, d) in g.dummies.iter().enumerate() {
                let moved = (d.mover.x - last[i].0).hypot(d.mover.z - last[i].1);
                moved_far[i] += moved;
                if sec > 0
                    && moved < 0.3
                    && d.body.stun <= 0.0
                    && !d.bot.winding
                    && d.dragged.is_none()
                {
                    still[i] += 1.0;
                    worst = worst.max(still[i]);
                } else {
                    still[i] = 0.0;
                }
                last[i] = (d.mover.x, d.mover.z);
            }
        }
        let g = app.world().resource::<Game>();
        assert!(
            moved_far.iter().all(|m| *m > 40.0),
            "heist bots should be busy: {moved_far:?}"
        );
        assert!(worst < 15.0, "a heist bot stood still for {worst} s");
        // they went for the objective: either teddies moved or something got banked
        let banked: u32 = [Team::Red, Team::Blue]
            .iter()
            .map(|t| g.heist.as_ref().unwrap().bank.get(*t))
            .sum();
        let carried = g.dummies.iter().any(|d| !d.bot.slots.is_empty());
        let hits = g.feed.len();
        assert!(
            banked > 0 || carried || hits > 0,
            "bots did nothing about the teddies"
        );
    }

    // ---------------------------------------------------------------- Phase 8: the screens

    fn run_flow(app: &mut App, f: impl FnOnce(&mut Game, &mut Player, &mut YardRes, &mut crate::characters::Cast)) {
        app.world_mut().init_resource::<crate::characters::Cast>();
        app.world_mut().resource_scope(|w, mut g: Mut<Game>| {
            w.resource_scope(|w, mut p: Mut<Player>| {
                w.resource_scope(|w, mut y: Mut<YardRes>| {
                    let mut c = w.resource_mut::<crate::characters::Cast>();
                    f(&mut g, &mut p, &mut y, &mut c);
                });
            });
        });
    }

    #[test]
    fn the_menu_yard_has_three_bots_and_no_clock_and_they_leave_you_alone() {
        let mut app = app();
        let s = crate::menu::Settings::default();
        run_flow(&mut app, |g, p, y, _| crate::menu::attract(&s, g, p, y));
        app.world_mut().resource_mut::<Game>().options.bots_on = true;
        ticks(&mut app, 60 * 30);
        let g = app.world().resource::<Game>();
        assert!(g.attract);
        assert!(!g.round.timed);
        assert_eq!(g.dummies.len(), 3);
        assert_eq!(g.me.body.stun, 0.0, "nobody hits the player behind the menu");
        assert_eq!(g.board.get(PLAYER_ID).unwrap().taken, 0);
    }

    #[test]
    fn play_starts_a_round_with_the_chosen_options_and_menu_goes_back() {
        let mut app = app();
        let mut s = crate::menu::Settings::default();
        s.mode = GameMode::Teams;
        s.bots = 2;
        s.cheeky = true;
        s.rounds = 3;
        s.round_len = 60.0;
        s.friendly_fire = true;
        s.skill = bbq_core::bots::Difficulty::Spicy;
        s.falls = false;
        run_flow(&mut app, |g, p, y, c| crate::menu::begin(&s, g, p, y, c));
        {
            let g = app.world().resource::<Game>();
            assert!(!g.attract);
            assert!(g.round.timed);
            assert_eq!(g.rules.phase, Phase::Countdown);
            assert_eq!(g.rules.mode, GameMode::Teams);
            assert!(g.rules.friendly_fire);
            assert_eq!(g.dummies.len(), 2);
            assert_eq!(g.round.mtch.len, 3);
            assert!(g.options.adult);
            assert!(!g.options.falls_on);
            assert_eq!(g.options.bot_difficulty, bbq_core::bots::Difficulty::Spicy);
            assert!((g.round.setup.round_len - 60.0).abs() < 0.01);
        }
        let s2 = crate::menu::Settings::default();
        run_flow(&mut app, |g, p, y, _| crate::menu::attract(&s2, g, p, y));
        let g = app.world().resource::<Game>();
        assert!(g.attract && !g.round.timed);
        assert_eq!(g.rules.mode, GameMode::FreeForAll);
        assert!(g.teams.get(PLAYER_ID).is_none());
        assert_eq!(g.dummies.len(), 3);
    }

    #[test]
    fn the_results_card_words_a_win_a_loss_and_a_draw() {
        let mut app = app();
        let s = crate::menu::Settings::default();
        run_flow(&mut app, |g, p, y, c| crate::menu::begin(&s, g, p, y, c));
        let rules = Rules { mode: GameMode::FreeForAll, friendly_fire: false, phase: Phase::Play };
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.options.bots_on = false;
            g.board.award(&rules, PLAYER_ID, 300);
            g.rules.phase = Phase::Play;
            g.round.time_left = 0.01;
        }
        ticks(&mut app, 400);
        let g = app.world().resource::<Game>();
        assert!(g.round.results.is_some());
        let t = crate::results::build(g, &s);
        assert_eq!(t.title, "You own the yard!");
        assert_eq!(t.rows[0].name, "You");
        assert_eq!(t.again_label, "Play again");
        assert_eq!(t.rows.len(), 4);
    }

    #[test]
    fn a_tap_with_a_pool_noodle_slaps_and_a_hold_throws_it() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.world.clear();
            let mut rng = Rng::new(9);
            let id = g.world.spawn(ItemKind::Noodle, 8.0, 0.0, false, &mut rng);
            g.world.give(id, PLAYER_ID);
            g.slots.add(id);
            // a bot right in front of you, within slapping reach
            g.dummies[0].mover.x = 8.0;
            g.dummies[0].mover.z = 1.3;
            g.dummies[0].home = (8.0, 1.3);
        }
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 3);
        app.world_mut().resource_mut::<Wanted>().throw_up = true;
        ticks(&mut app, 3);
        {
            let g = app.world().resource::<Game>();
            let d = &g.dummies[0];
            assert!(d.body.stun > 0.0, "the slap should have stunned them");
            assert_eq!(g.board.score(PLAYER_ID), 50, "a noodle slap pays 50");
            assert!(g.life.me_swing > 0.0, "and the swing is drawn");
            // the noodle is still in your hand (it has 6 uses)
            assert_eq!(g.slots.len(), 1);
        }
        // a long hold throws it instead
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.dummies[0].mover.z = 7.0;
            g.dummies[0].home = (8.0, 7.0);
            g.dummies[0].body = bbq_core::stun::Body::default();
        }
        app.world_mut().resource_mut::<Wanted>().throw_down = true;
        ticks(&mut app, 40);
        app.world_mut().resource_mut::<Wanted>().throw_up = true;
        ticks(&mut app, 4);
        let g = app.world().resource::<Game>();
        assert_eq!(g.slots.len(), 0, "held for a while it flies");
    }

    #[test]
    fn pressing_play_after_the_menu_yard_starts_round_one_and_any_bot_count_is_fine() {
        for bots in [0usize, 1, 2, 3] {
            let mut app = app();
            let mut s = crate::menu::Settings::default();
            s.bots = bots;
            run_flow(&mut app, |g, p, y, _| crate::menu::attract(&s, g, p, y));
            run_flow(&mut app, |g, p, y, c| crate::menu::begin(&s, g, p, y, c));
            ticks(&mut app, 30);
            let g = app.world().resource::<Game>();
            assert_eq!(g.round.mtch.no, 1, "the first round after the menu is round 1");
            assert_eq!(g.dummies.len(), bots);
            assert_eq!(g.board.iter().count(), bots + 1);
            assert_eq!(g.crowd.brains.len(), bots);
        }
    }

    #[test]
    fn g_throws_away_what_you_hold_and_you_cannot_grab_it_straight_back() {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.world.clear();
            let mut rng = Rng::new(9);
            let a = g.world.spawn(ItemKind::Teddy, 8.0, 0.0, false, &mut rng);
            g.world.give(a, PLAYER_ID);
            g.slots.add(a);
            let b = g.world.spawn(ItemKind::Gnome, 8.0, 0.0, false, &mut rng);
            g.world.give(b, PLAYER_ID);
            g.slots.add(b);
        }
        let held = app.world().resource::<Game>().slots.selected().unwrap();
        app.world_mut().resource_mut::<Wanted>().drop_held = true;
        ticks(&mut app, 2);
        {
            let g = app.world().resource::<Game>();
            assert_eq!(g.slots.len(), 1, "one item is gone from your hands");
            assert!(!g.slots.ids().contains(&held));
            let it = &g.world.items[&held];
            assert_ne!(it.state, ItemState::Held);
            assert!(!it.live, "a dropped item is harmless");
            assert!(it.block.is_some(), "and blocked from being picked straight back up");
        }
        // standing right next to it for 2 seconds does not pick it up
        ticks(&mut app, 100);
        assert!(!app.world().resource::<Game>().slots.ids().contains(&held), "still blocked");
        // after the 2.5 s block it can be picked up again
        ticks(&mut app, 120);
        // (it landed somewhere in front of you; walk onto it)
        let at = app.world().resource::<Game>().world.items[&held].pos;
        put_player(&mut app, at.x, at.z);
        ticks(&mut app, 5);
        assert!(app.world().resource::<Game>().slots.ids().contains(&held), "picked up again later");
    }

    #[test]
    fn g_does_nothing_with_empty_hands_or_when_stunned() {
        let mut app = app();
        app.world_mut().resource_mut::<Wanted>().drop_held = true;
        ticks(&mut app, 2);
        assert!(app.world().resource::<Game>().slots.is_empty());
        give_teddy(&mut app);
        app.world_mut().resource_mut::<Game>().me.body.apply_hit(1.0, None);
        app.world_mut().resource_mut::<Wanted>().drop_held = true;
        ticks(&mut app, 2);
        assert_eq!(app.world().resource::<Game>().slots.len(), 1, "G is ignored while you are stunned");
    }
}
