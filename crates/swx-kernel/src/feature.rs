//! Part features built from kernel primitives.
//!
//! [`extrude_feature`] sweeps a planar face and combines the result with an
//! existing body through the [`Kernel`] booleans, as an extrude feature's
//! result mode does (PRODUCT.md §1 items 6 and 9).

use crate::{Axis, Extent, Kernel, KernelError};

/// Results with a volume below this (model units cubed) count as empty.
pub const EMPTY_VOLUME_TOLERANCE: f64 = 1e-9;

/// How an extrusion is combined with the target body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtrudeMode {
    /// The extrusion becomes a separate body; the target is not used.
    NewSolid,
    /// Target ∪ extrusion.
    Join,
    /// Target − extrusion.
    Cut,
    /// Target ∩ extrusion.
    Intersect,
}

impl ExtrudeMode {
    fn name(self) -> &'static str {
        match self {
            ExtrudeMode::NewSolid => "new solid",
            ExtrudeMode::Join => "join",
            ExtrudeMode::Cut => "cut",
            ExtrudeMode::Intersect => "intersect",
        }
    }
}

/// Extrudes `face` by `extent` and combines it with `target` as `mode` says.
///
/// - [`ExtrudeMode::NewSolid`] returns the extrusion alone and ignores `target`.
/// - [`ExtrudeMode::Join`], [`ExtrudeMode::Cut`] and [`ExtrudeMode::Intersect`]
///   return `target ∪ e`, `target − e` and `target ∩ e`, where `e` is the
///   extrusion. They need a `target`: `None` is [`KernelError::InvalidInput`],
///   checked before anything reaches the backend.
///
/// A result whose volume is below [`EMPTY_VOLUME_TOLERANCE`] (a cut that
/// removes the whole target, an intersect with no overlap) is
/// [`KernelError::EmptyResult`]. Errors from [`Kernel::extrude`] and the
/// booleans are passed through.
pub fn extrude_feature<K: Kernel>(
    k: &K,
    face: &K::Shape,
    extent: Extent,
    mode: ExtrudeMode,
    target: Option<&K::Shape>,
) -> Result<K::Shape, KernelError> {
    let target = feature_target("extrude", mode, target)?;
    let tool = k.extrude(face, extent)?;
    combine(k, "extrude", tool, mode, target)
}

/// Revolves `face` by `angle` radians about `axis` ([`Kernel::revolve`]) and
/// combines the result with `target` as `mode` says.
///
/// The rules are those of [`extrude_feature`]: [`ExtrudeMode::NewSolid`]
/// ignores `target`; the other modes need one ([`KernelError::InvalidInput`]
/// before anything reaches the backend); a result with a volume below
/// [`EMPTY_VOLUME_TOLERANCE`] is [`KernelError::EmptyResult`]. Errors from
/// [`Kernel::revolve`] (including its input checks) and the booleans are
/// passed through.
pub fn revolve_feature<K: Kernel>(
    k: &K,
    face: &K::Shape,
    axis: Axis,
    angle: f64,
    mode: ExtrudeMode,
    target: Option<&K::Shape>,
) -> Result<K::Shape, KernelError> {
    let target = feature_target("revolve", mode, target)?;
    let tool = k.revolve(face, axis, angle)?;
    combine(k, "revolve", tool, mode, target)
}

/// The target a `feature` in `mode` combines with: `None` for
/// [`ExtrudeMode::NewSolid`], otherwise the required `target`.
fn feature_target<'a, S>(
    feature: &str,
    mode: ExtrudeMode,
    target: Option<&'a S>,
) -> Result<Option<&'a S>, KernelError> {
    match (mode, target) {
        (ExtrudeMode::NewSolid, _) => Ok(None),
        (_, Some(t)) => Ok(Some(t)),
        (_, None) => Err(KernelError::InvalidInput(format!(
            "{feature} mode \"{}\" needs a target body",
            mode.name()
        ))),
    }
}

/// Combines a feature's `tool` body with `target` as `mode` says and rejects
/// an empty result.
fn combine<K: Kernel>(
    k: &K,
    feature: &str,
    tool: K::Shape,
    mode: ExtrudeMode,
    target: Option<&K::Shape>,
) -> Result<K::Shape, KernelError> {
    let result = match (mode, target) {
        (ExtrudeMode::Join, Some(t)) => k.union(t, &tool)?,
        (ExtrudeMode::Cut, Some(t)) => k.cut(t, &tool)?,
        (ExtrudeMode::Intersect, Some(t)) => k.intersect(t, &tool)?,
        _ => tool,
    };
    let volume = k.volume(&result)?;
    if volume < EMPTY_VOLUME_TOLERANCE {
        return Err(KernelError::EmptyResult(format!(
            "{feature} {} leaves no solid (volume {volume:e})",
            mode.name()
        )));
    }
    Ok(result)
}

