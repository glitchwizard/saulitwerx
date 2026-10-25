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
//!
//! Every row has an analytic Jacobian. All rows but distance are linear in
//! the parameters. Distance uses the unsquared Euclidean norm so its residual
//! is in length units like the others; its gradient `±(q − p)/‖q − p‖` is the
//! unit direction vector, taken as zero when the points coincide.

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
}

/// Stable identifier of a constraint within a [`Sketch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ConstraintId(pub u32);

/// Parameter indices `(x, y)` of one point in the sketch vector.
type Xy = (usize, usize);

/// A constraint reduced to parameter indices `(x, y)` of the sketch vector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Resolved {
    /// `params[b] − params[a] = 0` (one row).
    Equal { a: usize, b: usize },
    /// `params[i] − value = 0` (one row).
    Value { i: usize, value: f64 },
    /// `‖(xb, yb) − (xa, ya)‖ − value = 0` (one row).
    Distance { a: Xy, b: Xy, value: f64 },
}

impl Resolved {
    fn eval(&self, params: &[f64]) -> f64 {
        match *self {
            Resolved::Equal { a, b } => params[b] - params[a],
            Resolved::Value { i, value } => params[i] - value,
            Resolved::Distance { a, b, value } => {
                (params[b.0] - params[a.0]).hypot(params[b.1] - params[a.1]) - value
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
        out
    }

    #[test]
    fn analytic_jacobian_matches_forward_difference() {
        let sketches = each_constraint();
        assert_eq!(sketches.len(), 9);
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
        ];
        for (constraint, err) in cases {
            assert_eq!(s.add_constraint(constraint), Err(err), "{constraint:?}");
        }
        assert!(s.constraints().is_empty());
        assert_eq!(s.num_residuals(), 0);
    }
}
