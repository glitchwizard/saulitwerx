//! Profile extraction: closed loops of sketch geometry → regions with holes.
//!
//! Lines and arcs become the edges of a planar graph. Its vertices are
//! endpoints welded within a tolerance, and each vertex takes the mean
//! position of the endpoints welded into it. Circles, and arcs that close on
//! themselves, are loops on their own. The steps are:
//!
//! 1. Any two edges that cross, touch away from a shared vertex, or overlap
//!    are rejected with [`ProfileError::SelfIntersecting`]. Splitting edges at
//!    intersections is out of scope.
//! 2. Edges with a free end are pruned repeatedly and reported as open.
//! 3. The faces of the graph are traced with the standard half-edge walk of a
//!    doubly-connected edge list (de Berg, Cheong, van Kreveld & Overmars,
//!    *Computational Geometry*, 3rd ed., §2.2): sort the half-edges leaving
//!    each vertex counter-clockwise by departure angle (tangent angle for
//!    arcs, ties broken by curvature), and continue each half-edge `u → v`
//!    with the half-edge clockwise-next to its twin at `v`. Bounded faces then
//!    run counter-clockwise; each connected component's unbounded face runs
//!    clockwise and is dropped. An edge with the same face on both sides is a
//!    bridge: it is reported as open, removed, and the walk repeated.
//! 4. Each loop is nested in the smallest loop of another component that
//!    contains it (winding number of a point of the loop). Loops at even
//!    depth are outer boundaries (counter-clockwise); loops at odd depth are
//!    holes of their parent (reversed to clockwise).
//!
//! Signed areas are exact for arcs: by Green's theorem the area enclosed by a
//! closed curve is `½ ∮ (x dy − y dx)` (Stewart, *Calculus*, 8th ed., §16.4).
//! A line `P → Q` contributes `½ (P × Q)`, the shoelace term. An arc of
//! radius `r` and center `c` from angle `a0` to `a1` contributes
//! `½ [r²(a1 − a0) + r cx (sin a1 − sin a0) − r cy (cos a1 − cos a0)]`.

use crate::{GeomId, Geometry, Point, Sketch, SketchError};
use std::f64::consts::{FRAC_PI_2, PI, TAU};
use std::fmt;

/// Settings for [`profiles`] and [`profile_report`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ProfileOptions {
    /// Endpoints closer than this are welded into one vertex. `None` uses
    /// 1e-6 × the diagonal of the sketch's bounding box.
    pub weld_tolerance: Option<f64>,
}

/// One sketch entity traversed forwards (line start → end, arc and circle
/// counter-clockwise) or reversed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OrientedEdge {
    pub geom: GeomId,
    pub reversed: bool,
}

/// A closed chain of oriented edges.
#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    pub edges: Vec<OrientedEdge>,
    /// Enclosed area, positive when the loop runs counter-clockwise.
    pub signed_area: f64,
}

impl Loop {
    fn reversed(&self) -> Loop {
        Loop {
            edges: self
                .edges
                .iter()
                .rev()
                .map(|e| OrientedEdge {
                    geom: e.geom,
                    reversed: !e.reversed,
                })
                .collect(),
            signed_area: -self.signed_area,
        }
    }
}

/// A face to extrude: a counter-clockwise outer loop minus clockwise holes.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub outer: Loop,
    pub holes: Vec<Loop>,
}

impl Region {
    /// Net area: the outer area minus the hole areas.
    pub fn area(&self) -> f64 {
        self.outer.signed_area + self.holes.iter().map(|h| h.signed_area).sum::<f64>()
    }
}

/// Regions plus the edges that belong to no closed loop.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProfileReport {
    pub regions: Vec<Region>,
    /// Dangling edges, bridges between loops, and degenerate (zero-length)
    /// edges, sorted by id.
    pub open_edges: Vec<GeomId>,
}

