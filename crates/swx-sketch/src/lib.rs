//! Saulitwerx 2D sketch geometry.
//!
//! A [`Sketch`] owns a flat parameter vector (`Vec<f64>`) that the constraint
//! solver optimizes over. Every entity records where its parameters live in
//! that vector:
//!
//! | Entity | Length | Layout |
//! |---|---|---|
//! | [`Point`] | 2 | x, y |
//! | [`Line`] | 4 | x0, y0, x1, y1 |
//! | [`Circle`] | 3 | cx, cy, r |
//! | [`Arc`] | 5 | cx, cy, r, a0, a1 (radians, counter-clockwise) |
//!
//! The [`solver`] module solves systems of residual equations over that
//! vector, the [`constraint`] module turns geometric constraints into such
//! equations ([`Sketch::add_constraint`], [`Sketch::solve`]), and the [`dof`]
//! module reports how many degrees of freedom such a system leaves and which
//! of its equations are redundant or conflicting. The [`profile`] module
//! turns closed loops of lines, arcs and circles into regions with holes.
//!
//! This crate must never depend on the geometry kernel (docs/ARCHITECTURE.md).

pub mod constraint;
pub mod dof;
pub mod profile;
pub mod solver;

pub use constraint::{Constraint, ConstraintId, PointRef};

use std::f64::consts::TAU;
use std::fmt;

/// Stable identifier of a geometry entity within a [`Sketch`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GeomId(pub u32);

/// Errors from building or editing sketch geometry.
#[derive(Debug, Clone, PartialEq)]
pub enum SketchError {
    /// A coordinate, radius or angle was NaN or infinite.
    NonFinite,
    /// A radius was zero or negative.
    NonPositiveRadius(f64),
    /// No entity with this id exists in the sketch.
    UnknownGeom(GeomId),
    /// The entity has the wrong kind for this use (e.g. a different kind
    /// than the one stored, or a constraint on an unsupported kind).
    KindMismatch(GeomId),
    /// A constraint value (distance, radius, diameter) was zero or negative.
    NonPositiveValue(f64),
}

impl fmt::Display for SketchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SketchError::NonFinite => write!(f, "geometry values must be finite"),
            SketchError::NonPositiveRadius(r) => write!(f, "radius must be > 0, got {r}"),
            SketchError::UnknownGeom(id) => write!(f, "unknown geometry {}", id.0),
            SketchError::KindMismatch(id) => write!(f, "geometry {} has a different kind", id.0),
            SketchError::NonPositiveValue(v) => write!(f, "value must be > 0, got {v}"),
        }
    }
}

impl std::error::Error for SketchError {}

fn check_finite(values: &[f64]) -> Result<(), SketchError> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(SketchError::NonFinite)
    }
}

fn check_radius(r: f64) -> Result<(), SketchError> {
    if r > 0.0 {
        Ok(())
    } else {
        Err(SketchError::NonPositiveRadius(r))
    }
}

/// A 2D point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Result<Self, SketchError> {
        check_finite(&[x, y])?;
        Ok(Point { x, y })
    }

    /// Euclidean distance to `other`.
    pub fn distance(&self, other: &Point) -> f64 {
        (other.x - self.x).hypot(other.y - self.y)
    }
}

/// A line segment between two endpoints.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Line {
    pub start: Point,
    pub end: Point,
}

impl Line {
    pub fn new(start: Point, end: Point) -> Result<Self, SketchError> {
        check_finite(&[start.x, start.y, end.x, end.y])?;
        Ok(Line { start, end })
    }

    pub fn length(&self) -> f64 {
        self.start.distance(&self.end)
    }
}

/// A full circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: Point,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point, radius: f64) -> Result<Self, SketchError> {
        check_finite(&[center.x, center.y, radius])?;
        check_radius(radius)?;
        Ok(Circle { center, radius })
    }

    pub fn circumference(&self) -> f64 {
        TAU * self.radius
    }
}

/// A circular arc running counter-clockwise from `start_angle` to `end_angle`
/// (radians). The constructor guarantees `end_angle > start_angle`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    pub center: Point,
    pub radius: f64,
    pub start_angle: f64,
    pub end_angle: f64,
}

