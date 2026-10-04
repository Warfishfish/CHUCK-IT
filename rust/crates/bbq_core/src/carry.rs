//! Grab, drag and throw someone who is down (spec section 6).

/// Longest anyone can be dragged, seconds.
pub const MAX_DRAG: f32 = 6.0;
/// Reach to start a grab (client check), metres.
pub const REACH: f32 = 1.9;
/// Reach the host allows, metres.
pub const HOST_REACH: f32 = 2.8;
/// Each wriggle (Space press) adds this much to the drag timer, seconds.
pub const WIGGLE: f32 = 0.8;
/// After release nobody can grab the same person for this long.
pub const IMMUNE: f32 = 8.0;
/// Hold F this long to throw instead of put down.
pub const HOLD: f32 = 0.4;
/// Grabber's speed multiplier while dragging.
pub const SLOW: f32 = 0.55;
/// The dragged person trails this far behind.
pub const DRAG_DISTANCE: f32 = 1.15;
/// Throw velocity (horizontal, up).
pub const THROW_HV: f32 = 10.5;
pub const THROW_VY: f32 = 8.5;
/// A thrown person is stunned this long.
pub const THROWN_STUN: f32 = 1.2;
/// A thrown person who hits someone knocks them flat for this long.
pub const CANNONBALL_DOWN: f32 = 2.2;
/// How close a thrown person has to get to someone to hit them.
pub const CANNONBALL_RADIUS: f32 = 1.05;
pub const CANNONBALL_HEIGHT: f32 = 1.5;
/// A thrown person can only hit someone between these times after the throw.
pub const CANNONBALL_WINDOW: (f32, f32) = (0.08, 1.2);
/// Gap between wriggles that count (anti-mashing), seconds.
pub const MIN_WIGGLE_GAP: f32 = 0.08;

/// How a drag ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    /// Tap F: set down gently (stun 0.5 s).
    PutDown,
    /// Wriggled free (or a bot's timer ran out).
    Wriggled,
    /// Timed out with no wriggling (stun 0.3 s).
    Dropped,
    /// Grabber was stunned, knocked down, fell, entered the pool, or someone left.
    Forced,
    /// Hold F: thrown.
    Chucked,
}

impl Release {
    /// Stun on the person who was dragged.
    pub fn victim_stun(self) -> f32 {
        match self {
            Release::PutDown => 0.5,
            Release::Wriggled | Release::Dropped => 0.3,
            Release::Chucked => THROWN_STUN,
            Release::Forced => 0.3,
        }
    }
}

/// Who is who in a grab: the bits of state that decide whether it is allowed.
#[derive(Clone, Copy, Debug, Default)]
pub struct Party {
    pub at_smoko: bool,
    pub in_naughty_corner: bool,
    pub stun: f32,
    pub down_t: f32,
    pub fall_t: f32,
    pub in_pool: bool,
    pub height: f32,
    pub carried: bool,
    pub carrying: bool,
}

/// Can `grabber` grab `target`? Only someone actually down (knocked flat or stacked it) can
/// be grabbed, not merely stunned. `round_active` is play or warm-up.
pub fn can_grab(round_active: bool, grabber: &Party, target: &Party) -> bool {
    round_active
        && !grabber.carried
        && !grabber.carrying
        && !target.carried
        && !target.carrying
        && !grabber.at_smoko
        && !target.at_smoko
        && !target.in_naughty_corner
        && grabber.stun <= 0.0
        && grabber.down_t <= 0.0
        && grabber.fall_t <= 0.0
        && !grabber.in_pool
        && !target.in_pool
        && target.height < 0.6
        && (target.down_t > 0.0 || target.fall_t > 0.0)
}

/// Is a held F a throw (rather than a put-down)?
pub fn is_throw(held_seconds: f32) -> bool {
    held_seconds >= HOLD
}

