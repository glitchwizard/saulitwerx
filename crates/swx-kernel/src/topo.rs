//! Sub-shapes of a solid: its faces, edges and vertices, addressed by index.
//!
//! See the Kernel API section of `docs/specs/M3-04.md`. An index is a 0-based
//! position among the distinct sub-shapes of one kind in one shape (OCCT:
//! `TopExp::MapShapes`), so it is valid only for the shape it came from.

use crate::KernelError;

/// The kind of a sub-shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TopoKind {
    /// A bounded piece of surface.
    Face,
    /// A bounded piece of curve where faces meet.
    Edge,
    /// A point where edges meet.
    Vertex,
}

impl TopoKind {
    /// The code the C shim uses for this kind.
    #[cfg_attr(not(feature = "occt"), allow(dead_code))]
    pub(crate) fn code(self) -> i32 {
        match self {
            TopoKind::Face => 0,
            TopoKind::Edge => 1,
            TopoKind::Vertex => 2,
        }
    }

    fn name(self) -> &'static str {
        match self {
            TopoKind::Face => "face",
            TopoKind::Edge => "edge",
            TopoKind::Vertex => "vertex",
        }
    }
}

/// One face, edge or vertex of a shape: the `index`-th distinct sub-shape of
/// its `kind`, counted from 0. Valid only for the shape it was taken from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubShape {
    /// Face, edge or vertex.
    pub kind: TopoKind,
    /// 0-based position among the shape's sub-shapes of `kind`.
    pub index: usize,
}

/// Where the faces of an extruded solid came from, as returned by
/// [`Kernel::extrude_tracked`](crate::Kernel::extrude_tracked). Every
/// [`SubShape`] here is a face of the result, except the first of each
/// `sides` pair, which is an edge of the input face.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtrudeHistory {
    /// The face the sweep starts from: the input face, moved by the extent's offset.
    pub start_cap: SubShape,
    /// The face where the sweep ends: the start cap translated by the length.
    pub end_cap: SubShape,
    /// Edge of the input face → the side face it swept, one pair per edge of
    /// the input face, in the input face's edge index order.
    pub sides: Vec<(SubShape, SubShape)>,
}

/// A boolean operation between two solids, for
/// [`Kernel::boolean_tracked`](crate::Kernel::boolean_tracked).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOp {
    /// Material of either solid (`Kernel::union`).
    Union = 0,
    /// Material of `a` not in `b` (`Kernel::cut`).
    Cut = 1,
    /// Material of both solids (`Kernel::intersect`).
    Intersect = 2,
}

/// Which input of an operation a sub-shape belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    /// The target, or the only input.
    A,
    /// The tool body.
    B,
}

/// Per-input images and generated sub-shapes of one operation, as returned
/// by [`Kernel::boolean_tracked`](crate::Kernel::boolean_tracked).
///
/// The first [`SubShape`] of each entry belongs to the input named by its
/// [`Operand`]; the listed ones are sub-shapes of the result, sorted by index.
/// Version 1 tracks faces only (edges are named by their faces,
/// `docs/specs/M3-04.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpHistory {
    /// Input sub-shape → its images in the result: itself if unchanged,
    /// several if it was split, none if it was deleted.
    pub modified: Vec<(Operand, SubShape, Vec<SubShape>)>,
    /// Input sub-shape → result sub-shapes generated from it. A boolean
    /// generates no faces (only section edges), so it is empty for booleans
    /// while only faces are tracked.
    pub generated: Vec<(Operand, SubShape, Vec<SubShape>)>,
}

/// Checks an adjacency query before it reaches the backend: `kind` must
/// differ from `of.kind`, and `of.index` must be below `count`, the number
/// of sub-shapes of `of.kind` in the shape.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn validate_adjacent(
    of: SubShape,
    kind: TopoKind,
    count: usize,
) -> Result<(), KernelError> {
    if of.kind == kind {
        return Err(KernelError::InvalidInput(format!(
            "adjacent needs two different kinds, got {} and {}",
            of.kind.name(),
            kind.name()
        )));
    }
    validate_index(of, count)
}

/// Checks that `of.index` is below `count`, the number of sub-shapes of
/// `of.kind` in the shape.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn validate_index(of: SubShape, count: usize) -> Result<(), KernelError> {
    if of.index >= count {
        return Err(KernelError::InvalidInput(format!(
            "{} index {} is out of range: the shape has {count} {}s",
            of.kind.name(),
            of.index,
            of.kind.name()
        )));
    }
    Ok(())
}

/// Checks an edge list for a fillet or chamfer before it reaches the
/// backend: it must not be empty, every entry must be an edge with an index
/// below `count` (the shape's edge count). Returns the indices sorted, with
/// duplicates removed.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn validate_edges(edges: &[SubShape], count: usize) -> Result<Vec<usize>, KernelError> {
    validate_list(edges, TopoKind::Edge, count)
}

