//! What things look like, as plain data: a list of shapes with size, place and colour.
//!
//! Everything here is copied from the browser game's `MAKERS` code (`public/index.html`), so the
//! Rust version can look the same. It is only data: `bbq_app` turns it into Bevy meshes, and a
//! Blender script can read the same lists later to make `.glb` models.
//!
//! Sizes are in metres. Rotations are Euler angles in X, Y, Z order (like three.js). Parts that
//! the JavaScript game put inside a group are flattened here: the group's move, turn and scale
//! are already worked into each part.

use crate::items::DildoVariant;
use crate::vec::{Quat, V3};

const PI: f32 = std::f32::consts::PI;
const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;

/// A shape, in the same terms as the three.js geometries it came from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    /// `SphereGeometry(r, ws, hs)`.
    Sphere { r: f32, ws: u32, hs: u32 },
    /// `CylinderGeometry(top, bottom, h, seg)`; `caps: false` leaves the ends open.
    Cylinder {
        top: f32,
        bottom: f32,
        h: f32,
        seg: u32,
        caps: bool,
    },
    /// `BoxGeometry(w, h, d)`.
    Cuboid { w: f32, h: f32, d: f32 },
    /// `ConeGeometry(r, h, seg)`: point up, base at the bottom.
    Cone { r: f32, h: f32, seg: u32 },
    /// `TorusGeometry(r, tube, radial, tubular, arc)`: a ring in the XY plane, from the +x axis
    /// round to `arc` radians.
    Torus {
        r: f32,
        tube: f32,
        radial: u32,
        tubular: u32,
        arc: f32,
    },
    /// `CircleGeometry(r, seg)`: a flat disc in the XY plane facing +z.
    Disc { r: f32, seg: u32 },
    /// `ShapeGeometry`: a flat outline in the XY plane facing +z. Points go round the edge.
    Poly(&'static [(f32, f32)]),
    /// `PlaneGeometry(w, h)`: a flat rectangle in the XY plane facing +z, picture 0..1.
    Quad { w: f32, h: f32 },
    /// `RingGeometry(inner, outer, seg)`: a flat ring in the XY plane facing +z.
    Ring { inner: f32, outer: f32, seg: u32 },
    /// `IcosahedronGeometry(r, detail)`: the rough balls used for leaves and clouds. Flat shaded.
    Ico { r: f32, detail: u32 },
    /// `CapsuleGeometry(r, len, cap, radial)`: a sausage, lying along y.
    Capsule {
        r: f32,
        len: f32,
        cap: u32,
        radial: u32,
    },
    /// A cylinder cut in half lengthways (`CylinderGeometry` with a theta length of pi): the lid
    /// of the chest. It spans the angles 0 to pi round y, so the round side faces +z.
    HalfCylinder { r: f32, h: f32, seg: u32 },
    /// A flat rectangle on the ground (facing up) with a rectangular hole, like the lawn with
    /// the pool cut out. The picture is laid by world position: `per_m` repeats per metre.
    Ground {
        x0: f32,
        x1: f32,
        z0: f32,
        z1: f32,
        hole: (f32, f32, f32, f32),
        per_m: f32,
    },
}

/// Pictures drawn in code by the browser game, saved as PNG files for the Rust version.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tex {
    /// The VP can's green label.
    VpLabel,
    /// Fish scales.
    Fish,
    /// Mown lawn with stripes (one tile is 16 m).
    Lawn,
    /// Paling fence.
    Paling,
    /// The house's weatherboards.
    Weatherboard,
    /// The shed's corrugated iron.
    Corrugated,
    /// Rippling pool water.
    PoolWater,
    /// Pool floor tiles.
    PoolFloor,
    /// Pool wall tiles, with the blue mosaic band at the top.
    PoolWall,
    /// Bright wobbly rings on the pool floor.
    Caustics,
    /// The low garden wall.
    Brick,
    /// "THE BAR".
    SignBar,
    /// The little drink tags on the bar front.
    TagVp,
    TagWine,
    TagRum,
    /// The smoko pad's paving.
    SmokoPad,
    /// "SMOKO".
    SignSmoko,
    /// "HANDS OFF", on the meat table.
    SignHands,
}

