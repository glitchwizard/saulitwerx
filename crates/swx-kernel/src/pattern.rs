//! Input checks and copy placements for [`Kernel::mirror`],
//! [`Kernel::pattern_rectangular`], [`Kernel::pattern_circular`] (M6-01a),
//! [`Kernel::pattern_path`] (M6-01b) and [`Kernel::pattern_path_aligned`]
//! (M6-01c). Plain functions, so they are tested without OCCT; the backend
//! only applies the resulting rigid transforms.
//!
//! [`Kernel::mirror`]: crate::Kernel::mirror
//! [`Kernel::pattern_rectangular`]: crate::Kernel::pattern_rectangular
//! [`Kernel::pattern_circular`]: crate::Kernel::pattern_circular
//! [`Kernel::pattern_path`]: crate::Kernel::pattern_path
//! [`Kernel::pattern_path_aligned`]: crate::Kernel::pattern_path_aligned

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::profile::{finite, normalized, prepare_path, Edge2, EdgeKind};
use crate::{Axis, KernelError, Point3, Segment2, Workplane};

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

/// Arc length of one path edge: its length for a line, `radius · |sweep|`
/// for an arc (the arc-length formula `s = rθ`; e.g. Stewart, *Calculus*,
/// §10.2).
fn edge_length(e: &Edge2) -> f64 {
    match e.kind {
        EdgeKind::Line => (e.to[0] - e.from[0]).hypot(e.to[1] - e.from[1]),
        _ => e.radius * e.sweep().abs(),
    }
}

/// The 2D point at arc length `t` (clamped to `0..=len`) along `e`, walked
/// from `e.from` to `e.to`.
fn point_on_edge(e: &Edge2, len: f64, t: f64) -> [f64; 2] {
    let t = t.clamp(0.0, len);
    match e.kind {
        EdgeKind::Line if len > 0.0 => {
            let f = t / len;
            [0, 1].map(|k| e.from[k] + f * (e.to[k] - e.from[k]))
        }
        EdgeKind::Line => e.from,
        _ => {
            let [cx, cy] = e.center;
            let a0 = (e.from[1] - cy).atan2(e.from[0] - cx);
            let a = a0 + e.sweep().signum() * t / e.radius;
            [cx + e.radius * a.cos(), cy + e.radius * a.sin()]
        }
    }
}

/// The translation of each copy of a path pattern: copy k moves by
/// `P(sₖ) − P(0)`, where `P(t)` is the model-space point at arc length `t`
/// along the path (mapped through `plane` as [`Kernel::sweep`] maps its
/// path) and `sₖ = k·L/(count − 1)`, `L` being the path's total length. So
/// the first offset is zero, the last copy sits at the path's end, and a
/// count of 1 gives the single offset zero.
///
/// The path is checked and chained as [`Kernel::sweep`] checks its path
/// (`prepare_path`): from the first segment's start, later segments may run
/// either way. A count of 0, and every path or plane that check rejects
/// (empty, a `Circle`, segments that don't meet, non-finite numbers, a
/// radius ≤ 0, an invalid plane), is `InvalidInput`.
///
/// [`Kernel::sweep`]: crate::Kernel::sweep
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn path_offsets(
    plane: &Workplane,
    path: &[Segment2],
    count: usize,
) -> Result<Vec<[f64; 3]>, KernelError> {
    let (_, _, copies) = path_aligned_placements(plane, path, count)?;
    Ok(copies.into_iter().map(|(_, offset)| offset).collect())
}

/// One copy of an aligned path pattern: its rotation angle in radians and
/// then its translation.
pub(crate) type AlignedCopy = (f64, [f64; 3]);

/// The placements of a path pattern whose copies turn with the path: the
/// pivot `P(0)`, the unit plane normal, and per copy the rotation angle
/// `θ(sₖ) − θ(0)` about the axis through the pivot along the normal
/// (applied first) and the translation `P(sₖ) − P(0)` (applied second, the
/// same offsets as [`path_offsets`]).
///
/// `θ(t)` is the 2D angle of the path's direction of travel at arc length
/// `t`, measured from the plane's `x_dir` toward `normal × x_dir`: a line's
/// direction, or for an arc the radius angle plus ±π/2 (the tangent of a
/// circle is perpendicular to its radius; e.g. Stewart, *Calculus*, §13.2).
/// Where two segments meet exactly at `sₖ`, the segment ending there gives
/// the tangent. Angles are reduced to (−π, π]; a rotation by a multiple of
/// 2π more is the same placement.
///
/// The same inputs are `InvalidInput` as for [`path_offsets`].
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn path_aligned_placements(
    plane: &Workplane,
    path: &[Segment2],
    count: usize,
) -> Result<(Point3, [f64; 3], Vec<AlignedCopy>), KernelError> {
    check_count("pattern count", count)?;
    let prepared = prepare_path(plane, path)?;
    let edges = &prepared.loops[0];
    let lengths: Vec<f64> = edges.iter().map(edge_length).collect();
    let total: f64 = lengths.iter().sum();
    let last = edges.len() - 1;
    // The edge index and the arc length into that edge at path length `s`;
    // a joint belongs to the edge ending there.
    let locate = |s: f64| -> (usize, f64) {
        let mut rest = s;
        for (i, &len) in lengths.iter().enumerate() {
            if rest <= len || i == last {
                return (i, rest);
            }
            rest -= len;
        }
        (last, rest)
    };
    let sample = |i: usize, t: f64| -> ([f64; 2], f64) {
        let (e, len) = (&edges[i], lengths[i]);
        (point_on_edge(e, len, t), tangent_on_edge(e, len, t))
    };
    let (_, theta0) = sample(0, 0.0);
    let start = prepared.point_3d(edges[0].from);
    let copies = (0..count)
        .map(|k| {
            let (p, theta) = if k == 0 {
                (edges[0].from, theta0)
            } else if k + 1 == count {
                (edges[last].to, sample(last, lengths[last]).1)
            } else {
                let (i, t) = locate(k as f64 * total / (count - 1) as f64);
                sample(i, t)
            };
            let q = prepared.point_3d(p);
            let offset = [0, 1, 2].map(|i| q[i] - start[i]);
            (wrap_angle(theta - theta0), offset)
        })
        .collect();
    Ok((start, prepared.normal, copies))
}

