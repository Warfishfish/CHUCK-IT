//! Online play, steps 9.2 and 9.3: joining a yard through the relay, and seeing each other move.
//!
//! One player hosts, the others join with the room code. Everybody moves their own blob and
//! says where it is 20 times a second; everybody else's blob is a puppet that follows those
//! messages smoothly (see `Dummy::remote`). The rules (hits, scores, bots, items) are not shared
//! yet: that is the next step, so for now it is "walk about in the same yard".
//!
//! `Session` is the whole conversation as plain code with no sockets, so it can be tested;
//! the Bevy systems at the bottom just feed it what the connection hears and send what it says.

use std::collections::HashMap;

use bbq_core::character::Character;
use bbq_core::net_act::{GuestAct, HitMsg};
use bbq_core::net::{Member, Msg, PROTOCOL, PlayerState, Refusal, Roster, STATE_HZ, clean};
use bevy::prelude::*;

use crate::game::{Game, RemoteInfo};
use crate::net_link::{In, Link, Out, ws_url};
use crate::player::{PLAYER_ID, Player};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Host,
    Guest,
}

/// What the session wants done (the glue does it).
#[derive(Debug, PartialEq)]
pub enum Do {
    /// `to` is a relay peer, or everybody else when `None`.
    Send { to: Option<String>, msg: Msg },
    Join(String),
    Disconnect,
}

/// Somebody else's latest word on where they are.
#[derive(Clone, Copy, Debug)]
pub struct Heard {
    pub state: PlayerState,
    /// Game clock when it arrived.
    pub at: f32,
}

pub struct Session {
    pub role: Role,
    pub room: String,
    name: String,
    character: u8,
    /// Our number in the yard (the host is 1; a guest has 0 until the host says).
    pub my_id: u32,
    roster: Roster,
    /// Everybody in the yard, as the host last listed them (always including us once in).
    pub members: Vec<Member>,
    /// Host only: which relay peer is which player.
    peers: HashMap<String, u32>,
    /// Guest only: who the host is on the relay.
    host_peer: Option<String>,
    pub heard: HashMap<u32, Heard>,
    /// Guest: the host's latest picture of the yard, until the game has used it.
    pub world: Option<bbq_core::net_world::WorldSnap>,
    world_seq: u32,
    /// Host: what guests have asked for (who, what), until the game has done it.
    pub acts: Vec<(u32, GuestAct)>,
    /// Guest: what happened to us, until the game has applied it.
    pub hits: Vec<HitMsg>,
    /// A line for the menu.
    pub status: String,
    /// The yard is closed to newcomers (a round is on).
    pub round_on: bool,
    /// Set when the session is over (refused, host left, connection lost).
    pub ended: bool,
}

impl Session {
    pub fn host(name: &str, character: Character, room: &str) -> Self {
        let name = clean(name);
        let character = char_code(character);
        let roster = Roster::new(&name, character);
        Session {
            role: Role::Host,
            room: room.to_string(),
            members: roster.members().to_vec(),
            roster,
            name,
            character,
            my_id: 1,
            peers: HashMap::new(),
            host_peer: None,
            heard: HashMap::new(),
            world: None,
            world_seq: 0,
            acts: Vec::new(),
            hits: Vec::new(),
            status: "Connecting...".into(),
            round_on: false,
            ended: false,
        }
    }

    pub fn guest(name: &str, character: Character, room: &str) -> Self {
        Session {
            role: Role::Guest,
            room: room.to_string(),
            name: clean(name),
            character: char_code(character),
            my_id: 0,
            roster: Roster::default(),
            members: Vec::new(),
            peers: HashMap::new(),
            host_peer: None,
            heard: HashMap::new(),
            world: None,
            world_seq: 0,
            acts: Vec::new(),
            hits: Vec::new(),
            status: "Connecting...".into(),
            round_on: false,
            ended: false,
        }
    }

    /// Are we in the yard (host: connected; guest: welcomed)?
    pub fn joined(&self) -> bool {
        self.my_id != 0 && !self.ended
    }

    fn hello(&self) -> Msg {
        Msg::Hello { protocol: PROTOCOL, name: self.name.clone(), character: self.character }
    }

    fn end(&mut self, why: &str) -> Vec<Do> {
        self.ended = true;
        self.status = why.to_string();
        vec![Do::Disconnect]
    }

    fn line(&mut self) {
        let others = self.members.len().saturating_sub(1);
        self.status = match (self.role, others) {
            (Role::Host, 0) => format!("Hosting {}: waiting for mates", self.room),
            (Role::Host, n) => format!("Hosting {}: {n} mate{} here", self.room, if n == 1 { "" } else { "s" }),
            (Role::Guest, n) => format!("In {}: {n} other{} here", self.room, if n == 1 { "" } else { "s" }),
        };
    }

