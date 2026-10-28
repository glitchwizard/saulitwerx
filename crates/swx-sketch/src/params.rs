//! Sketch dimensions bound to document parameters.
//!
//! Every sketch dimension is a parameter (docs/specs/PRODUCT.md §2): binding a
//! `Distance`, `Radius` or `Diameter` constraint with [`bind_dimension`]
//! creates a model parameter `d0`, `d1`, … in the document's parameter table
//! and records which constraint it drives. [`apply_params`] copies the current
//! parameter values into the bound constraints; call [`Sketch::solve`]
//! afterwards to update the geometry. Unbound constraints are unaffected.

use std::fmt;

use swx_core::Document;

use crate::{Constraint, ConstraintId, Sketch, SketchError};

/// Errors from binding sketch dimensions to parameters.
#[derive(Debug, Clone, PartialEq)]
pub enum ParamError {
    /// The constraint was rejected by the sketch.
    Sketch(SketchError),
    /// Only `Distance`, `Radius` and `Diameter` constraints can be bound.
    NotADimension(Constraint),
    /// A binding names a parameter that is not in the document.
    UnknownParameter(String),
}

impl fmt::Display for ParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParamError::Sketch(e) => write!(f, "{e}"),
            ParamError::NotADimension(c) => {
                write!(
                    f,
                    "only distance, radius and diameter can be bound, got {c:?}"
                )
            }
            ParamError::UnknownParameter(p) => {
                write!(f, "a sketch dimension is bound to unknown parameter `{p}`")
            }
        }
    }
}

impl std::error::Error for ParamError {}

impl From<SketchError> for ParamError {
    fn from(e: SketchError) -> Self {
        ParamError::Sketch(e)
    }
}

impl Sketch {
    /// `(constraint, parameter name)` pairs, in the order they were bound.
    pub fn bindings(&self) -> &[(ConstraintId, String)] {
        &self.bindings
    }
}

/// Adds the dimension `c` to `sketch` and a model parameter holding its value
/// (unit `mm`) to `doc`, and binds the two. A diameter keeps its own value: a
/// diameter of 10 creates a parameter of 10. Returns the constraint id and the
/// parameter name. On error neither the sketch nor the document changes.
pub fn bind_dimension(
    sketch: &mut Sketch,
    doc: &mut Document,
    c: Constraint,
) -> Result<(ConstraintId, String), ParamError> {
    let value = match c {
        Constraint::Distance { value, .. }
        | Constraint::Radius { value, .. }
        | Constraint::Diameter { value, .. } => value,
        _ => return Err(ParamError::NotADimension(c)),
    };
    let id = sketch.add_constraint(c)?;
    let name = doc.add_model_param(value, "mm");
    sketch.bindings.push((id, name.clone()));
    Ok((id, name))
}

/// Copies each bound parameter's current value from `doc` into its
/// constraint. Every value is checked first, so on error nothing changes.
pub fn apply_params(sketch: &mut Sketch, doc: &Document) -> Result<(), ParamError> {
    let mut values = Vec::with_capacity(sketch.bindings.len());
    for (id, name) in &sketch.bindings {
        let v = doc
            .param(name)
            .ok_or_else(|| ParamError::UnknownParameter(name.clone()))?;
        if !v.is_finite() {
            return Err(SketchError::NonFinite.into());
        }
        if v <= 0.0 {
            return Err(SketchError::NonPositiveValue(v).into());
        }
        values.push((*id, v));
    }
    for (id, v) in values {
        sketch.set_dimension_value(id, v)?;
    }
    Ok(())
}

