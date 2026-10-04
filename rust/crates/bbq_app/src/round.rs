//! Phase 7: rounds, matches and game modes. Free for all and Teams live here; the Teddy Heist
//! rules are in `heist_app.rs`. There is no menu yet (that is Phase 8), so the setup is changed
//! with keys and shown on screen.
//!
//! The yard starts as a free-play practice yard with no clock. Press Enter to start a real
//! round: a 3.2 s countdown, the round clock, the whistle, then the results.

use bbq_core::matchflow::{self, Key, Match};
use bbq_core::movement::Mover;
use bbq_core::rng::Rng;
use bbq_core::scoring::{Phase, Rules, Scoreboard};
use bbq_core::sim::{COUNTDOWN, PANEL_DELAY, TICK_DT};
use bbq_core::stun::Body;
use bbq_core::teams::Team;
use bbq_core::vec::V3;
use bbq_core::yard::{CHAR_SPAWNS, CHEST_SPOTS, Yard};
use bbq_core::{GameMode, PlayerId, heist};
use bevy::prelude::*;

use crate::characters::{BLOB_NAMES, MAX_BOTS};
use crate::game::{Dummy, Game};
use crate::player::{PLAYER_ID, Player};
use crate::yard_scene::YardRes;

/// Round lengths on offer, in seconds.
pub const ROUND_LENGTHS: [f32; 3] = [60.0, 180.0, 300.0];

/// What the next round will be like.
#[derive(Clone, Debug)]
pub struct Setup {
    pub mode: GameMode,
    /// Heist: how many teams (2 to 4).
    pub heist_teams: usize,
    pub bots: usize,
    pub round_len: f32,
    /// Rounds in the match: 1, 3 or 5.
    pub match_len: u32,
    pub friendly_fire: bool,
}

impl Default for Setup {
    fn default() -> Self {
        Setup {
            mode: GameMode::FreeForAll,
            heist_teams: 2,
            bots: 3,
            round_len: 180.0,
            match_len: 1,
            friendly_fire: false,
        }
    }
}

impl Setup {
    /// A short name for the mode ("Teddy Heist (3 teams)").
    pub fn mode_name(&self) -> String {
        match self.mode {
            GameMode::FreeForAll => "Free for all".to_string(),
            GameMode::Teams => "Teams".to_string(),
            GameMode::Heist => format!("Teddy Heist ({} teams)", self.heist_teams),
        }
    }

    /// Step through the modes: Free for all, Teams, then Heist with 2, 3 and 4 teams.
    pub fn next_mode(&mut self) {
        match (self.mode, self.heist_teams) {
            (GameMode::FreeForAll, _) => self.mode = GameMode::Teams,
            (GameMode::Teams, _) => {
                self.mode = GameMode::Heist;
                self.heist_teams = 2;
            }
            (GameMode::Heist, n) if n < 4 => self.heist_teams = n + 1,
            (GameMode::Heist, _) => self.mode = GameMode::FreeForAll,
        }
        if self.mode == GameMode::Heist {
            self.friendly_fire = false;
        }
    }
}

/// What the round ended with, for the results panel.
#[derive(Clone, Debug)]
pub struct Results {
    pub lines: Vec<String>,
    pub winner: Option<Key>,
    pub match_over: bool,
}

/// Where the round is up to.
pub struct RoundCtl {
    pub setup: Setup,
    pub mtch: Match,
    /// A real round is running (false in the practice yard at startup).
    pub timed: bool,
    pub time_left: f32,
    pub banner: Option<(String, f32)>,
    pub results: Option<Results>,
    /// The results panel opens this long after the whistle.
    pub panel_in: f32,
    pub last_beep: i32,
    pub rng: Rng,
}

impl RoundCtl {
    pub fn new() -> Self {
        RoundCtl {
            setup: Setup::default(),
            mtch: Match::new(1),
            timed: false,
            time_left: 0.0,
            banner: None,
            results: None,
            panel_in: 0.0,
            last_beep: 0,
            rng: Rng::new(0x20_0D),
        }
    }

    pub fn say_banner(&mut self, text: impl Into<String>, secs: f32) {
        self.banner = Some((text.into(), secs));
    }
}

impl Default for RoundCtl {
    fn default() -> Self {
        Self::new()
    }
}

