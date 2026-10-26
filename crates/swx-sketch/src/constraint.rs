//! Geometric constraints as residual equations.
//!
//! Each [`Constraint`] is resolved, when it is added to a [`Sketch`], into
//! parameter indices of the sketch vector, and contributes one or two rows to
//! the combined residual vector `r(x)` that [`solver::solve`] drives to zero:
//!
//! | Constraint | Rows | Residual |
//! |---|---|---|
//! | `Coincident(p, q)` | 2 | `xq − xp`, `yq − yp` |
//! | `Horizontal` | 1 | `yq − yp` |
//! | `Vertical` | 1 | `xq − xp` |
//! | `Fixed { p, x, y }` | 2 | `xp − x`, `yp − y` |
//! | `Distance { p, q, d }` | 1 | `‖q − p‖ − d` |
//! | `Radius` / `Diameter` | 1 | `r − value` (`r − value/2` for diameter) |
//! | `Parallel(l1, l2)` | 1 | `u₁ × u₂` |
//! | `Perpendicular(l1, l2)` | 1 | `u₁ · u₂` |
//! | `Equal` (lines) | 1 | `‖u₂‖ − ‖u₁‖` |
//! | `Equal` (circles/arcs) | 1 | `r₂ − r₁` |
//! | `Midpoint { point, line }` | 2 | `(s + e)/2 − p` |
//! | `Symmetric { a, b, line }` | 2 | `u × ((a + b)/2 − s)`, `(b − a) · u` |
//! | `Tangent` (line, circle/arc) | 1 | `σ·(u × (c − s))/‖u‖ − r` |
//! | `Tangent` (circle/arc, circle/arc) | 1 | `‖c₂ − c₁‖ − (r₁ + r₂)` or `‖c₂ − c₁‖ − σ·(r₁ − r₂)` |
//!
//! Here `s` and `e` are a line's start and end, `u = e − s` its direction,
//! `c` and `r` a circle's center and radius, and `×` the 2D cross product
//! `ax·by − ay·bx`.
//!
//! Every row has an analytic Jacobian. Distance uses the unsquared Euclidean
//! norm so its residual is in length units like the others; its gradient
//! `±(q − p)/‖q − p‖` is the unit direction vector, taken as zero when the
//! points coincide. Parallel and perpendicular use the unnormalized cross and
//! dot products of the two direction vectors, which vanish exactly when the
//! lines are parallel (`sin θ = 0`) or perpendicular (`cos θ = 0`); symmetry
//! is the standard reflection condition, the midpoint of `a b` on the line and
//! `a b` perpendicular to it.
//!
//! Tangency follows the analytic conditions in Bowyer & Woodwark, *A
//! Programmer's Geometry* (Butterworths, 1983), ch. 3 and 5: a line is tangent
//! to a circle when the distance from the center to the line equals the
//! radius, and two circles are tangent when the distance between their
//! centers is `r₁ + r₂` (external) or `|r₁ − r₂|` (internal). The distance to
//! the line is signed (`u × (c − s)/‖u‖`, positive with the center to the left
//! of the line), and the sign `σ = ±1` is fixed when the constraint is added:
//! the side the center is on, or for two circles which radius is larger. That
//! keeps every residual smooth near its solution. Whether two circles are
//! tangent externally or internally is likewise chosen when the constraint is
//! added, as whichever condition the current geometry is closer to.

use crate::solver::{self, Residuals, SolveError, SolveOptions, SolveReport};
use crate::{GeomId, GeomKind, Sketch, SketchError};

/// A point-valued location in a sketch: a point entity, a line endpoint, or
/// the center of a circle or arc.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointRef {
    /// A [`Point`](crate::Point) entity.
    Point(GeomId),
    /// The start point of a [`Line`](crate::Line).
    Start(GeomId),
    /// The end point of a [`Line`](crate::Line).
    End(GeomId),
    /// The center of a [`Circle`](crate::Circle) or [`Arc`](crate::Arc).
    Center(GeomId),
}

impl PointRef {
    fn geom(self) -> GeomId {
        match self {
            PointRef::Point(id)
            | PointRef::Start(id)
            | PointRef::End(id)
            | PointRef::Center(id) => id,
        }
    }
}

/// A geometric constraint between sketch entities.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Constraint {
    /// Two points at the same position.
    Coincident(PointRef, PointRef),
    /// A line with equal start and end y.
    Horizontal(GeomId),
    /// Two points with equal y.
    HorizontalPoints(PointRef, PointRef),
    /// A line with equal start and end x.
    Vertical(GeomId),
    /// Two points with equal x.
    VerticalPoints(PointRef, PointRef),
    /// A point fixed at `(x, y)`.
    Fixed { point: PointRef, x: f64, y: f64 },
    /// Euclidean distance `value > 0` between two points.
    Distance {
        a: PointRef,
        b: PointRef,
        value: f64,
    },
    /// Radius `value > 0` of a circle or arc.
    Radius { geom: GeomId, value: f64 },
    /// Diameter `value > 0` of a circle or arc.
    Diameter { geom: GeomId, value: f64 },
    /// Two lines with parallel directions.
    Parallel(GeomId, GeomId),
    /// Two lines with perpendicular directions.
    Perpendicular(GeomId, GeomId),
    /// Two lines of equal length, or two circles or arcs of equal radius.
    Equal(GeomId, GeomId),
    /// A point at the middle of a line.
    Midpoint { point: PointRef, line: GeomId },
    /// Two points that are mirror images of each other about a line.
    Symmetric {
        a: PointRef,
        b: PointRef,
        line: GeomId,
    },
    /// A line tangent to a circle or arc (either order), or two circles or
    /// arcs tangent to each other. Which side of the line the circle stays
    /// on, and whether two circles touch externally or internally, is taken
    /// from the geometry when the constraint is added (see the module docs).
    Tangent(GeomId, GeomId),
}

/// Stable identifier of a constraint within a [`Sketch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstraintId(pub u32);

/// Parameter indices `(x, y)` of one point in the sketch vector.
type Xy = (usize, usize);

/// Two points `(from, to)`, standing for the vector `to − from`.
type Seg = (Xy, Xy);

