//! How a character's body is posed and animated (spec sections 7, 15 and 16).
//!
//! Everything is a function of time and state, so every machine plays the same animation and
//! tests can check the poses. The game feeds an `Animator` once per frame and gets back a
//! `Pose` to put on the body, hands and mesh.

use crate::drinks::drunk_amt;
use crate::items::DOWN_TIME;
use crate::vec::{Quat, V3};

const TAU: f32 = std::f32::consts::TAU;
const PI: f32 = std::f32::consts::PI;

// --- the three slap outcomes ----------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlapKind {
    /// Flung backwards, lands flat on the back and skids.
    SentFlying,
    /// Two backflips, squash landing, twitching.
    Cartwheel,
    /// Stiff jelly wobble, topples sideways like a felled tree.
    Timber,
}

impl SlapKind {
    pub const ALL: [SlapKind; 3] = [SlapKind::SentFlying, SlapKind::Cartwheel, SlapKind::Timber];
}

/// Body offsets for a slap animation.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SlapPose {
    pub rx: f32,
    pub ry: f32,
    pub rz: f32,
    pub y: f32,
    pub sx: f32,
    pub sy: f32,
}

/// Shortest signed angle from `b` to `a`.
pub fn ang_diff(a: f32, b: f32) -> f32 {
    let d = (a - b + PI).rem_euclid(TAU) - PI;
    if d < -PI { d + TAU } else { d }
}

/// Pose `t` seconds after the slap. `dur` is how long they stay down; `face_slapper_ry` is the
/// angle to turn the body so they fly off backwards (only used by Sent Flying).
pub fn slap_pose(kind: SlapKind, t: f32, dur: f32, face_slapper_ry: f32) -> SlapPose {
    let g = ((dur - t) / 0.45).clamp(0.0, 1.0);
    let (mut rx, mut ry, mut rz, mut sx, mut sy) = (0.0f32, 0.0f32, 0.0f32, 1.0f32, 1.0f32);
    let mut y;
    match kind {
        SlapKind::SentFlying => {
            ry = face_slapper_ry;
            if t < 0.75 {
                let u = t / 0.75;
                rx = -(PI / 2.0) * (u * 1.3).min(1.0);
                y = 0.36 * (u * 1.3).min(1.0);
                rz = 0.12 * (t * 16.0).sin();
            } else {
                let q = t - 0.75;
                rx = -(PI / 2.0) * g;
                y = (0.36 + 0.22 * (-q * 6.0).exp() * (q * 15.0).sin().abs()) * g;
                rz = 0.1 * (q * 22.0).sin() * (-q * 2.5).exp() * g;
                sy = 1.0 - 0.25 * (-q * 8.0).exp();
                sx = 1.0 + 0.2 * (-q * 8.0).exp();
                ry *= g;
            }
        }
        SlapKind::Cartwheel => {
            let l = -(TAU + PI / 2.0);
            if t < 1.0 {
                let u = 1.0 - (1.0 - t).powi(2);
                rx = l * u;
                y = 1.5 * (PI * t).sin() + 0.36 * t;
            } else {
                let q = t - 1.0;
                rx = -TAU + (l + TAU) * g;
                y = 0.36 * g;
                rz = 0.18 * (q * 24.0).sin() * (-q * 0.9).exp() * g;
                sy = 1.0 - 0.3 * (-q * 9.0).exp();
                sx = 1.0 + 0.25 * (-q * 9.0).exp();
            }
        }
        SlapKind::Timber => {
            if t < 0.9 {
                let w = (t * 46.0).sin();
                sx = 1.0 + 0.2 * w;
                sy = 1.0 - 0.17 * w;
                y = 0.06 * (t * 30.0).sin().abs();
            } else if t < 1.9 {
                let v = t - 0.9;
                rz = (PI / 2.0) * v * v;
                y = 0.36 * rz.sin();
            } else {
                let q = t - 1.9;
                rz = (PI / 2.0 - 0.2 * (-q * 6.0).exp() * (q * 16.0).sin().abs()) * g;
                y = 0.36 * g;
                sy = 1.0 - 0.2 * (-q * 10.0).exp();
            }
        }
    }
    SlapPose {
        rx,
        ry,
        rz,
        y,
        sx,
        sy,
    }
}

