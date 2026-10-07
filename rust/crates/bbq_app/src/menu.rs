//! Phase 8: the screens around the game, as close to the browser page as the engine allows:
//! the main menu (Solo / With mates / How to play), the pause card ("Smoko") and the results
//! card, plus the flow between them and the settings that are saved on this computer.
//!
//! The yard behind the menu is the browser's "attract mode": three bots wander and chuck things
//! while the camera drifts round the yard. Play starts a round with the chosen options.
//!
//! Left out on purpose: the emoji on the browser's labels (the engine can't draw colour emoji
//! yet), the touch controls, and the online "With mates" tab, which waits for Phase 9.

use bbq_core::GameMode;
use bbq_core::bots::Difficulty;
use bbq_core::character::Character;
use bbq_core::sim::TICK_DT;
use bbq_core::teams::Team;
use bbq_core::yard::Features;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

use crate::characters::Cast;
use crate::game::Game;
use crate::player::{PLAYER_ID, Player};
use crate::round::{Setup, start_round};
use crate::ui::*;
use crate::yard_scene::YardRes;

/// Which screen is showing.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Menu,
    Playing,
    Paused,
    Results,
}

/// Everything the menus choose. Saved between runs.
#[derive(Resource, Clone, Debug)]
pub struct Settings {
    pub name: String,
    pub fov: f32,
    pub bots: usize,
    pub mode: GameMode,
    pub heist_teams: usize,
    pub cheeky: bool,
    pub round_len: f32,
    pub skill: Difficulty,
    pub rounds: u32,
    pub friendly_fire: bool,
    pub features: Features,
    pub naughty: bool,
    pub falls: bool,
    pub drunk_all: bool,
    pub character: Character,
    /// How big your beer belly is: 0 none, 1 normal, 2 enormous (looks only).
    pub belly: f32,
    pub sound: bool,
    /// Online: the relay's address (`npm start` on the host's computer, or a cloudflared link).
    pub server: String,
    /// Your blob's customised look: mouth, brows, hair and colours.
    pub look: bbq_core::appearance::Appearance,
    /// Online: the room code to host or join (a new one is made when hosting with this empty).
    pub room: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            name: "Mate".into(),
            fov: 85.0,
            bots: 3,
            mode: GameMode::FreeForAll,
            heist_teams: 2,
            cheeky: false,
            round_len: 180.0,
            skill: Difficulty::Fair,
            rounds: 1,
            friendly_fire: false,
            features: Features::default(),
            naughty: false,
            falls: true,
            drunk_all: false,
            character: Character::default(),
            belly: 1.0,
            sound: true,
            server: "localhost:3000".into(),
            look: bbq_core::appearance::Appearance::default(),
            room: String::new(),
        }
    }
}

/// Set for test runs (any of `TEST_FLAGS` on the command line): settings are read but never saved.
pub static NO_SAVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Command-line flags that mean "this is a test run".
pub const TEST_FLAGS: [&str; 14] = [
    "--shot", "--net-host", "--net-join", "--net-play", "--net-ready", "--net-give", "--net-throw", "--server", "--name", "--hair",
    "--mouth", "--brows", "--body", "--belly",
];

impl Settings {
    fn path() -> Option<std::path::PathBuf> {
        let base = if let Ok(h) = std::env::var("HOME") {
            if cfg!(target_os = "macos") {
                std::path::PathBuf::from(h).join("Library/Application Support")
            } else {
                std::env::var("XDG_CONFIG_HOME")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|_| std::path::PathBuf::from(h).join(".config"))
            }
        } else {
            std::path::PathBuf::from(std::env::var("APPDATA").ok()?)
        };
        Some(base.join("AustralianBBQ").join("settings.txt"))
    }

    /// Plain `key=value` lines, so Marcus can read and edit the file by hand.
    pub fn save(&self) {
        // test runs (screenshots, two copies online) never write over Marcus's own settings
        if NO_SAVE.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        let Some(p) = Self::path() else { return };
        let _ = std::fs::create_dir_all(p.parent().unwrap());
        let mode = match self.mode {
            GameMode::FreeForAll => "ffa",
            GameMode::Teams => "teams",
            GameMode::Heist => "heist",
        };
        let skill = match self.skill {
            Difficulty::Easy => "easy",
            Difficulty::Fair => "fair",
            Difficulty::Spicy => "spicy",
        };
        let f = &self.features;
        let s = format!(
            "name={}\nfov={}\nbots={}\nmode={mode}\nheist_teams={}\ncheeky={}\nround_len={}\nskill={skill}\nrounds={}\nfriendly_fire={}\nbar={}\nbbq={}\nchest={}\nsmoko={}\nnaughty={}\nfalls={}\ndrunk_all={}\ncharacter={}\nbelly={}\nsound={}\nserver={}\nroom={}\n",
            self.name, self.fov, self.bots, self.heist_teams, self.cheeky, self.round_len,
            self.rounds, self.friendly_fire, f.bar, f.bbq, f.chest, f.smoko, self.naughty,
            self.falls, self.drunk_all, self.character.name(), self.belly, self.sound,
            self.server, self.room
        ) + &self.look.to_lines();
        let _ = std::fs::write(p, s);
    }

    pub fn load() -> Self {
        let mut s = Settings::default();
        let Some(text) = Self::path().and_then(|p| std::fs::read_to_string(p).ok()) else {
            return s;
        };
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let b = v == "true";
            match k {
                "name" => s.name = v.chars().take(14).collect(),
                "fov" => s.fov = v.parse().unwrap_or(85.0f32).clamp(60.0, 105.0),
                "bots" => s.bots = v.parse().unwrap_or(3usize).min(3),
                "mode" => {
                    s.mode = match v {
                        "teams" => GameMode::Teams,
                        "heist" => GameMode::Heist,
                        _ => GameMode::FreeForAll,
                    }
                }
                "heist_teams" => s.heist_teams = v.parse().unwrap_or(2usize).clamp(2, 4),
                "cheeky" => s.cheeky = b,
                "round_len" => s.round_len = if v.parse::<f32>().unwrap_or(180.0) <= 60.0 { 60.0 } else { 180.0 },
                "skill" => {
                    s.skill = match v {
                        "easy" => Difficulty::Easy,
                        "spicy" => Difficulty::Spicy,
                        _ => Difficulty::Fair,
                    }
                }
                "rounds" => s.rounds = [1, 3, 5].into_iter().find(|r| v.parse() == Ok(*r)).unwrap_or(1),
                "friendly_fire" => s.friendly_fire = b,
                "bar" => s.features.bar = b,
                "bbq" => s.features.bbq = b,
                "chest" => s.features.chest = b,
                "smoko" => s.features.smoko = b,
                "naughty" => s.naughty = b,
                "falls" => s.falls = b,
                "drunk_all" => s.drunk_all = b,
                "character" => {
                    if let Some(c) = Character::ALL.iter().find(|c| c.name() == v) {
                        s.character = *c;
                    }
                }
                "belly" => s.belly = v.parse().unwrap_or(1.0f32).clamp(0.0, 2.0),
                "sound" => s.sound = b,
                "server" => s.server = v.chars().take(80).collect(),
                "room" => s.room = crate::online::clean_room(v),
                k => {
                    s.look.read_line(k, v);
                }
            }
        }
        s
    }
}

