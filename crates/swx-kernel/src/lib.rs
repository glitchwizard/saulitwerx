//! Saulitwerx geometry kernel.
//!
//! [`Kernel`] is the backend-neutral interface the rest of Saulitwerx uses for
//! solid modeling. Its public signatures carry no backend types, so a
//! different kernel can be swapped in later.
//!
//! The B-rep backend is OpenCascade (OCCT), enabled by the `occt` cargo
//! feature (off by default) as [`OcctKernel`]. OCCT is reached through a small
//! C ABI shim in `cpp/occt_shim.cpp` and linked dynamically against the system
//! libraries; see the M1-01 ADR in `docs/ARCHITECTURE.md`.

use std::ffi::CString;
use std::fmt;
use std::path::Path;

mod datum;
mod feature;
mod hole;
mod loft;
mod mesh;
mod profile;
mod topo;

#[cfg(feature = "occt")]
mod ffi;
#[cfg(feature = "occt")]
mod occt;

pub use datum::{plane_axis_intersection, DATUM_TOLERANCE, ORIGIN};
pub use feature::{
    extrude_feature, revolve_feature, ExtrudeMode, EMPTY_VOLUME_TOLERANCE, FACE_DISTANCE_TOLERANCE,
    PARALLEL_FACE_TOLERANCE,
};
pub use hole::{HoleDepth, HoleKind};
pub use loft::MIN_LOFT_SECTIONS;
pub use mesh::Mesh;
#[cfg(feature = "occt")]
pub use occt::{OcctKernel, OcctShape};
pub use profile::{
    Axis, Extent, ProfileLoop, Segment2, ThroughDirection, Workplane, LOOP_GAP_TOLERANCE,
    PERPENDICULAR_TOLERANCE, REVOLVE_AXIS_TOLERANCE,
};
pub use topo::{
    BooleanOp, ChamferSize, EdgeOnFace, ExtrudeHistory, OpHistory, Operand, SubShape, TopoKind,
};

/// A point or vector in model space, `[x, y, z]`.
pub type Point3 = [f64; 3];

/// Errors reported by kernel operations.
#[derive(Debug, Clone, PartialEq)]
pub enum KernelError {
    /// An input was rejected before reaching the kernel.
    InvalidInput(String),
    /// The kernel itself failed (e.g. an OCCT exception).
    Backend(String),
    /// The operation succeeded but left no solid (e.g. a cut that removes
    /// the whole body, or an intersect with no overlap).
    EmptyResult(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            KernelError::Backend(msg) => write!(f, "kernel failure: {msg}"),
            KernelError::EmptyResult(msg) => write!(f, "empty result: {msg}"),
        }
    }
}

impl std::error::Error for KernelError {}

/// Axis-aligned bounding box.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// Minimum corner.
    pub min: Point3,
    /// Maximum corner.
    pub max: Point3,
}

/// Backend-neutral solid modeling operations.
///
/// Constructors validate their inputs before reaching the backend: sizes must
/// be finite and strictly positive, positions must be finite.
pub trait Kernel {
    /// The backend's solid shape. Opaque to callers.
    type Shape;

    /// Axis-aligned box with its minimum corner at `corner` and edge lengths `size`.
    fn make_box(&self, corner: Point3, size: Point3) -> Result<Self::Shape, KernelError>;
    /// Cylinder along +Z whose base circle is centered at `base`.
    fn make_cylinder(
        &self,
        base: Point3,
        radius: f64,
        height: f64,
    ) -> Result<Self::Shape, KernelError>;
    /// Sphere centered at `center`.
    fn make_sphere(&self, center: Point3, radius: f64) -> Result<Self::Shape, KernelError>;

