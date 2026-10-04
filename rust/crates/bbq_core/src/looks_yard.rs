//! What the yard looks like, as plain data (see `looks`). Copied from `buildWorld`, the bar,
//! the BBQ, smoko and the chest in the browser game's `public/index.html`.
//!
//! Positions are world positions unless a function says a list is "local" (relative to a point
//! the game moves or turns, like the clothesline head or the chest lid).

use crate::items::DildoVariant;
use crate::looks::{self, Part, Shape, Surface, Tex};
use crate::rng::Rng;
use crate::vec::{Quat, V3, cross};
use crate::yard::{
    BAR, CHEST_SPOTS, DECOR_ESKIES, MEAT_TABLE, POOL_DEPTH, POOL_X0, POOL_X1, POOL_Z0, POOL_Z1, SMOKO_X, SMOKO_Z,
    TRAMP_H, TRAMP_R, TRAMP_X, TRAMP_Z, WATER_Y,
};
use crate::{YARD_HALF_X as W, YARD_HALF_Z as D};

const PI: f32 = std::f32::consts::PI;
const HALF_PI: f32 = std::f32::consts::FRAC_PI_2;
const Y: V3 = V3::new(0.0, 1.0, 0.0);

/// Where the clothesline's turning head sits, and where the chest lid hinges (local to the chest).
pub const HOIST_AT: V3 = V3::new(0.0, 2.2, 3.0);
pub const CHEST_PIVOT: V3 = V3::new(0.0, 0.55, -0.4);
/// The esky's hinge: the back edge of its rim.
pub const ESKY_PIVOT: V3 = V3::new(0.0, 0.64, -0.47);
/// The fence is 1.8 m tall.
pub const FENCE_H: f32 = 1.8;

/// One of the yard's openable eskies.
#[derive(Clone, Debug)]
pub struct Esky {
    pub at: V3,
    pub turn: f32,
    pub base: Vec<Part>,
    pub lid: Vec<Part>,
}

/// Everything the yard is made of.
#[derive(Clone, Debug, Default)]
pub struct YardLook {
    /// Always there.
    pub world: Vec<Part>,
    /// The turning part of the clothesline, local to `HOIST_AT`.
    pub hoist_head: Vec<Part>,
    /// The bar and what is on it.
    pub bar: Vec<Part>,
    /// The BBQ, the prep table, the meat on both.
    pub bbq: Vec<Part>,
    /// The smoko pole, umbrella, esky and signs (the pad and chairs depend on the player count).
    pub smoko: Vec<Part>,
    /// The chest box (local to the chest's own spot and turn).
    pub chest_base: Vec<Part>,
    /// The lid, local to `chest_pivot`.
    pub chest_lid: Vec<Part>,
    /// Where the lid hinges (`CHEST_PIVOT` for the old wooden chest, `ESKY_PIVOT` for the esky).
    pub chest_pivot: V3,
    /// The yard's other eskies (polished look): they open too, but hold nothing yet. Each is
    /// (spot, turn, base parts, lid parts, hinge).
    pub eskies: Vec<Esky>,
    /// Clouds: where each one starts, and its puffs (local).
    pub clouds: Vec<(V3, Vec<Part>)>,
}

fn matt(c: u32) -> Surface {
    Surface::matt(c)
}

fn cuboid(w: f32, h: f32, d: f32, s: Surface, x: f32, y: f32, z: f32) -> Part {
    Part::new(Shape::Cuboid { w, h, d }, s).at(x, y, z)
}

#[allow(clippy::too_many_arguments)] // the shapes are written out like the browser game's calls
fn cyl(top: f32, bottom: f32, h: f32, seg: u32, s: Surface, x: f32, y: f32, z: f32) -> Part {
    Part::new(
        Shape::Cylinder {
            top,
            bottom,
            h,
            seg,
            caps: true,
        },
        s,
    )
    .at(x, y, z)
}

fn torus(r: f32, tube: f32, rad: u32, tub: u32, arc: f32, s: Surface) -> Part {
    Part::new(
        Shape::Torus {
            r,
            tube,
            radial: rad,
            tubular: tub,
            arc,
        },
        s,
    )
}

/// A thin rod from `a` to `b` (the browser game's lines).
pub fn rod(a: V3, b: V3, thickness: f32, s: Surface) -> Part {
    let d = b - a;
    let len = d.len();
    let dir = d.normalised();
    // turn +y onto the direction
    let axis = cross(Y, dir);
    let rot = if axis.len() < 1e-6 {
        if dir.y >= 0.0 {
            Quat::IDENTITY
        } else {
            Quat::from_axis_angle(V3::new(1.0, 0.0, 0.0), PI)
        }
    } else {
        Quat::from_axis_angle(axis, Y.dot(dir).clamp(-1.0, 1.0).acos())
    };
    let mut p = cyl(thickness / 2.0, thickness / 2.0, len, 4, s, 0.0, 0.0, 0.0);
    p.pos = (a + b) * 0.5;
    p.rot = rot;
    p
}

/// Move a list of parts into a group at `at`, turned by `rot` and scaled by `k`.
pub fn place(parts: Vec<Part>, at: V3, rot: Quat, k: f32) -> Vec<Part> {
    parts.into_iter().map(|p| p.inside(at, rot, k)).collect()
}

fn turn_y(a: f32) -> Quat {
    Quat::from_axis_angle(Y, a)
}

/// Give every part of a model one colour (the steaks on the grill are darker).
fn recolour(parts: Vec<Part>, color: u32) -> Vec<Part> {
    parts
        .into_iter()
        .map(|mut p| {
            p.surface.color = color;
            p
        })
        .collect()
}

/// The paling fence panel for a run of `len` metres.
fn fence(len: f32, horizontal: bool, x: f32, z: f32) -> Part {
    let s = matt(0xffffff)
        .textured(Tex::Paling)
        .repeating(len / 2.0, 1.0);
    let (w, d) = if horizontal { (len, 0.1) } else { (0.1, len) };
    cuboid(w, FENCE_H, d, s, x, 0.9, z)
}

/// A chair at the smoko pad, facing +z (turn it to face the middle). Local to the chair.
pub fn chair() -> Vec<Part> {
    let fr = matt(0x2a2a2a);
    let fab = matt(0x1f6fd1);
    let mut v = vec![
        cuboid(0.52, 0.05, 0.46, fab, 0.0, 0.44, 0.0),
        cuboid(0.52, 0.52, 0.05, fab, 0.0, 0.72, -0.24).turn(-0.18, 0.0, 0.0),
    ];
    for (x, z) in [(-0.24, -0.2), (0.24, -0.2), (-0.24, 0.2), (0.24, 0.2)] {
        v.push(cyl(0.018, 0.018, 0.44, 6, fr, x, 0.22, z));
    }
    for x in [-0.28, 0.28] {
        v.push(cuboid(0.05, 0.04, 0.42, fr, x, 0.62, 0.0));
    }
    v.push(cyl(0.04, 0.04, 0.03, 10, matt(0xd63a2f), 0.28, 0.645, 0.12));
    v
}

/// The smoko pad: a flat disc that is `radius` metres across from the middle (the zone's size).
pub fn smoko_pad(radius: f32) -> Part {
    Part::new(
        Shape::Disc { r: 1.0, seg: 40 },
        matt(0xffffff)
            .textured(Tex::SmokoPad)
            .repeating(3.0, 3.0)
            .no_shadow(),
    )
    .at(SMOKO_X, 0.012, SMOKO_Z)
    .turn(-HALF_PI, 0.0, 0.0)
    .stretch(radius, radius, 1.0)
}

/// A shaky tree: trunk plus three rough balls of leaves.
fn gum_tree(rng: &mut Rng, x: f32, z: f32) -> Vec<Part> {
    let leaf = [0x7d9460, 0x6b8656, 0x8ea46b];
    let h = rng.range(6.0, 10.0);
    let mut v = vec![cyl(0.16, 0.32, h, 7, matt(0xd8cfbf), x, h / 2.0, z)];
    for _ in 0..3 {
        let s = rng.range(1.6, 2.6);
        let c = leaf[rng.index(3)];
        let px = x + rng.range(-1.4, 1.4);
        let py = h + rng.range(-0.5, 1.5);
        let pz = z + rng.range(-1.4, 1.4);
        v.push(
            Part::new(Shape::Ico { r: s, detail: 0 }, matt(c).faceted())
                .at(px, py, pz)
                .stretch(1.0, 0.7, 1.0),
        );
    }
    v
}

