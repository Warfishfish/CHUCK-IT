//! Online play, step 9.4: the host's picture of the yard, and a guest drawing from it.
//!
//! The host turns its game into a `WorldSnap` (`build_world`) about 15 times a second. A guest's
//! game is a mirror (`Game::mirror`): it does not run the bots, the items or the rules, it
//! copies what the snapshot says (`apply_world`) and in between moves the things that were
//! moving so they do not jump at 15 Hz (`mirror_tick`). Your own blob is still moved by you.
//!
//! People are named by net ids on the wire (the host is 1, guests 2 and up, bots 100 and up) and
//! by local ids in each game (you are `PLAYER_ID`, others online are `REMOTE_ID_BASE` plus their
//! net id, bots 100 and up); `local_of_net` and `net_of_local` turn one into the other.

use std::collections::BTreeMap;

use bbq_core::GameMode;
use bbq_core::dazza::DazzaState;
use bbq_core::flight::{ITEM_GRAV, Item, ItemState};
use bbq_core::items::{DildoVariant, ItemKind};
use bbq_core::net_world::*;
use bbq_core::scoring::{Phase, PlayerStats, Rules};
use bbq_core::teams::Team;
use bbq_core::vec::V3;
use bbq_core::yard::{Features, Yard};

use crate::fx::{FxEvent, FxKind};
use crate::game::{Game, REMOTE_ID_BASE};
use crate::player::{PLAYER_ID, Player, Wanted};
use bbq_core::hands::{self, Press, Release, Situation};
use bbq_core::items::Melee;
use bbq_core::melee::{self, SlapEffect};
use bbq_core::net_act::{GuestAct, HitMsg, MAX_THROW_START};
use bbq_core::pose::SlapKind;

/// How often the host sends the world.
pub const WORLD_HZ: f32 = 15.0;

/// A person's id on this computer, from their number on the network. `my_net` is our own.
pub fn local_of_net(net: u32, my_net: u32) -> u32 {
    if net == my_net {
        PLAYER_ID
    } else if net >= 100 {
        net
    } else {
        REMOTE_ID_BASE + net
    }
}

/// The other way round.
pub fn net_of_local(local: u32, my_net: u32) -> u32 {
    if local == PLAYER_ID {
        my_net
    } else if local >= REMOTE_ID_BASE {
        local - REMOTE_ID_BASE
    } else {
        local
    }
}

fn team_code(t: Team) -> u8 {
    match t {
        Team::Red => 0,
        Team::Blue => 1,
        Team::Green => 2,
        Team::Yellow => 3,
        Team::Wildcard => 4,
    }
}

fn team_of_code(c: u8) -> Option<Team> {
    Some(match c {
        0 => Team::Red,
        1 => Team::Blue,
        2 => Team::Green,
        3 => Team::Yellow,
        4 => Team::Wildcard,
        _ => return None,
    })
}

pub fn kind_code(k: ItemKind) -> u8 {
    ItemKind::ALL.iter().position(|x| *x == k).unwrap_or(0) as u8
}

fn mode_code(m: GameMode) -> u8 {
    match m {
        GameMode::FreeForAll => 0,
        GameMode::Teams => 1,
        GameMode::Heist => 2,
    }
}

fn mode_of_code(c: u8) -> GameMode {
    match c {
        1 => GameMode::Teams,
        2 => GameMode::Heist,
        _ => GameMode::FreeForAll,
    }
}

/// The host's picture of the yard. `feed_after` is the game time of the last feed line already
/// sent; `fx` are the effects since the last snapshot.
pub fn build_world(g: &Game, yard: &Yard, seq: u32, feed_after: f32, fx: &[FxEvent]) -> WorldSnap {
    let me = PLAYER_ID; // the host is the one building this
    let net = |id: u32| net_of_local(id, me);
    let mut features = 0u8;
    for (on, bit) in [
        (yard.features.bar, FEAT_BAR),
        (yard.features.bbq, FEAT_BBQ),
        (yard.features.chest, FEAT_CHEST),
        (yard.features.smoko, FEAT_SMOKO),
        (g.options.adult, FEAT_ADULT),
        (g.rules.friendly_fire, FEAT_FRIENDLY_FIRE),
        (g.round.timed, FEAT_TIMED),
    ] {
        if on {
            features |= bit;
        }
    }
    let round = RoundSnap {
        phase: g.rules.phase.index(),
        mode: mode_code(g.rules.mode),
        features,
        time_left: g.round.time_left,
        round_no: g.round.mtch.no.min(255) as u8,
        match_len: g.round.mtch.len.min(255) as u8,
        chest_spot: yard.chest_spot.min(255) as u8,
        chest_stock: g.life.chest.stock.min(255) as u8,
        smoko_at: g.life.smoko_at,
        heist_teams: yard.heist.unwrap_or(0).min(255) as u8,
        banner: g.round.banner.clone(),
    };
    let items = g
        .world
        .items
        .values()
        .take(MAX_ITEMS)
        .map(|it| ItemSnap {
            id: it.id,
            kind: kind_code(it.kind),
            variant: it.variant.map_or(255, |v| DildoVariant::ALL.iter().position(|x| *x == v).unwrap_or(0) as u8),
            state: match it.state {
                ItemState::Ground => STATE_GROUND,
                ItemState::Held => STATE_HELD,
                ItemState::Flying => STATE_FLYING,
            },
            holder: it.holder.map_or(0, net),
            pos: (it.pos.x, it.pos.y, it.pos.z),
            vel: (it.vel.x, it.vel.y, it.vel.z),
            ground_y: it.ground_y,
            team: it.team.unwrap_or(255),
            uses: it.uses.map_or(255, |u| u.min(254) as u8),
        })
        .collect();
    let bots = g
        .dummies
        .iter()
        .filter(|d| d.remote.is_none())
        .take(MAX_BOTS)
        .map(|d| {
            let mut flags = 0u16;
            for (on, bit) in [
                (d.mover.grounded, BOT_GROUNDED),
                (d.body.stun > 0.0, BOT_STUNNED),
                (d.body.is_down(), BOT_DOWN),
                (d.fallen, BOT_FALLEN),
                (d.bot.drunk.is_drinking(), BOT_DRINKING),
                (d.seat.is_some(), BOT_SEATED),
                (d.naughty_t > 0.0, BOT_NAUGHTY),
                (d.bot.winding, BOT_WINDING),
                (d.crown, BOT_CROWN),
                (d.smelly, BOT_SMELLY),
            ] {
                if on {
                    flags |= bit;
                }
            }
            BotSnap {
                id: d.id,
                pos: (d.mover.x, d.mover.y, d.mover.z),
                vel: (d.mover.vx, d.mover.vz),
                face: d.face,
                flags,
                charge: d.bot.wind_progress,
                seat: d.seat.map_or(255, |s| s.min(254) as u8),
                team: d.team.map_or(255, team_code),
                drunk: d.drunk.clamp(0.0, 100.0).round() as u8,
                belly: (d.belly.clamp(0.0, 2.0) / 2.0 * 255.0).round() as u8,
                selected: d.bot.slots.selected().unwrap_or(0),
                swing: (d.bot.swing_t.clamp(0.0, 2.0) * 100.0) as u8,
            }
        })
        .collect();
    let dz = &g.life.dazza;
    let dazza = DazzaSnap {
        x: dz.pos.x,
        z: dz.pos.z,
        face: dz.face,
        state: DazzaState::ALL.iter().position(|s| *s == dz.state).unwrap_or(0) as u8,
        say_seq: g.life.say_seq,
        swing_seq: g.life.swing_seq,
        flip_seq: g.life.flip_seq,
        say: g.life.say_text.clone(),
    };
    let board = g
        .board
        .iter()
        .take(MAX_BOARD)
        .map(|(id, s)| BoardRow {
            id: net(id),
            score: s.score,
            hits: s.hits.min(65535) as u16,
            taken: s.taken.min(65535) as u16,
            catches: s.catches.min(65535) as u16,
            throws: s.throws.min(65535) as u16,
            streak: s.streak.min(255) as u8,
            banked: s.banked.min(65535) as u16,
        })
        .collect();
    let teams = g.teams.iter().take(MAX_BOARD).map(|(id, t)| (net(id), team_code(t))).collect();
    // lines said as "You ..." by the host mean the host: whoever reads them is somebody else
    let feed = g
        .feed
        .iter()
        .filter(|(_, t)| *t > feed_after)
        .map(|(s, _)| s.clone())
        .rev()
        .take(MAX_FEED)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();
    let fx = fx.iter().take(MAX_FX).map(|e| FxSnap { kind: e.kind.index(), at: (e.at.x, e.at.y, e.at.z) }).collect();
    WorldSnap { seq, now: g.now, round, items, bots, dazza, board, teams, feed, fx }
}

