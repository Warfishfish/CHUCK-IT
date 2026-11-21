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
//! | X (left)            | Drink, grab, help up (like R)                        |
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PadAction {
    Jump, Boost, Throw, Catch, Next, Previous, Interact, Grab, Drop, Taunt, Dance, Laugh,
}

impl PadAction {
    pub const ALL: [Self; 12] = [
        Self::Jump, Self::Boost, Self::Throw, Self::Catch, Self::Next, Self::Previous,
        Self::Interact, Self::Grab, Self::Drop, Self::Taunt, Self::Dance, Self::Laugh,
    ];

    pub fn label(self) -> &'static str {
        ["Jump", "Boost / dive", "Throw / slap", "Catch", "Next item", "Previous item",
         "Interact", "Grab / chuck", "Drop item", "Taunt", "Dance", "Laugh"][self as usize]
    }
}

use GamepadButton as B;
const BUTTONS: [(B, &str); 12] = [
    (B::South, "Bottom face"), (B::LeftThumb, "Left stick press"),
    (B::RightTrigger2, "Right trigger"), (B::LeftTrigger2, "Left trigger"),
    (B::RightTrigger, "Right bumper"), (B::LeftTrigger, "Left bumper"),
    (B::West, "Left face"), (B::North, "Top face"), (B::East, "Right face"),
    (B::DPadUp, "D-pad up"), (B::DPadRight, "D-pad right"), (B::DPadLeft, "D-pad left"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PadBindings([usize; BUTTONS.len()]);

impl Default for PadBindings {
    fn default() -> Self { Self(std::array::from_fn(|i| i)) }
}

impl PadBindings {
    fn button(self, action: PadAction) -> B { BUTTONS[self.0[action as usize]].0 }
    pub fn label(self, action: PadAction) -> &'static str { BUTTONS[self.0[action as usize]].1 }

    pub fn reassign(&mut self, action: PadAction, button: B) -> bool {
        let Some(index) = BUTTONS.iter().position(|&(candidate, _)| candidate == button) else {
            return false;
        };
        let owner = self.0.iter().position(|&i| i == index).unwrap();
        self.0.swap(action as usize, owner);
        true
    }

    pub fn encode(self) -> String {
        self.0.map(|i| i.to_string()).join(",")
    }

    pub fn decode(raw: &str) -> Self {
        let mut values = raw.split(',');
        let mut bindings = Self::default();
        let mut used = [false; BUTTONS.len()];
        for slot in &mut bindings.0 {
            let Some(value) = values.next().and_then(|s| s.parse::<usize>().ok()) else {
                return Self::default();
            };
            if value >= used.len() || std::mem::replace(&mut used[value], true) {
                return Self::default();
            }
            *slot = value;
        }
        if values.next().is_some() { Self::default() } else { bindings }
    }
}

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
    settings: Res<crate::menu::Settings>,
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
        let button = |action| settings.pad_bindings.button(action);
        let pressed = |action| pad.just_pressed(button(action));
        if pressed(PadAction::Jump) {
            wanted.jump_pressed = true;
            if !sitting {
                player.mover.try_jump(stunned);
            }
        }
        if pressed(PadAction::Boost) && !sitting && game.life.carry.is_none() {
            player.mover.try_boost(&Modifiers { stunned, ..Default::default() });
        }
        wanted.dive |= pad.pressed(button(PadAction::Boost));
        wanted.throw_down |= pressed(PadAction::Throw);
        wanted.throw_up |= pad.just_released(button(PadAction::Throw));
        wanted.catch |= pressed(PadAction::Catch);
        if pressed(PadAction::Next) {
            wanted.swap = 1;
        }
        if pressed(PadAction::Previous) {
            wanted.swap = -1;
        }
        wanted.interact_pressed |= pressed(PadAction::Interact);
        wanted.interact_held |= pad.pressed(button(PadAction::Interact));
        wanted.grab_pressed |= pressed(PadAction::Grab);
        wanted.grab_released |= pad.just_released(button(PadAction::Grab));
        wanted.drop_held |= pressed(PadAction::Drop);
        for (b, e) in [
            (PadAction::Taunt, Emote::Taunt),
            (PadAction::Dance, Emote::Dance),
            (PadAction::Laugh, Emote::Laugh),
        ] {
            if pressed(b) {
                wanted.emote = Some(e);
            }
        }
    }
}

/// Start pauses and resumes. The menu system routes accept to the focused button.
fn pad_flow(
    pads: Query<&Gamepad>,
    mut screen: ResMut<Screen>,
    mut ui: ResMut<MenuUi>,
    mut game: ResMut<Game>,
) {
    let (mut b, mut start) = (false, false);
    for pad in &pads {
        b |= pad.just_pressed(GamepadButton::East);
        start |= pad.just_pressed(GamepadButton::Start);
    }
    match flow(*screen, b, start && !(*screen == Screen::Results && game.mirror)) {
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

fn flow(screen: Screen, b: bool, start: bool) -> Option<Flow> {
    match screen {
        Screen::Playing if start => Some(Flow::Pause),
        Screen::Paused if start => Some(Flow::Ask(Request::Resume)),
        Screen::Paused if b => Some(Flow::Ask(Request::Menu)),
        // Menu focus and activation are handled by menu::controller_menu.
        Screen::Menu => None,
        Screen::Results if start => Some(Flow::Ask(Request::Again)),
        Screen::Results if b => Some(Flow::Ask(Request::Menu)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_pauses_and_back_leaves_the_cards() {
        assert_eq!(flow(Screen::Playing, false, true), Some(Flow::Pause));
        assert_eq!(flow(Screen::Playing, true, false), None);
        assert_eq!(flow(Screen::Paused, false, true), Some(Flow::Ask(Request::Resume)));
        assert_eq!(flow(Screen::Paused, true, false), Some(Flow::Ask(Request::Menu)));
        assert_eq!(flow(Screen::Results, false, true), Some(Flow::Ask(Request::Again)));
        assert_eq!(flow(Screen::Results, true, false), Some(Flow::Ask(Request::Menu)));
        assert_eq!(flow(Screen::Menu, true, true), None);
    }

    #[test]
    fn remapping_swaps_the_conflict_and_roundtrips() {
        let mut controls = PadBindings::default();
        assert!(controls.reassign(PadAction::Jump, B::East));
        assert_eq!(controls.button(PadAction::Drop), B::South);
        assert_eq!(controls.button(PadAction::Jump), B::East);
        assert!(!controls.reassign(PadAction::Jump, B::Start));
        assert_eq!(PadBindings::decode(&controls.encode()), controls);
        assert_eq!(PadBindings::decode("1,1,2"), PadBindings::default());
    }
}
