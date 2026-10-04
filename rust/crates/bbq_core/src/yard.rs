//! The yard: its size, the pool, the trampoline and every solid box (spec section 1).
//!
//! All boxes are axis-aligned and stand on the ground. The yard fence is not a box: it is just
//! the edge of the yard (see `movement`).

use crate::{YARD_HALF_X, YARD_HALF_Z};

/// What a box is (so the game can pick a look for it and hide it with its feature).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Plain,
    Pole,
    Shed,
    Esky,
    Table,
    Grill,
    MeatTable,
    Bins,
    Crates,
    Hedge,
    Tyres,
    Woodpile,
    Wall,
    Planter,
    Bar,
    Chest,
}

/// The four map features the host can switch off.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Feature {
    Bar,
    Bbq,
    Chest,
    Smoko,
}

impl Kind {
    /// The feature that owns this box, if any.
    pub fn feature(self) -> Option<Feature> {
        match self {
            Kind::Bar => Some(Feature::Bar),
            Kind::Grill | Kind::MeatTable => Some(Feature::Bbq),
            Kind::Chest => Some(Feature::Chest),
            _ => None,
        }
    }
}

/// A box on the ground.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Collider {
    pub kind: Kind,
    pub x0: f32,
    pub x1: f32,
    pub z0: f32,
    pub z1: f32,
    /// Height of the top.
    pub h: f32,
    /// Too thin to stand on (the clothesline pole).
    pub no_top: bool,
}

impl Collider {
    /// Box from a centre and width (x) and depth (z), like the JavaScript `col()`.
    pub fn centred(cx: f32, cz: f32, w: f32, d: f32, h: f32) -> Self {
        Self {
            x0: cx - w / 2.0,
            x1: cx + w / 2.0,
            z0: cz - d / 2.0,
            z1: cz + d / 2.0,
            h,
            no_top: false,
            kind: Kind::Plain,
        }
    }

    pub fn of(mut self, kind: Kind) -> Self {
        self.kind = kind;
        self
    }
}

/// The swimming pool rectangle (spec section 1).
pub const POOL_X0: f32 = -27.0;
pub const POOL_X1: f32 = -13.0;
pub const POOL_Z0: f32 = 6.0;
pub const POOL_Z1: f32 = 14.0;
pub const WATER_Y: f32 = -0.16;
pub const POOL_DEPTH: f32 = 1.55;
/// How far you sink when you are in the pool.
pub const POOL_SINK: f32 = 1.0;

pub fn in_pool_rect(x: f32, z: f32) -> bool {
    x > POOL_X0 && x < POOL_X1 && z > POOL_Z0 && z < POOL_Z1
}

/// The trampoline.
pub const TRAMP_X: f32 = 15.0;
pub const TRAMP_Z: f32 = 12.0;
pub const TRAMP_R: f32 = 1.8;
pub const TRAMP_H: f32 = 0.6;

/// Bar, meat table, smoko pad (centres and sizes from spec section 1).
pub const BAR: Collider = Collider {
    x0: -1.8,
    x1: 1.8,
    z0: -21.95,
    z1: -21.05,
    h: 1.05,
    no_top: false,
    kind: Kind::Bar,
};
pub const MEAT_TABLE: (f32, f32, f32, f32, f32) = (-8.7, -18.2, 1.5, 0.8, 0.8);
pub const SMOKO_X: f32 = -24.0;
pub const SMOKO_Z: f32 = -3.0;

/// Where the chest can sit, as (x, z, rotation). Each round picks a new one.
#[allow(clippy::approx_constant)] // 3.14 is the JavaScript game's own number
pub const CHEST_SPOTS: [(f32, f32, f32); 9] = [
    (-29.0, -21.0, 0.5),
    (29.0, -21.5, -0.5),
    (27.5, 21.8, 2.6),
    (-29.0, 21.8, -2.6),
    (-31.0, 9.0, 1.57),
    (31.0, -9.0, -1.57),
    (13.0, -21.8, 0.0),
    (-13.0, 22.0, 3.14),
    (31.0, 6.0, -1.57),
];

