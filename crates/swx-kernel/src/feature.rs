//! Part features built from kernel primitives.
//!
//! [`extrude_feature`] sweeps a planar face and combines the result with an
//! existing body through the [`Kernel`] booleans, as an extrude feature's
//! result mode does (PRODUCT.md §1 items 6 and 9).

use crate::{Extent, Kernel, KernelError};

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
    let target = match (mode, target) {
        (ExtrudeMode::NewSolid, _) => None,
        (_, Some(t)) => Some(t),
        (_, None) => {
            return Err(KernelError::InvalidInput(format!(
                "extrude mode \"{}\" needs a target body",
                mode.name()
            )))
        }
    };
    let tool = k.extrude(face, extent)?;
    let result = match (mode, target) {
        (ExtrudeMode::Join, Some(t)) => k.union(t, &tool)?,
        (ExtrudeMode::Cut, Some(t)) => k.cut(t, &tool)?,
        (ExtrudeMode::Intersect, Some(t)) => k.intersect(t, &tool)?,
        _ => tool,
    };
    let volume = k.volume(&result)?;
    if volume < EMPTY_VOLUME_TOLERANCE {
        return Err(KernelError::EmptyResult(format!(
            "extrude {} leaves no solid (volume {volume:e})",
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
