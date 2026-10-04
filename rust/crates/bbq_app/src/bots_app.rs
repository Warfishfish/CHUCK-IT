//! Phase 6: the bots. This is the glue between the bot brain (`bbq_core::bots`, which only
//! decides) and the game: it shows each bot what it can see, then does what the brain asks with
//! the same rules the player is under (walking, picking up, winding up and throwing, catching,
//! slapping, drinking, sitting at smoko, taking meat and chest toys, helping mates up).

use bbq_core::bots::{
    self, Act, Command, FlyingView, GroundView, HeldView, OtherView, SelfView, Swing, WorldView,
};
use bbq_core::drinks;
use bbq_core::drunk_state::{self, DrunkState, Env};
use bbq_core::flight::{Item, ItemId, ItemState};
use bbq_core::hands::Slots;
use bbq_core::hitting::{self, Catcher};
use bbq_core::items::{DildoVariant, ItemKind, MEAT_PTS, Melee, SLAP_USES};
use bbq_core::melee::{self, Candidate};
use bbq_core::movement::EYE_HEIGHT;
use bbq_core::pose::SlapKind;
use bbq_core::scoring::{HitInput, Phase, SlapInput};
use bbq_core::smoko;
use bbq_core::vec::V3;
use bbq_core::yard::{self, BAR, CHEST_SPOTS, SMOKO_X, SMOKO_Z, Yard};
use bbq_core::{GameMode, PlayerId};
use bevy::prelude::*;

use crate::game::Game;
use crate::player::{PLAYER_ID, Player};

/// What a bot carries that the player keeps elsewhere (hands, catch window, drunk meter).
pub struct BotBody {
    pub slots: Slots,
    pub catcher: Catcher,
    pub drunk: DrunkState,
    /// Where it is walking, from its brain (world x, z).
    pub wish: (f32, f32),
    pub winding: bool,
    pub wind_progress: f32,
    /// Seconds left of its swing animation.
    pub swing_t: f32,
    /// Seconds spent sitting at smoko.
    pub smoko_t: f32,
    /// When it last took meat or something from the chest.
    pub grabbed_at: f32,
}

impl BotBody {
    pub fn new(phase: f32, rng: &mut bbq_core::rng::Rng) -> Self {
        BotBody {
            slots: Slots::default(),
            catcher: Catcher::default(),
            drunk: DrunkState::new(phase, rng),
            wish: (0.0, 0.0),
            winding: false,
            wind_progress: 0.0,
            swing_t: 0.0,
            smoko_t: 0.0,
            grabbed_at: -9.0,
        }
    }
}

/// F10 switches the bots off (they stand still like practice dummies) and on; F11 changes how
/// good they are.
pub struct BotsPlugin;

impl Plugin for BotsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, bot_keys);
    }
}

fn bot_keys(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>) {
    if keys.just_pressed(KeyCode::F10) {
        game.options.bots_on = !game.options.bots_on;
        let on = game.options.bots_on;
        game.say(if on {
            "Bots are on".to_string()
        } else {
            "Bots are frozen (practice dummies)".to_string()
        });
    }
    if keys.just_pressed(KeyCode::F11) {
        game.options.bot_difficulty = match game.options.bot_difficulty {
            bots::Difficulty::Easy => bots::Difficulty::Fair,
            bots::Difficulty::Fair => bots::Difficulty::Spicy,
            bots::Difficulty::Spicy => bots::Difficulty::Easy,
        };
        let d = format!("Bots are now {:?}", game.options.bot_difficulty);
        game.say(d);
    }
}

pub fn name_of(g: &Game, id: PlayerId) -> String {
    if id == PLAYER_ID {
        "You".to_string()
    } else {
        g.dummies
            .iter()
            .position(|d| d.id == id)
            .map(|i| {
                crate::characters::BLOB_NAMES[i % crate::characters::BLOB_NAMES.len()].to_string()
            })
            .unwrap_or_else(|| "Somebody".to_string())
    }
}