/// The throw velocity for a given facing (unit vector on the ground).
pub fn throw_velocity(facing_x: f32, facing_z: f32) -> (f32, f32, f32) {
    let l = facing_x.hypot(facing_z);
    let (x, z) = if l < 1e-3 {
        (0.0, 1.0)
    } else {
        (facing_x / l, facing_z / l)
    };
    (x * THROW_HV, THROW_VY, z * THROW_HV)
}

/// Does a thrown person at this spot hit someone else?
pub fn cannonball_hits(horizontal_dist: f32, height_diff: f32, since_throw: f32) -> bool {
    since_throw >= CANNONBALL_WINDOW.0
        && since_throw <= CANNONBALL_WINDOW.1
        && horizontal_dist < CANNONBALL_RADIUS
        && height_diff.abs() < CANNONBALL_HEIGHT
}

/// One drag in progress.
#[derive(Clone, Debug, Default)]
pub struct Drag {
    t: f32,
    wriggles: u32,
    last_wriggle: f32,
}

impl Drag {
    pub fn new() -> Self {
        Drag {
            t: 0.0,
            wriggles: 0,
            last_wriggle: -9.0,
        }
    }

    pub fn wriggles(&self) -> u32 {
        self.wriggles
    }

    /// Seconds on the drag timer.
    pub fn time(&self) -> f32 {
        self.t
    }

    /// The victim pressed Space. Presses closer than 0.08 s apart are ignored.
    pub fn wriggle(&mut self, now: f32) {
        if now - self.last_wriggle < MIN_WIGGLE_GAP {
            return;
        }
        self.last_wriggle = now;
        self.wriggles += 1;
        self.t += WIGGLE;
    }

