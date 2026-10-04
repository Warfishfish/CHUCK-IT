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
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, self.pos)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, self.nor)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, self.uv)
        .with_inserted_indices(Indices::U32(self.idx))
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(b: &Buf) -> (usize, usize) {
        (b.pos.len(), b.idx.len() / 3)
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