/// Make this game look like the host's. `my_net` is our own number; `host_name` replaces
/// "You" in the host's feed lines.
pub fn apply_world(g: &mut Game, yard: &mut Yard, snap: &WorldSnap, my_net: u32, host_name: &str) {
    let local = |net: u32| local_of_net(net, my_net);
    g.mirror = true;

    // ---- the round ----
    let r = &snap.round;
    let was_over = g.rules.phase == Phase::Results;
    g.rules = Rules {
        mode: mode_of_code(r.mode),
        friendly_fire: r.features & FEAT_FRIENDLY_FIRE != 0,
        phase: Phase::from_index(r.phase),
    };
    g.round.timed = r.features & FEAT_TIMED != 0;
    g.round.time_left = r.time_left;
    g.round.mtch.no = r.round_no as u32;
    g.round.mtch.len = (r.match_len as u32).max(1);
    g.round.setup.mode = g.rules.mode;
    g.round.banner = r.banner.clone();
    g.options.adult = r.features & FEAT_ADULT != 0;
    g.options.smoko_on = r.features & FEAT_SMOKO != 0;
    let features = Features {
        bar: r.features & FEAT_BAR != 0,
        bbq: r.features & FEAT_BBQ != 0,
        chest: r.features & FEAT_CHEST != 0,
        smoko: r.features & FEAT_SMOKO != 0,
    };
    let spot = r.chest_spot as usize;
    let heist = (r.heist_teams > 0).then_some(r.heist_teams as usize);
    if yard.features != features || yard.chest_spot != spot || yard.heist != heist {
        *yard = match heist {
            Some(n) if g.rules.mode == GameMode::Heist => Yard::heist(features, spot, n),
            _ => Yard::new(features, spot),
        };
    }
    // the whistle went: work out the results here, from the host's scores, so the results card
    // opens for guests too
    if !was_over && g.rules.phase == Phase::Results && g.round.timed {
        // the board is only copied below, so do this after it is
        g.round.results = None;
    }
    g.life.chest.stock = r.chest_stock as u32;
    g.life.chest.spot = spot;
    g.life.smoko_at = r.smoko_at;

    // ---- the bots (the people online keep their puppets) ----
    crate::round::sync_bot_count(g, snap.bots.len());
    let heard_at = g.now;
    for b in &snap.bots {
        let id = local(b.id);
        let Some(d) = g.dummies.iter_mut().find(|d| d.id == id && d.remote.is_none()) else { continue };
        let pos = V3::new(b.pos.0, b.pos.1, b.pos.2);
        if d.net_target.is_none() {
            d.mover.x = pos.x;
            d.mover.z = pos.z;
        }
        d.net_target = Some((pos, b.vel, heard_at));
        d.mover.y = pos.y;
        d.mover.vx = b.vel.0;
        d.mover.vz = b.vel.1;
        d.mover.grounded = b.flags & BOT_GROUNDED != 0;
        d.face = b.face;
        d.drunk = b.drunk as f32;
        d.belly = b.belly as f32 / 255.0 * 2.0;
        d.fallen = b.flags & BOT_FALLEN != 0;
        d.crown = b.flags & BOT_CROWN != 0;
        d.smelly = b.flags & BOT_SMELLY != 0;
        d.seat = (b.seat != 255).then_some(b.seat as usize);
        d.naughty_t = if b.flags & BOT_NAUGHTY != 0 { 1.0 } else { 0.0 };
        d.team = team_of_code(b.team);
        d.bot.winding = b.flags & BOT_WINDING != 0;
        d.bot.wind_progress = b.charge;
        // stun and knockdown only matter for how it is drawn: the timers are the host's business
        d.body.stun = if b.flags & BOT_STUNNED != 0 { d.body.stun.max(0.5) } else { 0.0 };
        if b.flags & BOT_DOWN != 0 {
            if !d.body.is_down() {
                d.body.down_t = 1.0;
            }
        } else {
            d.body.down_t = 0.0;
        }
        if b.swing > 0 && d.bot.swing_t <= 0.0 {
            d.anim.start_swing();
        }
        d.bot.swing_t = b.swing as f32 / 100.0;
    }

    // ---- the items (and who holds what) ----
    let mut items: BTreeMap<u32, Item> = BTreeMap::new();
    for s in &snap.items {
        let Some(&kind) = ItemKind::ALL.get(s.kind as usize) else { continue };
        let mut it = Item::new(s.id, kind, V3::new(s.pos.0, s.pos.1, s.pos.2));
        it.vel = V3::new(s.vel.0, s.vel.1, s.vel.2);
        it.state = match s.state {
            STATE_HELD => ItemState::Held,
            STATE_FLYING => ItemState::Flying,
            _ => ItemState::Ground,
        };
        it.live = it.state == ItemState::Flying;
        it.holder = (s.holder != 0).then(|| local(s.holder));
        it.ground_y = s.ground_y;
        it.variant = DildoVariant::ALL.get(s.variant as usize).copied();
        it.team = (s.team != 255).then_some(s.team);
        it.uses = (s.uses != 255).then_some(s.uses as u32);
        items.insert(s.id, it);
    }
    g.world.items = items;
    // our own hands: whatever the host says we hold (apart from what we have just thrown away)
    let now = g.now;
    let gone_ids: Vec<u32> = g.pending_gone.iter().filter(|(_, until)| *until > now).map(|(id, _)| *id).collect();
    let mine: Vec<u32> = g.world.items.values().filter(|it| it.holder == Some(PLAYER_ID) && !gone_ids.contains(&it.id)).map(|it| it.id).collect();
    let old_selected = g.slots.selected();
    let mut slots = bbq_core::hands::Slots::default();
    for id in g.slots.ids() {
        if mine.contains(id) {
            slots.add(*id);
        }
    }
    let mut added = false;
    for id in &mine {
        if !slots.ids().contains(id) {
            slots.add(*id);
            added = true;
        }
    }
    if !added
        && let Some(n) = old_selected.and_then(|sel| slots.ids().iter().position(|x| *x == sel))
    {
        slots.select_slot(n);
    }
    g.slots = slots;
    for d in g.dummies.iter_mut() {
        let selected = snap.bots.iter().find(|b| local(b.id) == d.id).map_or(0, |b| b.selected);
        let mut slots = bbq_core::hands::Slots::default();
        if selected != 0 && g.world.items.get(&selected).is_some_and(|it| it.holder == Some(d.id)) {
            slots.add(selected);
        }
        for it in g.world.items.values() {
            if it.holder == Some(d.id) && it.id != selected {
                slots.add(it.id);
            }
        }
        d.bot.slots = slots;
    }

    // ---- Dazza ----
    let dz = &mut g.life.dazza;
    dz.pos = V3::new(snap.dazza.x, 0.0, snap.dazza.z);
    dz.face = snap.dazza.face;
    dz.state = DazzaState::ALL.get(snap.dazza.state as usize).copied().unwrap_or(DazzaState::Cook);
    if snap.dazza.say_seq != g.life.say_seq && !snap.dazza.say.is_empty() {
        g.life.say_text = snap.dazza.say.clone();
        g.life.say_seq = snap.dazza.say_seq;
    }
    if snap.dazza.swing_seq != g.life.swing_seq {
        g.life.swing_seq = snap.dazza.swing_seq;
    }
    g.life.flip_seq = snap.dazza.flip_seq;

    // ---- scores and teams ----
    g.board.clear();
    for row in &snap.board {
        g.board.set(
            local(row.id),
            PlayerStats {
                score: row.score,
                hits: row.hits as u32,
                taken: row.taken as u32,
                catches: row.catches as u32,
                throws: row.throws as u32,
                streak: row.streak as u32,
                banked: row.banked as u32,
            },
        );
    }
    g.board.ensure(PLAYER_ID);
    g.teams.clear();
    for (id, t) in &snap.teams {
        if let Some(t) = team_of_code(*t) {
            g.teams.set(local(*id), t);
        }
    }
    if !was_over && g.rules.phase == Phase::Results && g.round.timed {
        crate::round::end_round(g);
    }
    if g.rules.phase != Phase::Results {
        g.round.results = None;
    }

    // ---- what happened ----
    for line in &snap.feed {
        let line = rename_you(line, host_name);
        g.say(line);
    }
    for f in &snap.fx {
        if let Some(&kind) = FxKind::ALL.get(f.kind as usize) {
            g.fx(kind, V3::new(f.at.0, f.at.1, f.at.2));
        }
    }
}

