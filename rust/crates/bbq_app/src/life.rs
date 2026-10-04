//! Phase 5B: life in the yard. Slaps, Dazza the cook, the meat table, the Cheeky chest, smoko
//! and the Naughty Corner, grabbing and throwing someone who is down, the pool and trampoline
//! bonuses, and emotes. The rules live in `bbq_core`; this file is the glue that runs them
//! each step and tells the player what happened.

use bbq_core::PlayerId;
use bbq_core::carry::{self, Drag, Party, Release as Drop};
use bbq_core::chest::{self, Chest};
use bbq_core::dazza::DazzaState;
use bbq_core::dazza_brain::{self, Brain, Event as DazzaEvent, Person, SlapResult};
use bbq_core::emotes::{self, Place};
use bbq_core::flight::ItemState;
use bbq_core::hitting;
use bbq_core::items::{DildoVariant, ItemKind, MEAT_PTS, Melee, SLAP_USES};
use bbq_core::melee::{self, Candidate, SlapClock};
use bbq_core::pose::{DraggerInfo, SlapKind};
use bbq_core::scoring::{self, Phase, SlapInput};
use bbq_core::smoko::{self, Over, Seated};
use bbq_core::vec::V3;
use bbq_core::yard::{self, CHEST_SPOTS, MEAT_TABLE};

use crate::game::Game;
use crate::player::{PLAYER_ID, Player, Wanted};

/// The player dragging someone by the ankles.
pub struct Carry {
    /// Index into `Game::dummies`.
    pub victim: usize,
    pub drag: Drag,
}

/// Everything about yard life that isn't the player's body or the items.
pub struct Life {
    pub dazza: Brain,
    /// Dazza's latest line; the picture shows it when `say_seq` changes.
    pub say_text: String,
    pub say_seq: u32,
    /// Bumps each time his spatula swing starts.
    pub swing_seq: u32,
    /// Bumps each time the grill is flipped.
    pub flip_seq: u32,
    pub slap: SlapClock,
    /// How long your own swing animation has left.
    pub me_swing: f32,
    pub chest: Chest,
    /// You, sitting at smoko.
    pub seated: Option<Seated>,
    pub emote_at: f32,
    pub carry: Option<Carry>,
    /// When F went down while carrying (a long press throws).
    pub f_down: Option<f32>,
    /// Per dummy: can't be grabbed again until this time.
    pub grab_immune: Vec<f32>,
    pub meat_at: f32,
}

impl Life {
    pub fn new() -> Self {
        Life {
            dazza: Brain::default(),
            say_text: String::new(),
            say_seq: 0,
            swing_seq: 0,
            flip_seq: 0,
            slap: SlapClock::default(),
            me_swing: 0.0,
            chest: Chest::default(),
            seated: None,
            emote_at: -9.0,
            carry: None,
            f_down: None,
            grab_immune: Vec::new(),
            meat_at: -9.0,
        }
    }

    pub fn dazza_says(&mut self, text: &str) {
        self.say_text = text.to_string();
        self.say_seq += 1;
    }
}

impl Default for Life {
    fn default() -> Self {
        Self::new()
    }
}

fn me_pos(p: &Player) -> V3 {
    V3::new(p.mover.x, p.mover.y, p.mover.z)
}

fn facing(p: &Player) -> (f32, f32) {
    (-p.yaw.sin(), -p.yaw.cos())
}

fn dummy_name(i: usize) -> &'static str {
    crate::characters::BLOB_NAMES[i % crate::characters::BLOB_NAMES.len()]
}

fn people_count(g: &Game) -> usize {
    1 + g.dummies.len()
}

// ------------------------------------------------------------------ slaps

/// Whoever is doing the slapping: you, or a bot.
pub struct Attacker {
    pub id: PlayerId,
    pub pos: V3,
    pub drunk_bonus: i32,
    pub name: String,
    pub is_player: bool,
}

/// Take one use off a slapping item; it falls apart at zero.
fn use_up(g: &mut Game, id: bbq_core::flight::ItemId) {
    let Some(it) = g.world.items.get_mut(&id) else {
        return;
    };
    let left = it
        .uses
        .or(it.kind.starting_uses())
        .unwrap_or(SLAP_USES)
        .saturating_sub(1);
    it.uses = Some(left);
    if left == 0 {
        let name = item_name(it.kind);
        g.slots.remove(id);
        g.world.remove(id);
        g.say(format!("Your {name} fell apart."));
    }
}

fn item_name(k: ItemKind) -> &'static str {
    match k {
        ItemKind::Steak => "Raw Steak",
        ItemKind::Fish => "Fish",
        ItemKind::Noodle => "Pool Noodle",
        ItemKind::Dildo => "Dildo",
        _ => "item",
    }
}