    /// Something happened on the connection.
    pub fn on_event(&mut self, ev: In, now: f32) -> Vec<Do> {
        match ev {
            In::Open { .. } => vec![Do::Join(self.room.clone())],
            In::Snap { peers, .. } => {
                if self.role == Role::Host {
                    // anybody already here is a mate who got here first (they say hello again
                    // when they see us arrive), so there is nothing to refuse
                    self.line();
                    Vec::new()
                } else {
                    self.status = "Asking to join...".into();
                    vec![Do::Send { to: None, msg: self.hello() }]
                }
            }
            In::Up { peer, .. } => {
                // a guest who came first says hello again to a host who turns up later
                if self.role == Role::Guest && self.my_id == 0 {
                    vec![Do::Send { to: Some(peer), msg: self.hello() }]
                } else {
                    Vec::new()
                }
            }
            In::Left { peer, .. } => match self.role {
                Role::Host => match self.peers.remove(&peer) {
                    Some(id) => {
                        self.roster.remove(id);
                        self.heard.remove(&id);
                        self.members = self.roster.members().to_vec();
                        self.line();
                        vec![Do::Send { to: None, msg: Msg::Roster(self.members.clone()) }]
                    }
                    None => Vec::new(),
                },
                Role::Guest => {
                    if self.host_peer.as_deref() == Some(peer.as_str()) {
                        self.end("The host left, so the yard is closed")
                    } else {
                        Vec::new()
                    }
                }
            },
            In::Refused { code, .. } => self.end(&if code == "limit_reached" { "The server is full".to_string() } else { format!("The server said no ({code})") }),
            In::Closed(why) => {
                if self.ended {
                    Vec::new()
                } else {
                    self.ended = true;
                    self.status = format!("Disconnected: {why}");
                    Vec::new()
                }
            }
            In::Msg { from, data, .. } => match Msg::decode(&data) {
                Ok(m) => self.on_msg(from, m, now),
                Err(_) => Vec::new(),
            },
        }
    }

    fn on_msg(&mut self, from: String, msg: Msg, now: f32) -> Vec<Do> {
        match (self.role, msg) {
            (Role::Host, Msg::Hello { protocol, name, character }) => {
                // the same person saying hello twice is still one person: say welcome again
                if let Some(&id) = self.peers.get(&from) {
                    return vec![Do::Send { to: Some(from), msg: Msg::Welcome { id, members: self.members.clone() } }];
                }
                match self.roster.admit(protocol, &name, character, self.round_on) {
                    Ok(id) => {
                        self.peers.insert(from.clone(), id);
                        self.members = self.roster.members().to_vec();
                        self.line();
                        vec![
                            Do::Send { to: Some(from), msg: Msg::Welcome { id, members: self.members.clone() } },
                            Do::Send { to: None, msg: Msg::Roster(self.members.clone()) },
                        ]
                    }
                    Err(r) => vec![Do::Send { to: Some(from), msg: Msg::Refused(r) }],
                }
            }
            (Role::Host, Msg::State(s)) => {
                // a guest may only speak for themselves
                if self.peers.get(&from) == Some(&s.id) {
                    self.heard.insert(s.id, Heard { state: s, at: now });
                }
                Vec::new()
            }
            (Role::Guest, Msg::Welcome { id, members }) => {
                if self.my_id == 0 && id != 0 {
                    self.my_id = id;
                    self.host_peer = Some(from);
                    self.members = members;
                    self.line();
                }
                Vec::new()
            }
            (Role::Guest, Msg::Refused(r)) => self.end(match r {
                Refusal::WrongVersion => "The host has a different version of the game",
                Refusal::Full => "That yard is full",
                Refusal::InProgress => "A round is already on: wait for it to finish",
            }),
            (Role::Guest, Msg::Roster(ms)) => {
                if self.my_id != 0 && self.host_peer.as_deref() == Some(from.as_str()) {
                    // forget the people who have left
                    self.heard.retain(|id, _| ms.iter().any(|m| m.id == *id));
                    self.members = ms;
                    self.line();
                }
                Vec::new()
            }
            (Role::Guest, Msg::World(w)) => {
                // only the host describes the yard, and only newer pictures count
                if self.my_id != 0 && self.host_peer.as_deref() == Some(from.as_str()) && (w.seq > self.world_seq || self.world.is_none()) {
                    self.world_seq = w.seq;
                    self.world = Some(*w);
                }
                Vec::new()
            }
            (Role::Host, Msg::Act(a)) => {
                // a guest asks for something: only from somebody we let in, and only sane things
                if let Some(&id) = self.peers.get(&from)
                    && a.sane()
                    && self.acts.len() < 64
                {
                    self.acts.push((id, a));
                }
                Vec::new()
            }
            (Role::Guest, Msg::Hit(h)) => {
                if self.my_id != 0 && self.host_peer.as_deref() == Some(from.as_str()) && self.hits.len() < 16 {
                    self.hits.push(h);
                }
                Vec::new()
            }
            (Role::Guest, Msg::State(s)) => {
                if self.my_id != 0 && s.id != self.my_id && self.members.iter().any(|m| m.id == s.id) {
                    self.heard.insert(s.id, Heard { state: s, at: now });
                }
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    /// Guest: who the host is on the relay.
    pub fn host_peer(&self) -> Option<&str> {
        self.host_peer.as_deref()
    }

    /// Host: the relay peer of a player (to send them something privately).
    pub fn peer_of(&self, id: u32) -> Option<&str> {
        self.peers.iter().find(|(_, v)| **v == id).map(|(k, _)| k.as_str())
    }

    /// The host's name (for the feed lines that say "You").
    pub fn host_name(&self) -> String {
        self.members.iter().find(|m| m.host).map_or("The host".to_string(), |m| m.name.clone())
    }

    /// The other people, as the game wants them.
    pub fn others(&self) -> Vec<RemoteInfo> {
        self.members
            .iter()
            .filter(|m| m.id != self.my_id && self.my_id != 0)
            .map(|m| RemoteInfo { net_id: m.id, name: m.name.clone(), character: code_char(m.character) })
            .collect()
    }
}

pub fn char_code(c: Character) -> u8 {
    Character::ALL.iter().position(|x| *x == c).unwrap_or(0) as u8
}

pub fn code_char(n: u8) -> Character {
    Character::ALL[(n as usize).min(Character::ALL.len() - 1)]
}

/// A made-up room code people can read out: `rbq-` and four letters and digits.
pub fn new_room_code(seed: u64) -> String {
    const SYMBOLS: &[u8] = b"abcdefghjkmnpqrstuvwxyz23456789";
    let mut x = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut s = String::from("rbq-");
    for _ in 0..4 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.push(SYMBOLS[(x % SYMBOLS.len() as u64) as usize] as char);
    }
    s
}

/// Room codes people type: lower case letters, digits and dashes, 3 to 24 long. The relay only
/// allows `[a-z0-9][a-z0-9_.-]{0,47}`.
pub fn clean_room(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .take(24)
        .collect()
}

// ---------------------------------------------------------------- the game side

/// What the menu asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetRequest {
    Host,
    Join,
    Leave,
}

#[derive(Resource, Default)]
pub struct Online {
    pub link: Option<Link>,
    pub session: Option<Session>,
    /// Seconds until the next report of where we are.
    send_in: f32,
    /// What the menu shows while there is no session (an error before connecting).
    pub note: String,
}

impl Online {
    pub fn status(&self) -> String {
        match &self.session {
            Some(s) => s.status.clone(),
            None => self.note.clone(),
        }
    }

