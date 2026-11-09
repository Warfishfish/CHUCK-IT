//! Turns the shape descriptions in `bbq_core::looks` into Bevy meshes.
//!
//! Each builder follows the three.js geometry of the same name step by step, so sizes, the way
//! pictures wrap round a shape and which way each shape faces all match the browser game.

use bbq_core::looks::Shape;
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

const TAU: f32 = std::f32::consts::TAU;
const PI: f32 = std::f32::consts::PI;

/// Positions, normals, uvs and triangle indices, collected before they become a `Mesh`.
#[derive(Default)]
struct Buf {
    pos: Vec<[f32; 3]>,
    nor: Vec<[f32; 3]>,
    uv: Vec<[f32; 2]>,
    /// Colour on each corner, when the shape has any (the patchy lawn).
    col: Vec<[f32; 4]>,
    idx: Vec<u32>,
}

impl Buf {
    /// `uv` is given the three.js way (up is +v); Bevy's pictures run the other way.
    fn vert(&mut self, p: [f32; 3], n: [f32; 3], uv: [f32; 2]) -> u32 {
        self.pos.push(p);
        self.nor.push(n);
        self.uv.push([uv[0], 1.0 - uv[1]]);
        (self.pos.len() - 1) as u32
    }

    fn tri(&mut self, a: u32, b: u32, c: u32) {
        self.idx.extend([a, b, c]);
    }

    fn into_mesh(self) -> Mesh {
        let mut m = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.nor)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uv)
        .with_inserted_indices(Indices::U32(self.idx));
        if !self.col.is_empty() {
            m.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.col);
        }
        m
    }
}

fn norm(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if l < 1e-9 {
        [0.0, 1.0, 0.0]
    } else {
        [v[0] / l, v[1] / l, v[2] / l]
    }
}

/// `SphereGeometry(r, ws, hs)`.
fn sphere(r: f32, ws: u32, hs: u32) -> Buf {
    let mut b = Buf::default();
    let mut grid: Vec<Vec<u32>> = Vec::new();
    for iy in 0..=hs {
        let v = iy as f32 / hs as f32;
        let mut row = Vec::new();
        for ix in 0..=ws {
            let u = ix as f32 / ws as f32;
            let p = [
                -r * (u * TAU).cos() * (v * PI).sin(),
                r * (v * PI).cos(),
                r * (u * TAU).sin() * (v * PI).sin(),
            ];
            row.push(b.vert(p, norm(p), [u, 1.0 - v]));
        }
        grid.push(row);
    }
    for iy in 0..hs as usize {
        for ix in 0..ws as usize {
            let a = grid[iy][ix + 1];
            let bb = grid[iy][ix];
            let c = grid[iy + 1][ix];
            let d = grid[iy + 1][ix + 1];
            if iy != 0 {
                b.tri(a, bb, d);
            }
            if iy != hs as usize - 1 {
                b.tri(bb, c, d);
            }
        }
    }
    b
}

/// `CylinderGeometry(top, bottom, h, seg)` (and `ConeGeometry`, with `top = 0`).
fn cylinder(top: f32, bottom: f32, h: f32, seg: u32, caps: bool) -> Buf {
    cylinder_arc(top, bottom, h, seg, caps, 0.0, TAU)
}

/// A cylinder that only goes part of the way round: from angle `start` for `len` radians
/// (`thetaStart` and `thetaLength` in three.js).
fn cylinder_arc(top: f32, bottom: f32, h: f32, seg: u32, caps: bool, start: f32, len: f32) -> Buf {
    let mut b = Buf::default();
    let half = h / 2.0;
    let slope = (bottom - top) / h;
    // the side: two rings of vertices
    let mut grid: Vec<[u32; 2]> = Vec::new();
    for x in 0..=seg {
        let u = x as f32 / seg as f32;
        let th = u * len + start;
        let (s, c) = th.sin_cos();
        let mut col = [0u32; 2];
        for (y, slot) in col.iter_mut().enumerate() {
            let v = y as f32;
            let radius = v * (bottom - top) + top;
            *slot = b.vert(
                [radius * s, -v * h + half, radius * c],
                norm([s, slope, c]),
                [u, 1.0 - v],
            );
        }
        grid.push(col);
    }
    for x in 0..seg as usize {
        let (a, bb, c, d) = (grid[x][0], grid[x][1], grid[x + 1][1], grid[x + 1][0]);
        if top > 0.0 {
            b.tri(a, bb, d);
        }
        if bottom > 0.0 {
            b.tri(bb, c, d);
        }
    }
    if caps {
        for (is_top, radius) in [(true, top), (false, bottom)] {
            if radius <= 0.0 {
                continue;
            }
            let sign = if is_top { 1.0 } else { -1.0 };
            let centres: Vec<u32> = (0..seg)
                .map(|_| b.vert([0.0, half * sign, 0.0], [0.0, sign, 0.0], [0.5, 0.5]))
                .collect();
            let ring: Vec<u32> = (0..=seg)
                .map(|x| {
                    let u = x as f32 / seg as f32;
                    let th = u * len + start;
                    let (s, c) = th.sin_cos();
                    b.vert(
                        [radius * s, half * sign, radius * c],
                        [0.0, sign, 0.0],
                        [c * 0.5 + 0.5, s * 0.5 * sign + 0.5],
                    )
                })
                .collect();
            for x in 0..seg as usize {
                let (c, i) = (centres[x], ring[x]);
                if is_top {
                    b.tri(i, i + 1, c);
                } else {
                    b.tri(i + 1, i, c);
                }
            }
        }
    }
    b
}

/// `TorusGeometry(r, tube, radial, tubular, arc)`.
fn torus(r: f32, tube: f32, radial: u32, tubular: u32, arc: f32) -> Buf {
    let mut b = Buf::default();
    for j in 0..=radial {
        for i in 0..=tubular {
            let u = i as f32 / tubular as f32 * arc;
            let v = j as f32 / radial as f32 * TAU;
            let p = [
                (r + tube * v.cos()) * u.cos(),
                (r + tube * v.cos()) * u.sin(),
                tube * v.sin(),
            ];
            let centre = [r * u.cos(), r * u.sin(), 0.0];
            b.vert(
                p,
                norm([p[0] - centre[0], p[1] - centre[1], p[2] - centre[2]]),
                [i as f32 / tubular as f32, j as f32 / radial as f32],
            );
        }
    }
    for j in 1..=radial {
        for i in 1..=tubular {
            let a = (tubular + 1) * j + i - 1;
            let bb = (tubular + 1) * (j - 1) + i - 1;
            let c = (tubular + 1) * (j - 1) + i;
            let d = (tubular + 1) * j + i;
            b.tri(a, bb, d);
            b.tri(bb, c, d);
        }
    }
    b
}

/// `CircleGeometry(r, seg)`.
fn disc(r: f32, seg: u32) -> Buf {
    let mut b = Buf::default();
    b.vert([0.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.5, 0.5]);
    for s in 0..=seg {
        let th = s as f32 / seg as f32 * TAU;
        let (sn, cs) = th.sin_cos();
        b.vert(
            [r * cs, r * sn, 0.0],
            [0.0, 0.0, 1.0],
            [(cs + 1.0) / 2.0, (sn + 1.0) / 2.0],
        );
    }
    for i in 1..=seg {
        b.tri(i, i + 1, 0);
    }
    b
}