/// What the victim's own camera does during a slap.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SlapCam {
    pub lift: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    /// 0..1: how far down on the ground the camera is.
    pub lie: f32,
}

pub fn slap_cam(kind: SlapKind, t: f32, dur: f32) -> SlapCam {
    let g = ((dur - t) / 0.45).clamp(0.0, 1.0);
    let mut c = SlapCam::default();
    match kind {
        SlapKind::SentFlying => {
            if t < 0.75 {
                c.pitch = 0.9 * (t / 0.35).min(1.0);
                c.roll = 0.15 * (t * 14.0).sin();
            } else {
                c.lie = g;
            }
        }
        SlapKind::Cartwheel => {
            if t < 1.0 {
                c.pitch = TAU * (1.0 - (1.0 - t).powi(2));
                c.lift = 1.5 * (PI * t).sin();
            } else {
                c.lie = g;
            }
        }
        SlapKind::Timber => {
            if t < 0.9 {
                c.roll = 0.06 * (t * 46.0).sin();
            } else if t < 1.9 {
                let v = t - 0.9;
                c.roll = (PI / 2.0) * v * v;
                c.lift = -1.15 * c.roll.sin();
            } else {
                c.roll = PI / 2.0 * g;
                c.lift = -1.15 * g;
            }
        }
    }
    c
}

// --- the throw swing -------------------------------------------------------------------

/// Key-frames: [time, hand x, hand y, hand z, item pitch, body twist].
pub const SWING_KEYS: [[f32; 6]; 4] = [
    [0.0, -0.47, 0.88, 0.05, 0.0, 0.0],
    [0.13, -0.36, 1.98, -0.32, -1.15, -0.5],
    [0.29, 0.1, 1.0, 0.68, 2.05, 0.55],
    [0.54, -0.47, 0.88, 0.05, 0.0, 0.0],
];
/// The swing is visible for this long.
pub const SWING_VIS: f32 = 0.54;

pub fn swing_key(u: f32) -> [f32; 6] {
    for w in SWING_KEYS.windows(2) {
        let (a, b) = (w[0], w[1]);
        if u <= b[0] {
            let mut f = (u - a[0]) / (b[0] - a[0]);
            f = f * f * (3.0 - 2.0 * f);
            let mut out = [0.0; 6];
            for j in 0..6 {
                out[j] = a[j] + (b[j] - a[j]) * f;
            }
            return out;
        }
    }
    SWING_KEYS[3]
}

// --- emotes -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Emote {
    Taunt,
    Laugh,
    Dance,
}

impl Emote {
    pub fn duration(self) -> f32 {
        match self {
            Emote::Dance => 2.4,
            Emote::Laugh => 1.7,
            Emote::Taunt => 1.4,
        }
    }
}

/// Global cooldown between emotes.
pub const EMOTE_COOLDOWN: f32 = 2.0;
/// How long a speech bubble stays up.
pub const BUBBLE_TIME: f32 = 2.6;

// --- walking, drunk sway, lying down ---------------------------------------------------------

/// Walk cycle phase advance: `speed * 2.4` per second while grounded and moving.
pub fn advance_walk(walk: f32, speed: f32, grounded: bool, dt: f32) -> f32 {
    if grounded && speed > 0.5 {
        walk + speed * dt * 2.4
    } else {
        walk
    }
}

/// How flat a knocked-down character lies: eases in over 0.22 s and out over the last 0.45 s.
pub fn down_amount(down_t: f32) -> f32 {
    if down_t <= 0.0 {
        return 0.0;
    }
    ((DOWN_TIME - down_t) / 0.22)
        .min(down_t / 0.45)
        .clamp(0.0, 1.0)
}

/// Stacked it: falls over in 0.3 s; gets up over 0.45 s.
pub const FALL_IN: f32 = 0.3;
pub const RISE_TIME: f32 = 0.45;