impl Tex {
    /// The PNG file (in `assets/textures/`) exported from the browser game.
    pub fn file(self) -> &'static str {
        match self {
            Tex::VpLabel => "vp_label.png",
            Tex::Fish => "fish.png",
            Tex::Lawn => "lawn.png",
            Tex::Paling => "paling.png",
            Tex::Weatherboard => "weatherboard.png",
            Tex::Corrugated => "corrugated.png",
            Tex::PoolWater => "pool_water.png",
            Tex::PoolFloor => "pool_floor.png",
            Tex::PoolWall => "pool_wall.png",
            Tex::Caustics => "caustics.png",
            Tex::Brick => "brick.png",
            Tex::SignBar => "sign_bar.png",
            Tex::TagVp => "tag_vp.png",
            Tex::TagWine => "tag_wine.png",
            Tex::TagRum => "tag_rum.png",
            Tex::SmokoPad => "smoko_pad.png",
            Tex::SignSmoko => "sign_smoko.png",
            Tex::SignHands => "sign_hands.png",
        }
    }

    pub const ALL: [Tex; 18] = [
        Tex::VpLabel,
        Tex::Fish,
        Tex::Lawn,
        Tex::Paling,
        Tex::Weatherboard,
        Tex::Corrugated,
        Tex::PoolWater,
        Tex::PoolFloor,
        Tex::PoolWall,
        Tex::Caustics,
        Tex::Brick,
        Tex::SignBar,
        Tex::TagVp,
        Tex::TagWine,
        Tex::TagRum,
        Tex::SmokoPad,
        Tex::SignSmoko,
        Tex::SignHands,
    ];
}

/// How a surface looks. `shine == 0` is the browser's matt "Lambert"; above that it is the shiny
/// "Phong" with a `specular` highlight colour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Surface {
    pub color: u32,
    pub emissive: u32,
    pub specular: u32,
    pub shine: f32,
    pub alpha: f32,
    pub double_sided: bool,
    pub tex: Option<Tex>,
    /// How many times the picture repeats across the shape (`texture.repeat`).
    pub repeat: (f32, f32),
    /// Drawn flat, with no lighting (`MeshBasicMaterial`).
    pub unlit: bool,
    /// Added on top of what is behind it (the pool's shimmer).
    pub additive: bool,
    /// Faceted look: every flat face is shaded on its own.
    pub flat: bool,
    /// Does not throw a shadow (the lawn, clouds, water, signs).
    pub no_shadow: bool,
    /// Flat ground, which is seen at a glancing angle and so needs its own roughness.
    pub ground: bool,
}

impl Surface {
    /// A matt colour (`mat(0xRRGGBB)` in the browser game).
    pub const fn matt(color: u32) -> Self {
        Surface {
            color,
            emissive: 0,
            specular: 0,
            shine: 0.0,
            alpha: 1.0,
            double_sided: false,
            tex: None,
            repeat: (1.0, 1.0),
            unlit: false,
            additive: false,
            flat: false,
            no_shadow: false,
            ground: false,
        }
    }

    /// A shiny colour (`MeshPhongMaterial`).
    pub const fn shiny(color: u32, specular: u32, shine: f32) -> Self {
        Surface {
            color,
            emissive: 0,
            specular,
            shine,
            alpha: 1.0,
            double_sided: false,
            tex: None,
            repeat: (1.0, 1.0),
            unlit: false,
            additive: false,
            flat: false,
            no_shadow: false,
            ground: false,
        }
    }

    pub const fn glow(mut self, emissive: u32) -> Self {
        self.emissive = emissive;
        self
    }

