//! Part origin and work-feature geometry: work planes, axes and points.
//!
//! Pure functions on [`Workplane`], [`Axis`] and [`Point3`], with no backend
//! calls, so they run without the `occt` feature. The vector algebra (cross
//! products, projection onto a plane, rotation about an axis, plane–plane and
//! line–plane intersection) follows the standard closed forms in Schneider and
//! Eberly, *Geometric Tools for Computer Graphics* (Morgan Kaufmann, 2003).

use crate::{Axis, KernelError, Point3, Workplane};

/// The part origin, the point the three origin planes and axes pass through.
pub const ORIGIN: Point3 = [0.0; 3];

/// Tolerance of the work-feature checks, on unit vectors: two directions
/// count as parallel (and three points as collinear) when the cross product
/// of their unit vectors is at most this long; a direction counts as lying
/// in a plane when its dot product with the unit normal is at most this.
/// Points closer than this (in model units) count as coincident, and an axis
/// point counts as on a plane within this distance.
pub const DATUM_TOLERANCE: f64 = 1e-9;

impl Axis {
    /// The origin X axis: through [`ORIGIN`] along +X.
    pub const X: Axis = Axis {
        origin: ORIGIN,
        dir: [1.0, 0.0, 0.0],
    };
    /// The origin Y axis: through [`ORIGIN`] along +Y.
    pub const Y: Axis = Axis {
        origin: ORIGIN,
        dir: [0.0, 1.0, 0.0],
    };
    /// The origin Z axis: through [`ORIGIN`] along +Z.
    pub const Z: Axis = Axis {
        origin: ORIGIN,
        dir: [0.0, 0.0, 1.0],
    };

    /// The work axis through `p` along `q − p` (normalized).
    ///
    /// Non-finite coordinates and points closer than [`DATUM_TOLERANCE`] are
    /// [`KernelError::InvalidInput`].
    pub fn through_points(p: Point3, q: Point3) -> Result<Axis, KernelError> {
        finite("axis point", &p)?;
        finite("axis point", &q)?;
        let dir = unit("axis direction (the points coincide)", sub(q, p))?;
        Ok(Axis { origin: p, dir })
    }

    /// The line where planes `a` and `b` meet: `dir` along
    /// `a.normal × b.normal` (normalized), `origin` the point of the line
    /// closest to [`ORIGIN`].
    ///
    /// Parallel planes (unit normals whose cross product is at most
    /// [`DATUM_TOLERANCE`] long) and invalid planes are
    /// [`KernelError::InvalidInput`].
    pub fn plane_intersection(a: &Workplane, b: &Workplane) -> Result<Axis, KernelError> {
        let (oa, _, na) = frame(a)?;
        let (ob, _, nb) = frame(b)?;
        let d = cross(na, nb);
        let len2 = dot(d, d);
        if len2.sqrt() <= DATUM_TOLERANCE {
            return Err(invalid(
                "the planes are parallel: they have no line in common",
            ));
        }
        // The point on both planes in the span of the normals, which is the
        // one closest to the origin: (h_a (n_b × d) + h_b (d × n_a)) / |d|².
        let (ha, hb) = (dot(na, oa), dot(nb, ob));
        let p = add(scale(cross(nb, d), ha), scale(cross(d, na), hb));
        Ok(Axis {
            origin: scale(p, 1.0 / len2),
            dir: scale(d, 1.0 / len2.sqrt()),
        })
    }
}

impl Workplane {
    /// This plane moved `distance` along its normal; `x_dir` and `normal`
    /// are kept (normalized).
    ///
    /// A non-finite `distance` and an invalid plane are
    /// [`KernelError::InvalidInput`].
    pub fn offset(&self, distance: f64) -> Result<Workplane, KernelError> {
        finite("offset distance", &[distance])?;
        let (o, x, n) = frame(self)?;
        Ok(Workplane {
            origin: add(o, scale(n, distance)),
            x_dir: x,
            normal: n,
        })
    }