    pub fn active(&self) -> bool {
        self.session.as_ref().is_some_and(|s| !s.ended)
    }

    pub fn leave(&mut self) {
        if let Some(l) = &self.link {
            let _ = l.out.send(Out::Close);
        }
        self.link = None;
        self.session = None;
        self.send_in = 0.0;
    }

    pub fn start(&mut self, req: NetRequest, s: &crate::menu::Settings, now_seed: u64) {
        self.leave();
        self.note.clear();
        if req == NetRequest::Leave {
            return;
        }
        let Some(url) = ws_url(&s.server) else {
            self.note = "Type the server's address first (the one running npm start)".into();
            return;
        };
        let room = if req == NetRequest::Host && clean_room(&s.room).len() < 3 { new_room_code(now_seed) } else { clean_room(&s.room) };
        if room.len() < 3 {
            self.note = "Type the room code your mate gave you".into();
            return;
        }
        self.session = Some(match req {
            NetRequest::Host => Session::host(&s.name, s.character, &room),
            _ => Session::guest(&s.name, s.character, &room),
        });
        self.link = Some(Link::connect(url));
    }
}

pub struct OnlinePlugin;

impl Plugin for OnlinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Online>()
            .add_systems(Update, (net_requests, online_poll, online_send, online_roster, online_world_send, online_world_apply, online_follow_host, net_test_actions, online_acts, online_hits).chain())
            .add_systems(FixedUpdate, online_puppets.after(crate::game::step_game));
    }
}

/// The menu's Host, Join and Leave buttons. For testing, `--net-host ROOM` and `--net-join ROOM`
/// (with `--server ADDRESS`) press them for you a moment after start-up, and `--net-debug`
/// prints who is where once a second.
fn net_requests(
    mut ui: ResMut<crate::menu::MenuUi>,
    mut settings: ResMut<crate::menu::Settings>,
    mut online: ResMut<Online>,
    time: Res<Time>,
    game: Res<Game>,
    mut frames: Local<u32>,
    mut last_debug: Local<f32>,
    mut played: Local<bool>,
    screen: Res<crate::menu::Screen>,
) {
    *frames += 1;
    if *frames == 3 {
        let args: Vec<String> = std::env::args().collect();
        let after = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        if let Some(sv) = after("--server") {
            settings.server = sv;
        }
        if let Some(secs) = after("--net-round").and_then(|v| v.parse::<f32>().ok()) {
            settings.round_len = secs;
        }
        for (flag, req) in [("--net-host", NetRequest::Host), ("--net-join", NetRequest::Join)] {
            if let Some(room) = after(flag) {
                settings.room = clean_room(&room);
                ui.net_request = Some(req);
            }
        }
    }
    // `--net-play SECONDS`: press Play that long after start-up (a host waiting for a guest)
    if !*played && let Some(secs) = std::env::args().position(|a| a == "--net-play").and_then(|i| std::env::args().nth(i + 1)).and_then(|v| v.parse::<f32>().ok()) {
        if time.elapsed_secs() > secs {
            *played = true;
            ui.request = Some(crate::menu::Request::Play);
        }
    }
    if let Some(req) = ui.net_request.take() {
        let seed = (time.elapsed_secs_f64() * 1000.0) as u64 ^ std::process::id() as u64;
        online.start(req, &settings, seed);
    }
    if std::env::args().any(|a| a == "--net-debug") && time.elapsed_secs() - *last_debug >= 1.0 {
        *last_debug = time.elapsed_secs();
        let who: Vec<String> = game
            .dummies
            .iter()
            .filter_map(|d| d.remote.as_ref().map(|r| format!("{} at ({:.1}, {:.1})", r.name, d.mover.x, d.mover.z)))
            .collect();
        let bot0 = game.dummies.iter().find(|d| d.remote.is_none()).map_or("none".to_string(), |d| format!("{:.1},{:.1}", d.mover.x, d.mover.z));
        println!(
            "ONLINE screen={:?} status=\"{}\" members=[{}] puppets=[{}] mirror={} items={} bots={} ids={:?} remotes={} bot0=({bot0}) phase={:?} clock={:.0} dazza=({:.1},{:.1})",
            *screen,
            online.status(),
            online.session.as_ref().map_or(String::new(), |s| s.members.iter().map(|m| format!("{}#{}", m.name, m.id)).collect::<Vec<_>>().join(",")),
            who.join("; "),
            game.mirror,
            game.world.items.len(),
            game.dummies.iter().filter(|d| d.remote.is_none()).count(),
            game.dummies.iter().map(|d| d.id).collect::<Vec<_>>(),
            game.remotes.len(),
            game.rules.phase,
            game.round.time_left,
            game.life.dazza.pos.x,
            game.life.dazza.pos.z
        );
    }
}