/// What the menu is doing right now (not saved).
#[derive(Resource)]
pub struct MenuUi {
    pub tab: Tab,
    pub more_open: bool,
    pub name_focus: bool,
    pub request: Option<Request>,
    /// Online: which box is being typed in (server address or room code).
    pub net_focus: Option<NetField>,
    /// Online: Host, Join or Leave was pressed (`online.rs` acts on it).
    pub net_request: Option<crate::online::NetRequest>,
    /// Lobby: the Ready button was pressed (`online.rs` toggles it).
    pub toggle_ready: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetField {
    Server,
    Room,
}

#[derive(Component)]
struct NetFieldText(NetField);

#[derive(Component)]
struct NetStatusText;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Request {
    Play,
    Again,
    Menu,
    Resume,
    /// Online guest: the host started a round, so join it (the guest's own Play button does not
    /// start anything).
    Follow,
}

/// Show this node only while the condition holds (the page's `hidden` toggles).
#[derive(Component, Clone, Copy)]
pub enum ShowWhen {
    Panel(Tab),
    /// The blob preview card: on the Solo and Customise tabs.
    Preview,
    /// Only the Solo and How to play tabs (the title and field of view make room for the
    /// Customise and With mates tabs).
    NotLook,
    /// The lobby card (online, waiting for a round).
    Lobby,
    /// Only the host sees this in the lobby (the Start button).
    LobbyHost,
    /// Not online.
    Offline,
    /// Online, in a yard (host or guest).
    InYard,
    /// Online as a guest.
    Guest,
    /// Anything but an online guest.
    NotGuest,
    HeistTeams,
    FriendlyFire,
    HeistNote,
    MoreOpen,
    Cheeky,
}

/// The old developer text (controls, numbers, key list). F3 shows it while playing.
#[derive(Component)]
pub struct DevHud;

#[derive(Resource, Default)]
struct ShowDev(bool);

#[derive(Component)]
struct MenuRoot;
#[derive(Component)]
struct PauseRoot;
#[derive(Component)]
struct ResultsRoot;
#[derive(Component)]
struct NameText;
#[derive(Component)]
struct BlobName;
#[derive(Component)]
struct PlayLabel;
#[derive(Component)]
struct AmmoNote;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        let args: Vec<String> = std::env::args().collect();
        // `--mode` and `--shot` start playing straight away (testing and screenshots)
        let direct = args.iter().any(|a| a == "--mode" || a == "--shot");
        let menu_shot = args.iter().any(|a| a == "--menu");
        app.init_resource::<ShowDev>()
            .insert_resource(Settings::load())
            .insert_resource(MenuUi {
                // `--tab look` (screenshots) opens on the Customise tab
                tab: match args.iter().position(|a| a == "--tab").and_then(|i| args.get(i + 1)).map(String::as_str) {
                    Some("look") => Tab::Look,
                    Some("mates") => Tab::Mates,
                    Some("how") => Tab::How,
                    _ => Tab::Solo,
                },
                more_open: false,
                name_focus: false,
                request: None,
                net_focus: None,
                net_request: None,
                toggle_ready: false,
            })
            .insert_resource(if direct && !menu_shot { Screen::Playing } else { Screen::Menu })
            .add_systems(Startup, (spawn_menu, spawn_pause, apply_loaded_settings).chain())
            .add_systems(
                Update,
                (
                    widget_clicks,
                    slider_drag,
                    name_typing,
                    net_typing,
                    look_marks,
                    paint_lobby,
                    test_looks,
                    net_texts,
                    scroll_cards,
                    run_requests,
                    flow_keys,
                    sync_cursor,
                    paint_widgets,
                    show_screens,
                    dev_hud,
                    open_results,
                    menu_camera,
                    menu_yard_preview,
                )
                    .chain(),
            );
    }
}

// ---------------------------------------------------------------- building the screens

fn spawn_menu(mut commands: Commands, settings: Res<Settings>, preview: Res<crate::preview::PreviewImage>) {
    let _ = settings;
    commands
        .spawn((overlay(false), MenuRoot))
        .with_children(|o| {
            o.spawn(card(452.0)).with_children(|c| {
                // brand (hidden on the Customise tab, to make room)
                c.spawn((Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                    row_gap: px(10.0),
                    padding: UiRect::top(px(4.0)),
                    flex_shrink: 0.0,
                    ..default()
                }, ShowWhen::NotLook))
                .with_children(|b| {
                    b.spawn(tag_pill("Backyard Brawl · Rust prototype"));
                    outlined_title(b, "Australian\nBBQ", 54.0);
                    b.spawn(Node { max_width: px(360.0), ..default() }).with_child(text(
                        "Chuck stuff at your mates in the backyard. Hit them before they hit you!",
                        15.0,
                        false,
                        MUTED,
                    ));
                });
                // name
                c.spawn(column(6.0)).with_children(|f| {
                    f.spawn(legend("Your name"));
                    f.spawn((
                        Button,
                        Node {
                            width: Val::Percent(100.0),
                            padding: UiRect::axes(px(12.0), px(10.0)),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::all(px(12.0)),
                            ..default()
                        },
                        BackgroundColor(WHITE),
                        BorderColor::all(INK),
                        Action::FocusName,
                        children![(text("", 15.0, true, INK), NameText)],
                    ));
                });
                c.spawn((column(0.0), ShowWhen::NotLook)).with_children(|w| {
                    slider(w, SliderId::Fov, "Field of view", 60.0, 105.0);
                });
                // tabs
                c.spawn(row(8.0)).with_children(|t| {
                    tab_button(t, Tab::Solo, "Solo");
                    tab_button(t, Tab::Look, "Customise");
                    tab_button(t, Tab::Mates, "With mates");
                    tab_button(t, Tab::How, "How to play");
                });
                solo_panel(c);
                look_panel(c);
                mates_panel(c);
                how_panel(c);
                c.spawn(Node { width: Val::Percent(100.0), ..default() })
                    .with_children(|w| {
                        check(w, CheckId::Sound, "Sound effects");
                    });
            });
            // the character preview, in a card beside the menu (Solo tab)
            o.spawn((
                Node {
                    width: px(340.0),
                    margin: UiRect::left(px(28.0)),
                    padding: UiRect::all(px(16.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10.0),
                    border: UiRect::all(px(3.0)),
                    border_radius: BorderRadius::all(px(18.0)),
                    align_self: AlignSelf::FlexStart,
                    ..default()
                },
                BackgroundColor(PAPER),
                BorderColor::all(INK),
                BoxShadow(vec![ShadowStyle {
                    color: INK,
                    x_offset: px(7.0),
                    y_offset: px(7.0),
                    spread_radius: px(0.0),
                    blur_radius: px(0.0),
                }]),
                ShowWhen::Preview,
            ))
            .with_children(|pv| {
                pv.spawn(legend("Your blob"));
                pv.spawn((
                    Text::new("Gumdrop"),
                    display_font(28.0),
                    TextColor(INK),
                    BlobName,
                ));
                pv.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        aspect_ratio: Some(crate::preview::PREVIEW_W as f32 / crate::preview::PREVIEW_H as f32),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(12.0)),
                        ..default()
                    },
                    BorderColor::all(INK),
                    ImageNode::new(preview.0.clone()),
                    // hold the left button and drag to spin the blob
                    Interaction::default(),
                    crate::preview::PreviewDrag,
                ));
                pv.spawn(text("Hold the left button and drag to spin", 12.0, false, MUTED));
                pv.spawn(text(
                    "Change your blob on the Customise tab. Only the look changes: every blob has the same speed and hit size.",
                    12.5,
                    false,
                    MUTED,
                ));
            });
            lobby_card(o);
        });
}