/// Who is in the yard: you, then the bots.
pub fn everyone(g: &Game) -> Vec<PlayerId> {
    let mut v = vec![PLAYER_ID];
    v.extend(g.dummies.iter().map(|d| d.id));
    v
}

pub fn name_of(g: &Game, id: PlayerId) -> String {
    if id == PLAYER_ID {
        "You".to_string()
    } else {
        crate::bots_app::name_of(g, id)
    }
}

/// Keep exactly `want` bots in the yard.
pub fn sync_bot_count(g: &mut Game, want: usize) {
    let want = want.min(MAX_BOTS);
    while g.dummies.len() > want {
        if let Some(d) = g.dummies.pop() {
            g.crowd.brains.remove(&d.id);
            g.board.remove(d.id);
        }
    }
    let mut i = g.dummies.len();
    while g.dummies.len() < want {
        let id = 100 + i as u32;
        let (x, z) = CHAR_SPAWNS[(i + 1) % CHAR_SPAWNS.len()];
        let d = Dummy::new(id, x, z, i, &mut g.rng);
        g.board.ensure(id);
        g.crowd.add(id, &mut g.rng);
        g.dummies.push(d);
        i += 1;
    }
    g.life.grab_immune.resize(g.dummies.len(), 0.0);
}

/// Put people on teams for the coming round.
pub fn assign_teams(g: &mut Game, reshuffle: bool) {
    let ids = everyone(g);
    match g.round.setup.mode {
        GameMode::FreeForAll => g.teams.clear(),
        GameMode::Teams => {
            let mut rng = Rng::new(g.rng.f32().to_bits() as u64 + 77);
            g.teams.balance(&ids, reshuffle, false, &mut rng);
        }
        GameMode::Heist => {
            let keys = heist::team_keys(g.round.setup.heist_teams);
            let mut rng = Rng::new(g.rng.f32().to_bits() as u64 + 99);
            g.teams.assign_heist(&ids, keys, reshuffle, &mut rng);
        }
    }
    show_teams(g);
}

/// Copy teams onto the blobs (their sashes).
pub fn show_teams(g: &mut Game) {
    let teamed = g.rules.mode != GameMode::FreeForAll;
    for i in 0..g.dummies.len() {
        let id = g.dummies[i].id;
        g.dummies[i].team = if teamed { g.teams.get(id) } else { None };
    }
}