/// A constraint reduced to parameter indices `(x, y)` of the sketch vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Resolved {
    /// `params[b] − params[a] = 0` (one row).
    Equal { a: usize, b: usize },
    /// `params[i] − value = 0` (one row).
    Value { i: usize, value: f64 },
    /// `‖(xb, yb) − (xa, ya)‖ − value = 0` (one row).
    Distance { a: Xy, b: Xy, value: f64 },
    /// `u × w = 0` for the vectors `u = a.1 − a.0`, `w = b.1 − b.0`.
    Cross { a: Seg, b: Seg },
    /// `u · w = 0` for the vectors `u = a.1 − a.0`, `w = b.1 − b.0`.
    Dot { a: Seg, b: Seg },
    /// `‖w‖ − ‖u‖ = 0` for the vectors `u = a.1 − a.0`, `w = b.1 − b.0`.
    LengthDiff { a: Seg, b: Seg },
    /// `(params[a] + params[b])/2 − params[m] = 0`.
    Midpoint { m: usize, a: usize, b: usize },
    /// `u × ((a + b)/2 − s) = 0` with `u = e − s` for `line = (s, e)`.
    MidOnLine { line: Seg, a: Xy, b: Xy },
    /// `side·(u × (c − s))/‖u‖ − params[r] = 0` with `u = e − s`.
    LineTangent {
        line: Seg,
        c: Xy,
        r: usize,
        side: f64,
    },
    /// `‖c2 − c1‖ − (k1·params[r1] + k2·params[r2]) = 0`.
    CircleTangent {
        c1: Xy,
        c2: Xy,
        r1: usize,
        r2: usize,
        k1: f64,
        k2: f64,
    },
}

/// The vector `seg.1 − seg.0` at `params`.
fn vec2(params: &[f64], (p, q): Seg) -> (f64, f64) {
    (params[q.0] - params[p.0], params[q.1] - params[p.1])
}

fn cross(a: (f64, f64), b: (f64, f64)) -> f64 {
    a.0 * b.1 - a.1 * b.0
}

/// Adds the gradient `(gx, gy)` with respect to the vector `seg.1 − seg.0`
/// to the gradients of its two points.
fn add_vec(row: &mut [f64], (p, q): Seg, (gx, gy): (f64, f64)) {
    row[q.0] += gx;
    row[q.1] += gy;
    row[p.0] -= gx;
    row[p.1] -= gy;
}

/// `‖v‖` and `v/‖v‖`, or a zero direction when `v = 0`.
fn unit(v: (f64, f64)) -> (f64, f64, f64) {
    let len = v.0.hypot(v.1);
    if len > 0.0 {
        (len, v.0 / len, v.1 / len)
    } else {
        (0.0, 0.0, 0.0)
    }
}

impl Resolved {
    fn eval(&self, params: &[f64]) -> f64 {
        match *self {
            Resolved::Equal { a, b } => params[b] - params[a],
            Resolved::Value { i, value } => params[i] - value,
            Resolved::Distance { a, b, value } => {
                (params[b.0] - params[a.0]).hypot(params[b.1] - params[a.1]) - value
            }
            Resolved::Cross { a, b } => cross(vec2(params, a), vec2(params, b)),
            Resolved::Dot { a, b } => {
                let (u, w) = (vec2(params, a), vec2(params, b));
                u.0 * w.0 + u.1 * w.1
            }
            Resolved::LengthDiff { a, b } => {
                let (u, w) = (vec2(params, a), vec2(params, b));
                w.0.hypot(w.1) - u.0.hypot(u.1)
            }
            Resolved::Midpoint { m, a, b } => (params[a] + params[b]) / 2.0 - params[m],
            Resolved::MidOnLine { line, a, b } => {
                let u = vec2(params, line);
                let s = line.0;
                let v = (
                    (params[a.0] + params[b.0]) / 2.0 - params[s.0],
                    (params[a.1] + params[b.1]) / 2.0 - params[s.1],
                );
                cross(u, v)
            }
            Resolved::LineTangent { line, c, r, side } => {
                let u = vec2(params, line);
                let len = u.0.hypot(u.1);
                let dist = if len > 0.0 {
                    cross(u, vec2(params, (line.0, c))) / len
                } else {
                    0.0
                };
                side * dist - params[r]
            }
            Resolved::CircleTangent {
                c1,
                c2,
                r1,
                r2,
                k1,
                k2,
            } => {
                let d = vec2(params, (c1, c2));
                d.0.hypot(d.1) - (k1 * params[r1] + k2 * params[r2])
            }
        }
    }

    /// Writes the non-zero entries of this row's gradient into `row`, which
    /// is already zeroed.
    fn gradient(&self, params: &[f64], row: &mut [f64]) {
        match *self {
            Resolved::Equal { a, b } => {
                row[a] -= 1.0;
                row[b] += 1.0;
            }
            Resolved::Value { i, .. } => row[i] = 1.0,
            Resolved::Distance { a, b, .. } => {
                let dx = params[b.0] - params[a.0];
                let dy = params[b.1] - params[a.1];
                let len = dx.hypot(dy);
                if len > 0.0 {
                    let (ux, uy) = (dx / len, dy / len);
                    row[a.0] -= ux;
                    row[a.1] -= uy;
                    row[b.0] += ux;
                    row[b.1] += uy;
                }
            }
            Resolved::Cross { a, b } => {
                let (u, w) = (vec2(params, a), vec2(params, b));
                add_vec(row, a, (w.1, -w.0));
                add_vec(row, b, (-u.1, u.0));
            }
            Resolved::Dot { a, b } => {
                let (u, w) = (vec2(params, a), vec2(params, b));
                add_vec(row, a, w);
                add_vec(row, b, u);
            }
            Resolved::LengthDiff { a, b } => {
                let (_, ux, uy) = unit(vec2(params, a));
                let (_, wx, wy) = unit(vec2(params, b));
                add_vec(row, a, (-ux, -uy));
                add_vec(row, b, (wx, wy));
            }
            Resolved::Midpoint { m, a, b } => {
                row[a] += 0.5;
                row[b] += 0.5;
                row[m] -= 1.0;
            }
            Resolved::MidOnLine { line, a, b } => {
                let u = vec2(params, line);
                let s = line.0;
                let v = (
                    (params[a.0] + params[b.0]) / 2.0 - params[s.0],
                    (params[a.1] + params[b.1]) / 2.0 - params[s.1],
                );
                add_vec(row, line, (v.1, -v.0));
                // ∂/∂v = (−uy, ux); v = (a + b)/2 − s.
                for p in [a, b] {
                    row[p.0] -= u.1 / 2.0;
                    row[p.1] += u.0 / 2.0;
                }
                row[s.0] += u.1;
                row[s.1] -= u.0;
            }
            Resolved::LineTangent { line, c, r, side } => {
                row[r] -= 1.0;
                let u = vec2(params, line);
                let len = u.0.hypot(u.1);
                if len > 0.0 {
                    let cs = (line.0, c);
                    let v = vec2(params, cs);
                    let k = cross(u, v) / (len * len * len);
                    // d(C/L)/du = (vy, −vx)/L − C·u/L³, d(C/L)/dv = (−uy, ux)/L.
                    add_vec(
                        row,
                        line,
                        (side * (v.1 / len - k * u.0), side * (-v.0 / len - k * u.1)),
                    );
                    add_vec(row, cs, (side * -u.1 / len, side * u.0 / len));
                }
            }
            Resolved::CircleTangent {
                c1,
                c2,
                r1,
                r2,
                k1,
                k2,
            } => {
                let (_, ux, uy) = unit(vec2(params, (c1, c2)));
                add_vec(row, (c1, c2), (ux, uy));
                row[r1] -= k1;
                row[r2] -= k2;
            }
        }
    }
}