fn cloud(rng: &mut Rng) -> (V3, Vec<Part>) {
    let mut puffs = Vec::new();
    for _ in 0..4 {
        let r = rng.range(2.5, 4.5);
        let (px, py, pz) = (
            rng.range(-5.0, 5.0),
            rng.range(-0.6, 0.6),
            rng.range(-2.0, 2.0),
        );
        puffs.push(
            Part::new(
                Shape::Ico { r, detail: 1 },
                matt(0xffffff).faceted().glow(0x8899aa).no_shadow(),
            )
            .at(px, py, pz)
            .stretch(1.0, 0.45, 1.0),
        );
    }
    let at = V3::new(
        rng.range(-90.0, 90.0),
        rng.range(32.0, 46.0),
        rng.range(-90.0, 60.0),
    );
    (at, puffs)
}

/// The Hills Hoist: the pole is in `world`; the turning head comes back separately.
fn clothesline() -> (Vec<Part>, Vec<Part>) {
    let grey = matt(0x9aa3a6);
    let pole = vec![cyl(0.05, 0.07, 2.3, 8, grey, 0.0, 1.15, 0.0)];
    let pole = place(pole, V3::new(0.0, 0.0, HOIST_AT.z), Quat::IDENTITY, 1.0);
    let mut head = Vec::new();
    let mut angles = Vec::new();
    for k in 0..4 {
        let a = k as f32 * HALF_PI + PI / 4.0;
        angles.push(a);
        let arm = cyl(0.025, 0.025, 2.7, 6, grey, 1.3, 0.12, 0.0).turn(0.0, 0.0, -HALF_PI + 0.09);
        head.push(arm.inside(V3::ZERO, turn_y(a), 1.0));
    }
    let line = matt(0xeef0ee).unlit().no_shadow();
    for r in [0.8f32, 1.4, 2.0, 2.55] {
        let pts: Vec<V3> = angles
            .iter()
            .map(|a| V3::new(a.cos() * r, 0.12 + r * 0.09, -a.sin() * r))
            .collect();
        for i in 0..pts.len() {
            head.push(rod(pts[i], pts[(i + 1) % pts.len()], 0.012, line));
        }
    }
    for (k, c) in [0xf25c54, 0x4cc9f0, 0xffd23f, 0xffffff]
        .into_iter()
        .enumerate()
    {
        let a = k as f32 * HALF_PI;
        head.push(
            Part::new(Shape::Quad { w: 1.1, h: 0.85 }, matt(c).both_sides())
                .at(a.cos() * 1.5, -0.3, -a.sin() * 1.5)
                .turn(0.0, a + HALF_PI, 0.0),
        );
    }
    (pole, head)
}

fn pool() -> Vec<Part> {
    let (pw, pd) = (POOL_X1 - POOL_X0, POOL_Z1 - POOL_Z0);
    let (pcx, pcz) = ((POOL_X0 + POOL_X1) / 2.0, (POOL_Z0 + POOL_Z1) / 2.0);
    let mut v = Vec::new();
    // the basin: four tiled walls and the floor, facing inwards
    let wall = |repeat: f32| {
        matt(0xffffff)
            .textured(Tex::PoolWall)
            .repeating(repeat, 1.0)
            .no_shadow()
    };
    let mid_y = -POOL_DEPTH / 2.0;
    v.push(
        Part::new(
            Shape::Quad {
                w: pd,
                h: POOL_DEPTH,
            },
            wall(pd),
        )
        .at(POOL_X1, mid_y, pcz)
        .turn(0.0, -HALF_PI, 0.0),
    );
    v.push(
        Part::new(
            Shape::Quad {
                w: pd,
                h: POOL_DEPTH,
            },
            wall(pd),
        )
        .at(POOL_X0, mid_y, pcz)
        .turn(0.0, HALF_PI, 0.0),
    );
    v.push(
        Part::new(
            Shape::Quad {
                w: pw,
                h: POOL_DEPTH,
            },
            wall(pw),
        )
        .at(pcx, mid_y, POOL_Z1)
        .turn(0.0, PI, 0.0),
    );
    v.push(
        Part::new(
            Shape::Quad {
                w: pw,
                h: POOL_DEPTH,
            },
            wall(pw),
        )
        .at(pcx, mid_y, POOL_Z0),
    );
    v.push(
        Part::new(
            Shape::Quad { w: pw, h: pd },
            matt(0xffffff)
                .textured(Tex::PoolFloor)
                .repeating(pw * 2.0, pd * 2.0)
                .no_shadow(),
        )
        .at(pcx, -POOL_DEPTH, pcz)
        .turn(-HALF_PI, 0.0, 0.0),
    );
    // a lane stripe, and the shimmer on the floor
    v.push(
        Part::new(
            Shape::Quad {
                w: pw - 1.2,
                h: 0.22,
            },
            matt(0x1f4f8a).no_shadow(),
        )
        .at(pcx, -POOL_DEPTH + 0.01, pcz)
        .turn(-HALF_PI, 0.0, 0.0),
    );
    v.push(
        Part::new(
            Shape::Quad { w: pw, h: pd },
            matt(0xffffff)
                .textured(Tex::Caustics)
                .repeating(pw / 2.5, pd / 2.5)
                .unlit()
                .adding()
                .see_through(0.28)
                .no_shadow(),
        )
        .at(pcx, -POOL_DEPTH + 0.02, pcz)
        .turn(-HALF_PI, 0.0, 0.0),
    );
    // the water
    v.push(
        Part::new(
            Shape::Quad { w: pw, h: pd },
            matt(0xffffff)
                .textured(Tex::PoolWater)
                .repeating(2.0, 1.3)
                .glow(0x0a3550)
                .see_through(0.74)
                .both_sides()
                .no_shadow(),
        )
        .at(pcx, WATER_Y, pcz)
        .turn(-HALF_PI, 0.0, 0.0),
    );
    // chrome ladder on the house side
    let chrome = matt(0xd8dde2).glow(0x202428);
    for o in [-0.28f32, 0.28] {
        let z = pcz + o;
        v.push(cyl(0.035, 0.035, 1.5, 8, chrome, POOL_X1 - 0.12, -0.55, z));
        v.push(torus(0.2, 0.035, 6, 12, PI, chrome).at(POOL_X1 + 0.08, 0.2, z));
        v.push(cyl(0.035, 0.035, 0.25, 8, chrome, POOL_X1 + 0.28, 0.08, z));
    }
    for k in 0..3 {
        v.push(
            cyl(
                0.025,
                0.025,
                0.56,
                6,
                chrome,
                POOL_X1 - 0.14,
                -0.3 - k as f32 * 0.35,
                pcz,
            )
            .turn(HALF_PI, 0.0, 0.0),
        );
    }
    // the white edge round the top
    let cope = matt(0xe4e9e5);
    v.push(cuboid(pw + 0.6, 0.14, 0.3, cope, pcx, 0.07, POOL_Z0 - 0.15));
    v.push(cuboid(pw + 0.6, 0.14, 0.3, cope, pcx, 0.07, POOL_Z1 + 0.15));
    v.push(cuboid(0.3, 0.14, pd, cope, POOL_X0 - 0.15, 0.07, pcz));
    v.push(cuboid(0.3, 0.14, pd, cope, POOL_X1 + 0.15, 0.07, pcz));
    v
}