// --- the animator -------------------------------------------------------------------------

/// What's going on with the character this frame.
#[derive(Clone, Copy, Debug, Default)]
pub struct Inputs {
    /// Game clock in seconds.
    pub t: f32,
    /// Direction they face (yaw).
    pub face: f32,
    pub vel: V3,
    pub grounded: bool,
    /// Drunk meter 0..100.
    pub drunk: f32,
    pub charging: bool,
    pub charge: f32,
    pub down_t: f32,
    pub stun: f32,
    /// Stacked it (fallen over from drink).
    pub fallen: bool,
    pub seated: bool,
    /// In the Naughty Corner (seated, but without the can).
    pub naughty: bool,
    pub slide_t: f32,
    /// Being dragged by someone: the grabber's facing, speed and walk phase.
    pub dragged_by: Option<DraggerInfo>,
    /// Carrying someone by the ankles.
    pub carrying: bool,
    pub wriggling: bool,
    /// Face the thrower of a slap (angle to turn the body).
    pub slap_face: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DraggerInfo {
    pub face: f32,
    pub speed: f32,
    pub walk: f32,
    pub grounded: bool,
}

/// The finished pose for this frame.
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    /// Turn of the whole character about the up axis.
    pub yaw: f32,
    /// Sideways drunk lurch added to the character's position.
    pub offset: (f32, f32),
    pub body_rot: Quat,
    pub body_y: f32,
    /// Squash and stretch: (x and z, y).
    pub body_scale: (f32, f32),
    /// Right hand (the throwing hand) and left hand, relative to the body.
    pub hand_r: V3,
    pub hand_l: V3,
    /// Pitch of the item held in the right hand.
    pub item_pitch: f32,
    pub stars: bool,
    /// The smoko can in hand is visible.
    pub can_visible: bool,
    /// Tilt the can while sipping.
    pub can_pitch: f32,
    /// If dragged: the whole mesh sits behind the grabber, facing the same way.
    pub drag: Option<DragPlace>,
}

#[derive(Clone, Copy, Debug)]
pub struct DragPlace {
    pub yaw: f32,
    /// Distance behind the grabber along their facing.
    pub behind: f32,
    /// Small vertical jig while the grabber is walking.
    pub jig: f32,
}

impl Default for Pose {
    fn default() -> Self {
        Pose {
            yaw: 0.0,
            offset: (0.0, 0.0),
            body_rot: Quat::IDENTITY,
            body_y: 0.0,
            body_scale: (1.0, 1.0),
            hand_r: V3::new(-0.47, 0.88, 0.05),
            hand_l: V3::new(0.47, 0.88, 0.05),
            item_pitch: 0.0,
            stars: false,
            can_visible: false,
            can_pitch: 0.0,
            drag: None,
        }
    }
}

/// An animation in progress.
#[derive(Clone, Copy, Debug)]
pub struct SlapAnim {
    pub kind: SlapKind,
    pub t: f32,
    pub dur: f32,
}

/// Per-character animation state.
#[derive(Clone, Debug)]
pub struct Animator {
    pub tilt: f32,
    tilt_v: f32,
    tilt_axis: V3,
    spin_v: f32,
    spin_a: f32,
    /// Walk cycle phase.
    pub walk: f32,
    /// Random phase so everyone sways differently.
    pub phase: f32,
    pub slap: Option<SlapAnim>,
    /// Seconds into the throw swing, if swinging.
    pub swing: Option<f32>,
    pub emote: Option<(Emote, f32)>,
    pub dizzy: f32,
    fall_age: f32,
    rise: f32,
}

impl Animator {
    pub fn new(phase: f32) -> Self {
        Self {
            tilt: 0.0,
            tilt_v: 0.0,
            tilt_axis: V3::new(1.0, 0.0, 0.0),
            spin_v: 0.0,
            spin_a: 0.0,
            walk: 0.0,
            phase,
            slap: None,
            swing: None,
            emote: None,
            dizzy: 0.0,
            fall_age: 0.0,
            rise: 0.0,
        }
    }

