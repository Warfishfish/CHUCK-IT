//! The look of the menus, ported from the browser page's CSS: cream cards with a thick dark
//! outline and a hard shadow, yellow "selected" buttons, a red Play button, pill-shaped chips.
//! The widgets here only draw and report clicks; what they mean lives in `menu.rs`.
//!
//! Fonts: Bowlby One (display) and Figtree (text), the same as the browser page. Both are free
//! under the SIL Open Font Licence (copies in `assets/fonts/`, from the Google Fonts repository).
//! `FontsPlugin` loads the files so the names below resolve.

use bevy::prelude::*;
use bevy::text::{FontSource, FontWeight};
use bevy::ui::RelativeCursorPosition;

pub const INK: Color = Color::srgb(0.090, 0.141, 0.094);
pub const PAPER: Color = Color::srgb(0.957, 0.976, 0.937);
pub const PAPER2: Color = Color::srgb(0.894, 0.933, 0.863);
pub const SUN: Color = Color::srgb(1.0, 0.824, 0.247);
pub const ESKY: Color = Color::srgb(0.122, 0.435, 0.820);
pub const TOMATO: Color = Color::srgb(1.0, 0.302, 0.239);
pub const MUTED: Color = Color::srgb(0.310, 0.376, 0.298);
pub const WHITE: Color = Color::WHITE;
pub const HUD: Color = Color::srgba(0.071, 0.125, 0.078, 0.74);
pub const GOOD: Color = Color::srgb(0.788, 0.949, 0.753);
pub const BAD: Color = Color::srgb(1.0, 0.816, 0.792);

/// Display text (the big yellow title, buttons, headings).
pub fn display_font(size: f32) -> TextFont {
    TextFont {
        font: FontSource::Family("Bowlby One".into()),
        font_size: FontSize::Px(size),
        ..default()
    }
}

/// Normal text; `bold` is the browser's weight 800.
pub fn body_font(size: f32, bold: bool) -> TextFont {
    TextFont {
        font: FontSource::Family("Figtree".into()),
        font_size: FontSize::Px(size),
        weight: if bold { FontWeight(800) } else { FontWeight(500) },
        ..default()
    }
}

/// A plain line of text.
pub fn text(s: impl Into<String>, size: f32, bold: bool, colour: Color) -> impl Bundle {
    (Text::new(s), body_font(size, bold), TextColor(colour))
}

/// A cream card with a dark outline and a hard shadow.
pub fn card(width: f32) -> impl Bundle {
    (
        Node {
            width: px(width),
            max_width: Val::Percent(100.0),
            max_height: Val::Percent(100.0),
            flex_shrink: 0.0,
            flex_direction: FlexDirection::Column,
            row_gap: px(16.0),
            padding: UiRect::axes(px(22.0), px(20.0)),
            border: UiRect::all(px(3.0)),
            border_radius: BorderRadius::all(px(18.0)),
            overflow: Overflow::scroll_y(),
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
        RelativeCursorPosition::default(),
        ScrollPosition::default(),
        Interaction::default(),
        bevy::ui::FocusPolicy::Block,
    )
}

/// Full-screen holder for a card (the page's `.overlay`).
pub fn overlay(centre: bool) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(px(20.0)),
            align_items: if centre { AlignItems::Center } else { AlignItems::FlexStart },
            justify_content: if centre { JustifyContent::Center } else { JustifyContent::FlexStart },
            ..default()
        },
        GlobalZIndex(20),
    )
}

/// A column of things with a gap.
pub fn column(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        row_gap: px(gap),
        width: Val::Percent(100.0),
        ..default()
    }
}

pub fn row(gap: f32) -> Node {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: px(gap),
        width: Val::Percent(100.0),
        ..default()
    }
}