/// The right hand of a bot, where its throws start.
pub fn bot_hand(g: &Game, i: usize) -> V3 {
    let d = &g.dummies[i];
    let f = (d.face.sin(), d.face.cos());
    V3::new(
        d.mover.x + f.0 * 0.5,
        d.mover.y + EYE_HEIGHT - d.mover.sink - 0.3,
        d.mover.z + f.1 * 0.5,
    )
}

fn drunk_of(g: &Game, id: PlayerId) -> f32 {
    if id == PLAYER_ID {
        g.me.drunk.meter
    } else {
        g.dummies
            .iter()
            .find(|d| d.id == id)
            .map_or(0.0, |d| d.bot.drunk.meter)
    }
}

/// Bonus points for hitting someone while drunk, for whoever `id` is.
pub fn drunk_bonus_of(g: &Game, id: PlayerId) -> i32 {
    drinks::drunk_bonus(drunk_of(g, id), g.options.drunk_mode)
}

fn teammate(g: &Game, a: PlayerId, b: PlayerId) -> bool {
    g.teams.same_team(a, b)
}

fn held_views(g: &Game, slots: &Slots) -> (Vec<HeldView>, Option<usize>) {
    let mut v = Vec::new();
    for id in slots.ids() {
        if let Some(it) = g.world.items.get(id) {
            v.push(HeldView {
                id: *id,
                kind: it.kind,
                heist_team: it.team,
            });
        }
    }
    let sel = slots
        .selected()
        .and_then(|s| slots.ids().iter().position(|i| *i == s));
    (v, sel)
}

fn taken_seats(g: &Game) -> Vec<bool> {
    let n = smoko::chair_count(1 + g.dummies.len());
    let mut t = vec![false; n];
    for d in &g.dummies {
        if let Some(s) = d.seat
            && s < n
        {
            t[s] = true;
        }
    }
    if let Some(s) = g.life.seated
        && s.seat < n
    {
        t[s.seat] = true;
    }
    t
}

fn others_for(g: &Game, p: &Player, me: usize) -> Vec<OtherView> {
    let my_id = g.dummies[me].id;
    let mut v = Vec::new();
    v.push(OtherView {
        id: PLAYER_ID,
        pos: V3::new(p.mover.x, p.mover.y, p.mover.z),
        vel: V3::new(p.mover.vx, p.mover.vy, p.mover.vz),
        stun: g.me.body.stun,
        down_t: g.me.body.down_t,
        fall_t: g.me.body.fall_t,
        at_smoko: g.life.seated.is_some() || g.attract,
        is_bot: false,
        in_pool: p.mover.in_pool,
        teammate: teammate(g, my_id, PLAYER_ID),
        in_carry_with_me: false,
        has_our_teddy: false,
    });
    for (j, d) in g.dummies.iter().enumerate() {
        if j == me {
            continue;
        }
        v.push(OtherView {
            id: d.id,
            pos: V3::new(d.mover.x, d.mover.y, d.mover.z),
            vel: V3::new(d.mover.vx, d.mover.vy, d.mover.vz),
            stun: d.body.stun,
            down_t: d.body.down_t,
            fall_t: d.body.fall_t,
            at_smoko: d.seat.is_some() && d.naughty_t <= 0.0,
            is_bot: true,
            in_pool: d.mover.in_pool,
            teammate: teammate(g, my_id, d.id),
            in_carry_with_me: false,
            has_our_teddy: false,
        });
    }
    v
}

