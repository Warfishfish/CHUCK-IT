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
    /// Teddy Heist: a base wall, the stone steps round the oval, or a crate in the middle.
    HeistWall,
    Steps,
    HeistCrate,
    BackyardCover,
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

/// Shared with the renderer: two lower boxes and one upper box.
pub const CRATE_SIZE:f32=1.15;
pub const CRATE_CENTRES:[(f32,f32,f32);3]=[(-26.1,0.575,-13.5),(-24.9,0.575,-13.5),(-25.5,1.725,-13.5)];
#[derive(Clone,Copy,Debug)]
pub struct Beam {pub a:crate::vec::V3,pub b:crate::vec::V3,pub radius:f32}
/// Fixed frame shared with clothesline rendering. Fabric remains non-solid.
pub fn clothesline_beams()-> &'static [Beam] {
    static BEAMS:std::sync::OnceLock<Vec<Beam>>=std::sync::OnceLock::new();
    BEAMS.get_or_init(|| {
    use crate::vec::V3;
    let mut beams=Vec::with_capacity(20);
    let angles:Vec<f32>=(0..4).map(|k| k as f32*std::f32::consts::FRAC_PI_2+std::f32::consts::FRAC_PI_4).collect();
    let point=|r:f32,a:f32| V3::new(a.cos()*r,2.32+r*0.09,3.0-a.sin()*r);
    for &a in &angles {beams.push(Beam{a:point(0.0,a),b:point(2.65,a),radius:0.025});}
    for r in [0.8,1.4,2.0,2.55] {for k in 0..4 {beams.push(Beam{a:point(r,angles[k]),b:point(r,angles[(k+1)%4]),radius:0.012});}}
    beams
    }).as_slice()
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

/// The four eskies standing about the yard, as (x, z, rotation). They open (R) but are empty
/// for now: only the Dildo Chest has anything in it.
pub const DECOR_ESKIES: [(f32, f32, f32); 4] = [
    (-7.5, -4.5, 0.3),
    (9.0, -6.0, -0.2),
    (-3.0, 18.0, 0.1),
    (24.0, 6.0, 1.4),
];

static EMPTY_ESKIES: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Turn the four empty yard eskies on or off (the polished look has them off for now, 6 Oct 2026,
/// at Marcus's request; they come back with GRAPHICS_2C.md item A1.7). Off removes the picture,
/// the collider and the "R: open the esky" prompt together. They stay on by default, so the
/// browser look and the tests are unchanged.
pub fn set_empty_eskies(on: bool) {
    EMPTY_ESKIES.store(on, std::sync::atomic::Ordering::Relaxed);
}

/// The yard eskies that are in the game right now (none while they are switched off).
pub fn decor_eskies() -> &'static [(f32, f32, f32)] {
    if EMPTY_ESKIES.load(std::sync::atomic::Ordering::Relaxed) {
        &DECOR_ESKIES
    } else {
        &[]
    }
}

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
            smoko: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Yard {
    /// Separate enlarged race arena; disables legacy pool/trampoline.
    pub race: bool,
    pub colliders: Vec<Collider>,
    pub features: Features,
    pub chest_spot: usize,
    /// Teddy Heist: how many teams the arena is built for.
    pub heist: Option<usize>,
}