/// `a` reduced to (−π, π].
fn wrap_angle(a: f64) -> f64 {
    let r = a.rem_euclid(TAU);
    if r > PI {
        r - TAU
    } else {
        r
    }
}

/// The 2D angle of the direction of travel at arc length `t` (clamped to
/// `0..=len`) along `e`: the line's direction, or for an arc the radius
/// angle turned by +π/2 when counterclockwise and −π/2 when clockwise.
fn tangent_on_edge(e: &Edge2, len: f64, t: f64) -> f64 {
    match e.kind {
        EdgeKind::Line => (e.to[1] - e.from[1]).atan2(e.to[0] - e.from[0]),
        _ => {
            let [cx, cy] = e.center;
            let sign = e.sweep().signum();
            let a0 = (e.from[1] - cy).atan2(e.from[0] - cx);
            a0 + sign * (t.clamp(0.0, len) / e.radius + FRAC_PI_2)
        }
    }
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

    fn line(start: [f64; 2], end: [f64; 2]) -> Segment2 {
        Segment2::Line { start, end }
    }

    /// `Line (0,0)→(30,0)` then a quarter arc centered (30,10), radius 10,
    /// from −π/2 to 0, ending at (40,10).
    fn line_then_arc() -> Vec<Segment2> {
        vec![
            line([0.0, 0.0], [30.0, 0.0]),
            Segment2::Arc {
                center: [30.0, 10.0],
                radius: 10.0,
                start_angle: -FRAC_PI_2,
                end_angle: 0.0,
            },
        ]
    }

    fn assert_offset(actual: [f64; 3], expected: [f64; 3]) {
        for k in 0..3 {
            assert!(
                (actual[k] - expected[k]).abs() <= 1e-9,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    #[test]
    fn path_offsets_along_a_line_then_an_arc() {
        let xy = Workplane::XY;
        let l = 30.0 + 5.0 * PI;
        let o = path_offsets(&xy, &line_then_arc(), 3).unwrap();
        assert_eq!(o.len(), 3);
        assert_offset(o[0], [0.0; 3]);
        assert_offset(o[1], [l / 2.0, 0.0, 0.0]);
        assert_offset(o[2], [40.0, 10.0, 0.0]);
        let o = path_offsets(&xy, &line_then_arc(), 5).unwrap();
        let phi = (3.0 * l / 4.0 - 30.0) / 10.0;
        assert_offset(
            o[3],
            [30.0 + 10.0 * phi.sin(), 10.0 - 10.0 * phi.cos(), 0.0],
        );
        assert_eq!(
            path_offsets(&xy, &line_then_arc(), 1).unwrap(),
            vec![[0.0; 3]]
        );
    }

    #[test]
    fn path_offsets_follow_a_segment_given_end_first() {
        let path = [
            line([0.0, 0.0], [30.0, 0.0]),
            line([30.0, 20.0], [30.0, 0.0]),
        ];
        let o = path_offsets(&Workplane::XY, &path, 3).unwrap();
        assert_offset(o[0], [0.0; 3]);
        assert_offset(o[1], [25.0, 0.0, 0.0]);
        assert_offset(o[2], [30.0, 20.0, 0.0]);
    }

    #[test]
    fn path_offsets_reject_bad_counts_paths_and_planes() {
        let xy = Workplane::XY;
        let ok = line_then_arc();
        let arc = |radius: f64| Segment2::Arc {
            center: [0.0, 0.0],
            radius,
            start_angle: 0.0,
            end_angle: 1.0,
        };
        let circle = Segment2::Circle {
            center: [0.0, 0.0],
            radius: 5.0,
        };
        let bad = [
            path_offsets(&xy, &ok, 0),
            path_offsets(&xy, &[], 3),
            path_offsets(&xy, &[circle], 3),
            path_offsets(&xy, &[line([0.0, 0.0], [1.0, 0.0]), circle], 3),
            path_offsets(
                &xy,
                &[line([0.0, 0.0], [1.0, 0.0]), line([5.0, 5.0], [6.0, 5.0])],
                3,
            ),
            path_offsets(&xy, &[line([f64::NAN, 0.0], [1.0, 0.0])], 3),
            path_offsets(&xy, &[line([0.0, 0.0], [f64::INFINITY, 0.0])], 3),
            path_offsets(&xy, &[arc(0.0)], 3),
            path_offsets(&xy, &[arc(-2.0)], 3),
            path_offsets(
                &Workplane {
                    normal: [0.0; 3],
                    ..xy
                },
                &ok,
                3,
            ),
            path_offsets(
                &Workplane {
                    x_dir: [0.0, 0.0, 1.0],
                    ..xy
                },
                &ok,
                3,
            ),
            path_offsets(
                &Workplane {
                    origin: [f64::NAN, 0.0, 0.0],
                    ..xy
                },
                &ok,
                3,
            ),
        ];
        for (n, r) in bad.into_iter().enumerate() {
            assert!(is_invalid(r), "case {n}");
        }
    }

    fn angles(plane: &Workplane, path: &[Segment2], count: usize) -> Vec<f64> {
        let (_, _, copies) = path_aligned_placements(plane, path, count).unwrap();
        copies.into_iter().map(|(a, _)| a).collect()
    }

    fn assert_angles(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len());
        for (a, e) in actual.iter().zip(expected) {
            assert!(
                (a - e).abs() <= 1e-9,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    #[test]
    fn aligned_placements_along_a_line_then_an_arc() {
        let xy = Workplane::XY;
        let l = 30.0 + 5.0 * PI;
        let (pivot, normal, copies) = path_aligned_placements(&xy, &line_then_arc(), 3).unwrap();
        assert_eq!(pivot, [0.0; 3]);
        assert_eq!(normal, [0.0, 0.0, 1.0]);
        assert_angles(
            &copies.iter().map(|c| c.0).collect::<Vec<_>>(),
            &[0.0, 0.0, FRAC_PI_2],
        );
        assert_offset(copies[0].1, [0.0; 3]);
        assert_offset(copies[1].1, [l / 2.0, 0.0, 0.0]);
        assert_offset(copies[2].1, [40.0, 10.0, 0.0]);
        let phi = (3.0 * l / 4.0 - 30.0) / 10.0;
        assert!((angles(&xy, &line_then_arc(), 5)[3] - phi).abs() <= 1e-9);
        assert_angles(&angles(&xy, &line_then_arc(), 1), &[0.0]);
    }

    #[test]
    fn aligned_placements_turn_at_a_joint_given_end_first() {
        let path = [
            line([0.0, 0.0], [30.0, 0.0]),
            line([30.0, 20.0], [30.0, 0.0]),
        ];
        assert_angles(&angles(&Workplane::XY, &path, 3), &[0.0, 0.0, FRAC_PI_2]);
        // L = 50, so count 6 puts copy 3 exactly on the joint at s = 30:
        // the segment ending there (along +X) gives the tangent.
        assert_angles(
            &angles(&Workplane::XY, &path, 6),
            &[0.0, 0.0, 0.0, 0.0, FRAC_PI_2, FRAC_PI_2],
        );
    }

    #[test]
    fn aligned_placements_never_turn_on_a_straight_path() {
        let path = [line([10.0, 0.0], [0.0, 0.0])];
        assert_angles(&angles(&Workplane::XY, &path, 2), &[0.0, 0.0]);
    }

    #[test]
    fn aligned_placements_reject_what_path_offsets_rejects() {
        let xy = Workplane::XY;
        let circle = Segment2::Circle {
            center: [0.0, 0.0],
            radius: 5.0,
        };
        let bad = [
            path_aligned_placements(&xy, &line_then_arc(), 0),
            path_aligned_placements(&xy, &[], 3),
            path_aligned_placements(&xy, &[circle], 3),
            path_aligned_placements(
                &xy,
                &[line([0.0, 0.0], [1.0, 0.0]), line([5.0, 5.0], [6.0, 5.0])],
                3,
            ),
            path_aligned_placements(&xy, &[line([f64::NAN, 0.0], [1.0, 0.0])], 3),
            path_aligned_placements(
                &xy,
                &[Segment2::Arc {
                    center: [0.0, 0.0],
                    radius: 0.0,
                    start_angle: 0.0,
                    end_angle: 1.0,
                }],
                3,
            ),
            path_aligned_placements(
                &Workplane {
                    normal: [0.0; 3],
                    ..xy
                },
                &line_then_arc(),
                3,
            ),
        ];
        for (n, r) in bad.into_iter().enumerate() {
            assert!(is_invalid(r), "case {n}");
        }
    }
}