/// Points every binding to parameter `old` at `new` instead, after the
/// parameter was renamed in the document. Returns how many bindings changed.
pub fn rename_bound(sketch: &mut Sketch, old: &str, new: &str) -> usize {
    let mut n = 0;
    for (_, name) in &mut sketch.bindings {
        if name == old {
            *name = new.to_owned();
            n += 1;
        }
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::SolveOptions;
    use crate::{Circle, GeomId, Geometry, Line, Point, PointRef};

    const TOL: f64 = 1e-8;

    fn line(s: &mut Sketch, x0: f64, y0: f64, x1: f64, y1: f64) -> GeomId {
        let p = |x, y| Point::new(x, y).unwrap();
        s.add(Geometry::Line(Line::new(p(x0, y0), p(x1, y1)).unwrap()))
            .unwrap()
    }

    fn distance(l: GeomId, value: f64) -> Constraint {
        Constraint::Distance {
            a: PointRef::Start(l),
            b: PointRef::End(l),
            value,
        }
    }

    /// Rectangle `bottom, right, top, left` with the bottom-left corner fixed
    /// at the origin; width (bottom) and height (right) bound as `d0`, `d1`.
    fn rectangle(doc: &mut Document) -> (Sketch, [GeomId; 4]) {
        let mut s = Sketch::new();
        let lines = [
            line(&mut s, 0.0, 0.0, 40.0, 0.0),
            line(&mut s, 40.0, 0.0, 40.0, 25.0),
            line(&mut s, 40.0, 25.0, 0.0, 25.0),
            line(&mut s, 0.0, 25.0, 0.0, 0.0),
        ];
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
        ] {
            s.add_constraint(c).unwrap();
        }
        let (_, w) = bind_dimension(&mut s, doc, distance(bottom, 40.0)).unwrap();
        let (_, h) = bind_dimension(&mut s, doc, distance(right, 25.0)).unwrap();
        assert_eq!((w.as_str(), h.as_str()), ("d0", "d1"));
        (s, lines)
    }

    fn size(s: &Sketch, lines: [GeomId; 4]) -> (f64, f64) {
        let len = |l| match s.get(l).unwrap() {
            Geometry::Line(l) => l.length(),
            _ => unreachable!(),
        };
        (len(lines[0]), len(lines[1]))
    }

    fn solve(s: &mut Sketch) {
        let rep = s.solve(SolveOptions::default()).unwrap();
        assert!(rep.converged, "{rep:?}");
    }

    #[test]
    fn binding_names_parameters_in_order() {
        let mut doc = Document::new();
        let mut s = Sketch::new();
        let l = line(&mut s, 0.0, 0.0, 3.0, 4.0);
        let c = s
            .add(Geometry::Circle(
                Circle::new(Point::new(9.0, 9.0).unwrap(), 2.0).unwrap(),
            ))
            .unwrap();
        let dims = [
            distance(l, 5.0),
            Constraint::Radius {
                geom: c,
                value: 2.0,
            },
            Constraint::Diameter {
                geom: c,
                value: 4.0,
            },
        ];
        let mut got = Vec::new();
        for d in dims {
            got.push(bind_dimension(&mut s, &mut doc, d).unwrap());
        }
        let names: Vec<_> = got.iter().map(|(_, n)| n.as_str()).collect();
        assert_eq!(names, ["d0", "d1", "d2"]);
        assert_eq!(doc.param("d0"), Some(5.0));
        assert_eq!(doc.param("d1"), Some(2.0));
        assert_eq!(doc.param("d2"), Some(4.0));
        assert!(doc.parameters().iter().all(|p| p.unit == "mm"));
        let ids: Vec<_> = got.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, [ConstraintId(0), ConstraintId(1), ConstraintId(2)]);
        assert_eq!(s.bindings(), got.as_slice());
        assert_eq!(s.constraints(), &dims);
    }

    #[test]
    fn non_dimensions_and_invalid_dimensions_add_nothing() {
        let mut doc = Document::new();
        let mut s = Sketch::new();
        let l = line(&mut s, 0.0, 0.0, 1.0, 0.0);
        let h = Constraint::Horizontal(l);
        assert_eq!(
            bind_dimension(&mut s, &mut doc, h),
            Err(ParamError::NotADimension(h))
        );
        assert_eq!(
            bind_dimension(&mut s, &mut doc, distance(l, -1.0)),
            Err(ParamError::Sketch(SketchError::NonPositiveValue(-1.0)))
        );
        // A radius on a line is rejected by the sketch.
        let r = Constraint::Radius {
            geom: l,
            value: 1.0,
        };
        assert_eq!(
            bind_dimension(&mut s, &mut doc, r),
            Err(ParamError::Sketch(SketchError::KindMismatch(l)))
        );
        assert!(s.constraints().is_empty() && s.bindings().is_empty());
        assert!(doc.parameters().is_empty());
    }

    #[test]
    fn editing_a_parameter_drives_the_width() {
        let mut doc = Document::new();
        let (mut s, lines) = rectangle(&mut doc);
        solve(&mut s);
        doc.set_param("d0", 60.0);
        apply_params(&mut s, &doc).unwrap();
        solve(&mut s);
        let (w, h) = size(&s, lines);
        assert!((w - 60.0).abs() <= TOL, "width {w}");
        assert!((h - 25.0).abs() <= TOL, "height {h}");
    }

    #[test]
    fn renamed_parameter_keeps_driving_the_width() {
        let mut doc = Document::new();
        let (mut s, lines) = rectangle(&mut doc);
        doc.rename_param("d0", "width").unwrap();
        assert_eq!(rename_bound(&mut s, "d0", "width"), 1);
        assert_eq!(s.bindings()[0].1, "width");
        doc.set_param("width", 50.0);
        apply_params(&mut s, &doc).unwrap();
        solve(&mut s);
        let (w, h) = size(&s, lines);
        assert!((w - 50.0).abs() <= TOL, "width {w}");
        assert!((h - 25.0).abs() <= TOL, "height {h}");
    }

    #[test]
    fn apply_with_missing_or_invalid_parameter_changes_nothing() {
        let mut doc = Document::new();
        let (mut s, _) = rectangle(&mut doc);
        doc.set_param("d1", 30.0);
        doc.rename_param("d0", "width").unwrap();
        let before = s.constraints().to_vec();
        let err = apply_params(&mut s, &doc).unwrap_err();
        assert_eq!(err, ParamError::UnknownParameter("d0".into()));
        assert!(err.to_string().contains("`d0`"));
        assert_eq!(s.constraints(), before.as_slice());

        rename_bound(&mut s, "d0", "width");
        doc.set_param("width", 0.0);
        assert_eq!(
            apply_params(&mut s, &doc),
            Err(ParamError::Sketch(SketchError::NonPositiveValue(0.0)))
        );
        assert_eq!(s.constraints(), before.as_slice());
    }

    #[test]
    fn set_dimension_value_error_table() {
        let mut s = Sketch::new();
        let l = line(&mut s, 0.0, 0.0, 1.0, 0.0);
        let d = s.add_constraint(distance(l, 1.0)).unwrap();
        let h = s.add_constraint(Constraint::Horizontal(l)).unwrap();
        let unknown = ConstraintId(9);
        for (id, v, want) in [
            (unknown, 2.0, SketchError::UnknownConstraint(unknown)),
            (h, 2.0, SketchError::NotADimension(h)),
            (d, 0.0, SketchError::NonPositiveValue(0.0)),
            (d, -3.0, SketchError::NonPositiveValue(-3.0)),
            (d, f64::NAN, SketchError::NonFinite),
            (d, f64::INFINITY, SketchError::NonFinite),
        ] {
            assert_eq!(s.set_dimension_value(id, v), Err(want));
        }
        assert_eq!(s.constraints()[0], distance(l, 1.0));
        s.set_dimension_value(d, 2.5).unwrap();
        assert_eq!(s.constraints()[0], distance(l, 2.5));
        solve(&mut s);
        let Geometry::Line(got) = s.get(l).unwrap() else {
            unreachable!()
        };
        assert!((got.length() - 2.5).abs() <= TOL);
    }

    #[test]
    fn set_dimension_value_on_radius_and_diameter() {
        let mut s = Sketch::new();
        let c = s
            .add(Geometry::Circle(
                Circle::new(Point::new(0.0, 0.0).unwrap(), 1.0).unwrap(),
            ))
            .unwrap();
        let r = s
            .add_constraint(Constraint::Radius {
                geom: c,
                value: 1.0,
            })
            .unwrap();
        s.set_dimension_value(r, 3.0).unwrap();
        solve(&mut s);
        assert!((s.entity_params(c).unwrap()[2] - 3.0).abs() <= TOL);

        let mut s2 = Sketch::new();
        let c2 = s2
            .add(Geometry::Circle(
                Circle::new(Point::new(0.0, 0.0).unwrap(), 1.0).unwrap(),
            ))
            .unwrap();
        let dm = s2
            .add_constraint(Constraint::Diameter {
                geom: c2,
                value: 2.0,
            })
            .unwrap();
        s2.set_dimension_value(dm, 10.0).unwrap();
        solve(&mut s2);
        assert!((s2.entity_params(c2).unwrap()[2] - 5.0).abs() <= TOL);
    }
}