/// Begin a round (and a fresh match if the last one finished).
pub fn start_round(g: &mut Game, p: &mut Player, yard: &mut YardRes) {
    let setup = g.round.setup.clone();
    if g.round.mtch.over || g.round.mtch.no == 0 {
        g.round.mtch = Match::new(if setup.mode == GameMode::Heist {
            1
        } else {
            setup.match_len
        });
    }
    g.round.mtch.start_round();
    g.round.timed = true;
    g.round.results = None;
    g.rules = Rules {
        mode: setup.mode,
        friendly_fire: setup.mode != GameMode::Heist && setup.friendly_fire,
        phase: Phase::Countdown,
    };

    // a clean yard and the right number of bots
    g.world.clear();
    sync_bot_count(g, setup.bots);
    assign_teams(g, false);
    g.board = Scoreboard::new();
    for id in everyone(g) {
        g.board.ensure(id);
    }

    // the heist arena (or the plain yard)
    let features = yard.0.features;
    let spot = pick_chest_spot(g, setup.mode, setup.heist_teams);
    yard.0 = if setup.mode == GameMode::Heist {
        Yard::heist(features, spot, setup.heist_teams)
    } else {
        Yard::new(features, spot)
    };
    g.life.chest.new_round(spot);
    g.life.dazza.reset();
    g.life.carry = None;
    g.life.f_down = None;
    g.life.seated = None;
    g.life.grab_immune.clear();
    g.life.grab_immune.resize(g.dummies.len(), 0.0);

    // everyone back to a sober start, in a shuffled spot
    let mut spots: Vec<(f32, f32)> = CHAR_SPAWNS.to_vec();
    g.rng.shuffle(&mut spots);
    let mut next = 0;
    let mut team_spawned: std::collections::BTreeMap<Team, usize> = Default::default();
    let mut place = |g: &mut Game, id: PlayerId| -> (f32, f32) {
        if setup.mode == GameMode::Heist
            && let Some(t) = g.teams.get(id)
            && let Some(at) = heist::spawn_in_base(setup.heist_teams, t, {
                let n = team_spawned.entry(t).or_insert(0);
                *n += 1;
                *n - 1
            })
        {
            return at;
        }
        let s = spots[next % spots.len()];
        next += 1;
        s
    };
    let (px, pz) = place(g, PLAYER_ID);
    *p = Player {
        mover: Mover::new(px, pz),
        prev: Vec3::new(px, 0.0, pz),
        yaw: px.atan2(pz),
        pitch: 0.0,
        walk: 0.0,
        shake: 0.0,
        fov_base: p.fov_base,
        fov: p.fov,
        rng: Rng::new(g.rng.f32().to_bits() as u64),
    };
    g.me.body = Body::default();
    g.me.drunk.reset();
    g.me.help = Default::default();
    g.slots.clear();
    g.wind.cancel();
    g.catcher = Default::default();
    for i in 0..g.dummies.len() {
        let id = g.dummies[i].id;
        let (x, z) = place(g, id);
        let mut fresh = Dummy::new(id, x, z, i, &mut g.rng);
        fresh.face = x.atan2(z) + std::f32::consts::PI;
        g.dummies[i] = fresh;
        if let Some(b) = g.crowd.brains.get_mut(&id) {
            b.reset(&mut g.rng);
        }
    }
    g.crowd.claims.clear();
    show_teams(g);

    // things to throw
    let players = g.dummies.len() + 1;
    let y = yard.0.clone();
    g.world
        .initial_fill(players, &|x, z| y.heist_clear(x, z, 1.2), &mut g.rng);

    if setup.mode == GameMode::Heist {
        crate::heist_app::start(g, &y);
    } else {
        g.heist = None;
    }

    g.round.time_left = COUNTDOWN;
    g.round.last_beep = 4;
    let n = g.round.mtch.clone();
    let head = if n.len > 1 {
        format!("ROUND {} OF {}", n.no, n.len)
    } else {
        "GET READY".to_string()
    };
    g.round.say_banner(head, 1.6);
    g.say(format!(
        "Round {}: {} with {} bot{}",
        g.round.mtch.no,
        setup.mode_name(),
        setup.bots,
        if setup.bots == 1 { "" } else { "s" }
    ));
}

/// A chest spot that is not where it is now (and, in Heist, not inside a wall).
fn pick_chest_spot(g: &mut Game, mode: GameMode, heist_teams: usize) -> usize {
    let probe = Yard::heist(Default::default(), 0, heist_teams);
    let n = CHEST_SPOTS.len();
    let cur = g.life.chest.spot;
    for _ in 0..40 {
        let i = g.rng.index(n);
        if i == cur % n {
            continue;
        }
        let (x, z, _) = CHEST_SPOTS[i];
        if mode != GameMode::Heist || probe.heist_clear(x, z, 1.4) {
            return i;
        }
    }
    (cur + 1) % n
}

/// The standings, best first.
fn standings(g: &Game) -> Vec<(PlayerId, i32)> {
    let mut v: Vec<(PlayerId, i32)> = g.board.iter().map(|(id, s)| (id, s.score)).collect();
    v.sort_by_key(|x| std::cmp::Reverse(x.1));
    v
}

fn key_name(g: &Game, k: Key) -> String {
    match k {
        Key::Team(t) => format!("{} team", t.name()),
        Key::Player(id) => name_of(g, id),
    }
}

