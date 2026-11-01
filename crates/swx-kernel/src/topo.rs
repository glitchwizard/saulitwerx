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
    fn codes_are_distinct() {
        let codes = [TopoKind::Face, TopoKind::Edge, TopoKind::Vertex].map(TopoKind::code);
        assert_eq!(codes, [0, 1, 2]);
    }
}
