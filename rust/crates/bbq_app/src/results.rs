//! The wording and table of the results card, ported from the browser's `showResults`.

use bbq_core::GameMode;
use bbq_core::heist;
use bbq_core::matchflow::Key;
use bbq_core::teams::Team;
use bevy::prelude::*;

use crate::game::Game;
use crate::menu::Settings;
use crate::player::PLAYER_ID;
use crate::round::name_of;
use crate::ui::*;

pub struct Row {
    pub place: usize,
    pub name: String,
    pub colour: Color,
    pub team: Option<Team>,
    pub score: i32,
    pub hits: u32,
    pub taken: u32,
    pub catches: u32,
    pub accuracy: String,
    pub banked: u32,
    pub me: bool,
}

pub struct ResultsText {
    pub title: String,
    pub line: String,
    pub match_line: Option<String>,
    pub again_label: String,
    pub heist: bool,
    pub rows: Vec<Row>,
}

pub fn colour_of(g: &Game, id: u32) -> Color {
    if id == PLAYER_ID {
        return crate::models::hex(0xffd23f);
    }
    let i = g.dummies.iter().position(|d| d.id == id).unwrap_or(0);
    crate::characters::BLOB_COLOURS[i % crate::characters::BLOB_COLOURS.len()]
}

fn team_name(t: Team) -> String {
    if t == Team::Wildcard {
        "Wildcard".into()
    } else {
        format!("{} team", t.name())
    }
}

fn key_name(g: &Game, k: Option<Key>) -> String {
    match k {
        None => "Nobody".into(),
        Some(Key::Team(t)) => team_name(t),
        Some(Key::Player(id)) => name_of(g, id),
    }
}

fn mine(g: &Game, k: Key) -> bool {
    match k {
        Key::Player(id) => id == PLAYER_ID,
        Key::Team(t) => g.teams.get(PLAYER_ID) == Some(t),
    }
}