/// The whistle: work out who won and open the results.
pub fn end_round(g: &mut Game) {
    g.rules.phase = Phase::Results;
    let rows = standings(g);
    let winner = match g.rules.mode {
        GameMode::FreeForAll => matchflow::round_winner_ffa(&rows),
        GameMode::Teams => matchflow::round_winner_teams(&rows, &g.teams),
        GameMode::Heist => g
            .heist
            .as_ref()
            .and_then(|h| matchflow::round_winner_heist(&h.bank)),
    };
    g.round.mtch.end_round(winner);
    let mut lines = Vec::new();
    lines.push(match winner {
        Some(k) => format!("{} wins the round!", key_name(g, k)),
        None => "A draw: nobody wins that round".to_string(),
    });
    if g.rules.mode == GameMode::Teams {
        let totals = g.board.team_totals(&g.teams);
        let mut t: Vec<_> = totals.into_iter().collect();
        t.sort_by_key(|x| std::cmp::Reverse(x.1));
        lines.push(
            t.iter()
                .map(|(team, pts)| format!("{} {pts}", team.name()))
                .collect::<Vec<_>>()
                .join("   "),
        );
    }
    if let Some(h) = &g.heist {
        lines.push(
            heist::team_keys(h.teams)
                .iter()
                .map(|t| {
                    format!(
                        "{} {} teddies ({} pts)",
                        t.name(),
                        h.bank.get(*t),
                        h.bank.points(*t)
                    )
                })
                .collect::<Vec<_>>()
                .join("   "),
        );
    }
    for (rank, (id, score)) in rows.iter().enumerate() {
        let s = g.board.get(*id).cloned().unwrap_or_default();
        let team = g
            .teams
            .get(*id)
            .filter(|_| g.rules.mode != GameMode::FreeForAll)
            .map_or(String::new(), |t| format!(" [{}]", t.name()));
        lines.push(format!(
            "{}. {}{team}: {score} (hits {}, taken {}, catches {}{})",
            rank + 1,
            name_of(g, *id),
            s.hits,
            s.taken,
            s.catches,
            if g.rules.mode == GameMode::Heist {
                format!(", banked {}", s.banked)
            } else {
                String::new()
            }
        ));
    }
    let m = &g.round.mtch;
    let match_over = m.over;
    if m.len > 1 {
        let wins: Vec<String> = m
            .wins
            .iter()
            .map(|(k, v)| format!("{} {v}", key_name(g, *k)))
            .collect();
        lines.push(format!(
            "Round wins (first to {}): {}",
            m.need(),
            if wins.is_empty() {
                "none yet".to_string()
            } else {
                wins.join(", ")
            }
        ));
    }
    if match_over {
        lines.push(match g.round.mtch.leader() {
            Some(k) if g.round.mtch.len > 1 => format!("{} takes the match!", key_name(g, k)),
            _ => "Match over".to_string(),
        });
        lines.push("Enter: play again".to_string());
    } else {
        lines.push("Enter: next round".to_string());
    }
    g.round.results = Some(Results {
        lines,
        winner,
        match_over,
    });
    g.round.say_banner("TIME!", 1.4);
    g.round.panel_in = PANEL_DELAY;
}

/// The round clock. Runs every fixed step, before everything else.
pub fn step(g: &mut Game) {
    show_teams(g);
    let dt = TICK_DT;
    if let Some((_, t)) = g.round.banner.as_mut() {
        *t -= dt;
        if *t <= 0.0 {
            g.round.banner = None;
        }
    }
    if !g.round.timed {
        return;
    }
    match g.rules.phase {
        Phase::Countdown => {
            g.round.time_left -= dt;
            let n = g.round.time_left.ceil() as i32;
            if n != g.round.last_beep && (1..=3).contains(&n) {
                g.round.last_beep = n;
                g.round.say_banner(n.to_string(), 0.9);
            }
            if g.round.time_left <= 0.0 {
                g.rules.phase = Phase::Play;
                g.round.time_left = g.round.setup.round_len;
                g.round.say_banner("CHUCK IT!", 1.0);
            }
        }
        Phase::Play => {
            g.round.time_left = (g.round.time_left - dt).max(0.0);
            if g.round.time_left <= 0.0 {
                end_round(g);
            }
        }
        Phase::Results => {
            if g.round.panel_in > 0.0 {
                g.round.panel_in -= dt;
            }
        }
        Phase::Menu | Phase::Warmup => {}
    }
}

/// `m:ss`
pub fn clock_text(secs: f32) -> String {
    let s = secs.max(0.0).ceil() as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

// ------------------------------------------------------------------ keys and the screen

pub struct RoundPlugin;

impl Plugin for RoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (setup_round_hud, start_from_args).chain())
            .add_systems(Update, (round_keys, update_round_hud));
    }
}

