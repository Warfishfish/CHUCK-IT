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
