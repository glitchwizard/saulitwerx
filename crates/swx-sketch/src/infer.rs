//! Constraint inference: the constraints a newly drawn entity already almost
//! satisfies.
//!
//! [`infer`] compares one new entity against every other entity of the sketch
//! and suggests, in this fixed order:
//!
//! 1. **Coincident** — for each point of the new entity (a line's start, then
//!    its end; a point; a circle's center; an arc's center, start, then end),
//!    the nearest other point, line or arc endpoint, or center within
//!    `point_tol`. Only that nearest candidate is suggested; ties go to the
//!    lower [`GeomId`], then to that same order.
//! 2. **Horizontal / Vertical** — a line within `angle_tol_deg` of the x or y
//!    axis. Never both, since the tolerance is below 45°.
//! 3. **Parallel / Perpendicular** — a line within `angle_tol_deg` of
//!    parallel or perpendicular to another line. At most one is suggested,
//!    the closest in angle (ties to the lower id), because one of them already
//!    fixes the new line's direction. When step 2 fixed the direction, lines
//!    that are themselves horizontal or vertical (within `angle_tol_deg`, or
//!    constrained so) are skipped, as the result would be redundant.
//! 4. **Tangent** — a line whose distance to a circle's or arc's center
//!    differs from its radius by less than `point_tol`, and two circles or
//!    arcs whose center distance is within `point_tol` of `r₁ + r₂` or
//!    `|r₁ − r₂|` (Bowyer & Woodwark, *A Programmer's Geometry*, ch. 3 and 5;
//!    the same conditions the tangent constraint solves for). Concentric
//!    pairs (center distance within `point_tol`) are skipped.
//!
//! The line-to-circle test uses the infinite line through the segment, like
//! the tangent constraint itself. A suggestion that the sketch already holds
//! (in either argument order) is left out, so inferring twice adds nothing.
//! [`infer`] never suggests a constraint that [`Sketch::add_constraint`]
//! would reject.

use crate::solver::{SolveError, SolveOptions, SolveReport};
use crate::{Constraint, ConstraintId, GeomId, GeomKind, PointRef, Sketch, SketchError};
use std::fmt;

/// Tolerances for [`infer`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InferOptions {
    /// Distance below which two points count as coincident, and below which a
    /// tangency gap is closed. Sketch length units.
    pub point_tol: f64,
    /// Angle below which a line counts as horizontal, vertical, parallel or
    /// perpendicular. Degrees.
    pub angle_tol_deg: f64,
}

impl InferOptions {
    /// The defaults for `sketch`: `point_tol` is 1e-3 times the diagonal of
    /// its bounding box (1e-3 for an empty or zero-size sketch), and
    /// `angle_tol_deg` is 1°. Circles and arcs count with their full circle.
    pub fn for_sketch(sketch: &Sketch) -> Self {
        let diagonal = bounding_diagonal(sketch);
        InferOptions {
            point_tol: if diagonal > 0.0 {
                1e-3 * diagonal
            } else {
                1e-3
            },
            angle_tol_deg: 1.0,
        }
    }

    fn validate(&self) -> Result<(), SketchError> {
        for v in [self.point_tol, self.angle_tol_deg] {
            if !v.is_finite() {
                return Err(SketchError::NonFinite);
            }
            if v <= 0.0 {
                return Err(SketchError::NonPositiveValue(v));
            }
        }
        Ok(())
    }
}

/// Errors from [`add_inferred`].
#[derive(Debug, Clone, PartialEq)]
pub enum InferError {
    /// Invalid options or an unknown entity (see [`infer`]).
    Sketch(SketchError),
    /// The solver could not start.
    Solve(SolveError),
    /// The constraints were added, but the solve did not converge.
    NotConverged(SolveReport),
}

impl fmt::Display for InferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InferError::Sketch(e) => write!(f, "{e}"),
            InferError::Solve(e) => write!(f, "{e}"),
            InferError::NotConverged(r) => write!(
                f,
                "solve did not converge (residual {:e})",
                r.final_residual_norm
            ),
        }
    }
}