fn check_value(value: f64) -> Result<(), SketchError> {
    if !value.is_finite() {
        Err(SketchError::NonFinite)
    } else if value <= 0.0 {
        Err(SketchError::NonPositiveValue(value))
    } else {
        Ok(())
    }
}

impl Sketch {
    /// Parameter indices `(x, y)` of a point reference.
    fn point_index(&self, p: PointRef) -> Result<Xy, SketchError> {
        let id = p.geom();
        let s = self.slot(id)?;
        let at = match (p, s.kind) {
            (PointRef::Point(_), GeomKind::Point) | (PointRef::Start(_), GeomKind::Line) => 0,
            (PointRef::End(_), GeomKind::Line) => 2,
            (PointRef::Center(_), GeomKind::Circle | GeomKind::Arc) => 0,
            _ => return Err(SketchError::KindMismatch(id)),
        };
        Ok((s.offset + at, s.offset + at + 1))
    }

    fn line_points(&self, id: GeomId) -> Result<(Xy, Xy), SketchError> {
        Ok((
            self.point_index(PointRef::Start(id))?,
            self.point_index(PointRef::End(id))?,
        ))
    }

    fn radius_index(&self, id: GeomId) -> Result<usize, SketchError> {
        let s = self.slot(id)?;
        match s.kind {
            GeomKind::Circle | GeomKind::Arc => Ok(s.offset + 2),
            _ => Err(SketchError::KindMismatch(id)),
        }
    }

    fn resolve(&self, c: &Constraint) -> Result<Vec<Resolved>, SketchError> {
        let equal = |a: usize, b: usize| Resolved::Equal { a, b };
        Ok(match *c {
            Constraint::Coincident(p, q) => {
                let (p, q) = (self.point_index(p)?, self.point_index(q)?);
                vec![equal(p.0, q.0), equal(p.1, q.1)]
            }
            Constraint::Horizontal(line) => {
                let (p, q) = self.line_points(line)?;
                vec![equal(p.1, q.1)]
            }
            Constraint::HorizontalPoints(p, q) => {
                vec![equal(self.point_index(p)?.1, self.point_index(q)?.1)]
            }
            Constraint::Vertical(line) => {
                let (p, q) = self.line_points(line)?;
                vec![equal(p.0, q.0)]
            }
            Constraint::VerticalPoints(p, q) => {
                vec![equal(self.point_index(p)?.0, self.point_index(q)?.0)]
            }
            Constraint::Fixed { point, x, y } => {
                if !(x.is_finite() && y.is_finite()) {
                    return Err(SketchError::NonFinite);
                }
                let (ix, iy) = self.point_index(point)?;
                vec![
                    Resolved::Value { i: ix, value: x },
                    Resolved::Value { i: iy, value: y },
                ]
            }
            Constraint::Distance { a, b, value } => {
                check_value(value)?;
                vec![Resolved::Distance {
                    a: self.point_index(a)?,
                    b: self.point_index(b)?,
                    value,
                }]
            }
            Constraint::Radius { geom, value } => {
                check_value(value)?;
                vec![Resolved::Value {
                    i: self.radius_index(geom)?,
                    value,
                }]
            }
            Constraint::Diameter { geom, value } => {
                check_value(value)?;
                vec![Resolved::Value {
                    i: self.radius_index(geom)?,
                    value: value / 2.0,
                }]
            }
            Constraint::Parallel(l1, l2) => vec![Resolved::Cross {
                a: self.line_points(l1)?,
                b: self.line_points(l2)?,
            }],
            Constraint::Perpendicular(l1, l2) => vec![Resolved::Dot {
                a: self.line_points(l1)?,
                b: self.line_points(l2)?,
            }],
            Constraint::Equal(g1, g2) => match (self.round_kind(g1)?, self.round_kind(g2)?) {
                (false, false) => vec![Resolved::LengthDiff {
                    a: self.line_points(g1)?,
                    b: self.line_points(g2)?,
                }],
                (true, true) => vec![equal(self.radius_index(g1)?, self.radius_index(g2)?)],
                _ => return Err(SketchError::KindMismatch(g2)),
            },
            Constraint::Midpoint { point, line } => {
                let m = self.point_index(point)?;
                let (s, e) = self.line_points(line)?;
                vec![
                    Resolved::Midpoint {
                        m: m.0,
                        a: s.0,
                        b: e.0,
                    },
                    Resolved::Midpoint {
                        m: m.1,
                        a: s.1,
                        b: e.1,
                    },
                ]
            }
            Constraint::Symmetric { a, b, line } => {
                let (a, b) = (self.point_index(a)?, self.point_index(b)?);
                let line = self.line_points(line)?;
                vec![
                    Resolved::MidOnLine { line, a, b },
                    Resolved::Dot { a: (a, b), b: line },
                ]
            }
            Constraint::Tangent(g1, g2) => match (self.round_kind(g1)?, self.round_kind(g2)?) {
                (false, true) => vec![self.line_tangent(g1, g2)?],
                (true, false) => vec![self.line_tangent(g2, g1)?],
                (true, true) => vec![self.circle_tangent(g1, g2)?],
                (false, false) => return Err(SketchError::KindMismatch(g2)),
            },
        })
    }

