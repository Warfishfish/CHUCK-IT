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
    let mut b = Buf::default();
    let half = h / 2.0;
    let slope = (bottom - top) / h;
    // the side: two rings of vertices
    let mut grid: Vec<[u32; 2]> = Vec::new();
    for x in 0..=seg {
        let u = x as f32 / seg as f32;
        let th = u * TAU;
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
                    let th = u * TAU;
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
            let blocked = idx.iter().any(|&k| {
                k != ia && k != ib && k != ic && in_tri(pts[k], a, b, c)
            });
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
        Shape::Cuboid { w, h, d } => Mesh::from(Cuboid::new(w, h, d)),
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
        let (lo, hi) = c.pos.iter().fold((f32::MAX, f32::MIN), |a, p| {
            (a.0.min(p[1]), a.1.max(p[1]))
        });
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
            let (a, b, c) = (d.pos[t[0] as usize], d.pos[t[1] as usize], d.pos[t[2] as usize]);
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