    /// Planar face on `plane` bounded by `outer`, with `holes` cut out.
    ///
    /// Loops may be given in either direction. It is an error, never a panic,
    /// when a loop is empty or doesn't close within [`LOOP_GAP_TOLERANCE`],
    /// when a number isn't finite or a radius isn't > 0, when `plane.x_dir`
    /// and `plane.normal` are zero or not perpendicular within
    /// [`PERPENDICULAR_TOLERANCE`], or when the backend finds the face
    /// invalid (e.g. a self-intersecting loop or a hole outside the outer loop).
    fn make_face(
        &self,
        plane: &Workplane,
        outer: &ProfileLoop,
        holes: &[ProfileLoop],
    ) -> Result<Self::Shape, KernelError>;
    /// Solid swept from a planar `face` along its plane normal by `extent`.
    ///
    /// A zero or non-finite length is an error.
    fn extrude(&self, face: &Self::Shape, extent: Extent) -> Result<Self::Shape, KernelError>;
    /// [`Kernel::make_face`], plus where each segment went: for each loop
    /// (`outer` first, then `holes` in order) and each of its segments in the
    /// caller's order, the face's edge built from it. Same inputs and errors
    /// as `make_face`.
    fn make_face_tracked(
        &self,
        plane: &Workplane,
        outer: &ProfileLoop,
        holes: &[ProfileLoop],
    ) -> Result<(Self::Shape, Vec<Vec<SubShape>>), KernelError>;
    /// [`Kernel::extrude`], plus its [`ExtrudeHistory`]: the start and end
    /// caps and, for every edge of `face`, the side face it swept. The same
    /// input checks as `extrude` apply; [`Extent::ThroughAll`],
    /// [`Extent::ToFace`] and [`Extent::BetweenFaces`] are
    /// [`KernelError::InvalidInput`], since they need a target body.
    fn extrude_tracked(
        &self,
        face: &Self::Shape,
        extent: Extent,
    ) -> Result<(Self::Shape, ExtrudeHistory), KernelError>;
    /// Solid swept by turning a planar `face` `angle` radians about `axis`.
    ///
    /// `axis.dir` is normalized first; a positive angle turns counterclockwise
    /// looking down it (right-hand rule). It is [`KernelError::InvalidInput`],
    /// checked before the backend, when the angle is not finite, is ≤ 0 or is
    /// above 2π (2π itself, within 1e-12, is a full turn), or when the axis
    /// origin is not finite or its direction is zero or not finite. It is
    /// also `InvalidInput`, checked before the revolve itself, when `face` is
    /// not a single planar face, when the axis is not in the face's plane
    /// (its direction is not perpendicular to the plane normal, or its origin
    /// is off the plane), or when the face crosses the axis (has points
    /// strictly on both sides of it). Each check allows
    /// [`REVOLVE_AXIS_TOLERANCE`]; a face that touches the axis is valid.
    fn revolve(
        &self,
        face: &Self::Shape,
        axis: Axis,
        angle: f64,
    ) -> Result<Self::Shape, KernelError>;
    /// Closed solid swept by moving the planar `profile` face along the open
    /// `path` of lines and arcs drawn on `plane`.
    ///
    /// The path runs from the first segment's start (a `Line`'s `start`, an
    /// `Arc`'s point at `start_angle`) through the segments in order; each
    /// next segment must begin where the previous one ends, within
    /// [`LOOP_GAP_TOLERANCE`] (either of its endpoints may meet it). The
    /// path's start must lie in the profile's plane (within the same
    /// tolerance). The profile keeps the backend's default orientation along
    /// the path. Method: OCCT reference documentation,
    /// `BRepOffsetAPI_MakePipe` (spine = the path as one wire).
    ///
    /// It is [`KernelError::InvalidInput`], checked before the backend, when
    /// the path is empty or contains a `Circle` (closed paths are out of
    /// scope), when a radius is ≤ 0 or a number is not finite, when `plane`
    /// is invalid (as for [`Kernel::make_face`]), when consecutive segments
    /// don't meet, when the path's start is off the profile's plane, when
    /// `profile` is not a single planar face, when it has a hole, or when an
    /// arc of the path has a radius ≤ the profile's largest distance from
    /// the path's start, measured in the profile's plane (the bend would
    /// fold the solid through itself; the message names the arc's segment
    /// index, its radius and that distance). A sweep
    /// the backend can't build (e.g. a path that starts along the profile's
    /// plane), or a result it finds invalid or empty, is an error. `profile`
    /// is unchanged either way.
    fn sweep(
        &self,
        profile: &Self::Shape,
        plane: &Workplane,
        path: &[Segment2],
    ) -> Result<Self::Shape, KernelError>;