/// `--mode heist|teams|ffa` starts a round straight away (handy for testing and screenshots).
fn start_from_args(mut game: ResMut<Game>, mut player: ResMut<Player>, mut yard: ResMut<YardRes>) {
    let args: Vec<String> = std::env::args().collect();
    let Some(mode) = args
        .iter()
        .position(|a| a == "--mode")
        .and_then(|i| args.get(i + 1))
    else {
        return;
    };
    if args.iter().any(|a| a == "--cheeky") {
        game.options.adult = true;
    }
    let s = &mut game.round.setup;
    s.mode = match mode.as_str() {
        "heist" => GameMode::Heist,
        "teams" => GameMode::Teams,
        _ => GameMode::FreeForAll,
    };
    start_round(&mut game, &mut player, &mut yard);
    // `--give noodle|dildo|steak|fish|teddy|stubby|gnome`: start with one in your hand (testing)
    if let Some(k) = args.iter().position(|a| a == "--give").and_then(|i| args.get(i + 1)) {
        let kind = match k.as_str() {
            "noodle" => Some(bbq_core::items::ItemKind::Noodle),
            "dildo" => Some(bbq_core::items::ItemKind::Dildo),
            "steak" => Some(bbq_core::items::ItemKind::Steak),
            "fish" => Some(bbq_core::items::ItemKind::Fish),
            "teddy" => Some(bbq_core::items::ItemKind::Teddy),
            "stubby" => Some(bbq_core::items::ItemKind::Stubby),
            "gnome" => Some(bbq_core::items::ItemKind::Gnome),
            _ => None,
        };
        if let Some(kind) = kind {
            let (x, z) = (player.mover.x, player.mover.z);
            let mut rng = game.rng.clone();
            let id = game.world.spawn(kind, x, z, false, &mut rng);
            game.world.give(id, crate::player::PLAYER_ID);
            game.slots.add(id);
        }
    }
    if let Some(n) = args
        .iter()
        .position(|a| a == "--chest")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse::<usize>().ok())
    {
        let feats = yard.0.features;
        yard.0 = bbq_core::yard::Yard::new(feats, n);
        game.life.chest.new_round(n);
    }
}

#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct BannerText;
#[derive(Component)]
struct ResultsText;

fn setup_round_hud(mut commands: Commands) {
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(12.0),
            top: Val::Px(8.0),
            ..default()
        },
        StatusText,
        crate::menu::DevHud,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(72.0),
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.95, 0.4)),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            top: Val::Percent(24.0),
            justify_content: JustifyContent::Center,
            ..default()
        },
        TextLayout::justify(Justify::Center),
        BannerText,
        Visibility::Hidden,
    ));
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        TextColor(Color::WHITE),
        BackgroundColor(Color::srgba(0.05, 0.1, 0.15, 0.82)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(22.0),
            width: Val::Percent(56.0),
            top: Val::Percent(14.0),
            padding: UiRect::all(Val::Px(18.0)),
            ..default()
        },
        Visibility::Hidden,
        ResultsText,
    ));
}

fn round_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut player: ResMut<Player>,
    mut yard: ResMut<YardRes>,
) {
    let g = &mut *game;
    let busy = g.round.timed && matches!(g.rules.phase, Phase::Countdown | Phase::Play);
    if keys.just_pressed(KeyCode::Enter) {
        // Enter starts a round from the practice yard or from the results, never mid-round
        if !g.round.timed || g.rules.phase == Phase::Results {
            start_round(g, &mut player, &mut yard);
        }
        return;
    }
    if busy {
        return;
    }
    let s = &mut g.round.setup;
    let mut changed = false;
    if keys.just_pressed(KeyCode::F12) {
        s.next_mode();
        changed = true;
    }
    if keys.just_pressed(KeyCode::Equal) && s.bots < MAX_BOTS {
        s.bots += 1;
        changed = true;
    }
    if keys.just_pressed(KeyCode::Minus) && s.bots > 0 {
        s.bots -= 1;
        changed = true;
    }
    if keys.just_pressed(KeyCode::Backslash) && s.mode == GameMode::Teams {
        s.friendly_fire = !s.friendly_fire;
        changed = true;
    }
    if keys.just_pressed(KeyCode::Semicolon) {
        s.match_len = match s.match_len {
            1 => 3,
            3 => 5,
            _ => 1,
        };
        changed = true;
    }
    if keys.just_pressed(KeyCode::Quote) {
        let i = ROUND_LENGTHS
            .iter()
            .position(|l| *l == s.round_len)
            .unwrap_or(0);
        s.round_len = ROUND_LENGTHS[(i + 1) % ROUND_LENGTHS.len()];
        changed = true;
    }
    if changed {
        // the setup shows at once: bots appear or go, teams are dealt
        let want = g.round.setup.bots;
        let mode = g.round.setup.mode;
        if !g.round.timed || g.rules.phase == Phase::Results {
            sync_bot_count(g, want);
            g.rules.mode = mode;
            assign_teams(g, false);
        }
    }
}

