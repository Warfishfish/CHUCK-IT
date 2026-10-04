//! Australian BBQ, Rust version. Phases 2 to 4: the yard, walking, items and throwing, and the blob
//! characters and Dazza (solo, with three practice blobs to hit; real bots come in Phase 5).
//!
//! Click the window to grab the mouse. WASD walk, mouse looks, Space jumps, Shift boosts,
//! hold and release the left button to throw, right button catches, Q/E/wheel swap items,
//! `[` and `]` change the field of view, F1-F4 switch the bar, BBQ, chest and smoko off and on,
//! Esc lets go of the mouse. All the rules live in `bbq_core`; this crate only draws and reads input.

// Bevy systems take many parameters by design.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod characters;
mod game;
mod items_view;
mod player;
mod yard_scene;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Australian BBQ (Rust, Phase 4)".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.78, 0.95)))
        .add_plugins((
            yard_scene::YardScenePlugin,
            player::PlayerPlugin,
            game::GamePlugin,
            items_view::ItemsViewPlugin,
            characters::CharactersPlugin,
        ))
        .run();
}