/// Everything the bots do in one fixed step. Runs before the people are moved.
pub fn step(g: &mut Game, p: &mut Player, yard: &Yard) {
    let dt = bbq_core::sim::TICK_DT;
    let now = g.now;
    // the leader wears the crown
    let leader = g.board.leader();
    for d in &mut g.dummies {
        d.crown = Some(d.id) == leader;
    }
    for d in &mut g.dummies {
        d.bot.catcher.tick(dt);
        d.bot.swing_t = (d.bot.swing_t - dt).max(0.0);
    }
    if !g.options.bots_on {
        for d in &mut g.dummies {
            d.bot.wish = (0.0, 0.0);
            d.bot.winding = false;
            d.bot.wind_progress = 0.0;
        }
        return;
    }
    let in_play = g.rules.in_play();
    for i in 0..g.dummies.len() {
        bot_body_tick(g, i, dt, in_play);
    }

    // who is where, shared by every bot this tick
    let flying: Vec<FlyingView> = g
        .world
        .items
        .values()
        .filter(|it| it.state == ItemState::Flying)
        .map(|it| FlyingView {
            id: it.id,
            pos: it.pos,
            vel: it.vel,
            thrower: it.thrower,
            live: it.live,
        })
        .collect();
    let seats_free: Vec<bool> = taken_seats(g).iter().map(|t| !t).collect();
    let chest_at = {
        let (x, z, _) = CHEST_SPOTS[yard.chest_spot % CHEST_SPOTS.len()];
        (x, z)
    };
    let features = yard.features;
    let adult = g.options.adult;
    let naughty = g.options.naughty && g.options.smoko_on;
    let counting = g.rules.phase == Phase::Countdown;
    let dazza_chasing = g.life.dazza.state == bbq_core::dazza::DazzaState::Chase;
    let chest_stock = g.life.chest.stock;
    let mode = g.rules.mode;
    let diff = g.options.bot_difficulty;

    for i in 0..g.dummies.len() {
        let id = g.dummies[i].id;
        if g.dummies[i].dragged.is_some() || g.dummies[i].naughty_t > 0.0 {
            // being dragged, or in the corner: not thinking about anything
            g.dummies[i].bot.wish = (0.0, 0.0);
            g.dummies[i].bot.winding = false;
            continue;
        }
        let others = others_for(g, p, i);
        let ground: Vec<GroundView> = g
            .world
            .items
            .values()
            .filter(|it| it.state == ItemState::Ground)
            .map(|it| GroundView {
                id: it.id,
                kind: it.kind,
                pos: it.pos,
                ground_y: it.ground_y,
                claimed_by_other: g.crowd.claimed_by_other(it.id, id),
                heist_team: it.team,
            })
            .collect();
        let (held, sel) = held_views(g, &g.dummies[i].bot.slots);
        let d = &g.dummies[i];
        let me = SelfView {
            id,
            pos: V3::new(d.mover.x, d.mover.y, d.mover.z),
            vel: V3::new(d.mover.vx, d.mover.vy, d.mover.vz),
            grounded: d.mover.grounded,
            in_pool: d.mover.in_pool,
            stun: d.body.stun,
            down_t: d.body.down_t,
            fall_t: d.body.fall_t,
            at_smoko: d.seat.is_some(),
            smoko_t: d.bot.smoko_t,
            carried: d.dragged.is_some(),
            drunk: d.bot.drunk.meter,
            drinking: d.bot.drunk.is_drinking(),
            catch_cd: d.bot.catcher.cooldown,
            hand: bot_hand(g, i),
            held,
            selected: sel,
        };
        let (heist_goal, carrying_stolen_teddy) = if mode == GameMode::Heist {
            crate::heist_app::goal_for_bot(g, i)
        } else {
            (None, false)
        };
        let d = &g.dummies[i];
        let _ = d;
        let w = WorldView {
            time: now,
            dt,
            yard,
            others: &others,
            flying: &flying,
            ground: &ground,
            leader,
            mode,
            counting_down: counting,
            features,
            adult,
            naughty,
            chest_stock,
            chest_at,
            free_seats: &seats_free,
            dazza_chasing,
            difficulty: diff,
            heist_goal,
            carrying_stolen_teddy,
        };
        let mut brain = match g.crowd.brains.remove(&id) {
            Some(b) => b,
            None => bots::BotBrain::new(&mut g.rng),
        };
        let cmd = brain.think(&me, &w, &mut g.rng);
        g.crowd.brains.insert(id, brain);
        g.crowd.claim(id, cmd.claim);
        act(g, p, yard, i, &cmd);
    }
}

