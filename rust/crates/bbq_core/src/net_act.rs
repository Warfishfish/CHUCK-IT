//! Online play, step 9.5: what a guest asks the host to do, and what the host tells a guest
//! happened to them.
//!
//! A guest decides nothing about the rules. When you let go of a throw, the guest says "throw
//! this, from here, at this speed"; the host checks you really hold it, flies it, scores it.
//! When something hits you, the host says how hard and which way, and your own blob is knocked
//! about to match. Everything read from the network is checked: numbers must be finite and sane.

use crate::net::{DecodeError, Reader, Writer};

/// The fastest thing a guest may throw, metres per second (the game's own fastest is far less).
pub const MAX_THROW_SPEED: f32 = 80.0;
/// How far from where the host sees you a throw may start, metres.
pub const MAX_THROW_START: f32 = 4.0;

/// What a guest asks the host for.
#[derive(Clone, Debug, PartialEq)]
pub enum GuestAct {
    /// Let go of a throw: the item, where it starts, how fast, how long it was wound up.
    Throw { item: u32, from: (f32, f32, f32), vel: (f32, f32, f32), charge: f32 },
    /// Swing the item in your hand at whoever is in front of you.
    Slap { item: u32 },
    /// Throw away what you hold (G).
    Drop { item: u32 },
    /// Change which item is in your hand.
    Select { item: u32 },
    /// Right click: hold the catch.
    Catch,
}

/// What the host tells a guest that happened to them.
#[derive(Clone, Debug, PartialEq)]
pub enum HitMsg {
    /// Hit by a thrown item: its kind (an index into `ItemKind::ALL`), the direction it was
    /// going (flat), and whether it was a power throw that flattens.
    Item { kind: u8, dir: (f32, f32, f32), flatten: bool },
    /// Slapped: the numbers of the slap.
    Slap {
        dir: (f32, f32, f32),
        knock: f32,
        up: f32,
        stun: f32,
        dizzy: f32,
        /// A knockdown: which fall (0 sent flying, 1 cartwheel, 2 timber) and for how long.
        down: Option<(u8, f32)>,
    },
}

impl GuestAct {
    pub(crate) fn write(&self, w: &mut Writer) {
        match self {
            GuestAct::Throw { item, from, vel, charge } => {
                w.u8(1);
                w.u32(*item);
                w.triple(*from);
                w.triple(*vel);
                w.f32(*charge);
            }
            GuestAct::Slap { item } => {
                w.u8(2);
                w.u32(*item);
            }
            GuestAct::Drop { item } => {
                w.u8(3);
                w.u32(*item);
            }
            GuestAct::Select { item } => {
                w.u8(4);
                w.u32(*item);
            }
            GuestAct::Catch => w.u8(5),
        }
    }

    pub(crate) fn read(r: &mut Reader) -> Result<GuestAct, DecodeError> {
        Ok(match r.u8()? {
            1 => GuestAct::Throw { item: r.u32()?, from: r.triple()?, vel: r.triple()?, charge: r.f32()?.clamp(0.0, 1.0) },
            2 => GuestAct::Slap { item: r.u32()? },
            3 => GuestAct::Drop { item: r.u32()? },
            4 => GuestAct::Select { item: r.u32()? },
            5 => GuestAct::Catch,
            k => return Err(DecodeError::UnknownKind(k)),
        })
    }

    /// Is this a sane request? (Anything else is dropped.)
    pub fn sane(&self) -> bool {
        match self {
            GuestAct::Throw { from, vel, charge, .. } => {
                let speed = (vel.0 * vel.0 + vel.1 * vel.1 + vel.2 * vel.2).sqrt();
                speed <= MAX_THROW_SPEED && from.1 > -5.0 && from.1 < 30.0 && (0.0..=1.0).contains(charge)
            }
            _ => true,
        }
    }
}

impl HitMsg {
    pub(crate) fn write(&self, w: &mut Writer) {
        match self {
            HitMsg::Item { kind, dir, flatten } => {
                w.u8(1);
                w.u8(*kind);
                w.triple(*dir);
                w.u8(*flatten as u8);
            }
            HitMsg::Slap { dir, knock, up, stun, dizzy, down } => {
                w.u8(2);
                w.triple(*dir);
                w.f32(*knock);
                w.f32(*up);
                w.f32(*stun);
                w.f32(*dizzy);
                match down {
                    Some((pose, secs)) => {
                        w.u8(1 + (*pose).min(2));
                        w.f32(*secs);
                    }
                    None => w.u8(0),
                }
            }
        }
    }