/// Checks a face list (e.g. the open faces of a shell) the same way as
/// [`validate_edges`]: not empty, every entry a face with an index below
/// `count` (the shape's face count). Returns the indices sorted, with
/// duplicates removed.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn validate_faces(faces: &[SubShape], count: usize) -> Result<Vec<usize>, KernelError> {
    validate_list(faces, TopoKind::Face, count)
}

fn validate_list(
    items: &[SubShape],
    kind: TopoKind,
    count: usize,
) -> Result<Vec<usize>, KernelError> {
    let name = kind.name();
    if items.is_empty() {
        return Err(KernelError::InvalidInput(format!(
            "the {name} list is empty"
        )));
    }
    let article = if name.starts_with(['a', 'e', 'i', 'o', 'u']) {
        "an"
    } else {
        "a"
    };
    let mut indices = Vec::with_capacity(items.len());
    for it in items {
        if it.kind != kind {
            return Err(KernelError::InvalidInput(format!(
                "expected {article} {name}, got {} {}",
                it.kind.name(),
                it.index
            )));
        }
        if it.index >= count {
            return Err(KernelError::InvalidInput(format!(
                "{name} index {} is out of range: the shape has {count} {name}s",
                it.index
            )));
        }
        indices.push(it.index);
    }
    indices.sort_unstable();
    indices.dedup();
    Ok(indices)
}

/// An edge of a shape together with one of the two faces it bounds: the
/// face an asymmetric chamfer measures its first setback on
/// ([`Kernel::chamfer_asymmetric`](crate::Kernel::chamfer_asymmetric)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EdgeOnFace {
    /// The edge to bevel.
    pub edge: SubShape,
    /// The reference face: one of the two faces that meet at `edge`.
    pub face: SubShape,
}

/// The size of an asymmetric chamfer, measured from the edge.
///
/// Convention: `d1` and `distance` are the setback measured on the
/// **reference face** ([`EdgeOnFace::face`]); `d2` is the setback on the
/// other face of the edge. `angle` (radians) is the angle between the new
/// chamfer face and the reference face, so where the two faces meet at a
/// right angle the setback on the other face is `distance · tan(angle)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChamferSize {
    /// Setback `d1` on the reference face and `d2` on the other face.
    TwoDistances {
        /// Setback on the reference face; finite and > 0.
        d1: f64,
        /// Setback on the other face; finite and > 0.
        d2: f64,
    },
    /// Setback `distance` on the reference face, and the chamfer face at
    /// `angle` to the reference face.
    DistanceAngle {
        /// Setback on the reference face; finite and > 0.
        distance: f64,
        /// Radians, strictly between 0 and π/2.
        angle: f64,
    },
}

impl ChamferSize {
    /// Rejects a distance that is not finite or ≤ 0, and an angle that is
    /// not finite or not strictly between 0 and π/2.
    #[cfg_attr(not(feature = "occt"), allow(dead_code))]
    pub(crate) fn validate(self) -> Result<(), KernelError> {
        match self {
            ChamferSize::TwoDistances { d1, d2 } => {
                crate::validate_size("chamfer d1", d1)?;
                crate::validate_size("chamfer d2", d2)
            }
            ChamferSize::DistanceAngle { distance, angle } => {
                crate::validate_size("chamfer distance", distance)?;
                if !angle.is_finite() || angle <= 0.0 || angle >= std::f64::consts::FRAC_PI_2 {
                    return Err(KernelError::InvalidInput(format!(
                        "chamfer angle must be finite and strictly between 0 and π/2, got {angle}"
                    )));
                }
                Ok(())
            }
        }
    }
}