/// The lobby card (online, waiting for the round): the room code to share, who is here and who
/// is ready, the Ready button, and the host's Start button. Everybody's blob stands in the yard
/// behind the cards (see `lobby.rs`).
fn lobby_card(o: &mut ChildSpawnerCommands) {
    o.spawn((
        Node {
            // at the right-hand edge, so the line-up stands in the middle between the cards
            position_type: PositionType::Absolute,
            right: px(24.0),
            top: px(20.0),
            width: px(320.0),
            padding: UiRect::all(px(16.0)),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            border: UiRect::all(px(3.0)),
            border_radius: BorderRadius::all(px(18.0)),
            align_self: AlignSelf::FlexStart,
            display: Display::None,
            ..default()
        },
        BackgroundColor(PAPER),
        BorderColor::all(INK),
        BoxShadow(vec![ShadowStyle { color: INK, x_offset: px(7.0), y_offset: px(7.0), spread_radius: px(0.0), blur_radius: px(0.0) }]),
        ShowWhen::Lobby,
    ))
    .with_children(|c| {
        c.spawn(legend("Lobby: room code"));
        c.spawn((Text::new(""), display_font(34.0), TextColor(INK), LobbyCode));
        c.spawn(text("Mates join with this code and your server link (With mates tab).", 12.5, false, MUTED));
        c.spawn(legend("Players"));
        c.spawn(column(5.0)).with_children(|list| {
            for i in 0..bbq_core::net::MAX_PLAYERS {
                list.spawn((
                    Node {
                        width: Val::Percent(100.0),
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::axes(px(10.0), px(5.0)),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::all(px(10.0)),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(WHITE),
                    BorderColor::all(INK),
                    LobbyRow(i),
                ))
                .with_children(|r| {
                    r.spawn((text("", 14.0, true, INK), LobbyRowName(i)));
                    r.spawn((text("", 13.0, true, MUTED), LobbyRowState(i)));
                });
            }
        });
        c.spawn((text("", 14.0, true, INK), LobbyStatus));
        c.spawn(row(8.0)).with_children(|r| {
            let b = button(r, Action::LobbyReady, "Ready!", Some(SUN));
            r.commands().entity(b).insert(LobbyReadyButton);
        });
        c.spawn((row(8.0), ShowWhen::LobbyHost)).with_children(|r| {
            button(r, Action::LobbyStart, "Start now", Some(TOMATO));
        });
        c.spawn(row(8.0)).with_children(|r| {
            button(r, Action::Leave, "Leave the yard", None);
        });
    });
}

#[derive(Component)]
struct LobbyCode;
#[derive(Component)]
struct LobbyStatus;
#[derive(Component)]
struct LobbyReadyButton;
#[derive(Component)]
struct LobbyRow(usize);
#[derive(Component)]
struct LobbyRowName(usize);
#[derive(Component)]
struct LobbyRowState(usize);

/// Fill the lobby card: the code, everybody with their ready state, the countdown, and the
/// Ready button's words.
fn paint_lobby(
    lobby: Res<crate::online::Lobby>,
    online: Res<crate::online::Online>,
    mut code: Query<&mut Text, (With<LobbyCode>, Without<LobbyStatus>, Without<LobbyRowName>, Without<LobbyRowState>)>,
    mut status: Query<&mut Text, (With<LobbyStatus>, Without<LobbyCode>, Without<LobbyRowName>, Without<LobbyRowState>)>,
    mut rows: Query<(&LobbyRow, &mut Node, &mut BackgroundColor)>,
    mut names: Query<(&LobbyRowName, &mut Text), (Without<LobbyCode>, Without<LobbyStatus>, Without<LobbyRowState>)>,
    mut states: Query<(&LobbyRowState, &mut Text, &mut TextColor), (Without<LobbyCode>, Without<LobbyStatus>, Without<LobbyRowName>)>,
    ready_btn: Query<&Children, With<LobbyReadyButton>>,
    mut btn_text: Query<&mut Text, (Without<LobbyCode>, Without<LobbyStatus>, Without<LobbyRowName>, Without<LobbyRowState>)>,
) {
    if !lobby.active {
        return;
    }
    let Some(s) = online.session.as_ref() else { return };
    let set = |t: &mut Text, v: String| {
        if t.0 != v {
            t.0 = v;
        }
    };
    for mut t in &mut code {
        set(&mut t, s.room.to_uppercase());
    }
    for (r, mut node, mut bg) in &mut rows {
        let m = s.members.get(r.0);
        let want = if m.is_some() { Display::Flex } else { Display::None };
        if node.display != want {
            node.display = want;
        }
        let ready = m.is_some_and(|m| m.ready);
        let colour = if ready { GOOD } else { WHITE };
        if bg.0 != colour {
            bg.0 = colour;
        }
    }
    for (n, mut t) in &mut names {
        let v = s.members.get(n.0).map_or(String::new(), |m| {
            let mut v = m.name.clone();
            if m.id == s.my_id {
                v += " (you)";
            }
            if m.host {
                v += " · host";
            }
            v
        });
        set(&mut t, v);
    }
    for (n, mut t, mut colour) in &mut states {
        let ready = s.members.get(n.0).is_some_and(|m| m.ready);
        set(&mut t, if ready { "READY".to_string() } else { "not ready".to_string() });
        let c = if ready { INK } else { MUTED };
        if colour.0 != c {
            colour.0 = c;
        }
    }
    let ready_count = s.members.iter().filter(|m| m.ready).count();
    let line = if s.start_in > 0.0 {
        format!("Everybody's ready! Starting in {}...", s.start_in.ceil() as u32)
    } else if s.members.len() < 2 {
        "Waiting for mates to join...".to_string()
    } else {
        format!("{ready_count} of {} ready. Everybody presses Ready to start.", s.members.len())
    };
    for mut t in &mut status {
        set(&mut t, line.clone());
    }
    let label = if s.i_am_ready() { "Not ready" } else { "Ready!" };
    for kids in &ready_btn {
        for k in kids.iter() {
            if let Ok(mut t) = btn_text.get_mut(k) {
                set(&mut t, label.to_string());
            }
        }
    }
}

/// The Customise tab: a row of arrows for each feature of your blob.
fn look_panel(c: &mut ChildSpawnerCommands) {
    use bbq_core::appearance::{Brows, Choice, Hair, Mouth, Palette};
    c.spawn((column(10.0), ShowWhen::Panel(Tab::Look))).with_children(|p| {
        // the body: its shape and its belly
        p.spawn(legend("Body"));
        let shapes: Vec<&str> = Character::ALL.iter().map(|c| c.name()).collect();
        style_buttons(p, LookRow::Shape, "Shape", &shapes);
        slider(p, SliderId::Belly, "Beer belly (Classic has none)", 0.0, 2.0);
        // the styles: a row of buttons each
        p.spawn(legend("Face and hair"));
        let styles: [(LookRow, &str, Vec<&str>); 3] = [
            (LookRow::Hair, "Hair", Hair::ALL.iter().map(|x| x.name()).collect()),
            (LookRow::Mouth, "Mouth", Mouth::ALL.iter().map(|x| x.name()).collect()),
            (LookRow::Brows, "Brows", Brows::ALL.iter().map(|x| x.name()).collect()),
        ];
        for (row, label, names) in styles {
            style_buttons(p, row, label, &names);
        }
        // the colours: a row of swatches each
        p.spawn(legend("Colours"));
        let colours = [
            (LookRow::Body, "Body", Palette::Body),
            (LookRow::Singlet, "Singlet", Palette::Singlet),
            (LookRow::Thongs, "Thongs", Palette::Thong),
            (LookRow::HairColour, "Hair", Palette::Hair),
            (LookRow::BrowColour, "Brows", Palette::Brow),
        ];
        for (row, label, pal) in colours {
            look_row(p, label, |r| {
                r.spawn(Node { flex_grow: 1.0, flex_wrap: FlexWrap::Wrap, column_gap: px(5.0), row_gap: px(5.0), ..default() })
                    .with_children(|w| {
                        for (i, c) in pal.colours().iter().enumerate() {
                            w.spawn((
                                Button,
                                Node {
                                    width: px(26.0),
                                    height: px(26.0),
                                    border: UiRect::all(px(2.0)),
                                    border_radius: BorderRadius::all(px(13.0)),
                                    ..default()
                                },
                                BackgroundColor(crate::models::hex(*c)),
                                BorderColor::all(INK),
                                Action::LookSet(row, i as u8),
                                LookChoice(row, i as u8),
                            ));
                        }
                    });
            });
        }
        p.spawn(row(8.0)).with_children(|r| {
            button(r, Action::LookRandom, "Randomise", Some(SUN));
            button(r, Action::LookReset, "Reset", None);
        });
    });
}

/// A Customise row of buttons, one per choice.
fn style_buttons(p: &mut ChildSpawnerCommands, row: LookRow, label: &str, names: &[&str]) {
    look_row(p, label, |r| {
        r.spawn(Node {
            flex_grow: 1.0,
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::all(px(10.0)),
            overflow: Overflow::clip(),
            ..default()
        })
        .insert((BackgroundColor(WHITE), BorderColor::all(INK)))
        .with_children(|g| {
            for (i, name) in names.iter().enumerate() {
                g.spawn((
                    Button,
                    Node {
                        flex_grow: 1.0,
                        flex_basis: px(0.0),
                        padding: UiRect::axes(px(2.0), px(6.0)),
                        justify_content: JustifyContent::Center,
                        border: UiRect::left(px(if i == 0 { 0.0 } else { 2.0 })),
                        ..default()
                    },
                    BorderColor::all(INK),
                    BackgroundColor(WHITE),
                    Action::LookSet(row, i as u8),
                    LookChoice(row, i as u8),
                    children![text(*name, 13.0, true, INK)],
                ));
            }
        });
    });
}

/// One line on the Customise tab: a label on the left, the choices to its right.
fn look_row(p: &mut ChildSpawnerCommands, label: &str, f: impl FnOnce(&mut ChildSpawnerCommands)) {
    p.spawn(Node { width: Val::Percent(100.0), align_items: AlignItems::Center, column_gap: px(10.0), ..default() })
        .with_children(|r| {
            r.spawn((Node { width: px(62.0), flex_shrink: 0.0, ..default() }, children![text(label, 14.0, true, INK)]));
            f(r);
        });
}

/// A choice on the Customise tab (which row, which number): it shows when it is picked.
#[derive(Component)]
struct LookChoice(LookRow, u8);

/// Show which choices are picked: the style buttons go yellow, the picked swatch gets a thick
/// ring.
fn look_marks(settings: Res<Settings>, mut q: Query<(&LookChoice, &mut BackgroundColor, &mut Node, &mut BorderColor)>) {
    use bbq_core::appearance::Choice;
    let l = settings.look;
    for (c, mut bg, mut node, mut border) in &mut q {
        let picked = match c.0 {
            LookRow::Shape => Character::ALL.iter().position(|x| *x == settings.character) == Some(c.1 as usize),
            LookRow::Hair => l.hair.index() == c.1,
            LookRow::Mouth => l.mouth.index() == c.1,
            LookRow::Brows => l.brows.index() == c.1,
            LookRow::Body => l.body == c.1,
            LookRow::Singlet => l.singlet == c.1,
            LookRow::Thongs => l.thong == c.1,
            LookRow::HairColour => l.hair_colour == c.1,
            LookRow::BrowColour => l.brow_colour == c.1,
        };
        let style = matches!(c.0, LookRow::Shape | LookRow::Hair | LookRow::Mouth | LookRow::Brows);
        if style {
            let want = if picked { SUN } else { WHITE };
            if bg.0 != want {
                bg.0 = want;
            }
        } else {
            // a swatch: a thick ink ring (and slightly bigger) when picked
            let (w, size) = if picked { (4.0, 30.0) } else { (2.0, 26.0) };
            if node.width != px(size) {
                node.width = px(size);
                node.height = px(size);
                node.border = UiRect::all(px(w));
                node.border_radius = BorderRadius::all(px(size / 2.0));
                *border = BorderColor::all(if picked { INK } else { Color::srgba(0.09, 0.14, 0.09, 0.45) });
            }
        }
    }
}


fn solo_panel(c: &mut ChildSpawnerCommands) {
    c.spawn((column(14.0), ShowWhen::Panel(Tab::Solo))).with_children(|outer| {
    // online as a guest: the host picks the game
    outer.spawn((
        Node { display: Display::None, ..default() },
        ShowWhen::Guest,
        children![text("You're in a mate's yard: the host picks the game mode and the rules. Customise your blob and press Ready in the lobby.", 14.0, true, INK)],
    ));
    outer.spawn((column(14.0), ShowWhen::NotGuest)).with_children(|p| {
        // online as the host: no bots, and the round starts from the lobby card
        p.spawn((
            Node { display: Display::None, ..default() },
            ShowWhen::InYard,
            children![text("Pick the game mode here, then press Start now in the lobby card (or wait for everybody to be Ready).", 13.0, true, INK)],
        ));
        p.spawn((row(0.0), ShowWhen::Offline)).with_children(|r| {
            let b = button(r, Action::Play, "Play", Some(TOMATO));
            r.commands().entity(b).insert(PlayLabelMarker);
        });
        p.spawn((column(0.0), ShowWhen::Offline)).with_children(|w| {
            seg(w, SegId::Bots, "Bots", &[("None", 0), ("1", 1), ("2", 2), ("3", 3)]);
        });
        seg(
            p,
            SegId::Mode,
            "Game mode",
            &[("Free for all", 0), ("Teams (2 v 2)", 1), ("Teddy Heist", 2)],
        );
        p.spawn((
            Node { display: Display::None, ..default() },
            ShowWhen::HeistNote,
            children![text(
                "Each team guards a walled base full of teddies. Sneak into an enemy base, grab a teddy and run it back to yours to bank it. Most banked when the clock runs out wins.",
                13.0,
                false,
                MUTED
            )],
        ));
        p.spawn((column(6.0), ShowWhen::HeistTeams))
            .with_children(|h| {
                seg(h, SegId::HeistTeams, "Heist teams", &[("2", 2), ("3", 3), ("4", 4)]);
            });
        p.spawn(column(6.0)).with_children(|f| {
            f.spawn(legend("Cheeky mode"));
            f.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: px(6.0), row_gap: px(6.0), ..default() })
                .with_children(|w| {
                    check(w, CheckId::Cheeky, "Rude jokes, cheeky names");
                });
            f.spawn(text("Off is fine for the kids.", 13.0, false, MUTED));
        });
        // more options
        p.spawn(column(8.0)).with_children(|m| {
            m.spawn((
                Button,
                Node { padding: UiRect::axes(px(0.0), px(4.0)), ..default() },
                Action::More,
                children![text("More options", 13.0, true, MUTED)],
            ));
            m.spawn((column(12.0), ShowWhen::MoreOpen)).with_children(|o| {
                seg(o, SegId::RoundLen, "Round length", &[("1 min", 60), ("3 min", 180)]);
                seg(o, SegId::Skill, "Bot skill", &[("Easy", 0), ("Fair dinkum", 1), ("Spicy", 2)]);
                seg(o, SegId::Rounds, "Rounds", &[("Single", 1), ("Best of 3", 3), ("Best of 5", 5)]);
                o.spawn((column(6.0), ShowWhen::FriendlyFire)).with_children(|h| {
                    seg(h, SegId::FriendlyFire, "Friendly fire", &[("Off", 0), ("On", 1)]);
                });
                o.spawn(column(6.0)).with_children(|f| {
                    f.spawn(legend("What's in the yard"));
                    f.spawn(Node {
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: px(6.0),
                        row_gap: px(6.0),
                        ..default()
                    })
                    .with_children(|w| {
                        check(w, CheckId::Bar, "Bar");
                        check(w, CheckId::Bbq, "BBQ & Dazza");
                        let chest = check(w, CheckId::Chest, "Chest");
                        w.commands().entity(chest).insert(ShowWhen::Cheeky);
                        check(w, CheckId::Smoko, "Smoko");
                        check(w, CheckId::Naughty, "Smoko is the Naughty Corner");
                        check(w, CheckId::Falls, "Stacking it");
                        check(w, CheckId::DrunkAll, "Drunk mode (everyone's hammered all game)");
                    });
                });
            });
        });
    });
    });
}