/// A bot's own body ticking over: its drink, its sobering up, sitting at smoko.
fn bot_body_tick(g: &mut Game, i: usize, dt: f32, in_play: bool) {
    let drunk_mode = g.options.drunk_mode;
    let fall_dur = g.options.fall_duration;
    let d = &mut g.dummies[i];
    let at_smoko = d.seat.is_some() && d.sat_by_choice;
    let env = Env {
        in_play,
        at_smoko,
        grounded: d.mover.grounded,
        in_pool: d.mover.in_pool,
        moving: false,
        falls_on: false,
        drunk_mode,
        can_fall: false, // bots never stack it
        fall_duration: fall_dur,
    };
    let _ = d.bot.drunk.tick(dt, &env, &mut d.body, false, &mut g.rng);
    if at_smoko {
        d.bot.smoko_t += dt;
        d.bot.drunk.add(drunk_state_smoko_sip() * dt);
    } else {
        d.bot.smoko_t = 0.0;
    }
    d.drunk = d.bot.drunk.meter;
}

fn drunk_state_smoko_sip() -> f32 {
    drinks::SMOKO_SIP
}

/// Do what the brain asked for.
fn act(g: &mut Game, p: &mut Player, yard: &Yard, i: usize, cmd: &Command) {
    let _ = g.now;
    let id = g.dummies[i].id;
    g.dummies[i].face = cmd.face;
    g.dummies[i].bot.wish = cmd.wish;
    g.dummies[i].bot.winding = cmd.winding;
    g.dummies[i].bot.wind_progress = cmd.wind_progress;
    let can_act = g.rules.in_play()
        && g.dummies[i].body.stun <= 0.0
        && !g.dummies[i].body.is_down()
        && g.dummies[i].seat.is_none();
    if cmd.jump && g.dummies[i].body.stun <= 0.0 {
        let stunned = g.dummies[i].body.stun > 0.0;
        g.dummies[i].mover.try_jump(stunned);
    }
    if cmd.catch && can_act {
        g.dummies[i].bot.catcher.press(true);
    }
    if let Some(t) = cmd.throw
        && can_act
    {
        throw_item(g, i, t);
    }
    if let Some(sw) = cmd.swing
        && can_act
    {
        swing(g, p, i, sw);
    }
    if let Some(a) = cmd.act {
        do_act(g, p, yard, i, a);
    }
    // pick things up as it walks over them
    let d = &g.dummies[i];
    if d.body.stun <= 0.0
        && g.rules.phase != Phase::Countdown
        && d.seat.is_none()
        && d.dragged.is_none()
    {
        let (pos, held) = (V3::new(d.mover.x, d.mover.y, d.mover.z), d.bot.slots.len());
        if let Some(item) = crate::heist_app::try_pickup(g, id, pos, held, false, true) {
            g.world.give(item, id);
            g.dummies[i].bot.slots.add(item);
        }
    }
}

fn throw_item(g: &mut Game, i: usize, t: bots::Throw) {
    let id = g.dummies[i].id;
    if g.dummies[i].bot.slots.selected() != Some(t.item) {
        return;
    }
    // power-ups are for players: bots never throw at power charge
    if g.world.throw(t.item, id, t.from, t.vel, t.charge, false) {
        g.dummies[i].bot.slots.remove(t.item);
        g.board.count_throw(id);
        g.dummies[i].anim.start_swing();
        g.dummies[i].bot.swing_t = melee::SWING_TIME;
    }
}

/// Use up a slapping item; it falls apart at zero.
fn use_up(g: &mut Game, i: usize, item: ItemId) {
    let Some(it) = g.world.items.get_mut(&item) else {
        return;
    };
    let left = it
        .uses
        .or(it.kind.starting_uses())
        .unwrap_or(SLAP_USES)
        .saturating_sub(1);
    it.uses = Some(left);
    if left == 0 {
        g.dummies[i].bot.slots.remove(item);
        g.world.remove(item);
    }
}