fn house() -> Vec<Part> {
    let roof = matt(0x8f3a2b);
    let mut v = vec![
        cuboid(
            40.0,
            4.6,
            8.0,
            matt(0xffffff)
                .textured(Tex::Weatherboard)
                .repeating(20.0, 3.0),
            0.0,
            2.3,
            -29.5,
        ),
        cuboid(41.0, 0.16, 4.9, roof, 0.0, 5.62, -27.4).turn(0.445, 0.0, 0.0),
        cuboid(41.0, 0.16, 4.9, roof, 0.0, 5.62, -31.6).turn(-0.445, 0.0, 0.0),
    ];
    for x in [-15.0, -9.0, -4.0, 8.0, 13.5] {
        v.push(cuboid(1.9, 1.5, 0.06, matt(0xf4f6f2), x, 2.5, -25.47));
        v.push(cuboid(1.6, 1.2, 0.08, matt(0x2d4b66), x, 2.5, -25.44));
    }
    v.push(cuboid(1.1, 2.2, 0.08, matt(0x7a4b2e), 3.0, 1.1, -25.44));
    // the rainwater tank
    v.push(cyl(1.3, 1.3, 2.6, 20, matt(0x9fb0a4), 22.0, 1.3, -27.8));
    v.push(cyl(1.35, 1.35, 0.1, 20, matt(0x8a9a8f), 22.0, 2.62, -27.8));
    v
}

fn props(polished: bool) -> Vec<Part> {
    let mut v = Vec::new();
    // trampoline
    v.push(cyl(
        TRAMP_R,
        TRAMP_R,
        0.04,
        28,
        matt(0x1d1f22),
        TRAMP_X,
        TRAMP_H,
        TRAMP_Z,
    ));
    v.push(
        torus(TRAMP_R + 0.05, 0.12, 8, 28, 2.0 * PI, matt(0x1f6fd1))
            .at(TRAMP_X, TRAMP_H + 0.02, TRAMP_Z)
            .turn(HALF_PI, 0.0, 0.0),
    );
    for k in 0..6 {
        let a = k as f32 * PI / 3.0;
        v.push(cyl(
            0.04,
            0.04,
            TRAMP_H,
            6,
            matt(0x444a50),
            TRAMP_X + a.cos() * TRAMP_R,
            TRAMP_H / 2.0,
            TRAMP_Z + a.sin() * TRAMP_R,
        ));
    }
    // shed
    v.push(cuboid(
        4.0,
        2.5,
        3.0,
        matt(0xffffff).textured(Tex::Corrugated).repeating(6.0, 1.0),
        22.5,
        1.25,
        -16.5,
    ));
    v.push(cuboid(4.4, 0.12, 3.4, matt(0x7f8b85), 22.5, 2.56, -16.5));
    v.push(cuboid(1.3, 2.0, 0.06, matt(0x56645d), 22.5, 1.0, -14.98));
    // eskies
    for (n, (x, z, r)) in DECOR_ESKIES.into_iter().enumerate() {
        if polished {
            continue; // these are real openable eskies in the polished look (see `Esky`)
        }
        // in the polished look the yard's eskies are never the Chest's bright blue
        let body = if polished {
            [0xc9392f, 0x2f8a55, 0xd9822b, 0x7d8a93][n % 4]
        } else {
            0x1f6fd1
        };
        let e = vec![
            cuboid(1.0, 0.5, 0.6, matt(body), 0.0, 0.25, 0.0),
            cuboid(1.04, 0.12, 0.64, matt(0xf4f6f2), 0.0, 0.56, 0.0),
        ];
        v.extend(place(e, V3::new(x, 0.0, z), turn_y(r), 1.0));
    }
    // outdoor table
    v.push(cuboid(2.0, 0.08, 1.0, matt(0xc79a61), 6.0, 0.76, -16.5));
    for (a, b) in [(-0.9, -0.4), (0.9, -0.4), (-0.9, 0.4), (0.9, 0.4)] {
        v.push(cuboid(
            0.07,
            0.72,
            0.07,
            matt(0x8f6a3e),
            6.0 + a,
            0.36,
            -16.5 + b,
        ));
    }
    // wheelie bins
    for (x, c) in [(31.1, 0xd63a2f), (31.95, 0xf2c230)] {
        v.push(cuboid(0.72, 1.0, 0.76, matt(0x2e6b3a), x, 0.5, 19.5));
        v.push(cuboid(0.76, 0.07, 0.82, matt(c), x, 1.04, 19.5));
    }
    // crates
    let crate_m = matt(0xc89a60);
    v.push(cuboid(1.15, 1.15, 1.15, crate_m, -26.1, 0.58, -13.5));
    v.push(cuboid(1.15, 1.15, 1.15, crate_m, -24.9, 0.58, -13.5));
    v.push(cuboid(1.15, 1.15, 1.15, crate_m, -25.5, 1.73, -13.5));
    // hedge
    v.push(cuboid(
        1.2,
        1.3,
        9.0,
        matt(0x3f7d3a).faceted(),
        -31.6,
        0.65,
        -3.0,
    ));
    // stack of old tyres
    let tyre = matt(0x26282b);
    for (n, (x, z)) in [(12.0f32, 2.0f32), (13.2, 2.4)].into_iter().enumerate() {
        for k in 0..(if n == 1 { 2 } else { 3 }) {
            v.push(
                torus(0.42, 0.17, 8, 18, 2.0 * PI, tyre)
                    .at(x, 0.17 + k as f32 * 0.32, z)
                    .turn(HALF_PI, 0.0, 0.0),
            );
        }
    }
    // woodpile
    let (log_m, end_m) = (matt(0x8a5a33), matt(0xd9b48a));
    for row in 0..3 {
        for k in 0..(5 - row) {
            let lx = -12.0 - 1.0 + k as f32 * 0.5 + row as f32 * 0.25;
            let ly = 0.22 + row as f32 * 0.36;
            v.push(cyl(0.2, 0.2, 1.0, 9, log_m, lx, ly, -3.0).turn(HALF_PI, 0.0, 0.0));
            for e in [-0.5f32, 0.5] {
                v.push(
                    Part::new(Shape::Disc { r: 0.19, seg: 9 }, end_m)
                        .at(lx, ly, -3.0 + e * 1.01)
                        .turn(0.0, if e > 0.0 { 0.0 } else { PI }, 0.0),
                );
            }
        }
    }
    // low brick garden wall
    v.push(cuboid(
        4.5,
        1.1,
        0.35,
        matt(0xffffff).textured(Tex::Brick).repeating(4.0, 1.0),
        20.0,
        0.55,
        -4.0,
    ));
    // raised veggie planter
    v.push(cuboid(5.0, 0.6, 0.9, matt(0x7a5a3a), -8.0, 0.3, 20.0));
    v.push(cuboid(4.8, 0.1, 0.75, matt(0x4a3322), -8.0, 0.61, 20.0));
    let mut rng = Rng::new(0x7E6);
    for k in 0..7 {
        v.push(
            Part::new(Shape::Ico { r: 0.28, detail: 0 }, matt(0x4f9a3c).faceted())
                .at(-10.2 + k as f32 * 0.72, 0.82, 20.0 + rng.range(-0.12, 0.12))
                .stretch(1.0, 0.8, 1.0),
        );
    }
    v
}

