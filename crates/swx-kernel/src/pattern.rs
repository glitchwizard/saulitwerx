//! Input checks and copy placements for [`Kernel::mirror`],
//! [`Kernel::pattern_rectangular`] and [`Kernel::pattern_circular`]
//! (M6-01a). Plain functions, so they are tested without OCCT; the backend
//! only applies the resulting rigid transforms.
//!
//! [`Kernel::mirror`]: crate::Kernel::mirror
//! [`Kernel::pattern_rectangular`]: crate::Kernel::pattern_rectangular
//! [`Kernel::pattern_circular`]: crate::Kernel::pattern_circular

use std::f64::consts::TAU;

use crate::profile::{finite, normalized};
use crate::{Axis, KernelError, Point3, Workplane};

/// Largest amount a circular pattern's total angle may differ from 2π and
/// still count as a full turn.
pub(crate) const FULL_TURN_TOLERANCE: f64 = 1e-12;

fn invalid(msg: String) -> KernelError {
    KernelError::InvalidInput(msg)
}

fn check_count(name: &str, count: usize) -> Result<(), KernelError> {
    if count == 0 {
        Err(invalid(format!("{name} must be at least 1")))
    } else {
        Ok(())
    }
}

fn check_spacing(name: &str, spacing: f64) -> Result<(), KernelError> {
    finite(name, &[spacing])
}

/// The mirror plane's origin and unit normal, from `plane.origin` and
/// `plane.normal` (`plane.x_dir` is not used). A non-finite origin, or a
/// zero or non-finite normal, is `InvalidInput`.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn prepare_mirror(plane: &Workplane) -> Result<(Point3, [f64; 3]), KernelError> {
    finite("mirror plane origin", &plane.origin)?;
    let normal = normalized("mirror plane normal", plane.normal)?;
    Ok((plane.origin, normal))
}

/// The translation of each copy of a rectangular pattern, in row-major
/// order (i outer, j inner): copy (i, j) moves by
/// `i·spacing1·d̂1 + j·spacing2·d̂2`, with `d̂` the normalized directions and
/// `count2 = 1, spacing2 = 0` without `dir2`. The first offset is zero.
///
/// A count of 0, a non-finite spacing, or a zero or non-finite direction is
/// `InvalidInput`.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn rectangular_offsets(
    dir1: [f64; 3],
    count1: usize,
    spacing1: f64,
    dir2: Option<([f64; 3], usize, f64)>,
) -> Result<Vec<[f64; 3]>, KernelError> {
    let d1 = normalized("pattern direction 1", dir1)?;
    check_count("pattern count 1", count1)?;
    check_spacing("pattern spacing 1", spacing1)?;
    let (d2, count2, spacing2) = match dir2 {
        Some((dir, count, spacing)) => {
            let d = normalized("pattern direction 2", dir)?;
            check_count("pattern count 2", count)?;
            check_spacing("pattern spacing 2", spacing)?;
            (d, count, spacing)
        }
        None => ([0.0; 3], 1, 0.0),
    };
    let mut offsets = Vec::with_capacity(count1.saturating_mul(count2));
    for i in 0..count1 {
        for j in 0..count2 {
            let (a, b) = (i as f64 * spacing1, j as f64 * spacing2);
            offsets.push([0, 1, 2].map(|k| a * d1[k] + b * d2[k]));
        }
    }
    Ok(offsets)
}