/// The host's feed says "You hit Kev"; everybody else reads that as the host's name.
fn rename_you(line: &str, host: &str) -> String {
    let mut out = String::with_capacity(line.len() + host.len());
    for (i, word) in line.split(' ').enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let bare = word.trim_matches(|c: char| !c.is_alphanumeric());
        if bare == "You" {
            out.push_str(&word.replacen("You", host, 1));
        } else if bare == "you" {
            out.push_str(&word.replacen("you", host, 1));
        } else {
            out.push_str(word);
        }
    }
    out
}

/// One tick of a mirrored game (and the hands of a guest, see `guest_hands`): move things that were moving between the host's pictures, and
/// forget button presses (the guest's actions go to the host in the next step).
pub fn mirror_tick(g: &mut Game, p: &mut Player, wanted: &mut Wanted, dt: f32) {
    g.now += dt;
    for pop in &mut g.popups {
        pop.t += dt;
    }
    g.popups.retain(|p| p.t < 1.6);
    // our own body still wobbles, stuns and falls over on our own clock
    g.me.body.tick(dt);
    if g.round.panel_in > 0.0 {
        g.round.panel_in -= dt;
    }
    let now = g.now;
    g.pending_gone.retain(|(_, until)| *until > now);
    guest_hands(g, p, wanted, dt);
    *wanted = Wanted { wish: wanted.wish, ..Default::default() };
    // things in the air keep flying until the host says where they really are
    for it in g.world.items.values_mut() {
        if it.state == ItemState::Flying {
            it.pos = it.pos + it.vel * dt;
            it.vel.y -= ITEM_GRAV * it.kind.def().grav * dt;
        }
    }
    // the bots glide towards where the host last said they are (a little ahead, by their speed)
    let now = g.now;
    let k = 1.0 - (-14.0 * dt).exp();
    for d in g.dummies.iter_mut() {
        let Some((pos, vel, at)) = d.net_target else { continue };
        let age = (now - at).clamp(0.0, 0.2);
        let (tx, tz) = (pos.x + vel.0 * age, pos.z + vel.1 * age);
        let far = (tx - d.mover.x).hypot(tz - d.mover.z) > 4.0;
        let k = if far { 1.0 } else { k };
        d.mover.x += (tx - d.mover.x) * k;
        d.mover.z += (tz - d.mover.z) * k;
    }
    // the sky-high messages from the host that nobody has used for a while fade away
    if g.feed.len() > 30 {
        g.feed.remove(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::*;

    #[test]
    fn ids_go_to_the_network_and_back() {
        // on the host (net id 1): you are 1, a guest numbered 3 is 203, bots are as they are
        assert_eq!(net_of_local(PLAYER_ID, 1), 1);
        assert_eq!(net_of_local(REMOTE_ID_BASE + 3, 1), 3);
        assert_eq!(local_of_net(3, 1), REMOTE_ID_BASE + 3);
        assert_eq!(local_of_net(1, 1), PLAYER_ID);
        // on guest number 3: you are 3 on the wire, the host is a puppet numbered 201
        assert_eq!(net_of_local(PLAYER_ID, 3), 3);
        assert_eq!(local_of_net(3, 3), PLAYER_ID);
        assert_eq!(local_of_net(1, 3), REMOTE_ID_BASE + 1);
        assert_eq!(local_of_net(2, 3), REMOTE_ID_BASE + 2);
        for my in [1, 2, 7] {
            for net in [1u32, 2, 3, 7, 15, 100, 105, 110] {
                assert_eq!(net_of_local(local_of_net(net, my), my), net, "me {my} net {net}");
            }
        }
        assert_eq!(local_of_net(104, 2), 104, "bots keep their numbers");
    }

    #[test]
    fn you_in_the_hosts_feed_becomes_the_hosts_name() {
        assert_eq!(rename_you("You hit Kev (+100)", "Marcus"), "Marcus hit Kev (+100)");
        assert_eq!(rename_you("Kev hit you", "Marcus"), "Kev hit Marcus");
        assert_eq!(rename_you("Kev hit You!", "Marcus"), "Kev hit Marcus!");
        assert_eq!(rename_you("Young Yousuf", "Marcus"), "Young Yousuf");
    }

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(crate::yard_scene::YardRes(bbq_core::yard::Yard::default()));
        app.insert_resource(crate::player::Player {
            mover: bbq_core::movement::Mover::new(8.0, 0.0),
            prev: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            walk: 0.0,
            shake: 0.0,
            fov_base: 85.0,
            fov: 85.0,
            rng: bbq_core::rng::Rng::new(1),
        });
        app.init_resource::<Wanted>();
        app.add_plugins(crate::game::GamePlugin);
        app
    }

    /// A host that has been playing for a while: items about, a bot holding one, scores, a team.
    fn busy_host() -> App {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.world.clear();
            let mut rng = bbq_core::rng::Rng::new(4);
            let a = g.world.spawn(ItemKind::Teddy, 3.0, 4.0, false, &mut rng);
            let b = g.world.spawn(ItemKind::Dildo, -5.0, 2.0, false, &mut rng);
            if let Some(it) = g.world.items.get_mut(&b) {
                it.variant = Some(DildoVariant::LongJohn);
                it.uses = Some(2);
            }
            let c = g.world.spawn(ItemKind::Steak, 1.0, 1.0, false, &mut rng);
            g.world.give(c, 100);
            g.dummies[0].bot.slots.add(c);
            let _ = a;
            g.board.ensure(PLAYER_ID);
            g.board.set(PLAYER_ID, PlayerStats { score: 250, hits: 2, ..Default::default() });
            g.board.set(101, PlayerStats { score: -50, taken: 1, ..Default::default() });
            g.teams.set(PLAYER_ID, Team::Red);
            g.teams.set(100, Team::Blue);
            g.life.chest.stock = 2;
            g.life.dazza.pos = V3::new(-5.0, 0.0, -19.0);
            g.life.dazza_says("Oi!");
            g.say("You hit Kev (+100)");
            g.dummies[1].mover.x = 12.5;
            g.dummies[1].mover.z = -3.5;
            g.dummies[1].crown = true;
            g.dummies[1].bot.drunk.add(40.0);
            g.dummies[1].drunk = 40.0;
        }
        app
    }

    #[test]
    fn a_guest_ends_up_with_the_hosts_yard() {
        let host = busy_host();
        let hg = host.world().resource::<Game>();
        let hy = host.world().resource::<crate::yard_scene::YardRes>().0.clone();
        let snap = build_world(hg, &hy, 1, -1.0, &[FxEvent { kind: FxKind::Hit, at: V3::new(1.0, 2.0, 3.0) }]);
        // the trip over the wire
        let bytes = bbq_core::net::Msg::World(Box::new(snap)).encode();
        let Ok(bbq_core::net::Msg::World(snap)) = bbq_core::net::Msg::decode(&bytes) else { panic!("did not decode") };

        // a guest numbered 3 hears it
        let mut guest = app();
        {
            let mut g = guest.world_mut().resource_mut::<Game>();
            let mut yard = bbq_core::yard::Yard::default();
            apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
        }
        let g = guest.world().resource::<Game>();
        assert!(g.mirror);
        assert_eq!(g.world.items.len(), hg.world.items.len());
        for (id, it) in &hg.world.items {
            let got = &g.world.items[id];
            assert_eq!((got.kind, got.variant, got.uses, got.state), (it.kind, it.variant, it.uses, it.state), "item {id}");
            assert!((got.pos.x - it.pos.x).abs() < 1e-5 && (got.pos.z - it.pos.z).abs() < 1e-5);
        }
        // the steak the host's bot 100 holds is that bot's on the guest too
        let holder = g.world.items.values().find(|i| i.kind == ItemKind::Steak).unwrap().holder;
        assert_eq!(holder, Some(100));
        assert!(g.dummies.iter().find(|d| d.id == 100).unwrap().bot.slots.selected().is_some());
        // bots are where they were, and keep their looks
        assert_eq!(g.dummies.iter().filter(|d| d.remote.is_none()).count(), hg.dummies.len());
        let d1 = g.dummies.iter().find(|d| d.id == 101).unwrap();
        assert_eq!((d1.mover.x, d1.mover.z), (12.5, -3.5));
        assert!(d1.crown);
        assert!((d1.drunk - 40.0).abs() < 1.0);
        // the host's own player is net id 1: on this guest that is the puppet 201's score, and
        // the host's team too
        assert_eq!(g.board.score(REMOTE_ID_BASE + 1), 250);
        assert_eq!(g.board.score(101), -50);
        assert_eq!(g.teams.get(REMOTE_ID_BASE + 1), Some(Team::Red));
        assert_eq!(g.teams.get(100), Some(Team::Blue));
        assert_eq!(g.life.chest.stock, 2);
        assert_eq!(g.life.say_text, "Oi!");
        assert!((g.life.dazza.pos.x + 5.0).abs() < 1e-5);
        // what happened: the feed says the host's name, and the effect is queued
        assert!(g.feed.iter().any(|(l, _)| l == "Marcus hit Kev (+100)"), "{:?}", g.feed);
        assert!(g.fx.iter().any(|e| e.kind == FxKind::Hit));
    }

    #[test]
    fn a_mirrored_game_does_not_run_the_yard_but_keeps_things_moving() {
        let mut guest = app();
        {
            let mut g = guest.world_mut().resource_mut::<Game>();
            g.mirror = true;
            g.world.clear();
            let mut rng = bbq_core::rng::Rng::new(4);
            let id = g.world.spawn(ItemKind::Teddy, 0.0, 0.0, false, &mut rng);
            let it = g.world.items.get_mut(&id).unwrap();
            it.state = ItemState::Flying;
            it.pos = V3::new(0.0, 2.0, 0.0);
            it.vel = V3::new(6.0, 0.0, 0.0);
        }
        let start_time = guest.world().resource::<Game>().round.time_left;
        for _ in 0..30 {
            guest.world_mut().run_schedule(FixedUpdate);
        }
        let g = guest.world().resource::<Game>();
        let it = g.world.items.values().next().unwrap();
        assert!(it.pos.x > 2.0, "it keeps flying between pictures ({})", it.pos.x);
        assert!(it.pos.y < 2.0, "and falls ({})", it.pos.y);
        assert_eq!(g.round.time_left, start_time, "the clock is the host's, not ours");
        assert!(g.now > 0.4, "but time passes for popups and the like");
    }

    /// A host with one person online (net id 2) standing at (8, 0), the round on.
    fn host_with_a_guest() -> App {
        host_with_a_guest_and(0)
    }

    fn host_with_a_guest_and(bots: usize) -> App {
        let mut app = app();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.world.clear();
            g.rules.phase = Phase::Play;
            g.remotes = vec![crate::game::RemoteInfo { net_id: 2, name: "Davo".into(), character: bbq_core::character::Character::Pear }];
            crate::round::sync_bot_count(&mut g, bots);
            let d = g.dummies.iter_mut().find(|d| d.remote.is_some()).unwrap();
            d.mover.x = 8.0;
            d.mover.z = 0.0;
            d.face = std::f32::consts::PI; // looking along -z
        }
        app
    }

    fn give_guest(app: &mut App, kind: ItemKind, variant: Option<DildoVariant>) -> u32 {
        let mut g = app.world_mut().resource_mut::<Game>();
        let mut rng = bbq_core::rng::Rng::new(9);
        let id = g.world.spawn(kind, 8.0, 0.0, false, &mut rng);
        if let Some(it) = g.world.items.get_mut(&id) {
            it.variant = variant;
            it.uses = kind.starting_uses();
        }
        let guest = REMOTE_ID_BASE + 2;
        g.world.give(id, guest);
        g.dummies.iter_mut().find(|d| d.id == guest).unwrap().bot.slots.add(id);
        id
    }

    fn act(app: &mut App, a: GuestAct) {
        let mut player = app.world_mut().remove_resource::<Player>().unwrap();
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            apply_guest_act(&mut g, &mut player, 2, &a);
        }
        app.world_mut().insert_resource(player);
    }

    #[test]
    fn a_guests_throw_flies_from_the_host_and_counts_for_them() {
        let mut app = host_with_a_guest();
        let id = give_guest(&mut app, ItemKind::Teddy, None);
        act(&mut app, GuestAct::Throw { item: id, from: (8.0, 1.5, 0.0), vel: (0.0, 4.0, -12.0), charge: 0.6 });
        let g = app.world().resource::<Game>();
        let it = &g.world.items[&id];
        assert_eq!(it.state, ItemState::Flying);
        assert_eq!(it.thrower, Some(REMOTE_ID_BASE + 2));
        assert_eq!(it.vel.z, -12.0);
        let guest = g.dummies.iter().find(|d| d.id == REMOTE_ID_BASE + 2).unwrap();
        assert!(guest.bot.slots.is_empty());
        assert_eq!(g.board.get(REMOTE_ID_BASE + 2).map(|s| s.throws), Some(1));
    }

    #[test]
    fn a_throw_that_could_not_happen_is_ignored() {
        let mut app = host_with_a_guest();
        let id = give_guest(&mut app, ItemKind::Teddy, None);
        // from the other side of the yard
        act(&mut app, GuestAct::Throw { item: id, from: (-20.0, 1.5, 9.0), vel: (0.0, 1.0, -5.0), charge: 0.5 });
        // an item they do not hold
        act(&mut app, GuestAct::Throw { item: 9999, from: (8.0, 1.5, 0.0), vel: (0.0, 1.0, -5.0), charge: 0.5 });
        assert_eq!(app.world().resource::<Game>().world.items[&id].state, ItemState::Held);
        // stunned
        app.world_mut().resource_mut::<Game>().dummies.iter_mut().find(|d| d.remote.is_some()).unwrap().body.stun = 1.0;
        act(&mut app, GuestAct::Throw { item: id, from: (8.0, 1.5, 0.0), vel: (0.0, 1.0, -5.0), charge: 0.5 });
        assert_eq!(app.world().resource::<Game>().world.items[&id].state, ItemState::Held);
        // before the round
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.dummies.iter_mut().find(|d| d.remote.is_some()).unwrap().body.stun = 0.0;
            g.rules.phase = Phase::Countdown;
        }
        act(&mut app, GuestAct::Throw { item: id, from: (8.0, 1.5, 0.0), vel: (0.0, 1.0, -5.0), charge: 0.5 });
        assert_eq!(app.world().resource::<Game>().world.items[&id].state, ItemState::Held);
    }

    #[test]
    fn a_guest_walking_over_an_item_picks_it_up_and_can_choose_and_drop() {
        let mut app = host_with_a_guest();
        let (a, b) = {
            let mut g = app.world_mut().resource_mut::<Game>();
            let mut rng = bbq_core::rng::Rng::new(3);
            let a = g.world.spawn(ItemKind::Teddy, 8.0, 0.0, false, &mut rng);
            let b = g.world.spawn(ItemKind::Steak, 8.2, 0.1, false, &mut rng);
            (a, b)
        };
        for _ in 0..30 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        let guest = REMOTE_ID_BASE + 2;
        {
            let g = app.world().resource::<Game>();
            let d = g.dummies.iter().find(|d| d.id == guest).unwrap();
            assert!(d.bot.slots.ids().contains(&a) || d.bot.slots.ids().contains(&b), "picked something up");
        }
        let held: Vec<u32> = app.world().resource::<Game>().dummies.iter().find(|d| d.id == guest).unwrap().bot.slots.ids().to_vec();
        let first = held[0];
        act(&mut app, GuestAct::Select { item: first });
        assert_eq!(app.world().resource::<Game>().dummies.iter().find(|d| d.id == guest).unwrap().bot.slots.selected(), Some(first));
        // throw it away: it lands in front of them and they cannot grab it straight back
        act(&mut app, GuestAct::Drop { item: first });
        let g = app.world().resource::<Game>();
        let it = &g.world.items[&first];
        assert_ne!(it.state, ItemState::Held);
        assert_eq!(it.block.map(|b| b.0), Some(guest));
    }

    #[test]
    fn a_guests_slap_hits_the_person_in_front_of_them() {
        let mut app = host_with_a_guest_and(1);
        let id = give_guest(&mut app, ItemKind::Steak, None);
        // a bot 1.5 m in front of them (they look along -z); the host's own player is elsewhere
        app.world_mut().resource_mut::<Player>().mover = bbq_core::movement::Mover::new(-20.0, 10.0);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            g.options.bots_on = false;
            let b = g.dummies.iter_mut().find(|d| d.remote.is_none()).unwrap();
            b.mover.x = 8.0;
            b.mover.z = -1.5;
            g.world.items.get_mut(&id).unwrap().holder = Some(REMOTE_ID_BASE + 2);
        }
        act(&mut app, GuestAct::Slap { item: id });
        let g = app.world().resource::<Game>();
        let bot = g.dummies.iter().find(|d| d.remote.is_none()).unwrap();
        assert!(bot.body.stun > 0.5, "stunned ({})", bot.body.stun);
        assert_eq!(g.board.score(REMOTE_ID_BASE + 2), 50, "the guest is paid for it");
        // and they cannot swing again at once
        let before = g.board.score(REMOTE_ID_BASE + 2);
        act(&mut app, GuestAct::Slap { item: id });
        assert_eq!(app.world().resource::<Game>().board.score(REMOTE_ID_BASE + 2), before);
    }

    #[test]
    fn being_hit_online_is_told_to_the_person_hit() {
        let mut app = host_with_a_guest();
        // the host's own player throws a teddy straight at the guest, 6 m away
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let mut rng = bbq_core::rng::Rng::new(2);
            let id = g.world.spawn(ItemKind::Teddy, 8.0, 6.0, false, &mut rng);
            g.world.give(id, PLAYER_ID);
            g.world.throw(id, PLAYER_ID, V3::new(8.0, 1.2, 4.0), V3::new(0.0, 1.5, -14.0), 0.5, true);
        }
        for _ in 0..40 {
            app.world_mut().run_schedule(FixedUpdate);
        }
        let g = app.world().resource::<Game>();
        let guest = REMOTE_ID_BASE + 2;
        assert!(
            g.net_hits.iter().any(|(who, h)| *who == guest && matches!(h, HitMsg::Item { kind, .. } if ItemKind::ALL[*kind as usize] == ItemKind::Teddy)),
            "{:?}",
            g.net_hits
        );
        assert_eq!(g.board.get(PLAYER_ID).map(|s| s.hits), Some(1), "and the thrower scored");
    }

    #[test]
    fn a_slap_on_a_guest_is_told_to_them() {
        let mut app = host_with_a_guest_and(1);
        {
            let mut g = app.world_mut().resource_mut::<Game>();
            let i = g.dummies.iter().position(|d| d.remote.is_some()).unwrap();
            let fx = melee::dildo_slap(&melee::DildoRoll { pose: SlapKind::Cartwheel, crit: false, down: 3.5 });
            note_slap(&mut g, i, V3::new(1.0, 0.0, 0.0), &fx);
            // bots are not told anything
            if let Some(b) = g.dummies.iter().position(|d| d.remote.is_none()) {
                note_slap(&mut g, b, V3::new(1.0, 0.0, 0.0), &fx);
            }
        }
        let g = app.world().resource::<Game>();
        assert_eq!(g.net_hits.len(), 1);
        assert!(matches!(&g.net_hits[0].1, HitMsg::Slap { down: Some((1, d)), .. } if (*d - 3.5).abs() < 1e-5));
    }

    #[test]
    fn a_guest_is_knocked_about_when_the_host_says_so() {
        let mut guest = app();
        let mut p = guest.world_mut().remove_resource::<Player>().unwrap();
        let (x0, vy0) = (p.mover.x, p.mover.vy);
        {
            let mut g = guest.world_mut().resource_mut::<Game>();
            apply_hit(&mut g, &mut p, &HitMsg::Item { kind: kind_code(ItemKind::Stubby), dir: (1.0, 0.0, 0.0), flatten: false });
            assert!(g.me.body.stun > 0.0, "stunned");
            assert!(p.mover.vx > 1.0, "shoved the way it was going ({})", p.mover.vx);
            let _ = (x0, vy0);
            apply_hit(&mut g, &mut p, &HitMsg::Slap { dir: (0.0, 0.0, 1.0), knock: 13.0, up: 5.5, stun: 0.5, dizzy: 0.0, down: Some((0, 3.5)) });
            assert!(g.me.body.is_down(), "flattened by the dildo");
        }
        // nonsense is ignored
        let mut g = guest.world_mut().resource_mut::<Game>();
        apply_hit(&mut g, &mut p, &HitMsg::Item { kind: 200, dir: (0.0, 0.0, 1.0), flatten: false });
    }

    #[test]
    fn a_guests_throw_is_sent_to_the_host_and_the_item_leaves_their_hand() {
        let mut guest = app();
        let id = {
            let mut g = guest.world_mut().resource_mut::<Game>();
            g.mirror = true;
            g.world.clear();
            g.rules.phase = Phase::Play;
            let mut rng = bbq_core::rng::Rng::new(9);
            let id = g.world.spawn(ItemKind::Teddy, 8.0, 0.0, false, &mut rng);
            g.world.give(id, PLAYER_ID);
            g.slots.add(id);
            id
        };
        // wind up for a while, then let go
        guest.world_mut().resource_mut::<Wanted>().throw_down = true;
        for _ in 0..30 {
            guest.world_mut().run_schedule(FixedUpdate);
        }
        assert!(guest.world().resource::<Game>().wind.charging, "winding up");
        guest.world_mut().resource_mut::<Wanted>().throw_up = true;
        guest.world_mut().run_schedule(FixedUpdate);
        let g = guest.world().resource::<Game>();
        let sent: Vec<&GuestAct> = g.net_acts.iter().collect();
        assert!(matches!(sent.as_slice(), [GuestAct::Throw { item, charge, .. }] if *item == id && *charge > 0.2), "{sent:?}");
        assert!(g.slots.is_empty(), "the hand is empty at once");
        // the host's next picture still says we hold it: it must not jump back into the hand
        let snap = {
            let host = busy_host();
            let hg = host.world().resource::<Game>();
            let mut s = build_world(hg, &bbq_core::yard::Yard::default(), 1, 0.0, &[]);
            s.items = vec![ItemSnap { id, kind: kind_code(ItemKind::Teddy), variant: 255, state: STATE_HELD, holder: 3, pos: (8.0, 1.0, 0.0), vel: (0.0, 0.0, 0.0), ground_y: 0.0, team: 255, uses: 255 }];
            s
        };
        {
            let mut g = guest.world_mut().resource_mut::<Game>();
            let mut yard = bbq_core::yard::Yard::default();
            apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
            assert!(g.slots.is_empty(), "not yet: the host has not caught up");
        }
        // after a moment, if the host still says so, it is ours again
        for _ in 0..60 {
            guest.world_mut().run_schedule(FixedUpdate);
        }
        {
            let mut g = guest.world_mut().resource_mut::<Game>();
            let mut yard = bbq_core::yard::Yard::default();
            apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
            assert_eq!(g.slots.selected(), Some(id));
        }
    }

    #[test]
    fn a_guest_sees_what_the_host_says_they_hold() {
        let mut guest = app();
        let snap = {
            let host = busy_host();
            let hg = host.world().resource::<Game>();
            let mut s = build_world(hg, &bbq_core::yard::Yard::default(), 1, 0.0, &[]);
            s.items = vec![
                ItemSnap { id: 7, kind: kind_code(ItemKind::Dildo), variant: 4, state: STATE_HELD, holder: 3, pos: (0.0, 1.0, 0.0), vel: (0.0, 0.0, 0.0), ground_y: 0.0, team: 255, uses: 3 },
                ItemSnap { id: 8, kind: kind_code(ItemKind::Teddy), variant: 255, state: STATE_HELD, holder: 3, pos: (0.0, 1.0, 0.0), vel: (0.0, 0.0, 0.0), ground_y: 0.0, team: 255, uses: 255 },
                ItemSnap { id: 9, kind: kind_code(ItemKind::Steak), variant: 255, state: STATE_HELD, holder: 1, pos: (0.0, 1.0, 0.0), vel: (0.0, 0.0, 0.0), ground_y: 0.0, team: 255, uses: 3 },
            ];
            s
        };
        let mut g = guest.world_mut().resource_mut::<Game>();
        let mut yard = bbq_core::yard::Yard::default();
        apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
        assert_eq!(g.slots.ids(), &[7, 8], "what the host says guest number 3 holds");
        assert_eq!(g.slots.selected(), Some(8), "the newest in hand, as when you pick something up yourself");
        // somebody else's steak is on the host's puppet, not in our hands
        assert_eq!(g.world.items[&9].holder, Some(REMOTE_ID_BASE + 1));
        // a swap by us stays when the next picture arrives with nothing new
        g.slots.swap(1);
        let kept = g.slots.selected();
        apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
        assert_eq!(g.slots.selected(), kept);
    }

    #[test]
    fn the_whistle_opens_the_results_for_guests_too() {
        let host = busy_host();
        let hg = host.world().resource::<Game>();
        let hy = host.world().resource::<crate::yard_scene::YardRes>().0.clone();
        let mut snap = build_world(hg, &hy, 1, -1.0, &[]);
        snap.round.features |= FEAT_TIMED;
        snap.round.phase = Phase::Play.index();
        let mut guest = app();
        {
            let mut g = guest.world_mut().resource_mut::<Game>();
            let mut yard = bbq_core::yard::Yard::default();
            apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
            assert!(g.round.results.is_none(), "still playing");
            // the host's scores at the whistle: the host (net 1) 250, bot 101 on -50
            snap.round.phase = Phase::Results.index();
            apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
            let r = g.round.results.as_ref().expect("results are worked out from the host's scores");
            assert!(r.lines[0].contains("wins the round"), "{:?}", r.lines);
            assert_eq!(r.winner, Some(bbq_core::matchflow::Key::Player(REMOTE_ID_BASE + 1)), "the host won");
            assert!(g.round.panel_in > 0.0);
            // another picture of the same whistle does not work them out again
            let wins = g.round.mtch.wins.clone();
            apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
            assert_eq!(g.round.mtch.wins, wins);
            // and the next round clears them
            snap.round.phase = Phase::Countdown.index();
            apply_world(&mut g, &mut yard, &snap, 3, "Marcus");
            assert!(g.round.results.is_none());
        }
    }

    #[test]
    fn the_host_sends_only_new_feed_lines() {
        let host = busy_host();
        let g = host.world().resource::<Game>();
        let y = host.world().resource::<crate::yard_scene::YardRes>().0.clone();
        assert_eq!(build_world(g, &y, 1, -1.0, &[]).feed.len(), 1);
        let after = g.feed.last().unwrap().1;
        assert!(build_world(g, &y, 2, after, &[]).feed.is_empty());
    }
}