    /// Rounds `edges` of `s` with a constant-radius fillet.
    ///
    /// Duplicate edges count once. It is [`KernelError::InvalidInput`],
    /// checked before the backend, when `radius` is not finite or ≤ 0, when
    /// `edges` is empty, or when an entry is not an edge or its index is out
    /// of range. A radius the backend can't build (e.g. larger than a face it
    /// touches) is an error, and `s` is unchanged either way.
    fn fillet(
        &self,
        s: &Self::Shape,
        edges: &[SubShape],
        radius: f64,
    ) -> Result<Self::Shape, KernelError>;
    /// Rounds one `edge` of `s` with a variable-radius fillet: the radius is
    /// `r_start` at `start`, a vertex of `edge`, and `r_end` at the edge's
    /// other vertex. Between the two ends the radius follows a linear
    /// evolution law along the edge (OCCT reference documentation,
    /// `BRepFilletAPI_MakeFillet::SetRadius(R1, R2, IC, IinC)`).
    ///
    /// It is [`KernelError::InvalidInput`], checked before the backend, when
    /// a radius is not finite or ≤ 0, when `edge` is not an edge or `start`
    /// not a vertex, when an index is out of range, or when `start` is not a
    /// vertex of `edge`. A radius the backend can't build, or an edge that
    /// continues tangentially into others (the backend would round the whole
    /// tangent chain), is an error, and `s` is unchanged either way. The
    /// larger of the two radii must also build as a constant fillet of
    /// `edge` ([`Kernel::fillet`]); otherwise it is an error.
    fn fillet_variable(
        &self,
        s: &Self::Shape,
        edge: SubShape,
        start: SubShape,
        r_start: f64,
        r_end: f64,
    ) -> Result<Self::Shape, KernelError>;
    /// Bevels `edges` of `s` with an equal-distance chamfer: `distance` is
    /// set back along both faces of each edge. Same checks as
    /// [`Kernel::fillet`].
    fn chamfer(
        &self,
        s: &Self::Shape,
        edges: &[SubShape],
        distance: f64,
    ) -> Result<Self::Shape, KernelError>;
    /// Bevels `edges` of `s` with a chamfer whose two setbacks may differ.
    ///
    /// Each entry names an edge and its reference face; `size` follows the
    /// convention on [`ChamferSize`]: the first distance is measured on the
    /// reference face, the second (or the one the angle gives) on the other
    /// face of the edge. An identical entry listed twice counts once.
    ///
    /// It is [`KernelError::InvalidInput`], checked before the chamfer
    /// reaches the backend, when a distance is not finite or ≤ 0, when the
    /// angle is not finite or not strictly between 0 and π/2, when `edges`
    /// is empty, when an `edge` is not an edge or a `face` not a face, when
    /// an index is out of range, when a face does not bound its edge, or
    /// when the same edge is listed with two different faces. A size the
    /// backend can't build is an error, and `s` is unchanged either way.
    fn chamfer_asymmetric(
        &self,
        s: &Self::Shape,
        edges: &[EdgeOnFace],
        size: ChamferSize,
    ) -> Result<Self::Shape, KernelError>;

    /// Hollows `s` into a shell with walls of constant `thickness`, leaving
    /// the `open` faces open.
    ///
    /// The walls grow inward from the original faces, so the outer size
    /// (bounding box) is unchanged; corners of the inner cavity are joined
    /// with arcs where the offset needs them. Duplicate faces count once. It
    /// is [`KernelError::InvalidInput`], checked before the backend, when
    /// `thickness` is not finite or ≤ 0, when `open` is empty, or when an
    /// entry is not a face or its index is out of range. A thickness the
    /// backend can't build (e.g. half the part's width or more) is an error,
    /// and `s` is unchanged either way.
    fn shell(
        &self,
        s: &Self::Shape,
        open: &[SubShape],
        thickness: f64,
    ) -> Result<Self::Shape, KernelError>;

    /// Drafts (tilts) the `faces` of `s` by a constant `angle` in radians.
    ///
    /// The neutral plane passes through `neutral.origin` and is normal to
    /// `neutral.dir` (normalized first), which is the pull direction. Each
    /// face turns about its line on the neutral plane, which stays where it
    /// is. A positive `angle` tilts each face so the solid gets smaller
    /// moving along the pull direction; a negative one makes it larger.
    /// Duplicate faces count once. It is [`KernelError::InvalidInput`],
    /// checked before the backend, when `faces` is empty, an entry is not a
    /// face or its index is out of range, `neutral.dir` is zero-length or not
    /// finite, `neutral.origin` is not finite, or `angle` is zero, not finite
    /// or has |angle| ≥ π/2. A face the backend can't draft (e.g. one
    /// parallel to the neutral plane) is an error, and `s` is unchanged
    /// either way.
    fn draft(
        &self,
        s: &Self::Shape,
        faces: &[SubShape],
        neutral: Axis,
        angle: f64,
    ) -> Result<Self::Shape, KernelError>;

