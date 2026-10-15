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

use std::fmt;

mod mesh;

#[cfg(feature = "occt")]
mod ffi;
#[cfg(feature = "occt")]
mod occt;

pub use mesh::Mesh;
#[cfg(feature = "occt")]
pub use occt::{OcctKernel, OcctShape};

/// A point or vector in model space, `[x, y, z]`.
pub type Point3 = [f64; 3];

/// Errors reported by kernel operations.
#[derive(Debug, Clone, PartialEq)]
pub enum KernelError {
    /// An input was rejected before reaching the kernel.
    InvalidInput(String),
    /// The kernel itself failed (e.g. an OCCT exception).
    Backend(String),
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            KernelError::Backend(msg) => write!(f, "kernel failure: {msg}"),
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
}