fn update_round_hud(
    game: Res<Game>,
    mut status: Single<&mut Text, (With<StatusText>, Without<BannerText>, Without<ResultsText>)>,
    mut banner: Single<&mut Text, (With<BannerText>, Without<StatusText>, Without<ResultsText>)>,
    mut results: Single<
        (&mut Text, &mut Visibility),
        (With<ResultsText>, Without<StatusText>, Without<BannerText>),
    >,
) {
    let g = &*game;
    // top right: the setup and the clock
    let s = &g.round.setup;
    let mut lines = Vec::new();
    if g.round.timed {
        let phase = match g.rules.phase {
            Phase::Countdown => "get ready".to_string(),
            Phase::Play => clock_text(g.round.time_left),
            Phase::Results => "time!".to_string(),
            _ => String::new(),
        };
        let m = &g.round.mtch;
        let round = if m.len > 1 {
            format!("Round {}/{} | ", m.no, m.len)
        } else {
            String::new()
        };
        lines.push(format!("{round}{} | {phase}", s.mode_name()));
    } else {
        lines.push("Practice yard (no clock)".to_string());
    }
    if g.rules.mode == GameMode::Teams && g.round.timed {
        let mut t: Vec<_> = g.board.team_totals(&g.teams).into_iter().collect();
        t.sort_by_key(|x| std::cmp::Reverse(x.1));
        lines.push(
            t.iter()
                .map(|(team, pts)| format!("{} {pts}", team.name()))
                .collect::<Vec<_>>()
                .join(" | "),
        );
    }
    if let Some(h) = &g.heist {
        lines.push(crate::heist_app::tally_text(h));
    }
    if let Some(t) = g
        .teams
        .get(PLAYER_ID)
        .filter(|_| g.rules.mode != GameMode::FreeForAll)
    {
        lines.push(format!("You are on the {} team", t.name()));
    }
    let ready = !g.round.timed || g.rules.phase == Phase::Results;
    if ready {
        lines.push(format!(
            "Enter: start | F12 mode | -/= bots ({}) | ; match {} | ' round {}{}",
            s.bots,
            if s.match_len == 1 {
                "single".to_string()
            } else {
                format!("best of {}", s.match_len)
            },
            clock_text(s.round_len),
            if s.mode == GameMode::Teams {
                format!(
                    " | \\ friendly fire ({})",
                    if s.friendly_fire { "on" } else { "off" }
                )
            } else {
                String::new()
            }
        ));
    }
    status.0 = lines.join("\n");

    banner.0 = g
        .round
        .banner
        .as_ref()
        .map(|b| b.0.clone())
        .unwrap_or_default();

    // the results card (`menu.rs`) shows the results now
    let (_, vis) = &mut *results;
    **vis = Visibility::Hidden;
}

#[allow(dead_code)]
fn unused(_: V3, _: &[&str]) {
    let _ = BLOB_NAMES;
    let _ = TICK_DT;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_text_counts_down_in_minutes_and_seconds() {
        assert_eq!(clock_text(180.0), "3:00");
        assert_eq!(clock_text(59.2), "1:00");
        assert_eq!(clock_text(9.0), "0:09");
        assert_eq!(clock_text(-4.0), "0:00");
    }

    #[test]
    fn the_mode_key_goes_round_all_five_modes() {
        let mut s = Setup::default();
        let mut seen = vec![s.mode_name()];
        for _ in 0..5 {
            s.next_mode();
            seen.push(s.mode_name());
        }
        assert_eq!(
            seen,
            [
                "Free for all",
                "Teams",
                "Teddy Heist (2 teams)",
                "Teddy Heist (3 teams)",
                "Teddy Heist (4 teams)",
                "Free for all"
            ]
        );
    }

    #[test]
    fn heist_has_no_friendly_fire() {
        let mut s = Setup {
            friendly_fire: true,
            mode: GameMode::Teams,
            ..Setup::default()
        };
        s.next_mode();
        assert_eq!(s.mode, GameMode::Heist);
        assert!(!s.friendly_fire);
    }
}