#[cfg(all(test, feature = "occt"))]
mod tests {
    use super::*;
    use crate::{OcctKernel, OcctShape, Point3, ProfileLoop, Segment2, Workplane};
    use std::f64::consts::PI;

    const K: OcctKernel = OcctKernel;

    /// Workplane at z = 10 (the target's top face), normal +z.
    const TOP: Workplane = Workplane {
        origin: [0.0, 0.0, 10.0],
        x_dir: [1.0, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    };

    /// Relative tolerance 1e-6, falling back to absolute 1e-6 near zero.
    fn assert_close(actual: f64, expected: f64) {
        let tol = 1e-6 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= tol,
            "expected {expected}, got {actual}"
        );
    }

    fn assert_bbox(s: &OcctShape, min: Point3, max: Point3) {
        let bb = K.bounding_box(s).unwrap();
        for axis in 0..3 {
            assert_close(bb.min[axis], min[axis]);
            assert_close(bb.max[axis], max[axis]);
        }
    }

    fn target() -> OcctShape {
        K.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0]).unwrap()
    }

    fn circle_face() -> OcctShape {
        let outer = ProfileLoop {
            segments: vec![Segment2::Circle {
                center: [20.0, 12.5],
                radius: 5.0,
            }],
        };
        K.make_face(&TOP, &outer, &[]).unwrap()
    }

    fn square_face(corner: [f64; 2], side: f64) -> OcctShape {
        let [x, y] = corner;
        let pts = [[x, y], [x + side, y], [x + side, y + side], [x, y + side]];
        let outer = ProfileLoop {
            segments: (0..4)
                .map(|i| Segment2::Line {
                    start: pts[i],
                    end: pts[(i + 1) % 4],
                })
                .collect(),
        };
        K.make_face(&TOP, &outer, &[]).unwrap()
    }

    #[test]
    fn join_adds_a_boss() {
        let t = target();
        let r = extrude_feature(
            &K,
            &circle_face(),
            Extent::Distance(5.0),
            ExtrudeMode::Join,
            Some(&t),
        )
        .unwrap();
        assert_close(K.volume(&r).unwrap(), 10000.0 + 125.0 * PI);
        assert_bbox(&r, [0.0, 0.0, 0.0], [40.0, 25.0, 15.0]);
    }

    #[test]
    fn cut_makes_a_through_hole() {
        let t = target();
        let r = extrude_feature(
            &K,
            &circle_face(),
            Extent::Distance(-10.0),
            ExtrudeMode::Cut,
            Some(&t),
        )
        .unwrap();
        assert_close(K.volume(&r).unwrap(), 10000.0 - 250.0 * PI);
        assert_eq!(K.face_count(&r).unwrap(), 7);
    }

    #[test]
    fn intersect_keeps_the_overlap() {
        let t = target();
        let r = extrude_feature(
            &K,
            &square_face([0.0, 0.0], 10.0),
            Extent::Distance(-4.0),
            ExtrudeMode::Intersect,
            Some(&t),
        )
        .unwrap();
        assert_close(K.volume(&r).unwrap(), 400.0);
        assert_bbox(&r, [0.0, 0.0, 6.0], [10.0, 10.0, 10.0]);
    }

    #[test]
    fn new_solid_ignores_the_target() {
        let t = target();
        for target in [None, Some(&t)] {
            let r = extrude_feature(
                &K,
                &circle_face(),
                Extent::Distance(5.0),
                ExtrudeMode::NewSolid,
                target,
            )
            .unwrap();
            assert_close(K.volume(&r).unwrap(), 125.0 * PI);
            assert_bbox(&r, [15.0, 7.5, 10.0], [25.0, 17.5, 15.0]);
        }
        assert_close(K.volume(&t).unwrap(), 10000.0);
        assert_bbox(&t, [0.0, 0.0, 0.0], [40.0, 25.0, 10.0]);
    }

    #[test]
    fn combining_modes_need_a_target() {
        for mode in [ExtrudeMode::Join, ExtrudeMode::Cut, ExtrudeMode::Intersect] {
            let err =
                extrude_feature(&K, &circle_face(), Extent::Distance(5.0), mode, None).unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("needs a target")),
                "{mode:?}: {err}"
            );
        }
    }

    #[test]
    fn cut_removing_the_whole_target_is_an_error() {
        let t = target();
        let err = extrude_feature(
            &K,
            &square_face([-5.0, -5.0], 50.0),
            Extent::Distance(-10.0),
            ExtrudeMode::Cut,
            Some(&t),
        )
        .unwrap_err();
        assert!(matches!(err, KernelError::EmptyResult(_)), "{err}");
        assert!(err.to_string().contains("leaves no solid"), "{err}");
    }

    /// The z axis through the origin.
    const Z_AXIS: Axis = Axis {
        origin: [0.0; 3],
        dir: [0.0, 0.0, 1.0],
    };

    /// The rectangle x0..x1, z0..z1 in the y = 0 plane (u is x, v is z).
    fn rect_xz(x0: f64, x1: f64, z0: f64, z1: f64) -> OcctShape {
        let plane = Workplane {
            origin: [0.0; 3],
            x_dir: [1.0, 0.0, 0.0],
            normal: [0.0, -1.0, 0.0],
        };
        let pts = [[x0, z0], [x1, z0], [x1, z1], [x0, z1]];
        let outer = ProfileLoop {
            segments: (0..4)
                .map(|i| Segment2::Line {
                    start: pts[i],
                    end: pts[(i + 1) % 4],
                })
                .collect(),
        };
        K.make_face(&plane, &outer, &[]).unwrap()
    }

    #[test]
    fn revolve_full_turn_makes_a_ring() {
        let r = K
            .revolve(&rect_xz(10.0, 20.0, 0.0, 5.0), Z_AXIS, 2.0 * PI)
            .unwrap();
        assert_close(K.volume(&r).unwrap(), 1500.0 * PI);
        assert_bbox(&r, [-20.0, -20.0, 0.0], [20.0, 20.0, 5.0]);
        assert_eq!(K.face_count(&r).unwrap(), 4);
    }

    #[test]
    fn revolve_quarter_turn_goes_from_x_towards_y() {
        let r = K
            .revolve(&rect_xz(10.0, 20.0, 0.0, 5.0), Z_AXIS, PI / 2.0)
            .unwrap();
        assert_close(K.volume(&r).unwrap(), 375.0 * PI);
        assert_bbox(&r, [0.0, 0.0, 0.0], [20.0, 20.0, 5.0]);
        assert_eq!(K.face_count(&r).unwrap(), 6);
    }

    #[test]
    fn revolve_cut_cuts_a_ring_groove() {
        let t = K.make_box([-25.0, -25.0, 0.0], [50.0, 50.0, 10.0]).unwrap();
        let r = revolve_feature(
            &K,
            &rect_xz(10.0, 20.0, 5.0, 10.0),
            Z_AXIS,
            2.0 * PI,
            ExtrudeMode::Cut,
            Some(&t),
        )
        .unwrap();
        assert_close(K.volume(&r).unwrap(), 25000.0 - 1500.0 * PI);
    }

    #[test]
    fn revolve_join_fills_a_disc() {
        let t = K.make_cylinder([0.0; 3], 10.0, 5.0).unwrap();
        let r = revolve_feature(
            &K,
            &rect_xz(10.0, 20.0, 0.0, 5.0),
            Z_AXIS,
            2.0 * PI,
            ExtrudeMode::Join,
            Some(&t),
        )
        .unwrap();
        assert_close(K.volume(&r).unwrap(), 2000.0 * PI);
    }

    #[test]
    fn revolve_new_solid_ignores_the_target_and_others_need_one() {
        let face = rect_xz(10.0, 20.0, 0.0, 5.0);
        let r = revolve_feature(&K, &face, Z_AXIS, PI / 2.0, ExtrudeMode::NewSolid, None).unwrap();
        assert_close(K.volume(&r).unwrap(), 375.0 * PI);
        for mode in [ExtrudeMode::Join, ExtrudeMode::Cut, ExtrudeMode::Intersect] {
            let err = revolve_feature(&K, &face, Z_AXIS, PI, mode, None).unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("revolve mode") && m.contains("needs a target")),
                "{mode:?}: {err}"
            );
        }
    }

    #[test]
    fn revolve_rejects_bad_angles_and_axes() {
        let face = rect_xz(10.0, 20.0, 0.0, 5.0);
        for angle in [0.0, -1.0, 2.0 * PI + 1e-9, 7.0, f64::NAN, f64::INFINITY] {
            let err = K.revolve(&face, Z_AXIS, angle).unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("revolve angle")),
                "{angle}: {err}"
            );
        }
        let bad_axes = [
            Axis {
                dir: [0.0; 3],
                ..Z_AXIS
            },
            Axis {
                dir: [0.0, f64::NAN, 1.0],
                ..Z_AXIS
            },
            Axis {
                origin: [f64::INFINITY, 0.0, 0.0],
                ..Z_AXIS
            },
        ];
        for axis in bad_axes {
            let err = K.revolve(&face, axis, PI).unwrap_err();
            assert!(
                matches!(err, KernelError::InvalidInput(_)),
                "{axis:?}: {err}"
            );
        }
        // 2π within 1e-12 is a full turn.
        let r = K.revolve(&face, Z_AXIS, 2.0 * PI + 1e-13).unwrap();
        assert_close(K.volume(&r).unwrap(), 1500.0 * PI);
    }

    #[test]
    fn intersect_without_overlap_is_an_error() {
        let t = target();
        let err = extrude_feature(
            &K,
            &circle_face(),
            Extent::Distance(5.0),
            ExtrudeMode::Intersect,
            Some(&t),
        )
        .unwrap_err();
        assert!(matches!(err, KernelError::EmptyResult(_)), "{err}");
    }
}
