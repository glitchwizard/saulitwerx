//! OpenCascade-backed [`Kernel`] implementation (feature `occt`).

use crate::ffi::{self, BooleanOp, ShapeHandle};
use crate::{validate_point, validate_size, BoundingBox, Kernel, KernelError, Point3};

/// [`Kernel`] backed by the system OpenCascade libraries.
#[derive(Debug, Default, Clone, Copy)]
pub struct OcctKernel;

/// A solid owned by [`OcctKernel`]. Freed when dropped.
#[derive(Debug)]
pub struct OcctShape(ShapeHandle);

fn backend(op: &str) -> KernelError {
    KernelError::Backend(format!("{op} failed"))
}

impl Kernel for OcctKernel {
    type Shape = OcctShape;

    fn make_box(&self, corner: Point3, size: Point3) -> Result<OcctShape, KernelError> {
        validate_point("box corner", corner)?;
        for (name, v) in ["box dx", "box dy", "box dz"].into_iter().zip(size) {
            validate_size(name, v)?;
        }
        ffi::make_box(corner, size)
            .map(OcctShape)
            .ok_or_else(|| backend("make_box"))
    }

    fn make_cylinder(
        &self,
        base: Point3,
        radius: f64,
        height: f64,
    ) -> Result<OcctShape, KernelError> {
        validate_point("cylinder base", base)?;
        validate_size("cylinder radius", radius)?;
        validate_size("cylinder height", height)?;
        ffi::make_cylinder(base, radius, height)
            .map(OcctShape)
            .ok_or_else(|| backend("make_cylinder"))
    }

    fn make_sphere(&self, center: Point3, radius: f64) -> Result<OcctShape, KernelError> {
        validate_point("sphere center", center)?;
        validate_size("sphere radius", radius)?;
        ffi::make_sphere(center, radius)
            .map(OcctShape)
            .ok_or_else(|| backend("make_sphere"))
    }

    fn union(&self, a: &OcctShape, b: &OcctShape) -> Result<OcctShape, KernelError> {
        ffi::boolean(BooleanOp::Union, &a.0, &b.0)
            .map(OcctShape)
            .ok_or_else(|| backend("union"))
    }

    fn cut(&self, a: &OcctShape, b: &OcctShape) -> Result<OcctShape, KernelError> {
        ffi::boolean(BooleanOp::Cut, &a.0, &b.0)
            .map(OcctShape)
            .ok_or_else(|| backend("cut"))
    }

    fn intersect(&self, a: &OcctShape, b: &OcctShape) -> Result<OcctShape, KernelError> {
        ffi::boolean(BooleanOp::Intersect, &a.0, &b.0)
            .map(OcctShape)
            .ok_or_else(|| backend("intersect"))
    }

    fn volume(&self, s: &OcctShape) -> Result<f64, KernelError> {
        ffi::volume(&s.0).ok_or_else(|| backend("volume"))
    }

    fn surface_area(&self, s: &OcctShape) -> Result<f64, KernelError> {
        ffi::surface_area(&s.0).ok_or_else(|| backend("surface_area"))
    }

    fn bounding_box(&self, s: &OcctShape) -> Result<BoundingBox, KernelError> {
        let b = ffi::bounding_box(&s.0).ok_or_else(|| backend("bounding_box"))?;
        Ok(BoundingBox {
            min: [b[0], b[1], b[2]],
            max: [b[3], b[4], b[5]],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const K: OcctKernel = OcctKernel;

    /// Relative tolerance 1e-6, falling back to absolute 1e-6 near zero.
    fn assert_close(actual: f64, expected: f64) {
        let tol = 1e-6 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= tol,
            "expected {expected}, got {actual}"
        );
    }

    fn unit_box(corner: Point3, edge: f64) -> OcctShape {
        K.make_box(corner, [edge; 3]).expect("box should build")
    }

    #[test]
    fn box_10x20x30_volume_area_bbox() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_close(K.surface_area(&b).unwrap(), 2200.0);
        let bb = K.bounding_box(&b).unwrap();
        for (got, want) in bb.min.iter().zip([0.0, 0.0, 0.0]) {
            assert_close(*got, want);
        }
        for (got, want) in bb.max.iter().zip([10.0, 20.0, 30.0]) {
            assert_close(*got, want);
        }
    }

    #[test]
    fn cylinder_r5_h10_volume_area() {
        let c = K.make_cylinder([0.0; 3], 5.0, 10.0).unwrap();
        assert_close(K.volume(&c).unwrap(), 250.0 * PI);
        assert_close(K.surface_area(&c).unwrap(), 150.0 * PI);
    }

    #[test]
    fn sphere_r5_volume_area() {
        let s = K.make_sphere([0.0; 3], 5.0).unwrap();
        assert_close(K.volume(&s).unwrap(), 4.0 / 3.0 * PI * 125.0);
        assert_close(K.surface_area(&s).unwrap(), 100.0 * PI);
    }

    #[test]
    fn union_of_disjoint_unit_boxes_has_volume_2() {
        let a = unit_box([0.0; 3], 1.0);
        let b = unit_box([5.0, 0.0, 0.0], 1.0);
        let u = K.union(&a, &b).unwrap();
        assert_close(K.volume(&u).unwrap(), 2.0);
    }

    #[test]
    fn cut_corner_block_leaves_750() {
        let a = unit_box([0.0; 3], 10.0);
        let b = K.make_box([0.0; 3], [5.0, 5.0, 10.0]).unwrap();
        let c = K.cut(&a, &b).unwrap();
        assert_close(K.volume(&c).unwrap(), 750.0);
    }

    #[test]
    fn intersect_boxes_offset_by_5_has_volume_500() {
        let a = unit_box([0.0; 3], 10.0);
        let b = unit_box([5.0, 0.0, 0.0], 10.0);
        let i = K.intersect(&a, &b).unwrap();
        assert_close(K.volume(&i).unwrap(), 500.0);
    }

    #[test]
    fn constructors_reject_invalid_inputs() {
        let invalid =
            |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        assert!(invalid(K.make_box([0.0; 3], [0.0, 1.0, 1.0])));
        assert!(invalid(K.make_box([f64::NAN, 0.0, 0.0], [1.0; 3])));
        assert!(invalid(K.make_cylinder([0.0; 3], -1.0, 1.0)));
        assert!(invalid(K.make_cylinder([0.0; 3], 1.0, f64::INFINITY)));
        assert!(invalid(K.make_sphere([0.0; 3], 0.0)));
    }
}