// ------------------------------------------------------------------ the guest's hands

/// A guest's hands: winding up, swapping, dropping and slapping feel the same as in your own
/// yard, but what you do is sent to the host (`Game::net_acts`) instead of done here. The host's
/// next picture then shows the result (your item in the air, the other person knocked over).
pub fn guest_hands(g: &mut Game, p: &mut Player, wanted: &mut Wanted, dt: f32) {
    let now = g.now;
    g.catcher.tick(dt);
    let before = g.slots.selected();
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
    if g.slots.selected() != before
        && let Some(id) = g.slots.selected()
    {
        g.net_acts.push(GuestAct::Select { item: id });
    }
    let in_play = g.rules.in_play();
    // G: throw it away
    if std::mem::take(&mut wanted.drop_held)
        && g.rules.phase != Phase::Countdown
        && g.me.body.stun <= 0.0
        && let Some(id) = g.slots.selected()
    {
        gone(g, id);
        g.wind.cancel();
        g.net_acts.push(GuestAct::Drop { item: id });
    }
    let selected_kind = g.slots.selected().and_then(|id| g.world.items.get(&id)).map(|i| i.kind);
    let situation = Situation {
        can_act: in_play && g.me.body.fall_t <= 0.0,
        stunned_standing: g.me.body.stun > 0.0 && g.me.body.down_t <= 0.0,
        drinking: g.me.drunk.is_drinking(),
        at_smoko: false,
        carrying_someone: false,
    };
    if std::mem::take(&mut wanted.throw_down) {
        match g.wind.press(now, selected_kind, &situation) {
            Press::SlapNow => guest_slap(g),
            Press::Charging | Press::Refused(_) => {}
        }
    }
    if g.wind.tick(dt, selected_kind)
        && let Some(id) = g.slots.selected()
    {
        // held it too long: it drops
        gone(g, id);
        g.net_acts.push(GuestAct::Drop { item: id });
        g.say("Held it too long! You dropped it.");
    }
    if std::mem::take(&mut wanted.throw_up) {
        match g.wind.release(now, selected_kind, situation.can_act) {
            Release::Throw { charge } => {
                if let (Some(id), Some(kind)) = (g.slots.selected(), selected_kind) {
                    let vel = hands::throw_velocity(
                        crate::game::aim_dir(p.yaw, p.pitch),
                        kind.def().speed,
                        charge,
                        V3::new(p.mover.vx, 0.0, p.mover.vz),
                    );
                    let start = crate::game::hand_pos(p);
                    g.net_acts.push(GuestAct::Throw {
                        item: id,
                        from: (start.x, start.y, start.z),
                        vel: (vel.x, vel.y, vel.z),
                        charge,
                    });
                    gone(g, id);
                }
            }
            Release::Slap => guest_slap(g),
            Release::Nothing => {}
        }
    }
    if std::mem::take(&mut wanted.catch) {
        let can = in_play && g.me.body.stun <= 0.0;
        g.catcher.press(can);
        if can {
            g.net_acts.push(GuestAct::Catch);
        }
    }
}