fn candidates(g: &Game) -> Vec<Candidate> {
    g.dummies
        .iter()
        .map(|d| Candidate {
            id: d.id,
            pos: V3::new(d.mover.x, d.mover.y, d.mover.z),
            down_t: d.body.down_t.max(d.body.fall_t),
            at_smoko: d.seat.is_some(),
            teammate: false,
        })
        .collect()
}

fn dazza_in_reach(g: &Game, me: V3, face: (f32, f32)) -> bool {
    let d = &g.life.dazza;
    d.state != DazzaState::Ko
        && melee::in_front(me, face, d.pos)
        && (d.pos.y - me.y).abs() < melee::VERTICAL
}

/// Can the player swing at all right now?
fn can_swing(g: &Game, armed: bool) -> bool {
    g.rules.in_play()
        && g.me.body.stun <= 0.0
        && g.me.body.fall_t <= 0.0
        && g.life.seated.is_none()
        && g.life.carry.is_none()
        && !g.me.drunk.is_drinking()
        && g.life.slap.ready(g.now, armed)
}

/// Left click with a slapping item in hand (or a tap with the noodle).
pub fn player_slap(g: &mut Game, p: &mut Player) {
    let Some(id) = g.slots.selected() else {
        return;
    };
    let Some(item) = g.world.items.get(&id) else {
        return;
    };
    let (kind, variant) = (item.kind, item.variant);
    if kind.def().melee == Melee::None || !can_swing(g, true) {
        return;
    }
    g.life.slap.mark(g.now);
    g.life.me_swing = melee::SWING_TIME;
    let me = me_pos(p);
    let face = facing(p);
    let cands = candidates(g);
    if let Some(vid) = melee::pick_target(me, face, &cands, false) {
        let Some(i) = g.dummies.iter().position(|d| d.id == vid) else {
            return;
        };
        let att = Attacker {
            id: PLAYER_ID,
            pos: me,
            drunk_bonus: drunk_bonus(g),
            name: "You".to_string(),
            is_player: true,
        };
        slap_dummy(g, &att, i, kind, variant);
        use_up(g, id);
    } else if dazza_in_reach(g, me, face) && slap_dazza(g, kind) {
        use_up(g, id);
    }
}

pub fn slap_dummy(
    g: &mut Game,
    att: &Attacker,
    i: usize,
    kind: ItemKind,
    variant: Option<DildoVariant>,
) {
    let me = att.pos;
    let dpos = V3::new(g.dummies[i].mover.x, 0.0, g.dummies[i].mover.z);
    let dir = melee::direction(me, dpos);
    let vid = g.dummies[i].id;
    let name = dummy_name(i);
    let leader = g.board.leader() == Some(vid);
    let drunk = att.drunk_bonus;
    let now = g.now;
    g.dummies[i].last_hit = Some((att.id, now));
    let sign = if g.rng.chance(0.5) { -1.0 } else { 1.0 };
    match kind.def().melee {
        Melee::Stun => {
            let fx = melee::stun_slap(kind);
            let d = &mut g.dummies[i];
            let res = melee::apply(&mut d.body, &mut d.mover, dir, &fx);
            d.anim.tumble(dir, fx.knock, sign);
            d.anim.dizzy = fx.dizzy;
            d.anim.start_swing();
            // steak, fish and noodle all pay 50
            let gain = g
                .board
                .stun_slap(&g.rules, att.id, vid, MEAT_PTS, drunk, false);
            let verb = match kind {
                ItemKind::Fish => "fish-slapped",
                ItemKind::Noodle => "noodled",
                _ => "steaked",
            };
            let mut msg = if att.is_player {
                format!("You {verb} {name}! +{gain}")
            } else {
                format!("{} {verb} {name}", att.name)
            };
            if !res.stunned {
                msg += " (already stunned, no new stun)";
            }
            if kind == ItemKind::Fish {
                let smell = melee::fish_smell(&mut g.rng);
                if smell != melee::Smell::None {
                    g.dummies[i].smelly = true;
                    g.dummies[i].smell_t = smell.seconds();
                    msg += if smell == melee::Smell::Rancid {
                        " THAT FISH WAS OFF!"
                    } else {
                        " (pong)"
                    };
                }
            }
            g.say(msg);
        }
        Melee::Down => {
            let variant = variant.unwrap_or(DildoVariant::Classic);
            let roll = melee::roll_dildo(&mut g.rng, variant);
            let fx = melee::dildo_slap(&roll);
            let d = &mut g.dummies[i];
            let res = melee::apply(&mut d.body, &mut d.mover, dir, &fx);
            d.anim.start_slap(roll.pose, roll.down);
            d.anim.tumble(dir, fx.knock * 0.4, sign);
            let out = g.board.dildo_slap(
                &g.rules,
                att.id,
                vid,
                &SlapInput {
                    victim_is_leader: leader,
                    drunk_bonus: drunk,
                    variant,
                    crit: roll.crit,
                    same_team: g.teams.same_team(att.id, vid),
                },
            );
            let pose = match roll.pose {
                SlapKind::SentFlying => "SENT FLYING!",
                SlapKind::Cartwheel => "CARTWHEEL!",
                SlapKind::Timber => "TIMBERRR!",
            };
            let mut msg = if att.is_player {
                format!("{} {pose} +{}", variant.def().label, out.gain)
            } else {
                format!("{} slapped {name}: {pose}", att.name)
            };
            if roll.crit {
                msg += " CRITICAL!";
            }
            if !res.knocked_down {
                msg += " (they were already down)";
            }
            g.say(msg);
            if att.is_player {
                g.popup(pose, true);
            }
        }
        Melee::None => {}
    }
}