fn swing(g: &mut Game, p: &mut Player, i: usize, sw: Swing) {
    let Some(item) = g.dummies[i].bot.slots.selected() else {
        return;
    };
    let Some(it) = g.world.items.get(&item) else {
        return;
    };
    let (kind, variant) = (it.kind, it.variant);
    if kind.def().melee == Melee::None {
        return;
    }
    g.dummies[i].anim.start_swing();
    g.dummies[i].bot.swing_t = melee::SWING_TIME;
    let Swing::At(victim) = sw else {
        return; // swung at nothing
    };
    let id = g.dummies[i].id;
    let apos = V3::new(
        g.dummies[i].mover.x,
        g.dummies[i].mover.y,
        g.dummies[i].mover.z,
    );
    // must really be in reach and in front of it
    let face = (g.dummies[i].face.sin(), g.dummies[i].face.cos());
    let tpos = if victim == PLAYER_ID {
        V3::new(p.mover.x, p.mover.y, p.mover.z)
    } else if let Some(v) = g.dummies.iter().find(|d| d.id == victim) {
        V3::new(v.mover.x, v.mover.y, v.mover.z)
    } else {
        return;
    };
    let cand = Candidate {
        id: victim,
        pos: tpos,
        down_t: 0.0,
        at_smoko: false,
        teammate: false,
    };
    if melee::pick_target(apos, face, &[cand], true).is_none() {
        return;
    }
    let att = crate::life::Attacker {
        id,
        pos: apos,
        drunk_bonus: drunk_bonus_of(g, id),
        name: name_of(g, id),
        is_player: false,
    };
    if victim == PLAYER_ID {
        slap_player(g, p, &att, kind, variant);
    } else if let Some(j) = g.dummies.iter().position(|d| d.id == victim) {
        crate::life::slap_dummy(g, &att, j, kind, variant);
    }
    use_up(g, i, item);
}

/// A bot slaps the player.
fn slap_player(
    g: &mut Game,
    p: &mut Player,
    att: &crate::life::Attacker,
    kind: ItemKind,
    variant: Option<DildoVariant>,
) {
    if g.life.seated.is_some() {
        return;
    }
    let me = V3::new(p.mover.x, 0.0, p.mover.z);
    let dir = melee::direction(att.pos, me);
    let same_team = teammate(g, att.id, PLAYER_ID) && !g.rules.friendly_fire;
    if same_team {
        return;
    }
    let leader = g.board.leader() == Some(PLAYER_ID);
    p.shake = p.shake.max(0.25);
    g.me.drunk.cancel_drink();
    g.wind.cancel();
    match kind.def().melee {
        Melee::Stun => {
            let fx = melee::stun_slap(kind);
            let res = melee::apply(&mut g.me.body, &mut p.mover, dir, &fx);
            g.board.stun_slap(
                &g.rules,
                att.id,
                PLAYER_ID,
                MEAT_PTS,
                att.drunk_bonus,
                false,
            );
            let verb = match kind {
                ItemKind::Fish => "fish-slapped",
                ItemKind::Noodle => "noodled",
                _ => "steaked",
            };
            g.popup(format!("{} {verb} you!", att.name), true);
            g.say(format!(
                "{} {verb} you{}",
                att.name,
                if res.stunned { "" } else { " (no new stun)" }
            ));
        }
        Melee::Down => {
            let variant = variant.unwrap_or(DildoVariant::Classic);
            let roll = melee::roll_dildo(&mut g.rng, variant);
            let fx = melee::dildo_slap(&roll);
            melee::apply(&mut g.me.body, &mut p.mover, dir, &fx);
            g.board.dildo_slap(
                &g.rules,
                att.id,
                PLAYER_ID,
                &SlapInput {
                    victim_is_leader: leader,
                    drunk_bonus: att.drunk_bonus,
                    variant,
                    crit: roll.crit,
                    same_team: false,
                },
            );
            let pose = match roll.pose {
                SlapKind::SentFlying => "SENT FLYING!",
                SlapKind::Cartwheel => "CARTWHEEL!",
                SlapKind::Timber => "TIMBERRR!",
            };
            g.popup(format!("{} slapped you: {pose}", att.name), true);
            g.say(format!(
                "{} dildo-slapped you{}",
                att.name,
                if roll.crit { " CRITICAL!" } else { "" }
            ));
        }
        Melee::None => {}
    }
}