    /// A hit shoves them: they lean away and spin. `spin_sign` is +1 or -1 (random).
    pub fn tumble(&mut self, dir: V3, knock: f32, spin_sign: f32) {
        self.tilt_axis = V3::new(dir.z, 0.0, -dir.x);
        self.tilt_v += knock * 0.95;
        self.spin_v += spin_sign * knock * 0.45;
    }

    pub fn start_slap(&mut self, kind: SlapKind, dur: f32) {
        self.slap = Some(SlapAnim { kind, t: 0.0, dur });
    }

    pub fn start_swing(&mut self) {
        self.swing = Some(0.0);
    }

    pub fn start_emote(&mut self, e: Emote) {
        self.emote = Some((e, 0.0));
    }

    /// Extra spin from stepping on a puddle.
    pub fn add_spin(&mut self, v: f32) {
        self.spin_v += v;
    }

    /// Advance by `dt` and work out the pose.
    pub fn tick(&mut self, dt: f32, i: &Inputs) -> Pose {
        let mut p = Pose::default();
        let sp = i.vel.horiz_len();
        // springs and timers
        self.tilt_v += (-self.tilt * 55.0 - self.tilt_v * 7.0) * dt;
        self.tilt = (self.tilt + self.tilt_v * dt).clamp(-1.45, 1.45);
        self.spin_v *= (-3.0 * dt).exp();
        self.spin_a += self.spin_v * dt;
        self.walk = advance_walk(self.walk, sp, i.grounded, dt);
        self.dizzy = (self.dizzy - dt).max(0.0);

        // drunk body language
        let da = drunk_amt(i.drunk);
        let (t, ph) = (i.t, self.phase);
        let moving = i.grounded && sp > 0.8;
        let mut ry = i.face + self.spin_a;
        if da > 0.0 {
            ry += da
                * ((t * 0.9 + ph).sin() * 0.28
                    + if moving {
                        (self.walk * 0.5).sin() * 0.18
                    } else {
                        0.0
                    });
            let lurch = if moving {
                (self.walk * 0.5).sin() * 0.32 * da
            } else {
                (t * 0.7 + ph).sin() * 0.1 * da
            };
            p.offset = (ry.cos() * lurch, -ry.sin() * lurch);
        }
        p.yaw = ry;

        // lean from knocks
        let axis = rotate_about_y(self.tilt_axis, -ry);
        p.body_rot = if axis.len_sq() > 1e-6 {
            Quat::from_axis_angle(axis.normalised(), self.tilt)
        } else {
            Quat::IDENTITY
        };
        if da > 0.0 {
            let roll = da * ((t * 1.1 + ph).sin() * 0.23 + (t * 2.7 + ph * 2.0).sin() * 0.06)
                + if moving {
                    self.walk.sin() * 0.2 * da
                } else {
                    0.0
                };
            let pitch = da * (t * 0.8 + ph * 1.7).sin() * 0.13;
            p.body_rot = p.body_rot.mul(Quat::from_euler_xyz(pitch, 0.0, roll));
        }
        p.body_y = if i.grounded {
            self.walk.sin().abs() * (0.07 + 0.05 * da)
        } else {
            0.0
        };

        // the throwing hand winds up
        let wind = if i.charging { i.charge } else { 0.0 };
        p.hand_r = V3::new(
            -0.47 - da * 0.12 * (t * 2.3 + ph).sin().abs(),
            0.88 + wind * 0.75 + da * (t * 3.1 + ph).sin() * 0.22,
            0.05 - wind * 0.35 + da * (t * 1.9 + ph).sin() * 0.12,
        );

        // slap outcomes take over the whole body
        let mut slap_on = false;
        if let Some(s) = &mut self.slap {
            s.t += dt;
            if s.t > s.dur {
                self.slap = None;
            }
        }
        if let Some(s) = self.slap {
            let sp = slap_pose(s.kind, s.t, s.dur, i.slap_face);
            p.body_rot = Quat::from_euler_xyz(sp.rx, sp.ry, sp.rz);
            p.body_y = sp.y;
            p.body_scale = (sp.sx, sp.sy);
            slap_on = true;
        } else {
            let dn = down_amount(i.down_t);
            if dn > 0.0 {
                p.body_rot = Quat::from_euler_xyz(-PI / 2.0 * dn, 0.0, 0.0);
                p.body_y = 0.36 * dn;
            }
        }

        // throw swing
        if let Some(a) = &mut self.swing {
            *a += dt;
            if *a >= SWING_VIS {
                self.swing = None;
            }
        }
        if let Some(a) = self.swing
            && !slap_on
            && i.down_t <= 0.0
        {
            let k = swing_key(a);
            p.hand_r = V3::new(k[1], k[2], k[3]);
            p.item_pitch = k[4];
            p.body_rot = p
                .body_rot
                .mul(Quat::from_euler_xyz(k[4].max(0.0) * 0.08, k[5], 0.0));
        }

        // flailing arms on a slide
        if i.slide_t > 0.0 && !slap_on && i.down_t <= 0.0 {
            let u = (i.slide_t * 3.0).min(1.0);
            p.hand_r = V3::new(
                -0.62 + (t * 23.0).sin() * 0.14,
                1.2 + (t * 19.0).sin() * 0.38,
                0.05 + (t * 17.0).cos() * 0.25,
            );
            p.hand_l = V3::new(
                0.62 + (t * 21.0 + 1.0).sin() * 0.14,
                1.2 + (t * 18.0 + 2.0).sin() * 0.38,
                0.05 + (t * 16.0).cos() * 0.25,
            );
            p.body_rot = p.body_rot.mul(Quat::from_euler_xyz(
                -0.4 * u,
                0.0,
                (t * 13.0).sin() * 0.32 * u,
            ));
        }

        // stacked it, and getting back up
        let mut fl = 0.0;
        if i.fallen {
            self.fall_age += dt;
            fl = (self.fall_age / FALL_IN).min(1.0);
            self.rise = RISE_TIME;
        } else {
            self.fall_age = 0.0;
            if self.rise > 0.0 {
                self.rise = (self.rise - dt).max(0.0);
                fl = self.rise / RISE_TIME;
            }
        }
        if fl > 0.0 && !slap_on {
            p.body_rot = Quat::from_euler_xyz(PI / 2.0 * fl, 0.0, (t * 1.6 + ph).sin() * 0.06 * fl);
            p.body_y = 0.34 * fl;
        }

        // dizzy wobble after a steak, fish, noodle or spatula
        if self.dizzy > 0.0 && !slap_on && i.down_t <= 0.0 {
            let w = self.dizzy.min(1.0) * 0.24;
            p.body_rot = p.body_rot.mul(Quat::from_euler_xyz(
                (t * 7.0).sin() * w,
                0.0,
                (t * 7.0).cos() * w,
            ));
        }

        // sitting at smoko (or in the Naughty Corner)
        let seated = (i.seated || i.naughty) && !slap_on && fl <= 0.0;
        p.can_visible = seated && !i.naughty;

        // emotes
        if let Some((e, et)) = &mut self.emote {
            *et += dt;
            if *et > e.duration() || slap_on || fl > 0.0 || i.stun > 0.5 {
                self.emote = None;
            }
        }
        if let Some((e, et)) = self.emote
            && !seated
        {
            match e {
                Emote::Taunt => {
                    p.hand_r = V3::new(-0.47, 1.25 + 0.35 * (et * 10.0).sin().abs(), 0.12);
                    p.body_rot = p.body_rot.mul(Quat::from_euler_xyz(-0.14, 0.0, 0.0));
                }
                Emote::Laugh => {
                    p.hand_r = V3::new(-0.18, 1.02, 0.62);
                    p.body_rot = p.body_rot.mul(Quat::from_euler_xyz(
                        -0.22 + (et * 24.0).sin() * 0.1,
                        0.0,
                        (et * 12.0).sin() * 0.06,
                    ));
                }
                Emote::Dance => {
                    p.body_y += (et * 9.0).sin().abs() * 0.16;
                    p.body_rot = p.body_rot.mul(Quat::from_euler_xyz(
                        0.0,
                        et * 5.5,
                        (et * 9.0).sin() * 0.22,
                    ));
                    p.hand_r = V3::new(-0.5, 1.2 + 0.3 * (et * 9.0).sin(), 0.1);
                }
            }
        }
        if seated {
            p.body_y = -0.3;
            p.body_rot = Quat::from_euler_xyz(-0.2, 0.0, 0.0);
            let sip = (t * 1.5 + ph).sin().max(0.0).powi(6);
            p.hand_r = V3::new(-0.34, 0.9 + 0.5 * sip, 0.28);
            p.can_pitch = -sip * 1.2;
        }

        // dragged along on their back, or carrying someone
        if let Some(g) = i.dragged_by {
            let jig = if g.grounded && g.speed > 0.5 {
                (g.walk * 1.3).sin().abs() * 0.05
            } else {
                0.0
            };
            p.drag = Some(DragPlace {
                yaw: g.face,
                behind: 0.55,
                jig,
            });
            let flop = (t * 3.1 + ph).sin() * 0.12 * if g.speed > 0.5 { 1.6 } else { 1.0 }
                + if i.wriggling {
                    (t * 60.0).sin() * 0.4
                } else {
                    0.0
                };
            p.body_rot = Quat::from_euler_xyz(-PI / 2.0 + 0.12, 0.0, flop);
            p.body_y = 0.3;
            p.body_scale = (1.0, 1.0);
            p.hand_r = V3::new(-0.55, 0.35 + (t * 7.0 + ph).sin() * 0.15, -0.1);
            p.hand_l = V3::new(0.55, 0.35 + (t * 7.0 + ph).cos() * 0.15, -0.1);
        } else if i.carrying {
            p.body_rot = p.body_rot.mul(Quat::from_euler_xyz(0.2, 0.0, 0.0));
            p.hand_r = V3::new(-0.32, 0.62, -0.42);
            p.hand_l = V3::new(0.32, 0.62, -0.42);
        }

        p.stars = i.stun > 0.15 || self.dizzy > 0.0;
        p
    }
}