fn area2(p: &[(f32, f32)]) -> f32 {
    let mut a = 0.0;
    for i in 0..p.len() {
        let (x1, y1) = p[i];
        let (x2, y2) = p[(i + 1) % p.len()];
        a += x1 * y2 - x2 * y1;
    }
    a
}

fn in_tri(p: (f32, f32), a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let cross = |u: (f32, f32), v: (f32, f32), w: (f32, f32)| {
        (v.0 - u.0) * (w.1 - u.1) - (v.1 - u.1) * (w.0 - u.0)
    };
    cross(a, b, p) >= 0.0 && cross(b, c, p) >= 0.0 && cross(c, a, p) >= 0.0
}

/// Ear clipping, for the fish's fins (flat outlines, some of them dented).
/// Returns triangles as indices into `pts`, wound counter-clockwise.
pub fn triangulate(pts: &[(f32, f32)]) -> Vec<[usize; 3]> {
    let mut idx: Vec<usize> = (0..pts.len()).collect();
    if area2(pts) < 0.0 {
        idx.reverse();
    }
    let mut out = Vec::new();
    let mut guard = 0;
    while idx.len() > 3 && guard < 1000 {
        guard += 1;
        let n = idx.len();
        let mut clipped = false;
        for i in 0..n {
            let (ia, ib, ic) = (idx[(i + n - 1) % n], idx[i], idx[(i + 1) % n]);
            let (a, b, c) = (pts[ia], pts[ib], pts[ic]);
            let convex = (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0) > 0.0;
            if !convex {
                continue;
            }
            let blocked = idx
                .iter()
                .any(|&k| k != ia && k != ib && k != ic && in_tri(pts[k], a, b, c));
            if blocked {
                continue;
            }
            out.push([ia, ib, ic]);
            idx.remove(i);
            clipped = true;
            break;
        }
        if !clipped {
            break;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0], idx[1], idx[2]]);
    }
    out
}

/// `ShapeGeometry`: a flat outline facing +z.
fn poly(pts: &[(f32, f32)]) -> Buf {
    let mut b = Buf::default();
    for &(x, y) in pts {
        b.vert([x, y, 0.0], [0.0, 0.0, 1.0], [x, y]);
    }
    for t in triangulate(pts) {
        b.tri(t[0] as u32, t[1] as u32, t[2] as u32);
    }
    b
}

/// `BoxGeometry(w, h, d)`, with three.js's own face order and picture directions (so signs read
/// the right way round and planks run up and down).
fn cuboid(w: f32, h: f32, d: f32) -> Buf {
    let mut b = Buf::default();
    // build one face: axes u, v, w are indices 0, 1, 2 (x, y, z)
    let mut plane = |u: usize,
                     v: usize,
                     wi: usize,
                     udir: f32,
                     vdir: f32,
                     width: f32,
                     height: f32,
                     depth: f32| {
        let (hw, hh) = (width / 2.0, height / 2.0);
        let mut ids = [0u32; 4];
        let mut n = 0;
        for iy in 0..2 {
            let y = iy as f32 * height - hh;
            for ix in 0..2 {
                let x = ix as f32 * width - hw;
                let mut p = [0.0f32; 3];
                p[u] = x * udir;
                p[v] = y * vdir;
                p[wi] = depth / 2.0;
                let mut nr = [0.0f32; 3];
                nr[wi] = if depth > 0.0 { 1.0 } else { -1.0 };
                ids[n] = b.vert(p, nr, [ix as f32, 1.0 - iy as f32]);
                n += 1;
            }
        }
        // ids: 0 = (ix0,iy0), 1 = (ix1,iy0), 2 = (ix0,iy1), 3 = (ix1,iy1)
        let (a, bb, c, dd) = (ids[0], ids[2], ids[3], ids[1]);
        b.tri(a, bb, dd);
        b.tri(bb, c, dd);
    };
    plane(2, 1, 0, -1.0, -1.0, d, h, w); // +x
    plane(2, 1, 0, 1.0, -1.0, d, h, -w); // -x
    plane(0, 2, 1, 1.0, 1.0, w, d, h); // +y
    plane(0, 2, 1, 1.0, -1.0, w, d, -h); // -y
    plane(0, 1, 2, 1.0, -1.0, w, h, d); // +z
    plane(0, 1, 2, -1.0, -1.0, w, h, -d); // -z
    b
}

struct Paling {
    index: usize,
    centre: f32,
    height: f32,
    width: f32,
    lift: f32,
    shade: f32,
    phase: f32,
}

fn palings(len: f32, h: f32, boards: u32, damage: u32) -> impl Iterator<Item = Paling> {
    let count = boards.clamp(4, 64) as usize;
    let step = len / count as f32;
    (0..count).filter_map(move |index| {
        let defect = if index == (damage % 64) as usize { damage / 64 } else { 0 };
        if defect == 1 { return None; }
        let phase = ((index * 7 + 3) % 11) as f32 / 10.0;
        let height = (h + (phase - 0.42) * 0.09) * match defect {
            2 => 0.62,
            3 => 0.80,
            _ => 1.0,
        };
        Some(Paling {
            index,
            centre: -len * 0.5 + step * (index as f32 + 0.5),
            height,
            width: step * [0.91, 0.97, 0.94, 1.0, 0.89][index % 5],
            lift: (height - h) * 0.5 + if defect == 3 { 0.20 } else { 0.0 },
            shade: 0.93 + phase * 0.08,
            phase,
        })
    })
}

/// Locate two neighbouring tops under the bird's feet on an east-west panel.
pub(super) fn paling_perch(shape: &Shape, local_x: f32) -> Option<Vec3> {
    let Shape::PalingPanel { w, h, d, boards, damage } = *shape else { return None; };
    if w <= d || local_x.abs() > w * 0.5 { return None; }
    let mut previous: Option<Paling> = None;
    palings(w, h, boards, damage).filter_map(|board| {
        let left = previous.replace(board)?;
        let right = previous.as_ref()?;
        let y_left = left.lift + left.height * 0.5;
        let y_right = right.lift + right.height * 0.5;
        if left.index + 1 != right.index || (y_left - y_right).abs() > 0.05 { return None; }
        Some(Vec3::new((left.centre + right.centre) * 0.5, (y_left + y_right) * 0.5, 0.0))
    }).min_by(|a, b| (a.x - local_x).abs().total_cmp(&(b.x - local_x).abs()))
}