#[derive(Component)]
struct PlayLabelMarker;

fn mates_panel(c: &mut ChildSpawnerCommands) {
    let field = |p: &mut ChildSpawnerCommands, label: &str, field: NetField, action: Action| {
        p.spawn(column(6.0)).with_children(|f| {
            f.spawn(legend(label));
            f.spawn((
                Button,
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(px(12.0), px(10.0)),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::all(px(12.0)),
                    ..default()
                },
                BackgroundColor(WHITE),
                BorderColor::all(INK),
                action,
                children![(text("", 15.0, true, INK), NetFieldText(field))],
            ));
        });
    };
    c.spawn((column(12.0), ShowWhen::Panel(Tab::Mates))).with_children(|p| {
        p.spawn(text(
            "One of you hosts and gets a room code. Your mates type the host's server link and the code, then join. Everybody meets in the lobby, customises, and presses Ready.",
            13.0,
            false,
            MUTED,
        ));
        field(p, "Server link (the host's)", NetField::Server, Action::FocusServer);
        p.spawn(row(0.0)).with_children(|r| {
            button(r, Action::HostYard, "Host a yard", Some(SUN));
        });
        p.spawn(legend("Or join a mate's yard"));
        field(p, "Room code", NetField::Room, Action::FocusRoom);
        p.spawn(row(0.0)).with_children(|r| {
            button(r, Action::Join, "Join their yard", None);
        });
        p.spawn((text("", 14.0, true, INK), NetStatusText));
        p.spawn((row(0.0), ShowWhen::Lobby)).with_children(|r| {
            button(r, Action::Leave, "Leave the yard", None);
        });
    });
}

fn how_panel(c: &mut ChildSpawnerCommands) {
    c.spawn((column(14.0), ShowWhen::Panel(Tab::How))).with_children(|p| {
        for line in [
            "Grab stuff off the lawn and chuck it at your mates. +100 a hit, -50 if you cop one.",
            "R does everything else: drink at the bar, raid the BBQ, sit on smoko, help a mate up, raid the chest.",
            "Drunk = wobbly, but hits score extra. Don't stack it.",
            "Steaks and fish are for slapping, and so is anything from the chest. Get up close and click.",
            "Smoko keeps you safe for 20 seconds, but you're drinking the whole time.",
        ] {
            p.spawn(text(format!("•  {line}"), 14.0, false, INK));
        }
        p.spawn(legend("Controls"));
        p.spawn(Node {
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::auto(), GridTrack::flex(1.0)],
            column_gap: px(12.0),
            row_gap: px(7.0),
            align_items: AlignItems::Center,
            width: Val::Percent(100.0),
            ..default()
        })
        .with_children(|g| {
            for (k, d) in [
                ("WASD", "Move, mouse to aim"),
                ("Hold click", "Wind up, let go to chuck (let go near the end of the bar and a hit knocks them over)"),
                ("Space", "Jump"),
                ("Shift", "Speed boost"),
                ("Right click", "Catch"),
                ("R", "Drink, grab, smoko, help up (hold)"),
                ("F", "Grab someone who's down and drag them, tap again to let go, hold to throw them"),
                ("Q", "Swap item"),
                ("T H B", "Taunt, dance, laugh"),
                ("G", "Throw away what you are holding"),
                ("Esc", "Pause"),
                ("Gamepad", "Sticks walk and look, A jumps, right trigger throws, left trigger catches, bumpers swap, X is R, Y is F, B drops, Start pauses"),
            ] {
                g.spawn(kbd(k));
                g.spawn(text(d, 13.5, false, INK));
            }
        });
        p.spawn(legend("Points"));
        p.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: px(6.0), row_gap: px(6.0), ..default() })
            .with_children(|w| {
                for (s, c) in [
                    ("+100 hit", GOOD),
                    ("-50 get hit", BAD),
                    ("+50 catch", GOOD),
                    ("+50 hit the leader", SUN),
                    ("streaks x1.5 / x2", SUN),
                    ("+25 / +50 drunk", SUN),
                    ("+50 pool or tramp", GOOD),
                    ("+50 meat slap", GOOD),
                    ("+25 help a mate up", GOOD),
                ] {
                    w.spawn(chip(s, c));
                }
            });
    });
}

