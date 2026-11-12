//! Input checks for [`Kernel::helical_sweep`](crate::Kernel::helical_sweep).
//!
//! Everything here is pure Rust, so every rejected input is testable
//! without OCCT. The backend only reports measurements of the profile face
//! ([`RevolveProfile`]); the decisions are made here.

use crate::profile::{finite, normalized, RevolveProfile};
use crate::{KernelError, Point3, LOOP_GAP_TOLERANCE, PERPENDICULAR_TOLERANCE};

fn invalid(msg: String) -> KernelError {
    KernelError::InvalidInput(msg)
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `a × b`.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Validates the helix numbers and axis and returns the axis origin and its
/// unit direction. `pitch` and `turns` must be finite and > 0; the axis
/// origin finite; its direction finite and non-zero.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn prepare_helix(
    origin: Point3,
    dir: [f64; 3],
    pitch: f64,
    turns: f64,
) -> Result<(Point3, [f64; 3]), KernelError> {
    for (name, v) in [("helix pitch", pitch), ("helix turns", turns)] {
        if !v.is_finite() || v <= 0.0 {
            return Err(invalid(format!("{name} must be finite and > 0, got {v}")));
        }
    }
    finite("helix axis origin", &origin)?;
    let dir = normalized("helix axis direction", dir)?;
    Ok((origin, dir))
}

/// Where the helix starts, relative to the axis: the unit radial direction
/// `x_dir` from the axis towards the profile, the start radius, and the
/// start height along the axis direction (from the axis origin).
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct HelixStart {
    /// Unit vector, perpendicular to the axis, in the profile's plane,
    /// pointing from the axis to the profile.
    pub x_dir: [f64; 3],
    /// Distance of the helix from the axis (inside the profile's radial span).
    pub radius: f64,
    /// Height of the helix start along the axis (inside the profile's axial span).
    pub height: f64,
}