fn bar() -> Vec<Part> {
    let (bx, bz, bw, bd, bh) = (
        (BAR.x0 + BAR.x1) / 2.0,
        (BAR.z0 + BAR.z1) / 2.0,
        BAR.x1 - BAR.x0,
        BAR.z1 - BAR.z0,
        BAR.h,
    );
    let wood = matt(0x7a4a2a);
    let top = matt(0xc79a61);
    let mut v = vec![
        cuboid(bw, bh - 0.08, bd, wood, bx, (bh - 0.08) / 2.0, bz),
        cuboid(bw + 0.2, 0.08, bd + 0.2, top, bx, bh - 0.04, bz),
    ];
    for k in 0..6 {
        v.push(cuboid(
            0.05,
            bh - 0.2,
            0.02,
            matt(0x5e3820),
            bx - bw / 2.0 + 0.3 + k as f32 * 0.6,
            (bh - 0.2) / 2.0 + 0.05,
            bz + bd / 2.0 + 0.01,
        ));
    }
    // the sign on two posts
    for x in [-1.9, 1.9] {
        v.push(cuboid(
            0.1,
            2.6,
            0.1,
            matt(0x5e3820),
            bx + x,
            1.3,
            bz - 0.35,
        ));
    }
    v.push(cuboid(
        2.6,
        0.55,
        0.06,
        matt(0xffffff).textured(Tex::SignBar),
        bx,
        2.45,
        bz - 0.35,
    ));
    // drink tags along the front
    for (t, x) in [(Tex::TagVp, -1.2), (Tex::TagWine, 0.0), (Tex::TagRum, 1.2)] {
        v.push(
            Part::new(
                Shape::Quad { w: 0.62, h: 0.2 },
                matt(0xffffff).textured(t).unlit().no_shadow(),
            )
            .at(bx + x, bh - 0.2, bz + bd / 2.0 + 0.03),
        );
    }
    // drinks on top: VP stubbies on the left, wine in the middle, rum on the right
    for k in 0..4 {
        let z = bz + if k % 2 == 1 { 0.12 } else { -0.1 };
        v.extend(place(
            looks::stubby(),
            V3::new(bx - 1.55 + k as f32 * 0.22, bh + 0.1, z),
            Quat::IDENTITY,
            1.0,
        ));
    }
    let bottle = |x: f32, glass: u32, label: u32, h: f32| {
        let g = matt(glass).glow(0x100808);
        let parts = vec![
            cyl(0.075, 0.075, h, 12, g, 0.0, h / 2.0, 0.0),
            cyl(0.03, 0.075, 0.08, 12, matt(glass), 0.0, h + 0.04, 0.0),
            cyl(0.028, 0.028, 0.1, 10, matt(glass), 0.0, h + 0.13, 0.0),
            cyl(0.077, 0.077, h * 0.4, 12, matt(label), 0.0, h * 0.45, 0.0),
        ];
        place(parts, V3::new(x, bh, bz - 0.15), Quat::IDENTITY, 1.0)
    };
    v.extend(bottle(-0.12, 0x3a1020, 0xf2efe4, 0.24));
    for x in [0.08f32, 0.3] {
        let glass = matt(0xdfe8ea);
        let parts = vec![
            cyl(0.04, 0.04, 0.01, 10, glass, 0.0, 0.005, 0.0),
            cyl(0.008, 0.008, 0.08, 6, glass, 0.0, 0.045, 0.0),
            cyl(
                0.045,
                0.03,
                0.08,
                10,
                matt(0x8a1d3a).see_through(0.85),
                0.0,
                0.12,
                0.0,
            ),
        ];
        v.extend(place(parts, V3::new(x, bh, bz + 0.15), Quat::IDENTITY, 1.0));
    }
    v.extend(bottle(1.1, 0x9a5a1a, 0x2a2a2a, 0.26));
    for (x, z) in [(1.4f32, 0.12f32), (1.55, -0.05), (1.35, -0.12)] {
        v.push(cyl(
            0.03,
            0.025,
            0.06,
            10,
            matt(0xe6c07a).see_through(0.9),
            bx + x,
            bh + 0.03,
            bz + z,
        ));
    }
    v
}

fn bbq() -> Vec<Part> {
    let mut v = vec![
        cuboid(1.2, 0.7, 0.6, matt(0x3a3f45), -6.0, 0.35, -18.0),
        cuboid(1.6, 0.28, 0.8, matt(0x1f2226), -6.0, 0.84, -18.0),
        cuboid(1.5, 0.24, 0.72, matt(0xb33a2c), -6.0, 1.1, -18.0),
        cyl(0.16, 0.16, 0.55, 12, matt(0xd9dfe0), -7.05, 0.28, -18.0),
    ];
    // on the grill: two steaks and four snags
    let top = 1.23;
    for k in 0..2 {
        v.extend(place(
            recolour(looks::steak(), 0x6e2a1a),
            V3::new(-6.35 + k as f32 * 0.42, top, -18.05),
            Quat::IDENTITY,
            1.0,
        ));
    }
    for k in 0..4 {
        v.push(
            Part::new(
                Shape::Capsule {
                    r: 0.04,
                    len: 0.24,
                    cap: 4,
                    radial: 8,
                },
                matt(0x9b4a2a),
            )
            .at(-5.55 + k as f32 * 0.11, top + 0.02, -17.95)
            .turn(HALF_PI, 0.0, 0.0),
        );
    }
    // the prep table with raw steaks and a plate of fish
    let (mx, mz, mw, md, mh) = MEAT_TABLE;
    v.push(cuboid(mw, 0.06, md, matt(0xc79a61), mx, mh - 0.03, mz));
    for (a, b) in [(-0.65, -0.32), (0.65, -0.32), (-0.65, 0.32), (0.65, 0.32)] {
        v.push(cuboid(
            0.06,
            mh - 0.06,
            0.06,
            matt(0x8f6a3e),
            mx + a,
            (mh - 0.06) / 2.0,
            mz + b,
        ));
    }
    v.push(cuboid(
        0.62,
        0.03,
        0.5,
        matt(0xe8d9b8),
        mx - 0.38,
        mh + 0.015,
        mz,
    ));
    for (k, (a, b)) in [
        (-0.52f32, -0.1f32),
        (-0.24, -0.1),
        (-0.52, 0.12),
        (-0.24, 0.12),
    ]
    .into_iter()
    .enumerate()
    {
        v.extend(place(
            looks::steak(),
            V3::new(mx + a, mh + 0.06, mz + b),
            turn_y(k as f32),
            1.0,
        ));
    }
    v.push(cyl(
        0.36,
        0.3,
        0.03,
        20,
        matt(0xf4f6f2),
        mx + 0.38,
        mh + 0.015,
        mz,
    ));
    for (a, b, r) in [(0.3f32, -0.06f32, 0.4f32), (0.44, 0.08, -0.3)] {
        v.extend(place(
            looks::fish(),
            V3::new(mx + a, mh + 0.08, mz + b),
            turn_y(r),
            0.55,
        ));
    }
    v.push(
        Part::new(
            Shape::Quad { w: 1.1, h: 0.26 },
            matt(0xffffff).textured(Tex::SignHands).unlit().no_shadow(),
        )
        .at(mx, mh - 0.18, mz + md / 2.0 + 0.01),
    );
    v
}

fn smoko() -> Vec<Part> {
    let (sx, sz) = (SMOKO_X, SMOKO_Z);
    let mut v = vec![cyl(0.04, 0.04, 2.6, 8, matt(0xd9dfe0), sx, 1.3, sz)];
    // the umbrella: a yellow cone with a red wire frame
    v.push(
        Part::new(
            Shape::Cone {
                r: 1.7,
                h: 0.55,
                seg: 10,
            },
            matt(0xf2c230).both_sides(),
        )
        .at(sx, 2.55, sz),
    );
    let red = matt(0xd63a2f).unlit();
    let apex = V3::new(sx, 2.55 + 0.28, sz);
    let ring: Vec<V3> = (0..10)
        .map(|i| {
            let a = i as f32 / 10.0 * 2.0 * PI;
            V3::new(sx + 1.72 * a.sin(), 2.55 - 0.28, sz + 1.72 * a.cos())
        })
        .collect();
    for i in 0..10 {
        v.push(rod(apex, ring[i], 0.025, red));
        v.push(rod(ring[i], ring[(i + 1) % 10], 0.025, red));
    }
    // esky and a can on it
    v.push(cuboid(0.7, 0.42, 0.45, matt(0x1f6fd1), sx + 0.35, 0.21, sz));
    v.push(cuboid(0.74, 0.1, 0.49, matt(0xf4f6f2), sx + 0.35, 0.47, sz));
    v.push(cyl(
        0.09,
        0.07,
        0.04,
        12,
        matt(0x8a8f94),
        sx + 0.35,
        0.54,
        sz,
    ));
    // the SMOKO sign, both ways
    let sign = matt(0xffffff)
        .textured(Tex::SignSmoko)
        .unlit()
        .both_sides()
        .no_shadow();
    v.push(Part::new(Shape::Quad { w: 1.1, h: 0.36 }, sign).at(sx, 1.95, sz + 0.05));
    v.push(
        Part::new(Shape::Quad { w: 1.1, h: 0.36 }, sign)
            .at(sx + 0.05, 1.95, sz)
            .turn(0.0, HALF_PI, 0.0),
    );
    v
}