impl Arc {
    /// Builds an arc. If `end_angle <= start_angle`, `end_angle` is
    /// normalized by adding 2π so the arc sweeps counter-clockwise.
    pub fn new(
        center: Point,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    ) -> Result<Self, SketchError> {
        check_finite(&[center.x, center.y, radius, start_angle, end_angle])?;
        check_radius(radius)?;
        let end_angle = if end_angle <= start_angle {
            end_angle + TAU
        } else {
            end_angle
        };
        Ok(Arc {
            center,
            radius,
            start_angle,
            end_angle,
        })
    }

    /// Swept angle in radians.
    pub fn sweep(&self) -> f64 {
        self.end_angle - self.start_angle
    }

    pub fn length(&self) -> f64 {
        self.radius * self.sweep()
    }

    fn point_at(&self, angle: f64) -> Point {
        Point {
            x: self.center.x + self.radius * angle.cos(),
            y: self.center.y + self.radius * angle.sin(),
        }
    }

    pub fn start_point(&self) -> Point {
        self.point_at(self.start_angle)
    }

    pub fn end_point(&self) -> Point {
        self.point_at(self.end_angle)
    }
}

/// Any sketch entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Geometry {
    Point(Point),
    Line(Line),
    Circle(Circle),
    Arc(Arc),
}

/// Entity kind, which fixes the parameter count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeomKind {
    Point,
    Line,
    Circle,
    Arc,
}

impl GeomKind {
    /// Number of parameters this kind occupies in the parameter vector.
    pub fn param_len(self) -> usize {
        match self {
            GeomKind::Point => 2,
            GeomKind::Line => 4,
            GeomKind::Circle => 3,
            GeomKind::Arc => 5,
        }
    }
}

impl Geometry {
    pub fn kind(&self) -> GeomKind {
        match self {
            Geometry::Point(_) => GeomKind::Point,
            Geometry::Line(_) => GeomKind::Line,
            Geometry::Circle(_) => GeomKind::Circle,
            Geometry::Arc(_) => GeomKind::Arc,
        }
    }

    /// Flattens the entity into its parameter layout (see crate docs).
    pub fn to_params(&self) -> Vec<f64> {
        match *self {
            Geometry::Point(p) => vec![p.x, p.y],
            Geometry::Line(l) => vec![l.start.x, l.start.y, l.end.x, l.end.y],
            Geometry::Circle(c) => vec![c.center.x, c.center.y, c.radius],
            Geometry::Arc(a) => vec![a.center.x, a.center.y, a.radius, a.start_angle, a.end_angle],
        }
    }

    /// Rebuilds an entity of `kind` from its parameters, validating them.
    pub fn from_params(kind: GeomKind, p: &[f64]) -> Result<Self, SketchError> {
        debug_assert_eq!(p.len(), kind.param_len());
        let pt = |x, y| Point::new(x, y);
        Ok(match kind {
            GeomKind::Point => Geometry::Point(pt(p[0], p[1])?),
            GeomKind::Line => Geometry::Line(Line::new(pt(p[0], p[1])?, pt(p[2], p[3])?)?),
            GeomKind::Circle => Geometry::Circle(Circle::new(pt(p[0], p[1])?, p[2])?),
            GeomKind::Arc => Geometry::Arc(Arc::new(pt(p[0], p[1])?, p[2], p[3], p[4])?),
        })
    }

    /// Validates a value that may have been built without its constructor.
    fn validate(&self) -> Result<Self, SketchError> {
        Geometry::from_params(self.kind(), &self.to_params())
    }
}

/// Where one entity's parameters live in the sketch parameter vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    pub kind: GeomKind,
    pub offset: usize,
    pub len: usize,
}

/// A 2D sketch: geometry entities over one flat parameter vector, plus the
/// constraints between them.
#[derive(Debug, Clone, Default)]
pub struct Sketch {
    params: Vec<f64>,
    slots: Vec<Slot>,
    constraints: Vec<Constraint>,
    rows: Vec<constraint::Resolved>,
    /// The constraint that produced each entry of `rows`.
    row_owners: Vec<ConstraintId>,
}