fn standings(g: &Game) -> String {
    let mut w: Vec<_> = g.round.mtch.wins.iter().collect();
    w.sort_by_key(|x| std::cmp::Reverse(*x.1));
    if w.is_empty() {
        return "No rounds won yet".into();
    }
    w.iter()
        .map(|(k, v)| {
            let n = match k {
                Key::Player(id) if *id == PLAYER_ID => "You".to_string(),
                other => key_name(g, Some(**other)),
            };
            format!("{n} {v}")
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

pub fn build(g: &Game, _settings: &Settings) -> ResultsText {
    let mut rows: Vec<Row> = g
        .board
        .iter()
        .map(|(id, s)| {
            let teamed = g.rules.mode != GameMode::FreeForAll;
            Row {
                place: 0,
                name: if id == PLAYER_ID { "You".into() } else { name_of(g, id) },
                colour: colour_of(g, id),
                team: g.teams.get(id).filter(|_| teamed),
                score: s.score,
                hits: s.hits,
                taken: s.taken,
                catches: s.catches,
                accuracy: if s.throws > 0 {
                    format!("{}%", (s.hits as f32 / s.throws as f32 * 100.0).round())
                } else {
                    "–".into()
                },
                banked: s.banked,
                me: id == PLAYER_ID,
            }
        })
        .collect();
    rows.sort_by_key(|r| std::cmp::Reverse(r.score));
    for (i, r) in rows.iter_mut().enumerate() {
        r.place = i + 1;
    }
    let winner = g.round.results.as_ref().and_then(|r| r.winner);
    let top = &rows[0];
    let tie = rows.get(1).is_some_and(|r| r.score == top.score);
    let my_rank = rows.iter().position(|r| r.me);

    let (mut title, mut line) = if tie {
        ("Dead heat!".to_string(), "Nobody takes the esky home this time.".to_string())
    } else if top.me {
        (
            "You own the yard!".to_string(),
            format!("Top score of {}. Your mates are going to want a rematch.", top.score),
        )
    } else {
        let l = match my_rank {
            Some(k) => format!(
                "{} finished on {}. You finished #{} on {}.",
                top.name,
                top.score,
                k + 1,
                rows[k].score
            ),
            None => format!("{} finished on {}.", top.name, top.score),
        };
        (format!("{} wins", top.name), l)
    };
    let my_team = g.teams.get(PLAYER_ID);
    match g.rules.mode {
        GameMode::Heist => {
            title = match winner {
                None => "Dead heat!".into(),
                Some(k) if mine(g, k) => "Your team wins!".into(),
                Some(k) => format!("{} wins", key_name(g, Some(k))),
            };
            if let Some(h) = &g.heist {
                let parts: Vec<String> = heist::team_keys(h.teams)
                    .iter()
                    .map(|t| {
                        let best = rows
                            .iter()
                            .filter(|r| r.team == Some(*t) && r.banked > 0)
                            .map(|r| r.banked)
                            .max();
                        let top = best.map(|m| {
                            let names: Vec<String> = rows
                                .iter()
                                .filter(|r| r.team == Some(*t) && r.banked == m)
                                .map(|r| if r.me { "you".to_string() } else { r.name.clone() })
                                .collect();
                            format!(" (top banker {}, {m})", names.join(" & "))
                        });
                        format!("{} {}{}", t.name(), h.bank.get(*t), top.unwrap_or_default())
                    })
                    .collect();
                line = format!("Teddies banked — {}.", parts.join(", "));
            }
        }
        GameMode::Teams => {
            let totals = g.board.team_totals(&g.teams);
            let red = totals.get(&Team::Red).copied().unwrap_or(0);
            let blue = totals.get(&Team::Blue).copied().unwrap_or(0);
            let _ = my_team;
            title = match winner {
                None => "Dead heat!".into(),
                Some(k) if mine(g, k) => "Your team wins!".into(),
                Some(k) => format!("{} wins", key_name(g, Some(k))),
            };
            line = format!("Red {red}, Blue {blue}.");
            if let Some(w) = rows.iter().find(|r| r.team == Some(Team::Wildcard)) {
                let who = if w.me { "you".to_string() } else { w.name.clone() };
                line += &format!(
                    " Wildcard {who} finished on {}{}",
                    w.score,
                    if w.score > red.max(blue) { ", beating both teams on their own!" } else { "." }
                );
            }
        }
        GameMode::FreeForAll => {}
    }
    let m = &g.round.mtch;
    let match_line = (m.len > 1).then(|| {
        let lead = m.leader();
        if m.over {
            title = match lead {
                None => "Match drawn!".into(),
                Some(k) if mine(g, k) => {
                    if matches!(k, Key::Team(_)) { "Your team wins the match!".into() } else { "You win the match!".into() }
                }
                Some(k) => format!("{} wins the match!", key_name(g, Some(k))),
            };
            format!("Final after {} round{}: {}", m.no, if m.no == 1 { "" } else { "s" }, standings(g))
        } else {
            title = match winner {
                None => format!("Round {}: dead heat", m.no),
                Some(k) if mine(g, k) => {
                    format!("Round {} to {}!", m.no, if matches!(k, Key::Team(_)) { "your team" } else { "you" })
                }
                Some(k) => format!("Round {} to {}", m.no, key_name(g, Some(k))),
            };
            format!("Best of {} (first to {}): {}", m.len, m.need(), standings(g))
        }
    });
    let again_label = if m.len > 1 && !m.over {
        format!("Next round ({} of {})", m.no + 1, m.len)
    } else if m.len > 1 {
        "New match".to_string()
    } else {
        "Play again".to_string()
    };
    ResultsText {
        title,
        line,
        match_line,
        again_label,
        heist: g.rules.mode == GameMode::Heist,
        rows,
    }
}

/// The scores table.
pub fn table(c: &mut ChildSpawnerCommands, t: &ResultsText) {
    let mut cols: Vec<(&str, f32)> = vec![
        ("#", 26.0),
        ("Player", 0.0),
        ("Score", 52.0),
        ("Hits", 40.0),
        ("Got hit", 56.0),
        ("Catches", 56.0),
        ("Accuracy", 62.0),
    ];
    if t.heist {
        cols.push(("Banked", 54.0));
    }
    c.spawn(column(0.0)).with_children(|tb| {
        tb.spawn(row(6.0)).with_children(|h| {
            for (name, w) in &cols {
                h.spawn(cell(*w, *name == "Player")).with_child(text(name.to_uppercase(), 10.5, true, MUTED));
            }
        });
        for r in &t.rows {
            tb.spawn((
                Node {
                    width: Val::Percent(100.0),
                    column_gap: px(6.0),
                    padding: UiRect::axes(px(0.0), px(9.0)),
                    border: UiRect::top(px(2.0)),
                    align_items: AlignItems::Center,
                    ..default()
                },
                BorderColor::all(PAPER2),
                BackgroundColor(if r.me { Color::srgb(1.0, 0.965, 0.8) } else { Color::NONE }),
            ))
            .with_children(|row| {
                row.spawn(cell(cols[0].1, false)).with_child(text(r.place.to_string(), 14.0, false, INK));
                row.spawn(Node { flex_grow: 1.0, column_gap: px(7.0), align_items: AlignItems::Center, ..cell_node(0.0, true) })
                    .with_children(|n| {
                        n.spawn((
                            Node {
                                width: px(11.0),
                                height: px(11.0),
                                border: UiRect::all(px(2.0)),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(r.colour),
                            BorderColor::all(INK),
                        ));
                        n.spawn(text(r.name.clone(), 14.0, false, INK));
                        if let Some(tm) = r.team {
                            n.spawn((
                                Node {
                                    padding: UiRect::axes(px(7.0), px(1.0)),
                                    border_radius: BorderRadius::MAX,
                                    ..default()
                                },
                                BackgroundColor(crate::characters::team_colour(tm)),
                                children![text(tm.name(), 11.0, true, WHITE)],
                            ));
                        }
                    });
                let vals = [
                    (r.score.to_string(), true),
                    (r.hits.to_string(), false),
                    (r.taken.to_string(), false),
                    (r.catches.to_string(), false),
                    (r.accuracy.clone(), false),
                ];
                for (i, (v, bold)) in vals.into_iter().enumerate() {
                    row.spawn(cell(cols[i + 2].1, false))
                        .with_child(text(v, if bold { 16.0 } else { 14.0 }, bold, INK));
                }
                if t.heist {
                    row.spawn(cell(54.0, false)).with_child(text(r.banked.to_string(), 16.0, true, INK));
                }
            });
        }
    });
}

fn cell_node(w: f32, left: bool) -> Node {
    Node {
        width: if w > 0.0 { px(w) } else { Val::Auto },
        flex_grow: if w > 0.0 { 0.0 } else { 1.0 },
        justify_content: if left { JustifyContent::FlexStart } else { JustifyContent::FlexEnd },
        ..default()
    }
}

fn cell(w: f32, left: bool) -> impl Bundle {
    cell_node(w, left)
}