/// A single mesh section of fence palings with tiny gaps, uneven tops and per-board colour.
fn paling_panel(w: f32, h: f32, d: f32, boards: u32, damage: u32) -> Buf {
    let along_x = w >= d;
    let len = w.max(d);
    let depth = w.min(d);
    let mut out = Buf::default();
    for Paling { index: i, width, height, centre, lift, shade, phase } in palings(len, h, boards, damage) {
        let mut board = if along_x { cuboid(width, height, depth) } else { cuboid(depth, height, width) };
        for p in &mut board.pos {
            if along_x { p[0] += centre; } else { p[2] += centre; }
            p[1] += lift;
        }
        board.col.resize(board.pos.len(), [shade, shade, shade, 1.0]);
        let first = out.pos.len() as u32;
        out.pos.extend(board.pos);
        out.nor.extend(board.nor);
        out.uv.extend(board.uv);
        out.col.extend(board.col);
        out.idx.extend(board.idx.into_iter().map(|index| index + first));
        // A few elongated, low-contrast knots sit on both faces of selected boards. They share
        // the section mesh, so texture detail does not create per-board entities or draw calls.
        if (i * 7 + 3) % 8 == 0 {
            for side in [-1.0f32, 1.0] {
                let mut knot = sphere(0.018, 8, 6);
                let offset = (((i * 3 + 1) % 7) as f32 - 3.0) * width * 0.045;
                for p in &mut knot.pos {
                    if along_x {
                        p[0] = centre + offset + p[0] * 1.1;
                        p[1] = -h * 0.5 + height * (0.35 + phase * 0.18) + p[1] * 2.4;
                        p[2] = side * (depth * 0.5 + 0.002) + p[2] * 0.08;
                    } else {
                        p[0] = side * (depth * 0.5 + 0.002) + p[0] * 0.08;
                        p[1] = -h * 0.5 + height * (0.35 + phase * 0.18) + p[1] * 2.4;
                        p[2] = centre + offset + p[2] * 1.1;
                    }
                }
                knot.col.resize(knot.pos.len(), [0.61, 0.47, 0.34, 1.0]);
                let first = out.pos.len() as u32;
                out.pos.extend(knot.pos);
                out.nor.extend(knot.nor);
                out.uv.extend(knot.uv);
                out.col.extend(knot.col);
                out.idx.extend(knot.idx.into_iter().map(|index| index + first));
            }
        }
    }
    out
}

/// `PlaneGeometry(w, h)`: a rectangle in the XY plane facing +z.
fn quad(w: f32, h: f32) -> Buf {
    let mut b = Buf::default();
    let (hw, hh) = (w / 2.0, h / 2.0);
    let n = [0.0, 0.0, 1.0];
    let a = b.vert([-hw, hh, 0.0], n, [0.0, 1.0]);
    let bb = b.vert([-hw, -hh, 0.0], n, [0.0, 0.0]);
    let c = b.vert([hw, -hh, 0.0], n, [1.0, 0.0]);
    let d = b.vert([hw, hh, 0.0], n, [1.0, 1.0]);
    b.tri(a, bb, d);
    b.tri(bb, c, d);
    b
}

/// `RingGeometry(inner, outer, seg)`: a flat ring in the XY plane facing +z.
fn ring(inner: f32, outer: f32, seg: u32) -> Buf {
    let mut b = Buf::default();
    let n = [0.0, 0.0, 1.0];
    for j in 0..=seg {
        let a = j as f32 / seg as f32 * TAU;
        let (s, c) = a.sin_cos();
        b.vert(
            [inner * c, inner * s, 0.0],
            n,
            [
                (c * inner / outer + 1.0) / 2.0,
                (s * inner / outer + 1.0) / 2.0,
            ],
        );
        b.vert(
            [outer * c, outer * s, 0.0],
            n,
            [(c + 1.0) / 2.0, (s + 1.0) / 2.0],
        );
    }
    for j in 0..seg {
        let i = j * 2;
        b.tri(i, i + 1, i + 3);
        b.tri(i, i + 3, i + 2);
    }
    b
}

// Arena ground paint: one picnic area and its approach to the shelter only.
// Vertex colours reuse the existing repeating dirt texture and material.
const SERVO_GROUND_STEP: f32 = 0.20;
const SERVO_PATH_WIDTH: f32 = 0.72;
const SERVO_WEAR_STRENGTH: f32 = 0.18;
const SERVO_DUST_STRENGTH: f32 = 0.065;
const SERVO_GRIT_STRENGTH: f32 = 0.10;
const SERVO_FOOTPATH: [(f32,f32);5] = [(8.5,-2.5),(7.,-0.5),(5.,1.),(3.,3.),(0.,4.9)];
const SERVO_PICNIC_EARTH_TINT: [f32;3] = [0.96, 0.985, 1.0];
fn servo_ground_colour(x:f32,z:f32)->[f32;4] {
    // Exact neutral boundary prevents colour spreading into the rest of the map.
    if x<=-2. || x>=12. || z<=-5. || z>=7. {return [1.;4];}
    let mut distance=f32::MAX;
    for pair in SERVO_FOOTPATH.windows(2) {
        let (a,b)=(Vec2::from(pair[0]),Vec2::from(pair[1]));
        let p=Vec2::new(x,z);let ab=b-a;
        let t=((p-a).dot(ab)/ab.length_squared()).clamp(0.,1.);
        distance=distance.min((p-(a+ab*t)).length());
    }
    let noise=((x*1.37+z*0.63).sin()+(z*2.13-x*0.51).sin())*0.5;
    let width=SERVO_PATH_WIDTH*(1.+noise*0.17);
    let path=(-((distance/width).powi(2))*1.5).exp();
    let picnic=(-(((x-8.5)/2.25).powi(2)+((z+2.5)/1.55).powi(2))*1.4).exp();
    let wear=path.max(picnic)*(0.86+0.14*noise);
    let fringe=(-((distance/1.45).powi(2))).exp()*(1.-path);
    let border=((x+2.)/0.8).min((12.-x)/0.8).min((z+5.)/0.8).min((7.-z)/0.8).clamp(0.,1.);
    let fade=border*border*(3.-2.*border);
    // Quiet, irregular aggregate flecks soften into the existing grit at distance.
    let fleck=((x*19.1+z*7.3).sin()*(z*23.7-x*11.9).sin()).max(0.).powi(4);
    let grit=fleck*SERVO_GRIT_STRENGTH*(picnic+fringe).min(1.);
    let dust=SERVO_DUST_STRENGTH*fringe;
    let earth=SERVO_PICNIC_EARTH_TINT.map(|t|1.+(t-1.)*fade*picnic);
    [earth[0]*(1.+fade*(-SERVO_WEAR_STRENGTH*wear+dust-grit)),
     earth[1]*(1.+fade*(-SERVO_WEAR_STRENGTH*0.30*wear+dust*1.1-grit)),
     earth[2]*(1.+fade*(SERVO_WEAR_STRENGTH*0.30*wear+dust*1.2-grit*0.8)),1.]
}
/// Same flat 260m ground and original UVs, with a finely sampled local paint patch.
/// Outside the patch, coarse neutral strips retain the original appearance.
pub fn servo_ground_mesh()->Mesh {
    let mut xs=vec![-130.];
    for i in 0..=70 {xs.push(-2.+i as f32*SERVO_GROUND_STEP);}
    xs.push(130.);
    let mut zs=vec![-130.];
    for i in 0..=60 {zs.push(-5.+i as f32*SERVO_GROUND_STEP);}
    zs.push(130.);
    let mut b=Buf::default();
    for &z in &zs {for &x in &xs {
        b.vert([x,-z,0.],[0.,0.,1.],[x/260.+0.5,0.5-z/260.]);
        b.col.push(servo_ground_colour(x,z));
    }}
    let n=xs.len() as u32;
    for j in 0..zs.len()-1 {for i in 0..xs.len()-1 {
        let a=j as u32*n+i as u32;let bb=a+n;let c=bb+1;let d=a+1;
        b.tri(a,bb,d);b.tri(bb,c,d);
    }}
    b.into_mesh()
}

