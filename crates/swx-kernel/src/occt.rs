//! OpenCascade-backed [`Kernel`] implementation (feature `occt`).

use std::path::Path;

use crate::ffi::{self, BooleanOp, ShapeHandle};
use crate::profile::prepare_face;
use crate::topo::validate_adjacent;
use crate::{
    input_file_cstring, path_to_cstring, validate_point, validate_size, BoundingBox, Extent,
    Kernel, KernelError, Mesh, Point3, ProfileLoop, SubShape, TopoKind, Workplane,
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

    fn make_face(
        &self,
        plane: &Workplane,
        outer: &ProfileLoop,
        holes: &[ProfileLoop],
    ) -> Result<OcctShape, KernelError> {
        let face = prepare_face(plane, outer, holes)?;
        let (loop_sizes, edges) = face.encode();
        ffi::make_face(&face.plane_array(), &loop_sizes, &edges)
            .map(OcctShape)
            .ok_or_else(|| {
                KernelError::Backend(
                    "make_face failed: the face is invalid (e.g. a self-intersecting loop \
                     or a hole outside the outer loop)"
                        .to_string(),
                )
            })
    }

    fn extrude(&self, face: &OcctShape, extent: Extent) -> Result<OcctShape, KernelError> {
        let (offset, length) = extent.offset_and_length()?;
        ffi::extrude(&face.0, offset, length)
            .map(OcctShape)
            .ok_or_else(|| backend("extrude"))
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

    fn sub_shape_count(&self, s: &OcctShape, kind: TopoKind) -> Result<usize, KernelError> {
        ffi::sub_shape_count(&s.0, kind.code()).ok_or_else(|| backend("sub_shape_count"))
    }

    fn adjacent(
        &self,
        s: &OcctShape,
        of: SubShape,
        kind: TopoKind,
    ) -> Result<Vec<SubShape>, KernelError> {
        validate_adjacent(of, kind, self.sub_shape_count(s, of.kind)?)?;
        let cap = self.sub_shape_count(s, kind)?;
        let mut indices = ffi::adjacent(&s.0, of.kind.code(), of.index, kind.code(), cap)
            .ok_or_else(|| backend("adjacent"))?;
        indices.sort_unstable();
        indices.dedup();
        Ok(indices
            .into_iter()
            .map(|index| SubShape { kind, index })
            .collect())
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
    use crate::Segment2;
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

    fn line(start: [f64; 2], end: [f64; 2]) -> Segment2 {
        Segment2::Line { start, end }
    }

    fn polygon(points: &[[f64; 2]]) -> ProfileLoop {
        let n = points.len();
        ProfileLoop {
            segments: (0..n)
                .map(|i| line(points[i], points[(i + 1) % n]))
                .collect(),
        }
    }

    fn rect(w: f64, h: f64) -> ProfileLoop {
        polygon(&[[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]])
    }

    fn circle(center: [f64; 2], radius: f64) -> ProfileLoop {
        ProfileLoop {
            segments: vec![Segment2::Circle { center, radius }],
        }
    }

    const PLATE_VOLUME: f64 = 10000.0 - 250.0 * PI;

    fn plate_face(plane: &Workplane) -> OcctShape {
        K.make_face(plane, &rect(40.0, 25.0), &[circle([20.0, 12.5], 5.0)])
            .unwrap()
    }

    fn assert_bbox(s: &OcctShape, min: Point3, max: Point3) {
        let bb = K.bounding_box(s).unwrap();
        for axis in 0..3 {
            assert_close(bb.min[axis], min[axis]);
            assert_close(bb.max[axis], max[axis]);
        }
    }

    #[test]
    fn extrude_plate_with_hole() {
        let plate = K
            .extrude(&plate_face(&Workplane::XY), Extent::Distance(10.0))
            .unwrap();
        assert_close(K.volume(&plate).unwrap(), PLATE_VOLUME);
        assert_bbox(&plate, [0.0; 3], [40.0, 25.0, 10.0]);
        assert_eq!(K.face_count(&plate).unwrap(), 7);
    }

    #[test]
    fn extrude_slot_of_lines_and_arcs() {
        let slot = ProfileLoop {
            segments: vec![
                line([0.0, 0.0], [30.0, 0.0]),
                Segment2::Arc {
                    center: [30.0, 5.0],
                    radius: 5.0,
                    start_angle: -PI / 2.0,
                    end_angle: PI / 2.0,
                },
                line([30.0, 10.0], [0.0, 10.0]),
                Segment2::Arc {
                    center: [0.0, 5.0],
                    radius: 5.0,
                    start_angle: PI / 2.0,
                    end_angle: 3.0 * PI / 2.0,
                },
            ],
        };
        let face = K.make_face(&Workplane::XY, &slot, &[]).unwrap();
        let solid = K.extrude(&face, Extent::Distance(4.0)).unwrap();
        assert_close(K.volume(&solid).unwrap(), 4.0 * (300.0 + 25.0 * PI));
        assert_eq!(K.face_count(&solid).unwrap(), 6);
    }

    #[test]
    fn extrude_directions_and_workplanes() {
        let face = plate_face(&Workplane::XY);
        let down = K.extrude(&face, Extent::Distance(-10.0)).unwrap();
        assert_bbox(&down, [0.0, 0.0, -10.0], [40.0, 25.0, 0.0]);
        assert_close(K.volume(&down).unwrap(), PLATE_VOLUME);
        let both = K.extrude(&face, Extent::Symmetric(10.0)).unwrap();
        assert_bbox(&both, [0.0, 0.0, -5.0], [40.0, 25.0, 5.0]);
        assert_close(K.volume(&both).unwrap(), PLATE_VOLUME);

        let yz = K.make_face(&Workplane::YZ, &rect(40.0, 25.0), &[]).unwrap();
        let yz = K.extrude(&yz, Extent::Distance(10.0)).unwrap();
        assert_bbox(&yz, [0.0; 3], [10.0, 40.0, 25.0]);

        let raised = Workplane {
            origin: [0.0, 0.0, 20.0],
            ..Workplane::XY
        };
        let raised = K
            .extrude(&plate_face(&raised), Extent::Distance(10.0))
            .unwrap();
        assert_bbox(&raised, [0.0, 0.0, 20.0], [40.0, 25.0, 30.0]);
    }

    #[test]
    fn step_round_trip_extruded_plate() {
        let plate = K
            .extrude(&plate_face(&Workplane::XY), Extent::Distance(10.0))
            .unwrap();
        assert_step_round_trip("extruded-plate", &plate, PLATE_VOLUME, 7);
    }

    #[test]
    fn make_face_rejects_bad_inputs_before_the_backend() {
        let invalid =
            |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        let wp = Workplane::XY;
        assert!(invalid(K.make_face(&wp, &ProfileLoop::default(), &[])));
        let mut gap = rect(1.0, 1.0);
        gap.segments[2] = line([1.0, 1.0 + 1e-6], [0.0, 1.0]);
        assert!(invalid(K.make_face(&wp, &gap, &[])));
        assert!(invalid(K.make_face(
            &wp,
            &circle([0.0, f64::NAN], 1.0),
            &[]
        )));
        assert!(invalid(K.make_face(&wp, &circle([0.0; 2], 0.0), &[])));
        let skew = Workplane {
            x_dir: [1.0, 0.0, 0.1],
            ..wp
        };
        assert!(invalid(K.make_face(&skew, &rect(1.0, 1.0), &[])));
        let zero = Workplane {
            normal: [0.0; 3],
            ..wp
        };
        assert!(invalid(K.make_face(&zero, &rect(1.0, 1.0), &[])));
    }

    #[test]
    fn make_face_rejects_faces_occt_finds_invalid() {
        let backend = |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::Backend(_)));
        let wp = Workplane::XY;
        // A bow-tie outer loop crosses itself between (5, 5) and (10, 0).
        let bow_tie = polygon(&[[0.0, 0.0], [10.0, 10.0], [10.0, 0.0], [0.0, 4.0]]);
        assert!(backend(K.make_face(&wp, &bow_tie, &[])));
        let outside = circle([60.0, 12.5], 5.0);
        assert!(backend(K.make_face(&wp, &rect(40.0, 25.0), &[outside])));
        let crossing = circle([40.0, 12.5], 5.0);
        assert!(backend(K.make_face(&wp, &rect(40.0, 25.0), &[crossing])));
    }

    #[test]
    fn extrude_rejects_bad_extents_and_non_faces() {
        let face = plate_face(&Workplane::XY);
        for bad in [0.0, f64::NAN, f64::INFINITY] {
            for extent in [Extent::Distance(bad), Extent::Symmetric(bad)] {
                assert!(matches!(
                    K.extrude(&face, extent),
                    Err(KernelError::InvalidInput(_))
                ));
            }
        }
        let solid = unit_box([0.0; 3], 1.0);
        assert!(matches!(
            K.extrude(&solid, Extent::Distance(1.0)),
            Err(KernelError::Backend(_))
        ));
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

    const KINDS: [TopoKind; 3] = [TopoKind::Face, TopoKind::Edge, TopoKind::Vertex];

    fn sub(kind: TopoKind, index: usize) -> SubShape {
        SubShape { kind, index }
    }

    /// For each sub-shape of `of`, how many of `kind` are adjacent to it.
    fn adjacency_counts(s: &OcctShape, of: TopoKind, kind: TopoKind) -> Vec<usize> {
        (0..K.sub_shape_count(s, of).unwrap())
            .map(|i| {
                let adj = K.adjacent(s, sub(of, i), kind).unwrap();
                assert!(adj.windows(2).all(|w| w[0].index < w[1].index));
                assert!(adj.iter().all(|a| a.kind == kind));
                adj.len()
            })
            .collect()
    }

    /// Up and down agree: `b ∈ adjacent(a, kb)` exactly when `a ∈ adjacent(b, ka)`.
    fn assert_symmetric(s: &OcctShape) {
        for ka in KINDS {
            for kb in KINDS.into_iter().filter(|&k| k != ka) {
                for i in 0..K.sub_shape_count(s, ka).unwrap() {
                    for b in K.adjacent(s, sub(ka, i), kb).unwrap() {
                        assert!(
                            K.adjacent(s, b, ka).unwrap().contains(&sub(ka, i)),
                            "{ka:?} {i} lists {b:?}, but not the other way round"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn box_sub_shapes_and_adjacency() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        let counts = KINDS.map(|k| K.sub_shape_count(&b, k).unwrap());
        assert_eq!(counts, [6, 12, 8]);
        use TopoKind::{Edge, Face, Vertex};
        assert_eq!(adjacency_counts(&b, Face, Edge), vec![4; 6]);
        assert_eq!(adjacency_counts(&b, Face, Vertex), vec![4; 6]);
        assert_eq!(adjacency_counts(&b, Edge, Face), vec![2; 12]);
        assert_eq!(adjacency_counts(&b, Edge, Vertex), vec![2; 12]);
        assert_eq!(adjacency_counts(&b, Vertex, Edge), vec![3; 8]);
        assert_eq!(adjacency_counts(&b, Vertex, Face), vec![3; 8]);
        assert_symmetric(&b);
    }

    #[test]
    fn plate_with_hole_sub_shapes_and_adjacency() {
        let plate = K
            .extrude(&plate_face(&Workplane::XY), Extent::Distance(10.0))
            .unwrap();
        let counts = KINDS.map(|k| K.sub_shape_count(&plate, k).unwrap());
        assert_eq!(counts, [7, 15, 10]);
        let mut edges_per_face = adjacency_counts(&plate, TopoKind::Face, TopoKind::Edge);
        edges_per_face.sort_unstable();
        assert_eq!(edges_per_face, vec![3, 4, 4, 4, 4, 5, 5]);
        assert_symmetric(&plate);
    }

    #[test]
    fn adjacent_rejects_same_kind_and_out_of_range() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        for kind in KINDS {
            let err = K.adjacent(&b, sub(kind, 0), kind).unwrap_err();
            assert!(
                matches!(err, KernelError::InvalidInput(_)),
                "{kind:?}: {err}"
            );
        }
        for (kind, n) in KINDS.into_iter().zip([6, 12, 8]) {
            let other = if kind == TopoKind::Face {
                TopoKind::Edge
            } else {
                TopoKind::Face
            };
            assert!(K.adjacent(&b, sub(kind, n - 1), other).is_ok());
            let err = K.adjacent(&b, sub(kind, n), other).unwrap_err();
            assert!(
                matches!(err, KernelError::InvalidInput(_)),
                "{kind:?}: {err}"
            );
            let err = K.adjacent(&b, sub(kind, 99), other).unwrap_err();
            assert!(
                matches!(err, KernelError::InvalidInput(_)),
                "{kind:?}: {err}"
            );
        }
    }
}
