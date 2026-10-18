//! Stun, knockdown and stacking-it rules for one character (spec section 3, "What a hit does").
//!
//! The two rules that matter most: stuns **never stack**, and after a stun ends there is a
//! **1 second grace** during which no new stun can start (you can still be shoved).

/// A stun that has just ended gives this long before another can start.
pub const STUN_GRACE: f32 = 1.0;
/// A power throw (charge of 0.90 or more) flattens for this long.
pub const POWER_DOWN: f32 = 2.0;
/// Charge needed for a power throw.
pub const POWER_CHARGE: f32 = 0.90;

#[derive(Clone, Debug, Default)]
pub struct Body {
    /// Seconds of stun left.
    pub stun: f32,
    /// Seconds of grace left after a stun ended.
    pub stun_grace: f32,
    /// Seconds left knocked flat.
    pub down_t: f32,
    /// Seconds left after stacking it (drunk fall).
    pub fall_t: f32,
}

/// What happened when a hit landed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HitEffect {
    /// A fresh stun started (false if they were already stunned or in grace).
    pub stunned: bool,
    /// They were knocked flat.
    pub knocked_down: bool,
}

/// What happened during a tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickEvents {
    /// A stun just ended (grace starts).
    pub stun_ended: bool,
    /// They just got back up after stacking it.
    pub fall_ended: bool,
}

impl Body {
    /// Knocked flat or stacked it.
    pub fn is_down(&self) -> bool {
        self.down_t > 0.0 || self.fall_t > 0.0
    }

    /// Currently stunned or in the grace period: a new stun can't start.
    pub fn is_stun_protected(&self) -> bool {
        self.stun > 0.0 || self.stun_grace > 0.0
    }

    /// Apply a hit. `stun` is the stun length; `knockdown` is `Some(seconds)` to also knock
    /// them flat. Knockdown only works if they aren't already down or fallen.
    pub fn apply_hit(&mut self, stun: f32, knockdown: Option<f32>) -> HitEffect {
        let was_stunned = self.is_stun_protected();
        let mut stunned = false;
        if !was_stunned {
            self.stun = self.stun.max(stun);
            stunned = true;
        }
        let mut knocked_down = false;
        if let Some(dur) = knockdown
            && self.down_t <= 0.0
            && self.fall_t <= 0.0
        {
            self.down_t = dur;
            self.stun = self.stun.max(dur);
            knocked_down = true;
        }
        HitEffect {
            stunned,
            knocked_down,
        }
    }

    /// A power-throw hit knocks the victim flat only if they aren't in stun grace, down or fallen.
    /// Otherwise it is an ordinary hit.
    pub fn power_throw_flattens(&self) -> bool {
        self.stun_grace <= 0.0 && self.down_t <= 0.0 && self.fall_t <= 0.0
    }

    /// Stack it: down for `duration`, stunned for the same time.
    pub fn start_fall(&mut self, duration: f32) {
        self.fall_t = duration;
        self.stun = self.stun.max(duration);
    }

    /// Someone helped them up (or the fall otherwise ended early).
    pub fn get_up(&mut self) {
        self.fall_t = 0.0;
        self.stun = 0.0;
    }

    pub fn tick(&mut self, dt: f32) -> TickEvents {
        let mut ev = TickEvents::default();
        let before = self.stun;
        self.stun = (self.stun - dt).max(0.0);
        if before > 0.0 && self.stun <= 0.0 {
            self.stun_grace = STUN_GRACE;
            ev.stun_ended = true;
        } else if self.stun_grace > 0.0 {
            self.stun_grace = (self.stun_grace - dt).max(0.0);
        }
        self.down_t = (self.down_t - dt).max(0.0);
        if self.fall_t > 0.0 {
            self.fall_t = (self.fall_t - dt).max(0.0);
            self.stun = self.stun.max(self.fall_t);
            if self.fall_t <= 0.0 {
                self.stun = 0.0;
                ev.fall_ended = true;
            }
        }
        ev
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hit_stuns() {
        let mut b = Body::default();
        let e = b.apply_hit(0.56, None);
        assert!(e.stunned && !e.knocked_down);
        assert_eq!(b.stun, 0.56);
    }

    #[test]
    fn stuns_never_stack() {
        let mut b = Body::default();
        b.apply_hit(0.5, None);
        let e = b.apply_hit(2.0, None);
        assert!(!e.stunned);
        assert_eq!(b.stun, 0.5, "second hit must not start or extend a stun");
    }

    #[test]
    fn one_second_grace_after_a_stun_ends() {
        let mut b = Body::default();
        b.apply_hit(0.5, None);
        let mut ended = false;
        for _ in 0..40 {
            ended |= b.tick(1.0 / 60.0).stun_ended;
        }
        assert!(ended);
        assert_eq!(b.stun, 0.0);
        assert!(b.stun_grace > 0.0);
        assert!(!b.apply_hit(1.0, None).stunned, "no stun during grace");
        assert_eq!(b.stun, 0.0);
        // grace runs out after a second
        for _ in 0..61 {
            b.tick(1.0 / 60.0);
        }
        assert_eq!(b.stun_grace, 0.0);
        assert!(b.apply_hit(1.0, None).stunned);
    }

    #[test]
    fn a_knockdown_works_even_on_a_stunned_standing_player() {
        let mut b = Body::default();
        b.apply_hit(0.5, None);
        let e = b.apply_hit(0.5, Some(3.5));
        assert!(e.knocked_down && !e.stunned);
        assert_eq!(b.down_t, 3.5);
        assert_eq!(
            b.stun, 3.5,
            "knockdown holds them stunned for the same time"
        );
    }

    #[test]
    fn no_double_knockdown() {
        let mut b = Body::default();
        assert!(b.apply_hit(0.5, Some(3.5)).knocked_down);
        assert!(!b.apply_hit(0.5, Some(5.0)).knocked_down);
        assert_eq!(b.down_t, 3.5);
    }

    #[test]
    fn power_throw_only_flattens_when_not_protected() {
        let mut b = Body::default();
        assert!(b.power_throw_flattens());
        b.stun_grace = 0.5;
        assert!(!b.power_throw_flattens());
        let mut f = Body::default();
        f.start_fall(10.0);
        assert!(!f.power_throw_flattens());
    }

    #[test]
    fn stacking_it_lasts_and_then_ends() {
        let mut b = Body::default();
        b.start_fall(10.0);
        assert!(b.is_down());
        let mut ended = false;
        for _ in 0..(10 * 60 + 5) {
            ended |= b.tick(1.0 / 60.0).fall_ended;
        }
        assert!(ended && !b.is_down());
        assert_eq!(b.stun, 0.0);
    }

    #[test]
    fn helping_up_ends_a_fall_at_once() {
        let mut b = Body::default();
        b.start_fall(10.0);
        b.get_up();
        assert!(!b.is_down());
        assert_eq!(b.stun, 0.0);
    }
}