/// The item is out of our hands (thrown or dropped); until the host says so, do not show it.
fn gone(g: &mut Game, id: u32) {
    g.slots.remove(id);
    let until = g.now + 0.7;
    g.pending_gone.push((id, until));
}

/// Swing what is in the hand (if it is time to).
fn guest_slap(g: &mut Game) {
    let Some(id) = g.slots.selected() else { return };
    let Some(item) = g.world.items.get(&id) else { return };
    if item.kind.def().melee == Melee::None {
        return;
    }
    let mul = item.variant.filter(|_| item.kind == ItemKind::Dildo).map_or(1.0, |v| v.def().swing);
    if !(g.rules.in_play() && g.me.body.stun <= 0.0 && g.me.body.fall_t <= 0.0 && g.life.slap.ready_scaled(g.now, true, mul)) {
        return;
    }
    g.life.slap.mark(g.now);
    g.life.me_swing = melee::SWING_TIME * mul;
    g.net_acts.push(GuestAct::Slap { item: id });
}

/// The host says we were hit: knock our own blob about the same way.
pub fn apply_hit(g: &mut Game, p: &mut Player, hit: &HitMsg) {
    g.me.drunk.cancel_drink();
    g.wind.cancel();
    match hit {
        HitMsg::Item { kind, dir, flatten } => {
            let Some(&kind) = ItemKind::ALL.get(*kind as usize) else { return };
            let item = Item::new(0, kind, V3::ZERO);
            let dir = V3::new(dir.0, dir.1, dir.2);
            bbq_core::hitting::apply_item_hit(&mut g.me.body, &mut p.mover, &item, dir, *flatten);
            p.shake = p.shake.max(0.12 + kind.def().knock * 0.012);
        }
        HitMsg::Slap { dir, knock, up, stun, dizzy, down } => {
            let dir = V3::new(dir.0, dir.1, dir.2);
            let fx = SlapEffect {
                knock: *knock,
                up: *up,
                stun: *stun,
                dizzy: *dizzy,
                down: down.map(|(pose, secs)| {
                    (
                        match pose {
                            0 => SlapKind::SentFlying,
                            1 => SlapKind::Cartwheel,
                            _ => SlapKind::Timber,
                        },
                        secs,
                    )
                }),
                tumble: false,
            };
            melee::apply(&mut g.me.body, &mut p.mover, dir, &fx);
            p.shake = p.shake.max(0.25);
        }
    }
}