/// The chest's box and lid (see `CHEST_PIVOT`), and the toys that sit in it (local to the chest).
pub fn chest() -> (Vec<Part>, Vec<Part>) {
    let wood = matt(0x6b3f1f);
    let band = matt(0xd9b34a);
    let mut base = vec![cuboid(1.3, 0.55, 0.8, wood, 0.0, 0.275, 0.0)];
    for x in [-0.45, 0.45] {
        base.push(cuboid(0.08, 0.57, 0.82, band, x, 0.275, 0.0));
    }
    base.push(cuboid(0.16, 0.18, 0.04, band, 0.0, 0.45, 0.41));
    let mut lid = vec![
        Part::new(
            Shape::HalfCylinder {
                r: 0.4,
                h: 1.3,
                seg: 16,
            },
            wood,
        )
        .at(0.0, 0.0, 0.4)
        .turn(0.0, 0.0, HALF_PI),
    ];
    for x in [-0.45, 0.45] {
        lid.push(
            Part::new(
                Shape::HalfCylinder {
                    r: 0.41,
                    h: 0.08,
                    seg: 16,
                },
                band,
            )
            .at(x, 0.0, 0.4)
            .turn(0.0, 0.0, HALF_PI),
        );
    }
    (base, lid)
}

/// The Dildo Chest as a big bright-blue esky (the step 2c look): rounded corners, a white rim and
/// lid, a chunky arched handle, fat hinges, finger slots and a few stickers. Same footprint as the
/// chest's collider (about 1.45 x 1.1) and the same lid mechanics, hinged at `ESKY_PIVOT`.
pub fn chest_esky() -> (Vec<Part>, Vec<Part>) {
    esky_with(0x1b7bf0, 0x0e4aa6)
}

/// An esky in any colour (`dark` is the darker trim of the same colour).
pub fn esky_with(body: u32, dark: u32) -> (Vec<Part>, Vec<Part>) {
    let blue = matt(body);
    let navy = matt(dark);
    let white = matt(0xf1f3ee);
    let grey = matt(0x4a5057);
    let (hw, hd) = (0.62f32, 0.38f32);
    let mut base = vec![
        // the body is two overlapping boxes plus four round corners
        cuboid(1.24, 0.6, 0.92, blue, 0.0, 0.32, 0.0),
        cuboid(1.46, 0.6, 0.7, blue, 0.0, 0.32, 0.0),
        // a dark foot band and a white rim
        cuboid(1.48, 0.09, 0.94, navy, 0.0, 0.05, 0.0),
        cuboid(1.5, 0.05, 0.98, white, 0.0, 0.625, 0.0),
    ];
    for (sx, sz) in [(-1.0f32, -1.0f32), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
        base.push(cyl(0.11, 0.11, 0.6, 10, blue, sx * hw, 0.32, sz * hd));
    }
    // finger slots on the ends
    for sx in [-1.0f32, 1.0] {
        base.push(cuboid(0.05, 0.12, 0.36, navy, sx * 0.745, 0.42, 0.0));
    }
    // the fat hinges at the back
    for sx in [-1.0f32, 1.0] {
        base.push(
            cyl(0.055, 0.055, 0.2, 8, grey, sx * 0.45, 0.64, -0.47).turn(0.0, 0.0, HALF_PI),
        );
    }
    // stickers on the front (z is the front face)
    let z = 0.468;
    base.push(cuboid(0.36, 0.2, 0.012, matt(0xe8443a), -0.4, 0.38, z));
    base.push(cuboid(0.2, 0.2, 0.012, matt(0xf2c230), 0.12, 0.3, z).turn(0.0, 0.0, 0.5));
    base.push(cuboid(0.3, 0.12, 0.012, matt(0x2e9e4f), 0.42, 0.44, z));
    base.push(cuboid(0.22, 0.1, 0.012, white, 0.45, 0.22, z).turn(0.0, 0.0, -0.12));
    // the lid, local to ESKY_PIVOT: a slab with a raised blue plate and an arched handle
    let mut lid = vec![
        cuboid(1.5, 0.12, 0.98, white, 0.0, 0.06, 0.49),
        cuboid(1.12, 0.07, 0.62, blue, 0.0, 0.14, 0.49),
        cuboid(1.5, 0.06, 0.08, navy, 0.0, 0.03, 0.97),
    ];
    lid.push(
        Part::new(
            Shape::Torus {
                r: 0.2,
                tube: 0.045,
                radial: 6,
                tubular: 12,
                arc: PI,
            },
            navy,
        )
        .at(0.0, 0.17, 0.49),
    );
    (base, lid)
}

/// The toy in slot `k` of the chest (the sizes cycle: classic, mini, jumbo).
pub fn chest_toy(k: usize) -> Vec<Part> {
    let v = DildoVariant::ALL[k % DildoVariant::ALL.len()];
    let toy = looks::dildo(v);
    let (at, rot, s) = chest_toy_place(k);
    place(toy, at, rot, s)
}

/// Which size is in slot `k`, and where it stands in the chest (local to the chest) and how it is
/// turned and scaled. They are bigger than before (0.9, was 0.52) so they stick well out of the
/// esky and read from a distance.
pub fn chest_toy_variant(k: usize) -> DildoVariant {
    DildoVariant::ALL[k % DildoVariant::ALL.len()]
}

pub fn chest_toy_place(k: usize) -> (V3, Quat, f32) {
    (
        V3::new(-0.42 + k as f32 * 0.42, 0.62, 0.0),
        Quat::from_euler_xyz(-0.5, 0.0, (k as f32 - 1.0) * 0.3),
        0.9,
    )
}

/// A team's colour (the browser game's `TEAMS` table).
pub fn team_colour(t: crate::teams::Team) -> u32 {
    use crate::teams::Team::*;
    match t {
        Red => 0xe8443a,
        Blue => 0x2f7fe0,
        Green => 0x2e9e4f,
        Yellow => 0xf2b705,
        Wildcard => 0xf2c230,
    }
}

/// The little flag on a Heist teddy: a pole and a pennant in its team's colour (local to the
/// teddy; the browser game sets it 0.22 up).
pub fn teddy_flag(team: crate::teams::Team) -> Vec<Part> {
    let parts = vec![
        cyl(0.015, 0.015, 0.32, 6, matt(0x3b2a1a), 0.0, 0.16, 0.0),
        Part::new(
            Shape::Cone {
                r: 0.09,
                h: 0.14,
                seg: 4,
            },
            matt(team_colour(team)),
        )
        .at(0.05, 0.28, 0.0)
        .turn(0.0, 0.0, HALF_PI),
    ];
    place(parts, V3::new(0.0, 0.22, 0.0), Quat::IDENTITY, 1.0)
}

/// The Teddy Heist arena: walls, steps and crates, and for each base the coloured banking pad,
/// its ring, a light beam, a flag pole with a flag and a ball on top. The big scoreboard cube is
/// drawn by the game (its writing changes).
pub fn heist_look(teams: usize) -> Vec<Part> {
    use crate::heist::{self, Tint};
    let mut v = Vec::new();
    for piece in heist::arena(teams) {
        let c = piece.collider;
        let colour = match piece.tint {
            Tint::Team(t) => team_colour(t),
            Tint::Stone(light) => {
                if light {
                    0x80848b
                } else {
                    0x6d7178
                }
            }
            Tint::Crate(small) => {
                if small {
                    0x9a6a40
                } else {
                    0x8a5a36
                }
            }
        };
        v.push(cuboid(
            c.x1 - c.x0,
            c.h,
            c.z1 - c.z0,
            matt(colour),
            (c.x0 + c.x1) / 2.0,
            c.h / 2.0,
            (c.z0 + c.z1) / 2.0,
        ));
    }
    for (def, team) in heist::layout(teams)
        .into_iter()
        .zip(heist::team_keys(teams))
    {
        let col = team_colour(*team);
        let flat = |op: f32| matt(col).unlit().see_through(op).both_sides().no_shadow();
        // the coloured banking pad and its ring
        v.push(
            Part::new(
                Shape::Disc {
                    r: heist::BASE_RADIUS,
                    seg: 40,
                },
                flat(0.4),
            )
            .at(def.x, 0.03, def.z)
            .turn(-HALF_PI, 0.0, 0.0),
        );
        v.push(
            Part::new(
                Shape::Ring {
                    inner: heist::BASE_RADIUS - 0.3,
                    outer: heist::BASE_RADIUS,
                    seg: 48,
                },
                flat(0.9),
            )
            .at(def.x, 0.04, def.z)
            .turn(-HALF_PI, 0.0, 0.0),
        );
        // a light beam you can spot across the yard
        v.push(
            Part::new(
                Shape::Cylinder {
                    top: 0.95,
                    bottom: 0.95,
                    h: 15.0,
                    seg: 18,
                    caps: false,
                },
                flat(0.16),
            )
            .at(def.x, 7.5, def.z),
        );
        v.push(cyl(0.13, 0.13, 7.5, 8, matt(0xf2f2f2), def.x, 3.75, def.z));
        v.push(
            Part::new(
                Shape::Quad { w: 2.6, h: 1.5 },
                matt(col).unlit().both_sides().no_shadow(),
            )
            .at(def.x + 1.3, 6.6, def.z),
        );
        v.push(
            Part::new(
                Shape::Sphere {
                    r: 0.28,
                    ws: 10,
                    hs: 8,
                },
                matt(0xffffff),
            )
            .at(def.x, 7.6, def.z),
        );
    }
    // the scoreboard's pole (the cube itself is the game's)
    v.push(cyl(0.3, 0.3, 7.5, 10, matt(0x9aa0a8), 0.0, 3.75, 0.0));
    v
}

