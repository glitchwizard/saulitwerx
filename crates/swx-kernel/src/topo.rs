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
    fn codes_are_distinct() {
        let codes = [TopoKind::Face, TopoKind::Edge, TopoKind::Vertex].map(TopoKind::code);
        assert_eq!(codes, [0, 1, 2]);
    }
}