// ------------------------------------------------------------------ the host does what guests ask

/// The host noted that somebody playing online was slapped: tell them.
pub fn note_slap(g: &mut Game, i: usize, dir: V3, fx: &SlapEffect) {
    let d = &g.dummies[i];
    if d.remote.is_none() {
        return;
    }
    let down = fx.down.map(|(pose, secs)| {
        (
            match pose {
                SlapKind::SentFlying => 0u8,
                SlapKind::Cartwheel => 1,
                SlapKind::Timber => 2,
            },
            secs,
        )
    });
    let id = d.id;
    g.net_hits.push((id, HitMsg::Slap { dir: (dir.x, dir.y, dir.z), knock: fx.knock, up: fx.up, stun: fx.stun, dizzy: fx.dizzy, down }));
}

/// Somebody playing online walks over things and picks them up, like anybody else.
pub fn puppet_pickup(g: &mut Game, i: usize) {
    let d = &g.dummies[i];
    if d.body.stun > 0.0 || g.rules.phase == Phase::Countdown || d.seat.is_some() || d.dragged.is_some() {
        return;
    }
    let (id, pos, held) = (d.id, V3::new(d.mover.x, d.mover.y, d.mover.z), d.bot.slots.len());
    // a person, not a bot: bots leave the slapping things alone, people take everything
    if let Some(item) = crate::heist_app::try_pickup(g, id, pos, held, false, false) {
        g.world.give(item, id);
        g.dummies[i].bot.slots.add(item);
    }
}