impl std::error::Error for InferError {}

impl From<SketchError> for InferError {
    fn from(e: SketchError) -> Self {
        InferError::Sketch(e)
    }
}

type Xy = (f64, f64);

/// Suggests the constraints for `new` against all other geometry, in the
/// order of the module docs. Errors on invalid options (≤ 0 or not finite)
/// and on an unknown `new`.
pub fn infer(
    sketch: &Sketch,
    new: GeomId,
    opts: &InferOptions,
) -> Result<Vec<Constraint>, SketchError> {
    opts.validate()?;
    let kind = sketch.slot(new)?.kind;
    let others: Vec<GeomId> = (0..sketch.len() as u32)
        .map(GeomId)
        .filter(|&id| id != new)
        .collect();
    let angle_tol = opts.angle_tol_deg.to_radians();
    let mut out = Vec::new();

    // 1. Coincident: nearest other point per point of `new`.
    let candidates: Vec<(PointRef, Xy)> = others
        .iter()
        .flat_map(|&id| point_refs(sketch, id))
        .collect();
    for (p, at) in point_refs(sketch, new) {
        let nearest = candidates
            .iter()
            .map(|&(q, q_at)| (q, dist(at, q_at)))
            .filter(|&(_, d)| d <= opts.point_tol)
            .min_by(|a, b| a.1.total_cmp(&b.1));
        if let Some((q, _)) = nearest {
            out.push(Constraint::Coincident(p, q));
        }
    }

    if kind == GeomKind::Line {
        let dir = line_dir(sketch, new);
        // 2. Horizontal / vertical.
        let axis = dir.and_then(|u| axis_of(u, angle_tol));
        match axis {
            Some(Axis::X) => out.push(Constraint::Horizontal(new)),
            Some(Axis::Y) => out.push(Constraint::Vertical(new)),
            None => {}
        }

        // 3. Parallel / perpendicular: the closest match only.
        if let Some(u) = dir {
            let best = others
                .iter()
                .filter(|&&id| sketch.slot(id).map(|s| s.kind) == Ok(GeomKind::Line))
                .filter(|&&id| axis.is_none() || !is_axis_line(sketch, id, angle_tol))
                .filter_map(|&id| {
                    let v = line_dir(sketch, id)?;
                    // Angle between the undirected lines, in [0, π/2].
                    let a = cross(u, v).abs().atan2(dot(u, v).abs());
                    let perp = std::f64::consts::FRAC_PI_2 - a;
                    if a <= angle_tol {
                        Some((a, Constraint::Parallel(new, id)))
                    } else if perp <= angle_tol {
                        Some((perp, Constraint::Perpendicular(new, id)))
                    } else {
                        None
                    }
                })
                .min_by(|a, b| a.0.total_cmp(&b.0));
            if let Some((_, c)) = best {
                out.push(c);
            }
        }
    }

    // 4. Tangent.
    for &id in &others {
        if tangent_within(sketch, new, id, opts.point_tol) {
            out.push(Constraint::Tangent(new, id));
        }
    }

    out.retain(|c| !already_held(sketch, c));
    Ok(out)
}

/// Adds the suggestions of [`infer`] in order, then solves the sketch, and
/// returns the new constraint ids. If the solve does not converge the
/// constraints stay added and the geometry is the solver's best attempt.
pub fn add_inferred(
    sketch: &mut Sketch,
    new: GeomId,
    opts: &InferOptions,
) -> Result<Vec<ConstraintId>, InferError> {
    let suggestions = infer(sketch, new, opts)?;
    let ids = suggestions
        .into_iter()
        .map(|c| sketch.add_constraint(c))
        .collect::<Result<Vec<_>, _>>()?;
    let report = sketch
        .solve(SolveOptions::default())
        .map_err(InferError::Solve)?;
    if !report.converged {
        return Err(InferError::NotConverged(report));
    }
    Ok(ids)
}