fn spawn_pause(mut commands: Commands) {
    commands
        .spawn((overlay(true), PauseRoot, Visibility::Hidden))
        .with_children(|o| {
            o.spawn(card(520.0)).with_children(|c| {
                c.spawn((
                    Text::new("Smoko"),
                    display_font(34.0),
                    TextColor(INK),
                ));
                c.spawn((
                    Node { display: Display::Flex, ..default() },
                    ShowWhen::Offline,
                    children![text("Round paused. Resume grabs the mouse again.", 14.0, false, MUTED)],
                ));
                c.spawn((
                    Node { display: Display::None, ..default() },
                    ShowWhen::InYard,
                    children![text("Playing online: the round keeps going for everybody while you're here.", 14.0, false, MUTED)],
                ));
                slider(c, SliderId::Fov, "Field of view", 60.0, 105.0);
                c.spawn(row(10.0)).with_children(|r| {
                    button(r, Action::Resume, "Resume", Some(SUN));
                    // online, a guest cannot end the round for everybody: they leave instead
                    r.spawn((Node { flex_grow: 1.0, flex_basis: px(0.0), display: Display::Flex, ..default() }, ShowWhen::NotGuest))
                        .with_children(|w| {
                            button(w, Action::Quit, "Quit to menu", None);
                        });
                    r.spawn((Node { flex_grow: 1.0, flex_basis: px(0.0), display: Display::None, ..default() }, ShowWhen::Guest))
                        .with_children(|w| {
                            button(w, Action::Leave, "Leave the yard", None);
                        });
                });
            });
        });
}

fn apply_loaded_settings(mut settings: ResMut<Settings>, mut player: ResMut<Player>, mut cast: ResMut<Cast>) {
    player.fov_base = settings.fov;
    player.fov = settings.fov;
    cast.mine = settings.character;
    // `--char classic|pear|egg|gumdrop` (testing): the first bot wears this one
    let args: Vec<String> = std::env::args().collect();
    if let Some(b) = args.iter().position(|a| a == "--belly").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<f32>().ok()) {
        settings.belly = b.clamp(0.0, 2.0);
    }
    if let Some(c) = args.iter().position(|a| a == "--char").and_then(|i| args.get(i + 1)) {
        if let Some(ch) = Character::ALL.iter().find(|x| x.name().eq_ignore_ascii_case(c)) {
            cast.mine = *ch;
        }
    }
    // `--name NAME` (testing two copies on one computer)
    if let Some(n) = args.iter().position(|a| a == "--name").and_then(|i| args.get(i + 1)) {
        settings.name = n.chars().take(14).collect();
    }
    // `--hair tuft|mullet|bowl|none` and `--body N` (testing)
    if let Some(m) = args.iter().position(|a| a == "--hair").and_then(|i| args.get(i + 1)) {
        use bbq_core::appearance::Choice;
        if let Some(m) = bbq_core::appearance::Hair::from_name(m) {
            settings.look.hair = m;
            settings.look.hair_colour = 1;
        }
    }
    if let Some(n) = args.iter().position(|a| a == "--body").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<u8>().ok()) {
        settings.look.body = n;
        settings.look = settings.look.tidy();
    }
    // `--brows flat|arched|bushy|none` (testing)
    if let Some(m) = args.iter().position(|a| a == "--brows").and_then(|i| args.get(i + 1)) {
        use bbq_core::appearance::Choice;
        if let Some(m) = bbq_core::appearance::Brows::from_name(m) {
            settings.look.brows = m;
        }
    }
    // `--mouth smile|grin|smirk|none` (testing): your look's mouth, and every bot's
    if let Some(m) = args.iter().position(|a| a == "--mouth").and_then(|i| args.get(i + 1)) {
        use bbq_core::appearance::Choice;
        if let Some(m) = bbq_core::appearance::Mouth::from_name(m) {
            settings.look.mouth = m;
        }
    }
}

/// `--mouth` also dresses every bot the same (testing and screenshots).
fn test_looks(mut game: ResMut<Game>) {
    use bbq_core::appearance::Choice;
    let args: Vec<String> = std::env::args().collect();
    let mouth = args.iter().position(|a| a == "--mouth").and_then(|i| args.get(i + 1)).and_then(|m| bbq_core::appearance::Mouth::from_name(m));
    let brows = args.iter().position(|a| a == "--brows").and_then(|i| args.get(i + 1)).and_then(|m| bbq_core::appearance::Brows::from_name(m));
    let hair = args.iter().position(|a| a == "--hair").and_then(|i| args.get(i + 1)).and_then(|m| bbq_core::appearance::Hair::from_name(m));
    for d in &mut game.dummies {
        if let Some(h) = hair {
            d.look.hair = h;
            d.look.hair_colour = 0;
        }
        if let Some(m) = mouth {
            d.look.mouth = m;
        }
        if let Some(b) = brows {
            d.look.brows = b;
            d.look.brow_colour = 0;
        }
    }
}

// ---------------------------------------------------------------- clicks

