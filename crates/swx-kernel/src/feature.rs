//! Part features built from kernel primitives.
//!
//! [`extrude_feature`] sweeps a planar face and combines the result with an
//! existing body through the [`Kernel`] booleans, as an extrude feature's
//! result mode does (PRODUCT.md §1 items 6 and 9).

use crate::{BoundingBox, Extent, Kernel, KernelError, ThroughDirection};

/// Results with a volume below this (model units cubed) count as empty.
pub const EMPTY_VOLUME_TOLERANCE: f64 = 1e-9;

/// Relative volume change below which a through-all cut counts as removing nothing.
const MISSED_CUT_TOLERANCE: f64 = 1e-9;

/// Rejects an [`Extent::ThroughAll`] with any mode but [`ExtrudeMode::Cut`].
fn check_extent_mode(extent: Extent, mode: ExtrudeMode) -> Result<(), KernelError> {
    match (extent, mode) {
        (Extent::ThroughAll(_), m) if m != ExtrudeMode::Cut => {
            Err(KernelError::InvalidInput(format!(
                "a through-all extent works only with the cut mode, not \"{}\"",
                m.name()
            )))
        }
        _ => Ok(()),
    }
}

/// Replaces a through-all direction with a finite extent that reaches past
/// every point of the target.
///
/// With box centers `c_f`, `c_t` and diagonals `d_f`, `d_t` of the face and
/// the target, `L = |c_f − c_t| + d_f/2 + d_t/2 + 1`. The face plane passes
/// through a face point `q` with `|q − c_f| ≤ d_f/2`, and every target point
/// `p` has `|p − c_t| ≤ d_t/2`, so by the triangle inequality `p` is at most
/// `|p − q| ≤ L − 1` from the plane. `Distance(±L)` or `Symmetric(2L)` thus
/// overshoots the target by at least 1 model unit on the chosen side(s).
fn through_all_extent(dir: ThroughDirection, face: BoundingBox, target: BoundingBox) -> Extent {
    let center = |b: BoundingBox| [0, 1, 2].map(|i| (b.min[i] + b.max[i]) / 2.0);
    let norm = |v: [f64; 3]| v.iter().map(|c| c * c).sum::<f64>().sqrt();
    let half_diag = |b: BoundingBox| norm([0, 1, 2].map(|i| b.max[i] - b.min[i])) / 2.0;
    let (cf, ct) = (center(face), center(target));
    let l = norm([0, 1, 2].map(|i| cf[i] - ct[i])) + half_diag(face) + half_diag(target) + 1.0;
    match dir {
        ThroughDirection::Normal => Extent::Distance(l),
        ThroughDirection::Reversed => Extent::Distance(-l),
        ThroughDirection::Both => Extent::Symmetric(2.0 * l),
    }
}

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
///
/// [`Extent::ThroughAll`] works only with [`ExtrudeMode::Cut`] (any other mode
/// is [`KernelError::InvalidInput`], checked first). It is replaced by a finite
/// extent measured from the bounding boxes of `face` and `target`, long enough
/// to pass through the whole target (see `through_all_extent` for the bound).
/// A through-all cut whose result has the target's volume within a relative
/// 1e-9 misses the target and is [`KernelError::InvalidInput`].
pub fn extrude_feature<K: Kernel>(
    k: &K,
    face: &K::Shape,
    extent: Extent,
    mode: ExtrudeMode,
    target: Option<&K::Shape>,
) -> Result<K::Shape, KernelError> {
    check_extent_mode(extent, mode)?;
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
    let (extent, through_all) = match (extent, target) {
        (Extent::ThroughAll(dir), Some(t)) => (
            through_all_extent(dir, k.bounding_box(face)?, k.bounding_box(t)?),
            true,
        ),
        _ => (extent, false),
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
    if let (true, Some(t)) = (through_all, target) {
        let before = k.volume(t)?;
        if (before - volume).abs() <= MISSED_CUT_TOLERANCE * before.abs() {
            return Err(KernelError::InvalidInput(
                "the through-all cut misses the target: it removes nothing".into(),
            ));
        }
    }
    Ok(result)
}

#[cfg(test)]
mod pure_tests {
    use super::*;

    #[test]
    fn through_all_needs_the_cut_mode() {
        let ext = Extent::ThroughAll(ThroughDirection::Both);
        assert_eq!(check_extent_mode(ext, ExtrudeMode::Cut), Ok(()));
        for mode in [
            ExtrudeMode::NewSolid,
            ExtrudeMode::Join,
            ExtrudeMode::Intersect,
        ] {
            let err = check_extent_mode(ext, mode).unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("only with the cut mode")),
                "{mode:?}: {err}"
            );
        }
        for mode in [ExtrudeMode::NewSolid, ExtrudeMode::Join] {
            assert_eq!(check_extent_mode(Extent::Distance(1.0), mode), Ok(()));
        }
    }

    #[test]
    fn through_all_length_reaches_past_the_target() {
        // Face: a 10 × 10 square at z = 100, centered at (20, 12.5).
        let face = BoundingBox {
            min: [15.0, 7.5, 100.0],
            max: [25.0, 17.5, 100.0],
        };
        let target = BoundingBox {
            min: [0.0; 3],
            max: [40.0, 25.0, 100.0],
        };
        // |c_f − c_t| = 50, d_f/2 = √200/2, d_t/2 = √12225/2.
        let l = 50.0 + 200f64.sqrt() / 2.0 + 12225f64.sqrt() / 2.0 + 1.0;
        let close = |e: Extent, want: Extent| match (e, want) {
            (Extent::Distance(a), Extent::Distance(b))
            | (Extent::Symmetric(a), Extent::Symmetric(b)) => (a - b).abs() < 1e-12,
            _ => false,
        };
        let r = |d| through_all_extent(d, face, target);
        assert!(close(r(ThroughDirection::Normal), Extent::Distance(l)));
        assert!(close(r(ThroughDirection::Reversed), Extent::Distance(-l)));
        assert!(close(r(ThroughDirection::Both), Extent::Symmetric(2.0 * l)));
        // The whole target (depth 100 below the face) is within reach.
        assert!(l > 101.0);
    }
}