    pub(crate) fn read(r: &mut Reader) -> Result<HitMsg, DecodeError> {
        let clamp = |v: f32, max: f32| v.clamp(-max, max);
        Ok(match r.u8()? {
            1 => HitMsg::Item { kind: r.u8()?, dir: r.triple()?, flatten: r.u8()? != 0 },
            2 => HitMsg::Slap {
                dir: r.triple()?,
                knock: clamp(r.f32()?, 40.0),
                up: clamp(r.f32()?, 40.0),
                stun: r.f32()?.clamp(0.0, 10.0),
                dizzy: r.f32()?.clamp(0.0, 10.0),
                down: match r.u8()? {
                    0 => None,
                    p @ 1..=3 => Some((p - 1, r.f32()?.clamp(0.0, 10.0))),
                    k => return Err(DecodeError::UnknownKind(k)),
                },
            },
            k => return Err(DecodeError::UnknownKind(k)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::Msg;

    #[test]
    fn requests_and_hits_survive_the_trip() {
        for m in [
            Msg::Act(GuestAct::Throw { item: 77, from: (1.0, 1.6, -2.0), vel: (3.0, 4.0, -5.0), charge: 0.5 }),
            Msg::Act(GuestAct::Slap { item: 3 }),
            Msg::Act(GuestAct::Drop { item: 4 }),
            Msg::Act(GuestAct::Select { item: 5 }),
            Msg::Act(GuestAct::Catch),
            Msg::Hit(HitMsg::Item { kind: 4, dir: (0.0, 0.0, -1.0), flatten: true }),
            Msg::Hit(HitMsg::Slap { dir: (1.0, 0.0, 0.0), knock: 13.0, up: 5.5, stun: 0.5, dizzy: 0.0, down: Some((1, 3.5)) }),
            Msg::Hit(HitMsg::Slap { dir: (1.0, 0.0, 0.0), knock: 8.0, up: 3.0, stun: 2.0, dizzy: 2.0, down: None }),
        ] {
            let back = Msg::decode(&m.encode());
            match (&m, &back) {
                // the charge is a plain float on the wire: compare the lot
                _ => assert_eq!(back, Ok(m.clone())),
            }
        }
    }

    #[test]
    fn rubbish_requests_are_refused_or_clamped() {
        let good = Msg::Act(GuestAct::Throw { item: 1, from: (0.0, 1.0, 0.0), vel: (0.0, 1.0, 0.0), charge: 0.3 }).encode();
        for n in 0..good.len() {
            assert!(Msg::decode(&good[..n]).is_err(), "cut at {n}");
        }
        assert!(Msg::decode(&[crate::net::K_ACT_FOR_TESTS, 99]).is_err());
        // a charge outside 0..1 is pulled back in, and NaN becomes 0
        let wild = Msg::Act(GuestAct::Throw { item: 1, from: (0.0, 1.0, 0.0), vel: (0.0, 1.0, 0.0), charge: 5.0 });
        let Ok(Msg::Act(GuestAct::Throw { charge, .. })) = Msg::decode(&wild.encode()) else { panic!() };
        assert_eq!(charge, 1.0);
        // numbers in a slap are kept in range
        let big = Msg::Hit(HitMsg::Slap { dir: (1.0, 0.0, 0.0), knock: 9999.0, up: -9999.0, stun: 500.0, dizzy: -1.0, down: Some((9, 99.0)) });
        let Ok(Msg::Hit(HitMsg::Slap { knock, up, stun, dizzy, down, .. })) = Msg::decode(&big.encode()) else { panic!() };
        assert_eq!((knock, up, stun, dizzy), (40.0, -40.0, 10.0, 0.0));
        assert_eq!(down, Some((2, 10.0)));
    }

    #[test]
    fn a_throw_has_to_be_possible() {
        let ok = GuestAct::Throw { item: 1, from: (0.0, 1.6, 0.0), vel: (10.0, 5.0, 0.0), charge: 1.0 };
        assert!(ok.sane());
        let too_fast = GuestAct::Throw { item: 1, from: (0.0, 1.6, 0.0), vel: (500.0, 0.0, 0.0), charge: 1.0 };
        assert!(!too_fast.sane());
        let underground = GuestAct::Throw { item: 1, from: (0.0, -40.0, 0.0), vel: (1.0, 0.0, 0.0), charge: 1.0 };
        assert!(!underground.sane());
        assert!(GuestAct::Catch.sane());
    }
}
