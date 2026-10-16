//! OpenCascade-backed [`Kernel`] implementation (feature `occt`).

use std::path::Path;

use crate::ffi::{self, BooleanOp, ShapeHandle};
use crate::{
    input_file_cstring, path_to_cstring, validate_point, validate_size, BoundingBox, Kernel,
    KernelError, Mesh, Point3,
};

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

    fn tessellate(&self, s: &OcctShape, deflection: f64) -> Result<Mesh, KernelError> {
        validate_size("deflection", deflection)?;
        ffi::tessellate(&s.0, deflection).ok_or_else(|| backend("tessellate"))
    }

    fn face_count(&self, s: &OcctShape) -> Result<usize, KernelError> {
        ffi::face_count(&s.0).ok_or_else(|| backend("face_count"))
    }

    fn export_step(&self, s: &OcctShape, path: &Path) -> Result<(), KernelError> {
        let c_path = path_to_cstring(path)?;
        if ffi::export_step(&s.0, &c_path) {
            Ok(())
        } else {
            Err(KernelError::Backend(format!(
                "export_step to {} failed",
                path.display()
            )))
        }
    }

    fn import_step(&self, path: &Path) -> Result<OcctShape, KernelError> {
        let c_path = input_file_cstring(path)?;
        ffi::import_step(&c_path).map(OcctShape).ok_or_else(|| {
            KernelError::Backend(format!("import_step from {} failed", path.display()))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;
    use std::path::PathBuf;

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
    fn box_mesh_is_closed_with_exact_volume_and_bbox() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        let m = K.tessellate(&b, 0.1).unwrap();
        m.validate().unwrap();
        assert!(
            (m.signed_volume() - 6000.0).abs() <= 1e-3,
            "{}",
            m.signed_volume()
        );
        for axis in 0..3 {
            let lo = m
                .positions
                .iter()
                .map(|p| p[axis])
                .fold(f32::INFINITY, f32::min);
            let hi = m
                .positions
                .iter()
                .map(|p| p[axis])
                .fold(f32::NEG_INFINITY, f32::max);
            assert_eq!(lo, 0.0);
            assert_eq!(hi, [10.0, 20.0, 30.0][axis]);
        }
    }

    #[test]
    fn sphere_mesh_volume_and_vertices_match_exact_sphere() {
        let s = K.make_sphere([0.0; 3], 5.0).unwrap();
        let m = K.tessellate(&s, 0.01).unwrap();
        m.validate().unwrap();
        let exact = 4.0 / 3.0 * PI * 125.0;
        let rel = (m.signed_volume() - exact).abs() / exact;
        assert!(rel <= 0.005, "relative volume error {rel}");
        for p in &m.positions {
            let r = p.iter().map(|c| f64::from(*c).powi(2)).sum::<f64>().sqrt();
            assert!((r - 5.0).abs() <= 0.02, "vertex {p:?} at radius {r}");
        }
    }

    #[test]
    fn smaller_deflection_gives_more_triangles() {
        let s = K.make_sphere([0.0; 3], 5.0).unwrap();
        let coarse = K.tessellate(&s, 0.1).unwrap().triangle_count();
        let fine = K.tessellate(&s, 0.01).unwrap().triangle_count();
        assert!(fine > coarse, "fine {fine} vs coarse {coarse}");
        // Re-meshing coarsely after a fine mesh honors the new deflection.
        assert_eq!(K.tessellate(&s, 0.1).unwrap().triangle_count(), coarse);
    }

    #[test]
    fn mesh_normals_are_unit_and_outward() {
        let shapes = [
            K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap(),
            K.make_sphere([0.0; 3], 5.0).unwrap(),
            K.make_cylinder([0.0; 3], 5.0, 10.0).unwrap(),
        ];
        for shape in &shapes {
            let m = K.tessellate(shape, 0.05).unwrap();
            assert!(!m.normals.is_empty());
            for n in &m.normals {
                let len = n.iter().map(|c| f64::from(*c).powi(2)).sum::<f64>().sqrt();
                assert!((len - 1.0).abs() <= 1e-4, "normal {n:?} has length {len}");
            }
        }
        // On a sphere centered at the origin, outward normals point along the position.
        let m = K.tessellate(&shapes[1], 0.05).unwrap();
        for (p, n) in m.positions.iter().zip(&m.normals) {
            let dot: f32 = p.iter().zip(n).map(|(a, b)| a * b).sum();
            assert!(dot / 5.0 > 0.99, "normal {n:?} at {p:?} not outward");
        }
    }

    #[test]
    fn tessellate_rejects_bad_deflection() {
        let b = unit_box([0.0; 3], 1.0);
        for bad in [0.0, -0.1, f64::NAN, f64::INFINITY] {
            assert!(matches!(
                K.tessellate(&b, bad),
                Err(KernelError::InvalidInput(_))
            ));
        }
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

    /// A fresh per-test scratch directory under the system temp dir.
    fn scratch_dir(test: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("swx-kernel-{}-{test}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Exports `shape`, re-imports it, and checks volume and face count.
    fn assert_step_round_trip(test: &str, shape: &OcctShape, volume: f64, faces: usize) {
        assert_close(K.volume(shape).unwrap(), volume);
        assert_eq!(K.face_count(shape).unwrap(), faces);
        let dir = scratch_dir(test);
        let path = dir.join("shape.step");
        K.export_step(shape, &path).unwrap();
        let back = K.import_step(&path).unwrap();
        assert_close(K.volume(&back).unwrap(), volume);
        assert_eq!(K.face_count(&back).unwrap(), faces);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn step_round_trip_box() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        assert_step_round_trip("box", &b, 6000.0, 6);
    }

    #[test]
    fn step_round_trip_cylinder() {
        let c = K.make_cylinder([0.0; 3], 5.0, 10.0).unwrap();
        assert_step_round_trip("cylinder", &c, 250.0 * PI, 3);
    }

    #[test]
    fn step_round_trip_box_with_hole() {
        let block = unit_box([0.0; 3], 10.0);
        let pin = K.make_cylinder([5.0, 5.0, 0.0], 2.0, 10.0).unwrap();
        let holed = K.cut(&block, &pin).unwrap();
        assert_step_round_trip("holed", &holed, 1000.0 - 40.0 * PI, 7);
    }

    /// Golden STEP file produced by our own `export_step`. Set
    /// `SWX_BLESS_GOLDEN=1` to regenerate it.
    #[test]
    fn step_golden_box_10x20x30() {
        let golden =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/box_10x20x30.step");
        if std::env::var_os("SWX_BLESS_GOLDEN").is_some() {
            let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
            std::fs::create_dir_all(golden.parent().unwrap()).unwrap();
            K.export_step(&b, &golden).unwrap();
        }
        let s = K.import_step(&golden).unwrap();
        assert_close(K.volume(&s).unwrap(), 6000.0);
        assert_eq!(K.face_count(&s).unwrap(), 6);
    }

    #[test]
    fn step_import_rejects_missing_and_garbage_files() {
        let dir = scratch_dir("bad-import");
        assert!(K.import_step(&dir.join("missing.step")).is_err());
        let garbage = dir.join("garbage.step");
        std::fs::write(&garbage, "this is not an ISO 10303-21 file\n").unwrap();
        assert!(K.import_step(&garbage).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn step_export_to_missing_directory_fails() {
        let dir = scratch_dir("bad-export");
        let b = unit_box([0.0; 3], 1.0);
        assert!(K
            .export_step(&b, &dir.join("no/such/dir/out.step"))
            .is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