/// Bush Servo road only: chipped shoulders and world-aligned baked surface UVs.
pub fn servo_road_mesh() -> Mesh {
    let mut b=ring(18.,26.,96);
    for (p,uv) in b.pos.iter_mut().zip(b.uv.iter_mut()) {
        let a=p[1].atan2(p[0]);
        let r=p[0].hypot(p[1]);
        let edge=(17.*a).sin()*0.18+(39.*a+1.2).sin()*0.11;
        let new_r=r+edge*if r>22. {1.4}else{1.};
        p[0]=new_r*a.cos();p[1]=new_r*a.sin();
        *uv=[p[0]/58.+0.5,p[1]/58.+0.5];
    }
    b.into_mesh()
}

/// `CapsuleGeometry(r, len, cap, radial)`: a sausage along y.
fn capsule(r: f32, len: f32, cap: u32, radial: u32) -> Buf {
    let mut b = Buf::default();
    // the outline from the bottom pole up the side to the top pole: (distance out, height)
    let mut prof: Vec<(f32, f32, f32, f32)> = Vec::new(); // x, y, nx, ny
    for i in 0..=cap {
        let a = -std::f32::consts::FRAC_PI_2 + i as f32 / cap as f32 * std::f32::consts::FRAC_PI_2;
        prof.push((r * a.cos(), -len / 2.0 + r * a.sin(), a.cos(), a.sin()));
    }
    for i in 0..=cap {
        let a = i as f32 / cap as f32 * std::f32::consts::FRAC_PI_2;
        prof.push((r * a.cos(), len / 2.0 + r * a.sin(), a.cos(), a.sin()));
    }
    let total = prof.len() as f32 - 1.0;
    let mut grid: Vec<Vec<u32>> = Vec::new();
    for j in 0..=radial {
        let u = j as f32 / radial as f32;
        let th = u * TAU;
        let (s, c) = th.sin_cos();
        let col: Vec<u32> = prof
            .iter()
            .enumerate()
            .map(|(k, &(x, y, nx, ny))| {
                b.vert(
                    [x * s, y, x * c],
                    norm([nx * s, ny, nx * c]),
                    [u, k as f32 / total],
                )
            })
            .collect();
        grid.push(col);
    }
    for j in 0..radial as usize {
        for k in 0..prof.len() - 1 {
            let (a, bb, c, d) = (
                grid[j][k],
                grid[j][k + 1],
                grid[j + 1][k + 1],
                grid[j + 1][k],
            );
            b.tri(a, d, bb);
            b.tri(bb, d, c);
        }
    }
    b
}

/// The twelve corners of an icosahedron and its twenty faces.
fn icosahedron() -> (Vec<[f32; 3]>, Vec<[usize; 3]>) {
    let t = (1.0 + 5f32.sqrt()) / 2.0;
    let v = [
        [-1.0, t, 0.0],
        [1.0, t, 0.0],
        [-1.0, -t, 0.0],
        [1.0, -t, 0.0],
        [0.0, -1.0, t],
        [0.0, 1.0, t],
        [0.0, -1.0, -t],
        [0.0, 1.0, -t],
        [t, 0.0, -1.0],
        [t, 0.0, 1.0],
        [-t, 0.0, -1.0],
        [-t, 0.0, 1.0],
    ];
    let f = [
        [0, 11, 5],
        [0, 5, 1],
        [0, 1, 7],
        [0, 7, 10],
        [0, 10, 11],
        [1, 5, 9],
        [5, 11, 4],
        [11, 10, 2],
        [10, 7, 6],
        [7, 1, 8],
        [3, 9, 4],
        [3, 4, 2],
        [3, 2, 6],
        [3, 6, 8],
        [3, 8, 9],
        [4, 9, 5],
        [2, 4, 11],
        [6, 2, 10],
        [8, 6, 7],
        [9, 8, 1],
    ];
    (v.to_vec(), f.to_vec())
}

/// `IcosahedronGeometry(r, detail)`, with flat faces. Each face is cut into `(detail + 1)^2`
/// triangles and pushed out onto the sphere.
fn ico(r: f32, detail: u32) -> Buf {
    let (verts, faces) = icosahedron();
    let on_sphere = |p: [f32; 3]| {
        let n = norm(p);
        [n[0] * r, n[1] * r, n[2] * r]
    };
    let n = detail + 1;
    let mut b = Buf::default();
    let tri = |b: &mut Buf, p: [[f32; 3]; 3]| {
        let q = [on_sphere(p[0]), on_sphere(p[1]), on_sphere(p[2])];
        let e1 = [q[1][0] - q[0][0], q[1][1] - q[0][1], q[1][2] - q[0][2]];
        let e2 = [q[2][0] - q[0][0], q[2][1] - q[0][1], q[2][2] - q[0][2]];
        let nrm = norm([
            e1[1] * e2[2] - e1[2] * e2[1],
            e1[2] * e2[0] - e1[0] * e2[2],
            e1[0] * e2[1] - e1[1] * e2[0],
        ]);
        let ids: Vec<u32> = q.iter().map(|p| b.vert(*p, nrm, [0.0, 0.0])).collect();
        b.tri(ids[0], ids[1], ids[2]);
    };
    let lerp = |a: [f32; 3], c: [f32; 3], t: f32| {
        [
            a[0] + (c[0] - a[0]) * t,
            a[1] + (c[1] - a[1]) * t,
            a[2] + (c[2] - a[2]) * t,
        ]
    };
    for f in faces {
        let (a, bb, c) = (verts[f[0]], verts[f[1]], verts[f[2]]);
        // the point at row i, column j of the cut-up face
        let pt = |i: u32, j: u32| {
            let ab = lerp(a, bb, i as f32 / n as f32);
            let ac = lerp(a, c, i as f32 / n as f32);
            if i == 0 {
                a
            } else {
                lerp(ab, ac, j as f32 / i as f32)
            }
        };
        for i in 0..n {
            for j in 0..=i {
                tri(&mut b, [pt(i, j), pt(i + 1, j), pt(i + 1, j + 1)]);
                if j < i {
                    tri(&mut b, [pt(i, j), pt(i + 1, j + 1), pt(i, j + 1)]);
                }
            }
        }
    }
    b
}

/// A flat rectangle on the ground with a rectangular hole, picture laid by world position.
fn ground(x0: f32, x1: f32, z0: f32, z1: f32, hole: (f32, f32, f32, f32), per_m: f32) -> Buf {
    let mut b = Buf::default();
    let (hx0, hx1, hz0, hz1) = hole;
    let n = [0.0, 1.0, 0.0];
    // four rectangles round the hole: back, front, left, right
    let rects = [
        (x0, x1, z0, hz0),
        (x0, x1, hz1, z1),
        (x0, hx0, hz0, hz1),
        (hx1, x1, hz0, hz1),
    ];
    for (ax, bx, az, bz) in rects {
        if bx <= ax || bz <= az {
            continue;
        }
        let p = |x: f32, z: f32, b: &mut Buf| b.vert([x, 0.0, z], n, [x * per_m, -z * per_m]);
        let a = p(ax, az, &mut b);
        let bb = p(bx, az, &mut b);
        let c = p(bx, bz, &mut b);
        let d = p(ax, bz, &mut b);
        // counter-clockwise seen from above
        b.tri(a, d, c);
        b.tri(a, c, bb);
    }
    b
}