/// Hear the connection and act on it.
fn online_poll(mut online: ResMut<Online>, game: Res<Game>) {
    let Online { link, session, .. } = &mut *online;
    let (Some(link), Some(session)) = (link.as_ref(), session.as_mut()) else {
        return;
    };
    let now = game.now;
    while let Some(ev) = link.poll() {
        if std::env::args().any(|a| a == "--net-debug") {
            match &ev {
                In::Msg { from, data, .. } => println!("NET-EVENT msg from {from}: {:?}", Msg::decode(data).map(|m| format!("{m:?}").chars().take(60).collect::<String>())),
                other => println!("NET-EVENT {other:?}"),
            }
        }
        for act in session.on_event(ev, now) {
            match act {
                Do::Join(room) => {
                    let _ = link.out.send(Out::Join(room));
                }
                Do::Send { to, msg } => link.send(&session.room, to.as_deref(), msg.encode()),
                Do::Disconnect => {
                    let _ = link.out.send(Out::Close);
                }
            }
        }
    }
    if session.ended && !session.joined() {
        // nothing more to do; the status line says why
    }
}

/// Say where we are, 20 times a second.
fn online_send(time: Res<Time>, mut online: ResMut<Online>, player: Res<Player>, game: Res<Game>, settings: Res<crate::menu::Settings>) {
    let Online { link, session, send_in, .. } = &mut *online;
    let (Some(link), Some(session)) = (link.as_ref(), session.as_ref()) else {
        return;
    };
    if !session.joined() {
        return;
    }
    *send_in -= time.delta_secs();
    if *send_in > 0.0 {
        return;
    }
    *send_in = 1.0 / STATE_HZ;
    let held = game
        .slots
        .selected()
        .and_then(|id| game.world.items.get(&id))
        .map_or((0, 0), |it| (item_code(it.kind), it.variant.map_or(0, variant_code)));
    let m = &player.mover;
    let state = PlayerState {
        id: session.my_id,
        pos: (m.x, m.y, m.z),
        yaw: player.yaw,
        vel: (m.vx, m.vz),
        walk: player.walk,
        grounded: m.grounded,
        stunned: game.me.body.stun > 0.0,
        down: game.me.body.is_down(),
        drinking: game.me.drunk.is_drinking(),
        charge: if game.wind.charging { game.wind.charge } else { 0.0 },
        held: held.0,
        held_variant: held.1,
        belly: (settings.belly.clamp(0.0, 2.0) / 2.0 * 255.0).round() as u8,
    };
    link.send(&session.room, None, Msg::State(state).encode());
}

fn item_code(k: bbq_core::items::ItemKind) -> u8 {
    bbq_core::items::ItemKind::ALL.iter().position(|x| *x == k).map_or(0, |i| i as u8 + 1)
}

fn variant_code(v: bbq_core::items::DildoVariant) -> u8 {
    bbq_core::items::DildoVariant::ALL.iter().position(|x| *x == v).unwrap_or(0) as u8
}

/// Keep the puppets in the game matching who is in the yard.
fn online_roster(mut online: ResMut<Online>, mut game: ResMut<Game>, screen: Res<crate::menu::Screen>) {
    // the host closes the door once a round is on
    if let Some(s) = online.session.as_mut() {
        s.round_on = *screen == crate::menu::Screen::Playing;
    }
    let want = online.session.as_ref().filter(|s| !s.ended).map(Session::others).unwrap_or_default();
    let (guest, host_with_mates) = online
        .session
        .as_ref()
        .filter(|s| !s.ended)
        .map_or((false, false), |s| (s.role == Role::Guest && s.joined(), s.role == Role::Host && s.members.len() > 1));
    if game.mirror != guest {
        game.mirror = guest;
        if !guest {
            // back to running our own yard
            game.world.clear();
        }
    }
    game.net_fx = host_with_mates;
    if game.remotes != want {
        game.remotes = want;
        let bots = game.dummies.iter().filter(|d| d.remote.is_none()).count();
        crate::round::sync_bot_count(&mut game, bots);
    }
}

/// The host tells everyone how the yard looks, 15 times a second.
fn online_world_send(
    time: Res<Time>,
    mut online: ResMut<Online>,
    mut game: ResMut<Game>,
    yard: Res<crate::yard_scene::YardRes>,
    mut every: Local<f32>,
    mut seq: Local<u32>,
    mut feed_sent: Local<f32>,
) {
    let Online { link, session, .. } = &mut *online;
    let (Some(link), Some(session)) = (link.as_ref(), session.as_ref()) else { return };
    if session.role != Role::Host || !session.joined() || session.members.len() < 2 {
        game.fx_out.clear();
        return;
    }
    *every -= time.delta_secs();
    if *every > 0.0 {
        return;
    }
    *every = 1.0 / crate::online_world::WORLD_HZ;
    *seq = seq.wrapping_add(1);
    let fx = std::mem::take(&mut game.fx_out);
    let snap = crate::online_world::build_world(&game, &yard.0, *seq, *feed_sent, &fx);
    if let Some((_, t)) = game.feed.last() {
        *feed_sent = *t;
    }
    link.send(&session.room, None, Msg::World(Box::new(snap)).encode());
}