    /// Removes a hole of `kind` and `depth` from `s`.
    ///
    /// `at.origin` is the hole's center on the start face and `at.dir`
    /// (normalized first) points into the material. A blind depth, and a
    /// counterbore's depth, are measured from `at.origin` along the drill
    /// direction to a flat bottom (no drill point). [`HoleDepth::ThroughAll`]
    /// cuts through the whole solid along the direction from `at.origin`.
    ///
    /// It is [`KernelError::InvalidInput`], checked before the backend, when
    /// a diameter, blind depth, `cb_diameter` or `cb_depth` is not finite or
    /// ≤ 0, when `cb_diameter` ≤ `diameter`, when a blind depth ≤ `cb_depth`,
    /// when `at.dir` is zero-length or not finite, or when `at.origin` is not
    /// finite. A hole that removes nothing (its axis misses the solid) is
    /// `InvalidInput` too, and a result the backend finds invalid is an
    /// error. `s` is unchanged either way.
    fn hole(
        &self,
        s: &Self::Shape,
        at: Axis,
        kind: HoleKind,
        depth: HoleDepth,
    ) -> Result<Self::Shape, KernelError>;

    /// Closed solid lofted through the outer loops of the planar section
    /// faces `sections` (each made with [`Kernel::make_face`]), in the order
    /// given.
    ///
    /// With `ruled`, each pair of neighboring sections is joined by
    /// straight-line (ruled) surfaces; otherwise one smooth surface passes
    /// through all sections. The ends are capped with the first and last
    /// sections. Method: OCCT reference documentation,
    /// `BRepOffsetAPI_ThruSections` (solid mode, compatibility check on, one
    /// wire per section from `BRepTools::OuterWire`).
    ///
    /// It is [`KernelError::InvalidInput`], checked before the backend, when
    /// there are fewer than [`MIN_LOFT_SECTIONS`] sections, when a section is
    /// not a single planar face (e.g. a solid), or when a section face has a
    /// hole. Sections the backend can't loft (e.g. two equal sections in the
    /// same plane), or a result it finds invalid or empty, are an error. The
    /// sections are unchanged either way.
    fn loft(&self, sections: &[Self::Shape], ruled: bool) -> Result<Self::Shape, KernelError>;

    /// Boolean union `a ∪ b`.
    fn union(&self, a: &Self::Shape, b: &Self::Shape) -> Result<Self::Shape, KernelError>;
    /// Boolean difference `a − b`.
    fn cut(&self, a: &Self::Shape, b: &Self::Shape) -> Result<Self::Shape, KernelError>;
    /// Boolean intersection `a ∩ b`.
    fn intersect(&self, a: &Self::Shape, b: &Self::Shape) -> Result<Self::Shape, KernelError>;
    /// The same solid as [`Kernel::union`], [`Kernel::cut`] or
    /// [`Kernel::intersect`] (by `op`), plus its [`OpHistory`].
    ///
    /// `modified` has exactly one entry per face of `a` (as [`Operand::A`])
    /// and then per face of `b` ([`Operand::B`]), each in face index order,
    /// with its images in the result: itself if unchanged, several if split,
    /// none if deleted. `generated` is empty (faces only, see [`OpHistory`]).
    /// Errors are those of the untracked call.
    fn boolean_tracked(
        &self,
        op: BooleanOp,
        a: &Self::Shape,
        b: &Self::Shape,
    ) -> Result<(Self::Shape, OpHistory), KernelError>;

    /// Number of distinct solid bodies in `s` (0 for a lone face).
    fn body_count(&self, s: &Self::Shape) -> Result<usize, KernelError>;

    /// Each solid body of `s` as its own shape, sorted by bounding-box
    /// minimum (x, then y, then z). A shape with no solids (a lone face)
    /// gives an empty list. `s` is unchanged.
    fn bodies(&self, s: &Self::Shape) -> Result<Vec<Self::Shape>, KernelError>;