impl Yard {
    pub fn new(mut features: Features, chest_spot: usize) -> Self {
        // The standard yard (including Teams and Heist) has no Smoko area.
        // Ignore legacy saved/network feature flags; shared Smoko primitives remain reusable.
        features.smoko = false;
        let mut c = Vec::new();
        let mut pole = Collider::centred(0.0, 3.0, 0.3, 0.3, 2.3).of(Kind::Pole);
        pole.no_top = true;
        c.push(pole); // clothesline pole
        c.push(Collider::centred(22.5, -16.5, 4.0, 3.0, 2.6).of(Kind::Shed)); // shed
        // Eskies: the box is the axis-aligned bounds of a 1.5 x 1.0 box turned by r.
        for &(x, z, r) in decor_eskies() {
            let (s, co) = (r.sin().abs(), r.cos().abs());
            c.push(Collider::centred(x, z, 1.5 * co + 1.0 * s, 1.5 * s + 1.0 * co, 0.75).of(Kind::Esky));
        }
        c.push(Collider::centred(6.0, -16.5, 2.0, 1.0, 0.8).of(Kind::Table)); // outdoor table
        if features.bbq {
            c.push(Collider::centred(-6.0, -18.0, 1.7, 0.8, 1.25).of(Kind::Grill)); // grill
            let (x, z, w, d, h) = MEAT_TABLE;
            c.push(Collider::centred(x, z, w, d, h).of(Kind::MeatTable));
        }
        c.push(Collider::centred(31.5, 19.5, 1.7, 0.85, 1.1).of(Kind::Bins)); // wheelie bins
        for (x,y,z) in CRATE_CENTRES {
            c.push(Collider::centred(x,z,CRATE_SIZE,CRATE_SIZE,y+CRATE_SIZE*0.5).of(Kind::Crates));
        }
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
            heist: None,
            race: false,
        }
    }

    /// The yard turned into the Teddy Heist arena: the usual props stay, and the oval of steps,
    /// the walled bases and the crates in the middle are added.
    pub fn heist(features: Features, chest_spot: usize, teams: usize) -> Self {
        let mut y = Self::new(features, chest_spot);
        y.heist = Some(teams.clamp(2, 4));
        y.colliders
            .extend(crate::heist::arena(teams).into_iter().map(|p| p.collider));
        y
    }

    /// Dedicated Smoko layout; existing maps cannot opt in through legacy flags.
    pub fn smoko(features: Features, chest_spot: usize) -> Self {
        let mut y = Self::new(features, chest_spot);
        y.features.smoko = true;
        y.colliders.retain(|c|matches!(c.kind,Kind::Bar|Kind::Grill|Kind::MeatTable|Kind::Chest|Kind::Esky));
        y.colliders.extend(crate::bush_servo::colliders());
        y
    }

    /// True when `(x, z)` is at least `r` clear of every Heist wall, step and crate.
    pub fn heist_clear(&self, x: f32, z: f32, r: f32) -> bool {
        !self.colliders.iter().any(|c| {
            matches!(c.kind, Kind::HeistWall | Kind::Steps | Kind::HeistCrate)
                && x > c.x0 - r
                && x < c.x1 + r
                && z > c.z0 - r
                && z < c.z1 + r
        })
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
    fn current_maps_ignore_legacy_smoko_flags() {
        assert!(!Features::default().smoko);
        let legacy = Features { smoko: true, ..Features::default() };
        assert!(!Yard::new(legacy, 0).features.smoko);
        for teams in 2..=4 {
            let yard = Yard::heist(legacy, 0, teams);
            assert!(!yard.features.smoko);
            assert_eq!(yard.heist, Some(teams));
            assert!(yard.colliders.iter().any(|c| c.kind == Kind::HeistWall));
        }
    }

    #[test]
    fn full_yard_has_all_the_boxes() {
        // pole, shed, 4 eskies, table, grill, meat table, bins, crates, hedge, 2 tyres,
        // woodpile, wall, planter, bar, chest
        assert_eq!(Yard::default().colliders.len(), 21);
    }

    #[test]
    fn switching_features_off_removes_their_colliders() {
        let off = Features {
            bar: false,
            bbq: false,
            chest: false,
            smoko: false,
        };
        assert_eq!(Yard::new(off, 0).colliders.len(), 21 - 4);
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
            .find(|c| (c.h - 0.75).abs() < 1e-6)
            .unwrap();
        // first esky: rotation 0.3 -> width 1.5 cos + 1.0 sin
        let w = 1.5 * 0.3f32.cos() + 0.3f32.sin();
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

/// Modest cover away from starts, pool, clothesline and pickup stations: x,z,width,depth,height.
pub const BACKYARD_COVER:[(f32,f32,f32,f32,f32);3]=[
 (-8.0,4.5,1.8,1.1,0.9),(7.5,11.0,1.5,1.1,0.75),(17.0,-9.0,1.8,1.1,0.9),
];
impl Yard {
 pub fn set_backyard_cover(&mut self,on:bool) {
  let has=self.colliders.iter().any(|c|c.kind==Kind::BackyardCover);
  if has==on{return;}
  self.colliders.retain(|c|c.kind!=Kind::BackyardCover);
  if on && !self.race {for (x,z,w,d,h) in BACKYARD_COVER {self.colliders.push(Collider::centred(x,z,w,d,h).of(Kind::BackyardCover));}}
 }
}

#[cfg(test)] mod backyard_cover_tests {
 use super::*;
 #[test] fn cover_is_mode_scoped_and_never_duplicates(){let mut y=Yard::default();let n=y.colliders.len();y.set_backyard_cover(true);y.set_backyard_cover(true);assert_eq!(y.colliders.len(),n+3);for (x,z,w,d,h) in BACKYARD_COVER {assert!(h<=1.05);assert!(CHAR_SPAWNS.iter().all(|(a,b)|(*a-x).abs()>w/2.0+0.5 || (*b-z).abs()>d/2.0+0.5));}y.set_backyard_cover(false);assert_eq!(y.colliders.len(),n);}
}