fn rotate_about_y(v: V3, angle: f32) -> V3 {
    let (s, c) = angle.sin_cos();
    V3::new(v.x * c + v.z * s, v.y, -v.x * s + v.z * c)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn idle() -> Inputs {
        Inputs {
            grounded: true,
            ..Default::default()
        }
    }

    #[test]
    fn sent_flying_ends_flat_on_the_back() {
        let p = slap_pose(SlapKind::SentFlying, 3.0, 3.5, 0.0);
        assert!((p.rx + PI / 2.0).abs() < 0.05);
        assert!((p.y - 0.36).abs() < 0.05);
    }

    #[test]
    fn cartwheel_is_in_the_air_mid_flip_and_flat_at_the_end() {
        let mid = slap_pose(SlapKind::Cartwheel, 0.5, 3.5, 0.0);
        assert!(mid.y > 1.5, "{}", mid.y);
        let end = slap_pose(SlapKind::Cartwheel, 3.4, 3.5, 0.0);
        assert!((end.y - 0.36 * ((3.5 - 3.4) / 0.45)).abs() < 0.05);
    }

    #[test]
    fn timber_wobbles_then_topples_sideways() {
        let w = slap_pose(SlapKind::Timber, 0.3, 3.5, 0.0);
        assert!(w.rz == 0.0 && (w.sx - 1.0).abs() > 0.0);
        let down = slap_pose(SlapKind::Timber, 3.0, 3.5, 0.0);
        assert!((down.rz - PI / 2.0).abs() < 0.1);
    }

    #[test]
    fn every_slap_gets_back_up_by_the_end() {
        for k in SlapKind::ALL {
            let p = slap_pose(k, 3.5, 3.5, 0.0);
            assert!(
                p.rx.abs() < 1e-3 || (p.rx + TAU).abs() < 1e-3,
                "{k:?} {p:?}"
            );
            assert!(p.rz.abs() < 1e-3 && p.y.abs() < 1e-3);
        }
    }

    #[test]
    fn slap_cameras_end_lying_down() {
        for k in SlapKind::ALL {
            assert!(slap_cam(k, 2.5, 3.5).lie > 0.9 || k == SlapKind::Timber);
        }
        let c = slap_cam(SlapKind::Timber, 3.0, 3.5);
        assert!((c.lift + 1.15).abs() < 0.01);
    }

    #[test]
    fn swing_goes_up_then_forward_then_home() {
        assert_eq!(swing_key(0.0)[2], 0.88);
        let up = swing_key(0.13);
        assert!((up[2] - 1.98).abs() < 1e-5);
        let fwd = swing_key(0.29);
        assert!((fwd[3] - 0.68).abs() < 1e-5);
        assert_eq!(swing_key(0.54)[1], -0.47);
        assert_eq!(swing_key(5.0)[1], -0.47);
    }

    #[test]
    fn emote_lengths() {
        assert_eq!(Emote::Dance.duration(), 2.4);
        assert_eq!(Emote::Laugh.duration(), 1.7);
        assert_eq!(Emote::Taunt.duration(), 1.4);
    }

    #[test]
    fn walking_bobs_up_and_down_only_when_moving() {
        let mut a = Animator::new(0.0);
        let still = a.tick(DT, &idle());
        assert_eq!(still.body_y, 0.0);
        let mut i = idle();
        i.vel = V3::new(6.0, 0.0, 0.0);
        let mut maxy = 0.0f32;
        for _ in 0..60 {
            maxy = maxy.max(a.tick(DT, &i).body_y);
        }
        assert!(maxy > 0.05 && maxy <= 0.07 + 1e-4);
    }

    #[test]
    fn walk_phase_advances_at_speed_times_2_4() {
        assert!((advance_walk(0.0, 5.0, true, 1.0) - 12.0).abs() < 1e-5);
        assert_eq!(advance_walk(1.0, 0.2, true, 1.0), 1.0);
        assert_eq!(advance_walk(1.0, 5.0, false, 1.0), 1.0);
    }

    #[test]
    fn a_sober_character_stands_straight_and_a_drunk_one_sways() {
        let mut a = Animator::new(1.0);
        let mut i = idle();
        i.t = 3.0;
        let sober = a.tick(DT, &i);
        assert!((sober.body_rot.w - 1.0).abs() < 1e-5 && sober.yaw == 0.0);
        i.drunk = 85.0;
        let drunk = a.tick(DT, &i);
        assert!(drunk.body_rot.w < 0.999 || drunk.yaw != 0.0);
    }

    #[test]
    fn winding_up_raises_the_hand() {
        let mut a = Animator::new(0.0);
        let mut i = idle();
        let base = a.tick(DT, &i).hand_r;
        i.charging = true;
        i.charge = 1.0;
        let up = a.tick(DT, &i).hand_r;
        assert!((up.y - base.y - 0.75).abs() < 1e-4);
        assert!((base.z - up.z - 0.35).abs() < 1e-4);
    }

    #[test]
    fn a_swing_plays_for_054_seconds_then_stops() {
        let mut a = Animator::new(0.0);
        a.start_swing();
        let mut frames = 0;
        for _ in 0..60 {
            a.tick(DT, &idle());
            if a.swing.is_some() {
                frames += 1;
            }
        }
        assert!((31..=34).contains(&frames), "{frames}");
    }

    #[test]
    fn knocked_flat_eases_in_and_out() {
        assert_eq!(down_amount(0.0), 0.0);
        assert!((down_amount(DOWN_TIME - 0.11) - 0.5).abs() < 1e-4);
        assert_eq!(down_amount(DOWN_TIME / 2.0), 1.0);
        assert!((down_amount(0.225) - 0.5).abs() < 1e-4);
    }

    #[test]
    fn stacking_it_falls_in_over_0_3_s_and_rises_over_0_45_s() {
        let mut a = Animator::new(0.0);
        let mut i = idle();
        i.fallen = true;
        let mut y = 0.0;
        for _ in 0..30 {
            y = a.tick(DT, &i).body_y;
        }
        assert!((y - 0.34).abs() < 0.01, "{y}");
        i.fallen = false;
        let mid = (0..13).map(|_| a.tick(DT, &i).body_y).last().unwrap();
        assert!(mid > 0.0 && mid < 0.34);
        for _ in 0..40 {
            y = a.tick(DT, &i).body_y;
        }
        assert!(y < 0.1);
    }

    #[test]
    fn being_seated_lowers_the_body_and_shows_the_can_unless_naughty() {
        let mut a = Animator::new(0.0);
        let mut i = idle();
        i.seated = true;
        let p = a.tick(DT, &i);
        assert_eq!(p.body_y, -0.3);
        assert!(p.can_visible);
        i.seated = false;
        i.naughty = true;
        assert!(!a.tick(DT, &i).can_visible);
    }

    #[test]
    fn an_emote_ends_when_its_time_is_up_or_you_are_hit() {
        let mut a = Animator::new(0.0);
        a.start_emote(Emote::Taunt);
        for _ in 0..90 {
            a.tick(DT, &idle());
        }
        assert!(a.emote.is_none());
        a.start_emote(Emote::Dance);
        let mut i = idle();
        i.stun = 1.0;
        a.tick(DT, &i);
        assert!(a.emote.is_none());
    }

    #[test]
    fn a_dragged_character_lies_on_their_back_behind_the_grabber() {
        let mut a = Animator::new(0.0);
        let mut i = idle();
        i.dragged_by = Some(DraggerInfo {
            face: 1.0,
            speed: 3.0,
            walk: 1.0,
            grounded: true,
        });
        let p = a.tick(DT, &i);
        let d = p.drag.expect("drag place");
        assert_eq!(d.yaw, 1.0);
        assert_eq!(d.behind, 0.55);
        assert_eq!(p.body_y, 0.3);
    }

    #[test]
    fn the_carrier_leans_forward_with_hands_back() {
        let mut a = Animator::new(0.0);
        let mut i = idle();
        i.carrying = true;
        let p = a.tick(DT, &i);
        assert_eq!(p.hand_r, V3::new(-0.32, 0.62, -0.42));
        assert_eq!(p.hand_l, V3::new(0.32, 0.62, -0.42));
    }

    #[test]
    fn a_hit_makes_them_lean_then_spring_back() {
        let mut a = Animator::new(0.0);
        a.tumble(V3::new(0.0, 0.0, 1.0), 10.0, 1.0);
        let mut peak = 0.0f32;
        for _ in 0..180 {
            a.tick(DT, &idle());
            peak = peak.max(a.tilt.abs());
        }
        assert!(peak > 0.1);
        assert!(a.tilt.abs() < 0.01, "should settle: {}", a.tilt);
    }

    #[test]
    fn stars_show_while_stunned_or_dizzy() {
        let mut a = Animator::new(0.0);
        let mut i = idle();
        assert!(!a.tick(DT, &i).stars);
        i.stun = 1.0;
        assert!(a.tick(DT, &i).stars);
        i.stun = 0.0;
        a.dizzy = 1.0;
        assert!(a.tick(DT, &i).stars);
    }

    #[test]
    fn slap_animation_runs_and_clears() {
        let mut a = Animator::new(0.0);
        a.start_slap(SlapKind::Cartwheel, 3.5);
        let p = a.tick(0.5, &idle());
        assert!(p.body_y > 1.0);
        for _ in 0..60 * 4 {
            a.tick(DT, &idle());
        }
        assert!(a.slap.is_none());
    }

    #[test]
    fn angle_difference_wraps() {
        assert!((ang_diff(0.1, TAU - 0.1) - 0.2).abs() < 1e-5);
        assert!((ang_diff(-3.0, 3.0) - (TAU - 6.0)).abs() < 1e-4);
    }
}