fn widget_clicks(
    mut settings: ResMut<Settings>,
    mut ui: ResMut<MenuUi>,
    segs: Query<(&Interaction, &SegOption), Changed<Interaction>>,
    checks: Query<(&Interaction, &CheckBox), Changed<Interaction>>,
    actions: Query<(&Interaction, &Action), Changed<Interaction>>,
    mut cast: ResMut<Cast>,
) {
    let mut changed = false;
    for (i, s) in &segs {
        if *i != Interaction::Pressed {
            continue;
        }
        changed = true;
        let v = s.value;
        match s.id {
            SegId::Bots => settings.bots = v as usize,
            SegId::Mode => {
                settings.mode = match v {
                    1 => GameMode::Teams,
                    2 => GameMode::Heist,
                    _ => GameMode::FreeForAll,
                }
            }
            SegId::HeistTeams => settings.heist_teams = v as usize,
            SegId::RoundLen => settings.round_len = v as f32,
            SegId::Skill => {
                settings.skill = match v {
                    0 => Difficulty::Easy,
                    2 => Difficulty::Spicy,
                    _ => Difficulty::Fair,
                }
            }
            SegId::Rounds => settings.rounds = v as u32,
            SegId::FriendlyFire => settings.friendly_fire = v == 1,
            SegId::Character => {
                settings.character = Character::ALL[(v as usize).min(3)];
                cast.mine = settings.character;
            }
        }
    }
    for (i, c) in &checks {
        if *i != Interaction::Pressed {
            continue;
        }
        changed = true;
        match c.0 {
            CheckId::Cheeky => settings.cheeky = !settings.cheeky,
            CheckId::Bar => settings.features.bar = !settings.features.bar,
            CheckId::Bbq => settings.features.bbq = !settings.features.bbq,
            CheckId::Chest => settings.features.chest = !settings.features.chest,
            CheckId::Smoko => settings.features.smoko = !settings.features.smoko,
            CheckId::Naughty => settings.naughty = !settings.naughty,
            CheckId::Falls => settings.falls = !settings.falls,
            CheckId::DrunkAll => settings.drunk_all = !settings.drunk_all,
            CheckId::Sound => settings.sound = !settings.sound,
        }
    }
    for (i, a) in &actions {
        if *i != Interaction::Pressed {
            continue;
        }
        match a {
            Action::Tab(t) => ui.tab = *t,
            Action::More => ui.more_open = !ui.more_open,
            Action::FocusName => ui.name_focus = true,
            Action::Play => ui.request = Some(Request::Play),
            Action::Resume => ui.request = Some(Request::Resume),
            Action::Quit | Action::ToMenu => ui.request = Some(Request::Menu),
            Action::Again => ui.request = Some(Request::Again),
            Action::HostYard => ui.net_request = Some(crate::online::NetRequest::Host),
            Action::Join => ui.net_request = Some(crate::online::NetRequest::Join),
            Action::Leave => ui.net_request = Some(crate::online::NetRequest::Leave),
            Action::LookStep(row, dir) => {
                use bbq_core::appearance::Choice;
                match row {
                    LookRow::Shape => {}
                    LookRow::Body => settings.look.step_colour(bbq_core::appearance::Palette::Body, *dir),
                    LookRow::Singlet => settings.look.step_colour(bbq_core::appearance::Palette::Singlet, *dir),
                    LookRow::Thongs => settings.look.step_colour(bbq_core::appearance::Palette::Thong, *dir),
                    LookRow::Hair => settings.look.hair = settings.look.hair.step(*dir),
                    LookRow::HairColour => settings.look.step_colour(bbq_core::appearance::Palette::Hair, *dir),
                    LookRow::Mouth => settings.look.mouth = settings.look.mouth.step(*dir),
                    LookRow::Brows => settings.look.brows = settings.look.brows.step(*dir),
                    LookRow::BrowColour => settings.look.step_colour(bbq_core::appearance::Palette::Brow, *dir),
                }
                changed = true;
            }
            Action::LookSet(row, v) => {
                use bbq_core::appearance::{Brows, Choice, Hair, Mouth};
                if *row == LookRow::Shape {
                    settings.character = Character::ALL[(*v as usize).min(Character::ALL.len() - 1)];
                    cast.mine = settings.character;
                }
                let l = &mut settings.look;
                match row {
                    LookRow::Shape => {}
                    LookRow::Hair => l.hair = Hair::from_index(*v),
                    LookRow::Mouth => l.mouth = Mouth::from_index(*v),
                    LookRow::Brows => l.brows = Brows::from_index(*v),
                    LookRow::Body => l.body = *v,
                    LookRow::Singlet => l.singlet = *v,
                    LookRow::Thongs => l.thong = *v,
                    LookRow::HairColour => l.hair_colour = *v,
                    LookRow::BrowColour => l.brow_colour = *v,
                }
                *l = l.tidy();
                changed = true;
            }
            Action::LobbyReady => ui.toggle_ready = true,
            Action::LobbyStart => ui.request = Some(Request::Play),
            Action::LookRandom => {
                let seed = (std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.as_nanos() as u64)) | 1;
                settings.look = bbq_core::appearance::Appearance::random(&mut bbq_core::rng::Rng::new(seed));
                changed = true;
            }
            Action::LookReset => {
                settings.look = bbq_core::appearance::Appearance::default();
                changed = true;
            }
            Action::FocusServer => ui.net_focus = Some(NetField::Server),
            Action::FocusRoom => ui.net_focus = Some(NetField::Room),
        }
        if *a != Action::FocusName {
            ui.name_focus = false;
        }
        if !matches!(a, Action::FocusServer | Action::FocusRoom) {
            ui.net_focus = None;
        }
    }
    if changed {
        settings.save();
    }
}

fn slider_drag(
    mut settings: ResMut<Settings>,
    mut player: ResMut<Player>,
    sliders: Query<(&Interaction, &RelativeCursorPosition, &Slider)>,
) {
    for (i, rel, s) in &sliders {
        if *i != Interaction::Pressed {
            continue;
        }
        let Some(n) = rel.normalized else { continue };
        let t = (n.x + 0.5).clamp(0.0, 1.0);
        match s.id {
            SliderId::Fov => {
                let v = (s.min + (s.max - s.min) * t).round();
                if v != settings.fov {
                    settings.fov = v;
                    player.fov_base = v;
                    player.fov = v;
                    settings.save();
                }
            }
            SliderId::Belly => {
                let v = ((s.min + (s.max - s.min) * t) * 20.0).round() / 20.0;
                if v != settings.belly {
                    settings.belly = v;
                    settings.save();
                }
            }
        }
    }
}

fn name_typing(
    mut ev: MessageReader<KeyboardInput>,
    mut settings: ResMut<Settings>,
    mut ui: ResMut<MenuUi>,
    screen: Res<Screen>,
    mouse: Res<ButtonInput<MouseButton>>,
    names: Query<&RelativeCursorPosition, With<NameText>>,
) {
    let _ = names;
    if *screen != Screen::Menu || !ui.name_focus {
        ev.clear();
        return;
    }
    let _ = mouse;
    for e in ev.read() {
        if !e.state.is_pressed() {
            continue;
        }
        match &e.logical_key {
            Key::Backspace => {
                settings.name.pop();
            }
            Key::Enter | Key::Escape => ui.name_focus = false,
            Key::Space => {
                if settings.name.chars().count() < 14 {
                    settings.name.push(' ');
                }
            }
            Key::Character(s) => {
                for ch in s.chars() {
                    if !ch.is_control() && settings.name.chars().count() < 14 {
                        settings.name.push(ch);
                    }
                }
            }
            _ => {}
        }
        settings.save();
    }
}

/// Typing into the server and room boxes.
fn net_typing(
    mut ev: MessageReader<KeyboardInput>,
    mut settings: ResMut<Settings>,
    mut ui: ResMut<MenuUi>,
    screen: Res<Screen>,
) {
    let Some(field) = ui.net_focus else {
        ev.clear();
        return;
    };
    if *screen != Screen::Menu {
        ev.clear();
        return;
    }
    for e in ev.read() {
        if !e.state.is_pressed() {
            continue;
        }
        let (text, max) = match field {
            NetField::Server => (&mut settings.server, 80),
            NetField::Room => (&mut settings.room, 24),
        };
        match &e.logical_key {
            Key::Backspace => {
                text.pop();
            }
            Key::Enter | Key::Escape => ui.net_focus = None,
            Key::Character(s) => {
                for ch in s.chars() {
                    let ok = match field {
                        NetField::Server => ch.is_ascii_alphanumeric() || ":./-_".contains(ch),
                        NetField::Room => ch.is_ascii_alphanumeric() || ch == '-',
                    };
                    if ok && text.chars().count() < max {
                        text.push(if field == NetField::Room { ch.to_ascii_lowercase() } else { ch });
                    }
                }
            }
            _ => {}
        }
        settings.save();
    }
}

/// The server and room boxes, and the line saying how it is going.
fn net_texts(
    settings: Res<Settings>,
    ui: Res<MenuUi>,
    online: Res<crate::online::Online>,
    mut fields: Query<(&NetFieldText, &mut Text), Without<NetStatusText>>,
    mut status: Query<&mut Text, With<NetStatusText>>,
) {
    for (f, mut t) in &mut fields {
        let (value, blank) = match f.0 {
            NetField::Server => (&settings.server, "address of the host's server"),
            NetField::Room => (&settings.room, "the code your mate gives you"),
        };
        let caret = if ui.net_focus == Some(f.0) { "|" } else { "" };
        t.0 = if value.is_empty() && caret.is_empty() { blank.to_string() } else { format!("{value}{caret}") };
    }
    for mut t in &mut status {
        let s = online.status();
        if t.0 != s {
            t.0 = s;
        }
    }
}

fn scroll_cards(
    wheel: Res<AccumulatedMouseScroll>,
    mut cards: Query<(&RelativeCursorPosition, &mut ScrollPosition), With<Node>>,
) {
    if wheel.delta.y == 0.0 {
        return;
    }
    let step = match wheel.unit {
        MouseScrollUnit::Line => 40.0,
        MouseScrollUnit::Pixel => 1.0,
    };
    for (rel, mut sp) in &mut cards {
        if rel.cursor_over {
            sp.y = (sp.y - wheel.delta.y * step).max(0.0);
        }
    }
}

// ---------------------------------------------------------------- painting