fn do_act(g: &mut Game, p: &mut Player, yard: &Yard, i: usize, a: Act) {
    let now = g.now;
    let id = g.dummies[i].id;
    match a {
        Act::Drink => {
            let d = &g.dummies[i];
            let spot = if yard.features.bar {
                drunk_state::bar_spot(&BAR, d.mover.x, d.mover.y, d.mover.z)
            } else {
                None
            };
            let env = Env {
                in_play: g.rules.in_play(),
                at_smoko: false,
                grounded: d.mover.grounded,
                in_pool: d.mover.in_pool,
                moving: false,
                falls_on: false,
                drunk_mode: g.options.drunk_mode,
                can_fall: false,
                fall_duration: g.options.fall_duration,
            };
            let d = &mut g.dummies[i];
            d.bot.drunk.start_drink(spot, &d.body, &env);
        }
        Act::Sit(seat) => {
            let n = smoko::chair_count(1 + g.dummies.len());
            if seat >= n || taken_seats(g)[seat] || !yard.features.smoko {
                return;
            }
            let (x, z) = smoko::seat_pos(seat, n);
            let d = &mut g.dummies[i];
            d.seat = Some(seat);
            d.sat_by_choice = true;
            d.naughty_t = 0.0;
            d.bot.smoko_t = 0.0;
            d.bot.drunk.cancel_drink();
            d.bot.winding = false;
            d.mover.x = x;
            d.mover.z = z;
            d.mover.vx = 0.0;
            d.mover.vz = 0.0;
            d.face = (SMOKO_X - x).atan2(SMOKO_Z - z);
        }
        Act::StandUp => stand_bot(g, i),
        Act::TakeMeat(kind) => {
            if !yard.features.bbq
                || g.dummies[i].bot.slots.len() >= 2
                || now - g.dummies[i].bot.grabbed_at < 0.8
            {
                return;
            }
            g.dummies[i].bot.grabbed_at = now;
            let (x, z) = (g.dummies[i].mover.x, g.dummies[i].mover.z);
            let item = g.world.spawn(kind, x, z, false, &mut g.rng);
            if let Some(it) = g.world.items.get_mut(&item) {
                it.uses = Some(SLAP_USES);
            }
            g.world.give(item, id);
            g.dummies[i].bot.slots.add(item);
            let adult = g.options.adult;
            if let Some(bbq_core::dazza_brain::Event::Say(t)) =
                g.life.dazza.meat_taken(id, adult, &mut g.rng)
            {
                g.life.dazza_says(t);
            }
        }
        Act::TakeFromChest => {
            if !yard.features.chest || !g.options.adult || g.dummies[i].bot.slots.len() >= 2 {
                return;
            }
            let (cx, cz, _) = CHEST_SPOTS[yard.chest_spot % CHEST_SPOTS.len()];
            let dist = (g.dummies[i].mover.x - cx).hypot(g.dummies[i].mover.z - cz);
            let held = g.dummies[i].bot.slots.len();
            if let Ok(variant) = g.life.chest.take(dist, held, &mut g.rng) {
                let (x, z) = (g.dummies[i].mover.x, g.dummies[i].mover.z);
                let item = g.world.spawn(ItemKind::Dildo, x, z, false, &mut g.rng);
                if let Some(it) = g.world.items.get_mut(&item) {
                    it.variant = Some(variant);
                    it.uses = Some(SLAP_USES);
                }
                g.world.give(item, id);
                g.dummies[i].bot.slots.add(item);
                g.say(format!(
                    "{} got a {} from the chest",
                    name_of(g, id),
                    variant.def().label
                ));
            }
        }
        Act::HelpUp(target) => {
            let pts = g.board.award(&g.rules, id, drunk_state::HELP_PTS);
            if target == PLAYER_ID {
                if g.me.body.fall_t > 0.0 {
                    g.me.drunk.helped_up(&mut g.me.body);
                    p.shake = p.shake.max(0.05);
                    g.popup(format!("{} helped you up", name_of(g, id)), false);
                }
            } else if let Some(j) = g.dummies.iter().position(|d| d.id == target) {
                g.dummies[j].body.get_up();
                g.say(format!(
                    "{} helped {} up (+{pts})",
                    name_of(g, id),
                    name_of(g, target)
                ));
            }
        }
    }
}