/// The lawn cut into squares (and along the hole's edges), each corner coloured by
/// `looks_yard::lawn_tint`. The hole is skipped cell by cell, so the cut lines up with the pool.
fn patchy_lawn(x0: f32, x1: f32, z0: f32, z1: f32, hole: (f32, f32, f32, f32), per_m: f32, cell: f32) -> Buf {
    let (hx0, hx1, hz0, hz1) = hole;
    let lines = |a: f32, b: f32, extra: [f32; 2]| -> Vec<f32> {
        let n = ((b - a) / cell).ceil() as i32;
        let mut v: Vec<f32> = (0..=n).map(|i| (a + i as f32 * cell).min(b)).collect();
        v.extend(extra.into_iter().filter(|e| *e > a && *e < b));
        v.sort_by(|p, q| p.total_cmp(q));
        v.dedup_by(|p, q| (*p - *q).abs() < 1e-4);
        v
    };
    let xs = lines(x0, x1, [hx0, hx1]);
    let zs = lines(z0, z1, [hz0, hz1]);
    let mut b = Buf::default();
    let n = [0.0, 1.0, 0.0];
    for zw in zs.windows(2) {
        for xw in xs.windows(2) {
            let (cx, cz) = ((xw[0] + xw[1]) / 2.0, (zw[0] + zw[1]) / 2.0);
            if cx > hx0 && cx < hx1 && cz > hz0 && cz < hz1 {
                continue;
            }
            let mut corner = |x: f32, z: f32, b: &mut Buf| {
                let t = bbq_core::looks_yard::lawn_tint(x, z);
                let i = b.vert([x, 0.0, z], n, [x * per_m, -z * per_m]);
                b.col.push([t[0], t[1], t[2], 1.0]);
                i
            };
            let a = corner(xw[0], zw[0], &mut b);
            let bb = corner(xw[1], zw[0], &mut b);
            let c = corner(xw[1], zw[1], &mut b);
            let d = corner(xw[0], zw[1], &mut b);
            b.tri(a, d, c);
            b.tri(a, c, bb);
        }
    }
    b
}

/// Split each face into a centre, four edge strips and four corners. Project their
/// shared boundary onto a rounded box so adjacent faces meet with the same normal.
fn rounded_box(w: f32, h: f32, d: f32, radius: f32, per_m: f32) -> Buf {
    let half = Vec3::new(w, h, d) * 0.5;
    let radius = radius.min(half.min_element() * 0.95);
    let inner = half - Vec3::splat(radius);
    let faces = cuboid(w, h, d);
    let mut mesh = Buf::default();
    for face in faces.pos.chunks_exact(4) {
        let origin = Vec3::from_array(face[0]);
        let u = Vec3::from_array(face[1]) - origin;
        let v = Vec3::from_array(face[2]) - origin;
        let du = radius / u.length();
        let dv = radius / v.length();
        let repeats = if per_m > 0.0 { Vec2::new(u.length(), v.length()) * per_m } else { Vec2::ONE };
        let first = mesh.pos.len() as u32;
        for y in [0.0, dv, 1.0 - dv, 1.0] {
            for x in [0.0, du, 1.0 - du, 1.0] {
                let point = origin + u * x + v * y;
                let core = point.clamp(-inner, inner);
                let normal = (point - core).normalize();
                mesh.vert((core + normal * radius).to_array(), normal.to_array(),
                    [x * repeats.x, 1.0 - y * repeats.y]);
            }
        }
        for row in 0..3 {
            for col in 0..3 {
                let a = first + row * 4 + col;
                mesh.tri(a, a + 4, a + 1);
                mesh.tri(a + 4, a + 5, a + 1);
            }
        }
    }
    mesh
}

/// Revolve two quarter-circle rims between a flat cap and the straight wall.
/// Duplicate cap vertices to keep the circular cap UVs separate from the wall seam.
fn rolled_cylinder(r: f32, h: f32, seg: u32, bevel: f32, per_m: f32) -> Buf {
    let half = h * 0.5;
    let bevel = bevel.min(r.min(half) * 0.95);
    let cap_r = r - bevel;
    let mut mesh = Buf::default();
    for (side, angles) in [(1.0, [90.0_f32, 45.0, 0.0]), (-1.0, [0.0, 45.0, 90.0])] {
        for angle in angles {
            let (rise, radial) = angle.to_radians().sin_cos();
            let ring_r = cap_r + bevel * radial;
            let y = side * (half - bevel + bevel * rise);
            for i in 0..=seg {
                let u = i as f32 / seg as f32;
                let (x, z) = (u * TAU).sin_cos();
                let uv = if per_m > 0.0 {
                    [u * TAU * r * per_m, 1.0 - (half - y) * per_m]
                } else {
                    [u, 0.5 + y / h]
                };
                mesh.vert([x * ring_r, y, z * ring_r], [x * radial, side * rise, z * radial], uv);
            }
        }
    }
    let stride = seg + 1;
    for row in 0..5 {
        for i in 0..seg {
            let a = row * stride + i;
            mesh.tri(a, a + stride, a + 1);
            mesh.tri(a + stride, a + stride + 1, a + 1);
        }
    }
    for side in [1.0, -1.0] {
        let normal = [0.0, side, 0.0];
        let centre = mesh.vert([0.0, half * side, 0.0], normal, [0.5, 0.5]);
        let uv_radius = if per_m > 0.0 { cap_r * per_m } else { 0.5 };
        for i in 0..=seg {
            let (x, z) = (TAU * i as f32 / seg as f32).sin_cos();
            mesh.vert([x * cap_r, half * side, z * cap_r], normal,
                [0.5 + z * uv_radius, 0.5 + x * uv_radius * side]);
        }
        for i in 0..seg {
            let a = centre + 1 + i;
            if side > 0.0 { mesh.tri(centre, a, a + 1); }
            else { mesh.tri(centre, a + 1, a); }
        }
    }
    mesh
}

/// Low timber and metal pick up dirt. Exposed top edges get a warmer tint, with a repeatable
/// low-amplitude colour grain so repeated mesh builds keep their finish without adding textures.
pub fn weather_by_height(mesh: &mut Mesh) {
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
        return;
    };
    if pos.is_empty() { return; }
    let (low, high) = pos.iter().fold((f32::MAX, f32::MIN), |(lo, hi), p| (lo.min(p[1]), hi.max(p[1])));
    let colours = pos.iter().map(|p| {
        let t = ((p[1] - low) / (high - low).max(1e-5)).clamp(0.0, 1.0);
        // Fixed seed + vertex position means builds are stable, and coincident vertices share
        // the same tint. The variation remains subtle enough not to read as a pattern.
        let sample = ((p[0] * 127.1 + p[1] * 311.7 + p[2] * 74.7 + 19.19).sin() * 43_758.547).fract().abs();
        let grain = 0.98 + sample * 0.04;
        [
            (0.79 + 0.21 * t) * grain,
            (0.75 + 0.24 * t) * grain,
            (0.72 + 0.24 * t) * grain,
            1.0,
        ]
    }).collect::<Vec<_>>();
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
}