    /// `true` for a circle or arc, `false` for a line, and a kind mismatch
    /// for anything else.
    fn round_kind(&self, id: GeomId) -> Result<bool, SketchError> {
        match self.slot(id)?.kind {
            GeomKind::Line => Ok(false),
            GeomKind::Circle | GeomKind::Arc => Ok(true),
            _ => Err(SketchError::KindMismatch(id)),
        }
    }

    fn line_tangent(&self, line: GeomId, circle: GeomId) -> Result<Resolved, SketchError> {
        let line = self.line_points(line)?;
        let c = self.point_index(PointRef::Center(circle))?;
        let side = if cross(vec2(&self.params, line), vec2(&self.params, (line.0, c))) < 0.0 {
            -1.0
        } else {
            1.0
        };
        Ok(Resolved::LineTangent {
            line,
            c,
            r: self.radius_index(circle)?,
            side,
        })
    }

    fn circle_tangent(&self, g1: GeomId, g2: GeomId) -> Result<Resolved, SketchError> {
        let c1 = self.point_index(PointRef::Center(g1))?;
        let c2 = self.point_index(PointRef::Center(g2))?;
        let (r1, r2) = (self.radius_index(g1)?, self.radius_index(g2)?);
        let p = &self.params;
        let d = vec2(p, (c1, c2));
        let d = d.0.hypot(d.1);
        let external = (d - (p[r1] + p[r2])).abs() <= (d - (p[r1] - p[r2]).abs()).abs();
        let (k1, k2) = if external {
            (1.0, 1.0)
        } else if p[r1] >= p[r2] {
            (1.0, -1.0)
        } else {
            (-1.0, 1.0)
        };
        Ok(Resolved::CircleTangent {
            c1,
            c2,
            r1,
            r2,
            k1,
            k2,
        })
    }

    /// Adds a constraint after checking its references and values. Ids are
    /// assigned in order and never reused.
    pub fn add_constraint(&mut self, c: Constraint) -> Result<ConstraintId, SketchError> {
        let rows = self.resolve(&c)?;
        let id = ConstraintId(self.constraints.len() as u32);
        self.constraints.push(c);
        self.row_owners
            .resize(self.row_owners.len() + rows.len(), id);
        self.rows.extend(rows);
        Ok(id)
    }

    /// The constraint that produced residual row `row` (as indexed by
    /// [`Residuals::residuals`] and [`DofReport::redundant`]), or `None` if
    /// there is no such row.
    ///
    /// [`DofReport::redundant`]: crate::dof::DofReport::redundant
    pub fn constraint_of_row(&self, row: usize) -> Option<ConstraintId> {
        self.row_owners.get(row).copied()
    }

    /// The constraints, in the order they were added.
    pub fn constraints(&self) -> &[Constraint] {
        &self.constraints
    }

    /// Solves all constraints, updating the geometry in place. On a solver
    /// error the geometry is unchanged.
    pub fn solve(&mut self, options: SolveOptions) -> Result<SolveReport, SolveError> {
        let mut params = self.params.clone();
        let report = solver::solve(self, &mut params, options)?;
        self.params = params;
        Ok(report)
    }
}

/// The combined system of all constraints over the sketch parameter vector.
impl Residuals for Sketch {
    fn num_residuals(&self) -> usize {
        self.rows.len()
    }

    fn num_params(&self) -> usize {
        self.params.len()
    }

    fn residuals(&self, params: &[f64], out: &mut [f64]) {
        for (o, row) in out.iter_mut().zip(&self.rows) {
            *o = row.eval(params);
        }
    }