/// Rejects a profile the helical sweep can't use and returns where the helix
/// starts.
///
/// `side` comes from the backend for the axis `origin`, `dir` (unit): the
/// profile's plane (point and unit normal `n`) and the extremes of the signed
/// distance `(P − origin) · w` over the profile, `w = dir × n`. `axial_min`
/// and `axial_max` are the extremes of `(P − origin) · dir`.
///
/// It is [`KernelError::InvalidInput`] when the axis is not in the profile's
/// plane (`|dir · n|` > [`PERPENDICULAR_TOLERANCE`], or the origin is more
/// than [`LOOP_GAP_TOLERANCE`] off the plane), when the profile touches or
/// crosses the axis (it does not lie strictly, by more than
/// `LOOP_GAP_TOLERANCE`, on one side of it), or when its extent along the
/// axis is ≥ `pitch` (neighboring turns would overlap).
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn check_helix_profile(
    origin: Point3,
    dir: [f64; 3],
    pitch: f64,
    side: &RevolveProfile,
    axial_min: f64,
    axial_max: f64,
) -> Result<HelixStart, KernelError> {
    let n = side.normal;
    let offset = [
        origin[0] - side.plane_point[0],
        origin[1] - side.plane_point[1],
        origin[2] - side.plane_point[2],
    ];
    if dot(dir, n).abs() > PERPENDICULAR_TOLERANCE || dot(offset, n).abs() > LOOP_GAP_TOLERANCE {
        return Err(invalid(format!(
            "helix axis (origin {origin:?}, direction {dir:?}) is not in the profile's plane \
             (through {:?}, normal {n:?})",
            side.plane_point
        )));
    }
    let tol = LOOP_GAP_TOLERANCE;
    let w = cross(dir, n);
    let (x_dir, near, far) = if side.side_min > tol {
        (w, side.side_min, side.side_max)
    } else if side.side_max < -tol {
        (w.map(|c| -c), -side.side_max, -side.side_min)
    } else {
        let what = if side.side_min < -tol && side.side_max > tol {
            "crosses"
        } else {
            "touches"
        };
        return Err(invalid(format!(
            "the profile {what} the helix axis: its distance from the axis spans [{}, {}]",
            side.side_min, side.side_max
        )));
    };
    let extent = axial_max - axial_min;
    if extent >= pitch {
        return Err(invalid(format!(
            "the profile's extent along the helix axis ({extent}) must be less than the \
             pitch ({pitch}), or neighboring turns overlap"
        )));
    }
    Ok(HelixStart {
        x_dir,
        radius: 0.5 * (near + far),
        height: 0.5 * (axial_min + axial_max),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const Z: [f64; 3] = [0.0, 0.0, 1.0];

    /// The XZ plane (normal −Y) with the given side and axial extremes.
    fn xz(side_min: f64, side_max: f64) -> RevolveProfile {
        RevolveProfile {
            plane_point: [0.0; 3],
            normal: [0.0, -1.0, 0.0],
            side_min,
            side_max,
        }
    }

    fn rejects(r: Result<HelixStart, KernelError>, needle: &str) {
        match r {
            Err(KernelError::InvalidInput(msg)) => assert!(msg.contains(needle), "{msg}"),
            other => panic!("expected InvalidInput containing {needle:?}, got {other:?}"),
        }
    }

    #[test]
    fn pitch_turns_and_axis_are_validated() {
        let o = [0.0; 3];
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                prepare_helix(o, Z, bad, 1.0),
                Err(KernelError::InvalidInput(m)) if m.contains("pitch")
            ));
            assert!(matches!(
                prepare_helix(o, Z, 1.0, bad),
                Err(KernelError::InvalidInput(m)) if m.contains("turns")
            ));
        }
        for bad in [[0.0; 3], [f64::NAN, 0.0, 1.0], [0.0, f64::INFINITY, 0.0]] {
            assert!(matches!(
                prepare_helix(o, bad, 1.0, 1.0),
                Err(KernelError::InvalidInput(m)) if m.contains("direction")
            ));
        }
        assert!(prepare_helix([f64::NAN, 0.0, 0.0], Z, 1.0, 1.0).is_err());
        assert_eq!(
            prepare_helix(o, [0.0, 0.0, 2.0], 2.0, 3.0),
            Ok((o, [0.0, 0.0, 1.0]))
        );
    }

    #[test]
    fn start_is_on_the_profile_side() {
        // w = Z × (−Y) = +X.
        let s = check_helix_profile([0.0; 3], Z, 2.0, &xz(5.0, 6.0), 0.0, 0.5).unwrap();
        assert_eq!(s.x_dir, [1.0, 0.0, 0.0]);
        assert_eq!((s.radius, s.height), (5.5, 0.25));
        let s = check_helix_profile([0.0; 3], Z, 2.0, &xz(-6.0, -5.0), 1.0, 2.0).unwrap();
        assert_eq!(s.x_dir, [-1.0, 0.0, 0.0]);
        assert_eq!((s.radius, s.height), (5.5, 1.5));
    }

    #[test]
    fn axis_must_lie_in_the_profile_plane() {
        let tilted = [0.0, 1e-6, 1.0];
        let n = (1.0f64 + 1e-12).sqrt();
        let tilted = tilted.map(|c| c / n);
        rejects(
            check_helix_profile([0.0; 3], tilted, 2.0, &xz(5.0, 6.0), 0.0, 0.5),
            "not in the profile's plane",
        );
        rejects(
            check_helix_profile([0.0, 1e-6, 0.0], Z, 2.0, &xz(5.0, 6.0), 0.0, 0.5),
            "not in the profile's plane",
        );
        assert!(check_helix_profile([0.0, 1e-8, 0.0], Z, 2.0, &xz(5.0, 6.0), 0.0, 0.5).is_ok());
    }

    #[test]
    fn profile_must_not_touch_or_cross_the_axis() {
        rejects(
            check_helix_profile([0.0; 3], Z, 2.0, &xz(-1.0, 1.0), 0.0, 0.5),
            "crosses",
        );
        rejects(
            check_helix_profile([0.0; 3], Z, 2.0, &xz(0.0, 1.0), 0.0, 0.5),
            "touches",
        );
        rejects(
            check_helix_profile([0.0; 3], Z, 2.0, &xz(-1.0, 1e-9), 0.0, 0.5),
            "touches",
        );
    }

    #[test]
    fn axial_extent_must_be_below_the_pitch() {
        rejects(
            check_helix_profile([0.0; 3], Z, 2.0, &xz(5.0, 6.0), 0.0, 2.5),
            "less than the pitch",
        );
        rejects(
            check_helix_profile([0.0; 3], Z, 2.0, &xz(5.0, 6.0), -1.0, 1.0),
            "less than the pitch",
        );
        assert!(check_helix_profile([0.0; 3], Z, 2.0, &xz(5.0, 6.0), -1.0, 0.999).is_ok());
    }
}