/// A deterministic, tiny tilt and size change for weathered scenery. The chosen scale is
/// clamped from the rotated box's projected extents so its mesh never escapes the authored box
/// that gameplay collision uses. The adjustment is transform-only and does not duplicate meshes.
pub fn seeded_weathered_adjustment(shape: &Shape, at: bbq_core::vec::V3) -> Option<(Quat, f32)> {
    let Shape::Cuboid { w, h, d } = *shape else { return None };
    if w <= 0.0 || h <= 0.0 || d <= 0.0 { return None; }
    let seed = at.x * 127.1 + at.y * 311.7 + at.z * 74.7 + w * 19.19 + h * 7.13 + d * 3.71;
    let sample = |offset: f32| ((seed + offset).sin() * 43_758.547).fract().abs() * 2.0 - 1.0;
    let rotation = Quat::from_euler(
        EulerRot::XYZ,
        sample(1.0) * 0.0015,
        sample(2.0) * 0.0015,
        sample(3.0) * 0.0015,
    );
    let half = Vec3::new(w * 0.5, h * 0.5, d * 0.5);
    let m = Mat3::from_quat(rotation);
    let projected = Vec3::new(
        m.x_axis.x.abs() * half.x + m.y_axis.x.abs() * half.y + m.z_axis.x.abs() * half.z,
        m.x_axis.y.abs() * half.x + m.y_axis.y.abs() * half.y + m.z_axis.y.abs() * half.z,
        m.x_axis.z.abs() * half.x + m.y_axis.z.abs() * half.y + m.z_axis.z.abs() * half.z,
    );
    let fit = (half.x / projected.x.max(1e-6))
        .min(half.y / projected.y.max(1e-6))
        .min(half.z / projected.z.max(1e-6)) * 0.995;
    let size = 0.99 + (sample(4.0) + 1.0) * 0.005;
    Some((rotation, size.min(fit).min(1.0)))
}

/// Build the mesh for a shape, with the picture laid by real size (`per_m` repeats per metre)
/// on boxes and cylinders when `per_m` is above 0 (the step 2c material textures).
pub fn build_mesh_uv(shape: &Shape, per_m: f32, bevel: f32) -> Mesh {
    if bevel > 0.0 {
        match *shape {
            Shape::Cuboid { w, h, d } if w.min(h).min(d) > 0.0 => {
                return rounded_box(w, h, d, bevel, per_m).into_mesh();
            }
            Shape::Cylinder { top, bottom, h, seg, caps: true }
                if top == bottom && top > 0.0 && h > 0.0 && seg >= 3 => {
                return rolled_cylinder(top, h, seg, bevel, per_m).into_mesh();
            }
            _ => {}
        }
    }
    if per_m <= 0.0 {
        return build_mesh(shape);
    }
    match *shape {
        Shape::Cuboid { w, h, d } => {
            let mut b = cuboid(w, h, d);
            // every face is a 0..1 square: stretch each to its real size. Faces come in the
            // order +x, -x, +y, -y, +z, -z, four corners each.
            let dims = [(d, h), (d, h), (w, d), (w, d), (w, h), (w, h)];
            for (f, (fw, fh)) in dims.iter().enumerate() {
                for k in 0..4 {
                    let uv = &mut b.uv[f * 4 + k];
                    uv[0] *= fw * per_m;
                    uv[1] *= fh * per_m;
                }
            }
            b.into_mesh()
        }
        Shape::Cylinder { top, bottom, h, seg, caps } => {
            let mut b = cylinder(top, bottom, h, seg, caps);
            let round = std::f32::consts::TAU * top.max(bottom);
            for uv in &mut b.uv {
                uv[0] *= round * per_m;
                uv[1] *= h * per_m;
            }
            b.into_mesh()
        }
        _ => build_mesh(shape),
    }
}

