//! Australian BBQ, Rust version. Phases 2 to 4: the yard, walking, items and throwing, and the blob
//! characters and Dazza (solo, with three practice blobs to hit; real bots come in Phase 5).
//!
//! Click the window to grab the mouse. WASD walk, mouse looks, Space jumps, Shift boosts,
//! hold and release the left button to throw, right button catches, Q/E/wheel swap items,
//! `[` and `]` change the field of view, F1-F4 switch the bar, BBQ, chest and smoko off and on,
//! Esc lets go of the mouse. All the rules live in `bbq_core`; this crate only draws and reads input.

// Bevy systems take many parameters by design.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod bots_app;
mod characters;
mod decals;
mod fx;
mod game;
mod heist_app;
mod hud;
mod hud_fx;
mod items_view;
mod life;
mod lighting;
mod models;
mod player;
mod preview;
mod menu;
mod results;
mod round;
mod shapes;
mod shot;
mod ui;
mod yard_scene;

use bevy::prelude::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let shot = shot::parse(&args);
    // Screenshot mode uses an 800 x 600 window at normal pixel size so pictures can be compared.
    let window = if shot.is_some() {
        Window {
            title: "Australian BBQ (screenshot)".into(),
            resolution: bevy::window::WindowResolution::new(800, 600)
                .with_scale_factor_override(1.0),
            ..default()
        }
    } else {
        Window {
            title: "Australian BBQ (Rust, Phase 4)".into(),
            ..default()
        }
    };
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(window),
        ..default()
    }))
    .insert_resource(lighting::LookMode::from_args(&args))
    .insert_resource(bevy::light::DirectionalLightShadowMap {
        // crisper, steadier shadows in the polished look
        size: if lighting::LookMode::from_args(&args) == lighting::LookMode::Polished { 4096 } else { 2048 },
    })
    .insert_resource(ClearColor(models::hex(0x9fd8f2)))
    .add_plugins(
        (
            yard_scene::YardScenePlugin,
            player::PlayerPlugin,
            game::GamePlugin,
            items_view::ItemsViewPlugin,
            characters::CharactersPlugin,
            hud_fx::HudFxPlugin,
            hud::HudPlugin,
            bots_app::BotsPlugin,
            round::RoundPlugin,
            ui::FontsPlugin,
            menu::MenuPlugin,
            heist_app::HeistViewPlugin,
            models::ModelsPlugin,
            bevy::core_pipeline::fullscreen_material::FullscreenMaterialPlugin::<
                lighting::DisplayRaw,
            >::default(),
        ),
    );
    app.add_plugins((fx::FxPlugin, preview::PreviewPlugin, decals::DecalPlugin));
    if let Some(cfg) = shot {
        app.add_plugins(shot::ShotPlugin(cfg));
    }
    app.run();
}