impl Sketch {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an entity, appending its parameters. Ids are assigned in order and
    /// never reused.
    pub fn add(&mut self, geom: Geometry) -> Result<GeomId, SketchError> {
        let geom = geom.validate()?;
        let id = GeomId(self.slots.len() as u32);
        let kind = geom.kind();
        self.slots.push(Slot {
            kind,
            offset: self.params.len(),
            len: kind.param_len(),
        });
        self.params.extend(geom.to_params());
        Ok(id)
    }

    /// Number of entities.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    /// The whole parameter vector (solver input).
    pub fn params(&self) -> &[f64] {
        &self.params
    }

    /// Location of an entity's parameters.
    pub fn slot(&self, id: GeomId) -> Result<Slot, SketchError> {
        self.slots
            .get(id.0 as usize)
            .copied()
            .ok_or(SketchError::UnknownGeom(id))
    }

    /// Raw parameters of one entity.
    pub fn entity_params(&self, id: GeomId) -> Result<&[f64], SketchError> {
        let s = self.slot(id)?;
        Ok(&self.params[s.offset..s.offset + s.len])
    }

    /// The entity as typed geometry.
    pub fn get(&self, id: GeomId) -> Result<Geometry, SketchError> {
        let s = self.slot(id)?;
        Geometry::from_params(s.kind, self.entity_params(id)?)
    }

    /// Replaces an entity's parameters. The kind must match.
    pub fn set(&mut self, id: GeomId, geom: Geometry) -> Result<(), SketchError> {
        let s = self.slot(id)?;
        if s.kind != geom.kind() {
            return Err(SketchError::KindMismatch(id));
        }
        let geom = geom.validate()?;
        self.params[s.offset..s.offset + s.len].copy_from_slice(&geom.to_params());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    const TOL: f64 = 1e-9;

    fn p(x: f64, y: f64) -> Point {
        Point::new(x, y).unwrap()
    }

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= TOL
    }

    #[test]
    fn layout_offsets_and_lengths() {
        let mut s = Sketch::new();
        let pt = s.add(Geometry::Point(p(1.0, 2.0))).unwrap();
        let ln = s
            .add(Geometry::Line(Line::new(p(0.0, 0.0), p(3.0, 4.0)).unwrap()))
            .unwrap();
        let ci = s
            .add(Geometry::Circle(Circle::new(p(5.0, 6.0), 2.0).unwrap()))
            .unwrap();
        let ar = s
            .add(Geometry::Arc(
                Arc::new(p(0.0, 0.0), 1.0, 0.0, FRAC_PI_2).unwrap(),
            ))
            .unwrap();
        assert_eq!(
            (pt, ln, ci, ar),
            (GeomId(0), GeomId(1), GeomId(2), GeomId(3))
        );
        let slots: Vec<_> = [pt, ln, ci, ar]
            .iter()
            .map(|&id| {
                let s = s.slot(id).unwrap();
                (s.offset, s.len)
            })
            .collect();
        assert_eq!(slots, vec![(0, 2), (2, 4), (6, 3), (9, 5)]);
        assert_eq!(s.params().len(), 14);
        assert_eq!(s.entity_params(ci).unwrap(), &[5.0, 6.0, 2.0]);
        assert_eq!(s.len(), 4);
    }