#[derive(Clone, Copy, PartialEq)]
enum Axis {
    X,
    Y,
}

fn axis_of(u: Xy, tol: f64) -> Option<Axis> {
    // Angle to the x axis of the undirected line, in [0, π/2].
    let a = u.1.abs().atan2(u.0.abs());
    if a <= tol {
        Some(Axis::X)
    } else if std::f64::consts::FRAC_PI_2 - a <= tol {
        Some(Axis::Y)
    } else {
        None
    }
}

/// A line within `tol` of an axis, or constrained horizontal or vertical.
fn is_axis_line(sketch: &Sketch, id: GeomId, tol: f64) -> bool {
    line_dir(sketch, id).is_some_and(|v| axis_of(v, tol).is_some())
        || sketch
            .constraints()
            .iter()
            .any(|c| matches!(*c, Constraint::Horizontal(l) | Constraint::Vertical(l) if l == id))
}

/// The point locations of an entity that a [`PointRef`] can name.
fn point_refs(sketch: &Sketch, id: GeomId) -> Vec<(PointRef, Xy)> {
    let (Ok(slot), Ok(p)) = (sketch.slot(id), sketch.entity_params(id)) else {
        return Vec::new();
    };
    match slot.kind {
        GeomKind::Point => vec![(PointRef::Point(id), (p[0], p[1]))],
        GeomKind::Line => vec![
            (PointRef::Start(id), (p[0], p[1])),
            (PointRef::End(id), (p[2], p[3])),
        ],
        GeomKind::Circle => vec![(PointRef::Center(id), (p[0], p[1]))],
        GeomKind::Arc => {
            let at = |a: f64| (p[0] + p[2] * a.cos(), p[1] + p[2] * a.sin());
            vec![
                (PointRef::Center(id), (p[0], p[1])),
                (PointRef::ArcStart(id), at(p[3])),
                (PointRef::ArcEnd(id), at(p[4])),
            ]
        }
    }
}

/// A line's start and direction `e − s`, or `None` for a zero-length line or
/// another kind.
fn line(sketch: &Sketch, id: GeomId) -> Option<(Xy, Xy)> {
    if sketch.slot(id).ok()?.kind != GeomKind::Line {
        return None;
    }
    let p = sketch.entity_params(id).ok()?;
    let u = (p[2] - p[0], p[3] - p[1]);
    (u != (0.0, 0.0)).then_some(((p[0], p[1]), u))
}

fn line_dir(sketch: &Sketch, id: GeomId) -> Option<Xy> {
    line(sketch, id).map(|(_, u)| u)
}

/// Center and radius of a circle or arc.
fn round(sketch: &Sketch, id: GeomId) -> Option<(Xy, f64)> {
    match sketch.slot(id).ok()?.kind {
        GeomKind::Circle | GeomKind::Arc => {
            let p = sketch.entity_params(id).ok()?;
            Some(((p[0], p[1]), p[2]))
        }
        _ => None,
    }
}

fn tangent_within(sketch: &Sketch, a: GeomId, b: GeomId, tol: f64) -> bool {
    let line_circle = |l: GeomId, c: GeomId| match (line(sketch, l), round(sketch, c)) {
        (Some((s, u)), Some((center, r))) => {
            let d = cross(u, (center.0 - s.0, center.1 - s.1)).abs() / u.0.hypot(u.1);
            (d - r).abs() < tol
        }
        _ => false,
    };
    if line_circle(a, b) || line_circle(b, a) {
        return true;
    }
    match (round(sketch, a), round(sketch, b)) {
        (Some((c1, r1)), Some((c2, r2))) => {
            let d = dist(c1, c2);
            d > tol && ((d - (r1 + r2)).abs() < tol || (d - (r1 - r2).abs()).abs() < tol)
        }
        _ => false,
    }
}