#[cfg(all(test, feature = "occt"))]
mod tests {
    use super::*;
    use crate::{
        OcctKernel, OcctShape, Point3, ProfileLoop, Segment2, ThroughDirection, Workplane,
    };
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

    /// The 40 × 25 × 100 block of the through-all tests.
    fn tall_block() -> OcctShape {
        K.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 100.0]).unwrap()
    }

    /// A radius-5 circle at `(u, v)` on `plane`.
    fn circle_on(plane: Workplane, center: [f64; 2]) -> OcctShape {
        let outer = ProfileLoop {
            segments: vec![Segment2::Circle {
                center,
                radius: 5.0,
            }],
        };
        K.make_face(&plane, &outer, &[]).unwrap()
    }

    /// Through-all cut of a radius-5 circle at (20, 12.5) on an XY workplane at height `z`.
    fn through_all_z(z: f64, dir: ThroughDirection) -> Result<OcctShape, KernelError> {
        let plane = Workplane {
            origin: [0.0, 0.0, z],
            ..Workplane::XY
        };
        extrude_feature(
            &K,
            &circle_on(plane, [20.0, 12.5]),
            Extent::ThroughAll(dir),
            ExtrudeMode::Cut,
            Some(&tall_block()),
        )
    }

    #[test]
    fn through_all_reversed_from_the_top_face() {
        let r = through_all_z(100.0, ThroughDirection::Reversed).unwrap();
        assert_close(K.volume(&r).unwrap(), 100000.0 - 2500.0 * PI);
        assert_eq!(K.face_count(&r).unwrap(), 7);
    }

    #[test]
    fn through_all_reversed_from_above_the_body() {
        let r = through_all_z(110.0, ThroughDirection::Reversed).unwrap();
        assert_close(K.volume(&r).unwrap(), 100000.0 - 2500.0 * PI);
        assert_eq!(K.face_count(&r).unwrap(), 7);
    }

    #[test]
    fn through_all_both_from_mid_height() {
        let r = through_all_z(50.0, ThroughDirection::Both).unwrap();
        assert_close(K.volume(&r).unwrap(), 100000.0 - 2500.0 * PI);
        assert_eq!(K.face_count(&r).unwrap(), 7);
    }

    #[test]
    fn through_all_normal_from_mid_height_is_a_blind_hole_to_the_top() {
        let r = through_all_z(50.0, ThroughDirection::Normal).unwrap();
        assert_close(K.volume(&r).unwrap(), 100000.0 - 1250.0 * PI);
        assert_eq!(K.face_count(&r).unwrap(), 8);
    }

    #[test]
    fn through_all_along_x_on_the_yz_plane() {
        let r = extrude_feature(
            &K,
            &circle_on(Workplane::YZ, [12.5, 50.0]),
            Extent::ThroughAll(ThroughDirection::Normal),
            ExtrudeMode::Cut,
            Some(&tall_block()),
        )
        .unwrap();
        assert_close(K.volume(&r).unwrap(), 100000.0 - 1000.0 * PI);
        assert_eq!(K.face_count(&r).unwrap(), 7);
    }

    #[test]
    fn through_all_cut_that_misses_is_an_error() {
        let err = through_all_z(100.0, ThroughDirection::Normal).unwrap_err();
        assert!(
            matches!(&err, KernelError::InvalidInput(m) if m.contains("misses the target")),
            "{err}"
        );
    }

    #[test]
    fn through_all_with_other_modes_is_an_error() {
        let t = tall_block();
        let face = circle_on(Workplane::XY, [20.0, 12.5]);
        for mode in [
            ExtrudeMode::NewSolid,
            ExtrudeMode::Join,
            ExtrudeMode::Intersect,
        ] {
            let err = extrude_feature(
                &K,
                &face,
                Extent::ThroughAll(ThroughDirection::Normal),
                mode,
                Some(&t),
            )
            .unwrap_err();
            assert!(
                matches!(&err, KernelError::InvalidInput(m) if m.contains("only with the cut mode")),
                "{mode:?}: {err}"
            );
        }
    }

    #[test]
    fn kernel_extrude_rejects_through_all() {
        let face = circle_on(Workplane::XY, [20.0, 12.5]);
        let err = K
            .extrude(&face, Extent::ThroughAll(ThroughDirection::Both))
            .unwrap_err();
        assert!(
            matches!(&err, KernelError::InvalidInput(m) if m.contains("through-all")),
            "{err}"
        );
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