/// The small blue "Backyard Brawl" pill.
pub fn tag_pill(s: &str) -> impl Bundle {
    (
        Node {
            padding: UiRect::axes(px(10.0), px(5.0)),
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(ESKY),
        BorderColor::all(INK),
        UiTransform::from_rotation(Rot2::degrees(1.5)),
        children![text(s.to_uppercase(), 11.0, true, WHITE)],
    )
}

/// The big yellow title with a dark outline and drop shadow: dark copies behind, yellow in front.
pub fn outlined_title(p: &mut ChildSpawnerCommands, s: &str, size: f32) {
    p.spawn((
        Node {
            margin: UiRect::new(px(2.0), px(0.0), px(8.0), px(2.0)),
            ..default()
        },
        UiTransform::from_rotation(Rot2::degrees(-4.0)),
    ))
    .with_children(|t| {
        let o = 2.5;
        // the drop shadow, then the outline, then the yellow
        for (dx, dy) in [(4.0, 5.0)] {
            t.spawn(title_text(s, size, INK, dx, dy));
        }
        for (dx, dy) in [
            (-o, 0.0),
            (o, 0.0),
            (0.0, -o),
            (0.0, o),
            (-o, -o),
            (o, -o),
            (-o, o),
            (o, o),
        ] {
            t.spawn(title_text(s, size, INK, dx, dy));
        }
        t.spawn((
            Text::new(s),
            display_font(size),
            TextColor(SUN),
            Node::default(),
        ));
    });
}

fn title_text(s: &str, size: f32, c: Color, dx: f32, dy: f32) -> impl Bundle {
    (
        Text::new(s),
        display_font(size),
        TextColor(c),
        Node {
            position_type: PositionType::Absolute,
            left: px(dx),
            top: px(dy),
            ..default()
        },
    )
}

// ---------------------------------------------------------------- the pieces menus are made of

/// What a clickable thing does. `menu.rs` reads these.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Tab(Tab),
    Play,
    More,
    Resume,
    Quit,
    Again,
    ToMenu,
    FocusName,
    HostYard,
    Join,
    Leave,
    FocusServer,
    FocusRoom,
    /// The Customise tab's arrows: which feature, and which way.
    LookStep(LookRow, i32),
    /// Customise: a random look, or back to the default.
    LookRandom,
    LookReset,
}

/// One row on the Customise tab.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum LookRow {
    Mouth,
    Brows,
    BrowColour,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Tab {
    Solo,
    Look,
    Mates,
    How,
}

/// One option of a segmented control (the page's `.seg`).
#[derive(Component, Clone, Copy, Debug)]
pub struct SegOption {
    pub id: SegId,
    pub value: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SegId {
    Bots,
    Mode,
    HeistTeams,
    RoundLen,
    Skill,
    Rounds,
    FriendlyFire,
    Character,
}

/// A tick-box chip (the page's `.feat label`).
#[derive(Component, Clone, Copy, Debug)]
pub struct CheckBox(pub CheckId);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CheckId {
    Cheeky,
    Bar,
    Bbq,
    Chest,
    Smoko,
    Naughty,
    Falls,
    DrunkAll,
    Sound,
}

/// Which setting a slider changes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SliderId {
    Fov,
    Belly,
}

/// A slider (field of view, beer belly).
#[derive(Component)]
pub struct Slider {
    pub id: SliderId,
    pub min: f32,
    pub max: f32,
}

#[derive(Component)]
pub struct SliderFill(pub SliderId);
#[derive(Component)]
pub struct SliderValue(pub SliderId);

/// A bold heading above a group (the page's `legend`).
pub fn legend(s: &str) -> impl Bundle {
    text(s.to_uppercase(), 11.0, true, MUTED)
}

/// A segmented radio row: the chosen one is yellow.
pub fn seg(
    p: &mut ChildSpawnerCommands,
    id: SegId,
    title: &str,
    opts: &[(&str, i32)],
) -> Entity {
    let mut group = Entity::PLACEHOLDER;
    p.spawn(column(6.0)).with_children(|g| {
        g.spawn(legend(title));
        group = g
            .spawn((
                Node {
                    width: Val::Percent(100.0),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::all(px(12.0)),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(WHITE),
                BorderColor::all(INK),
            ))
            .with_children(|row| {
                for (i, (label, value)) in opts.iter().enumerate() {
                    row.spawn((
                        Button,
                        Node {
                            flex_grow: 1.0,
                            flex_basis: px(0.0),
                            padding: UiRect::axes(px(6.0), px(9.0)),
                            justify_content: JustifyContent::Center,
                            border: UiRect::left(px(if i == 0 { 0.0 } else { 2.0 })),
                            ..default()
                        },
                        BorderColor::all(INK),
                        BackgroundColor(WHITE),
                        SegOption { id, value: *value },
                        children![text(*label, 14.0, true, INK)],
                    ));
                }
            })
            .id();
    });
    group
}

/// A tick-box chip. Its `Node` row is wrapped by the caller.
pub fn check(p: &mut ChildSpawnerCommands, id: CheckId, label: &str) -> Entity {
    p.spawn((
        Button,
        Node {
            padding: UiRect::axes(px(12.0), px(6.0)),
            column_gap: px(6.0),
            align_items: AlignItems::Center,
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(WHITE),
        BorderColor::all(INK),
        CheckBox(id),
        children![
            (
                Node {
                    width: px(14.0),
                    height: px(14.0),
                    border: UiRect::all(px(2.0)),
                    border_radius: BorderRadius::all(px(3.0)),
                    ..default()
                },
                BorderColor::all(INK),
                BackgroundColor(WHITE),
                CheckMark,
            ),
            text(label, 13.0, true, INK),
        ],
    ))
    .id()
}

#[derive(Component)]
pub struct CheckMark;

/// A chunky button. `primary` is the big red/yellow one.
pub fn button(
    p: &mut ChildSpawnerCommands,
    action: Action,
    label: &str,
    primary: Option<Color>,
) -> Entity {
    let big = primary.is_some();
    let bg = primary.unwrap_or(WHITE);
    let fg = if bg == TOMATO { WHITE } else { INK };
    p.spawn((
        Button,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0.0),
            padding: UiRect::axes(px(18.0), px(if big { 15.0 } else { 12.0 })),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(px(3.0)),
            border_radius: BorderRadius::all(px(if big { 16.0 } else { 14.0 })),
            ..default()
        },
        BackgroundColor(bg),
        BorderColor::all(INK),
        BoxShadow(vec![ShadowStyle {
            color: INK,
            x_offset: px(4.0),
            y_offset: px(4.0),
            spread_radius: px(0.0),
            blur_radius: px(0.0),
        }]),
        action,
        ButtonLook { base: bg },
        children![(
            Text::new(label),
            if big { display_font(21.0) } else { body_font(15.0, true) },
            TextColor(fg),
        )],
    ))
    .id()
}

