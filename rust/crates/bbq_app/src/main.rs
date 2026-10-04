//! Australian BBQ, Rust version. Phase 2: the yard and walking around it (solo, no bots).
//!
//! Click the window to grab the mouse. WASD walk, mouse looks, Space jumps, Shift boosts,
//! `[` and `]` change the field of view, F1-F4 switch the bar, BBQ, chest and smoko off and on,
//! Esc lets go of the mouse. All the rules live in `bbq_core`; this crate only draws and reads input.

mod player;
mod yard_scene;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Australian BBQ (Rust, Phase 2)".into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.55, 0.78, 0.95)))
        .add_plugins((yard_scene::YardScenePlugin, player::PlayerPlugin))
        .run();
}