/// Bare-handed slap (Cheeky mode, nothing in your hands).
pub fn bare_slap(g: &mut Game, p: &mut Player) {
    if !g.options.adult || !g.slots.is_empty() || !can_swing(g, false) {
        return;
    }
    let me = me_pos(p);
    let cands = candidates(g);
    let Some(vid) = melee::pick_target(me, facing(p), &cands, false) else {
        return;
    };
    let Some(i) = g.dummies.iter().position(|d| d.id == vid) else {
        return;
    };
    g.life.slap.mark(g.now);
    g.life.me_swing = melee::SWING_TIME;
    let dpos = V3::new(g.dummies[i].mover.x, 0.0, g.dummies[i].mover.z);
    let dir = melee::direction(me, dpos);
    let fx = melee::silly_slap();
    let sign = if g.rng.chance(0.5) { -1.0 } else { 1.0 };
    let now = g.now;
    let d = &mut g.dummies[i];
    d.last_hit = Some((PLAYER_ID, now));
    melee::apply(&mut d.body, &mut d.mover, dir, &fx);
    d.anim.tumble(dir, fx.knock, sign);
    d.anim.start_swing();
    let drunk = drunk_bonus(g);
    let gain = g.board.silly_slap(&g.rules, PLAYER_ID, vid, drunk, false);
    let n = (g.rng.f32() * 3.0) as usize;
    let name = melee::SILLY_NAMES[n.min(2)];
    g.popup(format!("{name} +{gain}"), false);
    g.say(format!(
        "You gave {} a {}",
        dummy_name(i),
        name.trim_end_matches('!')
    ));
}

fn drunk_bonus(g: &Game) -> i32 {
    bbq_core::drinks::drunk_bonus(g.me.drunk.meter, g.options.drunk_mode)
}

/// Returns true if the slap landed (and so used up a charge).
fn slap_dazza(g: &mut Game, kind: ItemKind) -> bool {
    let stun_item = kind.def().melee == Melee::Stun;
    let adult = g.options.adult;
    let (res, line) = g
        .life
        .dazza
        .slapped(PLAYER_ID, stun_item, adult, &mut g.rng);
    if res == SlapResult::Ignored {
        return false;
    }
    if let Some(l) = line {
        g.life.dazza_says(l);
    }
    match res {
        SlapResult::Stunned { secs } => {
            let pts = g.board.award(&g.rules, PLAYER_ID, scoring::DAZZA_STUN);
            g.popup(format!("Dazza stunned {secs:.1}s! +{pts}"), false);
        }
        other => {
            let idx = other.outcome_index().unwrap_or(0);
            let pts = g.board.award_dazza_slap(&g.rules, PLAYER_ID, idx);
            let label = ["KNOCKED OUT COLD!", "BERSERK!", "FLIPPED THE BARBIE!"][idx];
            g.popup(format!("{label} +{pts}"), true);
            if other == SlapResult::Flipped {
                flip_grill(g);
            }
            if let SlapResult::KnockedOut { bum_out: true } = other {
                gnome_from_bum(g);
            }
        }
    }
    true
}