/// The axis origin, unit axis direction and rotation angle of each copy of
/// a circular pattern. Copy k turns by `k·total/count` when `total_angle`
/// is a full turn (2π within [`FULL_TURN_TOLERANCE`]), so the copies are
/// evenly spaced around the circle without the last landing on the first.
/// Otherwise it turns by `k·total/(count − 1)`, so the last copy lands on
/// `total_angle`; a count of 1 gives the single angle 0.
///
/// A count of 0, a `total_angle` that is not finite, ≤ 0 or above 2π (more
/// than the tolerance), a non-finite axis origin, or a zero or non-finite
/// axis direction is `InvalidInput`.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn circular_angles(
    axis: Axis,
    count: usize,
    total_angle: f64,
) -> Result<(Point3, [f64; 3], Vec<f64>), KernelError> {
    finite("pattern axis origin", &axis.origin)?;
    let dir = normalized("pattern axis direction", axis.dir)?;
    check_count("pattern count", count)?;
    if !total_angle.is_finite() || total_angle <= 0.0 || total_angle > TAU + FULL_TURN_TOLERANCE {
        return Err(invalid(format!(
            "pattern total angle must be in (0, 2π] radians, got {total_angle}"
        )));
    }
    let step = if (total_angle - TAU).abs() <= FULL_TURN_TOLERANCE {
        TAU / count as f64
    } else if count == 1 {
        0.0
    } else {
        total_angle / (count - 1) as f64
    };
    let angles = (0..count).map(|k| k as f64 * step).collect();
    Ok((axis.origin, dir, angles))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::{FRAC_PI_2, PI};

    fn is_invalid<T: std::fmt::Debug>(r: Result<T, KernelError>) -> bool {
        matches!(r, Err(KernelError::InvalidInput(_)))
    }

    const X: [f64; 3] = [1.0, 0.0, 0.0];
    const Y: [f64; 3] = [0.0, 1.0, 0.0];
    const Z_AXIS: Axis = Axis {
        origin: [0.0; 3],
        dir: [0.0, 0.0, 1.0],
    };

    #[test]
    fn mirror_normalizes_and_rejects_bad_planes() {
        let plane = Workplane {
            origin: [20.0, 0.0, 0.0],
            x_dir: Y,
            normal: [2.0, 0.0, 0.0],
        };
        assert_eq!(prepare_mirror(&plane).unwrap(), ([20.0, 0.0, 0.0], X));
        for normal in [[0.0; 3], [f64::NAN, 0.0, 1.0], [f64::INFINITY, 0.0, 0.0]] {
            assert!(is_invalid(prepare_mirror(&Workplane { normal, ..plane })));
        }
        for origin in [[f64::NAN, 0.0, 0.0], [0.0, f64::NEG_INFINITY, 0.0]] {
            assert!(is_invalid(prepare_mirror(&Workplane { origin, ..plane })));
        }
    }

    #[test]
    fn rectangular_offsets_are_row_major_from_zero() {
        let o = rectangular_offsets([3.0, 0.0, 0.0], 3, 15.0, Some((Y, 2, 8.0))).unwrap();
        assert_eq!(
            o,
            vec![
                [0.0, 0.0, 0.0],
                [0.0, 8.0, 0.0],
                [15.0, 0.0, 0.0],
                [15.0, 8.0, 0.0],
                [30.0, 0.0, 0.0],
                [30.0, 8.0, 0.0],
            ]
        );
        let single = rectangular_offsets(X, 2, -5.0, None).unwrap();
        assert_eq!(single, vec![[0.0; 3], [-5.0, 0.0, 0.0]]);
    }

    #[test]
    fn rectangular_rejects_bad_counts_spacings_and_directions() {
        let bad = [
            rectangular_offsets(X, 0, 1.0, None),
            rectangular_offsets(X, 2, f64::NAN, None),
            rectangular_offsets(X, 2, f64::INFINITY, None),
            rectangular_offsets([0.0; 3], 2, 1.0, None),
            rectangular_offsets([f64::NAN, 0.0, 0.0], 2, 1.0, None),
            rectangular_offsets(X, 2, 1.0, Some((Y, 0, 1.0))),
            rectangular_offsets(X, 2, 1.0, Some((Y, 2, f64::NEG_INFINITY))),
            rectangular_offsets(X, 2, 1.0, Some(([0.0; 3], 2, 1.0))),
            rectangular_offsets(X, 2, 1.0, Some(([0.0, f64::INFINITY, 0.0], 2, 1.0))),
        ];
        for (n, r) in bad.into_iter().enumerate() {
            assert!(is_invalid(r), "case {n}");
        }
    }

    #[test]
    fn circular_full_turn_and_partial_spacing() {
        let (_, dir, a) = circular_angles(
            Axis {
                dir: [0.0, 0.0, 4.0],
                ..Z_AXIS
            },
            6,
            TAU,
        )
        .unwrap();
        assert_eq!(dir, [0.0, 0.0, 1.0]);
        assert_eq!(a.len(), 6);
        assert!((a[1] - PI / 3.0).abs() < 1e-15 && (a[5] - 5.0 * PI / 3.0).abs() < 1e-15);
        // Within the tolerance above 2π still counts as a full turn.
        let (_, _, a) = circular_angles(Z_AXIS, 4, TAU + 0.5e-12).unwrap();
        assert!((a[1] - FRAC_PI_2).abs() < 1e-15);
        let (_, _, a) = circular_angles(Z_AXIS, 3, FRAC_PI_2).unwrap();
        assert_eq!(a, vec![0.0, FRAC_PI_2 / 2.0, FRAC_PI_2]);
        let (_, _, a) = circular_angles(Z_AXIS, 1, PI).unwrap();
        assert_eq!(a, vec![0.0]);
    }

    #[test]
    fn circular_rejects_bad_counts_angles_and_axes() {
        let bad = [
            circular_angles(Z_AXIS, 0, TAU),
            circular_angles(Z_AXIS, 3, 0.0),
            circular_angles(Z_AXIS, 3, -1.0),
            circular_angles(Z_AXIS, 3, TAU + 1e-9),
            circular_angles(Z_AXIS, 3, f64::NAN),
            circular_angles(Z_AXIS, 3, f64::INFINITY),
            circular_angles(
                Axis {
                    dir: [0.0; 3],
                    ..Z_AXIS
                },
                3,
                PI,
            ),
            circular_angles(
                Axis {
                    dir: [0.0, f64::NAN, 1.0],
                    ..Z_AXIS
                },
                3,
                PI,
            ),
            circular_angles(
                Axis {
                    origin: [f64::INFINITY, 0.0, 0.0],
                    ..Z_AXIS
                },
                3,
                PI,
            ),
        ];
        for (n, r) in bad.into_iter().enumerate() {
            assert!(is_invalid(r), "case {n}");
        }
    }
}