    /// Combines `target` with each of `tools` in turn by `op`: the same
    /// result as `op(op(target, tools[0]), tools[1])…`, in order. The inputs
    /// are unchanged.
    ///
    /// No tools, or a target or tool with no solid body (e.g. a lone face),
    /// is [`KernelError::InvalidInput`], checked before the backend. A
    /// result that encloses no volume (at most [`EMPTY_VOLUME_TOLERANCE`]),
    /// such as an intersect with no common volume, is
    /// [`KernelError::EmptyResult`].
    fn combine(
        &self,
        op: BooleanOp,
        target: &Self::Shape,
        tools: &[Self::Shape],
    ) -> Result<Self::Shape, KernelError>;

    /// Enclosed volume.
    fn volume(&self, s: &Self::Shape) -> Result<f64, KernelError>;
    /// Total area of all faces.
    fn surface_area(&self, s: &Self::Shape) -> Result<f64, KernelError>;
    /// Tight axis-aligned bounding box.
    fn bounding_box(&self, s: &Self::Shape) -> Result<BoundingBox, KernelError>;

    /// Triangulates the boundary of `s` into a [`Mesh`].
    ///
    /// `deflection` is the maximum distance between the mesh and the exact
    /// surface, in model units; it must be finite and strictly positive.
    fn tessellate(&self, s: &Self::Shape, deflection: f64) -> Result<Mesh, KernelError>;

    /// Number of distinct faces in `s`.
    fn face_count(&self, s: &Self::Shape) -> Result<usize, KernelError>;

    /// Number of distinct sub-shapes of `kind` in `s` (shared ones counted once).
    ///
    /// Valid [`SubShape`] indices for `kind` are `0..count`.
    fn sub_shape_count(&self, s: &Self::Shape, kind: TopoKind) -> Result<usize, KernelError>;
    /// The sub-shapes of `kind` adjacent to `of` in `s`, sorted by index,
    /// without duplicates.
    ///
    /// Going up, it returns the sub-shapes that contain `of` (faces of an
    /// edge, edges or faces of a vertex); going down, the ones `of` contains
    /// (edges or vertices of a face, vertices of an edge). `of.kind == kind`
    /// and an `of.index` out of range for its kind are
    /// [`KernelError::InvalidInput`], checked before anything reaches the
    /// backend.
    fn adjacent(
        &self,
        s: &Self::Shape,
        of: SubShape,
        kind: TopoKind,
    ) -> Result<Vec<SubShape>, KernelError>;

    /// The plane of the planar face `face` of `s`: a point on it and its unit
    /// outward normal (pointing out of the solid; for a lone face, the
    /// direction a positive [`Kernel::extrude`] distance goes).
    ///
    /// A `face` whose kind isn't [`TopoKind::Face`], an index out of range
    /// and a non-planar face are [`KernelError::InvalidInput`].
    fn face_plane(&self, s: &Self::Shape, face: SubShape) -> Result<(Point3, Point3), KernelError>;

    /// A work plane on the planar face `face` of `s`: `normal` is the face's
    /// outward normal (as [`Kernel::face_plane`]), `origin` is the projection
    /// of the model origin onto the plane, and `x_dir` follows
    /// [`Workplane::from_point_normal`].
    ///
    /// A `face` whose kind isn't [`TopoKind::Face`], an index out of range
    /// and a non-planar face are [`KernelError::InvalidInput`].
    fn work_plane_on_face(&self, s: &Self::Shape, face: SubShape)
        -> Result<Workplane, KernelError>;

    /// A work axis on `of`: the line of a straight edge, or the axis of a
    /// cylindrical face. `dir` is a unit vector (either sign); `origin` is
    /// the point of the line closest to the model origin.
    ///
    /// A vertex, an index out of range, a curved edge and a face that isn't
    /// cylindrical are [`KernelError::InvalidInput`].
    fn work_axis_on(&self, s: &Self::Shape, of: SubShape) -> Result<Axis, KernelError>;

    /// The position of the vertex `vertex` of `s`.
    ///
    /// A `vertex` whose kind isn't [`TopoKind::Vertex`] and an index out of
    /// range are [`KernelError::InvalidInput`].
    fn vertex_point(&self, s: &Self::Shape, vertex: SubShape) -> Result<Point3, KernelError>;

    /// A work plane tangent to the cylindrical face `face` of `s`, on the
    /// side `toward` points to. Only the part of `toward` perpendicular to
    /// the cylinder's axis is used; normalized, it is the plane's `normal`.
    /// `origin` is the projection of the model origin onto the plane, and
    /// `x_dir` follows [`Workplane::from_point_normal`].
    ///
    /// A `face` whose kind isn't [`TopoKind::Face`], an index out of range,
    /// a face that isn't cylindrical, and a `toward` that is zero,
    /// non-finite or parallel to the axis are [`KernelError::InvalidInput`].
    fn work_plane_tangent(
        &self,
        s: &Self::Shape,
        face: SubShape,
        toward: [f64; 3],
    ) -> Result<Workplane, KernelError>;

