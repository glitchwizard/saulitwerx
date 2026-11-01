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

mod feature;
mod mesh;
mod profile;

#[cfg(feature = "occt")]
mod ffi;
#[cfg(feature = "occt")]
mod occt;

pub use feature::{extrude_feature, revolve_feature, ExtrudeMode, EMPTY_VOLUME_TOLERANCE};
pub use mesh::Mesh;
#[cfg(feature = "occt")]
pub use occt::{OcctKernel, OcctShape};
pub use profile::{
    Axis, Extent, ProfileLoop, Segment2, Workplane, LOOP_GAP_TOLERANCE, PERPENDICULAR_TOLERANCE,
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
    /// Solid swept by turning a planar `face` `angle` radians about `axis`.
    ///
    /// `axis.dir` is normalized first; a positive angle turns counterclockwise
    /// looking down it (right-hand rule). It is [`KernelError::InvalidInput`],
    /// checked before the backend, when the angle is not finite, is ≤ 0 or is
    /// above 2π (2π itself, within 1e-12, is a full turn), or when the axis
    /// origin is not finite or its direction is zero or not finite.
    fn revolve(
        &self,
        face: &Self::Shape,
        axis: Axis,
        angle: f64,
    ) -> Result<Self::Shape, KernelError>;

    /// Boolean union `a ∪ b`.
    fn union(&self, a: &Self::Shape, b: &Self::Shape) -> Result<Self::Shape, KernelError>;
    /// Boolean difference `a − b`.
    fn cut(&self, a: &Self::Shape, b: &Self::Shape) -> Result<Self::Shape, KernelError>;
    /// Boolean intersection `a ∩ b`.
    fn intersect(&self, a: &Self::Shape, b: &Self::Shape) -> Result<Self::Shape, KernelError>;

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