/// Remembers a button's resting colour so hover and press can nudge it.
#[derive(Component)]
pub struct ButtonLook {
    pub base: Color,
}

/// The wide tab buttons (Solo / With mates / How to play).
pub fn tab_button(p: &mut ChildSpawnerCommands, tab: Tab, label: &str) {
    p.spawn((
        Button,
        Node {
            flex_grow: 1.0,
            flex_basis: px(0.0),
            padding: UiRect::axes(px(4.0), px(11.0)),
            justify_content: JustifyContent::Center,
            border: UiRect::all(px(3.0)),
            border_radius: BorderRadius::all(px(14.0)),
            ..default()
        },
        BackgroundColor(WHITE),
        BorderColor::all(INK),
        Action::Tab(tab),
        TabButton(tab),
        children![text(label, 14.0, true, INK)],
    ));
}

#[derive(Component)]
pub struct TabButton(pub Tab);

/// A labelled slider.
pub fn slider(p: &mut ChildSpawnerCommands, id: SliderId, title: &str, min: f32, max: f32) {
    p.spawn(column(6.0)).with_children(|c| {
        c.spawn(row(8.0)).with_children(|r| {
            r.spawn(legend(title));
            r.spawn((text("", 11.0, true, INK), SliderValue(id)));
        });
        c.spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: px(22.0),
                align_items: AlignItems::Center,
                ..default()
            },
            RelativeCursorPosition::default(),
            Slider { id, min, max },
            children![
                (
                    Node {
                        width: Val::Percent(100.0),
                        height: px(8.0),
                        border: UiRect::all(px(2.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(PAPER2),
                    BorderColor::all(INK),
                ),
                (
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        width: px(20.0),
                        height: px(20.0),
                        margin: UiRect::left(px(-10.0)),
                        border: UiRect::all(px(3.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BackgroundColor(SUN),
                    BorderColor::all(INK),
                    SliderFill(id),
                ),
            ],
        ));
    });
}

/// A small keyboard-key chip (the page's `kbd`).
pub fn kbd(s: &str) -> impl Bundle {
    (
        Node {
            padding: UiRect::new(px(7.0), px(7.0), px(2.0), px(2.0)),
            border: UiRect::new(px(2.0), px(2.0), px(2.0), px(4.0)),
            border_radius: BorderRadius::all(px(7.0)),
            ..default()
        },
        BackgroundColor(WHITE),
        BorderColor::all(INK),
        children![text(s, 12.0, true, INK)],
    )
}

/// A coloured pill (the page's `.chip`).
pub fn chip(s: &str, bg: Color) -> impl Bundle {
    (
        Node {
            padding: UiRect::axes(px(9.0), px(5.0)),
            border: UiRect::all(px(2.0)),
            border_radius: BorderRadius::MAX,
            ..default()
        },
        BackgroundColor(bg),
        BorderColor::all(INK),
        children![text(s, 12.5, true, INK)],
    )
}

/// Keeps the two font files loaded so `FontSource::Family` can find them.
#[derive(Resource)]
pub struct Fonts(#[allow(dead_code)] Vec<Handle<Font>>);

pub struct FontsPlugin;

impl Plugin for FontsPlugin {
    fn build(&self, app: &mut App) {
        // built in (about 120 KB) and added at once, so the first menu layout already knows them
        let mut fonts = app.world_mut().resource_mut::<Assets<Font>>();
        let handles = vec![
            fonts.add(Font::from_bytes(include_bytes!("../assets/fonts/BowlbyOne-Regular.ttf").to_vec())),
            fonts.add(Font::from_bytes(include_bytes!("../assets/fonts/Figtree.ttf").to_vec())),
        ];
        app.insert_resource(Fonts(handles));
    }
}