/// Build the mesh for a shape.
pub fn build_mesh(shape: &Shape) -> Mesh {
    match *shape {
        Shape::Sphere { r, ws, hs } => sphere(r, ws, hs).into_mesh(),
        Shape::Cylinder {
            top,
            bottom,
            h,
            seg,
            caps,
        } => cylinder(top, bottom, h, seg, caps).into_mesh(),
        Shape::Cuboid { w, h, d } => cuboid(w, h, d).into_mesh(),
        Shape::PalingPanel { w, h, d, boards, damage } => paling_panel(w, h, d, boards, damage).into_mesh(),
        Shape::Cone { r, h, seg } => cylinder(0.0, r, h, seg, true).into_mesh(),
        Shape::Torus {
            r,
            tube,
            radial,
            tubular,
            arc,
        } => torus(r, tube, radial, tubular, arc).into_mesh(),
        Shape::Disc { r, seg } => disc(r, seg).into_mesh(),
        Shape::Poly(pts) => poly(pts).into_mesh(),
        Shape::Quad { w, h } => quad(w, h).into_mesh(),
        Shape::Ring { inner, outer, seg } => ring(inner, outer, seg).into_mesh(),
        Shape::Ico { r, detail } => ico(r, detail).into_mesh(),
        Shape::Capsule {
            r,
            len,
            cap,
            radial,
        } => capsule(r, len, cap, radial).into_mesh(),
        Shape::HalfCylinder { r, h, seg } => cylinder_arc(r, r, h, seg, true, 0.0, PI).into_mesh(),
        Shape::Ground {
            x0,
            x1,
            z0,
            z1,
            hole,
            per_m,
        } => ground(x0, x1, z0, z1, hole, per_m).into_mesh(),
        Shape::PatchyLawn {
            x0,
            x1,
            z0,
            z1,
            hole,
            per_m,
            cell,
        } => patchy_lawn(x0, x1, z0, z1, hole, per_m, cell).into_mesh(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(b: &Buf) -> (usize, usize) {
        (b.pos.len(), b.idx.len() / 3)
    }

    #[test]
    fn beveled_props_keep_the_original_bounds_and_outward_faces() {
        for (mesh, half) in [
            (rounded_box(5.0, 0.6, 0.9, 0.04, 2.0), Vec3::new(2.5, 0.3, 0.45)),
            (rolled_cylinder(1.3, 2.6, 20, 0.07, 1.4), Vec3::splat(1.3)),
            (rounded_box(1.28, 1.96, 0.065, 0.5, 0.0), Vec3::new(0.64, 0.98, 0.0325)),
        ] {
            let mut furthest = Vec3::ZERO;
            for &point in &mesh.pos {
                furthest = furthest.max(Vec3::from_array(point).abs());
            }
            assert!((furthest - half).abs().max_element() < 1e-5);
            for triangle in mesh.idx.chunks_exact(3) {
                let [a, b, c] = [triangle[0], triangle[1], triangle[2]].map(|i| Vec3::from_array(mesh.pos[i as usize]));
                let normal = Vec3::from_array(mesh.nor[triangle[0] as usize]);
                assert!((b - a).cross(c - a).dot(normal) > 0.0);
            }
            assert!(mesh.uv.iter().flatten().all(|v| v.is_finite()));
        }
    }

    #[test]
    fn weathered_vertex_colour_is_seeded_subtle_and_fades_upward() {
        let mut first = rounded_box(1.0, 1.0, 1.0, 0.08, 0.0).into_mesh();
        let mut second = rounded_box(1.0, 1.0, 1.0, 0.08, 0.0).into_mesh();
        weather_by_height(&mut first);
        weather_by_height(&mut second);
        let bevy::mesh::VertexAttributeValues::Float32x4(a) = first.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() else { panic!() };
        let bevy::mesh::VertexAttributeValues::Float32x4(b) = second.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() else { panic!() };
        assert_eq!(a, b, "the fixed position seed must reproduce identical vertex colours");
        assert!(a.iter().flatten().all(|v| v.is_finite() && *v >= 0.70 && *v <= 1.03));
        let bevy::mesh::VertexAttributeValues::Float32x3(pos) = first.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else { panic!() };
        let low = pos.iter().zip(a).filter(|(p, _)| p[1] < -0.45).map(|(_, c)| c[0]).sum::<f32>() / 4.0;
        let high = pos.iter().zip(a).filter(|(p, _)| p[1] > 0.45).map(|(_, c)| c[0]).sum::<f32>() / 4.0;
        assert!(high > low, "top should remain warmer/lighter than the dirt-darkened base");
    }

    #[test]
    fn seeded_weathered_tilts_are_repeatable_and_stay_inside_authored_bounds() {
        let shapes = [
            Shape::Cuboid { w: 5.0, h: 0.6, d: 0.9 },
            Shape::Cuboid { w: 1.28, h: 1.96, d: 0.065 },
            Shape::Cuboid { w: 0.8, h: 0.8, d: 0.8 },
        ];
        let positions = [
            bbq_core::vec::V3::new(-8.0, 0.3, 20.0),
            bbq_core::vec::V3::new(22.5, 0.99, -14.94),
            bbq_core::vec::V3::new(1.0, 0.4, 2.0),
        ];
        for (shape, at) in shapes.into_iter().zip(positions) {
            let a = seeded_weathered_adjustment(&shape, at).unwrap();
            let b = seeded_weathered_adjustment(&shape, at).unwrap();
            assert_eq!(a, b, "the same authored prop must rebuild identically");
            let Shape::Cuboid { w, h, d } = shape else { unreachable!() };
            let half = Vec3::new(w * 0.5, h * 0.5, d * 0.5);
            for x in [-half.x, half.x] { for y in [-half.y, half.y] { for z in [-half.z, half.z] {
                let p = a.0 * (Vec3::new(x, y, z) * a.1);
                assert!(p.abs().cmple(half + Vec3::splat(1e-5)).all(), "prop escaped authored bounds: {p:?} vs {half:?}");
            } } }
        }
        assert!(seeded_weathered_adjustment(&Shape::Sphere { r: 1.0, ws: 8, hs: 6 }, positions[0]).is_none());
    }

    #[test]
    fn sphere_has_the_three_js_vertex_and_triangle_counts() {
        // (ws + 1) * (hs + 1) vertices; the pole rows lose one triangle per segment
        let b = sphere(1.0, 8, 6);
        assert_eq!(count(&b), (9 * 7, 8 * 6 * 2 - 8 * 2));
        // every vertex is on the surface
        for p in &b.pos {
            assert!(((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn cylinder_and_cone_counts() {
        let c = cylinder(1.0, 1.0, 2.0, 10, true);
        // side 2 rows of 11, two caps of 10 centres and 11 ring vertices
        assert_eq!(c.pos.len(), 22 + 2 * 21);
        assert_eq!(c.idx.len() / 3, 20 + 2 * 10);
        // a cone has no top cap and no top triangles
        let k = cylinder(0.0, 1.0, 2.0, 10, true);
        assert_eq!(k.idx.len() / 3, 10 + 10);
        // open: just the side
        assert_eq!(cylinder(1.0, 1.0, 2.0, 10, false).idx.len() / 3, 20);
    }

    #[test]
    fn cylinder_stands_on_y_with_the_right_height() {
        let c = cylinder(0.5, 0.5, 2.0, 12, true);
        let (lo, hi) = c
            .pos
            .iter()
            .fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p[1]), a.1.max(p[1])));
        assert!((lo + 1.0).abs() < 1e-6 && (hi - 1.0).abs() < 1e-6);
    }

    #[test]
    fn torus_lies_in_the_xy_plane_and_arcs_round_from_plus_x() {
        let t = torus(1.0, 0.1, 6, 12, PI);
        assert_eq!(t.pos.len(), 7 * 13);
        // half a ring: every y is at or above zero (give or take the tube)
        assert!(t.pos.iter().all(|p| p[1] > -1e-4));
        // starts at +x
        assert!((t.pos[0][0] - 1.1).abs() < 1e-5);
        // flat in z apart from the tube
        assert!(t.pos.iter().all(|p| p[2].abs() <= 0.1 + 1e-5));
    }

    #[test]
    fn disc_faces_plus_z() {
        let d = disc(1.0, 8);
        assert_eq!(count(&d), (10, 8));
        assert!(d.nor.iter().all(|n| *n == [0.0, 0.0, 1.0]));
        // winding is counter-clockwise seen from +z
        for t in d.idx.chunks(3) {
            let (a, b, c) = (
                d.pos[t[0] as usize],
                d.pos[t[1] as usize],
                d.pos[t[2] as usize],
            );
            let cross = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            assert!(cross > 0.0);
        }
    }

    #[test]
    fn ear_clipping_fills_a_dented_outline_exactly() {
        // the fish's forked tail has a notch; its triangles must cover its exact area
        let tail: &[(f32, f32)] = &[
            (0.0, 0.0),
            (-0.1, 0.1),
            (-0.13, 0.1),
            (-0.075, 0.0),
            (-0.13, -0.1),
            (-0.1, -0.1),
        ];
        let tris = triangulate(tail);
        assert_eq!(tris.len(), tail.len() - 2);
        let total: f32 = tris
            .iter()
            .map(|t| {
                let (a, b, c) = (tail[t[0]], tail[t[1]], tail[t[2]]);
                ((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)) / 2.0
            })
            .sum();
        assert!((total - area2(tail).abs() / 2.0).abs() < 1e-6);
        // every triangle is counter-clockwise
        for t in &tris {
            let (a, b, c) = (tail[t[0]], tail[t[1]], tail[t[2]]);
            assert!((b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0) > 0.0);
        }
    }

    #[test]
    fn clockwise_outlines_are_turned_round() {
        let cw: &[(f32, f32)] = &[(0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0)];
        let tris = triangulate(cw);
        assert_eq!(tris.len(), 2);
    }

    #[test]
    fn box_has_six_faces_all_facing_outwards() {
        let b = cuboid(2.0, 1.0, 3.0);
        assert_eq!(count(&b), (24, 12));
        for t in b.idx.chunks(3) {
            let (a, bb, c) = (
                b.pos[t[0] as usize],
                b.pos[t[1] as usize],
                b.pos[t[2] as usize],
            );
            let e1 = [bb[0] - a[0], bb[1] - a[1], bb[2] - a[2]];
            let e2 = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [
                e1[1] * e2[2] - e1[2] * e2[1],
                e1[2] * e2[0] - e1[0] * e2[2],
                e1[0] * e2[1] - e1[1] * e2[0],
            ];
            // the triangle's own direction agrees with its stored normal and points away from the middle
            let stored = b.nor[t[0] as usize];
            assert!(
                n[0] * stored[0] + n[1] * stored[1] + n[2] * stored[2] > 0.0,
                "wound the wrong way"
            );
            assert!(a[0] * stored[0] + a[1] * stored[1] + a[2] * stored[2] > 0.0);
        }
        // the front (+z) picture reads left to right and top to bottom: top-left is (0, 0)
        let front: Vec<usize> = (0..24).filter(|i| b.nor[*i] == [0.0, 0.0, 1.0]).collect();
        assert_eq!(front.len(), 4);
        for i in front {
            let (p, uv) = (b.pos[i], b.uv[i]);
            assert_eq!(uv[0] == 0.0, p[0] < 0.0, "left edge is u = 0");
            assert_eq!(
                uv[1] == 0.0,
                p[1] > 0.0,
                "top edge is v = 0 (Bevy pictures run downwards)"
            );
        }
    }

    #[test]
    fn ring_is_an_annulus_facing_plus_z() {
        let r = ring(3.0, 4.0, 16);
        assert_eq!(count(&r), (34, 32));
        for p in &r.pos {
            let d = p[0].hypot(p[1]);
            assert!((d - 3.0).abs() < 1e-5 || (d - 4.0).abs() < 1e-5);
        }
        for t in r.idx.chunks(3) {
            let (a, b, c) = (
                r.pos[t[0] as usize],
                r.pos[t[1] as usize],
                r.pos[t[2] as usize],
            );
            assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
        }
    }

    #[test]
    fn quad_faces_plus_z_and_is_counter_clockwise() {
        let q = quad(2.0, 1.0);
        assert_eq!(count(&q), (4, 2));
        for t in q.idx.chunks(3) {
            let (a, b, c) = (
                q.pos[t[0] as usize],
                q.pos[t[1] as usize],
                q.pos[t[2] as usize],
            );
            assert!((b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]) > 0.0);
        }
    }

    #[test]
    fn icosahedron_counts_and_flat_normals() {
        // detail 0: 20 faces; detail 1: 80 faces; each triangle has its own three vertices
        assert_eq!(count(&ico(1.0, 0)), (60, 20));
        assert_eq!(count(&ico(1.0, 1)), (240, 80));
        let b = ico(2.0, 1);
        for p in &b.pos {
            assert!(((p[0] * p[0] + p[1] * p[1] + p[2] * p[2]).sqrt() - 2.0).abs() < 1e-4);
        }
        // faces point away from the middle
        for t in b.idx.chunks(3) {
            let n = b.nor[t[0] as usize];
            let c = b.pos[t[0] as usize];
            assert!(n[0] * c[0] + n[1] * c[1] + n[2] * c[2] > 0.0);
        }
    }

    #[test]
    fn capsule_is_as_long_as_asked_and_stays_inside_its_radius() {
        let c = capsule(0.04, 0.24, 4, 8);
        let (lo, hi) = c
            .pos
            .iter()
            .fold((f32::MAX, f32::MIN), |a, p| (a.0.min(p[1]), a.1.max(p[1])));
        assert!(
            (lo + 0.16).abs() < 1e-5 && (hi - 0.16).abs() < 1e-5,
            "{lo} {hi}"
        );
        assert!(c.pos.iter().all(|p| p[0].hypot(p[2]) <= 0.04 + 1e-5));
    }

    #[test]
    fn half_cylinder_only_covers_one_side() {
        let h = cylinder_arc(1.0, 1.0, 2.0, 8, true, 0.0, PI);
        assert!(
            h.pos.iter().all(|p| p[0] >= -1e-5),
            "x should stay on the +x side"
        );
        assert!(h.pos.iter().any(|p| p[2] > 0.9) && h.pos.iter().any(|p| p[2] < -0.9));
    }

    #[test]
    fn ground_leaves_a_hole() {
        let g = ground(-10.0, 10.0, -10.0, 10.0, (-2.0, 2.0, -1.0, 1.0), 1.0 / 16.0);
        assert_eq!(count(&g), (16, 8));
        // no triangle's middle falls inside the hole
        for t in g.idx.chunks(3) {
            let (a, b, c) = (
                g.pos[t[0] as usize],
                g.pos[t[1] as usize],
                g.pos[t[2] as usize],
            );
            let (cx, cz) = ((a[0] + b[0] + c[0]) / 3.0, (a[2] + b[2] + c[2]) / 3.0);
            assert!(!(cx > -2.0 && cx < 2.0 && cz > -1.0 && cz < 1.0));
        }
        // all faces up
        for t in g.idx.chunks(3) {
            let (a, b, c) = (
                g.pos[t[0] as usize],
                g.pos[t[1] as usize],
                g.pos[t[2] as usize],
            );
            let cross_y = (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]);
            assert!(cross_y > 0.0);
        }
    }

    #[test]
    fn the_patchy_lawn_covers_the_yard_except_the_pool_and_every_corner_has_a_colour() {
        let hole = (-6.0, -2.0, -3.0, 1.0);
        let g = patchy_lawn(-10.0, 10.0, -10.0, 10.0, hole, 1.0 / 16.0, 1.0);
        assert_eq!(g.pos.len(), g.col.len());
        let area: f32 = g
            .idx
            .chunks(3)
            .map(|t| {
                let (a, b, c) = (g.pos[t[0] as usize], g.pos[t[1] as usize], g.pos[t[2] as usize]);
                ((b[0] - a[0]) * (c[2] - a[2]) - (b[2] - a[2]) * (c[0] - a[0])).abs() / 2.0
            })
            .sum();
        assert!((area - (400.0 - 16.0)).abs() < 0.01, "area {area}");
        for t in g.idx.chunks(3) {
            let (a, b, c) = (g.pos[t[0] as usize], g.pos[t[1] as usize], g.pos[t[2] as usize]);
            let (cx, cz) = ((a[0] + b[0] + c[0]) / 3.0, (a[2] + b[2] + c[2]) / 3.0);
            assert!(!(cx > -6.0 && cx < -2.0 && cz > -3.0 && cz < 1.0));
            let cross_y = (b[2] - a[2]) * (c[0] - a[0]) - (b[0] - a[0]) * (c[2] - a[2]);
            assert!(cross_y > 0.0, "faces up");
        }
    }

    #[test]
    fn every_shape_in_every_item_builds() {
        use bbq_core::items::{DildoVariant, ItemKind};
        for k in ItemKind::ALL {
            for p in bbq_core::looks::item(k, 1, Some(DildoVariant::Gold)) {
                let m = build_mesh(&p.shape);
                assert!(m.count_vertices() > 0, "{k:?}");
            }
        }
        for p in bbq_core::looks::bum_crack() {
            assert!(build_mesh(&p.shape).count_vertices() > 0);
        }
    }
}

#[cfg(test)] mod servo_ground_tests {
    use super::*;
    #[test] fn ground_paint_is_local_finite_and_flat() {
        for (x,z) in [(-30.,0.),(30.,0.),(0.,10.),(0.,-20.),(-2.,3.),(12.,3.),(4.,-5.),(4.,7.)] {
            assert_eq!(servo_ground_colour(x,z),[1.;4]);
        }
        assert_ne!(servo_ground_colour(8.5,-2.5),[1.;4]);
        let mesh=servo_ground_mesh();
        let bevy::mesh::VertexAttributeValues::Float32x3(pos)=mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap() else {panic!()};
        let bevy::mesh::VertexAttributeValues::Float32x2(uv)=mesh.attribute(Mesh::ATTRIBUTE_UV_0).unwrap() else {panic!()};
        let bevy::mesh::VertexAttributeValues::Float32x4(col)=mesh.attribute(Mesh::ATTRIBUTE_COLOR).unwrap() else {panic!()};
        for ((p,u),c) in pos.iter().zip(uv).zip(col) {
            assert_eq!(p[2],0.);assert!((u[0]-(p[0]/260.+0.5)).abs()<0.00001);
            assert!((u[1]-(0.5-p[1]/260.)).abs()<0.00001);
            assert!(c.iter().all(|v|v.is_finite() && *v>=0.7 && *v<=1.15));
        }
    }
}