    pub const fn textured(mut self, tex: Tex) -> Self {
        self.tex = Some(tex);
        self
    }

    pub const fn see_through(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    pub const fn both_sides(mut self) -> Self {
        self.double_sided = true;
        self
    }

    /// Repeat the picture `x` times across and `y` times up.
    pub const fn repeating(mut self, x: f32, y: f32) -> Self {
        self.repeat = (x, y);
        self
    }

    pub const fn unlit(mut self) -> Self {
        self.unlit = true;
        self
    }

    pub const fn adding(mut self) -> Self {
        self.additive = true;
        self
    }

    pub const fn faceted(mut self) -> Self {
        self.flat = true;
        self
    }

    pub const fn no_shadow(mut self) -> Self {
        self.no_shadow = true;
        self
    }

    pub const fn ground(mut self) -> Self {
        self.ground = true;
        self
    }
}

/// One shape, placed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    pub shape: Shape,
    pub surface: Surface,
    pub pos: V3,
    pub rot: Quat,
    pub scale: V3,
}

impl Part {
    pub fn new(shape: Shape, surface: Surface) -> Self {
        Part {
            shape,
            surface,
            pos: V3::ZERO,
            rot: Quat::IDENTITY,
            scale: V3::new(1.0, 1.0, 1.0),
        }
    }

    pub fn at(mut self, x: f32, y: f32, z: f32) -> Self {
        self.pos = V3::new(x, y, z);
        self
    }

    /// Turn by Euler angles in X, Y, Z order.
    pub fn turn(mut self, x: f32, y: f32, z: f32) -> Self {
        self.rot = Quat::from_euler_xyz(x, y, z);
        self
    }

    pub fn stretch(mut self, x: f32, y: f32, z: f32) -> Self {
        self.scale = V3::new(x, y, z);
        self
    }

    /// Put this part inside a group that is moved by `at`, turned by `rot` and scaled by `k`
    /// (the same on every axis).
    pub fn inside(mut self, at: V3, rot: Quat, k: f32) -> Self {
        self.pos = at + rot.rotate(self.pos * k);
        self.rot = rot.mul(self.rot);
        self.scale *= k;
        self
    }
}

/// Scale a whole model about its own origin.
pub fn scaled(parts: Vec<Part>, k: f32) -> Vec<Part> {
    parts
        .into_iter()
        .map(|p| p.inside(V3::ZERO, Quat::IDENTITY, k))
        .collect()
}