    /// The plane through three points: origin `p0`, `x_dir` along `p1 − p0`,
    /// normal along `(p1 − p0) × (p2 − p0)`.
    ///
    /// Non-finite coordinates, coincident points and collinear points (the
    /// cross product of the two unit directions is at most
    /// [`DATUM_TOLERANCE`] long) are [`KernelError::InvalidInput`].
    pub fn through_points(p0: Point3, p1: Point3, p2: Point3) -> Result<Workplane, KernelError> {
        for p in [p0, p1, p2] {
            finite("plane point", &p)?;
        }
        let u = unit("plane points (two coincide)", sub(p1, p0))?;
        let v = unit("plane points (two coincide)", sub(p2, p0))?;
        let n = cross(u, v);
        if norm(n) <= DATUM_TOLERANCE {
            return Err(invalid("the three plane points are collinear"));
        }
        Ok(Workplane {
            origin: p0,
            x_dir: u,
            normal: unit("plane normal", n)?,
        })
    }

    /// The plane through `p` with normal `n` (normalized). `x_dir` is +X
    /// projected onto the plane, or +Y when `n` is parallel to X within
    /// [`DATUM_TOLERANCE`].
    ///
    /// Non-finite coordinates and a zero normal are
    /// [`KernelError::InvalidInput`].
    pub fn from_point_normal(p: Point3, n: [f64; 3]) -> Result<Workplane, KernelError> {
        finite("plane point", &p)?;
        let n = unit("plane normal", n)?;
        let e = if norm(cross(n, [1.0, 0.0, 0.0])) <= DATUM_TOLERANCE {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let x = unit("plane x direction", sub(e, scale(n, dot(e, n))))?;
        Ok(Workplane {
            origin: p,
            x_dir: x,
            normal: n,
        })
    }

    /// This plane turned `angle` radians about `axis` (right-hand rule about
    /// `axis.dir`), which must lie in the plane: its direction perpendicular
    /// to the normal and its origin on the plane, each within
    /// [`DATUM_TOLERANCE`]. Rotation by Rodrigues' formula.
    ///
    /// A non-finite angle, an invalid plane or axis, and an axis not in the
    /// plane are [`KernelError::InvalidInput`].
    pub fn angled(&self, axis: Axis, angle: f64) -> Result<Workplane, KernelError> {
        finite("plane angle", &[angle])?;
        let (o, x, n) = frame(self)?;
        finite("axis origin", &axis.origin)?;
        let d = unit("axis direction", axis.dir)?;
        if dot(d, n).abs() > DATUM_TOLERANCE || dot(sub(axis.origin, o), n).abs() > DATUM_TOLERANCE
        {
            return Err(invalid("the axis does not lie in the plane"));
        }
        let rotate = |v: [f64; 3]| {
            let (s, c) = angle.sin_cos();
            add(
                add(scale(v, c), scale(cross(d, v), s)),
                scale(d, dot(d, v) * (1.0 - c)),
            )
        };
        Ok(Workplane {
            origin: add(axis.origin, rotate(sub(o, axis.origin))),
            x_dir: rotate(x),
            normal: rotate(n),
        })
    }
}

/// The work point where `axis` crosses `plane`.
///
/// An axis parallel to the plane (`|axis.dir · normal|` at most
/// [`DATUM_TOLERANCE`] on unit vectors), non-finite inputs and an invalid
/// plane are [`KernelError::InvalidInput`].
pub fn plane_axis_intersection(plane: &Workplane, axis: &Axis) -> Result<Point3, KernelError> {
    let (o, _, n) = frame(plane)?;
    finite("axis origin", &axis.origin)?;
    let d = unit("axis direction", axis.dir)?;
    let denom = dot(n, d);
    if denom.abs() <= DATUM_TOLERANCE {
        return Err(invalid("the axis is parallel to the plane"));
    }
    let t = dot(n, sub(o, axis.origin)) / denom;
    Ok(add(axis.origin, scale(d, t)))
}

/// The plane through `point` with normal `normal` (normalized), placed with
/// its origin at the orthogonal projection of [`ORIGIN`] onto it,
/// `(point · n) n`, and `x_dir` as in [`Workplane::from_point_normal`].
///
/// Non-finite inputs and a zero normal are [`KernelError::InvalidInput`].
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn plane_nearest_origin(
    point: Point3,
    normal: [f64; 3],
) -> Result<Workplane, KernelError> {
    finite("plane point", &point)?;
    let n = unit("plane normal", normal)?;
    Workplane::from_point_normal(scale(n, dot(point, n)), n)
}

/// The line through `point` along `dir`, as an [`Axis`] with a unit `dir`
/// and its origin at the line's point closest to [`ORIGIN`],
/// `point − (point · d) d`.
///
/// Non-finite inputs and a zero direction are [`KernelError::InvalidInput`].
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn axis_nearest_origin(point: Point3, dir: [f64; 3]) -> Result<Axis, KernelError> {
    finite("axis point", &point)?;
    let d = unit("axis direction", dir)?;
    Ok(Axis {
        origin: sub(point, scale(d, dot(point, d))),
        dir: d,
    })
}

/// A checked plane: finite origin, unit normal and a unit `x_dir`
/// perpendicular to it (the given `x_dir` with any normal component removed).
fn frame(p: &Workplane) -> Result<(Point3, [f64; 3], [f64; 3]), KernelError> {
    finite("plane origin", &p.origin)?;
    let n = unit("plane normal", p.normal)?;
    let x = unit("plane x direction", p.x_dir)?;
    let x = unit(
        "plane x direction (parallel to the normal)",
        sub(x, scale(n, dot(x, n))),
    )?;
    Ok((p.origin, x, n))
}

fn invalid(msg: &str) -> KernelError {
    KernelError::InvalidInput(msg.to_string())
}

fn finite(name: &str, values: &[f64]) -> Result<(), KernelError> {
    if values.iter().all(|v| v.is_finite()) {
        Ok(())
    } else {
        Err(KernelError::InvalidInput(format!(
            "{name} must be finite, got {values:?}"
        )))
    }
}

/// `v` normalized; non-finite or shorter than [`DATUM_TOLERANCE`] is an error.
fn unit(name: &str, v: [f64; 3]) -> Result<[f64; 3], KernelError> {
    finite(name, &v)?;
    let len = norm(v);
    if len <= DATUM_TOLERANCE {
        return Err(KernelError::InvalidInput(format!(
            "{name} must be non-zero"
        )));
    }
    Ok(scale(v, 1.0 / len))
}

fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f64; 3], s: f64) -> [f64; 3] {
    a.map(|c| c * s)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::FRAC_PI_2;

    fn assert_vec(actual: [f64; 3], expected: [f64; 3]) {
        for i in 0..3 {
            assert!(
                (actual[i] - expected[i]).abs() <= 1e-9,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    fn assert_plane(p: &Workplane, origin: Point3, x_dir: [f64; 3], normal: [f64; 3]) {
        assert_vec(p.origin, origin);
        assert_vec(p.x_dir, x_dir);
        assert_vec(p.normal, normal);
    }

    fn assert_invalid<T: std::fmt::Debug>(r: Result<T, KernelError>) {
        assert!(matches!(r, Err(KernelError::InvalidInput(_))), "{r:?}");
    }

    #[test]
    fn origin_constants() {
        assert_eq!(ORIGIN, [0.0; 3]);
        for (a, d) in [
            (Axis::X, [1.0, 0.0, 0.0]),
            (Axis::Y, [0.0, 1.0, 0.0]),
            (Axis::Z, [0.0, 0.0, 1.0]),
        ] {
            assert_eq!(a.origin, ORIGIN);
            assert_eq!(a.dir, d);
        }
    }

    #[test]
    fn offset_moves_along_the_normal() {
        let p = Workplane::XY.offset(5.0).unwrap();
        assert_plane(&p, [0.0, 0.0, 5.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let p = Workplane::XY.offset(-2.5).unwrap();
        assert_vec(p.origin, [0.0, 0.0, -2.5]);
        let p = Workplane::XZ.offset(1.0).unwrap();
        assert_vec(p.origin, [0.0, -1.0, 0.0]);
    }

    #[test]
    fn plane_through_three_points() {
        let p = Workplane::through_points([0.0; 3], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]).unwrap();
        assert_plane(
            &p,
            Workplane::XY.origin,
            Workplane::XY.x_dir,
            Workplane::XY.normal,
        );
        let p =
            Workplane::through_points([0.0, 0.0, 3.0], [1.0, 0.0, 3.0], [0.0, 0.0, 4.0]).unwrap();
        assert_plane(&p, [0.0, 0.0, 3.0], [1.0, 0.0, 0.0], [0.0, -1.0, 0.0]);
    }

    #[test]
    fn plane_from_point_and_normal() {
        let p = Workplane::from_point_normal([1.0, 2.0, 3.0], [0.0, 0.0, 2.0]).unwrap();
        assert_plane(&p, [1.0, 2.0, 3.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        let p = Workplane::from_point_normal([1.0, 2.0, 3.0], [5.0, 0.0, 0.0]).unwrap();
        assert_plane(&p, [1.0, 2.0, 3.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        // A tilted normal: x_dir is +X projected onto the plane.
        let s = 0.5f64.sqrt();
        let p = Workplane::from_point_normal([0.0; 3], [1.0, 0.0, 1.0]).unwrap();
        assert_plane(&p, [0.0; 3], [s, 0.0, -s], [s, 0.0, s]);
    }

    #[test]
    fn angled_plane_turns_about_an_axis_in_it() {
        let p = Workplane::XY.angled(Axis::X, FRAC_PI_2).unwrap();
        assert_vec(p.normal, [0.0, -1.0, 0.0]);
        // Contains the X axis: its origin is on it and X is perpendicular to the normal.
        assert!(dot(sub(Axis::X.origin, p.origin), p.normal).abs() <= 1e-9);
        assert!(dot(Axis::X.dir, p.normal).abs() <= 1e-9);
        assert!(dot(p.x_dir, p.normal).abs() <= 1e-9);
        // An axis away from the plane origin: the plane z = 2 about the line
        // y = 4, z = 2 along +X keeps that line.
        let base = Workplane::XY.offset(2.0).unwrap();
        let hinge = Axis {
            origin: [0.0, 4.0, 2.0],
            dir: [1.0, 0.0, 0.0],
        };
        let p = base.angled(hinge, FRAC_PI_2).unwrap();
        assert_vec(p.normal, [0.0, -1.0, 0.0]);
        assert_vec(p.origin, [0.0, 4.0, -2.0]);
        assert!(dot(sub(hinge.origin, p.origin), p.normal).abs() <= 1e-9);
    }

    #[test]
    fn axis_through_points_and_plane_intersection() {
        let a = Axis::through_points([1.0, 1.0, 0.0], [2.0, 2.0, 1.0]).unwrap();
        let s = 3f64.sqrt().recip();
        assert_vec(a.origin, [1.0, 1.0, 0.0]);
        assert_vec(a.dir, [s, s, s]);
        let a = Axis::plane_intersection(
            &Workplane::XY.offset(4.0).unwrap(),
            &Workplane::YZ.offset(3.0).unwrap(),
        )
        .unwrap();
        assert_vec(a.origin, [3.0, 0.0, 4.0]);
        assert!((a.dir[1].abs() - 1.0).abs() <= 1e-9 && a.dir[0] == 0.0 && a.dir[2] == 0.0);
    }

    #[test]
    fn work_point_where_an_axis_crosses_a_plane() {
        let p = plane_axis_intersection(
            &Workplane::XY.offset(7.0).unwrap(),
            &Axis::through_points([1.0, 1.0, 0.0], [2.0, 2.0, 1.0]).unwrap(),
        )
        .unwrap();
        assert_vec(p, [8.0, 8.0, 7.0]);
    }

    #[test]
    fn bad_inputs_are_invalid_input() {
        let nan = f64::NAN;
        let inf = f64::INFINITY;
        // Non-finite inputs.
        assert_invalid(Workplane::XY.offset(nan));
        assert_invalid(Workplane::XY.offset(inf));
        assert_invalid(Workplane::through_points(
            [nan, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
        ));
        assert_invalid(Workplane::from_point_normal(
            [0.0, inf, 0.0],
            [0.0, 0.0, 1.0],
        ));
        assert_invalid(Workplane::from_point_normal([0.0; 3], [0.0, 0.0, nan]));
        assert_invalid(Workplane::XY.angled(Axis::X, nan));
        assert_invalid(Axis::through_points([0.0; 3], [inf, 0.0, 0.0]));
        let bad_origin = Workplane {
            origin: [nan, 0.0, 0.0],
            ..Workplane::XY
        };
        assert_invalid(bad_origin.offset(1.0));
        assert_invalid(plane_axis_intersection(
            &Workplane::XY,
            &Axis {
                origin: [nan, 0.0, 0.0],
                dir: [0.0, 0.0, 1.0],
            },
        ));
        // Coincident or collinear points.
        assert_invalid(Axis::through_points([1.0, 2.0, 3.0], [1.0, 2.0, 3.0]));
        assert_invalid(Workplane::through_points(
            [0.0; 3],
            [0.0; 3],
            [0.0, 1.0, 0.0],
        ));
        assert_invalid(Workplane::through_points(
            [0.0; 3],
            [1.0, 0.0, 0.0],
            [0.0; 3],
        ));
        assert_invalid(Workplane::through_points(
            [0.0; 3],
            [1.0, 1.0, 1.0],
            [3.0, 3.0, 3.0],
        ));
        // Zero normals and directions.
        assert_invalid(Workplane::from_point_normal([0.0; 3], [0.0; 3]));
        let zero_normal = Workplane {
            normal: [0.0; 3],
            ..Workplane::XY
        };
        assert_invalid(zero_normal.offset(1.0));
        assert_invalid(Workplane::XY.angled(
            Axis {
                origin: ORIGIN,
                dir: [0.0; 3],
            },
            1.0,
        ));
        // Axis not in the plane: tilted out of it, or parallel but off it.
        assert_invalid(Workplane::XY.angled(Axis::Z, 1.0));
        let off = Axis {
            origin: [0.0, 0.0, 1e-6],
            dir: [1.0, 0.0, 0.0],
        };
        assert_invalid(Workplane::XY.angled(off, 1.0));
        // Parallel planes.
        assert_invalid(Axis::plane_intersection(
            &Workplane::XY,
            &Workplane::XY.offset(3.0).unwrap(),
        ));
        assert_invalid(Axis::plane_intersection(&Workplane::XY, &Workplane::XY));
        // Axis parallel to the plane.
        assert_invalid(plane_axis_intersection(&Workplane::XY, &Axis::X));
        assert_invalid(plane_axis_intersection(
            &Workplane::XY.offset(7.0).unwrap(),
            &Axis {
                origin: [0.0; 3],
                dir: [1.0, 1.0, 0.0],
            },
        ));
    }

    #[test]
    fn plane_and_axis_are_placed_nearest_the_origin() {
        // A point anywhere on the plane z = 30 gives the origin (0, 0, 30).
        let p = plane_nearest_origin([7.0, -4.0, 30.0], [0.0, 0.0, 2.0]).unwrap();
        assert_plane(&p, [0.0, 0.0, 30.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]);
        // A normal along X takes +Y as x_dir.
        let p = plane_nearest_origin([10.0, 20.0, 30.0], [1.0, 0.0, 0.0]).unwrap();
        assert_plane(&p, [10.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]);
        // The line y = 20, z = 30 along X is nearest the origin at (0, 20, 30).
        let a = axis_nearest_origin([10.0, 20.0, 30.0], [-3.0, 0.0, 0.0]).unwrap();
        assert_vec(a.origin, [0.0, 20.0, 30.0]);
        assert_vec(a.dir, [-1.0, 0.0, 0.0]);
        let a = axis_nearest_origin([5.0, 5.0, 9.0], [0.0, 0.0, 1.0]).unwrap();
        assert_vec(a.origin, [5.0, 5.0, 0.0]);
        // Bad inputs.
        assert_invalid(plane_nearest_origin([0.0; 3], [0.0; 3]));
        assert_invalid(plane_nearest_origin([f64::NAN, 0.0, 0.0], [0.0, 0.0, 1.0]));
        assert_invalid(axis_nearest_origin([0.0; 3], [0.0; 3]));
        assert_invalid(axis_nearest_origin(
            [0.0, f64::INFINITY, 0.0],
            [1.0, 0.0, 0.0],
        ));
    }
}