/// Get a bot out of its smoko chair.
pub fn stand_bot(g: &mut Game, i: usize) {
    let d = &mut g.dummies[i];
    if let Some(seat) = d.seat.take() {
        let n = smoko::chair_count(1 + g.dummies.len());
        let (x, z) = smoko::seat_pos(seat.min(n - 1), n);
        let (sx, sz) = smoko::stand_pos(x, z);
        let d = &mut g.dummies[i];
        d.mover.x = sx;
        d.mover.z = sz;
        d.sat_by_choice = false;
        d.bot.smoko_t = 0.0;
    }
}

/// A thrown item hit the player.
#[allow(clippy::too_many_arguments)]
pub fn hit_player(
    g: &mut Game,
    p: &mut Player,
    kind: ItemKind,
    thrower: Option<PlayerId>,
    dir: V3,
    flatten: bool,
    charge: f32,
    start: V3,
) {
    let item = Item::new(0, kind, V3::ZERO);
    let res = hitting::apply_item_hit(&mut g.me.body, &mut p.mover, &item, dir, flatten);
    g.me.drunk.cancel_drink();
    g.wind.cancel();
    p.shake = p.shake.max(0.12 + kind.def().knock * 0.012);
    let leader = g.board.leader() == Some(PLAYER_ID);
    let bonus = thrower.map_or(0, |t| drunk_bonus_of(g, t));
    let same_team = thrower.is_some_and(|t| teammate(g, t, PLAYER_ID));
    let out = g.board.thrown_hit(
        &g.rules,
        thrower,
        PLAYER_ID,
        &HitInput {
            charge,
            dist: start.horiz_dist(V3::new(p.mover.x, 0.0, p.mover.z)),
            victim_is_leader: leader,
            drunk_bonus: bonus,
            bum_out: false,
            same_team,
        },
    );
    let who = thrower.map_or("Somebody".to_string(), |t| name_of(g, t));
    let mut msg = format!("{who} hit you");
    if out.victim_loss > 0 {
        msg += &format!(" (-{})", out.victim_loss);
    }
    if flatten && res.effect.knocked_down {
        msg += " KNOCKED OVER!";
    }
    g.say(msg);
    g.popup(format!("{who} got you!"), true);
}

/// Items that thrown or knocked about by bots are told apart from yours.
#[allow(dead_code)]
pub fn is_bot_item(g: &Game, item: ItemId) -> bool {
    g.world
        .items
        .get(&item)
        .and_then(|it| it.holder)
        .is_some_and(|h| h != PLAYER_ID)
}

/// Where a bot's held item is drawn (in its right hand).
pub fn held_item_pos(g: &Game, holder: PlayerId) -> Option<V3> {
    let i = g.dummies.iter().position(|d| d.id == holder)?;
    let d = &g.dummies[i];
    let f = (d.face.sin(), d.face.cos());
    let right = (f.1, -f.0);
    Some(V3::new(
        d.mover.x + f.0 * 0.3 + right.0 * 0.45,
        d.mover.y + 0.95 - d.mover.sink,
        d.mover.z + f.1 * 0.3 + right.1 * 0.45,
    ))
}

/// Is `holder`'s selected item this one?
pub fn is_selected_by_bot(g: &Game, holder: PlayerId, item: ItemId) -> bool {
    g.dummies
        .iter()
        .find(|d| d.id == holder)
        .is_some_and(|d| d.bot.slots.selected() == Some(item))
}

#[allow(dead_code)]
fn _unused(_: GameMode, _: &Yard, _: yard::Feature) {}