/// Checks an edge/face list for an asymmetric chamfer before it reaches the
/// backend: it must not be empty; every `edge` must be an edge below
/// `edge_count` and every `face` a face below `face_count`; an identical
/// entry listed twice counts once, but the same edge with two different
/// faces is an error. Returns `(edge, face)` index pairs sorted by edge.
/// Whether each face actually bounds its edge is checked by the caller.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn validate_edges_on_faces(
    list: &[EdgeOnFace],
    edge_count: usize,
    face_count: usize,
) -> Result<Vec<(usize, usize)>, KernelError> {
    if list.is_empty() {
        return Err(KernelError::InvalidInput(
            "the edge list is empty".to_string(),
        ));
    }
    let mut pairs = Vec::with_capacity(list.len());
    for p in list {
        for (s, kind, count) in [
            (p.edge, TopoKind::Edge, edge_count),
            (p.face, TopoKind::Face, face_count),
        ] {
            if s.kind != kind {
                return Err(KernelError::InvalidInput(format!(
                    "expected {} {}, got {} {}",
                    if kind == TopoKind::Edge { "an" } else { "a" },
                    kind.name(),
                    s.kind.name(),
                    s.index
                )));
            }
            validate_index(s, count)?;
        }
        pairs.push((p.edge.index, p.face.index));
    }
    pairs.sort_unstable();
    pairs.dedup();
    if let Some(w) = pairs.windows(2).find(|w| w[0].0 == w[1].0) {
        return Err(KernelError::InvalidInput(format!(
            "edge {} is listed with two different faces ({} and {})",
            w[0].0, w[0].1, w[1].1
        )));
    }
    Ok(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_kind_and_out_of_range_are_invalid() {
        let e3 = SubShape {
            kind: TopoKind::Edge,
            index: 3,
        };
        assert!(validate_adjacent(e3, TopoKind::Face, 12).is_ok());
        let err = validate_adjacent(e3, TopoKind::Edge, 12).unwrap_err();
        assert!(
            matches!(&err, KernelError::InvalidInput(m) if m.contains("two different kinds")),
            "{err}"
        );
        let err = validate_adjacent(e3, TopoKind::Face, 3).unwrap_err();
        assert!(
            matches!(&err, KernelError::InvalidInput(m) if m.contains("edge index 3 is out of range")),
            "{err}"
        );
    }

    #[test]
    fn edge_lists_are_checked_and_deduplicated() {
        let e = |index| SubShape {
            kind: TopoKind::Edge,
            index,
        };
        assert_eq!(validate_edges(&[e(3), e(1), e(3)], 12).unwrap(), vec![1, 3]);
        let invalid =
            |r: Result<Vec<usize>, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        assert!(invalid(validate_edges(&[], 12)));
        assert!(invalid(validate_edges(&[e(12)], 12)));
        let face = SubShape {
            kind: TopoKind::Face,
            index: 0,
        };
        assert!(invalid(validate_edges(&[e(0), face], 12)));
    }

    #[test]
    fn face_lists_are_checked_and_deduplicated() {
        let f = |index| SubShape {
            kind: TopoKind::Face,
            index,
        };
        assert_eq!(validate_faces(&[f(5), f(2), f(5)], 6).unwrap(), vec![2, 5]);
        let invalid =
            |r: Result<Vec<usize>, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        assert!(invalid(validate_faces(&[], 6)));
        assert!(invalid(validate_faces(&[f(6)], 6)));
        let edge = SubShape {
            kind: TopoKind::Edge,
            index: 0,
        };
        assert!(invalid(validate_faces(&[f(0), edge], 6)));
    }

    #[test]
    fn chamfer_sizes_are_checked() {
        use std::f64::consts::FRAC_PI_2;
        let two = |d1, d2| ChamferSize::TwoDistances { d1, d2 }.validate();
        let da = |distance, angle| ChamferSize::DistanceAngle { distance, angle }.validate();
        assert!(two(2.0, 4.0).is_ok());
        assert!(da(2.0, FRAC_PI_2 / 3.0).is_ok());
        let invalid = |r: Result<(), KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(invalid(two(bad, 4.0)), "d1 {bad}");
            assert!(invalid(two(2.0, bad)), "d2 {bad}");
            assert!(invalid(da(bad, 0.5)), "distance {bad}");
        }
        for bad in [0.0, -0.1, FRAC_PI_2, 2.0, f64::NAN, f64::INFINITY] {
            assert!(invalid(da(2.0, bad)), "angle {bad}");
        }
    }

    #[test]
    fn edge_face_lists_are_checked_and_deduplicated() {
        let sub = |kind, index| SubShape { kind, index };
        let p = |e, f| EdgeOnFace {
            edge: sub(TopoKind::Edge, e),
            face: sub(TopoKind::Face, f),
        };
        assert_eq!(
            validate_edges_on_faces(&[p(3, 1), p(1, 0), p(3, 1)], 12, 6).unwrap(),
            vec![(1, 0), (3, 1)]
        );
        let invalid = |r: Result<Vec<(usize, usize)>, KernelError>| {
            matches!(r, Err(KernelError::InvalidInput(_)))
        };
        assert!(invalid(validate_edges_on_faces(&[], 12, 6)));
        assert!(invalid(validate_edges_on_faces(&[p(12, 0)], 12, 6)));
        assert!(invalid(validate_edges_on_faces(&[p(0, 6)], 12, 6)));
        let err = validate_edges_on_faces(&[p(3, 1), p(3, 2)], 12, 6).unwrap_err();
        assert!(
            matches!(&err, KernelError::InvalidInput(m) if m.contains("two different faces")),
            "{err}"
        );
        for wrong in [sub(TopoKind::Face, 0), sub(TopoKind::Vertex, 0)] {
            let e = EdgeOnFace {
                edge: wrong,
                face: sub(TopoKind::Face, 0),
            };
            assert!(invalid(validate_edges_on_faces(&[e], 12, 6)));
        }
        for wrong in [sub(TopoKind::Edge, 0), sub(TopoKind::Vertex, 0)] {
            let e = EdgeOnFace {
                edge: sub(TopoKind::Edge, 0),
                face: wrong,
            };
            assert!(invalid(validate_edges_on_faces(&[e], 12, 6)));
        }
    }

    #[test]
    fn codes_are_distinct() {
        let codes = [TopoKind::Face, TopoKind::Edge, TopoKind::Vertex].map(TopoKind::code);
        assert_eq!(codes, [0, 1, 2]);
    }
}