/// Five bits of meat fly off the grill.
fn flip_grill(g: &mut Game) {
    g.life.flip_seq += 1;
    for _ in 0..5 {
        let kind = if g.rng.chance(0.5) {
            ItemKind::Steak
        } else {
            ItemKind::Fish
        };
        let id = g.world.spawn(kind, -6.0, -18.0, false, &mut g.rng);
        let an = g.rng.range(-0.3, std::f32::consts::PI + 0.3);
        let (sp1, sp2, up) = (
            g.rng.range(3.0, 7.0),
            g.rng.range(3.0, 7.0),
            g.rng.range(5.0, 8.0),
        );
        let jx = g.rng.range(-0.3, 0.3);
        if let Some(it) = g.world.items.get_mut(&id) {
            it.pos = V3::new(-6.0 + jx, 1.4, -18.0);
            it.vel = V3::new(an.cos() * sp1, up, an.sin() * sp2 + 2.0);
            it.state = ItemState::Flying;
            it.live = false;
            it.uses = kind.starting_uses();
        }
    }
}

/// Cheeky mode: a gnome pops out of Dazza's bum when he's knocked out.
fn gnome_from_bum(g: &mut Game) {
    let d = g.life.dazza.pos;
    let (jx, jz) = (g.rng.range(-1.0, 1.0), g.rng.range(-1.0, 1.0));
    let id = g
        .world
        .spawn(ItemKind::Gnome, d.x + jx, d.z + jz, true, &mut g.rng);
    let (vx, vy, vz) = (
        g.rng.range(-2.0, 2.0),
        g.rng.range(4.0, 6.0),
        g.rng.range(-2.0, 2.0),
    );
    if let Some(it) = g.world.items.get_mut(&id) {
        it.pos = V3::new(d.x + jx, 1.0, d.z + jz);
        it.vel = V3::new(vx, vy, vz);
    }
    g.say("A gnome popped out of Dazza's bum!");
}

// ------------------------------------------------------------------ R: meat, chest, smoko

fn meat_spot(p: &Player, bbq_on: bool) -> Option<ItemKind> {
    if !bbq_on || p.mover.y > 0.5 {
        return None;
    }
    let (mx, mz, w, d, _) = MEAT_TABLE;
    let (dx, dz) = (p.mover.x - mx, p.mover.z - mz);
    let ex = (dx.abs() - w / 2.0).max(0.0);
    let ez = (dz.abs() - d / 2.0).max(0.0);
    if ex.hypot(ez) > 1.3 {
        return None;
    }
    Some(if dx < 0.0 {
        ItemKind::Steak
    } else {
        ItemKind::Fish
    })
}

fn chest_dist(p: &Player, chest_spot: usize) -> f32 {
    let (cx, cz, _) = CHEST_SPOTS[chest_spot % CHEST_SPOTS.len()];
    (p.mover.x - cx).hypot(p.mover.z - cz)
}

fn in_smoko_zone(g: &Game, p: &Player) -> bool {
    g.options.smoko_on && smoko::in_zone(p.mover.x, p.mover.y, p.mover.z, people_count(g))
}

/// What R would do here, as a line of text (empty if nothing).
pub fn prompt(g: &Game, p: &Player, f: &yard::Features, chest_spot: usize) -> String {
    if g.life.seated.is_some() {
        return "Smoko! R or Space to get up (the drinks keep coming)".into();
    }
    if g.life.carry.is_some() {
        return "Dragging them! Tap F to put them down, hold F to chuck them".into();
    }
    if in_smoko_zone(g, p) {
        return if g.options.naughty {
            "The Naughty Corner. No safe break here!".into()
        } else {
            "R: sit down for smoko (nobody can touch you, but you have to drink)".into()
        };
    }
    if let Some(k) = meat_spot(p, f.bbq) {
        return format!(
            "R: grab {} (Dazza won't like it)",
            item_name(k).to_lowercase()
        );
    }
    if f.chest && g.options.adult && chest_dist(p, chest_spot) < chest::REACH {
        return if g.life.chest.stock == 0 {
            "The chest is empty. It restocks every 12 seconds.".into()
        } else {
            format!("R: open the chest ({} left)", g.life.chest.stock)
        };
    }
    if g.life.carry.is_none() && grab_target(g, p).is_some() {
        return "F: grab them and drag them off".into();
    }
    String::new()
}