/// Errors from profile extraction.
#[derive(Debug, Clone, PartialEq)]
pub enum ProfileError {
    /// The weld tolerance was zero, negative or not finite.
    InvalidTolerance(f64),
    /// These two entities cross, overlap, or touch away from a shared vertex.
    SelfIntersecting(GeomId, GeomId),
    Sketch(SketchError),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProfileError::InvalidTolerance(t) => write!(f, "weld tolerance must be > 0, got {t}"),
            ProfileError::SelfIntersecting(a, b) => {
                write!(f, "geometry {} and {} intersect", a.0, b.0)
            }
            ProfileError::Sketch(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ProfileError {}

impl From<SketchError> for ProfileError {
    fn from(e: SketchError) -> Self {
        ProfileError::Sketch(e)
    }
}

/// The closed regions of `sketch`, ready to become kernel faces.
pub fn profiles(sketch: &Sketch, opts: &ProfileOptions) -> Result<Vec<Region>, ProfileError> {
    profile_report(sketch, opts).map(|r| r.regions)
}

#[derive(Debug, Clone, Copy)]
enum Curve {
    Line(Point, Point),
    /// Counter-clockwise from `a0` to `a1`; a circle is `0..2π`.
    Arc {
        c: Point,
        r: f64,
        a0: f64,
        a1: f64,
    },
}

struct Edge {
    id: GeomId,
    curve: Curve,
    /// Start and end vertex; `None` for a curve that closes on itself.
    ends: Option<(usize, usize)>,
}

fn sub(a: Point, b: Point) -> Point {
    Point {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

fn cross(a: Point, b: Point) -> f64 {
    a.x * b.y - a.y * b.x
}

fn dot(a: Point, b: Point) -> f64 {
    a.x * b.x + a.y * b.y
}

fn along(p: Point, d: Point, t: f64) -> Point {
    Point {
        x: p.x + t * d.x,
        y: p.y + t * d.y,
    }
}

fn polar(c: Point, r: f64, a: f64) -> Point {
    Point {
        x: c.x + r * a.cos(),
        y: c.y + r * a.sin(),
    }
}

/// Every closed region of `sketch`, plus the edges left open.
pub fn profile_report(
    sketch: &Sketch,
    opts: &ProfileOptions,
) -> Result<ProfileReport, ProfileError> {
    let mut curves = Vec::new();
    for i in 0..sketch.len() {
        let id = GeomId(i as u32);
        let curve = match sketch.get(id)? {
            Geometry::Point(_) => continue,
            Geometry::Line(l) => Curve::Line(l.start, l.end),
            Geometry::Circle(c) => Curve::Arc {
                c: c.center,
                r: c.radius,
                a0: 0.0,
                a1: TAU,
            },
            Geometry::Arc(a) => Curve::Arc {
                c: a.center,
                r: a.radius,
                a0: a.start_angle,
                a1: a.end_angle,
            },
        };
        curves.push((id, curve));
    }
    let tol = match opts.weld_tolerance {
        Some(t) if t > 0.0 && t.is_finite() => t,
        Some(t) => return Err(ProfileError::InvalidTolerance(t)),
        None => 1e-6 * bbox_diagonal(&curves),
    };

    // Weld endpoints into vertices: (sum of positions, count, first position).
    let mut clusters: Vec<(Point, f64, Point)> = Vec::new();
    let mut weld = |p: Point| -> usize {
        if let Some(i) = clusters.iter().position(|c| c.2.distance(&p) <= tol) {
            let c = &mut clusters[i];
            c.0 = Point {
                x: c.0.x + p.x,
                y: c.0.y + p.y,
            };
            c.1 += 1.0;
            i
        } else {
            clusters.push((p, 1.0, p));
            clusters.len() - 1
        }
    };
    let mut edges = Vec::new();
    let mut open = Vec::new();
    for &(id, curve) in &curves {
        let (s, e, closes) = match curve {
            Curve::Line(a, b) => (a, b, false),
            Curve::Arc { c, r, a0, a1 } => (polar(c, r, a0), polar(c, r, a1), a1 - a0 > PI),
        };
        let ends = if closes && s.distance(&e) <= tol {
            None
        } else {
            let (u, v) = (weld(s), weld(e));
            if u == v {
                open.push(id); // zero-length edge
                continue;
            }
            Some((u, v))
        };
        edges.push(Edge { id, curve, ends });
    }
    let nodes: Vec<Point> = clusters
        .iter()
        .map(|&(sum, n, _)| Point {
            x: sum.x / n,
            y: sum.y / n,
        })
        .collect();

    for i in 0..edges.len() {
        for j in i + 1..edges.len() {
            check_pair(&edges[i], &edges[j], &nodes, tol)?;
        }
    }

    // Prune free ends and bridges until every remaining edge bounds two faces.
    let mut active: Vec<bool> = edges.iter().map(|e| e.ends.is_some()).collect();
    let faces = loop {
        prune(&edges, &mut active, nodes.len(), &mut open);
        let faces = trace_faces(&edges, &active, &nodes);
        let mut bridges = Vec::new();
        for f in &faces {
            for &h in f {
                if h % 2 == 0 && f.contains(&(h + 1)) {
                    bridges.push(h / 2);
                }
            }
        }
        if bridges.is_empty() {
            break faces;
        }
        for k in bridges {
            active[k] = false;
            open.push(edges[k].id);
        }
    };

    // Connected components, so that only loops of different components nest.
    let mut comp: Vec<usize> = (0..nodes.len()).collect();
    fn find(comp: &mut [usize], mut i: usize) -> usize {
        while comp[i] != i {
            comp[i] = comp[comp[i]];
            i = comp[i];
        }
        i
    }
    for (k, e) in edges.iter().enumerate() {
        if let (true, Some((u, v))) = (active[k], e.ends) {
            let (a, b) = (find(&mut comp, u), find(&mut comp, v));
            comp[a] = b;
        }
    }

    // Loops as (half-edges, signed area, component). Half-edge 2k runs edge k
    // forwards, 2k + 1 backwards.
    let mut loops: Vec<(Vec<usize>, f64, usize)> = Vec::new();
    for f in faces {
        let area = loop_area(&edges, &nodes, &f);
        if area > 0.0 {
            let (u, _) = edges[f[0] / 2].ends.expect("active edge");
            let c = find(&mut comp, u);
            loops.push((f, area, c));
        }
    }
    for (k, e) in edges.iter().enumerate() {
        if e.ends.is_none() {
            let f = vec![2 * k];
            let area = loop_area(&edges, &nodes, &f);
            loops.push((f, area, nodes.len() + k));
        }
    }

    // Nest: depth = number of loops (of other components) containing a loop.
    let n = loops.len();
    let mut parent: Vec<Option<usize>> = vec![None; n];
    let mut depth = vec![0usize; n];
    for i in 0..n {
        let p = start_point(&edges, &nodes, loops[i].0[0]);
        for j in 0..n {
            if loops[j].2 == loops[i].2 || winding(&edges, &nodes, &loops[j].0, p) == 0 {
                continue;
            }
            depth[i] += 1;
            match parent[i] {
                Some(q) if loops[q].1 <= loops[j].1 => {}
                _ => parent[i] = Some(j),
            }
        }
    }
    let to_loop = |i: usize| Loop {
        edges: loops[i]
            .0
            .iter()
            .map(|&h| OrientedEdge {
                geom: edges[h / 2].id,
                reversed: h % 2 == 1,
            })
            .collect(),
        signed_area: loops[i].1,
    };
    let regions = (0..n)
        .filter(|&i| depth[i] % 2 == 0)
        .map(|i| Region {
            outer: to_loop(i),
            holes: (0..n)
                .filter(|&h| depth[h] % 2 == 1 && parent[h] == Some(i))
                .map(|h| to_loop(h).reversed())
                .collect(),
        })
        .collect();
    open.sort();
    open.dedup();
    Ok(ProfileReport {
        regions,
        open_edges: open,
    })
}

fn bbox_diagonal(curves: &[(GeomId, Curve)]) -> f64 {
    let (mut lo, mut hi) = ([f64::INFINITY; 2], [f64::NEG_INFINITY; 2]);
    let mut add = |p: Point, pad: f64| {
        lo = [lo[0].min(p.x - pad), lo[1].min(p.y - pad)];
        hi = [hi[0].max(p.x + pad), hi[1].max(p.y + pad)];
    };
    for &(_, c) in curves {
        match c {
            Curve::Line(a, b) => {
                add(a, 0.0);
                add(b, 0.0);
            }
            Curve::Arc { c, r, .. } => add(c, r),
        }
    }
    if curves.is_empty() {
        0.0
    } else {
        (hi[0] - lo[0]).hypot(hi[1] - lo[1])
    }
}

/// Fails if `a` and `b` meet anywhere except at a vertex they share.
fn check_pair(a: &Edge, b: &Edge, nodes: &[Point], tol: f64) -> Result<(), ProfileError> {
    let err = Err(ProfileError::SelfIntersecting(a.id, b.id));
    let Some(points) = intersections(&a.curve, &b.curve, tol) else {
        return err;
    };
    let shared: Vec<Point> = match (a.ends, b.ends) {
        (Some((a0, a1)), Some((b0, b1))) => [a0, a1]
            .into_iter()
            .filter(|n| *n == b0 || *n == b1)
            .map(|n| nodes[n])
            .collect(),
        _ => Vec::new(),
    };
    let at_shared = |p: &Point| shared.iter().any(|s| s.distance(p) <= 10.0 * tol);
    if points.iter().all(at_shared) {
        Ok(())
    } else {
        err
    }
}

/// Points where two curves meet, or `None` if they overlap along a stretch.
fn intersections(a: &Curve, b: &Curve, tol: f64) -> Option<Vec<Point>> {
    match (*a, *b) {
        (Curve::Line(p0, p1), Curve::Line(q0, q1)) => line_line(p0, p1, q0, q1, tol),
        (Curve::Line(p0, p1), Curve::Arc { c, r, a0, a1 })
        | (Curve::Arc { c, r, a0, a1 }, Curve::Line(p0, p1)) => Some(
            line_circle(p0, p1, c, r, tol)
                .into_iter()
                .filter(|p| on_arc(*p, c, r, a0, a1, tol))
                .collect(),
        ),
        (
            Curve::Arc {
                c: c1,
                r: r1,
                a0: s1,
                a1: e1,
            },
            Curve::Arc {
                c: c2,
                r: r2,
                a0: s2,
                a1: e2,
            },
        ) => match circle_circle(c1, r1, c2, r2, tol) {
            Some(pts) => Some(
                pts.into_iter()
                    .filter(|p| on_arc(*p, c1, r1, s1, e1, tol) && on_arc(*p, c2, r2, s2, e2, tol))
                    .collect(),
            ),
            // Same circle: they overlap unless they only touch at endpoints.
            None => {
                let eps = tol / r1;
                let inside = |t: f64, s: f64, e: f64| {
                    let off = (t - s).rem_euclid(TAU);
                    off > eps && off < e - s - eps
                };
                let same_start = (s1 - s2).rem_euclid(TAU).min((s2 - s1).rem_euclid(TAU)) <= eps;
                if same_start || inside(s2, s1, e1) || inside(s1, s2, e2) {
                    None
                } else {
                    Some(Vec::new())
                }
            }
        },
    }
}

fn line_line(p0: Point, p1: Point, q0: Point, q1: Point, tol: f64) -> Option<Vec<Point>> {
    let (r, s, qp) = (sub(p1, p0), sub(q1, q0), sub(q0, p0));
    let (rl, sl) = (p0.distance(&p1), q0.distance(&q1));
    let den = cross(r, s);
    if den.abs() <= 1e-12 * rl * sl {
        if (cross(r, qp) / rl).abs() > tol {
            return Some(Vec::new()); // parallel, apart
        }
        // Collinear: overlap of the projections onto `r`, in length units.
        let (t0, t1) = (dot(qp, r) / rl, dot(sub(q1, p0), r) / rl);
        let (lo, hi) = (t0.min(t1).max(0.0), t0.max(t1).min(rl));
        return if hi - lo > tol {
            None
        } else if hi - lo >= -tol {
            Some(vec![along(p0, r, (lo + hi) / (2.0 * rl))])
        } else {
            Some(Vec::new())
        };
    }
    let (t, u) = (cross(qp, s) / den, cross(qp, r) / den);
    let within = |t: f64, l: f64| t >= -tol / l && t <= 1.0 + tol / l;
    Some(if within(t, rl) && within(u, sl) {
        vec![along(p0, r, t)]
    } else {
        Vec::new()
    })
}

fn line_circle(p0: Point, p1: Point, c: Point, r: f64, tol: f64) -> Vec<Point> {
    let l = p0.distance(&p1);
    let u = Point {
        x: (p1.x - p0.x) / l,
        y: (p1.y - p0.y) / l,
    };
    let t0 = dot(sub(c, p0), u);
    let h = along(p0, u, t0).distance(&c);
    if h > r + tol {
        return Vec::new();
    }
    let half = if h >= r - tol {
        0.0 // tangent
    } else {
        (r * r - h * h).sqrt()
    };
    [t0 - half, t0 + half]
        .into_iter()
        .filter(|t| *t >= -tol && *t <= l + tol)
        .map(|t| along(p0, u, t))
        .collect()
}

/// Intersection points of two full circles, or `None` if they coincide.
fn circle_circle(c1: Point, r1: f64, c2: Point, r2: f64, tol: f64) -> Option<Vec<Point>> {
    let d = c1.distance(&c2);
    if d <= tol {
        return if (r1 - r2).abs() <= tol {
            None
        } else {
            Some(Vec::new())
        };
    }
    if d > r1 + r2 + tol || d < (r1 - r2).abs() - tol {
        return Some(Vec::new());
    }
    let e = Point {
        x: (c2.x - c1.x) / d,
        y: (c2.y - c1.y) / d,
    };
    let a = (d * d + r1 * r1 - r2 * r2) / (2.0 * d);
    let m = along(c1, e, a);
    let h = (r1 * r1 - a * a).max(0.0).sqrt();
    let n = Point { x: -e.y, y: e.x };
    Some(if h <= tol {
        vec![m]
    } else {
        vec![along(m, n, h), along(m, n, -h)]
    })
}

fn on_arc(p: Point, c: Point, r: f64, a0: f64, a1: f64, tol: f64) -> bool {
    let off = ((p.y - c.y).atan2(p.x - c.x) - a0).rem_euclid(TAU);
    off <= a1 - a0 + tol / r || off >= TAU - tol / r
}

/// Deactivates edges with a free end until none is left.
fn prune(edges: &[Edge], active: &mut [bool], n_nodes: usize, open: &mut Vec<GeomId>) {
    let mut degree = vec![0usize; n_nodes];
    for (k, e) in edges.iter().enumerate() {
        if let (true, Some((u, v))) = (active[k], e.ends) {
            degree[u] += 1;
            degree[v] += 1;
        }
    }
    let mut changed = true;
    while changed {
        changed = false;
        for (k, e) in edges.iter().enumerate() {
            if let (true, Some((u, v))) = (active[k], e.ends) {
                if degree[u] == 1 || degree[v] == 1 {
                    active[k] = false;
                    open.push(e.id);
                    degree[u] -= 1;
                    degree[v] -= 1;
                    changed = true;
                }
            }
        }
    }
}

/// Tail and head vertex of half-edge `h`.
fn half_ends(edges: &[Edge], h: usize) -> (usize, usize) {
    let (u, v) = edges[h / 2].ends.expect("active edge");
    if h % 2 == 0 {
        (u, v)
    } else {
        (v, u)
    }
}

/// Departure angle in `[0, 2π)` and signed curvature of half-edge `h`.
fn departure(edges: &[Edge], nodes: &[Point], h: usize) -> (f64, f64) {
    let (angle, curvature) = match edges[h / 2].curve {
        Curve::Line(..) => {
            let (u, v) = half_ends(edges, h);
            let d = sub(nodes[v], nodes[u]);
            (d.y.atan2(d.x), 0.0)
        }
        Curve::Arc { r, a0, .. } if h % 2 == 0 => (a0 + FRAC_PI_2, 1.0 / r),
        Curve::Arc { r, a1, .. } => (a1 - FRAC_PI_2, -1.0 / r),
    };
    let angle = angle.rem_euclid(TAU);
    (if angle > TAU - 1e-12 { 0.0 } else { angle }, curvature)
}

/// Traces every face of the active edges as a cycle of half-edges.
fn trace_faces(edges: &[Edge], active: &[bool], nodes: &[Point]) -> Vec<Vec<usize>> {
    let halves: Vec<usize> = (0..2 * edges.len()).filter(|h| active[h / 2]).collect();
    let mut out: Vec<Vec<(f64, f64, usize)>> = vec![Vec::new(); nodes.len()];
    for &h in &halves {
        let (a, k) = departure(edges, nodes, h);
        out[half_ends(edges, h).0].push((a, k, h));
    }
    let mut pos = vec![0; 2 * edges.len()];
    for list in &mut out {
        list.sort_by(|x, y| {
            if (x.0 - y.0).abs() <= 1e-9 {
                x.1.total_cmp(&y.1)
            } else {
                x.0.total_cmp(&y.0)
            }
        });
        for (i, &(_, _, h)) in list.iter().enumerate() {
            pos[h] = i;
        }
    }
    let mut seen = vec![false; 2 * edges.len()];
    let mut faces = Vec::new();
    for &start in &halves {
        let mut face = Vec::new();
        let mut h = start;
        while !seen[h] {
            seen[h] = true;
            face.push(h);
            let twin = h ^ 1;
            let list = &out[half_ends(edges, twin).0];
            h = list[(pos[twin] + list.len() - 1) % list.len()].2;
        }
        if !face.is_empty() {
            faces.push(face);
        }
    }
    faces
}

/// Signed area of a closed chain of half-edges, relative to its first point
/// to limit cancellation (Green's theorem; see the module docs).
fn loop_area(edges: &[Edge], nodes: &[Point], halves: &[usize]) -> f64 {
    let o = start_point(edges, nodes, halves[0]);
    let mut sum = 0.0;
    for &h in halves {
        sum += match edges[h / 2].curve {
            Curve::Line(..) => {
                let (u, v) = half_ends(edges, h);
                0.5 * cross(sub(nodes[u], o), sub(nodes[v], o))
            }
            Curve::Arc { c, r, a0, a1 } => {
                let c = sub(c, o);
                let f = 0.5
                    * (r * r * (a1 - a0) + r * c.x * (a1.sin() - a0.sin())
                        - r * c.y * (a1.cos() - a0.cos()));
                if h % 2 == 0 {
                    f
                } else {
                    -f
                }
            }
        };
    }
    sum
}

fn start_point(edges: &[Edge], nodes: &[Point], h: usize) -> Point {
    match edges[h / 2].curve {
        Curve::Line(..) => nodes[half_ends(edges, h).0],
        Curve::Arc { c, r, a0, a1 } => polar(c, r, if h % 2 == 0 { a0 } else { a1 }),
    }
}

/// Winding number of a closed chain of half-edges around `p`, which must not
/// lie on it. Each line adds the angle it subtends at `p`. Each arc is split
/// into pieces of at most π; a piece adds the angle its chord subtends, plus
/// 2π when `p` lies in the circular segment between chord and arc.
fn winding(edges: &[Edge], nodes: &[Point], halves: &[usize], p: Point) -> i64 {
    let subtends = |a: Point, b: Point| {
        let (a, b) = (sub(a, p), sub(b, p));
        cross(a, b).atan2(dot(a, b))
    };
    let mut total = 0.0;
    for &h in halves {
        total += match edges[h / 2].curve {
            Curve::Line(..) => {
                let (u, v) = half_ends(edges, h);
                subtends(nodes[u], nodes[v])
            }
            Curve::Arc { c, r, a0, a1 } => {
                let pieces = ((a1 - a0) / PI).ceil().max(1.0);
                let step = (a1 - a0) / pieces;
                let mut f = 0.0;
                for i in 0..pieces as usize {
                    let s = polar(c, r, a0 + step * i as f64);
                    let e = polar(c, r, a0 + step * (i + 1) as f64);
                    f += subtends(s, e);
                    if p.distance(&c) < r && cross(sub(e, s), sub(p, s)) < 0.0 {
                        f += TAU;
                    }
                }
                if h % 2 == 0 {
                    f
                } else {
                    -f
                }
            }
        };
    }
    (total / TAU).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arc, Circle, Line};

    const TOL: f64 = 1e-9;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }

    fn line(s: &mut Sketch, a: (f64, f64), b: (f64, f64)) -> GeomId {
        let l = Line::new(p(a.0, a.1), p(b.0, b.1)).unwrap();
        s.add(Geometry::Line(l)).unwrap()
    }

    fn circle(s: &mut Sketch, x: f64, y: f64, r: f64) -> GeomId {
        s.add(Geometry::Circle(Circle::new(p(x, y), r).unwrap()))
            .unwrap()
    }

    fn arc(s: &mut Sketch, x: f64, y: f64, r: f64, a0: f64, a1: f64) -> GeomId {
        s.add(Geometry::Arc(Arc::new(p(x, y), r, a0, a1).unwrap()))
            .unwrap()
    }

    /// Axis-aligned rectangle from 4 lines, counter-clockwise from (x, y).
    fn rect(s: &mut Sketch, x: f64, y: f64, w: f64, h: f64) -> Vec<GeomId> {
        let c = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
        (0..4).map(|i| line(s, c[i], c[(i + 1) % 4])).collect()
    }

    fn run(s: &Sketch) -> Result<ProfileReport, ProfileError> {
        profile_report(s, &ProfileOptions::default())
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= TOL
    }

    #[test]
    fn a_rectangle_with_near_coincident_endpoints() {
        let mut s = Sketch::new();
        // Each shared corner gets two endpoints 8.5e-10 apart.
        let e = 3e-10;
        let c = [(0.0, 0.0), (40.0, 0.0), (40.0, 25.0), (0.0, 25.0)];
        for i in 0..4 {
            let (a, b) = (c[i], c[(i + 1) % 4]);
            line(&mut s, (a.0 + e, a.1 + e), (b.0 - e, b.1 - e));
        }
        s.add(Geometry::Point(p(5.0, 5.0))).unwrap(); // ignored
        let r = run(&s).unwrap();
        assert_eq!(r.regions.len(), 1);
        assert!(r.open_edges.is_empty());
        let reg = &r.regions[0];
        assert_eq!(reg.outer.edges.len(), 4);
        assert!(reg.holes.is_empty());
        assert!(close(reg.outer.signed_area, 1000.0), "{}", reg.area());
        assert!(close(reg.area(), 1000.0));
    }

    #[test]
    fn b_rectangle_with_circular_hole() {
        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 40.0, 25.0);
        let c = circle(&mut s, 20.0, 12.5, 5.0);
        let r = profiles(&s, &ProfileOptions::default()).unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].outer.signed_area > 0.0);
        assert_eq!(r[0].holes.len(), 1);
        let hole = &r[0].holes[0];
        assert!(hole.signed_area < 0.0, "hole must run clockwise");
        assert_eq!(
            hole.edges,
            vec![OrientedEdge {
                geom: c,
                reversed: true
            }]
        );
        assert!(close(r[0].area(), 1000.0 - 25.0 * PI));
    }

    #[test]
    fn c_slot_area_is_exact() {
        let mut s = Sketch::new();
        line(&mut s, (0.0, -5.0), (30.0, -5.0));
        arc(&mut s, 30.0, 0.0, 5.0, -FRAC_PI_2, FRAC_PI_2);
        line(&mut s, (30.0, 5.0), (0.0, 5.0));
        arc(&mut s, 0.0, 0.0, 5.0, FRAC_PI_2, 3.0 * FRAC_PI_2);
        let r = run(&s).unwrap();
        assert_eq!(r.regions.len(), 1);
        assert_eq!(r.regions[0].outer.edges.len(), 4);
        assert!(close(r.regions[0].area(), 300.0 + 25.0 * PI));
    }

    #[test]
    fn d_separate_and_nested_loops() {
        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 10.0, 10.0);
        rect(&mut s, 20.0, 0.0, 10.0, 5.0);
        let r = run(&s).unwrap();
        let mut areas: Vec<f64> = r.regions.iter().map(Region::area).collect();
        areas.sort_by(f64::total_cmp);
        assert_eq!(areas.len(), 2);
        assert!(close(areas[0], 50.0) && close(areas[1], 100.0));

        // Rectangle in a circle in a larger rectangle.
        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 100.0, 100.0);
        circle(&mut s, 50.0, 50.0, 40.0);
        rect(&mut s, 40.0, 40.0, 20.0, 20.0);
        let r = run(&s).unwrap();
        assert_eq!(r.regions.len(), 2);
        let outer = r.regions.iter().find(|g| g.holes.len() == 1).unwrap();
        let inner = r.regions.iter().find(|g| g.holes.is_empty()).unwrap();
        assert!(close(outer.area(), 10000.0 - 1600.0 * PI));
        assert!(close(inner.area(), 400.0));
    }

    #[test]
    fn e_open_geometry_is_reported_not_an_error() {
        let mut s = Sketch::new();
        let a = line(&mut s, (0.0, 0.0), (10.0, 0.0));
        let b = line(&mut s, (10.0, 0.0), (10.0, 5.0));
        let r = run(&s).unwrap();
        assert!(r.regions.is_empty());
        assert_eq!(r.open_edges, vec![a, b]);

        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 40.0, 25.0);
        let tail = line(&mut s, (40.0, 25.0), (50.0, 35.0));
        let r = run(&s).unwrap();
        assert_eq!(r.regions.len(), 1);
        assert!(close(r.regions[0].area(), 1000.0));
        assert_eq!(r.open_edges, vec![tail]);

        assert_eq!(run(&Sketch::new()).unwrap(), ProfileReport::default());
    }

    #[test]
    fn f_crossing_or_overlapping_edges_are_rejected() {
        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 40.0, 25.0);
        let d1 = line(&mut s, (0.0, 0.0), (40.0, 25.0));
        let d2 = line(&mut s, (0.0, 25.0), (40.0, 0.0));
        assert_eq!(run(&s), Err(ProfileError::SelfIntersecting(d1, d2)));

        let mut s = Sketch::new();
        let r = rect(&mut s, 0.0, 0.0, 40.0, 25.0);
        let dup = line(&mut s, (10.0, 0.0), (20.0, 0.0));
        assert_eq!(run(&s), Err(ProfileError::SelfIntersecting(r[0], dup)));

        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 40.0, 25.0);
        let c = circle(&mut s, 40.0, 12.5, 5.0);
        assert!(matches!(run(&s), Err(ProfileError::SelfIntersecting(_, x)) if x == c));
    }

    #[test]
    fn g_shared_edges_bridges_and_split_circles() {
        // A rectangle split by a middle line: two adjacent regions.
        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 40.0, 25.0);
        line(&mut s, (20.0, 0.0), (20.0, 25.0));
        assert_eq!(
            run(&s),
            Err(ProfileError::SelfIntersecting(GeomId(0), GeomId(4)))
        );
        let mut s = Sketch::new();
        line(&mut s, (0.0, 0.0), (20.0, 0.0));
        line(&mut s, (20.0, 0.0), (40.0, 0.0));
        line(&mut s, (40.0, 0.0), (40.0, 25.0));
        line(&mut s, (40.0, 25.0), (20.0, 25.0));
        line(&mut s, (20.0, 25.0), (0.0, 25.0));
        line(&mut s, (0.0, 25.0), (0.0, 0.0));
        line(&mut s, (20.0, 0.0), (20.0, 25.0));
        let r = run(&s).unwrap();
        assert_eq!(r.regions.len(), 2);
        assert!(r.regions.iter().all(|g| close(g.area(), 500.0)));

        // Two rectangles joined by a line: the line is a bridge.
        let mut s = Sketch::new();
        rect(&mut s, 0.0, 0.0, 10.0, 10.0);
        rect(&mut s, 20.0, 0.0, 10.0, 10.0);
        let bridge = line(&mut s, (10.0, 10.0), (20.0, 0.0));
        let r = run(&s).unwrap();
        assert_eq!(r.regions.len(), 2);
        assert_eq!(r.open_edges, vec![bridge]);

        // Two semicircles make one disc.
        let mut s = Sketch::new();
        arc(&mut s, 0.0, 0.0, 2.0, 0.0, PI);
        arc(&mut s, 0.0, 0.0, 2.0, PI, TAU);
        let r = run(&s).unwrap();
        assert_eq!(r.regions.len(), 1);
        assert!(close(r.regions[0].area(), 4.0 * PI));
    }

    #[test]
    fn h_invalid_tolerance() {
        let opts = ProfileOptions {
            weld_tolerance: Some(0.0),
        };
        assert_eq!(
            profiles(&Sketch::new(), &opts),
            Err(ProfileError::InvalidTolerance(0.0))
        );
    }
}