/// Multiply each of a colour's red, green and blue (the browser game's `Color.multiplyScalar`).
pub fn shade(color: u32, k: f32) -> u32 {
    let ch = |shift: u32| {
        let v = ((color >> shift) & 0xff) as f32 / 255.0;
        ((v * k).clamp(0.0, 1.0) * 255.0).round() as u32
    };
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

fn sphere(r: f32, ws: u32, hs: u32) -> Shape {
    Shape::Sphere { r, ws, hs }
}

fn cyl(top: f32, bottom: f32, h: f32, seg: u32) -> Shape {
    Shape::Cylinder {
        top,
        bottom,
        h,
        seg,
        caps: true,
    }
}

fn torus(r: f32, tube: f32, radial: u32, tubular: u32, arc: f32) -> Shape {
    Shape::Torus {
        r,
        tube,
        radial,
        tubular,
        arc,
    }
}

/// The teddy bear.
pub fn teddy() -> Vec<Part> {
    let fur = Surface::matt(0xa86b3c);
    let lt = Surface::matt(0xe2b98a);
    let dk = Surface::matt(0x24160c);
    let mut v = vec![
        Part::new(sphere(0.17, 14, 10), fur)
            .at(0.0, -0.07, 0.0)
            .stretch(1.0, 1.1, 1.0),
        Part::new(sphere(0.1, 10, 8), lt)
            .at(0.0, -0.06, 0.1)
            .stretch(1.0, 1.0, 0.5),
        Part::new(sphere(0.14, 14, 10), fur).at(0.0, 0.15, 0.0),
    ];
    for s in [-1.0, 1.0] {
        v.push(Part::new(sphere(0.055, 8, 6), fur).at(s * 0.1, 0.27, 0.0));
        v.push(Part::new(sphere(0.065, 8, 6), fur).at(s * 0.17, -0.03, 0.04));
        v.push(Part::new(sphere(0.07, 8, 6), fur).at(s * 0.09, -0.21, 0.07));
        v.push(Part::new(sphere(0.018, 6, 4), dk).at(s * 0.05, 0.19, 0.125));
    }
    v.push(Part::new(sphere(0.055, 8, 6), lt).at(0.0, 0.12, 0.12));
    v.push(Part::new(sphere(0.022, 6, 4), dk).at(0.0, 0.14, 0.17));
    v
}

/// The VP can (a stubby).
pub fn stubby() -> Vec<Part> {
    let alu = Surface::matt(0xc9ced3);
    let label = Surface::matt(0xffffff).textured(Tex::VpLabel);
    vec![
        // the barrel has the label on its side and aluminium ends
        Part::new(
            Shape::Cylinder {
                top: 0.066,
                bottom: 0.066,
                h: 0.18,
                seg: 18,
                caps: false,
            },
            label,
        ),
        Part::new(Shape::Disc { r: 0.066, seg: 18 }, alu)
            .at(0.0, 0.09, 0.0)
            .turn(-HALF_PI, 0.0, 0.0),
        Part::new(Shape::Disc { r: 0.066, seg: 18 }, alu)
            .at(0.0, -0.09, 0.0)
            .turn(HALF_PI, 0.0, 0.0),
        Part::new(cyl(0.056, 0.066, 0.02, 18), alu).at(0.0, 0.1, 0.0),
        Part::new(cyl(0.066, 0.056, 0.02, 18), alu).at(0.0, -0.1, 0.0),
        Part::new(
            Shape::Cuboid {
                w: 0.03,
                h: 0.004,
                d: 0.045,
            },
            Surface::matt(0x9aa0a6),
        )
        .at(0.0, 0.112, 0.015),
    ]
}

/// The garden gnome.
pub fn gnome() -> Vec<Part> {
    vec![
        Part::new(cyl(0.13, 0.18, 0.28, 12), Surface::matt(0x2d5fa8)).at(0.0, -0.14, 0.0),
        Part::new(cyl(0.135, 0.135, 0.04, 12), Surface::matt(0x3b2a1a)).at(0.0, -0.08, 0.0),
        Part::new(sphere(0.1, 12, 10), Surface::matt(0xf2c9a0)).at(0.0, 0.07, 0.0),
        Part::new(
            Shape::Cone {
                r: 0.11,
                h: 0.2,
                seg: 10,
            },
            Surface::matt(0xffffff),
        )
        .at(0.0, -0.02, 0.05)
        .turn(PI, 0.0, 0.0),
        Part::new(sphere(0.035, 8, 6), Surface::matt(0xe8847a)).at(0.0, 0.07, 0.1),
        Part::new(
            Shape::Cone {
                r: 0.12,
                h: 0.3,
                seg: 12,
            },
            Surface::matt(0xd63a2f),
        )
        .at(0.0, 0.28, 0.0),
    ]
}

/// The crack that only the rare Bum-Out gnome has, round the back.
pub fn bum_crack() -> Vec<Part> {
    let mut v = vec![
        Part::new(
            Shape::Cuboid {
                w: 0.016,
                h: 0.15,
                d: 0.01,
            },
            Surface::matt(0x1c3f73),
        )
        .at(0.0, -0.12, -0.168),
    ];
    for sd in [-1.0, 1.0] {
        v.push(
            Part::new(sphere(0.055, 8, 6), Surface::matt(0x6c93c9))
                .at(sd * 0.045, -0.12, -0.145)
                .stretch(1.0, 1.3, 0.6),
        );
    }
    v
}

/// A raw steak. (The snag looks the same.)
pub fn steak() -> Vec<Part> {
    vec![
        Part::new(cyl(0.15, 0.13, 0.05, 10), Surface::matt(0xc4455a)).stretch(1.25, 1.0, 0.9),
        Part::new(torus(0.14, 0.018, 5, 14, PI * 1.1), Surface::matt(0xf2d2c8))
            .at(0.0, 0.012, 0.0)
            .turn(HALF_PI, 0.0, 0.0)
            .stretch(1.25, 0.9, 1.0),
        Part::new(sphere(0.035, 8, 6), Surface::matt(0xf2d2c8))
            .at(0.05, 0.02, 0.02)
            .stretch(1.0, 0.4, 1.0),
    ]
}

const TAIL: &[(f32, f32)] = &[
    (0.0, 0.0),
    (-0.1, 0.1),
    (-0.13, 0.1),
    (-0.075, 0.0),
    (-0.13, -0.1),
    (-0.1, -0.1),
];
const DORSAL: &[(f32, f32)] = &[
    (-0.09, 0.0),
    (-0.07, 0.04),
    (-0.04, 0.065),
    (0.0, 0.07),
    (0.03, 0.055),
    (0.07, 0.0),
];
const ANAL: &[(f32, f32)] = &[(-0.06, 0.0), (-0.045, -0.03), (0.0, -0.035), (0.02, 0.0)];
const PEC: &[(f32, f32)] = &[(0.0, 0.0), (-0.06, 0.02), (-0.07, -0.01), (-0.05, -0.03)];

/// The raw fish. Its nose points along +x.
pub fn fish() -> Vec<Part> {
    let body = Surface::shiny(0xffffff, 0xbfd8e6, 70.0).textured(Tex::Fish);
    let fin = Surface::shiny(0x7d9fb2, 0xaaccdd, 40.0)
        .both_sides()
        .see_through(0.88);
    let gill = Surface::matt(0x2c3e4a);
    let eye_w = Surface::shiny(0xffffff, 0x111111, 90.0);
    let eye_r = Surface::matt(0xd8a23a);
    let eye_p = Surface::shiny(0x0a0a0a, 0x111111, 100.0);
    let lip = Surface::matt(0x9fb5c2);
    let mouth = Surface::matt(0x3a1418);
    let mut v = vec![
        Part::new(sphere(0.1, 24, 16), body).stretch(2.4, 0.95, 0.5),
        // the tail hinges at x = -0.225
        Part::new(sphere(0.035, 10, 8), body)
            .at(-0.225 - 0.02, 0.0, 0.0)
            .stretch(1.4, 0.7, 0.35),
        Part::new(Shape::Poly(TAIL), fin).at(-0.225 - 0.03, 0.0, 0.0),
        Part::new(Shape::Poly(DORSAL), fin).at(0.02, 0.075, 0.0),
        Part::new(Shape::Poly(ANAL), fin).at(-0.08, -0.075, 0.0),
    ];
    for z in [-1.0f32, 1.0] {
        v.push(
            Part::new(Shape::Poly(PEC), fin)
                .at(0.08, -0.03, z * 0.045)
                .turn(0.0, z * 0.5, -0.5),
        );
        v.push(
            Part::new(torus(0.04, 0.004, 4, 12, PI * 0.9), gill)
                .at(0.135, 0.0, z * 0.043)
                .turn(0.0, if z > 0.0 { 0.0 } else { PI }, 0.0)
                .stretch(1.0, 1.2, 1.0),
        );
        v.push(Part::new(sphere(0.026, 12, 10), eye_w).at(0.175, 0.03, z * 0.042));
        v.push(
            Part::new(torus(0.017, 0.004, 5, 12, 2.0 * PI), eye_r)
                .at(0.18, 0.03, z * 0.05)
                .turn(0.0, z * HALF_PI, 0.0),
        );
        v.push(Part::new(sphere(0.013, 8, 6), eye_p).at(0.183, 0.03, z * 0.057));
    }
    v.push(
        Part::new(torus(0.022, 0.009, 6, 14, 2.0 * PI), lip)
            .at(0.235, -0.01, 0.0)
            .turn(0.0, HALF_PI, 0.0),
    );
    v.push(
        Part::new(sphere(0.018, 8, 6), mouth)
            .at(0.232, -0.01, 0.0)
            .stretch(0.6, 1.0, 1.0),
    );
    // the whole fish is drawn three times its modelled size
    scaled(v, 3.0)
}

/// The four pool-noodle colours (pink, yellow, green, blue).
pub const NOODLE_COLOURS: [u32; 4] = [0xff5fa8, 0xffd23f, 0x3fd17a, 0x3fa9ff];

/// A pool noodle. `id` picks the colour.
pub fn noodle(id: u32) -> Vec<Part> {
    let c = NOODLE_COLOURS[(id % 4) as usize];
    let fm = Surface::matt(c);
    let rib = Surface::matt(shade(c, 0.82));
    let dk = Surface::matt(0x3a2a33);
    let (n, h, r) = (7usize, 0.2f32, 0.075f32);
    let mut v = Vec::new();
    let mut y = -0.36;
    for i in 0..n {
        if i > 0 {
            y += h;
        }
        v.push(Part::new(cyl(r, r, h + 0.01, 12), fm).at(0.0, y + h / 2.0, 0.0));
        if i < n - 1 {
            v.push(Part::new(sphere(r, 12, 8), fm).at(0.0, y + h, 0.0));
        }
        v.push(
            Part::new(torus(r, 0.008, 4, 12, 2.0 * PI), rib)
                .at(0.0, y + h * 0.5, 0.0)
                .turn(HALF_PI, 0.0, 0.0),
        );
    }
    v.push(
        Part::new(Shape::Disc { r: 0.03, seg: 10 }, dk)
            .at(0.0, -0.36 - 0.006, 0.0)
            .turn(HALF_PI, 0.0, 0.0),
    );
    v.push(
        Part::new(Shape::Disc { r: 0.03, seg: 10 }, dk)
            .at(0.0, y + h + 0.006, 0.0)
            .turn(-HALF_PI, 0.0, 0.0),
    );
    v
}

/// The base colour of each dildo size (the "cheeky" items in Cheeky mode).
pub fn dildo_colour(v: DildoVariant) -> u32 {
    match v {
        DildoVariant::Classic => 0xa44de0,
        DildoVariant::Mini => 0xff6fb0,
        DildoVariant::Jumbo => 0x5a2a8a,
        DildoVariant::Gold => 0xffd23f,
    }
}

/// The cheeky item, in one of its four sizes: a suction-cup base and a stack of ribbed, veiny
/// segments with a rounded tip. (The JavaScript game also lets it flop about; that comes later.)
pub fn dildo(variant: DildoVariant) -> Vec<Part> {
    let base_c = dildo_colour(variant);
    let mk = |mul: f32, spec: u32, shine: f32| {
        let c = shade(base_c, mul);
        Surface::shiny(c, spec, shine).glow(shade(c, 0.22))
    };
    let body = mk(1.0, 0x9a70b0, 55.0);
    let vein = mk(0.88, 0x9a70b0, 40.0);
    let base = mk(0.72, 0x604070, 50.0);

    let dr = 0.1f32;
    let h = 0.155f32;
    let mut v = vec![
        Part::new(cyl(0.145, 0.172, 0.07, 20), base).at(0.0, -0.17, 0.0),
        Part::new(torus(0.172, 0.019, 6, 20, 2.0 * PI), base)
            .at(0.0, -0.195, 0.0)
            .turn(HALF_PI, 0.0, 0.0),
    ];
    let mut y = -0.145;
    for i in 0..7usize {
        if i > 0 {
            y += h;
        }
        let r0 = dr - i as f32 * 0.004;
        let r1 = dr - (i as f32 + 1.0) * 0.004;
        v.push(Part::new(cyl(r1, r0, 0.167, 16), body).at(0.0, y + h / 2.0, 0.0));
        if i < 6 {
            v.push(Part::new(sphere(r1, 14, 8), body).at(0.0, y + h, 0.0));
        }
        if i <= 4 {
            let r = r0;
            let a = i as f32 * 1.9;
            v.push(
                Part::new(cyl(0.012, 0.012, 0.16, 5), vein)
                    .at(a.cos() * r * 0.93, y + h / 2.0, a.sin() * r * 0.93)
                    .turn(a.sin() * 0.22, 0.0, -a.cos() * 0.22),
            );
            if i % 2 == 0 {
                let b = a + 2.4;
                v.push(
                    Part::new(cyl(0.012, 0.012, 0.16, 5), vein)
                        .at(b.cos() * r * 0.93, y + h * 0.55, b.sin() * r * 0.93)
                        .stretch(1.0, 0.7, 1.0)
                        .turn(-b.sin() * 0.3, 0.0, b.cos() * 0.3),
                );
            }
        }
    }
    // the head: a rounded tip with a flared rim, on the last segment
    v.push(
        Part::new(sphere(0.097, 16, 12), body)
            .at(0.0, y + h + 0.08, 0.0)
            .stretch(1.0, 1.25, 1.0),
    );
    v.push(
        Part::new(torus(0.094, 0.022, 8, 18, 2.0 * PI), body)
            .at(0.0, y + h + 0.02, 0.0)
            .turn(HALF_PI, 0.0, 0.0),
    );
    scaled(v, dildo_scale(variant))
}

/// How big each size is.
pub fn dildo_scale(v: DildoVariant) -> f32 {
    v.def().scale
}

/// The model for any kind of item. `id` is used for the noodle's colour; `variant` for the
/// dildo's size.
pub fn item(kind: crate::items::ItemKind, id: u32, variant: Option<DildoVariant>) -> Vec<Part> {
    use crate::items::ItemKind as K;
    match kind {
        K::Teddy => teddy(),
        K::Stubby => stubby(),
        K::Gnome => gnome(),
        K::Steak | K::Snag => steak(),
        K::Fish => fish(),
        K::Noodle => noodle(id),
        K::Dildo => dildo(variant.unwrap_or(DildoVariant::Classic)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::ItemKind;

    #[test]
    fn every_item_has_a_model() {
        for k in ItemKind::ALL {
            assert!(!item(k, 0, None).is_empty(), "{k:?}");
        }
    }

    #[test]
    fn part_counts_match_the_browser_models() {
        assert_eq!(teddy().len(), 3 + 2 * 4 + 2); // body, belly, head, ears/arms/legs/eyes x2, muzzle, nose
        assert_eq!(stubby().len(), 6);
        assert_eq!(gnome().len(), 6);
        assert_eq!(steak().len(), 3);
        assert_eq!(bum_crack().len(), 3);
        // 5 body parts, then per side: pec, gill, eye white, iris, pupil; plus lip and mouth
        assert_eq!(fish().len(), 5 + 2 * 5 + 2);
        // 7 segments (cylinder + rib), 6 joints, 2 end caps
        assert_eq!(noodle(0).len(), 7 * 2 + 6 + 2);
        // cup, lip, 7 shafts, 6 joints, 5 + 3 veins, head, rim
        assert_eq!(dildo(DildoVariant::Classic).len(), 2 + 7 + 6 + 8 + 2);
    }

    #[test]
    fn fish_is_drawn_three_times_bigger() {
        let body = &fish()[0];
        assert!((body.scale.x - 2.4 * 3.0).abs() < 1e-5);
        assert!((body.scale.z - 0.5 * 3.0).abs() < 1e-5);
        // the nose (lip) sits at 0.235 * 3 along x
        let lip = fish().into_iter().rev().nth(1).unwrap();
        assert!((lip.pos.x - 0.705).abs() < 1e-5);
    }

    #[test]
    fn dildo_sizes_scale_the_whole_model() {
        let top = |v: DildoVariant| dildo(v).iter().map(|p| p.pos.y).fold(f32::MIN, f32::max);
        let classic = top(DildoVariant::Classic);
        let jumbo = top(DildoVariant::Jumbo);
        let mini = top(DildoVariant::Mini);
        assert!((jumbo / classic - 1.38).abs() < 1e-4);
        assert!((mini / classic - 0.66).abs() < 1e-4);
    }

    #[test]
    fn the_dildo_is_about_a_metre_tall_with_its_head_on() {
        let top = dildo(DildoVariant::Classic)
            .iter()
            .map(|p| p.pos.y)
            .fold(f32::MIN, f32::max);
        // base at -0.2, head near 1.02
        assert!((1.0..1.05).contains(&top), "{top}");
    }

    #[test]
    fn noodle_colours_cycle_with_the_id() {
        assert_eq!(noodle(0)[0].surface.color, NOODLE_COLOURS[0]);
        assert_eq!(noodle(5)[0].surface.color, NOODLE_COLOURS[1]);
        // ribs are a darker shade of the same colour
        let rib = noodle(0)[2].surface.color;
        assert_eq!(rib, shade(NOODLE_COLOURS[0], 0.82));
        assert!(rib < NOODLE_COLOURS[0]);
    }

    #[test]
    fn shade_scales_each_channel() {
        assert_eq!(shade(0x808080, 0.5), 0x404040);
        assert_eq!(shade(0xff0000, 2.0), 0xff0000);
        assert_eq!(shade(0x102030, 1.0), 0x102030);
    }

    #[test]
    fn grouped_parts_are_flattened_correctly() {
        // a part at (1, 0, 0) inside a group turned a quarter about y and scaled by 2
        let p = Part::new(
            Shape::Cuboid {
                w: 1.0,
                h: 1.0,
                d: 1.0,
            },
            Surface::matt(0),
        )
        .at(1.0, 0.0, 0.0)
        .inside(
            V3::new(0.0, 5.0, 0.0),
            Quat::from_axis_angle(V3::new(0.0, 1.0, 0.0), HALF_PI),
            2.0,
        );
        assert!(
            p.pos.x.abs() < 1e-5 && (p.pos.y - 5.0).abs() < 1e-5 && (p.pos.z + 2.0).abs() < 1e-5
        );
        assert!((p.scale.x - 2.0).abs() < 1e-6);
    }

    #[test]
    fn only_the_fish_and_the_can_use_pictures() {
        let textured = |parts: Vec<Part>| parts.iter().filter(|p| p.surface.tex.is_some()).count();
        assert_eq!(textured(stubby()), 1);
        assert_eq!(textured(fish()), 2); // the body and the tail stalk
        assert_eq!(textured(teddy()), 0);
        assert_eq!(textured(dildo(DildoVariant::Gold)), 0);
    }

    #[test]
    fn dildo_colours_match_the_variants() {
        assert_eq!(dildo_colour(DildoVariant::Classic), 0xa44de0);
        assert_eq!(dildo_colour(DildoVariant::Gold), 0xffd23f);
        // the body gets a glow of 22% of its own colour
        let p = &dildo(DildoVariant::Gold)[2];
        assert_eq!(p.surface.color, 0xffd23f);
        assert_eq!(p.surface.emissive, shade(0xffd23f, 0.22));
    }
}