/// R pressed with nothing else to do. Returns true if it did something.
pub fn interact(g: &mut Game, p: &mut Player, f: &yard::Features, chest_spot: usize) -> bool {
    if g.life.seated.is_some() {
        stand_up(g, p, None);
        return true;
    }
    if !g.rules.in_play() || g.me.body.stun > 0.0 || g.me.body.fall_t > 0.0 {
        return false;
    }
    if in_smoko_zone(g, p) {
        if g.options.naughty {
            g.popup("Naughty Corner! No safe break here", false);
        } else {
            sit_down(g, p);
        }
        return true;
    }
    if let Some(kind) = meat_spot(p, f.bbq) {
        if g.slots.len() >= 2 || g.now - g.life.meat_at < 0.8 {
            return true;
        }
        g.life.meat_at = g.now;
        let id = g.world.spawn(kind, p.mover.x, p.mover.z, false, &mut g.rng);
        if let Some(it) = g.world.items.get_mut(&id) {
            it.uses = Some(SLAP_USES);
        }
        g.world.give(id, PLAYER_ID);
        g.slots.add(id);
        let adult = g.options.adult;
        if let Some(DazzaEvent::Say(t)) = g.life.dazza.meat_taken(PLAYER_ID, adult, &mut g.rng) {
            g.life.dazza_says(t);
        }
        return true;
    }
    if f.chest && g.options.adult && chest_dist(p, chest_spot) < chest::REACH {
        let held = g.slots.len();
        match g
            .life
            .chest
            .take(chest_dist(p, chest_spot), held, &mut g.rng)
        {
            Ok(variant) => {
                let id = g
                    .world
                    .spawn(ItemKind::Dildo, p.mover.x, p.mover.z, false, &mut g.rng);
                if let Some(it) = g.world.items.get_mut(&id) {
                    it.variant = Some(variant);
                    it.uses = Some(SLAP_USES);
                }
                g.world.give(id, PLAYER_ID);
                g.slots.add(id);
                g.popup(format!("MYSTERY CHEST! {}!", variant.def().label), true);
            }
            Err(chest::Refused::Empty) => g.popup("The chest is empty", false),
            Err(chest::Refused::HandsFull) => g.popup("Hands full!", false),
            Err(chest::Refused::TooFar) => {}
        }
        return true;
    }
    false
}

fn taken_seats(g: &Game) -> Vec<bool> {
    let n = smoko::chair_count(people_count(g));
    let mut t = vec![false; n];
    for d in &g.dummies {
        if let Some(s) = d.seat
            && s < n
        {
            t[s] = true;
        }
    }
    t
}

fn sit_down(g: &mut Game, p: &mut Player) {
    let n = smoko::chair_count(people_count(g));
    let taken = taken_seats(g);
    let Some(seat) = smoko::nearest_free_seat(p.mover.x, p.mover.z, n, &taken) else {
        g.popup("No free chairs. Wait for someone to finish.", false);
        return;
    };
    g.life.seated = Some(Seated::new(seat));
    g.wind.cancel();
    g.me.drunk.cancel_drink();
    let (x, z) = smoko::seat_pos(seat, n);
    p.mover.x = x;
    p.mover.z = z;
    p.mover.vx = 0.0;
    p.mover.vz = 0.0;
    p.yaw = smoko::seat_facing(seat, n) + std::f32::consts::PI;
    p.pitch = -0.12;
}

fn stand_up(g: &mut Game, p: &mut Player, msg: Option<&str>) {
    if g.life.seated.take().is_some() {
        let (x, z) = smoko::stand_pos(p.mover.x, p.mover.z);
        p.mover.x = x;
        p.mover.z = z;
        if let Some(m) = msg {
            g.popup(m, false);
        }
    }
}

// ------------------------------------------------------------------ grab, drag, throw

fn party_of_me(g: &Game, p: &Player) -> Party {
    Party {
        at_smoko: g.life.seated.is_some(),
        in_naughty_corner: false,
        stun: g.me.body.stun,
        down_t: g.me.body.down_t,
        fall_t: g.me.body.fall_t,
        in_pool: p.mover.in_pool,
        height: p.mover.y,
        carried: false,
        carrying: g.life.carry.is_some(),
    }
}

fn party_of(g: &Game, i: usize) -> Party {
    let d = &g.dummies[i];
    Party {
        at_smoko: d.seat.is_some() && d.naughty_t <= 0.0,
        in_naughty_corner: d.naughty_t > 0.0,
        stun: 0.0,
        down_t: d.body.down_t,
        fall_t: d.body.fall_t,
        in_pool: d.mover.in_pool,
        height: d.mover.y,
        carried: d.dragged.is_some(),
        carrying: false,
    }
}

/// The nearest dummy lying down within reach who you may grab.
fn grab_target(g: &Game, p: &Player) -> Option<usize> {
    let me = party_of_me(g, p);
    let mut best: Option<(usize, f32)> = None;
    for (i, d) in g.dummies.iter().enumerate() {
        if g.life.grab_immune.get(i).copied().unwrap_or(0.0) > g.now {
            continue;
        }
        if !carry::can_grab(g.rules.phase == Phase::Play, &me, &party_of(g, i)) {
            continue;
        }
        let dist = (d.mover.x - p.mover.x).hypot(d.mover.z - p.mover.z);
        if dist < carry::REACH && best.is_none_or(|(_, b)| dist < b) {
            best = Some((i, dist));
        }
    }
    best.map(|b| b.0)
}