/// A guest copies the host's picture of the yard.
fn online_world_apply(mut online: ResMut<Online>, mut game: ResMut<Game>, mut yard: ResMut<crate::yard_scene::YardRes>) {
    let Some(session) = online.session.as_mut().filter(|s| s.role == Role::Guest && s.joined()) else { return };
    let Some(snap) = session.world.take() else { return };
    let (my_id, host) = (session.my_id, session.host_name());
    crate::online_world::apply_world(&mut game, &mut yard.0, &snap, my_id, &host);
}

/// A guest goes where the host goes: into the round when the host starts one, and back to the
/// menu when the host leaves the round.
fn online_follow_host(
    online: Res<Online>,
    screen: Res<crate::menu::Screen>,
    game: Res<Game>,
    mut ui: ResMut<crate::menu::MenuUi>,
    mut last: Local<Option<(u8, bool)>>,
) {
    use bbq_core::scoring::Phase;
    let Some(s) = online.session.as_ref().filter(|s| s.role == Role::Guest && s.joined()) else {
        *last = None;
        return;
    };
    if !game.mirror {
        return;
    }
    // what the host is doing (as of the last picture): in a round, or standing about in its menu
    let in_round = game.round.timed && matches!(game.rules.phase, Phase::Countdown | Phase::Play);
    let _ = s;
    let now = (game.rules.phase.index(), in_round);
    if *last == Some(now) {
        return;
    }
    // only act on a change we have seen, and only once a picture has arrived
    if game.rules.phase == Phase::Menu && !game.round.timed && last.is_none() {
        *last = Some(now);
        return;
    }
    *last = Some(now);
    if let Some(req) = follow_decision(*screen, in_round, game.round.timed) {
        ui.request = Some(req);
    }
}

/// What a guest does when the host's state changes: join a round that starts, go back to the
/// menu when the host leaves the round (but not at the whistle: the results card is shown).
pub fn follow_decision(screen: crate::menu::Screen, host_in_round: bool, host_timed: bool) -> Option<crate::menu::Request> {
    use crate::menu::{Request, Screen};
    match (screen, host_in_round) {
        (Screen::Menu | Screen::Results, true) => Some(Request::Follow),
        (Screen::Playing | Screen::Paused, false) if !host_timed => Some(Request::Menu),
        _ => None,
    }
}

/// Testing: `--net-give KIND` (host) hands the first person online one of those; `--net-throw`
/// (guest) throws whatever it holds, a moment after getting it. Both print what happened.
fn net_test_actions(
    online: Res<Online>,
    mut game: ResMut<Game>,
    mut wanted: ResMut<crate::player::Wanted>,
    mut given: Local<bool>,
    mut stage: Local<(u32, bool)>,
) {
    let args: Vec<String> = std::env::args().collect();
    let Some(session) = online.session.as_ref().filter(|s| s.joined()) else { return };
    if session.role == Role::Host && !*given && game.rules.phase == bbq_core::scoring::Phase::Play {
        if let Some(kind) = args.iter().position(|a| a == "--net-give").and_then(|i| args.get(i + 1)) {
            let want = match kind.as_str() {
                "steak" => bbq_core::items::ItemKind::Steak,
                _ => bbq_core::items::ItemKind::Teddy,
            };
            if let Some(i) = game.dummies.iter().position(|d| d.remote.is_some()) {
                let (id, x, z) = (game.dummies[i].id, game.dummies[i].mover.x, game.dummies[i].mover.z);
                let mut rng = game.rng.clone();
                let item = game.world.spawn(want, x, z, false, &mut rng);
                game.world.give(item, id);
                game.dummies[i].bot.slots.add(item);
                *given = true;
                println!("NET-TEST host gave {want:?} {item} to {id}");
            }
        }
    }
    if session.role == Role::Guest && args.iter().any(|a| a == "--net-throw") {
        if !game.slots.is_empty() && game.rules.phase == bbq_core::scoring::Phase::Play {
            stage.0 += 1;
            if stage.0 == 30 {
                wanted.throw_down = true;
            }
            if stage.0 == 55 && !stage.1 {
                wanted.throw_up = true;
                stage.1 = true;
                println!("NET-TEST guest let go of a throw");
            }
        }
    }
}

/// A guest sends what it wants done to the host; the host does what guests have asked.
fn online_acts(
    mut online: ResMut<Online>,
    mut game: ResMut<Game>,
    mut player: ResMut<Player>,
) {
    let Online { link, session, .. } = &mut *online;
    let (Some(link), Some(session)) = (link.as_ref(), session.as_mut()) else {
        game.net_acts.clear();
        return;
    };
    match session.role {
        Role::Guest => {
            let acts = std::mem::take(&mut game.net_acts);
            if let (true, Some(host)) = (session.joined(), session.host_peer().map(str::to_string)) {
                for a in acts {
                    link.send(&session.room, Some(&host), Msg::Act(a).encode());
                }
            }
        }
        Role::Host => {
            game.net_acts.clear();
            for (id, act) in std::mem::take(&mut session.acts) {
                crate::online_world::apply_guest_act(&mut game, &mut player, id, &act);
            }
        }
    }
}