    /// The work point at half the arc length of the edge `edge` of `s`.
    ///
    /// An `edge` whose kind isn't [`TopoKind::Edge`] and an index out of
    /// range are [`KernelError::InvalidInput`].
    fn edge_midpoint(&self, s: &Self::Shape, edge: SubShape) -> Result<Point3, KernelError>;

    /// Tight axis-aligned bounding box of one sub-shape of `s`. An index out
    /// of range for `of.kind` is [`KernelError::InvalidInput`], checked
    /// before anything reaches the backend.
    fn sub_shape_bounding_box(
        &self,
        s: &Self::Shape,
        of: SubShape,
    ) -> Result<BoundingBox, KernelError>;

    /// Writes `s` to a STEP (ISO 10303-21) file at `path`, replacing any
    /// existing file. Lengths are written in millimetres.
    fn export_step(&self, s: &Self::Shape, path: &Path) -> Result<(), KernelError>;
    /// Reads every shape in the STEP file at `path` as one shape.
    ///
    /// A missing, unreadable or malformed file is an error.
    fn import_step(&self, path: &Path) -> Result<Self::Shape, KernelError>;
}

/// Rejects a size that is not finite or not strictly positive.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn validate_size(name: &str, v: f64) -> Result<(), KernelError> {
    if !v.is_finite() || v <= 0.0 {
        return Err(KernelError::InvalidInput(format!(
            "{name} must be finite and > 0, got {v}"
        )));
    }
    Ok(())
}

/// Rejects a position with a non-finite coordinate.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn validate_point(name: &str, p: Point3) -> Result<(), KernelError> {
    if p.iter().any(|c| !c.is_finite()) {
        return Err(KernelError::InvalidInput(format!(
            "{name} must have finite coordinates, got {p:?}"
        )));
    }
    Ok(())
}

/// Converts a file path to the NUL-terminated UTF-8 string the backend takes.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn path_to_cstring(path: &Path) -> Result<CString, KernelError> {
    let utf8 = path.to_str().ok_or_else(|| {
        KernelError::InvalidInput(format!("path is not valid UTF-8: {}", path.display()))
    })?;
    CString::new(utf8)
        .map_err(|_| KernelError::InvalidInput(format!("path contains NUL: {}", path.display())))
}

/// Checks that `path` names an existing regular file, then converts it as
/// [`path_to_cstring`] does.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn input_file_cstring(path: &Path) -> Result<CString, KernelError> {
    if !path.is_file() {
        return Err(KernelError::InvalidInput(format!(
            "no such file: {}",
            path.display()
        )));
    }
    path_to_cstring(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_positive_or_non_finite_sizes() {
        assert!(validate_size("dx", 1.0).is_ok());
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(matches!(
                validate_size("dx", bad),
                Err(KernelError::InvalidInput(_))
            ));
        }
    }

    #[test]
    fn rejects_non_finite_points() {
        assert!(validate_point("corner", [0.0, -5.0, 1e9]).is_ok());
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(validate_point("corner", [bad, 0.0, 0.0]).is_err());
            assert!(validate_point("corner", [0.0, bad, 0.0]).is_err());
            assert!(validate_point("corner", [0.0, 0.0, bad]).is_err());
        }
    }

    #[test]
    fn input_file_must_exist() {
        let missing = std::env::temp_dir().join("swx-kernel-no-such-file.step");
        assert!(matches!(
            input_file_cstring(&missing),
            Err(KernelError::InvalidInput(_))
        ));
        assert!(matches!(
            input_file_cstring(&std::env::temp_dir()),
            Err(KernelError::InvalidInput(_))
        ));
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
        assert!(input_file_cstring(&manifest).is_ok());
    }

    #[test]
    fn path_with_nul_is_rejected() {
        assert!(matches!(
            path_to_cstring(Path::new("a\0b.step")),
            Err(KernelError::InvalidInput(_))
        ));
        assert_eq!(
            path_to_cstring(Path::new("out/a.step")).unwrap().as_bytes(),
            b"out/a.step"
        );
    }
}
