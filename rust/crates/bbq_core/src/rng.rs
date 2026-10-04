//! A tiny seedable random number generator, so rules that roll dice can be
//! tested with fixed seeds and replayed exactly.

#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        // Never let the state be zero.
        Rng(seed ^ 0x9E37_79B9_7F4A_7C15 | 1)
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in `[0, 1)`.
    pub fn f32(&mut self) -> f32 {
        ((self.next_u64() >> 40) as f32) / ((1u64 << 24) as f32)
    }

    /// A number in `[a, b)`.
    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + self.f32() * (b - a)
    }

    /// True with probability `p`.
    pub fn chance(&mut self, p: f32) -> bool {
        self.f32() < p
    }

    /// An index in `0..n` (`n` must be above 0).
    pub fn index(&mut self, n: usize) -> usize {
        ((self.f32() * n as f32) as usize).min(n - 1)
    }

    /// Shuffle a slice in place.
    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.index(i + 1);
            v.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_numbers() {
        let mut a = Rng::new(7);
        let mut b = Rng::new(7);
        for _ in 0..100 {
            assert_eq!(a.f32(), b.f32());
        }
    }

    #[test]
    fn stays_in_range() {
        let mut r = Rng::new(1);
        for _ in 0..10_000 {
            let x = r.f32();
            assert!((0.0..1.0).contains(&x));
            assert!(r.index(5) < 5);
        }
    }
}