fn release_carry(g: &mut Game, p: &Player, how: Drop, held: f32) {
    let Some(c) = g.life.carry.take() else {
        return;
    };
    let i = c.victim;
    let now = g.now;
    if g.life.grab_immune.len() <= i {
        g.life.grab_immune.resize(i + 1, 0.0);
    }
    g.life.grab_immune[i] = now + carry::IMMUNE;
    g.life.f_down = None;
    g.dummies[i].dragged = None;
    g.dummies[i].body.stun = g.dummies[i].body.stun.max(how.victim_stun());
    let name = dummy_name(i);
    match how {
        Drop::Chucked => {
            let f = facing(p);
            let (vx, vy, vz) = carry::throw_velocity(f.0, f.1);
            let d = &mut g.dummies[i];
            d.mover.vx = vx;
            d.mover.vy = vy;
            d.mover.vz = vz;
            d.mover.grounded = false;
            d.thrown_at = Some(now);
            d.thrown_hit = false;
            d.last_hit = Some((PLAYER_ID, now));
            d.anim.start_slap(SlapKind::SentFlying, carry::THROWN_STUN);
            g.say(format!("You chucked {name}!"));
        }
        Drop::PutDown => {
            if g.options.naughty
                && g.options.smoko_on
                && smoko::in_zone(
                    g.dummies[i].mover.x,
                    g.dummies[i].mover.y,
                    g.dummies[i].mover.z,
                    people_count(g),
                )
            {
                send_to_corner(g, i);
            }
        }
        Drop::Wriggled => g.say(format!("{name} wriggled free!")),
        Drop::Dropped | Drop::Forced => {}
    }
    let _ = held;
}

/// The Naughty Corner: five seconds in a smoko chair, and the sender gets 100.
fn send_to_corner(g: &mut Game, i: usize) {
    if g.dummies[i].naughty_t > 0.0 || g.dummies[i].seat.is_some() {
        return;
    }
    let n = smoko::chair_count(people_count(g));
    let taken = taken_seats(g);
    let (x, z) = (g.dummies[i].mover.x, g.dummies[i].mover.z);
    let Some(seat) = smoko::nearest_free_seat(x, z, n, &taken) else {
        return;
    };
    let d = &mut g.dummies[i];
    d.seat = Some(seat);
    d.naughty_t = smoko::NAUGHTY_SIT;
    d.body.down_t = 0.0;
    d.body.fall_t = 0.0;
    let pts = g.board.award(&g.rules, PLAYER_ID, smoko::NAUGHTY_PTS);
    g.popup(format!("NAUGHTY CORNER! +{pts}"), true);
    g.say(format!("{} is in the Naughty Corner", dummy_name(i)));
}

fn carry_step(g: &mut Game, p: &Player, wanted: &mut Wanted) {
    let now = g.now;
    let dt = bbq_core::sim::TICK_DT;
    let pressed = std::mem::take(&mut wanted.grab_pressed);
    let released = std::mem::take(&mut wanted.grab_released);

    if g.life.carry.is_none() {
        if pressed
            && g.me.body.stun <= 0.0
            && let Some(i) = grab_target(g, p)
        {
            g.life.carry = Some(Carry {
                victim: i,
                drag: Drag::new(),
            });
            g.wind.cancel();
            g.say(format!("You grabbed {} by the ankles", dummy_name(i)));
        }
        return;
    }

    if pressed {
        g.life.f_down = Some(now);
    }
    let grabber_ok = g.me.body.stun <= 0.0
        && !g.me.body.is_down()
        && !p.mover.in_pool
        && g.rules.phase == Phase::Play;
    let ended = g
        .life
        .carry
        .as_mut()
        .and_then(|c| c.drag.tick(dt, true, grabber_ok));
    if let Some(how) = ended {
        release_carry(g, p, how, 0.0);
        return;
    }
    if released && let Some(t0) = g.life.f_down {
        let held = now - t0;
        let how = if carry::is_throw(held) {
            Drop::Chucked
        } else {
            Drop::PutDown
        };
        release_carry(g, p, how, held);
        return;
    }
    // pin the victim behind us by the ankles
    let Some(c) = g.life.carry.as_ref() else {
        return;
    };
    let i = c.victim;
    let f = facing(p);
    let d = &mut g.dummies[i];
    d.mover.x = p.mover.x - f.0 * carry::DRAG_DISTANCE;
    d.mover.z = p.mover.z - f.1 * carry::DRAG_DISTANCE;
    d.mover.y = p.mover.y;
    d.mover.vx = p.mover.vx;
    d.mover.vz = p.mover.vz;
    d.mover.vy = 0.0;
    d.mover.grounded = p.mover.grounded;
    d.body.fall_t = 0.0;
    d.body.down_t = 0.0;
    d.body.stun = d.body.stun.max(0.3);
    d.dragged = Some(DraggerInfo {
        face: p.yaw,
        speed: p.mover.speed(),
        walk: p.walk,
        grounded: p.mover.grounded,
    });
}