fn paint_widgets(
    settings: Res<Settings>,
    ui: Res<MenuUi>,
    lobby: Res<crate::online::Lobby>,
    mut segs: Query<(&SegOption, &mut BackgroundColor, &Children), (Without<CheckBox>, Without<TabButton>)>,
    mut checks: Query<(&CheckBox, &mut BackgroundColor, &Children), (Without<SegOption>, Without<TabButton>)>,
    mut marks: Query<&mut BackgroundColor, (With<CheckMark>, Without<SegOption>, Without<CheckBox>, Without<TabButton>)>,
    mut tabs: Query<(&TabButton, &mut BackgroundColor), (Without<SegOption>, Without<CheckBox>)>,
    mut shows: Query<(&ShowWhen, &mut Node)>,
    mut texts: ParamSet<(
        Query<&mut Text, With<NameText>>,
        Query<(&SliderValue, &mut Text)>,
    )>,
    mut handles: Query<(&SliderFill, &mut Node), Without<ShowWhen>>,
    mut play_label: Query<&Children, With<PlayLabelMarker>>,
    mut label_text: Query<&mut Text, (Without<NameText>, Without<SliderValue>, Without<BlobName>)>,
    mut blob_name: Query<&mut Text, (With<BlobName>, Without<NameText>, Without<SliderValue>)>,
) {
    let selected = |id: SegId, v: i32| -> bool {
        match id {
            SegId::Bots => settings.bots as i32 == v,
            SegId::Mode => {
                v == match settings.mode {
                    GameMode::FreeForAll => 0,
                    GameMode::Teams => 1,
                    GameMode::Heist => 2,
                }
            }
            SegId::HeistTeams => settings.heist_teams as i32 == v,
            SegId::RoundLen => settings.round_len as i32 == v,
            SegId::Skill => {
                v == match settings.skill {
                    Difficulty::Easy => 0,
                    Difficulty::Fair => 1,
                    Difficulty::Spicy => 2,
                }
            }
            SegId::Rounds => settings.rounds as i32 == v,
            SegId::FriendlyFire => (v == 1) == settings.friendly_fire,
            SegId::Character => Character::ALL[(v as usize).min(3)] == settings.character,
        }
    };
    for (s, mut bg, _) in &mut segs {
        bg.0 = if selected(s.id, s.value) { SUN } else { WHITE };
    }
    let on = |id: CheckId| match id {
        CheckId::Cheeky => settings.cheeky,
        CheckId::Bar => settings.features.bar,
        CheckId::Bbq => settings.features.bbq,
        CheckId::Chest => settings.features.chest,
        CheckId::Smoko => settings.features.smoko,
        CheckId::Naughty => settings.naughty,
        CheckId::Falls => settings.falls,
        CheckId::DrunkAll => settings.drunk_all,
        CheckId::Sound => settings.sound,
    };
    for (c, mut bg, kids) in &mut checks {
        let v = on(c.0);
        bg.0 = if v { SUN } else { WHITE };
        for k in kids.iter() {
            if let Ok(mut m) = marks.get_mut(k) {
                m.0 = if v { INK } else { WHITE };
            }
        }
    }
    for (t, mut bg) in &mut tabs {
        bg.0 = if t.0 == ui.tab { SUN } else { WHITE };
    }
    let heist = settings.mode == GameMode::Heist;
    let teams = settings.mode == GameMode::Teams;
    for (w, mut n) in &mut shows {
        let show = match w {
            ShowWhen::Panel(t) => *t == ui.tab,
            ShowWhen::Preview => matches!(ui.tab, Tab::Solo | Tab::Look) && !lobby.active,
            ShowWhen::Lobby => lobby.active,
            ShowWhen::LobbyHost => lobby.active && lobby.host,
            ShowWhen::Offline => !lobby.in_yard,
            ShowWhen::InYard => lobby.in_yard,
            ShowWhen::Guest => lobby.in_yard && !lobby.host,
            ShowWhen::NotGuest => !(lobby.in_yard && !lobby.host),
            ShowWhen::NotLook => matches!(ui.tab, Tab::Solo | Tab::How),
            ShowWhen::HeistTeams | ShowWhen::HeistNote => heist,
            ShowWhen::FriendlyFire => teams,
            ShowWhen::MoreOpen => ui.more_open,
            ShowWhen::Cheeky => settings.cheeky,
        };
        n.display = if show {
            match w {
                ShowWhen::Panel(_) | ShowWhen::MoreOpen | ShowWhen::HeistTeams | ShowWhen::FriendlyFire => {
                    Display::Flex
                }
                _ => Display::Flex,
            }
        } else {
            Display::None
        };
    }
    for mut t in &mut blob_name {
        if t.0 != settings.character.name() {
            t.0 = settings.character.name().to_string();
        }
    }
    for mut t in &mut texts.p0() {
        let caret = if ui.name_focus { "|" } else { "" };
        t.0 = format!("{}{caret}", settings.name);
    }
    let belly_word = |b: f32| match b {
        b if b < 0.15 => "None",
        b if b < 0.6 => "Light",
        b if b < 1.2 => "Healthy",
        b if b < 1.65 => "Proper",
        _ => "Enormous",
    };
    for (id, mut t) in &mut texts.p1() {
        t.0 = match id.0 {
            SliderId::Fov => format!("{:.0}", settings.fov),
            SliderId::Belly => belly_word(settings.belly).to_string(),
        };
    }
    for (id, mut n) in &mut handles {
        let frac = match id.0 {
            SliderId::Fov => (settings.fov - 60.0) / 45.0,
            SliderId::Belly => settings.belly / 2.0,
        };
        n.left = Val::Percent(frac * 100.0);
    }
    let label = match settings.mode {
        GameMode::Heist => "Play Teddy Heist",
        GameMode::Teams => "Play Teams",
        GameMode::FreeForAll => "Play",
    };
    for kids in &mut play_label {
        for k in kids.iter() {
            if let Ok(mut t) = label_text.get_mut(k) {
                t.0 = label.to_string();
            }
        }
    }
}

fn show_screens(
    screen: Res<Screen>,
    mut menu: Query<&mut Visibility, (With<MenuRoot>, Without<PauseRoot>, Without<ResultsRoot>)>,
    mut pause: Query<&mut Visibility, (With<PauseRoot>, Without<MenuRoot>, Without<ResultsRoot>)>,
    mut results: Query<&mut Visibility, (With<ResultsRoot>, Without<MenuRoot>, Without<PauseRoot>)>,
) {
    let vis = |on: bool| if on { Visibility::Inherited } else { Visibility::Hidden };
    for mut v in &mut menu {
        *v = vis(*screen == Screen::Menu);
    }
    for mut v in &mut pause {
        *v = vis(*screen == Screen::Paused);
    }
    for mut v in &mut results {
        *v = vis(*screen == Screen::Results);
    }
}

// ---------------------------------------------------------------- the flow

/// Put the chosen options into the game and begin a round.
pub fn begin(settings: &Settings, g: &mut Game, p: &mut Player, yard: &mut YardRes, cast: &mut Cast) {
    cast.mine = settings.character;
    g.attract = false;
    g.round.setup = Setup {
        mode: settings.mode,
        heist_teams: settings.heist_teams,
        // with mates in the yard the bots stay out of it (for now)
        bots: if g.remotes.is_empty() { settings.bots } else { 0 },
        round_len: settings.round_len,
        match_len: if settings.mode == GameMode::Heist { 1 } else { settings.rounds },
        friendly_fire: settings.mode == GameMode::Teams && settings.friendly_fire,
    };
    g.options.adult = settings.cheeky;
    g.options.naughty = settings.naughty && settings.features.smoko;
    g.options.smoko_on = settings.features.smoko;
    g.options.falls_on = settings.falls;
    g.options.drunk_mode = settings.drunk_all;
    g.options.bot_difficulty = settings.skill;
    g.options.bots_on = true;
    yard.0.features = settings.features;
    p.fov_base = settings.fov;
    start_round(g, p, yard);
}

/// Back to the yard-behind-the-menu: three bots, some things lying about, no round.
pub fn attract(settings: &Settings, g: &mut Game, p: &mut Player, yard: &mut YardRes) {
    g.attract = true;
    g.round.timed = false;
    g.round.results = None;
    g.round.banner = None;
    g.round.mtch = bbq_core::matchflow::Match::new(1);
    g.rules.mode = GameMode::FreeForAll;
    g.rules.friendly_fire = false;
    g.rules.phase = bbq_core::scoring::Phase::Play;
    g.heist = None;
    g.teams.clear();
    g.options.bots_on = true;
    g.options.adult = settings.cheeky;
    g.options.naughty = settings.naughty && settings.features.smoko;
    g.options.smoko_on = settings.features.smoko;
    g.options.falls_on = settings.falls;
    g.options.drunk_mode = false;
    g.options.bot_difficulty = Difficulty::Fair;
    yard.0 = bbq_core::yard::Yard::new(settings.features, yard.0.chest_spot);
    g.round.setup.mode = GameMode::FreeForAll;
    g.round.setup.bots = 3;
    // reuse the round set-up to get a clean yard and bots, then turn the clock off again
    start_round(g, p, yard);
    // the menu yard is not a round: the real match starts fresh when you press Play
    g.round.mtch = bbq_core::matchflow::Match::new(1);
    g.round.timed = false;
    g.rules.phase = bbq_core::scoring::Phase::Play;
    g.round.banner = None;
    g.board = bbq_core::scoring::Scoreboard::new();
    g.board.ensure(PLAYER_ID);
    for d in &g.dummies {
        g.board.ensure(d.id);
    }
    g.feed.clear();
    g.popups.clear();
    for d in &mut g.dummies {
        d.team = None::<Team>;
    }
}

