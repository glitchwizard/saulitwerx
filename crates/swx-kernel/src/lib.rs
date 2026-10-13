//! Saulitwerx geometry kernel.
//!
//! The B-rep backend is OpenCascade (OCCT), enabled by the `occt` cargo
//! feature (off by default). OCCT is reached through a small C ABI shim in
//! `cpp/occt_shim.cpp` and linked dynamically against the system libraries;
//! see the M1-01 ADR in `docs/ARCHITECTURE.md`.
//!
//! The full `Kernel` trait arrives in M1-03. For now this crate exposes the
//! minimum needed to prove the binding: build a box and measure its volume.

use std::fmt;

#[cfg(feature = "occt")]
mod ffi;

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

/// Checks that box dimensions are finite and strictly positive.
#[cfg(any(feature = "occt", test))]
fn validate_box_dims(dx: f64, dy: f64, dz: f64) -> Result<(), KernelError> {
    for (name, v) in [("dx", dx), ("dy", dy), ("dz", dz)] {
        if !v.is_finite() || v <= 0.0 {
            return Err(KernelError::InvalidInput(format!(
                "box dimension {name} must be finite and > 0, got {v}"
            )));
        }
    }
    Ok(())
}

/// Builds an axis-aligned `dx × dy × dz` box with OCCT and returns its volume,
/// computed by OCCT's mass-properties integration (not by multiplying sides).
#[cfg(feature = "occt")]
pub fn box_volume(dx: f64, dy: f64, dz: f64) -> Result<f64, KernelError> {
    validate_box_dims(dx, dy, dz)?;
    ffi::box_volume(dx, dy, dz)
        .ok_or_else(|| KernelError::Backend("box construction failed".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_positive_or_non_finite_dims() {
        assert!(validate_box_dims(1.0, 2.0, 3.0).is_ok());
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                validate_box_dims(bad, 1.0, 1.0),
                Err(KernelError::InvalidInput(_))
            ));
            assert!(validate_box_dims(1.0, bad, 1.0).is_err());
            assert!(validate_box_dims(1.0, 1.0, bad).is_err());
        }
    }

    #[cfg(feature = "occt")]
    #[test]
    fn occt_box_10x20x30_has_volume_6000() {
        let v = box_volume(10.0, 20.0, 30.0).expect("box should build");
        assert!((v - 6000.0).abs() <= 1e-6, "volume was {v}");
    }

    #[cfg(feature = "occt")]
    #[test]
    fn occt_box_rejects_zero_dimension() {
        assert!(matches!(
            box_volume(0.0, 1.0, 1.0),
            Err(KernelError::InvalidInput(_))
        ));
    }
}
