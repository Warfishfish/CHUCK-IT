//! Emotes (spec section 9) and the pool / trampoline bonus (spec section 5).
//!
//! T taunts, G dances, B laughs. Each has a short line over your head. Two seconds must pass
//! between emotes, and you can't do one while stunned or lying down. Knocking someone into the
//! pool or onto the trampoline within 3.5 s of hitting them pays a bonus.

use crate::pose::Emote;
use crate::rng::Rng;

/// Seconds you must wait between emotes.
pub const COOLDOWN: f32 = 2.0;
/// How long the speech bubble stays up.
pub const SPEECH_TIME: f32 = 2.6;

/// How long each emote animation lasts, seconds.
pub fn duration(e: Emote) -> f32 {
    match e {
        Emote::Taunt => 1.4,
        Emote::Laugh => 1.7,
        Emote::Dance => 2.4,
    }
}

const TAUNT: &[&str] = &[
    "Is that all ya got?",
    "Ya throw like me nan!",
    "Too slow, mate!",
    "Couldn't hit a barn door!",
    "Yeah nah, missed!",
    "Come at me!",
    "Easy as!",
    "Oi! Over here, ya galah!",
    "Stone the crows, weak!",
];
const LAUGH: &[&str] = &[
    "HAHAHAHA!",
    "Ha! Get wrecked!",
    "HAHA, look at ya!",
    "Hehehe. Classic.",
];
const DANCE: &[&str] = &["Oi oi oi!", "Look at these moves!", "Bust a move!"];
const TAUNT_ADULT: &[&str] = &[
    "Get rooted!",
    "Ya couldn't hit water if ya fell out of a boat!",
    "Rack off!",
];
const LAUGH_ADULT: &[&str] = &["HAHA, ya drongo!", "Pathetic, mate, truly!"];
const DANCE_ADULT: &[&str] = &["Shake what ya mum gave ya!", "Oi, check out these hips!"];

/// Every line this emote can say (Cheeky mode adds the rude ones).
pub fn lines(e: Emote, adult: bool) -> Vec<&'static str> {
    let (a, b) = match e {
        Emote::Taunt => (TAUNT, TAUNT_ADULT),
        Emote::Laugh => (LAUGH, LAUGH_ADULT),
        Emote::Dance => (DANCE, DANCE_ADULT),
    };
    let mut v = a.to_vec();
    if adult {
        v.extend_from_slice(b);
    }
    v
}

pub fn pick_line(e: Emote, adult: bool, rng: &mut Rng) -> &'static str {
    let l = lines(e, adult);
    l[((rng.f32() * l.len() as f32) as usize).min(l.len() - 1)]
}

/// Can you emote right now?
pub fn can_emote(now: f32, last: f32, stunned: bool, down: bool) -> bool {
    now - last >= COOLDOWN && !stunned && !down
}

/// Pool or trampoline.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Pool,
    Tramp,
}

impl Place {
    pub fn shout(self) -> &'static str {
        match self {
            Place::Pool => "SPLASHDOWN!",
            Place::Tramp => "INTO ORBIT!",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Place::Pool => "pool shot",
            Place::Tramp => "tramp shot",
        }
    }
}

/// Points for knocking someone into the pool or onto the trampoline.
pub const BONUS_WINDOW: f32 = 3.5;

/// Does this landing pay? `since_hit` is the time since `attacker` hit them. Not for
/// hitting yourself or a teammate.
pub fn pays(since_hit: f32, attacker: u32, victim: u32, same_team: bool) -> bool {
    since_hit <= BONUS_WINDOW && attacker != victim && !same_team
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_second_cooldown_and_no_emoting_when_stunned_or_down() {
        assert!(can_emote(5.0, 2.9, false, false));
        assert!(!can_emote(5.0, 3.1, false, false));
        assert!(!can_emote(5.0, 0.0, true, false));
        assert!(!can_emote(5.0, 0.0, false, true));
    }

    #[test]
    fn durations_match_the_spec() {
        assert_eq!(duration(Emote::Taunt), 1.4);
        assert_eq!(duration(Emote::Laugh), 1.7);
        assert_eq!(duration(Emote::Dance), 2.4);
    }

    #[test]
    fn cheeky_mode_adds_the_rude_lines() {
        for e in [Emote::Taunt, Emote::Laugh, Emote::Dance] {
            assert!(lines(e, true).len() > lines(e, false).len());
        }
        assert_eq!(lines(Emote::Taunt, false).len(), 9);
        assert_eq!(lines(Emote::Laugh, false).len(), 4);
        assert_eq!(lines(Emote::Dance, false).len(), 3);
        assert!(!lines(Emote::Taunt, false).contains(&"Get rooted!"));
    }

    #[test]
    fn a_picked_line_is_always_one_of_the_list() {
        let mut rng = Rng::new(4);
        for _ in 0..50 {
            let l = pick_line(Emote::Dance, true, &mut rng);
            assert!(lines(Emote::Dance, true).contains(&l));
        }
    }

    #[test]
    fn pool_and_tramp_bonus_rules() {
        assert!(pays(3.4, 1, 2, false));
        assert!(!pays(3.6, 1, 2, false), "too long ago");
        assert!(!pays(1.0, 1, 1, false), "yourself");
        assert!(!pays(1.0, 1, 2, true), "teammate");
        assert_eq!(Place::Pool.shout(), "SPLASHDOWN!");
        assert_eq!(Place::Tramp.shout(), "INTO ORBIT!");
    }
}