    fn jacobian(&self, params: &[f64], jac: &mut [f64]) -> bool {
        let n = params.len();
        jac.fill(0.0);
        for (i, row) in self.rows.iter().enumerate() {
            row.gradient(params, &mut jac[i * n..(i + 1) * n]);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arc, Circle, Geometry, Line, Point};

    const TOL: f64 = 1e-8;

    fn pt(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }

    fn line(s: &mut Sketch, x0: f64, y0: f64, x1: f64, y1: f64) -> GeomId {
        s.add(Geometry::Line(Line::new(pt(x0, y0), pt(x1, y1)).unwrap()))
            .unwrap()
    }

    fn point(s: &mut Sketch, x: f64, y: f64) -> GeomId {
        s.add(Geometry::Point(pt(x, y))).unwrap()
    }

    fn xy(s: &Sketch, p: PointRef) -> (f64, f64) {
        let (ix, iy) = s.point_index(p).unwrap();
        (s.params()[ix], s.params()[iy])
    }

    fn solved(s: &mut Sketch) {
        let report = s.solve(SolveOptions::default()).unwrap();
        assert!(report.converged, "{report:?}");
    }

    /// Checks the analytic Jacobian against a forward difference
    /// (`h = 1e-7·max(|x|, 1)`) at the sketch's current parameters.
    fn assert_jacobian_matches(s: &Sketch) {
        let (m, n) = (s.num_residuals(), s.num_params());
        assert!(m > 0);
        let x = s.params().to_vec();
        let mut jac = vec![f64::NAN; m * n];
        assert!(s.jacobian(&x, &mut jac));
        let mut r0 = vec![0.0; m];
        let mut r1 = vec![0.0; m];
        s.residuals(&x, &mut r0);
        for j in 0..n {
            let h = 1e-7 * x[j].abs().max(1.0);
            let mut xp = x.clone();
            xp[j] += h;
            s.residuals(&xp, &mut r1);
            for i in 0..m {
                let fd = (r1[i] - r0[i]) / h;
                let an = jac[i * n + j];
                assert!(
                    (fd - an).abs() <= 1e-5,
                    "J[{i}][{j}]: analytic {an}, fd {fd}"
                );
            }
        }
    }

    /// One sketch per constraint kind at a non-trivial point.
    fn each_constraint() -> Vec<Sketch> {
        let mut out = Vec::new();
        let two_points = |c: fn(PointRef, PointRef) -> Constraint| {
            let mut s = Sketch::new();
            let a = point(&mut s, 0.3, -1.7);
            let b = point(&mut s, 2.9, 0.4);
            s.add_constraint(c(PointRef::Point(a), PointRef::Point(b)))
                .unwrap();
            s
        };
        out.push(two_points(Constraint::Coincident));
        out.push(two_points(Constraint::HorizontalPoints));
        out.push(two_points(Constraint::VerticalPoints));
        out.push(two_points(|a, b| Constraint::Distance { a, b, value: 2.5 }));
        for c in [Constraint::Horizontal, Constraint::Vertical] {
            let mut s = Sketch::new();
            let l = line(&mut s, 0.5, 1.25, 3.0, 1.0);
            s.add_constraint(c(l)).unwrap();
            out.push(s);
        }
        let mut s = Sketch::new();
        let a = point(&mut s, 1.1, 0.7);
        s.add_constraint(Constraint::Fixed {
            point: PointRef::Point(a),
            x: 2.0,
            y: -1.0,
        })
        .unwrap();
        out.push(s);
        for diameter in [false, true] {
            let mut s = Sketch::new();
            let c = s
                .add(Geometry::Circle(Circle::new(pt(0.2, 0.9), 3.3).unwrap()))
                .unwrap();
            let constraint = if diameter {
                Constraint::Diameter {
                    geom: c,
                    value: 8.0,
                }
            } else {
                Constraint::Radius {
                    geom: c,
                    value: 4.0,
                }
            };
            s.add_constraint(constraint).unwrap();
            out.push(s);
        }
        let two_lines = |c: fn(GeomId, GeomId) -> Constraint| {
            let mut s = Sketch::new();
            let a = line(&mut s, 0.2, -0.4, 3.1, 0.9);
            let b = line(&mut s, -1.3, 2.2, 0.7, 4.6);
            s.add_constraint(c(a, b)).unwrap();
            s
        };
        out.push(two_lines(Constraint::Parallel));
        out.push(two_lines(Constraint::Perpendicular));
        out.push(two_lines(Constraint::Equal));
        let mut s = Sketch::new();
        let a = circle(&mut s, 0.1, 0.2, 1.3);
        let b = arc(&mut s, 2.0, -1.0, 2.6);
        s.add_constraint(Constraint::Equal(a, b)).unwrap();
        out.push(s);
        let mut s = Sketch::new();
        let p = point(&mut s, 1.7, 2.9);
        let l = line(&mut s, -0.5, 0.3, 4.1, 1.2);
        s.add_constraint(Constraint::Midpoint {
            point: PointRef::Point(p),
            line: l,
        })
        .unwrap();
        out.push(s);
        let mut s = Sketch::new();
        let a = point(&mut s, -1.2, 0.8);
        let b = point(&mut s, 2.6, 1.9);
        let l = line(&mut s, 0.4, -2.0, 1.1, 3.3);
        s.add_constraint(Constraint::Symmetric {
            a: PointRef::Point(a),
            b: PointRef::Point(b),
            line: l,
        })
        .unwrap();
        out.push(s);
        // Center left of the line (σ = +1) and right of it (σ = −1).
        for cy in [2.4, -2.4] {
            let mut s = Sketch::new();
            let l = line(&mut s, -0.6, 0.3, 3.8, -0.5);
            let c = arc(&mut s, 1.1, cy, 1.9);
            s.add_constraint(Constraint::Tangent(c, l)).unwrap();
            out.push(s);
        }
        // External, internal with r₁ > r₂, internal with r₁ < r₂.
        for (r1, r2) in [(1.5, 1.8), (4.0, 2.7), (2.7, 4.0)] {
            let mut s = Sketch::new();
            let a = circle(&mut s, 0.3, -0.2, r1);
            let b = arc(&mut s, 1.4, 0.9, r2);
            s.add_constraint(Constraint::Tangent(a, b)).unwrap();
            out.push(s);
        }
        out
    }

    fn circle(s: &mut Sketch, x: f64, y: f64, r: f64) -> GeomId {
        s.add(Geometry::Circle(Circle::new(pt(x, y), r).unwrap()))
            .unwrap()
    }

    fn arc(s: &mut Sketch, x: f64, y: f64, r: f64) -> GeomId {
        s.add(Geometry::Arc(Arc::new(pt(x, y), r, 0.3, 2.2).unwrap()))
            .unwrap()
    }

    /// Unit direction of a line.
    fn dir(s: &Sketch, l: GeomId) -> (f64, f64) {
        let (a, b) = (xy(s, PointRef::Start(l)), xy(s, PointRef::End(l)));
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let len = dx.hypot(dy);
        (dx / len, dy / len)
    }

    fn length(s: &Sketch, l: GeomId) -> f64 {
        let (a, b) = (xy(s, PointRef::Start(l)), xy(s, PointRef::End(l)));
        (b.0 - a.0).hypot(b.1 - a.1)
    }

    fn radius(s: &Sketch, g: GeomId) -> f64 {
        s.entity_params(g).unwrap()[2]
    }

    /// Signed distance from `p` to the line through `l`.
    fn line_distance(s: &Sketch, l: GeomId, p: (f64, f64)) -> f64 {
        let a = xy(s, PointRef::Start(l));
        let d = dir(s, l);
        d.0 * (p.1 - a.1) - d.1 * (p.0 - a.0)
    }

    fn is_horizontal(s: &Sketch, l: GeomId) -> bool {
        (xy(s, PointRef::Start(l)).1 - xy(s, PointRef::End(l)).1).abs() <= TOL
    }

    #[test]
    fn analytic_jacobian_matches_forward_difference() {
        let sketches = each_constraint();
        assert_eq!(sketches.len(), 20);
        for s in &sketches {
            assert_jacobian_matches(s);
        }
    }

    #[test]
    fn horizontal_line() {
        let mut s = Sketch::new();
        let l = line(&mut s, 0.0, 0.0, 3.0, 1.0);
        s.add_constraint(Constraint::Horizontal(l)).unwrap();
        solved(&mut s);
        let (a, b) = (xy(&s, PointRef::Start(l)), xy(&s, PointRef::End(l)));
        assert!((a.1 - b.1).abs() <= TOL, "{a:?} {b:?}");
    }

    #[test]
    fn vertical_line() {
        let mut s = Sketch::new();
        let l = line(&mut s, 0.0, 0.0, 1.0, 3.0);
        s.add_constraint(Constraint::Vertical(l)).unwrap();
        solved(&mut s);
        let (a, b) = (xy(&s, PointRef::Start(l)), xy(&s, PointRef::End(l)));
        assert!((a.0 - b.0).abs() <= TOL, "{a:?} {b:?}");
    }

    #[test]
    fn horizontal_and_vertical_points() {
        let mut s = Sketch::new();
        let a = point(&mut s, 0.0, 0.0);
        let b = point(&mut s, 3.0, 1.0);
        let c = point(&mut s, -2.0, 4.0);
        let (a, b, c) = (PointRef::Point(a), PointRef::Point(b), PointRef::Point(c));
        s.add_constraint(Constraint::HorizontalPoints(a, b))
            .unwrap();
        s.add_constraint(Constraint::VerticalPoints(a, c)).unwrap();
        solved(&mut s);
        assert!((xy(&s, a).1 - xy(&s, b).1).abs() <= TOL);
        assert!((xy(&s, a).0 - xy(&s, c).0).abs() <= TOL);
    }

    #[test]
    fn coincident_points() {
        let mut s = Sketch::new();
        let a = PointRef::Point(point(&mut s, 1.0, 2.0));
        let b = PointRef::Point(point(&mut s, -3.0, 0.5));
        s.add_constraint(Constraint::Coincident(a, b)).unwrap();
        solved(&mut s);
        let (pa, pb) = (xy(&s, a), xy(&s, b));
        assert!((pa.0 - pb.0).abs() <= TOL && (pa.1 - pb.1).abs() <= TOL);
    }

    #[test]
    fn fixed_point() {
        let mut s = Sketch::new();
        let a = PointRef::Point(point(&mut s, 5.0, 7.0));
        s.add_constraint(Constraint::Fixed {
            point: a,
            x: 2.0,
            y: -1.0,
        })
        .unwrap();
        solved(&mut s);
        let p = xy(&s, a);
        assert!(
            (p.0 - 2.0).abs() <= TOL && (p.1 + 1.0).abs() <= TOL,
            "{p:?}"
        );
    }

    #[test]
    fn distance_between_points() {
        let mut s = Sketch::new();
        let a = PointRef::Point(point(&mut s, 0.0, 0.0));
        let b = PointRef::Point(point(&mut s, 1.0, 2.0));
        s.add_constraint(Constraint::Distance { a, b, value: 5.0 })
            .unwrap();
        solved(&mut s);
        let (pa, pb) = (xy(&s, a), xy(&s, b));
        let d = (pb.0 - pa.0).hypot(pb.1 - pa.1);
        assert!((d - 5.0).abs() <= TOL, "{d}");
    }

    #[test]
    fn radius_and_diameter() {
        for (constraint, arc) in [(0, false), (1, false), (0, true), (1, true)] {
            let mut s = Sketch::new();
            let g = if arc {
                Geometry::Arc(Arc::new(pt(1.0, 1.0), 2.0, 0.0, 1.5).unwrap())
            } else {
                Geometry::Circle(Circle::new(pt(1.0, 1.0), 2.0).unwrap())
            };
            let id = s.add(g).unwrap();
            let c = if constraint == 0 {
                Constraint::Radius {
                    geom: id,
                    value: 4.0,
                }
            } else {
                Constraint::Diameter {
                    geom: id,
                    value: 8.0,
                }
            };
            s.add_constraint(c).unwrap();
            solved(&mut s);
            let r = s.entity_params(id).unwrap()[2];
            assert!((r - 4.0).abs() <= TOL, "{c:?}: r = {r}");
        }
    }

    // M2-04: each case starts from a perturbed guess and is combined with an
    // M2-03 constraint to show the two kinds compose.

    #[test]
    fn a_parallel_with_horizontal() {
        let mut s = Sketch::new();
        let l1 = line(&mut s, 0.0, 0.0, 4.0, 0.3);
        let l2 = line(&mut s, 1.0, 1.0, 4.5, 2.6);
        s.add_constraint(Constraint::Horizontal(l1)).unwrap();
        s.add_constraint(Constraint::Parallel(l1, l2)).unwrap();
        assert_jacobian_matches(&s);
        solved(&mut s);
        let (u, w) = (dir(&s, l1), dir(&s, l2));
        assert!((u.0 * w.1 - u.1 * w.0).abs() < TOL, "{u:?} {w:?}");
        assert!(is_horizontal(&s, l1) && is_horizontal(&s, l2));
    }

    #[test]
    fn b_perpendicular_with_horizontal() {
        let mut s = Sketch::new();
        let l1 = line(&mut s, 0.0, 0.0, 4.0, 0.3);
        let l2 = line(&mut s, 1.0, 1.0, 1.8, 4.0);
        s.add_constraint(Constraint::Horizontal(l1)).unwrap();
        s.add_constraint(Constraint::Perpendicular(l1, l2)).unwrap();
        solved(&mut s);
        let (u, w) = (dir(&s, l1), dir(&s, l2));
        assert!((u.0 * w.0 + u.1 * w.1).abs() < TOL, "{u:?} {w:?}");
        assert!(is_horizontal(&s, l1));
        let (a, b) = (xy(&s, PointRef::Start(l2)), xy(&s, PointRef::End(l2)));
        assert!((a.0 - b.0).abs() <= TOL, "l2 vertical: {a:?} {b:?}");
    }

    #[test]
    fn c_equal_length_with_distance() {
        let mut s = Sketch::new();
        let l1 = line(&mut s, 0.0, 0.0, 4.0, 1.0);
        let l2 = line(&mut s, 0.0, 2.0, 3.0, 3.0);
        s.add_constraint(Constraint::Distance {
            a: PointRef::Start(l1),
            b: PointRef::End(l1),
            value: 5.0,
        })
        .unwrap();
        s.add_constraint(Constraint::Equal(l1, l2)).unwrap();
        solved(&mut s);
        assert!((length(&s, l1) - 5.0).abs() <= TOL);
        assert!((length(&s, l2) - length(&s, l1)).abs() < TOL);
    }

    #[test]
    fn c_equal_radius_with_radius() {
        for (first_arc, second_arc) in [(false, false), (false, true), (true, true)] {
            let mut s = Sketch::new();
            let make = |s: &mut Sketch, is_arc: bool, x: f64, r: f64| {
                if is_arc {
                    arc(s, x, 0.0, r)
                } else {
                    circle(s, x, 0.0, r)
                }
            };
            let a = make(&mut s, first_arc, 0.0, 2.0);
            let b = make(&mut s, second_arc, 10.0, 3.0);
            s.add_constraint(Constraint::Radius {
                geom: a,
                value: 4.0,
            })
            .unwrap();
            s.add_constraint(Constraint::Equal(a, b)).unwrap();
            solved(&mut s);
            assert!((radius(&s, a) - 4.0).abs() <= TOL);
            assert!((radius(&s, b) - radius(&s, a)).abs() < TOL);
        }
    }

    #[test]
    fn d_midpoint_with_fixed() {
        let mut s = Sketch::new();
        let l = line(&mut s, 0.5, -0.5, 5.0, 2.5);
        let p = PointRef::Point(point(&mut s, 1.0, 3.0));
        for (end, x, y) in [(PointRef::Start(l), 0.0, 0.0), (PointRef::End(l), 6.0, 2.0)] {
            s.add_constraint(Constraint::Fixed { point: end, x, y })
                .unwrap();
        }
        s.add_constraint(Constraint::Midpoint { point: p, line: l })
            .unwrap();
        solved(&mut s);
        let m = xy(&s, p);
        assert!((m.0 - 3.0).abs() < TOL && (m.1 - 1.0).abs() < TOL, "{m:?}");
    }

    #[test]
    fn e_symmetric_with_vertical() {
        let mut s = Sketch::new();
        let l = line(&mut s, 2.0, -1.0, 2.3, 5.0);
        let a = PointRef::Point(point(&mut s, 0.0, 1.0));
        let b = PointRef::Point(point(&mut s, 3.5, 1.4));
        s.add_constraint(Constraint::Vertical(l)).unwrap();
        s.add_constraint(Constraint::Symmetric { a, b, line: l })
            .unwrap();
        solved(&mut s);
        // Mirror b about the line and compare with a.
        let (pa, pb) = (xy(&s, a), xy(&s, b));
        let s0 = xy(&s, PointRef::Start(l));
        let u = dir(&s, l);
        let t = (pb.0 - s0.0) * u.0 + (pb.1 - s0.1) * u.1;
        let foot = (s0.0 + t * u.0, s0.1 + t * u.1);
        let mirror = (2.0 * foot.0 - pb.0, 2.0 * foot.1 - pb.1);
        assert!(
            (mirror.0 - pa.0).abs() < TOL && (mirror.1 - pa.1).abs() < TOL,
            "{mirror:?} != {pa:?}"
        );
        assert!((s0.0 - xy(&s, PointRef::End(l)).0).abs() <= TOL);
    }

    #[test]
    fn f_line_arc_tangent_with_horizontal() {
        // Arc above the line, circle below it, and both argument orders.
        for (cy, use_arc, line_first) in [(2.5, true, true), (-2.5, false, false)] {
            let mut s = Sketch::new();
            let l = line(&mut s, -3.0, 0.0, 4.0, 0.4);
            let c = if use_arc {
                arc(&mut s, 1.0, cy, 2.0)
            } else {
                circle(&mut s, 1.0, cy, 2.0)
            };
            s.add_constraint(Constraint::Horizontal(l)).unwrap();
            let t = if line_first {
                Constraint::Tangent(l, c)
            } else {
                Constraint::Tangent(c, l)
            };
            s.add_constraint(t).unwrap();
            assert_jacobian_matches(&s);
            solved(&mut s);
            let center = xy(&s, PointRef::Center(c));
            let d = line_distance(&s, l, center);
            assert!((d.abs() - radius(&s, c)).abs() < TOL, "{d}");
            assert_eq!(d > 0.0, cy > 0.0, "stays on its side");
            assert!(is_horizontal(&s, l));
        }
    }

    #[test]
    fn g_arc_arc_tangent_with_fixed_and_radius() {
        // (r1, r2, start of c2): external, internal r1 > r2, internal r1 < r2.
        for (r1, r2, c2, internal) in [
            (2.0, 2.0, (4.5, 0.3), false),
            (5.0, 3.5, (1.2, 0.5), true),
            (3.5, 5.0, (1.2, 0.5), true),
        ] {
            let mut s = Sketch::new();
            let a = arc(&mut s, 0.0, 0.0, r1);
            let b = circle(&mut s, c2.0, c2.1, r2);
            for c in [
                Constraint::Fixed {
                    point: PointRef::Center(a),
                    x: 0.0,
                    y: 0.0,
                },
                Constraint::Radius { geom: a, value: r1 },
                Constraint::Tangent(a, b),
            ] {
                s.add_constraint(c).unwrap();
            }
            solved(&mut s);
            let (p, q) = (xy(&s, PointRef::Center(a)), xy(&s, PointRef::Center(b)));
            let d = (q.0 - p.0).hypot(q.1 - p.1);
            let (ra, rb) = (radius(&s, a), radius(&s, b));
            let want = if internal { (ra - rb).abs() } else { ra + rb };
            assert!((d - want).abs() < TOL, "d = {d}, want {want}");
            assert!((ra - r1).abs() <= TOL && p.0.abs() <= TOL && p.1.abs() <= TOL);
        }
    }

    /// Four lines `bottom, right, top, left`, each from its own endpoints,
    /// with coincident corners, horizontal/vertical sides, the bottom-left
    /// corner fixed at the origin, width 40 and height 25.
    fn rectangle(ends: [(f64, f64, f64, f64); 4]) -> (Sketch, [GeomId; 4]) {
        let mut s = Sketch::new();
        let lines = ends.map(|(x0, y0, x1, y1)| line(&mut s, x0, y0, x1, y1));
        for i in 0..4 {
            s.add_constraint(Constraint::Coincident(
                PointRef::End(lines[i]),
                PointRef::Start(lines[(i + 1) % 4]),
            ))
            .unwrap();
        }
        let [bottom, right, top, left] = lines;
        for c in [
            Constraint::Horizontal(bottom),
            Constraint::Horizontal(top),
            Constraint::Vertical(right),
            Constraint::Vertical(left),
            Constraint::Fixed {
                point: PointRef::Start(bottom),
                x: 0.0,
                y: 0.0,
            },
            Constraint::Distance {
                a: PointRef::Start(bottom),
                b: PointRef::End(bottom),
                value: 40.0,
            },
            Constraint::Distance {
                a: PointRef::Start(right),
                b: PointRef::End(right),
                value: 25.0,
            },
        ] {
            s.add_constraint(c).unwrap();
        }
        (s, lines)
    }

    #[test]
    fn rectangle_from_perturbed_guess() {
        // Every endpoint is off by at most 20% of the side lengths (8 in x,
        // 5 in y), and the line ends don't meet at the corners yet.
        let (mut s, lines) = rectangle([
            (2.0, -3.0, 44.0, 1.5),
            (43.0, -2.0, 37.0, 28.0),
            (38.0, 27.0, -4.0, 22.0),
            (-5.0, 24.0, 3.0, 4.0),
        ]);
        assert_jacobian_matches(&s);
        solved(&mut s);
        let expected = [(0.0, 0.0), (40.0, 0.0), (40.0, 25.0), (0.0, 25.0)];
        for (i, l) in lines.iter().enumerate() {
            for (p, e) in [
                (xy(&s, PointRef::Start(*l)), expected[i]),
                (xy(&s, PointRef::End(*l)), expected[(i + 1) % 4]),
            ] {
                assert!(
                    (p.0 - e.0).abs() <= TOL && (p.1 - e.1).abs() <= TOL,
                    "line {i}: {p:?} != {e:?}"
                );
            }
        }
        assert_eq!(s.constraints().len(), 11);
        assert_eq!(s.num_residuals(), 16);
    }

    #[test]
    fn invalid_constraints_are_errors() {
        let mut s = Sketch::new();
        let p = point(&mut s, 0.0, 0.0);
        let q = point(&mut s, 1.0, 1.0);
        let l = line(&mut s, 0.0, 0.0, 1.0, 0.0);
        let l2 = line(&mut s, 0.0, 1.0, 1.0, 1.0);
        let c = s
            .add(Geometry::Circle(Circle::new(pt(0.0, 0.0), 1.0).unwrap()))
            .unwrap();
        let (pp, pq) = (PointRef::Point(p), PointRef::Point(q));
        let unknown = GeomId(99);
        let cases = [
            (
                Constraint::Distance {
                    a: pp,
                    b: pq,
                    value: 0.0,
                },
                SketchError::NonPositiveValue(0.0),
            ),
            (
                Constraint::Distance {
                    a: pp,
                    b: pq,
                    value: -2.0,
                },
                SketchError::NonPositiveValue(-2.0),
            ),
            (
                Constraint::Radius {
                    geom: c,
                    value: -1.0,
                },
                SketchError::NonPositiveValue(-1.0),
            ),
            (
                Constraint::Diameter {
                    geom: c,
                    value: 0.0,
                },
                SketchError::NonPositiveValue(0.0),
            ),
            (
                Constraint::Radius {
                    geom: c,
                    value: f64::NAN,
                },
                SketchError::NonFinite,
            ),
            (
                Constraint::Fixed {
                    point: pp,
                    x: f64::INFINITY,
                    y: 0.0,
                },
                SketchError::NonFinite,
            ),
            (
                Constraint::Coincident(pp, PointRef::Point(unknown)),
                SketchError::UnknownGeom(unknown),
            ),
            (
                Constraint::Horizontal(unknown),
                SketchError::UnknownGeom(unknown),
            ),
            (
                Constraint::Radius {
                    geom: unknown,
                    value: 1.0,
                },
                SketchError::UnknownGeom(unknown),
            ),
            (Constraint::Vertical(p), SketchError::KindMismatch(p)),
            (
                Constraint::Radius {
                    geom: l,
                    value: 1.0,
                },
                SketchError::KindMismatch(l),
            ),
            (
                Constraint::Coincident(PointRef::Start(c), pp),
                SketchError::KindMismatch(c),
            ),
            (
                Constraint::Coincident(PointRef::Point(l), pp),
                SketchError::KindMismatch(l),
            ),
            (Constraint::Parallel(l, c), SketchError::KindMismatch(c)),
            (Constraint::Parallel(p, l), SketchError::KindMismatch(p)),
            (
                Constraint::Perpendicular(l, unknown),
                SketchError::UnknownGeom(unknown),
            ),
            (Constraint::Equal(l, c), SketchError::KindMismatch(c)),
            (Constraint::Equal(c, l), SketchError::KindMismatch(l)),
            (Constraint::Equal(p, l), SketchError::KindMismatch(p)),
            (
                Constraint::Midpoint { point: pp, line: c },
                SketchError::KindMismatch(c),
            ),
            (
                Constraint::Midpoint {
                    point: PointRef::Center(l),
                    line: l,
                },
                SketchError::KindMismatch(l),
            ),
            (
                Constraint::Symmetric {
                    a: pp,
                    b: pq,
                    line: p,
                },
                SketchError::KindMismatch(p),
            ),
            (Constraint::Tangent(l, l2), SketchError::KindMismatch(l2)),
            (Constraint::Tangent(p, c), SketchError::KindMismatch(p)),
            (Constraint::Tangent(c, q), SketchError::KindMismatch(q)),
            (
                Constraint::Tangent(c, unknown),
                SketchError::UnknownGeom(unknown),
            ),
        ];
        for (constraint, err) in cases {
            assert_eq!(s.add_constraint(constraint), Err(err), "{constraint:?}");
        }
        assert!(s.constraints().is_empty());
        assert_eq!(s.num_residuals(), 0);
    }
}