    #[test]
    fn set_then_get_round_trips_exactly() {
        let mut s = Sketch::new();
        let geoms = [
            Geometry::Point(p(0.1, -0.2)),
            Geometry::Line(Line::new(p(0.3, 0.7), p(-1.1, 2.9)).unwrap()),
            Geometry::Circle(Circle::new(p(1.0 / 3.0, 2.0 / 7.0), 0.123_456_789).unwrap()),
            Geometry::Arc(Arc::new(p(-4.0, 1e-7), 9.5, 0.25, 3.0).unwrap()),
        ];
        // Add a placeholder of each kind, then overwrite it through `set`.
        let ids: Vec<_> = geoms
            .iter()
            .map(|g| {
                let zeros = [0.0, 0.0, 1.0, 0.0, 1.0];
                let placeholder = Geometry::from_params(g.kind(), &zeros[..g.kind().param_len()]);
                s.add(placeholder.unwrap()).unwrap()
            })
            .collect();
        for (id, g) in ids.iter().zip(geoms.iter()) {
            s.set(*id, *g).unwrap();
            assert_eq!(s.get(*id).unwrap(), *g);
            assert_eq!(s.entity_params(*id).unwrap(), g.to_params().as_slice());
        }
    }

    #[test]
    fn set_rejects_kind_mismatch_and_unknown_id() {
        let mut s = Sketch::new();
        let id = s.add(Geometry::Point(p(0.0, 0.0))).unwrap();
        let c = Geometry::Circle(Circle::new(p(0.0, 0.0), 1.0).unwrap());
        assert_eq!(s.set(id, c), Err(SketchError::KindMismatch(id)));
        assert_eq!(s.get(GeomId(9)), Err(SketchError::UnknownGeom(GeomId(9))));
    }

    #[test]
    fn line_length() {
        let l = Line::new(p(0.0, 0.0), p(3.0, 4.0)).unwrap();
        assert!(close(l.length(), 5.0));
    }

    #[test]
    fn circle_circumference() {
        let c = Circle::new(p(0.0, 0.0), 2.0).unwrap();
        assert!(close(c.circumference(), 4.0 * PI));
    }

    #[test]
    fn quarter_arc_length_and_endpoints() {
        let a = Arc::new(p(0.0, 0.0), 1.0, 0.0, FRAC_PI_2).unwrap();
        assert!(close(a.length(), FRAC_PI_2));
        let (s, e) = (a.start_point(), a.end_point());
        assert!(close(s.x, 1.0) && close(s.y, 0.0));
        assert!(close(e.x, 0.0) && close(e.y, 1.0));
    }

    #[test]
    fn arc_end_not_after_start_is_normalized() {
        let a = Arc::new(p(0.0, 0.0), 1.0, FRAC_PI_2, 0.0).unwrap();
        assert!(close(a.end_angle, TAU));
        assert!(close(a.sweep(), 3.0 * FRAC_PI_2));
        let full = Arc::new(p(0.0, 0.0), 1.0, 1.0, 1.0).unwrap();
        assert!(close(full.sweep(), TAU));
    }

    #[test]
    fn invalid_input_is_an_error() {
        assert_eq!(Point::new(f64::NAN, 0.0), Err(SketchError::NonFinite));
        assert_eq!(Point::new(0.0, f64::INFINITY), Err(SketchError::NonFinite));
        assert_eq!(
            Circle::new(p(0.0, 0.0), 0.0),
            Err(SketchError::NonPositiveRadius(0.0))
        );
        assert_eq!(
            Circle::new(p(0.0, 0.0), -1.0),
            Err(SketchError::NonPositiveRadius(-1.0))
        );
        assert_eq!(
            Arc::new(p(0.0, 0.0), 1.0, 0.0, f64::NAN),
            Err(SketchError::NonFinite)
        );
        assert_eq!(
            Arc::new(p(0.0, 0.0), -2.0, 0.0, 1.0),
            Err(SketchError::NonPositiveRadius(-2.0))
        );
        // Values built without a constructor are still validated by the sketch.
        let mut s = Sketch::new();
        let bad = Geometry::Circle(Circle {
            center: p(0.0, 0.0),
            radius: -3.0,
        });
        assert_eq!(s.add(bad), Err(SketchError::NonPositiveRadius(-3.0)));
        let id = s.add(Geometry::Point(p(0.0, 0.0))).unwrap();
        let nan = Geometry::Point(Point {
            x: f64::NAN,
            y: 0.0,
        });
        assert_eq!(s.set(id, nan), Err(SketchError::NonFinite));
        assert!(!s.is_empty());
        assert_eq!(s.params(), &[0.0, 0.0]);
    }
}
