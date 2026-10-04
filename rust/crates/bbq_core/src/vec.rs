//! A tiny 3D vector, so the rules don't need a graphics library.

use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl V3 {
    pub const ZERO: V3 = V3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }

    pub fn len_sq(self) -> f32 {
        self.dot(self)
    }

    pub fn dot(self, o: V3) -> f32 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }

    /// Length ignoring height.
    pub fn horiz_len(self) -> f32 {
        self.x.hypot(self.z)
    }

    /// Unit length, or zero if it has no length.
    pub fn normalised(self) -> V3 {
        let l = self.len();
        if l < 1e-9 { V3::ZERO } else { self * (1.0 / l) }
    }

    /// Horizontal distance to another point.
    pub fn horiz_dist(self, o: V3) -> f32 {
        (self.x - o.x).hypot(self.z - o.z)
    }
}

impl Add for V3 {
    type Output = V3;
    fn add(self, o: V3) -> V3 {
        V3::new(self.x + o.x, self.y + o.y, self.z + o.z)
    }
}

impl Sub for V3 {
    type Output = V3;
    fn sub(self, o: V3) -> V3 {
        V3::new(self.x - o.x, self.y - o.y, self.z - o.z)
    }
}

impl Mul<f32> for V3 {
    type Output = V3;
    fn mul(self, k: f32) -> V3 {
        V3::new(self.x * k, self.y * k, self.z * k)
    }
}

impl Neg for V3 {
    type Output = V3;
    fn neg(self) -> V3 {
        V3::new(-self.x, -self.y, -self.z)
    }
}

impl AddAssign for V3 {
    fn add_assign(&mut self, o: V3) {
        *self = *self + o;
    }
}

impl MulAssign<f32> for V3 {
    fn mul_assign(&mut self, k: f32) {
        *self = *self * k;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_maths() {
        let a = V3::new(3.0, 0.0, 4.0);
        assert_eq!(a.len(), 5.0);
        assert_eq!(a.horiz_len(), 5.0);
        assert!((a.normalised().len() - 1.0).abs() < 1e-6);
        assert_eq!(V3::ZERO.normalised(), V3::ZERO);
        assert_eq!(a + V3::new(1.0, 1.0, 1.0), V3::new(4.0, 1.0, 5.0));
        assert_eq!(a.dot(V3::new(1.0, 0.0, 0.0)), 3.0);
    }
}

/// A rotation (x, y, z, w), same convention as three.js and Bevy.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quat {
    pub const IDENTITY: Quat = Quat {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    /// Euler angles applied in X, then Y, then Z order (three.js default "XYZ").
    pub fn from_euler_xyz(x: f32, y: f32, z: f32) -> Quat {
        let (s1, c1) = (x / 2.0).sin_cos();
        let (s2, c2) = (y / 2.0).sin_cos();
        let (s3, c3) = (z / 2.0).sin_cos();
        Quat {
            x: s1 * c2 * c3 + c1 * s2 * s3,
            y: c1 * s2 * c3 - s1 * c2 * s3,
            z: c1 * c2 * s3 + s1 * s2 * c3,
            w: c1 * c2 * c3 - s1 * s2 * s3,
        }
    }

    pub fn from_axis_angle(axis: V3, angle: f32) -> Quat {
        let a = axis.normalised();
        let (s, c) = (angle / 2.0).sin_cos();
        Quat {
            x: a.x * s,
            y: a.y * s,
            z: a.z * s,
            w: c,
        }
    }

    /// `self * o`: apply `o` first, then `self` (same as `quaternion.multiply(o)` in three.js).
    pub fn mul(self, o: Quat) -> Quat {
        Quat {
            x: self.x * o.w + self.w * o.x + self.y * o.z - self.z * o.y,
            y: self.y * o.w + self.w * o.y + self.z * o.x - self.x * o.z,
            z: self.z * o.w + self.w * o.z + self.x * o.y - self.y * o.x,
            w: self.w * o.w - self.x * o.x - self.y * o.y - self.z * o.z,
        }
    }

    /// Turn a vector by this rotation.
    pub fn rotate(self, v: V3) -> V3 {
        let u = V3::new(self.x, self.y, self.z);
        let t = cross(u, v) * 2.0;
        v + t * self.w + cross(u, t)
    }
}

pub fn cross(a: V3, b: V3) -> V3 {
    V3::new(
        a.y * b.z - a.z * b.y,
        a.z * b.x - a.x * b.z,
        a.x * b.y - a.y * b.x,
    )
}

#[cfg(test)]
mod quat_tests {
    use super::*;

    #[test]
    fn quarter_turn_about_y_sends_z_to_x() {
        let q = Quat::from_axis_angle(V3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        let v = q.rotate(V3::new(0.0, 0.0, 1.0));
        assert!((v.x - 1.0).abs() < 1e-5 && v.z.abs() < 1e-5);
    }

    #[test]
    fn euler_matches_axis_angle_for_one_axis() {
        let a = Quat::from_euler_xyz(0.7, 0.0, 0.0);
        let b = Quat::from_axis_angle(V3::new(1.0, 0.0, 0.0), 0.7);
        assert!((a.x - b.x).abs() < 1e-6 && (a.w - b.w).abs() < 1e-6);
    }

    #[test]
    fn multiplying_applies_the_right_hand_side_first() {
        let rx = Quat::from_axis_angle(V3::new(1.0, 0.0, 0.0), std::f32::consts::FRAC_PI_2);
        let ry = Quat::from_axis_angle(V3::new(0.0, 1.0, 0.0), std::f32::consts::FRAC_PI_2);
        // ry * rx: rotate about X first, then Y. (0,0,1) -> X turn -> (0,-1,0) -> Y turn -> (0,-1,0)
        let v = ry.mul(rx).rotate(V3::new(0.0, 0.0, 1.0));
        assert!(
            v.x.abs() < 1e-5 && (v.y + 1.0).abs() < 1e-5 && v.z.abs() < 1e-5,
            "{v:?}"
        );
    }
}