/// Do what a guest asked, as far as the rules allow.
pub fn apply_guest_act(g: &mut Game, p: &mut Player, net_id: u32, act: &GuestAct) {
    let Some(i) = g.dummies.iter().position(|d| d.remote.as_ref().is_some_and(|r| r.net_id == net_id)) else { return };
    if std::env::args().any(|a| a == "--net-debug") {
        println!("NET-HOST guest {net_id} asked: {act:?}");
    }
    let id = g.dummies[i].id;
    let now = g.now;
    let in_play = g.rules.in_play();
    let can_act = in_play && g.dummies[i].body.stun <= 0.0 && !g.dummies[i].body.is_down() && g.dummies[i].seat.is_none();
    let holds = |g: &Game, item: u32| g.dummies[i].bot.slots.ids().contains(&item);
    match act {
        GuestAct::Select { item } => {
            let slots = &mut g.dummies[i].bot.slots;
            if let Some(n) = slots.ids().iter().position(|x| x == item) {
                slots.select_slot(n);
            }
        }
        GuestAct::Catch => {
            if in_play && g.dummies[i].body.stun <= 0.0 {
                g.dummies[i].bot.catcher.press(true);
            }
        }
        GuestAct::Drop { item } => {
            if holds(g, *item) && g.rules.phase != Phase::Countdown {
                let d = &g.dummies[i];
                let f = (d.face.sin(), d.face.cos());
                let at = V3::new(d.mover.x + f.0 * 0.7, d.mover.y + 1.1, d.mover.z + f.1 * 0.7);
                g.dummies[i].bot.slots.remove(*item);
                g.world.drop_item(*item, at, Some((id, now + hands::OVERHOLD_BLOCK)));
                if let Some(it) = g.world.items.get_mut(item) {
                    it.vel = V3::new(f.0 * 2.2, 1.0, f.1 * 2.2);
                }
            }
        }
        GuestAct::Throw { item, from, vel, charge } => {
            if !can_act || !holds(g, *item) {
                return;
            }
            let d = &g.dummies[i];
            let start = V3::new(from.0, from.1, from.2);
            // the throw has to start roughly where we last saw them
            if (start.x - d.mover.x).hypot(start.z - d.mover.z) > MAX_THROW_START {
                return;
            }
            if g.world.throw(*item, id, start, V3::new(vel.0, vel.1, vel.2), *charge, true) {
                g.dummies[i].bot.slots.remove(*item);
                g.board.count_throw(id);
                g.dummies[i].anim.start_swing();
                g.dummies[i].bot.swing_t = melee::SWING_TIME;
            }
        }
        GuestAct::Slap { item } => {
            let mul = g.world.items.get(item).filter(|it| it.kind == ItemKind::Dildo).and_then(|it| it.variant).map_or(1.0, |v| v.def().swing);
            if !can_act || !holds(g, *item) || now - g.dummies[i].net_slap_at < 0.4 * mul {
                return;
            }
            let Some(kind) = g.world.items.get(item).map(|it| it.kind) else { return };
            if kind.def().melee == Melee::None {
                return;
            }
            g.dummies[i].net_slap_at = now;
            // make it the item in their hand, then swing it at whoever is in front of them
            let slots = &mut g.dummies[i].bot.slots;
            if let Some(n) = slots.ids().iter().position(|x| x == item) {
                slots.select_slot(n);
            }
            let d = &g.dummies[i];
            let me = V3::new(d.mover.x, d.mover.y, d.mover.z);
            let face = (d.face.sin(), d.face.cos());
            let mut cands: Vec<melee::Candidate> = g
                .dummies
                .iter()
                .filter(|o| o.id != id)
                .map(|o| melee::Candidate {
                    id: o.id,
                    pos: V3::new(o.mover.x, o.mover.y, o.mover.z),
                    down_t: o.body.down_t.max(o.body.fall_t),
                    at_smoko: o.seat.is_some(),
                    teammate: g.teams.same_team(id, o.id),
                })
                .collect();
            cands.push(melee::Candidate {
                id: PLAYER_ID,
                pos: V3::new(p.mover.x, p.mover.y, p.mover.z),
                down_t: g.me.body.down_t.max(g.me.body.fall_t),
                at_smoko: g.life.seated.is_some(),
                teammate: g.teams.same_team(id, PLAYER_ID),
            });
            let reach = melee::REACH * g.world.items.get(item).filter(|it| it.kind == ItemKind::Dildo).and_then(|it| it.variant).map_or(1.0, |v| v.def().reach);
            let sw = match melee::pick_target_reach(me, face, &cands, g.rules.friendly_fire, reach) {
                Some(v) => bbq_core::bots::Swing::At(v),
                None => bbq_core::bots::Swing::Air,
            };
            crate::bots_app::swing(g, p, i, sw);
        }
    }
}
