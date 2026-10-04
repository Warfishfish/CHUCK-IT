//! Phase 8: the in-game HUD, laid out like the browser page's: the scoreboard on the left, the
//! clock in a yellow box at the top with the drunk meter under it, the feed on the right, the
//! crosshair with its charge bar, the three cooldown chips bottom left and the two item slots
//! bottom right, plus the big banner, the hint line and the "R does this" prompt.
//!
//! Simplified: the item icons are coloured discs with the item's name (the browser draws small
//! pictures), the red hit flash is a plain tint, and there are no touch buttons.

use bbq_core::GameMode;
use bbq_core::hands::OVERHOLD;
use bbq_core::items::Melee;
use bbq_core::movement::{BOOST_CD, BOOST_TIME};
use bbq_core::scoring::Phase;
use bbq_core::stun::POWER_CHARGE;
use bbq_core::teams::Team;
use bevy::prelude::*;

use crate::game::Game;
use crate::menu::Screen;
use crate::player::{PLAYER_ID, Player};
use crate::ui::*;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Hints>()
            .add_systems(Startup, spawn_hud)
            .add_systems(
                Update,
                (
                    show_hud,
                    update_board,
                    update_clock,
                    update_drunk,
                    update_heist_bar,
                    update_feed,
                    update_crosshair,
                    update_charge,
                    update_cooldowns,
                    update_slots,
                    update_banner,
                    update_hint,
                    update_prompt,
                    update_tints,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
struct HudRoot;
#[derive(Component)]
struct Board;
#[derive(Component)]
struct ClockBox;
#[derive(Component)]
struct ClockText;
#[derive(Component)]
struct RoundTag;
#[derive(Component)]
struct DrunkLabel;
#[derive(Component)]
struct DrunkFill;
#[derive(Component)]
struct HeistBar;
#[derive(Component)]
struct Feed;
#[derive(Component)]
struct HitMark;
#[derive(Component)]
struct CatchRing;
#[derive(Component)]
struct ChargeBox;
#[derive(Component)]
struct ChargeFill;
#[derive(Component)]
struct CdFill(Cd);
#[derive(Clone, Copy, PartialEq)]
enum Cd {
    Boost,
    Catch,
}
#[derive(Component)]
struct Slot(usize);
#[derive(Component)]
struct BannerCopy;
#[derive(Component)]
struct BannerBox;
#[derive(Component)]
struct HintBox;
#[derive(Component)]
struct PromptBox;
#[derive(Component)]
struct HitTint;
#[derive(Component)]
struct PoolTint;

/// The first-time hints (the browser's `hintStage`) and whatever hint is showing.
#[derive(Resource, Default)]
pub struct Hints {
    pub text: String,
    pub secs: f32,
    stage: u8,
    last_phase: Option<Phase>,
}

impl Hints {
    pub fn show(&mut self, text: impl Into<String>, secs: f32) {
        self.text = text.into();
        self.secs = secs;
    }
}

fn abs() -> Node {
    Node {
        position_type: PositionType::Absolute,
        ..default()
    }
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(5),
            Visibility::Hidden,
            HudRoot,
        ))
        .with_children(|root| {
            // the blue pool tint and the red hit tint sit under everything
            root.spawn((
                Node {
                    left: px(0.0),
                    bottom: px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(45.0),
                    ..abs()
                },
                BackgroundColor(Color::srgba(0.157, 0.627, 0.902, 0.0)),
                PoolTint,
            ));
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..abs()
                },
                BackgroundColor(Color::srgba(0.9, 0.12, 0.08, 0.0)),
                HitTint,
            ));
            // scoreboard
            root.spawn((
                Node {
                    left: px(16.0),
                    top: px(16.0),
                    min_width: px(170.0),
                    padding: UiRect::axes(px(10.0), px(8.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(2.0),
                    border_radius: BorderRadius::all(px(12.0)),
                    ..abs()
                },
                BackgroundColor(HUD),
                Board,
            ));
            // clock, drunk meter, heist bar: a column in the middle of the top
            root.spawn(Node {
                left: px(0.0),
                top: px(14.0),
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(8.0),
                ..abs()
            })
            .with_children(|c| {
                c.spawn(row(8.0)).with_children(|r| {
                    r.spawn(Node { flex_grow: 1.0, ..default() });
                    r.spawn((
                        Node {
                            padding: UiRect::axes(px(16.0), px(3.0)),
                            border: UiRect::all(px(3.0)),
                            border_radius: BorderRadius::all(px(14.0)),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        BackgroundColor(SUN),
                        BorderColor::all(INK),
                        BoxShadow(vec![ShadowStyle {
                            color: INK,
                            x_offset: px(4.0),
                            y_offset: px(4.0),
                            spread_radius: px(0.0),
                            blur_radius: px(0.0),
                        }]),
                        ClockBox,
                        children![(
                            Text::new("3:00"),
                            display_font(30.0),
                            TextColor(INK),
                            ClockText,
                        )],
                    ));
                    r.spawn(Node { flex_grow: 1.0, align_items: AlignItems::Center, padding: UiRect::left(px(10.0)), ..default() })
                        .with_children(|t| {
                            t.spawn((
                                Node {
                                    padding: UiRect::axes(px(10.0), px(4.0)),
                                    border_radius: BorderRadius::MAX,
                                    display: Display::None,
                                    ..default()
                                },
                                BackgroundColor(HUD),
                                RoundTag,
                                children![text("", 12.0, true, WHITE)],
                            ));
                        });
                });
                c.spawn((
                    Node {
                        padding: UiRect::new(px(10.0), px(12.0), px(5.0), px(5.0)),
                        column_gap: px(9.0),
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(HUD),
                    children![
                        (text("Sober", 12.0, true, WHITE), DrunkLabel),
                        (
                            Node {
                                width: px(120.0),
                                height: px(8.0),
                                border_radius: BorderRadius::all(px(4.0)),
                                overflow: Overflow::clip(),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.18)),
                            children![(
                                Node {
                                    width: Val::Percent(0.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.48, 0.82, 0.28)),
                                DrunkFill,
                            )],
                        ),
                    ],
                ));
                c.spawn((
                    Node {
                        padding: UiRect::axes(px(12.0), px(5.0)),
                        column_gap: px(11.0),
                        border_radius: BorderRadius::MAX,
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(HUD),
                    HeistBar,
                ));
            });
            // the feed
            root.spawn((
                Node {
                    right: px(16.0),
                    top: px(16.0),
                    max_width: Val::Percent(46.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    row_gap: px(5.0),
                    ..abs()
                },
                Feed,
            ));
            // the crosshair, hit marker and catch ring: a 28 px box kept in the middle
            root.spawn(Node {
                left: px(0.0),
                top: px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..abs()
            })
            .with_children(|m| {
                m.spawn(Node {
                    width: px(28.0),
                    height: px(28.0),
                    ..default()
                })
                .with_children(|x| {
                    let tick = |l: f32, t: f32, w: f32, h: f32| {
                        (
                            Node {
                                left: px(14.0 + l),
                                top: px(14.0 + t),
                                width: px(w),
                                height: px(h),
                                border_radius: BorderRadius::all(px(2.0)),
                                ..abs()
                            },
                            BackgroundColor(WHITE),
                            Outline::new(px(1.5), px(0.0), Color::srgba(0.0, 0.0, 0.0, 0.45)),
                        )
                    };
                    x.spawn(tick(-1.5, -14.0, 3.0, 8.0));
                    x.spawn(tick(-1.5, 6.0, 3.0, 8.0));
                    x.spawn(tick(-14.0, -1.5, 8.0, 3.0));
                    x.spawn(tick(6.0, -1.5, 8.0, 3.0));
                    for deg in [45.0f32, -45.0] {
                        x.spawn((
                            Node {
                                left: px(12.0),
                                top: px(-1.0),
                                width: px(4.0),
                                height: px(30.0),
                                border_radius: BorderRadius::all(px(2.0)),
                                ..abs()
                            },
                            UiTransform::from_rotation(Rot2::degrees(deg)),
                            BackgroundColor(TOMATO),
                            Visibility::Hidden,
                            HitMark,
                        ));
                    }
                    x.spawn((
                        Node {
                            left: px(-12.0),
                            top: px(-12.0),
                            width: px(52.0),
                            height: px(52.0),
                            border: UiRect::all(px(4.0)),
                            border_radius: BorderRadius::MAX,
                            ..abs()
                        },
                        BorderColor::all(Color::srgb(0.56, 0.96, 0.9)),
                        Visibility::Hidden,
                        CatchRing,
                    ));
                });
            });
            // the charge bar under the crosshair
            root.spawn(Node {
                left: px(0.0),
                top: Val::Percent(50.0),
                margin: UiRect::top(px(30.0)),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                ..abs()
            })
            .with_children(|c| {
                c.spawn((
                    Node {
                        width: px(110.0),
                        height: px(9.0),
                        border_radius: BorderRadius::all(px(5.0)),
                        overflow: Overflow::clip(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                    Visibility::Hidden,
                    ChargeBox,
                    children![
                        (
                            Node {
                                width: Val::Percent(0.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(SUN),
                            ChargeFill,
                        ),
                        (
                            Node {
                                left: Val::Percent(90.0),
                                top: px(0.0),
                                bottom: px(0.0),
                                width: px(2.0),
                                ..abs()
                            },
                            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.75)),
                        ),
                    ],
                ));
            });
            // cooldown chips, bottom left
            root.spawn(Node {
                left: px(16.0),
                bottom: px(16.0),
                column_gap: px(8.0),
                ..abs()
            })
            .with_children(|c| {
                cd_chip(c, "Space", "Jump", None);
                cd_chip(c, "Shift", "Boost", Some(Cd::Boost));
                cd_chip(c, "Right click", "Catch", Some(Cd::Catch));
            });
            // item slots, bottom right
            root.spawn(Node {
                right: px(16.0),
                bottom: px(16.0),
                column_gap: px(8.0),
                ..abs()
            })
            .with_children(|s| {
                for i in 0..2 {
                    s.spawn((
                        Node {
                            width: px(86.0),
                            height: px(86.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            row_gap: px(2.0),
                            border: UiRect::all(px(3.0)),
                            border_radius: BorderRadius::all(px(14.0)),
                            ..default()
                        },
                        BackgroundColor(HUD),
                        BorderColor::all(Color::NONE),
                        Slot(i),
                    ));
                }
            });
            // the big banner
            root.spawn((
                Node {
                    left: px(0.0),
                    top: Val::Percent(30.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..abs()
                },
                Visibility::Hidden,
                BannerBox,
            ))
            .with_children(|b| {
                b.spawn(Node::default()).with_children(|t| {
                    let size = 88.0;
                    let o = 3.0;
                    for (dx, dy) in [(5.0, 6.0)] {
                        t.spawn(banner_copy(size, INK, dx, dy));
                    }
                    for (dx, dy) in [(-o, 0.0), (o, 0.0), (0.0, -o), (0.0, o), (-o, -o), (o, -o), (-o, o), (o, o)] {
                        t.spawn(banner_copy(size, INK, dx, dy));
                    }
                    t.spawn((Text::new(""), display_font(size), TextColor(SUN), BannerCopy, Node::default()));
                });
            });
            // hint and prompt pills
            root.spawn((
                Node {
                    left: px(0.0),
                    bottom: px(120.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..abs()
                },
                Visibility::Hidden,
                HintBox,
            ));
            root.spawn((
                Node {
                    left: px(0.0),
                    bottom: px(170.0),
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    ..abs()
                },
                Visibility::Hidden,
                PromptBox,
            ));
        });
}

fn banner_copy(size: f32, c: Color, dx: f32, dy: f32) -> impl Bundle {
    (
        Text::new(""),
        display_font(size),
        TextColor(c),
        BannerCopy,
        Node {
            left: px(dx),
            top: px(dy),
            ..abs()
        },
    )
}

fn cd_chip(p: &mut ChildSpawnerCommands, key: &str, label: &str, fill: Option<Cd>) {
    p.spawn((
        Node {
            padding: UiRect::axes(px(12.0), px(8.0)),
            column_gap: px(8.0),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(11.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(HUD),
    ))
    .with_children(|c| {
        c.spawn(kbd(key));
        c.spawn(text(label, 13.0, false, WHITE));
        if let Some(f) = fill {
            c.spawn((
                Node {
                    left: px(0.0),
                    bottom: px(0.0),
                    height: px(4.0),
                    width: Val::Percent(100.0),
                    ..abs()
                },
                BackgroundColor(SUN),
                CdFill(f),
            ));
        }
    });
}

// ---------------------------------------------------------------- showing and updating

fn show_hud(screen: Res<Screen>, mut q: Query<&mut Visibility, With<HudRoot>>) {
    for mut v in &mut q {
        *v = if matches!(*screen, Screen::Playing | Screen::Paused) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn update_board(
    mut commands: Commands,
    game: Res<Game>,
    board: Single<Entity, With<Board>>,
    mut last: Local<String>,
) {
    let g = &*game;
    let teamed = g.rules.mode != GameMode::FreeForAll;
    let mut rows: Vec<(u32, String, Color, i32, Option<Team>, u32)> = g
        .board
        .iter()
        .map(|(id, s)| {
            (
                id,
                if id == PLAYER_ID { "You".to_string() } else { crate::round::name_of(g, id) },
                crate::results::colour_of(g, id),
                s.score,
                g.teams.get(id).filter(|_| teamed),
                s.banked,
            )
        })
        .collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.3));
    let leader = g.board.leader();
    let sig = format!(
        "{:?}|{:?}",
        rows.iter().map(|r| (r.0, r.3, r.4, r.5)).collect::<Vec<_>>(),
        (leader, g.heist.as_ref().map(|h| heist_sig(h)))
    );
    if *last == sig {
        return;
    }
    *last = sig;
    commands.entity(*board).despawn_children();
    commands.entity(*board).with_children(|b| {
        let row_of = |b: &mut ChildSpawnerCommands, r: &(u32, String, Color, i32, Option<Team>, u32)| {
            b.spawn((
                Node {
                    column_gap: px(8.0),
                    padding: UiRect::axes(px(4.0), px(3.0)),
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(px(7.0)),
                    ..default()
                },
                BackgroundColor(if r.0 == PLAYER_ID { Color::srgba(1.0, 0.824, 0.247, 0.22) } else { Color::NONE }),
            ))
            .with_children(|row| {
                row.spawn((
                    Node {
                        width: px(12.0),
                        height: px(12.0),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(r.2),
                    BorderColor::all(Color::srgba(0.0, 0.0, 0.0, 0.4)),
                ));
                row.spawn(Node { flex_grow: 1.0, column_gap: px(4.0), ..default() })
                    .with_children(|n| {
                        n.spawn(text(r.1.clone(), 14.0, false, WHITE));
                        if leader == Some(r.0) {
                            n.spawn(text("*", 14.0, true, SUN));
                        }
                    });
                let score = if g.rules.mode == GameMode::Heist {
                    format!("{} · {}", r.5, r.3)
                } else {
                    r.3.to_string()
                };
                row.spawn(text(score, 14.0, false, WHITE));
            });
        };
        if !teamed {
            for r in &rows {
                row_of(b, r);
            }
            return;
        }
        let order = [Team::Red, Team::Blue, Team::Green, Team::Yellow, Team::Wildcard];
        for t in order {
            let grp: Vec<_> = rows.iter().filter(|r| r.4 == Some(t)).collect();
            if grp.is_empty() {
                continue;
            }
            let total: i32 = grp.iter().map(|r| r.3).sum();
            let mine = g.teams.get(PLAYER_ID) == Some(t);
            let name = if t == Team::Wildcard {
                "Wildcard".to_string()
            } else {
                format!("{} team", t.name())
            };
            let right = if t == Team::Wildcard {
                String::new()
            } else if g.rules.mode == GameMode::Heist {
                g.heist.as_ref().map_or(0, |h| h.bank.get(t)).to_string()
            } else {
                total.to_string()
            };
            b.spawn((
                Node {
                    padding: UiRect::axes(px(8.0), px(3.0)),
                    margin: UiRect::top(px(4.0)),
                    justify_content: JustifyContent::SpaceBetween,
                    border_radius: BorderRadius::all(px(7.0)),
                    ..default()
                },
                BackgroundColor(crate::characters::team_colour(t)),
                children![
                    text(format!("{name}{}", if mine { " (you)" } else { "" }), 13.0, true, WHITE),
                    text(right, 13.0, true, WHITE),
                ],
            ));
            for r in grp {
                row_of(b, r);
            }
        }
    });
}

fn heist_sig(h: &crate::heist_app::HeistState) -> Vec<u32> {
    bbq_core::heist::team_keys(h.teams).iter().map(|t| h.bank.get(*t)).collect()
}

fn update_clock(
    game: Res<Game>,
    time: Res<Time>,
    mut boxq: Query<(&mut BackgroundColor, &mut UiTransform), With<ClockBox>>,
    mut txt: Query<(&mut Text, &mut TextColor), With<ClockText>>,
    mut tag: Query<(&mut Node, &Children), With<RoundTag>>,
    mut tag_text: Query<&mut Text, Without<ClockText>>,
) {
    let g = &*game;
    let shown = if g.round.timed {
        if g.rules.phase == Phase::Countdown { g.round.setup.round_len } else { g.round.time_left }
    } else {
        g.round.setup.round_len
    };
    let secs = shown.max(0.0).ceil() as i32;
    let low = g.round.timed && g.rules.phase == Phase::Play && g.round.time_left <= 10.0;
    for (mut t, mut c) in &mut txt {
        let s = format!("{}:{:02}", secs / 60, secs % 60);
        if t.0 != s {
            t.0 = s;
        }
        c.0 = if low { WHITE } else { INK };
    }
    for (mut bg, mut tf) in &mut boxq {
        bg.0 = if low { TOMATO } else { SUN };
        let pulse = if low { 1.04 + 0.04 * (time.elapsed_secs() * 12.0).sin() } else { 1.0 };
        tf.scale = Vec2::splat(pulse);
    }
    let m = &g.round.mtch;
    let show = m.len > 1 && m.no > 0;
    for (mut n, kids) in &mut tag {
        n.display = if show { Display::Flex } else { Display::None };
        for k in kids.iter() {
            if let Ok(mut t) = tag_text.get_mut(k) {
                t.0 = format!("Round {}/{}", m.no, m.len);
            }
        }
    }
}

fn update_drunk(
    game: Res<Game>,
    mut label: Query<&mut Text, With<DrunkLabel>>,
    mut fill: Query<(&mut Node, &mut BackgroundColor), With<DrunkFill>>,
) {
    let d = &game.me.drunk;
    let name = match d.tier() {
        bbq_core::drinks::Tier::Sober => "Sober",
        bbq_core::drinks::Tier::Tipsy => "Tipsy",
        bbq_core::drinks::Tier::Drunk => "Drunk",
        bbq_core::drinks::Tier::Maggot => "Maggot",
        bbq_core::drinks::Tier::AbsolutelyMaggoted => "Absolutely maggoted",
    };
    for mut t in &mut label {
        if t.0 != name {
            t.0 = name.to_string();
        }
    }
    let lvl = d.meter.clamp(0.0, 100.0);
    for (mut n, mut bg) in &mut fill {
        n.width = Val::Percent(lvl);
        bg.0 = Color::hsl((110.0 - lvl * 1.1).max(0.0), 0.75, 0.52);
    }
}

fn update_heist_bar(
    mut commands: Commands,
    game: Res<Game>,
    mut q: Query<(Entity, &mut Node), With<HeistBar>>,
    mut last: Local<Vec<u32>>,
) {
    let Ok((e, mut n)) = q.single_mut() else { return };
    let Some(h) = game.heist.as_ref().filter(|_| game.rules.mode == GameMode::Heist) else {
        n.display = Display::None;
        last.clear();
        return;
    };
    let sig = heist_sig(h);
    if *last == sig && n.display == Display::Flex {
        return;
    }
    *last = sig;
    n.display = Display::Flex;
    commands.entity(e).despawn_children();
    commands.entity(e).with_children(|b| {
        for t in bbq_core::heist::team_keys(h.teams) {
            b.spawn(Node { column_gap: px(5.0), align_items: AlignItems::Center, ..default() })
                .with_children(|s| {
                    s.spawn((
                        Node {
                            width: px(10.0),
                            height: px(10.0),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(crate::characters::team_colour(*t)),
                    ));
                    s.spawn(text(h.bank.get(*t).to_string(), 13.0, true, WHITE));
                });
        }
    });
}

fn update_feed(
    mut commands: Commands,
    game: Res<Game>,
    feed: Single<Entity, With<Feed>>,
    mut last: Local<Vec<String>>,
) {
    let g = &*game;
    let lines: Vec<String> = g
        .feed
        .iter()
        .rev()
        .take(5)
        .filter(|(_, t)| g.now - *t < 6.0)
        .map(|(s, _)| s.clone())
        .collect();
    if *last == lines {
        return;
    }
    *last = lines.clone();
    commands.entity(*feed).despawn_children();
    commands.entity(*feed).with_children(|f| {
        for l in lines {
            f.spawn((
                Node {
                    padding: UiRect::axes(px(10.0), px(5.0)),
                    border_radius: BorderRadius::all(px(9.0)),
                    ..default()
                },
                BackgroundColor(HUD),
                children![text(l, 13.0, false, WHITE)],
            ));
        }
    });
}

fn update_crosshair(
    game: Res<Game>,
    mut hit: Query<&mut Visibility, (With<HitMark>, Without<CatchRing>)>,
    mut ring: Query<&mut Visibility, (With<CatchRing>, Without<HitMark>)>,
    mut last_hits: Local<u32>,
    mut flash: Local<f32>,
    time: Res<Time>,
) {
    let hits = game.board.get(PLAYER_ID).map_or(0, |s| s.hits);
    if hits > *last_hits {
        *flash = 0.15;
    }
    *last_hits = hits;
    *flash = (*flash - time.delta_secs()).max(0.0);
    for mut v in &mut hit {
        *v = if *flash > 0.0 { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut v in &mut ring {
        *v = if game.catcher.open() { Visibility::Inherited } else { Visibility::Hidden };
    }
}

fn update_charge(
    game: Res<Game>,
    time: Res<Time>,
    mut boxq: Query<&mut Visibility, With<ChargeBox>>,
    mut fill: Query<(&mut Node, &mut BackgroundColor), With<ChargeFill>>,
) {
    let w = &game.wind;
    for mut v in &mut boxq {
        *v = if w.charging { Visibility::Inherited } else { Visibility::Hidden };
    }
    for (mut n, mut bg) in &mut fill {
        n.width = Val::Percent(w.charge.clamp(0.0, 1.0) * 100.0);
        let hot = w.charge >= POWER_CHARGE;
        let blink = w.over_t > OVERHOLD * 0.35 && (time.elapsed_secs() * 6.0) as i32 % 2 == 0;
        bg.0 = (if hot { TOMATO } else { SUN }).with_alpha(if blink { 0.15 } else { 1.0 });
    }
}

fn update_cooldowns(
    game: Res<Game>,
    player: Res<Player>,
    mut q: Query<(&CdFill, &mut Node, &mut BackgroundColor)>,
) {
    let m = &player.mover;
    for (c, mut n, mut bg) in &mut q {
        match c.0 {
            Cd::Boost => {
                let r = if m.boost_t > 0.0 { m.boost_t / BOOST_TIME } else { 1.0 - m.boost_cd / BOOST_CD };
                n.width = Val::Percent(r.clamp(0.0, 1.0) * 100.0);
                bg.0 = if m.boost_t > 0.0 {
                    Color::srgb(0.07, 0.65, 0.58)
                } else if m.boost_cd > 0.0 {
                    TOMATO
                } else {
                    SUN
                };
            }
            Cd::Catch => {
                n.width = Val::Percent((100.0 - game.catcher.cooldown / 0.9 * 100.0).clamp(0.0, 100.0));
            }
        }
    }
}

fn item_colour(kind: bbq_core::items::ItemKind) -> Color {
    use bbq_core::items::ItemKind::*;
    crate::models::hex(match kind {
        Teddy => 0xa86b3c,
        Stubby => 0x1e7d3a,
        Gnome => 0xd63a2f,
        Dildo => 0xa44de0,
        Steak => 0xc4455a,
        Fish => 0xa9bcc8,
        Noodle => 0xff5fa8,
        Snag => 0xc4455a,
    })
}

fn update_slots(
    mut commands: Commands,
    game: Res<Game>,
    slots: Query<(Entity, &Slot, &mut BorderColor)>,
    mut last: Local<String>,
) {
    let g = &*game;
    let ids = g.slots.ids();
    let sel = g.slots.selected();
    let state: Vec<_> = ids
        .iter()
        .map(|id| g.world.items.get(id).map(|i| (*id, i.kind, i.uses)))
        .collect();
    let sig = format!("{state:?}|{sel:?}");
    if *last == sig {
        return;
    }
    *last = sig;
    for (e, s, mut border) in slots {
        let item = ids.get(s.0).and_then(|id| g.world.items.get(id));
        *border = BorderColor::all(if item.is_some() && ids.get(s.0).copied() == sel { SUN } else { Color::NONE });
        commands.entity(e).despawn_children();
        commands.entity(e).with_children(|p| {
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(4.0),
                    top: px(4.0),
                    ..default()
                },
                children![kbd(&(s.0 + 1).to_string())],
            ));
            match item {
                Some(it) => {
                    p.spawn((
                        Node {
                            width: px(34.0),
                            height: px(34.0),
                            border: UiRect::all(px(2.0)),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(item_colour(it.kind)),
                        BorderColor::all(INK),
                    ));
                    let def = it.kind.def();
                    let name = crate::items_view::kind_name(it.kind);
                    let label = if def.melee != Melee::None {
                        let u = it.uses.unwrap_or(3);
                        format!("{name} · {u} slap{}", if u == 1 { "" } else { "s" })
                    } else {
                        name.to_string()
                    };
                    p.spawn(text(label, 11.0, false, WHITE));
                }
                None => {
                    p.spawn(text("Empty", 11.0, false, Color::srgba(1.0, 1.0, 1.0, 0.45)));
                }
            }
        });
    }
}

fn update_banner(
    game: Res<Game>,
    mut boxq: Query<&mut Visibility, With<BannerBox>>,
    mut copies: Query<&mut Text, With<BannerCopy>>,
) {
    let b = game.round.banner.as_ref();
    for mut v in &mut boxq {
        *v = if b.is_some() { Visibility::Inherited } else { Visibility::Hidden };
    }
    if let Some((s, _)) = b {
        for mut t in &mut copies {
            if t.0 != *s {
                t.0 = s.clone();
            }
        }
    }
}

/// Split "Hold <b>left click</b> to ..." into plain and bold parts.
fn rich_parts(s: &str) -> Vec<(String, bool)> {
    let mut v = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find("<b>") {
        if i > 0 {
            v.push((rest[..i].to_string(), false));
        }
        rest = &rest[i + 3..];
        match rest.find("</b>") {
            Some(j) => {
                v.push((rest[..j].to_string(), true));
                rest = &rest[j + 4..];
            }
            None => {
                v.push((rest.to_string(), true));
                rest = "";
            }
        }
    }
    if !rest.is_empty() {
        v.push((rest.to_string(), false));
    }
    v
}

fn pill(commands: &mut Commands, holder: Entity, markup: &str, size: f32) {
    commands.entity(holder).despawn_children();
    if markup.is_empty() {
        return;
    }
    commands.entity(holder).with_children(|h| {
        h.spawn((
            Node {
                padding: UiRect::axes(px(14.0), px(9.0)),
                border_radius: BorderRadius::all(px(11.0)),
                max_width: Val::Percent(90.0),
                ..default()
            },
            BackgroundColor(HUD),
            Text::new(""),
            body_font(size, false),
            TextColor(WHITE),
            TextLayout::justify(Justify::Center),
        ))
        .with_children(|t| {
            for (s, bold) in rich_parts(markup) {
                t.spawn((
                    TextSpan::new(s),
                    body_font(size, bold),
                    TextColor(if bold { SUN } else { WHITE }),
                ));
            }
        });
    });
}

fn update_hint(
    mut commands: Commands,
    game: Res<Game>,
    time: Res<Time>,
    mut hints: ResMut<Hints>,
    holder: Single<(Entity, &mut Visibility), With<HintBox>>,
    mut shown: Local<String>,
) {
    let g = &*game;
    let dt = time.delta_secs();
    let (e, mut vis) = holder.into_inner();
    // a new round: the team line, and the first-time hints start again unless all were seen
    let phase = g.rules.phase;
    let prev = hints.last_phase.replace(phase);
    if g.round.timed && prev == Some(Phase::Countdown) && phase == Phase::Play {
        if let Some(t) = g.teams.get(PLAYER_ID).filter(|_| g.rules.mode != GameMode::FreeForAll) {
            let line = if t == Team::Wildcard {
                "You're the <b>Wildcard</b>. Everyone is fair game, and everyone is after you.".to_string()
            } else {
                format!(
                    "You're on the <b>{}</b> team. Get the other lot{}.",
                    t.name(),
                    if g.rules.friendly_fire { ". Friendly fire is on, so watch your aim" } else { "" }
                )
            };
            hints.show(line, 5.0);
        }
    }
    if prev != Some(Phase::Countdown) && phase == Phase::Countdown && hints.stage < 3 {
        hints.stage = 0;
    }
    // first-time hints
    if hints.stage == 0 && !g.slots.ids().is_empty() {
        hints.stage = 1;
    }
    if hints.stage == 1 && g.board.get(PLAYER_ID).is_some_and(|s| s.throws > 0) {
        hints.stage = 2;
    }
    let can_act = g.rules.phase == Phase::Play || !g.round.timed;
    if hints.secs <= 0.0 && can_act && !g.attract {
        match hints.stage {
            0 => hints.show("Walk over anything <b>glowing</b> on the lawn to pick it up.", 1.0),
            1 => hints.show("Hold <b>left click</b> to wind up, let go to chuck it.", 1.0),
            2 => {
                hints.show(
                    "<b>Shift</b> for a speed boost, <b>Space</b> to jump. <b>Right click</b> just before a hit to catch it.",
                    7.0,
                );
                hints.stage = 3;
            }
            _ => {}
        }
    }
    if hints.secs > 0.0 {
        hints.secs -= dt;
    }
    let want = if hints.secs > 0.0 { hints.text.clone() } else { String::new() };
    if *shown != want {
        *shown = want.clone();
        pill(&mut commands, e, &want, 14.0);
    }
    *vis = if want.is_empty() { Visibility::Hidden } else { Visibility::Inherited };
}

fn update_prompt(
    mut commands: Commands,
    game: Res<Game>,
    holder: Single<(Entity, &mut Visibility), With<PromptBox>>,
    mut shown: Local<String>,
) {
    let (e, mut vis) = holder.into_inner();
    let want = game.prompt.clone();
    if *shown != want {
        *shown = want.clone();
        pill(&mut commands, e, &want, 15.0);
    }
    *vis = if want.is_empty() { Visibility::Hidden } else { Visibility::Inherited };
}

fn update_tints(
    game: Res<Game>,
    player: Res<Player>,
    time: Res<Time>,
    mut hit: Query<&mut BackgroundColor, (With<HitTint>, Without<PoolTint>)>,
    mut pool: Query<&mut BackgroundColor, (With<PoolTint>, Without<HitTint>)>,
    mut vignette: Local<f32>,
    mut last_taken: Local<u32>,
) {
    let taken = game.board.get(PLAYER_ID).map_or(0, |s| s.taken);
    if taken > *last_taken {
        *vignette = 0.28;
    }
    *last_taken = taken;
    *vignette = (*vignette - time.delta_secs() * 0.6).max(0.0);
    for mut bg in &mut hit {
        bg.0 = Color::srgba(0.9, 0.12, 0.08, *vignette);
    }
    for mut bg in &mut pool {
        bg.0 = Color::srgba(0.157, 0.627, 0.902, if player.mover.in_pool { 0.28 } else { 0.0 });
    }
}