/// A repeatable pseudo-random number from a grid corner (no state, so a rebuild gives the same).
fn hash2(ix: i32, iz: i32, salt: u32) -> f32 {
    let mut h = (ix as u32).wrapping_mul(0x9E37_79B1)
        ^ (iz as u32).wrapping_mul(0x85EB_CA77)
        ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    h = h.wrapping_mul(0x297A_2D39);
    h ^= h >> 15;
    (h & 0xFFFF) as f32 / 65535.0
}

/// Smooth value noise, 0 to 1, with features about `scale` metres across.
fn value_noise(x: f32, z: f32, scale: f32, salt: u32) -> f32 {
    let (fx, fz) = (x / scale, z / scale);
    let (ix, iz) = (fx.floor() as i32, fz.floor() as i32);
    let (tx, tz) = (fx - ix as f32, fz - iz as f32);
    let ease = |t: f32| t * t * (3.0 - 2.0 * t);
    let (ux, uz) = (ease(tx), ease(tz));
    let a = hash2(ix, iz, salt);
    let b = hash2(ix + 1, iz, salt);
    let c = hash2(ix, iz + 1, salt);
    let d = hash2(ix + 1, iz + 1, salt);
    (a + (b - a) * ux) + ((c + (d - c) * ux) - (a + (b - a) * ux)) * uz
}

/// How dry (0 = lush, 1 = bleached) the lawn is at a spot: big slow patches, drier towards the
/// edges and the sunny middle, with a few stubborn green patches that stay lush.
pub fn lawn_dryness(x: f32, z: f32) -> f32 {
    let big = value_noise(x, z, 17.0, 1);
    let mid = value_noise(x, z, 6.0, 2);
    let edge = (x.abs() / W).max(z.abs() / D).clamp(0.0, 1.0);
    let base = 0.38 + 0.55 * big + 0.25 * (mid - 0.5) + 0.20 * edge * edge;
    // a handful of green patches (round the pool and the hose spot, plus noise-picked ones)
    let green = value_noise(x + 40.0, z - 17.0, 9.0, 3);
    let patch = ((green - 0.62) * 6.0).clamp(0.0, 1.0);
    (base * (1.0 - 0.85 * patch)).clamp(0.0, 1.0)
}

/// The colour a lawn corner is multiplied by (over the green lawn picture).
pub fn lawn_tint(x: f32, z: f32) -> [f32; 3] {
    let d = lawn_dryness(x, z);
    let lush = [0.72, 0.90, 0.58];
    let dry = [1.38, 1.10, 0.50];
    [0, 1, 2].map(|i| lush[i] + (dry[i] - lush[i]) * d)
}

/// One soft patch of bare earth: a squashed, turned ellipse in one of a few earthy tones.
#[derive(Clone, Copy, Debug)]
pub struct DirtSpot {
    pub x: f32,
    pub z: f32,
    pub rx: f32,
    pub rz: f32,
    pub turn: f32,
    /// 0 = pale dust, 1 = dark damp earth.
    pub tone: f32,
}

/// Worn dirt, placed the way it really wears: along the paths people walk (back door to the bar,
/// the BBQ, the Hills Hoist, then on to smoko), in a wider scuffed patch where people stand (bar,
/// BBQ, hoist, under the trampoline, smoko), and a few ragged bare patches that creep in from the
/// fence lines. Each patch is a cluster of overlapping ellipses of different sizes and tones so no
/// edge is a neat circle.
pub fn dirt_spots(seed: u64) -> Vec<DirtSpot> {
    let mut rng = Rng::new(seed ^ 0xD127);
    let mut v = Vec::new();
    let mut spot = |rng: &mut Rng, x: f32, z: f32, r: f32, tone: f32| {
        v.push(DirtSpot {
            x,
            z,
            rx: r * rng.range(0.8, 1.35),
            rz: r * rng.range(0.6, 1.0),
            turn: rng.range(0.0, PI),
            tone: (tone + rng.range(-0.2, 0.2)).clamp(0.0, 1.0),
        });
    };
    // paths: a wandering line of small overlaps, widest and darkest in the middle of each leg
    let paths: [&[(f32, f32)]; 3] = [
        &[(3.0, -23.8), (1.0, -20.0), (-4.0, -17.0), (-8.7, -16.4)],
        &[(3.0, -23.8), (3.2, -16.0), (1.5, -8.0), (0.4, 2.2)],
        &[(-8.7, -16.4), (-16.0, -10.0), (-22.6, -4.4)],
    ];
    for pts in paths {
        for w in pts.windows(2) {
            let (a, b) = (w[0], w[1]);
            let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            let n = (len / 0.9).ceil() as usize;
            for k in 0..n {
                let t = k as f32 / n as f32;
                // the line wanders a little, side to side
                let wob = (t * 6.0 + a.0).sin() * 0.3;
                let (nx, nz) = ((b.1 - a.1) / len, -(b.0 - a.0) / len);
                let mid = 1.0 - (2.0 * t - 1.0).abs() * 0.4;
                let (j1, j2, rr) = (rng.range(-0.3, 0.3), rng.range(-0.3, 0.3), rng.range(0.6, 1.0));
                spot(
                    &mut rng,
                    a.0 + (b.0 - a.0) * t + nx * (wob + j1),
                    a.1 + (b.1 - a.1) * t + nz * (wob + j2),
                    rr * mid,
                    0.5,
                );
            }
        }
    }
    // where people stand about: a core of dark earth with scuffed pale dust round it
    for (x, z, r) in [
        (0.0, -18.8, 2.4),
        (-8.7, -16.8, 2.2),
        (0.0, 3.0, 1.8),
        (TRAMP_X, TRAMP_Z, TRAMP_R + 0.6),
        (-24.0, -3.0, 2.6),
    ] {
        for _ in 0..4 {
            let a = rng.range(0.0, 2.0 * PI);
            let d = rng.range(0.3, 1.0) * r * 0.5;
            let rr = r * rng.range(0.45, 0.75);
            spot(&mut rng, x + a.cos() * d, z + a.sin() * d, rr, 0.7);
        }
        for _ in 0..5 {
            let a = rng.range(0.0, 2.0 * PI);
            let d = r * rng.range(0.7, 1.2);
            let rr = r * rng.range(0.25, 0.5);
            spot(&mut rng, x + a.cos() * d, z + a.sin() * d, rr, 0.2);
        }
    }
    // ragged bare patches creeping in from the fence lines and a couple out in the open
    for _ in 0..7 {
        let side = rng.index(4);
        let (x, z) = match side {
            0 => (rng.range(-W + 3.0, W - 3.0), -D + rng.range(1.0, 3.5)),
            1 => (rng.range(-W + 3.0, W - 3.0), D - rng.range(1.0, 3.5)),
            2 => (-W + rng.range(1.0, 3.5), rng.range(-D + 3.0, D - 3.0)),
            _ => (W - rng.range(1.0, 3.5), rng.range(-D + 3.0, D - 3.0)),
        };
        if (POOL_X0 - 3.0..POOL_X1 + 3.0).contains(&x) && (POOL_Z0 - 3.0..POOL_Z1 + 3.0).contains(&z) {
            continue;
        }
        for _ in 0..rng.index(3) + 3 {
            let (dx, dz, rr) = (rng.range(-1.6, 1.6), rng.range(-1.6, 1.6), rng.range(0.5, 1.4));
            spot(&mut rng, x + dx, z + dz, rr, 0.35);
        }
    }
    v
}