    /// Advance the drag. Returns how it ended, if it did. `victim_is_bot` bots wriggle by
    /// themselves (their timer runs 1.7x). `grabber_ok` is false if the grabber was stunned,
    /// knocked down, fell, or is in the pool.
    pub fn tick(&mut self, dt: f32, victim_is_bot: bool, grabber_ok: bool) -> Option<Release> {
        if !grabber_ok {
            return Some(Release::Forced);
        }
        self.t += dt;
        if victim_is_bot {
            self.t += dt * 0.7;
        }
        if self.t >= MAX_DRAG {
            return Some(if self.wriggles > 0 || victim_is_bot {
                Release::Wriggled
            } else {
                Release::Dropped
            });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standing() -> Party {
        Party::default()
    }
    fn down() -> Party {
        Party {
            down_t: 2.0,
            ..Party::default()
        }
    }

    #[test]
    fn only_someone_down_can_be_grabbed() {
        assert!(can_grab(true, &standing(), &down()));
        assert!(!can_grab(true, &standing(), &standing()));
        // merely stunned is not enough
        assert!(!can_grab(
            true,
            &standing(),
            &Party {
                stun: 2.0,
                ..Party::default()
            }
        ));
        // a stacked-it player counts
        assert!(can_grab(
            true,
            &standing(),
            &Party {
                fall_t: 5.0,
                ..Party::default()
            }
        ));
    }

    #[test]
    fn grab_blockers() {
        let t = down();
        assert!(!can_grab(false, &standing(), &t), "round not active");
        assert!(!can_grab(
            true,
            &Party {
                stun: 0.1,
                ..standing()
            },
            &t
        ));
        assert!(!can_grab(
            true,
            &Party {
                down_t: 1.0,
                ..standing()
            },
            &t
        ));
        assert!(!can_grab(
            true,
            &Party {
                in_pool: true,
                ..standing()
            },
            &t
        ));
        assert!(!can_grab(
            true,
            &Party {
                carrying: true,
                ..standing()
            },
            &t
        ));
        assert!(!can_grab(true, &standing(), &Party { in_pool: true, ..t }));
        assert!(!can_grab(
            true,
            &standing(),
            &Party {
                at_smoko: true,
                ..t
            }
        ));
        assert!(!can_grab(
            true,
            &standing(),
            &Party {
                in_naughty_corner: true,
                ..t
            }
        ));
        assert!(!can_grab(true, &standing(), &Party { carried: true, ..t }));
        assert!(!can_grab(true, &standing(), &Party { height: 1.0, ..t }));
    }

    #[test]
    fn no_wriggling_means_dropped_after_six_seconds() {
        let mut d = Drag::new();
        let mut result = None;
        let mut secs = 0.0_f32;
        while result.is_none() && secs < 10.0 {
            result = d.tick(1.0 / 60.0, false, true);
            secs += 1.0 / 60.0;
        }
        assert_eq!(result, Some(Release::Dropped));
        assert!((secs - 6.0).abs() < 0.05, "took {secs}s");
    }

    #[test]
    fn about_eight_wriggles_free_you() {
        let mut d = Drag::new();
        // 8 presses, 0.1 s apart
        let mut now = 0.0;
        for _ in 0..8 {
            now += 0.1;
            d.wriggle(now);
            d.tick(0.1, false, true);
        }
        // 8 presses * 0.8 = 6.4 >= 6
        assert!(d.time() >= MAX_DRAG);
        assert_eq!(d.tick(0.0, false, true), Some(Release::Wriggled));
        assert_eq!(d.wriggles(), 8);
    }

    #[test]
    fn seven_wriggles_is_not_quite_enough_straight_away() {
        let mut d = Drag::new();
        for i in 0..7 {
            d.wriggle(i as f32 * 0.1);
        }
        assert!(d.time() < MAX_DRAG); // 5.6
    }

    #[test]
    fn mashing_faster_than_80ms_is_ignored() {
        let mut d = Drag::new();
        d.wriggle(1.0);
        d.wriggle(1.03);
        d.wriggle(1.05);
        assert_eq!(d.wriggles(), 1);
        d.wriggle(1.09);
        assert_eq!(d.wriggles(), 2);
    }

    #[test]
    fn bots_wriggle_by_themselves_faster() {
        let mut d = Drag::new();
        let mut secs = 0.0_f32;
        let mut r = None;
        while r.is_none() {
            r = d.tick(1.0 / 60.0, true, true);
            secs += 1.0 / 60.0;
        }
        assert_eq!(r, Some(Release::Wriggled));
        assert!((secs - 6.0 / 1.7).abs() < 0.05, "took {secs}s");
    }

    #[test]
    fn a_stunned_grabber_forces_a_drop() {
        let mut d = Drag::new();
        assert_eq!(d.tick(0.016, false, false), Some(Release::Forced));
    }

    #[test]
    fn put_down_versus_throw() {
        assert!(!is_throw(0.39));
        assert!(is_throw(0.4));
        assert_eq!(Release::PutDown.victim_stun(), 0.5);
        assert_eq!(Release::Dropped.victim_stun(), 0.3);
        assert_eq!(Release::Chucked.victim_stun(), 1.2);
    }

    #[test]
    fn throw_velocity_is_10_5_along_facing_and_8_5_up() {
        let (x, y, z) = throw_velocity(0.0, 2.0);
        assert!((x - 0.0).abs() < 1e-6 && (z - 10.5).abs() < 1e-5 && y == 8.5);
        let (x, _, z) = throw_velocity(0.0, 0.0);
        assert_eq!((x, z), (0.0, 10.5), "no facing falls back to +z");
    }

    #[test]
    fn cannonball_window_and_reach() {
        assert!(cannonball_hits(0.5, 0.0, 0.5));
        assert!(!cannonball_hits(0.5, 0.0, 0.05), "too soon after the throw");
        assert!(!cannonball_hits(0.5, 0.0, 1.3), "too late");
        assert!(!cannonball_hits(1.1, 0.0, 0.5), "too far");
        assert!(!cannonball_hits(0.5, 1.6, 0.5), "wrong height");
    }
}