/// Whether the sketch already holds `c`, in either argument order.
fn already_held(sketch: &Sketch, c: &Constraint) -> bool {
    let swapped = match *c {
        Constraint::Coincident(p, q) => Constraint::Coincident(q, p),
        Constraint::Parallel(a, b) => Constraint::Parallel(b, a),
        Constraint::Perpendicular(a, b) => Constraint::Perpendicular(b, a),
        Constraint::Tangent(a, b) => Constraint::Tangent(b, a),
        other => other,
    };
    sketch
        .constraints()
        .iter()
        .any(|held| *held == *c || *held == swapped)
}

fn bounding_diagonal(sketch: &Sketch) -> f64 {
    let mut lo = (f64::INFINITY, f64::INFINITY);
    let mut hi = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    let mut grow = |(x, y): Xy, r: f64| {
        lo = (lo.0.min(x - r), lo.1.min(y - r));
        hi = (hi.0.max(x + r), hi.1.max(y + r));
    };
    for id in (0..sketch.len() as u32).map(GeomId) {
        match round(sketch, id) {
            Some((c, r)) => grow(c, r.abs()),
            None => point_refs(sketch, id)
                .into_iter()
                .for_each(|(_, p)| grow(p, 0.0)),
        }
    }
    if lo.0.is_finite() {
        dist(lo, hi)
    } else {
        0.0
    }
}

fn dist(a: Xy, b: Xy) -> f64 {
    (b.0 - a.0).hypot(b.1 - a.1)
}

fn cross(a: Xy, b: Xy) -> f64 {
    a.0 * b.1 - a.1 * b.0
}