fn run_requests(
    mut ui: ResMut<MenuUi>,
    mut screen: ResMut<Screen>,
    mut settings: ResMut<Settings>,
    mut game: ResMut<Game>,
    mut player: ResMut<Player>,
    mut yard: ResMut<YardRes>,
    mut cast: ResMut<Cast>,
    mut started: Local<bool>,
    mut frames: Local<u32>,
) {
    // `--autoplay` presses Play for you after a moment (testing); `--bots N` sets the bot count
    *frames += 1;
    if *frames == 15 && *screen == Screen::Menu && std::env::args().any(|a| a == "--autoplay") {
        ui.request = Some(Request::Play);
    }
    // the first frame: the yard behind the menu
    if !*started {
        *started = true;
        {
            let args: Vec<String> = std::env::args().collect();
            if let Some(n) = args.iter().position(|a| a == "--bots").and_then(|i| args.get(i + 1)).and_then(|v| v.parse::<usize>().ok()) {
                settings.bots = n.min(3);
            }
        }
        if *screen == Screen::Menu {
            attract(&settings, &mut game, &mut player, &mut yard);
        }
        // `--screen pause|results` jumps to those cards (screenshots and testing)
        let args: Vec<String> = std::env::args().collect();
        match args.iter().position(|a| a == "--screen").and_then(|i| args.get(i + 1)).map(String::as_str) {
            Some("pause") => *screen = Screen::Paused,
            Some("results") => {
                let rules = bbq_core::scoring::Rules { phase: bbq_core::scoring::Phase::Play, ..game.rules };
                for (k, id) in crate::round::everyone(&game).into_iter().enumerate() {
                    game.board.award(&rules, id, 400 - 90 * k as i32);
                    game.board.count_throw(id);
                    game.board.count_throw(id);
                }
                crate::round::end_round(&mut game);
                game.round.panel_in = 0.0;
            }
            _ => {}
        }
    }
    let Some(req) = ui.request.take() else { return };
    match req {
        Request::Follow => {
            begin(&settings, &mut game, &mut player, &mut yard, &mut cast);
            *screen = Screen::Playing;
        }
        // a guest does not start rounds: the host does
        Request::Play | Request::Again if game.mirror => {}
        Request::Play => {
            begin(&settings, &mut game, &mut player, &mut yard, &mut cast);
            *screen = Screen::Playing;
        }
        Request::Again => {
            if game.round.mtch.over {
                game.round.mtch = bbq_core::matchflow::Match::new(1);
            }
            begin(&settings, &mut game, &mut player, &mut yard, &mut cast);
            *screen = Screen::Playing;
        }
        Request::Resume => {
            if *screen == Screen::Paused {
                *screen = Screen::Playing;
            }
        }
        Request::Menu => {
            attract(&settings, &mut game, &mut player, &mut yard);
            *screen = Screen::Menu;
            ui.tab = Tab::Solo;
        }
    }
}

fn flow_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut screen: ResMut<Screen>,
    mut ui: ResMut<MenuUi>,
    mut game: ResMut<Game>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        match *screen {
            Screen::Playing => {
                *screen = Screen::Paused;
                game.wind.cancel();
            }
            Screen::Paused => *screen = Screen::Playing,
            Screen::Menu if ui.name_focus => ui.name_focus = false,
            Screen::Menu if ui.net_focus.is_some() => ui.net_focus = None,
            _ => {}
        }
    }
}

/// The mouse is locked to the game only while playing.
fn sync_cursor(screen: Res<Screen>, mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    if !screen.is_changed() {
        return;
    }
    let play = *screen == Screen::Playing;
    cursor.grab_mode = if play { CursorGrabMode::Locked } else { CursorGrabMode::None };
    cursor.visible = !play;
}

/// The camera drifts round the yard behind the menu: radius 40, height 17, 0.05 rad/s.
fn menu_camera(
    screen: Res<Screen>,
    time: Res<Time>,
    lobby: Res<crate::online::Lobby>,
    mut cam: Query<&mut Transform, With<crate::player::EyeCamera>>,
) {
    if *screen != Screen::Menu {
        return;
    }
    if lobby.active {
        // the online lobby: looking at everybody's blobs in a row on the lawn
        for mut tf in &mut cam {
            let (at, look) = crate::lobby::stage_camera(lobby.count);
            *tf = Transform::from_translation(at).looking_at(look, Vec3::Y);
        }
        return;
    }
    let a = time.elapsed_secs() * 0.05;
    for mut tf in &mut cam {
        tf.translation = Vec3::new(a.sin() * 40.0, 17.0, a.cos() * 40.0);
        tf.look_at(Vec3::new(0.0, 1.0, 0.0), Vec3::Y);
    }
}

/// The yard behind the menu shows what is ticked.
fn menu_yard_preview(screen: Res<Screen>, settings: Res<Settings>, mut yard: ResMut<YardRes>) {
    if *screen != Screen::Menu || !settings.is_changed() {
        return;
    }
    if yard.0.features != settings.features {
        let spot = yard.0.chest_spot;
        yard.0 = bbq_core::yard::Yard::new(settings.features, spot);
    }
}

// ---------------------------------------------------------------- results

fn open_results(
    mut commands: Commands,
    mut screen: ResMut<Screen>,
    game: Res<Game>,
    settings: Res<Settings>,
    online: Res<crate::online::Online>,
    old: Query<Entity, With<ResultsRoot>>,
) {
    // online, the host chooses what happens next; a guest waits for the host
    let online_role = online.session.as_ref().filter(|s| s.joined()).map(|s| s.role);
    if *screen == Screen::Results && game.round.results.is_none() {
        // a new round began behind our back
        *screen = Screen::Playing;
    }
    if *screen != Screen::Playing {
        return;
    }
    let g = &*game;
    let ready = g.round.timed
        && g.rules.phase == bbq_core::scoring::Phase::Results
        && g.round.panel_in <= 0.0
        && g.round.results.is_some();
    if !ready {
        return;
    }
    for e in &old {
        commands.entity(e).despawn();
    }
    let t = crate::results::build(g, &settings);
    commands
        .spawn((overlay(true), ResultsRoot))
        .with_children(|o| {
            o.spawn(card(560.0)).with_children(|c| {
                c.spawn((Text::new(t.title.clone()), display_font(32.0), TextColor(INK)));
                c.spawn(text(t.line.clone(), 14.0, false, MUTED));
                if let Some(m) = &t.match_line {
                    c.spawn((
                        Node {
                            padding: UiRect::axes(px(10.0), px(6.0)),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::all(px(10.0)),
                            align_self: AlignSelf::FlexStart,
                            ..default()
                        },
                        BackgroundColor(SUN),
                        BorderColor::all(INK),
                        children![text(m.clone(), 14.0, true, INK)],
                    ));
                }
                crate::results::table(c, &t);
                match online_role {
                    Some(crate::online::Role::Guest) => {
                        c.spawn(text("Waiting for the host to play again or go back to the lobby...", 14.0, true, INK));
                        c.spawn(row(10.0)).with_children(|r| {
                            button(r, Action::Leave, "Leave the yard", None);
                        });
                    }
                    Some(crate::online::Role::Host) => {
                        c.spawn(row(10.0)).with_children(|r| {
                            button(r, Action::Again, &t.again_label, Some(SUN));
                            button(r, Action::ToMenu, "Back to the lobby", None);
                        });
                    }
                    None => {
                        c.spawn(row(10.0)).with_children(|r| {
                            button(r, Action::Again, &t.again_label, Some(SUN));
                            button(r, Action::ToMenu, "Menu", None);
                        });
                    }
                }
            });
        });
    *screen = Screen::Results;
    let _ = TICK_DT;
}

/// Run conditions for the other plugins.
pub fn playing(s: Res<Screen>) -> bool {
    *s == Screen::Playing
}

pub fn not_in_menu(s: Res<Screen>) -> bool {
    *s != Screen::Menu
}

/// The yard keeps moving behind the menu and the results card, but not while paused.
pub fn world_runs(s: Option<Res<Screen>>) -> bool {
    s.is_none_or(|s| *s != Screen::Paused)
}

fn dev_hud(
    keys: Res<ButtonInput<KeyCode>>,
    screen: Res<Screen>,
    mut show: ResMut<ShowDev>,
    mut q: Query<&mut Visibility, With<DevHud>>,
) {
    if keys.just_pressed(KeyCode::F3) {
        show.0 = !show.0;
    }
    let on = show.0 && *screen == Screen::Playing;
    for mut v in &mut q {
        *v = if on { Visibility::Inherited } else { Visibility::Hidden };
    }
}