/// The whole yard in the browser game's look. `seed` picks where the gum trees and clouds go.
pub fn yard(seed: u64) -> YardLook {
    yard_styled(seed, false)
}

/// The whole yard. `polished` is the step 2c look (patchy dry lawn, worn dirt).
pub fn yard_styled(seed: u64, polished: bool) -> YardLook {
    let mut rng = Rng::new(seed);
    let mut world = Vec::new();
    // lawn and the paddock beyond, both with the pool cut out
    let hole = (POOL_X0, POOL_X1, POOL_Z0, POOL_Z1);
    if polished {
        world.push(Part::new(
            Shape::PatchyLawn {
                x0: -W,
                x1: W,
                z0: -D,
                z1: D,
                hole,
                per_m: 1.0 / 16.0,
                cell: 1.0,
            },
            matt(0xffffff).textured(Tex::Lawn).no_shadow().ground(),
        ));
        // bare earth on top, soft at the edges so it blends into the grass
        let (pale, dark) = ([0xb09a6c_u32, 0x8a6f48_u32], 0.0);
        let _ = dark;
        for d in dirt_spots(seed) {
            // mix pale dust and dark earth by tone
            let mix = |a: u32, b: u32, t: f32| {
                let ch = |s: u32| {
                    let (x, y) = (((a >> s) & 0xff) as f32, ((b >> s) & 0xff) as f32);
                    (x + (y - x) * t).round() as u32
                };
                (ch(16) << 16) | (ch(8) << 8) | ch(0)
            };
            let c = mix(pale[0], 0x6a5236, d.tone);
            let _ = pale[1];
            world.push(
                Part::new(
                    Shape::Disc { r: 1.0, seg: 12 },
                    matt(c).textured(Tex::SoftDot).see_through(0.96).no_shadow(),
                )
                .at(d.x, 0.012, d.z)
                .turn(-HALF_PI, 0.0, d.turn)
                .stretch(d.rx * 1.15, d.rz * 1.15, 1.0),
            );
        }
    } else {
        world.push(Part::new(
            Shape::Ground {
                x0: -W,
                x1: W,
                z0: -D,
                z1: D,
                hole,
                per_m: 1.0 / 16.0,
            },
            matt(0xffffff).textured(Tex::Lawn).no_shadow().ground(),
        ));
    }
    world.push(
        Part::new(
            Shape::Ground {
                x0: -160.0,
                x1: 160.0,
                z0: -160.0,
                z1: 160.0,
                hole,
                per_m: 0.0,
            },
            matt(0x8aaa62).no_shadow().ground(),
        )
        .at(0.0, -0.02, 0.0),
    );
    // the paling fence
    world.push(fence(2.0 * W + 0.2, true, 0.0, -D - 0.05));
    world.push(fence(2.0 * W + 0.2, true, 0.0, D + 0.05));
    world.push(fence(2.0 * D, false, -W - 0.05, 0.0));
    world.push(fence(2.0 * D, false, W + 0.05, 0.0));
    world.extend(house());
    // gum trees, anywhere outside the fence and not in front of the house
    for _ in 0..34 {
        let (mut x, mut z);
        loop {
            x = rng.range(-72.0, 72.0);
            z = rng.range(-66.0, 66.0);
            let inside_fence = x.abs() < W + 3.0 && z.abs() < D + 3.0;
            let house_spot = x.abs() < 22.0 && z < -23.0 && z > -36.0;
            if !inside_fence && !house_spot {
                break;
            }
        }
        world.extend(gum_tree(&mut rng, x, z));
    }
    let clouds: Vec<(V3, Vec<Part>)> = (0..9).map(|_| cloud(&mut rng)).collect();
    let (pole, hoist_head) = clothesline();
    world.extend(pole);
    world.extend(pool());
    world.extend(props(polished));
    let (chest_base, chest_lid) = if polished { chest_esky() } else { chest() };
    let eskies = if polished {
        DECOR_ESKIES
            .into_iter()
            .enumerate()
            .map(|(n, (x, z, r))| {
                let (body, dark) = [(0xc9392f, 0x7d1f19), (0x2f8a55, 0x1b5232), (0xd9822b, 0x8a4c10), (0x7d8a93, 0x4a545c)][n % 4];
                let (base, lid) = esky_with(body, dark);
                Esky { at: V3::new(x, 0.0, z), turn: r, base, lid }
            })
            .collect()
    } else {
        Vec::new()
    };
    YardLook {
        eskies,
        chest_pivot: if polished { ESKY_PIVOT } else { CHEST_PIVOT },
        world,
        hoist_head,
        bar: bar(),
        bbq: bbq(),
        smoko: smoko(),
        chest_base,
        chest_lid,
        clouds,
    }
}