/// Character spawn spots (spec section 1). Shuffled each round and handed out in order.
pub const CHAR_SPAWNS: [(f32, f32); 12] = [
    (0.0, 19.5),
    (-24.0, -19.5),
    (27.0, -9.0),
    (24.0, 16.5),
    (-27.0, 15.0),
    (12.0, -19.5),
    (-9.0, -10.5),
    (30.0, 0.0),
    (-18.0, -6.0),
    (18.0, 4.0),
    (-6.0, 14.0),
    (6.0, -8.0),
];

/// The map features the host can switch off. Off means hidden and no collider.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Features {
    pub bar: bool,
    pub bbq: bool,
    pub chest: bool,
    pub smoko: bool,
}

impl Features {
    pub fn get(&self, f: Feature) -> bool {
        match f {
            Feature::Bar => self.bar,
            Feature::Bbq => self.bbq,
            Feature::Chest => self.chest,
            Feature::Smoko => self.smoko,
        }
    }

    pub fn toggle(&mut self, f: Feature) {
        let v = match f {
            Feature::Bar => &mut self.bar,
            Feature::Bbq => &mut self.bbq,
            Feature::Chest => &mut self.chest,
            Feature::Smoko => &mut self.smoko,
        };
        *v = !*v;
    }
}

impl Default for Features {
    fn default() -> Self {
        Self {
            bar: true,
            bbq: true,
            chest: true,
            smoko: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Yard {
    pub colliders: Vec<Collider>,
    pub features: Features,
    pub chest_spot: usize,
}

impl Yard {
    pub fn new(features: Features, chest_spot: usize) -> Self {
        let mut c = Vec::new();
        let mut pole = Collider::centred(0.0, 3.0, 0.3, 0.3, 2.3).of(Kind::Pole);
        pole.no_top = true;
        c.push(pole); // clothesline pole
        c.push(Collider::centred(22.5, -16.5, 4.0, 3.0, 2.6).of(Kind::Shed)); // shed
        // Eskies: the box is the axis-aligned bounds of a 1 x 0.6 box turned by r.
        for (x, z, r) in [
            (-7.5f32, -4.5f32, 0.3f32),
            (9.0, -6.0, -0.2),
            (-3.0, 18.0, 0.1),
            (24.0, 6.0, 1.4),
        ] {
            let (s, co) = (r.sin().abs(), r.cos().abs());
            c.push(Collider::centred(x, z, co + 0.6 * s, s + 0.6 * co, 0.62).of(Kind::Esky));
        }
        c.push(Collider::centred(6.0, -16.5, 2.0, 1.0, 0.8).of(Kind::Table)); // outdoor table
        if features.bbq {
            c.push(Collider::centred(-6.0, -18.0, 1.7, 0.8, 1.25).of(Kind::Grill)); // grill
            let (x, z, w, d, h) = MEAT_TABLE;
            c.push(Collider::centred(x, z, w, d, h).of(Kind::MeatTable));
        }
        c.push(Collider::centred(31.5, 19.5, 1.7, 0.85, 1.1).of(Kind::Bins)); // wheelie bins
        c.push(Collider::centred(-25.5, -13.5, 2.4, 1.2, 2.3).of(Kind::Crates)); // crates
        c.push(Collider::centred(-31.6, -3.0, 1.2, 9.0, 1.3).of(Kind::Hedge)); // hedge
        c.push(Collider::centred(12.0, 2.0, 1.2, 1.2, 1.0).of(Kind::Tyres)); // tyres
        c.push(Collider::centred(13.2, 2.4, 1.2, 1.2, 0.68).of(Kind::Tyres));
        c.push(Collider::centred(-12.0, -3.0, 2.6, 1.05, 1.0).of(Kind::Woodpile)); // woodpile
        c.push(Collider::centred(20.0, -4.0, 4.5, 0.35, 1.1).of(Kind::Wall)); // brick wall
        c.push(Collider::centred(-8.0, 20.0, 5.0, 0.9, 0.95).of(Kind::Planter)); // veggie planter
        if features.bar {
            c.push(BAR);
        }
        if features.chest {
            let (x, z, _) = CHEST_SPOTS[chest_spot % CHEST_SPOTS.len()];
            c.push(Collider::centred(x, z, 1.45, 1.1, 0.8).of(Kind::Chest));
        }
        Self {
            colliders: c,
            features,
            chest_spot,
        }
    }

    /// Highest standable top under (x, z) for someone at height `y` (and `prev_y` last step).
    /// You can step up onto things when you are at least `h - 0.35` high, or were `h - 0.05` high.
    pub fn floor_at(&self, x: f32, z: f32, y: f32, prev_y: f32) -> f32 {
        let mut floor = 0.0f32;
        for c in &self.colliders {
            if !c.no_top
                && x > c.x0 - 0.1
                && x < c.x1 + 0.1
                && z > c.z0 - 0.1
                && z < c.z1 + 0.1
                && (y >= c.h - 0.35 || prev_y >= c.h - 0.05)
            {
                floor = floor.max(c.h);
            }
        }
        floor
    }
}

impl Default for Yard {
    fn default() -> Self {
        Self::new(Features::default(), 0)
    }
}

/// Is (x, z) inside the yard fence with `margin` to spare?
pub fn inside_yard(x: f32, z: f32, margin: f32) -> bool {
    x.abs() <= YARD_HALF_X - margin && z.abs() <= YARD_HALF_Z - margin
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_yard_has_all_the_boxes() {
        // pole, shed, 4 eskies, table, grill, meat table, bins, crates, hedge, 2 tyres,
        // woodpile, wall, planter, bar, chest
        assert_eq!(Yard::default().colliders.len(), 19);
    }

    #[test]
    fn switching_features_off_removes_their_colliders() {
        let off = Features {
            bar: false,
            bbq: false,
            chest: false,
            smoko: false,
        };
        assert_eq!(Yard::new(off, 0).colliders.len(), 19 - 4);
    }

    #[test]
    fn bar_box_matches_centre_and_size() {
        let b = Collider::centred(0.0, -21.5, 3.6, 0.9, 1.05);
        assert!((b.x0 - BAR.x0).abs() < 1e-5 && (b.z1 - BAR.z1).abs() < 1e-5);
    }

    #[test]
    fn eskies_use_rotated_bounds() {
        let y = Yard::default();
        let e = y
            .colliders
            .iter()
            .find(|c| (c.h - 0.62).abs() < 1e-6)
            .unwrap();
        // first esky: rotation 0.3 -> width cos+0.6 sin
        let w = 0.3f32.cos() + 0.6 * 0.3f32.sin();
        assert!(((e.x1 - e.x0) - w).abs() < 1e-5);
    }

    #[test]
    fn floor_is_the_box_top_when_high_enough() {
        let y = Yard::default();
        // standing over the wall at (20,-4), h 1.1
        assert_eq!(y.floor_at(20.0, -4.0, 1.2, 1.2), 1.1);
        // on the ground next to it you are too low to step up
        assert_eq!(y.floor_at(20.0, -4.0, 0.0, 0.0), 0.0);
        // pole can't be stood on
        assert_eq!(y.floor_at(0.0, 3.0, 5.0, 5.0), 0.0);
    }

    #[test]
    fn every_featured_box_goes_when_its_feature_goes() {
        let mut f = Features::default();
        let all = Yard::new(f, 0).colliders;
        f.toggle(Feature::Bbq);
        let no_bbq = Yard::new(f, 0).colliders;
        assert_eq!(all.len() - no_bbq.len(), 2);
        assert!(
            no_bbq
                .iter()
                .all(|c| c.kind.feature() != Some(Feature::Bbq))
        );
        assert!(!f.get(Feature::Bbq) && f.get(Feature::Bar));
    }

    #[test]
    fn pool_rect() {
        assert!(in_pool_rect(-20.0, 10.0));
        assert!(!in_pool_rect(-10.0, 10.0));
    }

    #[test]
    fn twelve_spawns_all_inside_the_yard() {
        for (x, z) in CHAR_SPAWNS {
            assert!(inside_yard(x, z, 0.45));
        }
    }
}
