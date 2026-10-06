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
use crate::player::{PLAYER_ID, Wanted};

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

fn kind_code(k: ItemKind) -> u8 {
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

/// One tick of a mirrored game: move things that were moving between the host's pictures, and
/// forget button presses (the guest's actions go to the host in the next step).
pub fn mirror_tick(g: &mut Game, wanted: &mut Wanted, dt: f32) {
    g.now += dt;
    for pop in &mut g.popups {
        pop.t += dt;
    }
    g.popups.retain(|p| p.t < 1.6);
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
