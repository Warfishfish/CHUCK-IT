//! Gamepad support (Phase 8). The sticks are turned into movement and looking by
//! `bbq_core::gamepad`; this file maps the buttons onto the same things the keyboard and mouse
//! ask for (`Wanted`), so the game itself never needs to know which one you are using.
//!
//! | Pad                 | Does                                                        |
//! |---------------------|-------------------------------------------------------------|
//! | Left stick          | Walk (a small tilt walks slowly)                            |
//! | Right stick         | Look                                                        |
//! | A (bottom)          | Jump / wriggle / stand up                                   |
//! | Right trigger       | Hold to wind up, let go to throw or slap (like left click)  |
//! | Left trigger        | Catch (like right click)                                    |
//! | Bumpers             | Swap item                                                   |
//! | X (left)            | Drink, grab, smoko, help up (like R)                        |
//! | Y (top)             | Grab someone who's down, drag, throw (like F)               |
//! | B (right)           | Throw away what you're holding (like G)                     |
//! | Left stick press    | Speed boost (like Shift)                                    |
//! | D-pad up/right/left | Taunt / dance / laugh                                       |
//! | Start               | Pause; on the cards A presses the main button, B goes back  |

use bevy::input::gamepad::{Gamepad, GamepadButton};
use bevy::prelude::*;

use bbq_core::gamepad::{look_pixels, walk_stick};
use bbq_core::movement::{self, Modifiers};
use bbq_core::pose::Emote;

use crate::game::Game;
use crate::menu::{MenuUi, Request, Screen};
use crate::player::{Player, Wanted};

pub struct GamepadPlugin;

impl Plugin for GamepadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                read_pad.run_if(crate::menu::playing).after(crate::player::read_input),
                pad_flow,
            ),
        );
    }
}

/// Buttons and sticks while playing.
fn read_pad(
    pads: Query<&Gamepad>,
    time: Res<Time>,
    mut player: ResMut<Player>,
    mut wanted: ResMut<Wanted>,
    game: Res<Game>,
) {
    let dt = time.delta_secs().min(0.05);
    for pad in &pads {
        let ls = pad.left_stick();
        let (mx, my) = walk_stick(ls.x, ls.y);
        if mx != 0.0 || my != 0.0 {
            wanted.wish = movement::wish_dir(player.yaw, mx, my);
        }
        let rs = pad.right_stick();
        let (dx, dy) = look_pixels(rs.x, rs.y, dt);
        if dx != 0.0 || dy != 0.0 {
            let (y, p) = movement::look(player.yaw, player.pitch, dx, dy);
            player.yaw = y;
            player.pitch = p;
        }

        let stunned = game.me.body.stun > 0.0;
        let sitting = game.life.seated.is_some();
        if pad.just_pressed(GamepadButton::South) {
            wanted.jump_pressed = true;
            if !sitting {
                player.mover.try_jump(stunned);
            }
        }
        if pad.just_pressed(GamepadButton::LeftThumb) && !sitting && game.life.carry.is_none() {
            player.mover.try_boost(&Modifiers { stunned, ..Default::default() });
        }
        wanted.throw_down |= pad.just_pressed(GamepadButton::RightTrigger2);
        wanted.throw_up |= pad.just_released(GamepadButton::RightTrigger2);
        wanted.catch |= pad.just_pressed(GamepadButton::LeftTrigger2);
        if pad.just_pressed(GamepadButton::RightTrigger) {
            wanted.swap = 1;
        }
        if pad.just_pressed(GamepadButton::LeftTrigger) {
            wanted.swap = -1;
        }
        wanted.interact_pressed |= pad.just_pressed(GamepadButton::West);
        wanted.interact_held |= pad.pressed(GamepadButton::West);
        wanted.grab_pressed |= pad.just_pressed(GamepadButton::North);
        wanted.grab_released |= pad.just_released(GamepadButton::North);
        wanted.drop_held |= pad.just_pressed(GamepadButton::East);
        for (b, e) in [
            (GamepadButton::DPadUp, Emote::Taunt),
            (GamepadButton::DPadRight, Emote::Dance),
            (GamepadButton::DPadLeft, Emote::Laugh),
        ] {
            if pad.just_pressed(b) {
                wanted.emote = Some(e);
            }
        }
    }
}

/// Start pauses and resumes; on the menu and cards A presses the main button and B goes back.
fn pad_flow(
    pads: Query<&Gamepad>,
    mut screen: ResMut<Screen>,
    mut ui: ResMut<MenuUi>,
    mut game: ResMut<Game>,
) {
    let (mut a, mut b, mut start) = (false, false, false);
    for pad in &pads {
        a |= pad.just_pressed(GamepadButton::South);
        b |= pad.just_pressed(GamepadButton::East);
        start |= pad.just_pressed(GamepadButton::Start);
    }
    match flow(*screen, a, b, start) {
        Some(Flow::Pause) => {
            *screen = Screen::Paused;
            game.wind.cancel();
        }
        Some(Flow::Ask(req)) => ui.request = Some(req),
        None => {}
    }
}

/// What a press does on each screen (kept separate so it can be tested).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Flow {
    Pause,
    Ask(Request),
}

fn flow(screen: Screen, a: bool, b: bool, start: bool) -> Option<Flow> {
    match screen {
        Screen::Playing if start => Some(Flow::Pause),
        Screen::Paused if start || a => Some(Flow::Ask(Request::Resume)),
        Screen::Paused if b => Some(Flow::Ask(Request::Menu)),
        Screen::Menu if a || start => Some(Flow::Ask(Request::Play)),
        Screen::Results if a || start => Some(Flow::Ask(Request::Again)),
        Screen::Results if b => Some(Flow::Ask(Request::Menu)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_pauses_and_the_cards_answer_to_a_and_b() {
        assert_eq!(flow(Screen::Playing, false, false, true), Some(Flow::Pause));
        assert_eq!(flow(Screen::Playing, true, true, false), None, "A and B belong to the game");
        assert_eq!(flow(Screen::Paused, true, false, false), Some(Flow::Ask(Request::Resume)));
        assert_eq!(flow(Screen::Paused, false, false, true), Some(Flow::Ask(Request::Resume)));
        assert_eq!(flow(Screen::Paused, false, true, false), Some(Flow::Ask(Request::Menu)));
        assert_eq!(flow(Screen::Menu, true, false, false), Some(Flow::Ask(Request::Play)));
        assert_eq!(flow(Screen::Results, true, false, false), Some(Flow::Ask(Request::Again)));
        assert_eq!(flow(Screen::Results, false, true, false), Some(Flow::Ask(Request::Menu)));
        assert_eq!(flow(Screen::Menu, false, false, false), None);
    }
}