/// Someone who was thrown: lands in the Naughty Corner, or flattens whoever they hit.
fn thrown_step(g: &mut Game) {
    let now = g.now;
    let n = g.dummies.len();
    for i in 0..n {
        let Some(t0) = g.dummies[i].thrown_at else {
            continue;
        };
        let since = now - t0;
        if since > carry::CANNONBALL_WINDOW.1 + 0.3 {
            g.dummies[i].thrown_at = None;
            continue;
        }
        let (x, y, z, grounded) = {
            let m = &g.dummies[i].mover;
            (m.x, m.y, m.z, m.grounded)
        };
        if g.options.naughty
            && g.options.smoko_on
            && smoko::naughty_lands(since, grounded, smoko::in_zone(x, y, z, people_count(g)))
        {
            g.dummies[i].thrown_at = None;
            send_to_corner(g, i);
            continue;
        }
        if g.dummies[i].thrown_hit {
            continue;
        }
        for j in 0..n {
            if j == i || g.dummies[j].body.is_down() || g.dummies[j].seat.is_some() {
                continue;
            }
            let o = &g.dummies[j].mover;
            let dist = (o.x - x).hypot(o.z - z);
            if carry::cannonball_hits(dist, (o.y - y).abs(), since) {
                g.dummies[i].thrown_hit = true;
                let vid = g.dummies[j].id;
                let dir = melee::direction(
                    V3::new(x, 0.0, z),
                    V3::new(g.dummies[j].mover.x, 0.0, g.dummies[j].mover.z),
                );
                let d = &mut g.dummies[j];
                d.last_hit = Some((PLAYER_ID, now));
                d.body.apply_hit(0.0, Some(carry::CANNONBALL_DOWN));
                d.anim.start_slap(SlapKind::Timber, carry::CANNONBALL_DOWN);
                hitting::knock_mover(&mut d.mover, dir, 3.0, 2.0);
                let pts = g.board.cannonball(&g.rules, PLAYER_ID, vid, false);
                g.popup(format!("HUMAN CANNONBALL! +{pts}"), true);
                break;
            }
        }
    }
}

// ------------------------------------------------------------------ emotes

fn emote_step(g: &mut Game, wanted: &mut Wanted) {
    let Some(e) = wanted.emote.take() else {
        return;
    };
    let down = g.me.body.is_down();
    if !emotes::can_emote(g.now, g.life.emote_at, g.me.body.stun > 0.0, down) {
        return;
    }
    g.life.emote_at = g.now;
    let line = emotes::pick_line(e, g.options.adult, &mut g.rng);
    g.popup(line, false);
}

// ------------------------------------------------------------------ the per-step update