/// Where the chest stands for a spot number, and which way it faces.
pub fn chest_place(spot: usize) -> (V3, f32) {
    let (x, z, r) = CHEST_SPOTS[spot % CHEST_SPOTS.len()];
    (V3::new(x, 0.0, z), r)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(parts: &[Part]) -> (V3, V3) {
        let mut lo = V3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut hi = V3::new(f32::MIN, f32::MIN, f32::MIN);
        for p in parts {
            for (l, h, v) in [
                (&mut lo.x, &mut hi.x, p.pos.x),
                (&mut lo.y, &mut hi.y, p.pos.y),
                (&mut lo.z, &mut hi.z, p.pos.z),
            ] {
                *l = l.min(v);
                *h = h.max(v);
            }
        }
        (lo, hi)
    }

    #[test]
    fn the_yard_has_all_its_parts() {
        let y = yard(1);
        assert!(y.world.len() > 150, "{}", y.world.len());
        assert!(!y.bar.is_empty() && !y.bbq.is_empty() && !y.smoko.is_empty());
        assert_eq!(y.clouds.len(), 9);
        assert!(y.clouds.iter().all(|(_, p)| p.len() == 4));
        assert!(!y.hoist_head.is_empty());
        assert!(!y.chest_base.is_empty() && y.chest_lid.len() == 3);
    }

    #[test]
    fn same_seed_same_trees() {
        assert_eq!(yard(5).world.len(), yard(5).world.len());
        let a = yard(5).world[10].pos;
        let b = yard(5).world[10].pos;
        assert_eq!(a, b);
    }

    #[test]
    fn trees_stay_outside_the_fence_and_out_of_the_houses_way() {
        let y = yard(3);
        // gum-tree trunks are the 7-sided cylinders
        let trunks: Vec<&Part> = y
            .world
            .iter()
            .filter(|p| matches!(p.shape, Shape::Cylinder { seg: 7, .. }))
            .collect();
        assert_eq!(trunks.len(), 34);
        for t in trunks {
            assert!(
                !(t.pos.x.abs() < W + 3.0 && t.pos.z.abs() < D + 3.0),
                "{:?}",
                t.pos
            );
            assert!(!(t.pos.x.abs() < 22.0 && t.pos.z < -23.0 && t.pos.z > -36.0));
        }
    }

    #[test]
    fn the_fence_goes_round_the_yard() {
        let y = yard(1);
        let fences: Vec<&Part> = y
            .world
            .iter()
            .filter(|p| p.surface.tex == Some(Tex::Paling))
            .collect();
        assert_eq!(fences.len(), 4);
        // two long runs (66.2 m) and two short ones (48 m)
        let long = fences
            .iter()
            .filter(|p| (p.surface.repeat.0 - 33.1).abs() < 1e-4)
            .count();
        let short = fences
            .iter()
            .filter(|p| (p.surface.repeat.0 - 24.0).abs() < 1e-4)
            .count();
        assert_eq!((long, short), (2, 2));
        assert!(fences.iter().all(|p| (p.pos.y - 0.9).abs() < 1e-6));
    }

    #[test]
    fn the_house_is_behind_the_back_fence() {
        let y = yard(1);
        let house = y
            .world
            .iter()
            .find(|p| p.surface.tex == Some(Tex::Weatherboard))
            .unwrap();
        assert_eq!(house.pos, V3::new(0.0, 2.3, -29.5));
        assert_eq!(house.surface.repeat, (20.0, 3.0));
    }

    #[test]
    fn the_pool_has_water_a_floor_and_four_walls() {
        let y = yard(1);
        let count = |t: Tex| y.world.iter().filter(|p| p.surface.tex == Some(t)).count();
        assert_eq!(count(Tex::PoolWall), 4);
        assert_eq!(count(Tex::PoolFloor), 1);
        assert_eq!(count(Tex::PoolWater), 1);
        assert_eq!(count(Tex::Caustics), 1);
        let water = y
            .world
            .iter()
            .find(|p| p.surface.tex == Some(Tex::PoolWater))
            .unwrap();
        assert!((water.pos.y - WATER_Y).abs() < 1e-6);
        assert!((water.surface.alpha - 0.74).abs() < 1e-6);
        // walls face inwards: the wall on the +x side points towards -x
        let w = y
            .world
            .iter()
            .find(|p| p.surface.tex == Some(Tex::PoolWall) && (p.pos.x - POOL_X1).abs() < 1e-6)
            .unwrap();
        let n = w.rot.rotate(V3::new(0.0, 0.0, 1.0));
        assert!((n.x + 1.0).abs() < 1e-5);
    }

    #[test]
    fn lawn_has_the_pool_cut_out() {
        let y = yard(1);
        match y.world[0].shape {
            Shape::Ground { hole, per_m, .. } => {
                assert_eq!(hole, (-27.0, -13.0, 6.0, 14.0));
                assert!((per_m - 1.0 / 16.0).abs() < 1e-9);
            }
            _ => panic!("the lawn should be first"),
        }
    }

    #[test]
    fn the_bar_stands_where_the_rules_put_it() {
        let y = yard(1);
        let (lo, hi) = bounds(&y.bar);
        // from the sign posts' x (-1.9, 1.9) plus the drinks, all near the bar's z
        assert!(lo.z > BAR.z0 - 1.0 && hi.z < BAR.z1 + 1.0);
        assert!(hi.y > 2.4, "the sign is 2.45 m up");
        let tags = y
            .bar
            .iter()
            .filter(|p| p.surface.unlit && p.surface.tex.is_some())
            .count();
        assert_eq!(tags, 3);
    }

    #[test]
    fn the_bbq_has_steaks_snags_fish_and_a_sign() {
        let y = yard(1);
        let snags = y
            .bbq
            .iter()
            .filter(|p| matches!(p.shape, Shape::Capsule { .. }))
            .count();
        assert_eq!(snags, 4);
        let sign = y
            .bbq
            .iter()
            .filter(|p| p.surface.tex == Some(Tex::SignHands))
            .count();
        assert_eq!(sign, 1);
        // fish are drawn at 0.55 of their (already 3x) size
        let fish_body = y
            .bbq
            .iter()
            .find(|p| {
                p.surface.tex == Some(Tex::Fish) && matches!(p.shape, Shape::Sphere { ws: 24, .. })
            })
            .unwrap();
        assert!((fish_body.scale.x - 2.4 * 3.0 * 0.55).abs() < 1e-4);
        // the grill steaks are the dark colour
        assert!(y.bbq.iter().any(|p| p.surface.color == 0x6e2a1a));
    }

    #[test]
    fn smoko_has_a_pole_an_umbrella_an_esky_and_two_signs() {
        let y = yard(1);
        let signs = y
            .smoko
            .iter()
            .filter(|p| p.surface.tex == Some(Tex::SignSmoko))
            .count();
        assert_eq!(signs, 2);
        assert!(
            y.smoko
                .iter()
                .any(|p| matches!(p.shape, Shape::Cone { .. }))
        );
        let pad = smoko_pad(3.0);
        assert_eq!(pad.pos, V3::new(SMOKO_X, 0.012, SMOKO_Z));
        assert_eq!(pad.scale, V3::new(3.0, 3.0, 1.0));
        assert_eq!(chair().len(), 2 + 4 + 2 + 1);
    }

    #[test]
    fn the_chest_lid_is_a_half_cylinder_and_the_toys_cycle_through_the_sizes() {
        let (base, lid) = chest();
        assert_eq!(base.len(), 4);
        assert!(matches!(lid[0].shape, Shape::HalfCylinder { .. }));
        let toy0 = chest_toy(0);
        let toy1 = chest_toy(1);
        assert_ne!(toy0[0].surface.color, toy1[0].surface.color);
        // toys are drawn big (0.9), standing well out of the esky
        let (at, _, scale) = chest_toy_place(0);
        assert!(scale > 0.7);
        let top = toy0.iter().map(|p| p.pos.y).fold(f32::MIN, f32::max);
        assert!(top > at.y && top < at.y + 1.6);
    }

    #[test]
    fn rods_run_from_one_end_to_the_other() {
        let r = rod(V3::new(0.0, 0.0, 0.0), V3::new(0.0, 2.0, 0.0), 0.1, matt(0));
        assert!((r.pos.y - 1.0).abs() < 1e-6);
        match r.shape {
            Shape::Cylinder { h, .. } => assert!((h - 2.0).abs() < 1e-6),
            _ => panic!(),
        }
        // a sideways rod: its length axis (y) is turned onto x
        let r = rod(
            V3::new(-1.0, 0.0, 0.0),
            V3::new(1.0, 0.0, 0.0),
            0.1,
            matt(0),
        );
        let axis = r.rot.rotate(V3::new(0.0, 1.0, 0.0));
        assert!((axis.x - 1.0).abs() < 1e-5, "{axis:?}");
        // upside down
        let r = rod(V3::new(0.0, 2.0, 0.0), V3::new(0.0, 0.0, 0.0), 0.1, matt(0));
        assert!((r.rot.rotate(V3::new(0.0, 1.0, 0.0)).y + 1.0).abs() < 1e-5);
    }

    #[test]
    fn the_clothesline_has_four_arms_sixteen_cords_and_four_sheets() {
        let y = yard(1);
        let arms = y
            .hoist_head
            .iter()
            .filter(|p| matches!(p.shape, Shape::Cylinder { seg: 6, .. }))
            .count();
        let cords = y.hoist_head.iter().filter(|p| p.surface.unlit).count();
        let sheets = y
            .hoist_head
            .iter()
            .filter(|p| matches!(p.shape, Shape::Quad { .. }))
            .count();
        assert_eq!((arms, cords, sheets), (4, 16, 4));
    }

    #[test]
    fn the_heist_look_has_walls_pads_beams_and_flags_for_every_team() {
        for n in 2..=4usize {
            let v = heist_look(n);
            let beams = v
                .iter()
                .filter(|p| matches!(p.shape, Shape::Cylinder { h, caps: false, .. } if (h - 15.0).abs() < 1e-5))
                .count();
            assert_eq!(beams, n);
            let pads = v
                .iter()
                .filter(|p| matches!(p.shape, Shape::Disc { seg: 40, .. }))
                .count();
            assert_eq!(pads, n);
            assert!(v.len() > 60, "steps alone are dozens");
            // base walls are in the team colours
            for t in crate::heist::team_keys(n) {
                let walls = v
                    .iter()
                    .filter(|p| {
                        p.surface.color == team_colour(*t)
                            && matches!(p.shape, Shape::Cuboid { .. })
                    })
                    .count();
                assert_eq!(walls, 3);
            }
        }
    }

    #[test]
    fn team_colours_and_the_teddy_flag() {
        use crate::teams::Team;
        assert_eq!(team_colour(Team::Red), 0xe8443a);
        assert_eq!(team_colour(Team::Wildcard), 0xf2c230);
        let f = teddy_flag(Team::Blue);
        assert_eq!(f.len(), 2);
        assert!(f.iter().any(|p| p.surface.color == 0x2f7fe0));
        // the flag sits 0.22 above the teddy's middle
        assert!((f[0].pos.y - (0.22 + 0.16)).abs() < 1e-5);
    }

    #[test]
    fn chests_stand_at_the_rules_spots() {
        for (i, spot) in CHEST_SPOTS.iter().enumerate() {
            let (p, _) = chest_place(i);
            assert_eq!((p.x, p.z), (spot.0, spot.1));
        }
    }
}