fn dot(a: Xy, b: Xy) -> f64 {
    a.0 * b.0 + a.1 * b.1
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dof::DofOptions;
    use crate::{Arc, Circle, Geometry, Line, Point};
    use std::f64::consts::FRAC_PI_2;

    fn pt(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }

    fn line(s: &mut Sketch, x0: f64, y0: f64, x1: f64, y1: f64) -> GeomId {
        s.add(Geometry::Line(Line::new(pt(x0, y0), pt(x1, y1)).unwrap()))
            .unwrap()
    }

    /// A line from `(x, y)` of length `len` at `deg` degrees.
    fn ray(s: &mut Sketch, x: f64, y: f64, len: f64, deg: f64) -> GeomId {
        let a = deg.to_radians();
        line(s, x, y, x + len * a.cos(), y + len * a.sin())
    }

    fn circle(s: &mut Sketch, x: f64, y: f64, r: f64) -> GeomId {
        s.add(Geometry::Circle(Circle::new(pt(x, y), r).unwrap()))
            .unwrap()
    }

    fn opts() -> InferOptions {
        InferOptions {
            point_tol: 0.01,
            angle_tol_deg: 1.0,
        }
    }

    #[test]
    fn default_options_scale_with_the_sketch() {
        let empty = InferOptions::for_sketch(&Sketch::new());
        assert_eq!(empty.point_tol, 1e-3);
        assert_eq!(empty.angle_tol_deg, 1.0);
        let mut s = Sketch::new();
        line(&mut s, 0.0, 0.0, 30.0, 40.0);
        assert!((InferOptions::for_sketch(&s).point_tol - 0.05).abs() < 1e-12);
        // A circle counts with its full extent: box 20 × 20 around (0, 0).
        let mut s = Sketch::new();
        circle(&mut s, 0.0, 0.0, 10.0);
        let want = 1e-3 * 20.0 * 2f64.sqrt();
        assert!((InferOptions::for_sketch(&s).point_tol - want).abs() < 1e-12);
    }

    #[test]
    fn invalid_options_are_rejected() {
        let mut s = Sketch::new();
        let l = line(&mut s, 0.0, 0.0, 1.0, 0.0);
        for (point_tol, angle_tol_deg, err) in [
            (0.0, 1.0, SketchError::NonPositiveValue(0.0)),
            (-1.0, 1.0, SketchError::NonPositiveValue(-1.0)),
            (0.1, 0.0, SketchError::NonPositiveValue(0.0)),
            (f64::NAN, 1.0, SketchError::NonFinite),
            (0.1, f64::INFINITY, SketchError::NonFinite),
        ] {
            let o = InferOptions {
                point_tol,
                angle_tol_deg,
            };
            assert_eq!(infer(&s, l, &o), Err(err));
        }
        assert_eq!(
            infer(&s, GeomId(9), &opts()),
            Err(SketchError::UnknownGeom(GeomId(9)))
        );
    }

    #[test]
    fn rule_a_coincident_picks_the_nearest_candidate() {
        let mut s = Sketch::new();
        let a = line(&mut s, -5.0, 3.0, 0.004, 3.0);
        let p = s.add(Geometry::Point(pt(0.001, 3.0))).unwrap();
        let c = circle(&mut s, 20.0, 9.0, 2.0);
        // Start is near a's end (0.006) and p (0.003): p wins. End is near c.
        let new = line(&mut s, -0.002, 3.0, 20.005, 9.0);
        let got = infer(&s, new, &opts()).unwrap();
        assert_eq!(
            got,
            vec![
                Constraint::Coincident(PointRef::Start(new), PointRef::Point(p)),
                Constraint::Coincident(PointRef::End(new), PointRef::Center(c)),
            ]
        );
        let _ = a;
    }

    #[test]
    fn rule_a_arc_ends_are_candidates_both_ways() {
        // Quarter arc from (5, 0) to (0, 5); a line drawn down from near its
        // start joins it, tangent and vertical.
        let mut s = Sketch::new();
        let arc = s
            .add(Geometry::Arc(
                Arc::new(pt(0.0, 0.0), 5.0, 0.0, FRAC_PI_2).unwrap(),
            ))
            .unwrap();
        let l = line(&mut s, 5.004, -0.003, 5.004, -10.0);
        assert_eq!(
            infer(&s, l, &opts()).unwrap(),
            vec![
                Constraint::Coincident(PointRef::Start(l), PointRef::ArcStart(arc)),
                Constraint::Vertical(l),
                Constraint::Tangent(l, arc),
            ]
        );
        add_inferred(&mut s, l, &opts()).unwrap();
        let (q, a) = (s.entity_params(l).unwrap(), s.entity_params(arc).unwrap());
        let start = (a[0] + a[2] * a[3].cos(), a[1] + a[2] * a[3].sin());
        assert!(dist((q[0], q[1]), start) < 1e-8, "{q:?} {a:?}");

        // A new arc whose end lands near an existing line's start.
        let mut s = Sketch::new();
        let l = line(&mut s, -0.002, 5.003, -10.0, 5.003);
        let arc = s
            .add(Geometry::Arc(
                Arc::new(pt(0.0, 0.0), 5.0, 0.0, FRAC_PI_2).unwrap(),
            ))
            .unwrap();
        assert_eq!(
            infer(&s, arc, &opts()).unwrap()[0],
            Constraint::Coincident(PointRef::ArcEnd(arc), PointRef::Start(l))
        );
    }

    #[test]
    fn rule_b_horizontal_or_vertical_never_both() {
        let mut s = Sketch::new();
        let h = ray(&mut s, 0.0, 0.0, 10.0, 180.6);
        let v = ray(&mut s, 50.0, 50.0, 10.0, 89.3);
        assert_eq!(
            infer(&s, h, &opts()).unwrap(),
            vec![Constraint::Horizontal(h)]
        );
        assert_eq!(
            infer(&s, v, &opts()).unwrap(),
            vec![Constraint::Vertical(v)]
        );
        // Huge tolerance still gives just one.
        let wide = InferOptions {
            point_tol: 0.01,
            angle_tol_deg: 60.0,
        };
        let d = ray(&mut s, 100.0, 100.0, 10.0, 45.0);
        let got = infer(&s, d, &wide).unwrap();
        let axes = got
            .iter()
            .filter(|c| matches!(c, Constraint::Horizontal(_) | Constraint::Vertical(_)))
            .count();
        assert_eq!(axes, 1);
    }

    #[test]
    fn rule_c_parallel_and_perpendicular_to_the_closest_line() {
        let mut s = Sketch::new();
        let a = ray(&mut s, 0.0, 0.0, 10.0, 30.0);
        let b = ray(&mut s, 0.0, 20.0, 10.0, 119.2);
        let c = ray(&mut s, 0.0, 40.0, 10.0, 30.9);
        // 30.4°: parallel to a (0.4°), c (0.5°), perpendicular to b (1.2°, out).
        let new = ray(&mut s, 50.0, 0.0, 10.0, 30.4);
        assert_eq!(
            infer(&s, new, &opts()).unwrap(),
            vec![Constraint::Parallel(new, a)]
        );
        // 29.5°: perpendicular to b (0.3°) beats parallel to a (0.5°).
        let new2 = ray(&mut s, 50.0, 20.0, 10.0, 29.5);
        assert_eq!(
            infer(&s, new2, &opts()).unwrap(),
            vec![Constraint::Perpendicular(new2, b)]
        );
        let _ = c;
    }

    #[test]
    fn rule_c_skips_axis_lines_when_the_direction_is_already_fixed() {
        let mut s = Sketch::new();
        let base = line(&mut s, 0.0, 0.0, 10.0, 0.0);
        s.add_constraint(Constraint::Horizontal(base)).unwrap();
        let slanted = ray(&mut s, 0.0, 30.0, 10.0, 0.3);
        let new = ray(&mut s, 0.0, 10.0, 10.0, 0.5);
        // Horizontal only: no Parallel to `base` (constrained) or `slanted`
        // (within tolerance of horizontal).
        assert_eq!(
            infer(&s, new, &opts()).unwrap(),
            vec![Constraint::Horizontal(new)]
        );
        let steep = ray(&mut s, 0.0, 20.0, 10.0, 3.0);
        assert_eq!(infer(&s, steep, &opts()).unwrap(), vec![]);
        let _ = slanted;
    }

    #[test]
    fn rule_d_tangent_line_and_circles() {
        let mut s = Sketch::new();
        let c = circle(&mut s, 0.0, 0.0, 5.0);
        // A 45° line at distance 5.004 from the center.
        let k = 5.004 * 2f64.sqrt();
        let l = line(&mut s, k, 0.0, 0.0, k);
        assert_eq!(
            infer(&s, l, &opts()).unwrap(),
            vec![Constraint::Tangent(l, c)]
        );
        // External: center distance 8.006 vs 5 + 3.
        let ext = circle(&mut s, 8.006, 0.0, 3.0);
        assert_eq!(
            infer(&s, ext, &opts()).unwrap(),
            vec![Constraint::Tangent(ext, c)]
        );
        // Internal, with an arc: center distance 2.995 vs 5 − 2.
        let arc = s
            .add(Geometry::Arc(
                Arc::new(pt(-2.995, 0.0), 2.0, 1.0, 5.0).unwrap(),
            ))
            .unwrap();
        assert_eq!(
            infer(&s, arc, &opts()).unwrap(),
            vec![Constraint::Tangent(arc, c)]
        );
        // Concentric circles of equal radius are not tangent to each other.
        let same = circle(&mut s, 0.0, 0.0, 5.0);
        assert!(!infer(&s, same, &opts())
            .unwrap()
            .contains(&Constraint::Tangent(same, c)));
    }

    #[test]
    fn rule_e_nothing_within_tolerance() {
        let mut s = Sketch::new();
        line(&mut s, 0.0, 0.0, 10.0, 0.0);
        circle(&mut s, 30.0, 30.0, 4.0);
        let new = ray(&mut s, 0.5, 0.5, 7.0, 40.0);
        assert_eq!(infer(&s, new, &opts()).unwrap(), vec![]);
    }

    #[test]
    fn suggestions_are_ordered_and_not_repeated() {
        let mut s = Sketch::new();
        let base = ray(&mut s, 0.0, 0.0, 10.0, 20.0);
        let c = circle(&mut s, 30.0, 0.0, 4.003);
        // Starts at base's start, parallel to base, tangent to c... and the
        // circle check is last.
        let new = ray(&mut s, 0.002, 0.0, 30.0, 20.3);
        let (sx, sy) = (0.002, 0.0);
        let u = (20.3f64.to_radians().cos(), 20.3f64.to_radians().sin());
        let d = cross(u, (30.0 - sx, 0.0 - sy)).abs();
        s.set(
            c,
            Geometry::Circle(Circle::new(pt(30.0, 0.0), d + 0.004).unwrap()),
        )
        .unwrap();
        let got = infer(&s, new, &opts()).unwrap();
        assert_eq!(
            got,
            vec![
                Constraint::Coincident(PointRef::Start(new), PointRef::Start(base)),
                Constraint::Parallel(new, base),
                Constraint::Tangent(new, c),
            ]
        );
        for c in got {
            s.add_constraint(c).unwrap();
        }
        assert_eq!(infer(&s, new, &opts()).unwrap(), vec![]);
    }

    /// The rectangle of the issue: about 40 × 25 (diagonal ≈ 47.2), drawn as
    /// four lines whose consecutive corners are off by up to 5e-4 and whose
    /// sides are tilted by up to 0.5°. Inference then runs on each line in
    /// drawing order.
    #[test]
    fn wobbly_rectangle_solves_exactly() {
        let mut s = Sketch::new();
        let sides = [
            (0.0, 0.0, 40.0, 0.349),            // 0.50°
            (40.0004, 0.3493, 39.8, 25.1),      // 0.46°
            (39.8003, 25.0996, -0.15, 24.9),    // 0.29°
            (-0.1497, 24.9004, 0.0003, 0.0004), // 0.35°
        ];
        let lines: Vec<GeomId> = sides
            .iter()
            .map(|&(x0, y0, x1, y1)| line(&mut s, x0, y0, x1, y1))
            .collect();
        let o = InferOptions::for_sketch(&s);
        assert!((o.point_tol - 0.0472).abs() < 1e-3, "{o:?}");
        for &l in &lines {
            add_inferred(&mut s, l, &o).unwrap();
        }
        let report = s.solve(SolveOptions::default()).unwrap();
        assert!(report.converged);
        let p = |l: GeomId| {
            let q = s.entity_params(l).unwrap();
            [(q[0], q[1]), (q[2], q[3])]
        };
        for i in 0..4 {
            let end = p(lines[i])[1];
            let start = p(lines[(i + 1) % 4])[0];
            assert!(dist(end, start) < 1e-8, "corner {i}: {end:?} {start:?}");
            let [a, b] = p(lines[i]);
            let off = if i % 2 == 0 { b.1 - a.1 } else { b.0 - a.0 };
            assert!(off.abs() < 1e-8, "side {i} off by {off}");
        }
        let dof = s.dof(DofOptions::default()).unwrap();
        assert!(dof.report.redundant.is_empty(), "{dof:?}");
        assert_eq!(dof.report.dof, 4);
    }

    #[test]
    fn add_inferred_reports_invalid_options() {
        let mut s = Sketch::new();
        let l = line(&mut s, 0.0, 0.0, 1.0, 0.0);
        let bad = InferOptions {
            point_tol: 0.0,
            angle_tol_deg: 1.0,
        };
        assert_eq!(
            add_inferred(&mut s, l, &bad),
            Err(InferError::Sketch(SketchError::NonPositiveValue(0.0)))
        );
        assert!(s.constraints().is_empty());
    }
}