/// Run everything above once per fixed step. Called at the end of `step_game`.
pub fn step(g: &mut Game, p: &mut Player, wanted: &mut Wanted, yard: &yard::Yard) {
    let dt = bbq_core::sim::TICK_DT;
    let now = g.now;
    if g.life.grab_immune.len() < g.dummies.len() {
        g.life.grab_immune.resize(g.dummies.len(), 0.0);
    }
    g.life.me_swing = (g.life.me_swing - dt).max(0.0);
    g.life.chest.tick(dt);

    // you, at smoko
    if g.life.seated.is_some() {
        let can_act = g.rules.in_play();
        let stunned = g.me.body.stun > 0.0;
        let space = std::mem::take(&mut wanted.jump_pressed);
        let t = g.life.seated.as_mut().map(|s| s.tick(dt, can_act, stunned));
        if let Some(t) = t {
            g.me.drunk.add(t.drunk);
            match t.over {
                Some(Over::TimeUp) => stand_up(g, p, Some("Smoko's over! Back to it.")),
                Some(Over::Interrupted) => stand_up(g, p, None),
                _ if space => stand_up(g, p, None),
                _ => {
                    let n = smoko::chair_count(people_count(g));
                    let seat = g.life.seated.map(|s| s.seat).unwrap_or(0);
                    let (x, z) = smoko::seat_pos(seat.min(n - 1), n);
                    p.mover.x = x;
                    p.mover.z = z;
                    p.mover.vx = 0.0;
                    p.mover.vz = 0.0;
                }
            }
        }
    }
    wanted.jump_pressed = false;

    carry_step(g, p, wanted);
    thrown_step(g);
    emote_step(g, wanted);

    // dummies in the Naughty Corner sit still, then get up
    let n = smoko::chair_count(people_count(g));
    for d in &mut g.dummies {
        if let Some(seat) = d.seat
            && d.sat_by_choice
        {
            // a bot having a smoko: pinned in its chair until its own brain stands it up
            let (x, z) = smoko::seat_pos(seat.min(n - 1), n);
            d.mover.x = x;
            d.mover.z = z;
            d.mover.vx = 0.0;
            d.mover.vz = 0.0;
        } else if let Some(seat) = d.seat {
            d.naughty_t = (d.naughty_t - dt).max(0.0);
            let (x, z) = smoko::seat_pos(seat.min(n - 1), n);
            d.mover.x = x;
            d.mover.z = z;
            d.mover.vx = 0.0;
            d.mover.vz = 0.0;
            if d.naughty_t <= 0.0 {
                let (sx, sz) = smoko::stand_pos(x, z);
                d.mover.x = sx;
                d.mover.z = sz;
                d.seat = None;
            }
        }
        d.smell_t = (d.smell_t - dt).max(0.0);
        d.smelly = d.smell_t > 0.0;
    }

    // Dazza
    let mut people = vec![Person {
        id: PLAYER_ID,
        pos: me_pos(p),
        down_t: g.me.body.down_t.max(g.me.body.fall_t),
        at_smoko: g.life.seated.is_some(),
    }];
    for d in &g.dummies {
        people.push(Person {
            id: d.id,
            pos: V3::new(d.mover.x, d.mover.y, d.mover.z),
            down_t: d.body.down_t.max(d.body.fall_t),
            at_smoko: d.seat.is_some(),
        });
    }
    let countdown = g.rules.phase == Phase::Countdown;
    let adult = g.options.adult;
    let events = if yard.features.bbq {
        g.life
            .dazza
            .tick(dt, &people, &yard.colliders, countdown, adult, &mut g.rng)
    } else {
        Vec::new()
    };
    for ev in events {
        match ev {
            DazzaEvent::Say(t) => g.life.dazza_says(t),
            DazzaEvent::Swing => g.life.swing_seq += 1,
            DazzaEvent::SpatulaHit {
                victim,
                dir,
                berserk,
            } => spatula(g, p, victim, dir, berserk),
        }
    }
    let _ = now;
}

fn spatula(g: &mut Game, p: &mut Player, victim: PlayerId, dir: V3, berserk: bool) {
    let secs = if berserk {
        dazza_brain::SPATULA_STUN_BERSERK
    } else {
        dazza_brain::SPATULA_STUN
    };
    let knock = dazza_brain::SPATULA_KNOCK;
    if victim == PLAYER_ID {
        hitting::knock_mover(&mut p.mover, dir, knock, knock * 0.38);
        g.me.body.apply_hit(secs, None);
        g.me.drunk.cancel_drink();
        g.wind.cancel();
        p.shake = p.shake.max(0.3);
        g.popup("SPATULA'D!", true);
        g.say("Dazza spatula'd you");
    } else if let Some(i) = g.dummies.iter().position(|d| d.id == victim) {
        let d = &mut g.dummies[i];
        hitting::knock_mover(&mut d.mover, dir, knock, knock * 0.38);
        d.body.apply_hit(secs, None);
        d.anim.dizzy = secs;
        d.anim.tumble(dir, knock, 1.0);
        let name = dummy_name(i);
        g.say(format!("Dazza spatula'd {name}"));
    }
}

/// The pool and the trampoline: someone you hit lands there within 3.5 s.
pub fn env_bonus(g: &mut Game, victim: usize, place: Place) {
    let now = g.now;
    let Some((who, t)) = g.dummies[victim].last_hit else {
        return;
    };
    let vid = g.dummies[victim].id;
    if !emotes::pays(now - t, who, vid, false) {
        return;
    }
    g.dummies[victim].last_hit = None;
    let pts = g.board.award(&g.rules, who, scoring::POOL_OR_TRAMP);
    g.popup(format!("{} +{pts}", place.shout()), true);
    g.say(format!(
        "{} sent {} {}",
        if who == PLAYER_ID { "You" } else { "Someone" },
        dummy_name(victim),
        match place {
            Place::Pool => "into the pool",
            Place::Tramp => "onto the tramp",
        }
    ));
}