/// The host tells people when they were hit; a guest gets knocked about to match.
fn online_hits(mut online: ResMut<Online>, mut game: ResMut<Game>, mut player: ResMut<Player>) {
    let Online { link, session, .. } = &mut *online;
    let (Some(link), Some(session)) = (link.as_ref(), session.as_mut()) else {
        game.net_hits.clear();
        return;
    };
    match session.role {
        Role::Host => {
            for (victim, hit) in std::mem::take(&mut game.net_hits) {
                let net = crate::online_world::net_of_local(victim, 1);
                if let Some(peer) = session.peer_of(net).map(str::to_string) {
                    link.send(&session.room, Some(&peer), Msg::Hit(hit).encode());
                }
            }
        }
        Role::Guest => {
            game.net_hits.clear();
            for hit in std::mem::take(&mut session.hits) {
                crate::online_world::apply_hit(&mut game, &mut player, &hit);
            }
        }
    }
}

/// Move each puppet to where its person says they are (a little ahead, by their speed, and
/// smoothed so 20 reports a second look like 60 frames).
fn online_puppets(online: Res<Online>, mut game: ResMut<Game>) {
    let Some(session) = online.session.as_ref().filter(|s| !s.ended) else {
        return;
    };
    let now = game.now;
    for d in game.dummies.iter_mut() {
        let Some(r) = &d.remote else { continue };
        let Some(h) = session.heard.get(&r.net_id) else { continue };
        let s = h.state;
        let age = (now - h.at).clamp(0.0, 0.25);
        let target = (s.pos.0 + s.vel.0 * age, s.pos.1, s.pos.2 + s.vel.1 * age);
        let k = 1.0 - (-18.0 * bbq_core::sim::TICK_DT).exp();
        let far = (target.0 - d.mover.x).hypot(target.2 - d.mover.z) > 4.0;
        let k = if far { 1.0 } else { k };
        d.mover.x += (target.0 - d.mover.x) * k;
        d.mover.z += (target.2 - d.mover.z) * k;
        d.mover.y = target.1;
        d.mover.vx = s.vel.0;
        d.mover.vz = s.vel.1;
        d.mover.grounded = s.grounded;
        // the game's `yaw` looks along (-sin, -cos); a puppet's `face` looks along (sin, cos)
        d.face = (-s.yaw.sin()).atan2(-s.yaw.cos());
        d.bot.winding = s.charge > 0.0;
        d.bot.wind_progress = s.charge;
        d.belly = s.belly as f32 / 255.0 * 2.0;
    }
    let _ = PLAYER_ID;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bytes(m: &Msg) -> Vec<u8> {
        m.encode()
    }

    fn msg_in(from: &str, m: &Msg) -> In {
        In::Msg { room: "r".into(), from: from.into(), data: bytes(m) }
    }

    /// Two sessions wired to each other the way the relay does it.
    struct Pair {
        host: Session,
        guest: Session,
    }

    impl Pair {
        fn new() -> Pair {
            let mut p = Pair { host: Session::host("Marcus", Character::Gumdrop, "rbq-abc"), guest: Session::guest("Davo", Character::Pear, "rbq-abc") };
            // the host connects first and finds an empty room
            p.deliver_host(vec![In::Open { peer: "H".into() }, In::Snap { room: "rbq-abc".into(), peers: vec![] }]);
            // then the guest joins and finds the host
            p.deliver_guest(vec![In::Open { peer: "G".into() }, In::Snap { room: "rbq-abc".into(), peers: vec!["H".into()] }]);
            p
        }

        /// Give the host some events (what it says before a guest is there goes nowhere).
        fn deliver_host(&mut self, evs: Vec<In>) {
            for ev in evs {
                self.host.on_event(ev, 1.0);
            }
        }

        fn to_guest(&mut self, ev: In) {
            for a in self.guest.on_event(ev, 1.0) {
                if let Do::Send { to, msg } = a {
                    if to.as_deref().is_none_or(|t| t == "H") {
                        for b in self.host.on_event(msg_in("G", &msg), 1.0) {
                            if let Do::Send { to, msg } = b {
                                if to.as_deref().is_none_or(|t| t == "G") {
                                    let _ = self.guest.on_event(msg_in("H", &msg), 1.0);
                                }
                            }
                        }
                    }
                }
            }
        }

        fn deliver_guest(&mut self, evs: Vec<In>) {
            for e in evs {
                self.to_guest(e);
            }
        }
    }

    #[test]
    fn a_guest_joins_a_host_and_both_see_each_other() {
        let p = Pair::new();
        assert_eq!(p.guest.my_id, 2);
        assert!(p.guest.joined());
        let names = |s: &Session| s.members.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
        assert_eq!(names(&p.host), vec!["Marcus", "Davo"]);
        assert_eq!(names(&p.guest), vec!["Marcus", "Davo"]);
        assert_eq!(p.host.others().len(), 1);
        assert_eq!(p.host.others()[0].name, "Davo");
        assert_eq!(p.host.others()[0].character, Character::Pear);
        assert_eq!(p.guest.others()[0].name, "Marcus");
        assert_eq!(p.guest.others()[0].net_id, 1);
        assert!(p.host.status.contains("1 mate here"), "{}", p.host.status);
        assert!(p.guest.status.contains("1 other here"), "{}", p.guest.status);
    }

    #[test]
    fn a_guest_who_arrives_first_asks_again_when_the_host_turns_up() {
        let mut g = Session::guest("Davo", Character::Pear, "rbq-abc");
        let out = g.on_event(In::Snap { room: "rbq-abc".into(), peers: vec![] }, 0.0);
        assert_eq!(out.len(), 1, "asks everybody, nobody answers");
        let out = g.on_event(In::Up { room: "rbq-abc".into(), peer: "H".into() }, 0.0);
        assert!(matches!(&out[..], [Do::Send { to: Some(t), msg: Msg::Hello { .. } }] if t == "H"));
        // once welcomed it stops asking
        g.on_event(msg_in("H", &Msg::Welcome { id: 2, members: vec![] }), 0.0);
        assert!(g.on_event(In::Up { room: "r".into(), peer: "X".into() }, 0.0).is_empty());
    }

    #[test]
    fn positions_only_count_from_the_person_they_belong_to() {
        let mut p = Pair::new();
        let mut s = PlayerState::new(2);
        s.pos = (3.0, 0.0, 4.0);
        // the host hears the guest (peer "G" is id 2)
        p.host.on_event(msg_in("G", &Msg::State(s)), 5.0);
        assert_eq!(p.host.heard[&2].state.pos, (3.0, 0.0, 4.0));
        assert_eq!(p.host.heard[&2].at, 5.0);
        // a guest pretending to be the host (id 1) is ignored, and so is a stranger
        let mut fake = PlayerState::new(1);
        fake.pos = (9.0, 0.0, 9.0);
        p.host.on_event(msg_in("G", &Msg::State(fake)), 6.0);
        assert!(!p.host.heard.contains_key(&1));
        p.host.on_event(msg_in("Z", &Msg::State(s)), 7.0);
        assert_eq!(p.host.heard[&2].at, 5.0);
        // the guest hears the host, but never itself
        p.guest.on_event(msg_in("H", &Msg::State(PlayerState::new(1))), 8.0);
        assert!(p.guest.heard.contains_key(&1));
        p.guest.on_event(msg_in("H", &Msg::State(PlayerState::new(2))), 8.0);
        assert!(!p.guest.heard.contains_key(&2));
        p.guest.on_event(msg_in("H", &Msg::State(PlayerState::new(77))), 8.0);
        assert!(!p.guest.heard.contains_key(&77), "not in the yard");
    }

    #[test]
    fn a_guest_leaving_is_dropped_and_everyone_is_told() {
        let mut p = Pair::new();
        let out = p.host.on_event(In::Left { room: "r".into(), peer: "G".into() }, 1.0);
        assert!(matches!(&out[..], [Do::Send { to: None, msg: Msg::Roster(ms) }] if ms.len() == 1));
        assert_eq!(p.host.members.len(), 1);
        assert!(p.host.others().is_empty());
        assert!(p.host.status.contains("waiting for mates"));
        // an unknown peer leaving changes nothing
        assert!(p.host.on_event(In::Left { room: "r".into(), peer: "Z".into() }, 1.0).is_empty());
    }

    #[test]
    fn when_the_host_leaves_the_yard_ends_for_guests() {
        let mut p = Pair::new();
        // somebody else leaving does not end it
        assert!(p.guest.on_event(In::Left { room: "r".into(), peer: "X".into() }, 1.0).is_empty());
        assert!(!p.guest.ended);
        let out = p.guest.on_event(In::Left { room: "r".into(), peer: "H".into() }, 1.0);
        assert_eq!(out, vec![Do::Disconnect]);
        assert!(p.guest.ended && !p.guest.joined());
        assert!(p.guest.status.contains("host left"));
    }

    #[test]
    fn the_host_says_no_politely() {
        let mut h = Session::host("Marcus", Character::Gumdrop, "rbq-abc");
        h.on_event(In::Snap { room: "r".into(), peers: vec![] }, 0.0);
        let hello = |protocol| Msg::Hello { protocol, name: "Kev".into(), character: 0 };
        let out = h.on_event(msg_in("K", &hello(PROTOCOL + 5)), 0.0);
        assert_eq!(out, vec![Do::Send { to: Some("K".into()), msg: Msg::Refused(Refusal::WrongVersion) }]);
        h.round_on = true;
        let out = h.on_event(msg_in("K", &hello(PROTOCOL)), 0.0);
        assert_eq!(out, vec![Do::Send { to: Some("K".into()), msg: Msg::Refused(Refusal::InProgress) }]);
        assert_eq!(h.members.len(), 1);
        // and the guest who is refused gives up with a reason
        let mut g = Session::guest("Kev", Character::Egg, "rbq-abc");
        let out = g.on_event(msg_in("H", &Msg::Refused(Refusal::Full)), 0.0);
        assert_eq!(out, vec![Do::Disconnect]);
        assert!(g.status.contains("full"));
    }

    #[test]
    fn saying_hello_twice_does_not_make_two_people() {
        let mut h = Session::host("Marcus", Character::Gumdrop, "rbq-abc");
        h.on_event(In::Snap { room: "r".into(), peers: vec![] }, 0.0);
        let hello = Msg::Hello { protocol: PROTOCOL, name: "Davo".into(), character: 1 };
        h.on_event(msg_in("G", &hello), 0.0);
        let out = h.on_event(msg_in("G", &hello), 0.0);
        assert_eq!(h.members.len(), 2, "still one guest");
        assert!(matches!(&out[..], [Do::Send { to: Some(t), msg: Msg::Welcome { id: 2, .. } }] if t == "G"));
    }

    #[test]
    fn a_host_who_arrives_after_a_mate_still_hosts_and_takes_their_hello() {
        let mut h = Session::host("Marcus", Character::Gumdrop, "rbq-abc");
        let out = h.on_event(In::Snap { room: "rbq-abc".into(), peers: vec!["early-bird".into()] }, 0.0);
        assert!(out.is_empty());
        assert!(!h.ended && h.status.contains("Hosting"));
        // the early guest says hello when it sees the host arrive
        let out = h.on_event(msg_in("early-bird", &Msg::Hello { protocol: PROTOCOL, name: "Davo".into(), character: 1 }), 0.0);
        assert!(matches!(&out[0], Do::Send { to: Some(t), msg: Msg::Welcome { id: 2, .. } } if t == "early-bird"));
    }

    #[test]
    fn rubbish_from_the_network_changes_nothing() {
        let mut p = Pair::new();
        let before = p.host.members.clone();
        for junk in [vec![], vec![200], vec![5, 1, 2], vec![2, 255, 255, 255, 255, 9]] {
            p.host.on_event(In::Msg { room: "r".into(), from: "G".into(), data: junk.clone() }, 0.0);
            p.guest.on_event(In::Msg { room: "r".into(), from: "H".into(), data: junk }, 0.0);
        }
        assert_eq!(p.host.members, before);
        assert!(p.guest.joined());
        // a host does not obey Welcome or Roster messages, and a guest ignores Hello
        p.host.on_event(msg_in("G", &Msg::Roster(vec![])), 0.0);
        assert_eq!(p.host.members, before);
        assert!(p.guest.on_event(msg_in("G", &Msg::Hello { protocol: PROTOCOL, name: "x".into(), character: 0 }), 0.0).is_empty());
        // a roster from somebody who is not the host is ignored
        p.guest.on_event(msg_in("Z", &Msg::Roster(vec![])), 0.0);
        assert_eq!(p.guest.members.len(), 2);
    }

    #[test]
    fn a_guest_keeps_only_newer_world_pictures_from_the_host() {
        use bbq_core::net_world::*;
        let mut p = Pair::new();
        let snap = |seq| WorldSnap {
            seq,
            now: 1.0,
            round: RoundSnap { phase: 3, mode: 0, features: 0, time_left: 9.0, round_no: 1, match_len: 1, chest_spot: 0, chest_stock: 3, smoko_at: (0.0, 0.0), heist_teams: 0, banner: None },
            items: vec![],
            bots: vec![],
            dazza: DazzaSnap { x: 0.0, z: 0.0, face: 0.0, state: 0, say_seq: 0, swing_seq: 0, flip_seq: 0, say: String::new() },
            board: vec![],
            teams: vec![],
            feed: vec![],
            fx: vec![],
        };
        p.guest.on_event(msg_in("H", &Msg::World(Box::new(snap(5)))), 0.0);
        assert_eq!(p.guest.world.as_ref().unwrap().seq, 5);
        p.guest.on_event(msg_in("H", &Msg::World(Box::new(snap(4)))), 0.0);
        assert_eq!(p.guest.world.as_ref().unwrap().seq, 5, "an old picture is ignored");
        p.guest.on_event(msg_in("H", &Msg::World(Box::new(snap(6)))), 0.0);
        assert_eq!(p.guest.world.as_ref().unwrap().seq, 6);
        // somebody who is not the host cannot describe the yard
        p.guest.on_event(msg_in("Z", &Msg::World(Box::new(snap(99)))), 0.0);
        assert_eq!(p.guest.world.as_ref().unwrap().seq, 6);
        // and a host does not take pictures from anybody
        p.host.on_event(msg_in("G", &Msg::World(Box::new(snap(50)))), 0.0);
        assert!(p.host.world.is_none());
        assert_eq!(p.guest.host_name(), "Marcus");
    }

    #[test]
    fn a_guest_goes_where_the_host_goes() {
        use crate::menu::{Request, Screen};
        // the host starts a round: a guest waiting in the menu, or looking at the last results, joins
        assert_eq!(follow_decision(Screen::Menu, true, true), Some(Request::Follow));
        assert_eq!(follow_decision(Screen::Results, true, true), Some(Request::Follow));
        // already playing: nothing to do
        assert_eq!(follow_decision(Screen::Playing, true, true), None);
        // the whistle: the host's round is over but still "timed": the guest stays for the results
        assert_eq!(follow_decision(Screen::Playing, false, true), None);
        // the host went back to its menu: the guest does too
        assert_eq!(follow_decision(Screen::Playing, false, false), Some(Request::Menu));
        assert_eq!(follow_decision(Screen::Paused, false, false), Some(Request::Menu));
        assert_eq!(follow_decision(Screen::Menu, false, false), None);
    }

    #[test]
    fn room_codes_are_readable_and_allowed() {
        for seed in 0..50u64 {
            let c = new_room_code(seed);
            assert!(c.len() == 8 && c.starts_with("rbq-"), "{c}");
            assert!(c.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-'));
        }
        assert_ne!(new_room_code(1), new_room_code(2));
        assert_eq!(clean_room("  RBQ-Ab_c! "), "rbq-abc");
        assert_eq!(clean_room(&"x".repeat(60)).len(), 24);
    }

    #[test]
    fn character_codes_round_trip() {
        for c in Character::ALL {
            assert_eq!(code_char(char_code(c)), c);
        }
        assert_eq!(code_char(200), *Character::ALL.last().unwrap());
    }
}
