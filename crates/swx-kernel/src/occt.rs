//! OpenCascade-backed [`Kernel`] implementation (feature `occt`).

use std::f64::consts::FRAC_PI_2;
use std::path::Path;

use crate::datum::{axis_nearest_origin, plane_nearest_origin, plane_tangent_to_cylinder};
use crate::ffi::{self, ChamferMode, EdgeOp, FaceCylinder, FacePlane, ShapeHandle, SubShapeAxis};
use crate::hole::{perpendicular, prepare_hole, HoleCylinder};
use crate::loft::check_section_count;
use crate::profile::{
    check_revolve_profile, check_sweep_bends, prepare_face, prepare_path, prepare_revolve,
    profile_reach,
};
use crate::topo::{
    compose_images, validate_adjacent, validate_edges, validate_edges_on_faces, validate_faces,
    validate_index,
};
use crate::{
    input_file_cstring, path_to_cstring, validate_point, validate_size, Axis, BooleanOp,
    BoundingBox, ChamferSize, CombineHistory, EdgeOnFace, Extent, ExtrudeHistory, HoleDepth,
    HoleKind, Kernel, KernelError, Mesh, OpHistory, Operand, Point3, ProfileLoop, Segment2,
    SubShape, TopoKind, Workplane, EMPTY_VOLUME_TOLERANCE, LOOP_GAP_TOLERANCE,
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

fn make_face_error() -> KernelError {
    KernelError::Backend(
        "make_face failed: the face is invalid (e.g. a self-intersecting loop \
         or a hole outside the outer loop)"
            .to_string(),
    )
}

fn bbox(b: [f64; 6]) -> BoundingBox {
    BoundingBox {
        min: [b[0], b[1], b[2]],
        max: [b[3], b[4], b[5]],
    }
}

fn face_sub(index: usize) -> SubShape {
    SubShape {
        kind: TopoKind::Face,
        index,
    }
}

impl OcctKernel {
    /// The input checks of [`Kernel::combine`], before the backend: at
    /// least one tool, and a solid body in the target and every tool.
    fn check_combine_inputs(
        &self,
        target: &OcctShape,
        tools: &[OcctShape],
    ) -> Result<(), KernelError> {
        if tools.is_empty() {
            return Err(KernelError::InvalidInput(
                "combine needs at least one tool body".to_string(),
            ));
        }
        if self.body_count(target)? == 0 {
            return Err(KernelError::InvalidInput(
                "the combine target has no solid body".to_string(),
            ));
        }
        for (i, tool) in tools.iter().enumerate() {
            if self.body_count(tool)? == 0 {
                return Err(KernelError::InvalidInput(format!(
                    "combine tool {i} has no solid body"
                )));
            }
        }
        Ok(())
    }

    /// Checked after every combine step: once a cut or intersect is empty it
    /// stays empty, and an empty shape is no valid operand for the next step.
    fn check_combine_step(&self, r: OcctShape) -> Result<OcctShape, KernelError> {
        if self.volume(&r)?.abs() <= EMPTY_VOLUME_TOLERANCE {
            return Err(KernelError::EmptyResult(
                "the combined result encloses no volume".to_string(),
            ));
        }
        Ok(r)
    }
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
            .ok_or_else(make_face_error)
    }

    fn make_face_tracked(
        &self,
        plane: &Workplane,
        outer: &ProfileLoop,
        holes: &[ProfileLoop],
    ) -> Result<(OcctShape, Vec<Vec<SubShape>>), KernelError> {
        let face = prepare_face(plane, outer, holes)?;
        let (loop_sizes, edges) = face.encode();
        let (shape, indices) = ffi::make_face_tracked(&face.plane_array(), &loop_sizes, &edges)
            .ok_or_else(make_face_error)?;
        // The shim reports edges in traversal order; a loop that
        // `prepare_face` reversed lists its segments back to front.
        let mut rest = indices.as_slice();
        let loops = loop_sizes
            .iter()
            .zip(&face.reversed)
            .map(|(&n, &reversed)| {
                let (this, tail) = rest.split_at(n);
                rest = tail;
                (0..n)
                    .map(|seg| SubShape {
                        kind: TopoKind::Edge,
                        index: this[if reversed { n - 1 - seg } else { seg }],
                    })
                    .collect()
            })
            .collect();
        Ok((OcctShape(shape), loops))
    }

    fn extrude(&self, face: &OcctShape, extent: Extent) -> Result<OcctShape, KernelError> {
        let (offset, length) = extent.offset_and_length()?;
        ffi::extrude(&face.0, offset, length)
            .map(OcctShape)
            .ok_or_else(|| backend("extrude"))
    }

    fn extrude_tracked(
        &self,
        face: &OcctShape,
        extent: Extent,
    ) -> Result<(OcctShape, ExtrudeHistory), KernelError> {
        let (offset, length) = extent.offset_and_length()?;
        let n_edges = self.sub_shape_count(face, TopoKind::Edge)?;
        let (shape, caps, sides) = ffi::extrude_tracked(&face.0, offset, length, n_edges)
            .ok_or_else(|| backend("extrude"))?;
        let history = ExtrudeHistory {
            start_cap: face_sub(caps[0]),
            end_cap: face_sub(caps[1]),
            sides: sides
                .into_iter()
                .enumerate()
                .map(|(i, side)| {
                    let edge = SubShape {
                        kind: TopoKind::Edge,
                        index: i,
                    };
                    (edge, face_sub(side))
                })
                .collect(),
        };
        Ok((OcctShape(shape), history))
    }

    fn revolve(&self, face: &OcctShape, axis: Axis, angle: f64) -> Result<OcctShape, KernelError> {
        let (origin, dir, angle) = prepare_revolve(axis, angle)?;
        let profile = ffi::revolve_profile(&face.0, origin, dir).ok_or_else(|| {
            KernelError::InvalidInput("revolve needs a single planar face".to_string())
        })?;
        check_revolve_profile(origin, dir, &profile)?;
        ffi::revolve(&face.0, origin, dir, angle)
            .map(OcctShape)
            .ok_or_else(|| backend("revolve"))
    }

    fn sweep(
        &self,
        profile: &OcctShape,
        plane: &Workplane,
        path: &[Segment2],
    ) -> Result<OcctShape, KernelError> {
        let prepared = prepare_path(plane, path)?;
        let (point, normal, wires) = ffi::sweep_profile(&profile.0).ok_or_else(|| {
            KernelError::InvalidInput("sweep profile is not a single planar face".to_string())
        })?;
        if wires != 1 {
            return Err(KernelError::InvalidInput(
                "sweep profile has a hole; profiles with holes are not supported".to_string(),
            ));
        }
        let start = prepared.point_3d(prepared.loops[0][0].from);
        let off: f64 = (0..3).map(|k| (start[k] - point[k]) * normal[k]).sum();
        if off.abs() > LOOP_GAP_TOLERANCE {
            return Err(KernelError::InvalidInput(format!(
                "sweep path starts {off} off the profile's plane"
            )));
        }
        // The profile's reach from the path's start, from its mesh points: a
        // fine mesh (deflection 1e-5 of its size) of a face has its vertices
        // on the face's boundary, so this is the reach to within that.
        let b = self.bounding_box(profile)?;
        let size = (0..3)
            .map(|k| (b.max[k] - b.min[k]).powi(2))
            .sum::<f64>()
            .sqrt();
        let mesh = self.tessellate(profile, (size * 1e-5).max(1e-9))?;
        let points: Vec<Point3> = mesh.positions.iter().map(|p| p.map(f64::from)).collect();
        check_sweep_bends(path, profile_reach(&points, start, normal))?;
        let (_, edges) = prepared.encode();
        let result = ffi::sweep(&profile.0, &prepared.plane_array(), &edges)
            .map(OcctShape)
            .ok_or_else(|| backend("sweep"))?;
        if self.volume(&result)?.abs() <= EMPTY_VOLUME_TOLERANCE {
            return Err(KernelError::EmptyResult(
                "the sweep encloses no volume".to_string(),
            ));
        }
        Ok(result)
    }

    fn fillet(
        &self,
        s: &OcctShape,
        edges: &[SubShape],
        radius: f64,
    ) -> Result<OcctShape, KernelError> {
        validate_size("fillet radius", radius)?;
        let indices = validate_edges(edges, self.sub_shape_count(s, TopoKind::Edge)?)?;
        ffi::fillet_chamfer(EdgeOp::Fillet, &s.0, &indices, radius)
            .map(OcctShape)
            .ok_or_else(|| backend("fillet"))
    }

    fn fillet_variable(
        &self,
        s: &OcctShape,
        edge: SubShape,
        start: SubShape,
        r_start: f64,
        r_end: f64,
    ) -> Result<OcctShape, KernelError> {
        validate_size("fillet start radius", r_start)?;
        validate_size("fillet end radius", r_end)?;
        if edge.kind != TopoKind::Edge || start.kind != TopoKind::Vertex {
            return Err(KernelError::InvalidInput(format!(
                "a variable fillet needs an edge and a vertex, got {:?} {} and {:?} {}",
                edge.kind, edge.index, start.kind, start.index
            )));
        }
        validate_index(edge, self.sub_shape_count(s, TopoKind::Edge)?)?;
        validate_index(start, self.sub_shape_count(s, TopoKind::Vertex)?)?;
        if !self.adjacent(s, edge, TopoKind::Vertex)?.contains(&start) {
            return Err(KernelError::InvalidInput(format!(
                "vertex {} is not on edge {}",
                start.index, edge.index
            )));
        }
        // OCCT's variable-radius builder can return a shape that passes
        // `BRepCheck_Analyzer` although its larger radius does not fit the
        // faces next to the edge (seen with 2 → 25 on a 10 mm face). At the
        // end with the larger radius the cross-section is that of a constant
        // fillet of the same radius, so if that one can't be built, neither
        // can the variable one.
        self.fillet(s, &[edge], r_start.max(r_end))
            .map_err(|_| backend("fillet"))?;
        ffi::fillet_variable(&s.0, edge.index, start.index, r_start, r_end)
            .map(OcctShape)
            .ok_or_else(|| backend("fillet"))
    }

    fn chamfer(
        &self,
        s: &OcctShape,
        edges: &[SubShape],
        distance: f64,
    ) -> Result<OcctShape, KernelError> {
        validate_size("chamfer distance", distance)?;
        let indices = validate_edges(edges, self.sub_shape_count(s, TopoKind::Edge)?)?;
        ffi::fillet_chamfer(EdgeOp::Chamfer, &s.0, &indices, distance)
            .map(OcctShape)
            .ok_or_else(|| backend("chamfer"))
    }

    fn shell(
        &self,
        s: &OcctShape,
        open: &[SubShape],
        thickness: f64,
    ) -> Result<OcctShape, KernelError> {
        validate_size("shell thickness", thickness)?;
        let indices = validate_faces(open, self.sub_shape_count(s, TopoKind::Face)?)?;
        ffi::shell(&s.0, &indices, thickness)
            .map(OcctShape)
            .ok_or_else(|| backend("shell"))
    }

    fn draft(
        &self,
        s: &OcctShape,
        faces: &[SubShape],
        neutral: Axis,
        angle: f64,
    ) -> Result<OcctShape, KernelError> {
        if !angle.is_finite() || angle == 0.0 || angle.abs() >= FRAC_PI_2 {
            return Err(KernelError::InvalidInput(format!(
                "draft angle must be finite, non-zero and below π/2 in size, got {angle}"
            )));
        }
        validate_point("draft neutral origin", neutral.origin)?;
        let [x, y, z] = neutral.dir;
        let len = (x * x + y * y + z * z).sqrt();
        if !len.is_finite() || len == 0.0 {
            return Err(KernelError::InvalidInput(format!(
                "draft pull direction must be finite and non-zero, got {:?}",
                neutral.dir
            )));
        }
        let indices = validate_faces(faces, self.sub_shape_count(s, TopoKind::Face)?)?;
        let dir = [x / len, y / len, z / len];
        ffi::draft(&s.0, &indices, neutral.origin, dir, angle)
            .map(OcctShape)
            .ok_or_else(|| backend("draft"))
    }

    /// Cuts one cylinder per [`HoleCylinder`] from `s`: a circle on the
    /// plane through the start point normal to the drill direction, extruded
    /// along it ([`Kernel::make_face`], [`Kernel::extrude`], [`Kernel::cut`]).
    /// A through hole's cylinder runs past the far side of the solid's
    /// bounding box. A countersink then also cuts its cone, a trapezoid
    /// revolved about the drill axis ([`Kernel::revolve`]). The result is
    /// checked with OCCT's shape checker
    /// (`BRepCheck_Analyzer`, via [`ffi::is_valid`]).
    fn hole(
        &self,
        s: &OcctShape,
        at: Axis,
        kind: HoleKind,
        depth: HoleDepth,
    ) -> Result<OcctShape, KernelError> {
        let plan = prepare_hole(at, kind, depth)?;
        let misses = || {
            KernelError::InvalidInput("the hole misses the solid: it removes nothing".to_string())
        };
        // Farthest reach of the solid along the drill direction from the start point.
        let b = self.bounding_box(s)?;
        let mut reach = f64::NEG_INFINITY;
        for i in 0..8 {
            let corner = [
                if i & 1 == 0 { b.min[0] } else { b.max[0] },
                if i & 2 == 0 { b.min[1] } else { b.max[1] },
                if i & 4 == 0 { b.min[2] } else { b.max[2] },
            ];
            let along = (0..3)
                .map(|k| (corner[k] - plan.origin[k]) * plan.dir[k])
                .sum::<f64>();
            reach = reach.max(along);
        }
        if reach <= 0.0 {
            return Err(misses());
        }
        let through = reach + (0.01 * reach).max(1.0);
        let plane = Workplane {
            origin: plan.origin,
            x_dir: perpendicular(plan.dir),
            normal: plan.dir,
        };
        let tool = |c: &HoleCylinder| {
            let circle = ProfileLoop {
                segments: vec![Segment2::Circle {
                    center: [0.0, 0.0],
                    radius: c.radius,
                }],
            };
            let face = self.make_face(&plane, &circle, &[])?;
            self.extrude(&face, Extent::Distance(c.length.unwrap_or(through)))
        };
        let (first, rest) = plan
            .cylinders
            .split_first()
            .ok_or_else(|| KernelError::InvalidInput("a hole needs a cylinder".to_string()))?;
        let mut result = self.cut(s, &tool(first)?)?;
        for c in rest {
            result = self.cut(&result, &tool(c)?)?;
        }
        if let Some(cone) = plan.cone {
            // The countersink cone: the trapezoid (0, 0), (R, 0), (r, h),
            // (0, h) on a plane through the drill axis, with v along the
            // drill direction, revolved a full turn about that axis
            // ([`Kernel::revolve`], OCCT `BRepPrimAPI_MakeRevol`). It is the
            // frustum from radius R at the start point to r at depth h.
            let x_dir = perpendicular(plan.dir);
            let d = plan.dir;
            let axial = Workplane {
                origin: plan.origin,
                x_dir,
                normal: [
                    x_dir[1] * d[2] - x_dir[2] * d[1],
                    x_dir[2] * d[0] - x_dir[0] * d[2],
                    x_dir[0] * d[1] - x_dir[1] * d[0],
                ],
            };
            let corners = [
                [0.0, 0.0],
                [cone.top_radius, 0.0],
                [cone.bottom_radius, cone.length],
                [0.0, cone.length],
            ];
            let trapezoid = ProfileLoop {
                segments: (0..4)
                    .map(|i| Segment2::Line {
                        start: corners[i],
                        end: corners[(i + 1) % 4],
                    })
                    .collect(),
            };
            let face = self.make_face(&axial, &trapezoid, &[])?;
            let axis = Axis {
                origin: plan.origin,
                dir: plan.dir,
            };
            let cone_solid = self.revolve(&face, axis, std::f64::consts::TAU)?;
            result = self.cut(&result, &cone_solid)?;
        }
        if !ffi::is_valid(&result.0) {
            return Err(KernelError::Backend(
                "hole failed: the result is not a valid solid".to_string(),
            ));
        }
        let before = self.volume(s)?;
        let after = self.volume(&result)?;
        if after <= EMPTY_VOLUME_TOLERANCE {
            return Err(KernelError::EmptyResult(
                "the hole removes the whole solid".to_string(),
            ));
        }
        if before - after <= 1e-9 * before.abs() {
            return Err(misses());
        }
        Ok(result)
    }

    fn loft(&self, sections: &[OcctShape], ruled: bool) -> Result<OcctShape, KernelError> {
        check_section_count(sections.len())?;
        for (i, s) in sections.iter().enumerate() {
            match ffi::planar_face_wires(&s.0) {
                Some(1) => {}
                Some(_) => {
                    return Err(KernelError::InvalidInput(format!(
                        "loft section {i} has a hole; sections with holes are not supported"
                    )))
                }
                None => {
                    return Err(KernelError::InvalidInput(format!(
                        "loft section {i} is not a single planar face"
                    )))
                }
            }
        }
        let handles: Vec<&ShapeHandle> = sections.iter().map(|s| &s.0).collect();
        let result = ffi::loft(&handles, ruled)
            .map(OcctShape)
            .ok_or_else(|| backend("loft"))?;
        if self.volume(&result)?.abs() <= EMPTY_VOLUME_TOLERANCE {
            return Err(KernelError::EmptyResult(
                "the loft encloses no volume".to_string(),
            ));
        }
        Ok(result)
    }

    fn chamfer_asymmetric(
        &self,
        s: &OcctShape,
        edges: &[EdgeOnFace],
        size: ChamferSize,
    ) -> Result<OcctShape, KernelError> {
        size.validate()?;
        let pairs = validate_edges_on_faces(
            edges,
            self.sub_shape_count(s, TopoKind::Edge)?,
            self.sub_shape_count(s, TopoKind::Face)?,
        )?;
        for &(e, f) in &pairs {
            let edge = SubShape {
                kind: TopoKind::Edge,
                index: e,
            };
            let face = SubShape {
                kind: TopoKind::Face,
                index: f,
            };
            if !self.adjacent(s, edge, TopoKind::Face)?.contains(&face) {
                return Err(KernelError::InvalidInput(format!(
                    "face {f} does not bound edge {e}"
                )));
            }
        }
        let (mode, v1, v2) = match size {
            ChamferSize::TwoDistances { d1, d2 } => (ChamferMode::TwoDistances, d1, d2),
            ChamferSize::DistanceAngle { distance, angle } => {
                (ChamferMode::DistanceAngle, distance, angle)
            }
        };
        ffi::chamfer_asymmetric(mode, &s.0, &pairs, v1, v2)
            .map(OcctShape)
            .ok_or_else(|| backend("chamfer"))
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

    fn boolean_tracked(
        &self,
        op: BooleanOp,
        a: &OcctShape,
        b: &OcctShape,
    ) -> Result<(OcctShape, OpHistory), KernelError> {
        let name = match op {
            BooleanOp::Union => "union",
            BooleanOp::Cut => "cut",
            BooleanOp::Intersect => "intersect",
        };
        let n_a = self.sub_shape_count(a, TopoKind::Face)?;
        let n_b = self.sub_shape_count(b, TopoKind::Face)?;
        let (shape, images) =
            ffi::boolean_tracked(op, &a.0, &b.0, n_a + n_b).ok_or_else(|| backend(name))?;
        let modified = images
            .into_iter()
            .enumerate()
            .map(|(i, faces)| {
                let (operand, index) = if i < n_a {
                    (Operand::A, i)
                } else {
                    (Operand::B, i - n_a)
                };
                let images = faces.into_iter().map(face_sub).collect();
                (operand, face_sub(index), images)
            })
            .collect();
        let history = OpHistory {
            modified,
            generated: Vec::new(),
        };
        Ok((OcctShape(shape), history))
    }

    fn body_count(&self, s: &OcctShape) -> Result<usize, KernelError> {
        ffi::solid_count(&s.0).ok_or_else(|| backend("body_count"))
    }

    fn bodies(&self, s: &OcctShape) -> Result<Vec<OcctShape>, KernelError> {
        let mut keyed = (0..self.body_count(s)?)
            .map(|i| {
                let body = ffi::solid(&s.0, i)
                    .map(OcctShape)
                    .ok_or_else(|| backend("bodies"))?;
                Ok((self.bounding_box(&body)?.min, body))
            })
            .collect::<Result<Vec<_>, KernelError>>()?;
        keyed.sort_by(|(a, _), (b, _)| {
            a[0].total_cmp(&b[0])
                .then(a[1].total_cmp(&b[1]))
                .then(a[2].total_cmp(&b[2]))
        });
        Ok(keyed.into_iter().map(|(_, body)| body).collect())
    }

    fn combine(
        &self,
        op: BooleanOp,
        target: &OcctShape,
        tools: &[OcctShape],
    ) -> Result<OcctShape, KernelError> {
        self.check_combine_inputs(target, tools)?;
        let step = |a: &OcctShape, tool: &OcctShape| -> Result<OcctShape, KernelError> {
            let r = ffi::boolean(op, &a.0, &tool.0)
                .map(OcctShape)
                .ok_or_else(|| backend("combine"))?;
            self.check_combine_step(r)
        };
        let first = step(target, &tools[0])?;
        tools[1..]
            .iter()
            .try_fold(first, |acc, tool| step(&acc, tool))
    }

    fn combine_tracked(
        &self,
        op: BooleanOp,
        target: &OcctShape,
        tools: &[OcctShape],
    ) -> Result<(OcctShape, CombineHistory), KernelError> {
        self.check_combine_inputs(target, tools)?;
        // Before the first step every target face is its own image.
        let mut history = CombineHistory {
            target: (0..self.face_count(target)?)
                .map(|i| vec![face_sub(i)])
                .collect(),
            tools: Vec::with_capacity(tools.len()),
        };
        let mut acc: Option<OcctShape> = None;
        for tool in tools {
            let a = acc.as_ref().unwrap_or(target);
            let (r, h) = self.boolean_tracked(op, a, tool)?;
            let r = self.check_combine_step(r)?;
            // `modified` lists every face of `a`, then every face of `tool`,
            // each in face index order (`Kernel::boolean_tracked`).
            let (from_a, from_tool): (Vec<_>, Vec<_>) = h
                .modified
                .into_iter()
                .partition(|(operand, _, _)| *operand == Operand::A);
            let next: Vec<Vec<SubShape>> = from_a.into_iter().map(|(_, _, imgs)| imgs).collect();
            history.target = compose_images(&history.target, &next)?;
            for t in &mut history.tools {
                *t = compose_images(t, &next)?;
            }
            history
                .tools
                .push(from_tool.into_iter().map(|(_, _, imgs)| imgs).collect());
            acc = Some(r);
        }
        let result = acc.ok_or_else(|| backend("combine_tracked"))?;
        Ok((result, history))
    }

    fn volume(&self, s: &OcctShape) -> Result<f64, KernelError> {
        ffi::volume(&s.0).ok_or_else(|| backend("volume"))
    }

    fn surface_area(&self, s: &OcctShape) -> Result<f64, KernelError> {
        ffi::surface_area(&s.0).ok_or_else(|| backend("surface_area"))
    }

    fn bounding_box(&self, s: &OcctShape) -> Result<BoundingBox, KernelError> {
        ffi::bounding_box(&s.0)
            .map(bbox)
            .ok_or_else(|| backend("bounding_box"))
    }

    fn sub_shape_bounding_box(
        &self,
        s: &OcctShape,
        of: SubShape,
    ) -> Result<BoundingBox, KernelError> {
        validate_index(of, self.sub_shape_count(s, of.kind)?)?;
        ffi::sub_shape_bounding_box(&s.0, of.kind.code(), of.index)
            .map(bbox)
            .ok_or_else(|| backend("sub_shape_bounding_box"))
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

    fn face_plane(&self, s: &OcctShape, face: SubShape) -> Result<(Point3, Point3), KernelError> {
        if face.kind != TopoKind::Face {
            return Err(KernelError::InvalidInput(format!(
                "face_plane needs a face, got a {:?} sub-shape",
                face.kind
            )));
        }
        let count = self.sub_shape_count(s, TopoKind::Face)?;
        if face.index >= count {
            return Err(KernelError::InvalidInput(format!(
                "face index {} is out of range (the shape has {count} faces)",
                face.index
            )));
        }
        match ffi::face_plane(&s.0, face.index) {
            Some(FacePlane::Plane(point, normal)) => Ok((point, normal)),
            Some(FacePlane::NotPlanar) => Err(KernelError::InvalidInput(format!(
                "face {} is not planar",
                face.index
            ))),
            None => Err(backend("face_plane")),
        }
    }

    fn work_plane_on_face(&self, s: &OcctShape, face: SubShape) -> Result<Workplane, KernelError> {
        let (point, normal) = self.face_plane(s, face)?;
        plane_nearest_origin(point, normal)
    }

    fn work_axis_on(&self, s: &OcctShape, of: SubShape) -> Result<Axis, KernelError> {
        if of.kind == TopoKind::Vertex {
            return Err(KernelError::InvalidInput(
                "work_axis_on needs an edge or a face, got a vertex".to_string(),
            ));
        }
        validate_index(of, self.sub_shape_count(s, of.kind)?)?;
        match ffi::sub_shape_axis(&s.0, of.kind.code(), of.index) {
            Some(SubShapeAxis::Line(point, dir)) => axis_nearest_origin(point, dir),
            Some(SubShapeAxis::NoLine) => Err(KernelError::InvalidInput(format!(
                "{:?} {} is not a straight edge or a cylindrical face",
                of.kind, of.index
            ))),
            None => Err(backend("work_axis_on")),
        }
    }

    fn vertex_point(&self, s: &OcctShape, vertex: SubShape) -> Result<Point3, KernelError> {
        if vertex.kind != TopoKind::Vertex {
            return Err(KernelError::InvalidInput(format!(
                "vertex_point needs a vertex, got a {:?} sub-shape",
                vertex.kind
            )));
        }
        validate_index(vertex, self.sub_shape_count(s, TopoKind::Vertex)?)?;
        ffi::vertex_point(&s.0, vertex.index).ok_or_else(|| backend("vertex_point"))
    }

    fn work_plane_tangent(
        &self,
        s: &OcctShape,
        face: SubShape,
        toward: [f64; 3],
    ) -> Result<Workplane, KernelError> {
        if face.kind != TopoKind::Face {
            return Err(KernelError::InvalidInput(format!(
                "work_plane_tangent needs a face, got a {:?} sub-shape",
                face.kind
            )));
        }
        validate_index(face, self.sub_shape_count(s, TopoKind::Face)?)?;
        match ffi::face_cylinder(&s.0, face.index) {
            Some(FaceCylinder::Cylinder(point, dir, radius)) => {
                plane_tangent_to_cylinder(point, dir, radius, toward)
            }
            Some(FaceCylinder::NotCylindrical) => Err(KernelError::InvalidInput(format!(
                "face {} is not cylindrical",
                face.index
            ))),
            None => Err(backend("work_plane_tangent")),
        }
    }

    fn edge_midpoint(&self, s: &OcctShape, edge: SubShape) -> Result<Point3, KernelError> {
        if edge.kind != TopoKind::Edge {
            return Err(KernelError::InvalidInput(format!(
                "edge_midpoint needs an edge, got a {:?} sub-shape",
                edge.kind
            )));
        }
        validate_index(edge, self.sub_shape_count(s, TopoKind::Edge)?)?;
        ffi::edge_midpoint(&s.0, edge.index).ok_or_else(|| backend("edge_midpoint"))
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

    fn assert_point6(actual: Point3, expected: Point3) {
        for k in 0..3 {
            assert!(
                (actual[k] - expected[k]).abs() <= 1e-6,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    fn assert_bbox6(s: &OcctShape, min: Point3, max: Point3) {
        let b = K.bounding_box(s).unwrap();
        assert_point6(b.min, min);
        assert_point6(b.max, max);
    }

    #[test]
    fn bodies_of_a_union_of_disjoint_boxes() {
        let a = unit_box([0.0; 3], 10.0);
        let b = unit_box([20.0, 0.0, 0.0], 10.0);
        // Given in the other order, so the sort has something to do.
        let u = K.union(&b, &a).unwrap();
        assert_eq!(K.body_count(&u).unwrap(), 2);
        assert_close(K.volume(&u).unwrap(), 2000.0);
        let parts = K.bodies(&u).unwrap();
        assert_eq!(parts.len(), 2);
        for (part, x) in parts.iter().zip([0.0, 20.0]) {
            assert_eq!(K.body_count(part).unwrap(), 1);
            assert_close(K.volume(part).unwrap(), 1000.0);
            assert_point6(K.bounding_box(part).unwrap().min, [x, 0.0, 0.0]);
        }
        // The input is unchanged.
        assert_eq!(K.body_count(&u).unwrap(), 2);
        assert_close(K.volume(&u).unwrap(), 2000.0);
    }

    #[test]
    fn combine_join_cut_and_intersect_with_several_tools() {
        // Join.
        let target = unit_box([0.0; 3], 10.0);
        let tools = [
            unit_box([10.0, 0.0, 0.0], 10.0),
            unit_box([20.0, 0.0, 0.0], 10.0),
        ];
        let j = K.combine(BooleanOp::Union, &target, &tools).unwrap();
        assert_eq!(K.body_count(&j).unwrap(), 1);
        assert_close(K.volume(&j).unwrap(), 3000.0);
        assert_bbox6(&j, [0.0; 3], [30.0, 10.0, 10.0]);
        assert_close(K.volume(&target).unwrap(), 1000.0);
        for t in &tools {
            assert_close(K.volume(t).unwrap(), 1000.0);
        }

        // Cut with two cylinders.
        let bar = K.make_box([0.0; 3], [30.0, 10.0, 10.0]).unwrap();
        let pins = [
            K.make_cylinder([5.0, 5.0, 0.0], 2.0, 10.0).unwrap(),
            K.make_cylinder([25.0, 5.0, 0.0], 2.0, 10.0).unwrap(),
        ];
        let c = K.combine(BooleanOp::Cut, &bar, &pins).unwrap();
        assert_eq!(K.body_count(&c).unwrap(), 1);
        assert_close(K.volume(&c).unwrap(), 3000.0 - 80.0 * PI);
        assert!((K.volume(&c).unwrap() - 2748.6726).abs() <= 1e-3);
        assert_close(K.volume(&bar).unwrap(), 3000.0);
        for p in &pins {
            assert_close(K.volume(p).unwrap(), 40.0 * PI);
        }

        // Cut that splits the bar in two.
        let slab = K.make_box([10.0, -1.0, -1.0], [10.0, 12.0, 12.0]).unwrap();
        let s = K
            .combine(BooleanOp::Cut, &bar, std::slice::from_ref(&slab))
            .unwrap();
        assert_eq!(K.body_count(&s).unwrap(), 2);
        assert_close(K.volume(&s).unwrap(), 2000.0);
        let parts = K.bodies(&s).unwrap();
        assert_eq!(parts.len(), 2);
        assert_close(K.bounding_box(&parts[0]).unwrap().min[0], 0.0);
        assert_close(K.bounding_box(&parts[1]).unwrap().min[0], 20.0);
        assert_close(K.volume(&slab).unwrap(), 1440.0);

        // Intersect.
        let tools = [
            unit_box([5.0, 0.0, 0.0], 10.0),
            unit_box([0.0, 0.0, 5.0], 10.0),
        ];
        let i = K.combine(BooleanOp::Intersect, &target, &tools).unwrap();
        assert_eq!(K.body_count(&i).unwrap(), 1);
        assert_close(K.volume(&i).unwrap(), 250.0);
        assert_bbox6(&i, [5.0, 0.0, 5.0], [10.0, 10.0, 10.0]);
        assert_close(K.volume(&target).unwrap(), 1000.0);
    }

    #[test]
    fn combine_rejects_bad_inputs_and_empty_results() {
        let target = unit_box([0.0; 3], 10.0);
        let tool = unit_box([5.0, 0.0, 0.0], 10.0);
        let face = K
            .make_face(
                &Workplane::XY,
                &polygon(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]),
                &[],
            )
            .unwrap();
        let invalid = |r: Result<OcctShape, KernelError>, what: &str| {
            assert!(
                matches!(r, Err(KernelError::InvalidInput(_))),
                "{what}: {r:?}"
            );
        };
        invalid(K.combine(BooleanOp::Union, &target, &[]), "no tools");
        invalid(
            K.combine(BooleanOp::Cut, &face, std::slice::from_ref(&tool)),
            "face target",
        );
        let face_tool = K
            .make_face(
                &Workplane::XY,
                &polygon(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]),
                &[],
            )
            .unwrap();
        invalid(
            K.combine(BooleanOp::Union, &target, &[tool, face_tool]),
            "face tool",
        );
        // No common volume: the first tool misses entirely, or the second
        // misses what the first left.
        let far = unit_box([50.0, 0.0, 0.0], 10.0);
        let r = K.combine(BooleanOp::Intersect, &target, std::slice::from_ref(&far));
        assert!(matches!(r, Err(KernelError::EmptyResult(_))), "{r:?}");
        let tools = [
            unit_box([5.0, 0.0, 0.0], 10.0),
            unit_box([-20.0, 0.0, 0.0], 10.0),
        ];
        let r = K.combine(BooleanOp::Intersect, &target, &tools);
        assert!(matches!(r, Err(KernelError::EmptyResult(_))), "{r:?}");
        // A face has no bodies.
        assert_eq!(K.body_count(&face).unwrap(), 0);
        assert!(K.bodies(&face).unwrap().is_empty());
        // The inputs are unchanged.
        assert_close(K.volume(&target).unwrap(), 1000.0);
        assert_close(K.volume(&far).unwrap(), 1000.0);
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

    /// Square 10 × 10 centered on the origin on `Workplane::XY`.
    fn sweep_square() -> OcctShape {
        K.make_face(
            &Workplane::XY,
            &polygon(&[[-5.0, -5.0], [5.0, -5.0], [5.0, 5.0], [-5.0, 5.0]]),
            &[],
        )
        .unwrap()
    }

    /// Circle r 2 centered on the origin on `Workplane::XY`.
    fn sweep_circle() -> OcctShape {
        K.make_face(&Workplane::XY, &circle([0.0, 0.0], 2.0), &[])
            .unwrap()
    }

    /// Quarter arc on `Workplane::XZ` from (0, 0, v0) down to (20, 0, v0 − 20).
    fn quarter_arc(v0: f64) -> Segment2 {
        Segment2::Arc {
            center: [20.0, v0],
            radius: 20.0,
            start_angle: PI,
            end_angle: 1.5 * PI,
        }
    }

    fn assert_profile_unchanged(p: &OcctShape, area: f64) {
        assert_eq!(K.face_count(p).unwrap(), 1);
        assert_close(K.surface_area(p).unwrap(), area);
    }

    #[test]
    fn sweep_square_along_a_straight_line() {
        let p = sweep_square();
        let path = [line([0.0, 0.0], [0.0, 30.0])];
        let s = K.sweep(&p, &Workplane::XZ, &path).unwrap();
        assert_close(K.volume(&s).unwrap(), 3000.0);
        assert_eq!(K.face_count(&s).unwrap(), 6);
        assert_bbox(&s, [-5.0, -5.0, 0.0], [5.0, 5.0, 30.0]);
        assert_profile_unchanged(&p, 100.0);
    }

    #[test]
    fn sweep_circle_along_a_quarter_arc() {
        let p = sweep_circle();
        let s = K.sweep(&p, &Workplane::XZ, &[quarter_arc(0.0)]).unwrap();
        // Pappus: area 4π times the centroid's path length 10π.
        let v = K.volume(&s).unwrap();
        let expected = 40.0 * PI * PI;
        assert!((v - expected).abs() <= 1e-4 * expected, "volume {v}");
        assert_eq!(K.face_count(&s).unwrap(), 3);
        // The end cap lies in the plane x = 20 with normal ±X.
        let end = (0..3)
            .filter_map(|i| K.face_plane(&s, face_sub(i)).ok())
            .find(|(o, n)| (o[0] - 20.0).abs() <= 1e-6 && (n[0].abs() - 1.0).abs() <= 1e-6);
        assert!(end.is_some(), "no planar face at x = 20 with normal ±X");
        assert_profile_unchanged(&p, 4.0 * PI);
    }

    #[test]
    fn sweep_circle_along_a_line_then_an_arc() {
        let p = sweep_circle();
        let path = [line([0.0, 0.0], [0.0, -10.0]), quarter_arc(-10.0)];
        let s = K.sweep(&p, &Workplane::XZ, &path).unwrap();
        let v = K.volume(&s).unwrap();
        let expected = 40.0 * PI + 40.0 * PI * PI;
        assert!((v - expected).abs() <= 1e-4 * expected, "volume {v}");
        assert_profile_unchanged(&p, 4.0 * PI);
    }

    #[test]
    fn sweep_rejects_bad_inputs_before_the_backend() {
        let invalid = |r: Result<OcctShape, KernelError>| {
            assert!(
                matches!(r, Err(KernelError::InvalidInput(_))),
                "expected InvalidInput, got {r:?}"
            );
        };
        let p = sweep_square();
        let up = line([0.0, 0.0], [0.0, 30.0]);
        let xz = &Workplane::XZ;
        invalid(K.sweep(&p, xz, &[]));
        invalid(K.sweep(&p, xz, &[circle([0.0, 0.0], 5.0).segments[0]]));
        invalid(K.sweep(&p, xz, &[quarter_arc_with_radius(0.0)]));
        invalid(K.sweep(&p, xz, &[line([0.0, 0.0], [0.0, f64::NAN])]));
        let skew = Workplane {
            x_dir: [1.0, 1.0, 0.0],
            ..Workplane::XZ
        };
        invalid(K.sweep(&p, &skew, &[up]));
        let zero = Workplane {
            normal: [0.0; 3],
            ..Workplane::XZ
        };
        invalid(K.sweep(&p, &zero, &[up]));
        invalid(K.sweep(&p, xz, &[up, line([0.0, 31.0], [5.0, 40.0])]));
        invalid(K.sweep(&p, xz, &[line([0.0, 5.0], [0.0, 30.0])]));
        let solid = K.make_box([-5.0, -5.0, 0.0], [10.0, 10.0, 10.0]).unwrap();
        invalid(K.sweep(&solid, xz, &[up]));
        let holed = K
            .make_face(
                &Workplane::XY,
                &polygon(&[[-5.0, -5.0], [5.0, -5.0], [5.0, 5.0], [-5.0, 5.0]]),
                &[circle([0.0, 0.0], 2.0)],
            )
            .unwrap();
        invalid(K.sweep(&holed, xz, &[up]));
        assert_profile_unchanged(&p, 100.0);
    }

    fn quarter_arc_with_radius(radius: f64) -> Segment2 {
        Segment2::Arc {
            center: [20.0, 0.0],
            radius,
            start_angle: PI,
            end_angle: 1.5 * PI,
        }
    }

    #[test]
    fn sweep_along_the_profile_plane_is_an_error() {
        let p = sweep_square();
        let r = K.sweep(&p, &Workplane::XZ, &[line([0.0, 0.0], [30.0, 0.0])]);
        assert!(
            matches!(
                r,
                Err(KernelError::Backend(_) | KernelError::InvalidInput(_))
            ),
            "expected an error, got {r:?}"
        );
        assert_profile_unchanged(&p, 100.0);
        // The profile is still usable.
        let s = K.extrude(&p, Extent::Distance(2.0)).unwrap();
        assert_close(K.volume(&s).unwrap(), 200.0);
    }

    /// On `Workplane::XZ`: up from (0, 0) to (0, 30), then a quarter arc of
    /// `radius` that bends towards +x (the M3-11c reproduction path).
    fn line_then_bend(radius: f64) -> [Segment2; 2] {
        [
            line([0.0, 0.0], [0.0, 30.0]),
            Segment2::Arc {
                center: [radius, 30.0],
                radius,
                start_angle: 0.5 * PI,
                end_angle: PI,
            },
        ]
    }

    #[test]
    fn sweep_refuses_a_bend_no_wider_than_the_profile() {
        let p = K
            .make_face(&Workplane::XY, &circle([0.0, 0.0], 5.0), &[])
            .unwrap();
        for r in [3.0, 5.0] {
            let res = K.sweep(&p, &Workplane::XZ, &line_then_bend(r));
            assert!(
                matches!(&res, Err(KernelError::InvalidInput(m))
                    if m.contains("arc segment 1") && m.contains(&format!("radius {r}"))),
                "radius {r}: {res:?}"
            );
        }
        let s = K.sweep(&p, &Workplane::XZ, &line_then_bend(20.0)).unwrap();
        let v = K.volume(&s).unwrap();
        let expected = PI * 25.0 * (30.0 + 10.0 * PI);
        assert!((v - expected).abs() <= 1e-4 * expected, "volume {v}");
        assert_profile_unchanged(&p, 25.0 * PI);

        // A 12 × 6 rectangle reaches √45 ≈ 6.7 from the path's start.
        let rect = K
            .make_face(
                &Workplane::XY,
                &polygon(&[[-6.0, -3.0], [6.0, -3.0], [6.0, 3.0], [-6.0, 3.0]]),
                &[],
            )
            .unwrap();
        for r in [5.0, 6.7] {
            let res = K.sweep(&rect, &Workplane::XZ, &line_then_bend(r));
            assert!(
                matches!(&res, Err(KernelError::InvalidInput(m)) if m.contains("arc segment 1")),
                "radius {r}: {res:?}"
            );
        }
        assert!(K.sweep(&rect, &Workplane::XZ, &line_then_bend(6.8)).is_ok());
        assert_profile_unchanged(&rect, 72.0);
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

    fn box_10x20x30() -> OcctShape {
        K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap()
    }

    fn edge(index: usize) -> SubShape {
        sub(TopoKind::Edge, index)
    }

    /// Treats each of the box's 12 edges on its own and checks the removed
    /// volume, which is `per_length · L` with L the edge length, so the
    /// lengths found must be 10, 20 and 30, four times each.
    fn assert_each_edge_removes(
        op: impl Fn(&OcctShape, &[SubShape]) -> Result<OcctShape, KernelError>,
        per_length: f64,
    ) {
        let b = box_10x20x30();
        assert_eq!(K.sub_shape_count(&b, TopoKind::Edge).unwrap(), 12);
        let mut lengths = Vec::new();
        for i in 0..12 {
            let r = op(&b, &[edge(i)]).unwrap();
            let removed = 6000.0 - K.volume(&r).unwrap();
            let l = removed / per_length;
            let nearest = [10.0, 20.0, 30.0]
                .into_iter()
                .min_by(|a: &f64, c: &f64| (a - l).abs().total_cmp(&(c - l).abs()))
                .unwrap();
            assert_close(removed, per_length * nearest);
            lengths.push(nearest as u32);
            let counts = KINDS.map(|k| K.sub_shape_count(&r, k).unwrap());
            assert_eq!(counts, [7, 15, 10], "edge {i}");
            assert_bbox(&r, [0.0; 3], [10.0, 20.0, 30.0]);
        }
        lengths.sort_unstable();
        assert_eq!(lengths, [10, 10, 10, 10, 20, 20, 20, 20, 30, 30, 30, 30]);
    }

    #[test]
    fn fillet_each_box_edge_radius_2() {
        assert_each_edge_removes(|s, e| K.fillet(s, e, 2.0), 4.0 * (1.0 - PI / 4.0));
    }

    /// The one sub-shape of `kind` in `s` whose bounding box is `min`–`max`
    /// (within 1e-6), if any.
    fn find_by_box(s: &OcctShape, kind: TopoKind, min: Point3, max: Point3) -> Option<SubShape> {
        let near = |a: Point3, b: Point3| (0..3).all(|i| (a[i] - b[i]).abs() < 1e-6);
        (0..K.sub_shape_count(s, kind).unwrap())
            .map(|index| sub(kind, index))
            .find(|&x| {
                let b = K.sub_shape_bounding_box(s, x).unwrap();
                near(b.min, min) && near(b.max, max)
            })
    }

    fn has_vertex(s: &OcctShape, p: Point3) -> bool {
        find_by_box(s, TopoKind::Vertex, p, p).is_some()
    }

    /// The box's vertical edge where x = 10 and y = 20 meet, and its vertices
    /// at z = 0 and z = 30.
    fn variable_fillet_inputs(b: &OcctShape) -> (SubShape, SubShape, SubShape) {
        let e = find_by_box(b, TopoKind::Edge, [10.0, 20.0, 0.0], [10.0, 20.0, 30.0]).unwrap();
        let low = find_by_box(b, TopoKind::Vertex, [10.0, 20.0, 0.0], [10.0, 20.0, 0.0]).unwrap();
        let high =
            find_by_box(b, TopoKind::Vertex, [10.0, 20.0, 30.0], [10.0, 20.0, 30.0]).unwrap();
        (e, low, high)
    }

    #[test]
    fn fillet_variable_2_to_4_from_the_bottom() {
        let b = box_10x20x30();
        let (e, low, _) = variable_fillet_inputs(&b);
        let r = K.fillet_variable(&b, e, low, 2.0, 4.0).unwrap();
        for p in [
            [8.0, 20.0, 0.0],
            [10.0, 18.0, 0.0],
            [6.0, 20.0, 30.0],
            [10.0, 16.0, 30.0],
        ] {
            assert!(has_vertex(&r, p), "no vertex at {p:?}");
        }
        let counts = KINDS.map(|k| K.sub_shape_count(&r, k).unwrap());
        assert_eq!(counts, [7, 15, 10]);
        assert_bbox(&r, [0.0; 3], [10.0, 20.0, 30.0]);
        let removed = 6000.0 - K.volume(&r).unwrap();
        let removed_at = |radius| 6000.0 - K.volume(&K.fillet(&b, &[e], radius).unwrap()).unwrap();
        let (lo, hi) = (removed_at(2.0), removed_at(4.0));
        assert_close(lo, 30.0 * 4.0 * (1.0 - PI / 4.0));
        assert_close(hi, 30.0 * 16.0 * (1.0 - PI / 4.0));
        assert!(lo < removed && removed < hi, "{lo} < {removed} < {hi}");
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_eq!(K.sub_shape_count(&b, TopoKind::Face).unwrap(), 6);
    }

    #[test]
    fn fillet_variable_start_at_the_top_swaps_the_radii() {
        let b = box_10x20x30();
        let (e, low, high) = variable_fillet_inputs(&b);
        let r = K.fillet_variable(&b, e, high, 2.0, 4.0).unwrap();
        assert!(has_vertex(&r, [6.0, 20.0, 0.0]));
        assert!(has_vertex(&r, [10.0, 16.0, 0.0]));
        assert!(has_vertex(&r, [8.0, 20.0, 30.0]));
        assert!(has_vertex(&r, [10.0, 18.0, 30.0]));
        // Mirror image of the bottom-start fillet: the same volume.
        let other = K.fillet_variable(&b, e, low, 2.0, 4.0).unwrap();
        assert_close(K.volume(&r).unwrap(), K.volume(&other).unwrap());
    }

    #[test]
    fn fillet_variable_with_equal_radii_matches_the_constant_fillet() {
        let b = box_10x20x30();
        let (e, low, _) = variable_fillet_inputs(&b);
        let r = K.fillet_variable(&b, e, low, 3.0, 3.0).unwrap();
        let c = K.fillet(&b, &[e], 3.0).unwrap();
        assert_close(K.volume(&r).unwrap(), K.volume(&c).unwrap());
    }

    #[test]
    fn fillet_variable_rejects_bad_inputs() {
        let b = box_10x20x30();
        let (e, low, _) = variable_fillet_inputs(&b);
        let invalid =
            |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(
                invalid(K.fillet_variable(&b, e, low, bad, 4.0)),
                "r_start {bad}"
            );
            assert!(
                invalid(K.fillet_variable(&b, e, low, 2.0, bad)),
                "r_end {bad}"
            );
        }
        let face = sub(TopoKind::Face, 0);
        assert!(invalid(K.fillet_variable(&b, face, low, 2.0, 4.0)));
        assert!(invalid(K.fillet_variable(&b, e, edge(0), 2.0, 4.0)));
        assert!(invalid(K.fillet_variable(&b, edge(12), low, 2.0, 4.0)));
        assert!(invalid(K.fillet_variable(
            &b,
            e,
            sub(TopoKind::Vertex, 8),
            2.0,
            4.0
        )));
        let off_edge = find_by_box(&b, TopoKind::Vertex, [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]).unwrap();
        let err = K.fillet_variable(&b, e, off_edge, 2.0, 4.0).unwrap_err();
        assert!(
            matches!(&err, KernelError::InvalidInput(m) if m.contains("not on edge")),
            "{err}"
        );
    }

    #[test]
    fn fillet_variable_too_large_is_an_error_and_keeps_the_input() {
        let b = box_10x20x30();
        let (e, low, _) = variable_fillet_inputs(&b);
        assert!(K.fillet_variable(&b, e, low, 2.0, 25.0).is_err());
        // Wider than the 10 mm face: OCCT's variable builder alone returns a
        // shape here; the constant-radius check must reject it.
        assert!(K.fillet_variable(&b, e, low, 12.0, 2.0).is_err());
        assert_close(K.volume(&b).unwrap(), 6000.0);
        let r = K.fillet_variable(&b, e, low, 2.0, 4.0).unwrap();
        assert_eq!(K.sub_shape_count(&r, TopoKind::Face).unwrap(), 7);
    }

    #[test]
    fn fillet_variable_is_reliable_on_several_threads_at_once() {
        // Each thread has its own box; without the lock in `ffi`, OCCT's
        // variable-radius builder failed intermittently here.
        let handles: Vec<_> = (0..8)
            .map(|_| {
                std::thread::spawn(|| {
                    let b = box_10x20x30();
                    let (e, low, _) = variable_fillet_inputs(&b);
                    (0..5)
                        .filter(|_| K.fillet_variable(&b, e, low, 2.0, 4.0).is_err())
                        .count()
                })
            })
            .collect();
        let failures: usize = handles.into_iter().map(|h| h.join().unwrap()).sum();
        assert_eq!(failures, 0, "{failures} of 40 concurrent calls failed");
    }

    #[test]
    fn chamfer_each_box_edge_distance_2() {
        assert_each_edge_removes(|s, e| K.chamfer(s, e, 2.0), 2.0);
    }

    #[test]
    fn fillet_two_opposite_edges_and_duplicates() {
        let b = box_10x20x30();
        let faces_of = |i| K.adjacent(&b, edge(i), TopoKind::Face).unwrap();
        let (i, j) = (0..12)
            .flat_map(|i| (i + 1..12).map(move |j| (i, j)))
            .find(|&(i, j)| faces_of(i).iter().all(|f| !faces_of(j).contains(f)))
            .expect("a box has edges that share no face");
        let removed = |s: &OcctShape| 6000.0 - K.volume(s).unwrap();
        let one = removed(&K.fillet(&b, &[edge(i)], 2.0).unwrap());
        let other = removed(&K.fillet(&b, &[edge(j)], 2.0).unwrap());
        let both = K.fillet(&b, &[edge(i), edge(j)], 2.0).unwrap();
        assert_eq!(K.sub_shape_count(&both, TopoKind::Face).unwrap(), 8);
        assert_close(removed(&both), one + other);
        assert_bbox(&both, [0.0; 3], [10.0, 20.0, 30.0]);
        let twice = K.fillet(&b, &[edge(i), edge(i)], 2.0).unwrap();
        assert_close(removed(&twice), one);
        let twice = K.chamfer(&b, &[edge(j), edge(j)], 2.0).unwrap();
        assert_close(
            removed(&twice),
            removed(&K.chamfer(&b, &[edge(j)], 2.0).unwrap()),
        );
    }

    #[test]
    fn fillet_and_chamfer_reject_bad_inputs() {
        let b = box_10x20x30();
        let invalid =
            |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(invalid(K.fillet(&b, &[edge(0)], bad)), "radius {bad}");
            assert!(invalid(K.chamfer(&b, &[edge(0)], bad)), "distance {bad}");
        }
        assert!(invalid(K.fillet(&b, &[], 2.0)));
        assert!(invalid(K.chamfer(&b, &[], 2.0)));
        for not_edge in [sub(TopoKind::Face, 0), sub(TopoKind::Vertex, 0)] {
            assert!(invalid(K.fillet(&b, &[edge(0), not_edge], 2.0)));
            assert!(invalid(K.chamfer(&b, &[not_edge], 2.0)));
        }
        assert!(invalid(K.fillet(&b, &[edge(12)], 2.0)));
        assert!(invalid(K.chamfer(&b, &[edge(0), edge(99)], 2.0)));
    }

    #[test]
    fn fillet_radius_too_large_is_an_error_and_keeps_the_input() {
        // A radius-r fillet sets back r along both faces of the edge. The
        // edges of length 20 and 30 each border a face 10 wide, so r = 15
        // can't fit there; the edges of length 10 border faces 20 and 30
        // wide, so r = 15 fits. r = 25 fits nowhere (every edge borders a
        // face at most 20 wide).
        let b = box_10x20x30();
        let per_length = |r: f64| r * r * (1.0 - PI / 4.0);
        let mut built = 0;
        for i in 0..12 {
            let small = K.fillet(&b, &[edge(i)], 2.0).unwrap();
            let length = (6000.0 - K.volume(&small).unwrap()) / per_length(2.0);
            let r15 = K.fillet(&b, &[edge(i)], 15.0);
            if (length - 10.0).abs() < 1e-3 {
                let r15 = r15.unwrap();
                assert_close(6000.0 - K.volume(&r15).unwrap(), per_length(15.0) * 10.0);
                built += 1;
            } else {
                assert!(
                    matches!(r15, Err(KernelError::Backend(_))),
                    "edge {i} (length {length}): radius 15 should fail"
                );
            }
            assert!(
                matches!(K.fillet(&b, &[edge(i)], 25.0), Err(KernelError::Backend(_))),
                "edge {i}: radius 25 should fail"
            );
        }
        assert_eq!(built, 4);
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
        // The input still works as an operand afterwards.
        assert!(K.fillet(&b, &[edge(0)], 2.0).is_ok());
    }

    /// Every (edge, reference face) pair of the 10 × 20 × 30 box: 12 edges,
    /// each with its 2 faces.
    fn edge_face_pairs(b: &OcctShape) -> Vec<EdgeOnFace> {
        (0..12)
            .flat_map(|i| {
                let faces = K.adjacent(b, edge(i), TopoKind::Face).unwrap();
                assert_eq!(faces.len(), 2, "edge {i}");
                faces.into_iter().map(move |face| EdgeOnFace {
                    edge: edge(i),
                    face,
                })
            })
            .collect()
    }

    /// Chamfers each of the 24 edge/face pairs of the box on its own with
    /// `size` and checks the removed volume, `per_length · L` with L the
    /// edge length (so the lengths found are 10, 20 and 30, eight times
    /// each), the sub-shape counts and the unchanged bounding box.
    fn assert_each_edge_face_removes(size: ChamferSize, per_length: f64) {
        let b = box_10x20x30();
        let mut lengths = Vec::new();
        for p in edge_face_pairs(&b) {
            let r = K.chamfer_asymmetric(&b, &[p], size).unwrap();
            let removed = 6000.0 - K.volume(&r).unwrap();
            let l = removed / per_length;
            let nearest = [10.0, 20.0, 30.0]
                .into_iter()
                .min_by(|a: &f64, c: &f64| (a - l).abs().total_cmp(&(c - l).abs()))
                .unwrap();
            assert_close(removed, per_length * nearest);
            lengths.push(nearest as u32);
            let counts = KINDS.map(|k| K.sub_shape_count(&r, k).unwrap());
            assert_eq!(counts, [7, 15, 10], "{p:?}");
            assert_bbox(&r, [0.0; 3], [10.0, 20.0, 30.0]);
        }
        lengths.sort_unstable();
        let mut expected = [10; 24];
        expected[8..16].fill(20);
        expected[16..].fill(30);
        assert_eq!(lengths, expected);
    }

    #[test]
    fn chamfer_two_distances_each_edge_and_face() {
        // d1·d2/2 per unit length.
        assert_each_edge_face_removes(ChamferSize::TwoDistances { d1: 2.0, d2: 4.0 }, 4.0);
    }

    #[test]
    fn chamfer_distance_angle_each_edge_and_face() {
        // distance · (distance · tan(angle)) / 2 per unit length.
        let size = ChamferSize::DistanceAngle {
            distance: 2.0,
            angle: PI / 6.0,
        };
        assert_each_edge_face_removes(size, 2.0 * (2.0 * (PI / 6.0).tan()) / 2.0);
    }

    #[test]
    fn asymmetric_chamfer_with_equal_setbacks_matches_chamfer() {
        let b = box_10x20x30();
        let removed = |s: &OcctShape| 6000.0 - K.volume(s).unwrap();
        for p in edge_face_pairs(&b) {
            let equal = removed(&K.chamfer(&b, &[p.edge], 2.0).unwrap());
            let at_45 = ChamferSize::DistanceAngle {
                distance: 2.0,
                angle: PI / 4.0,
            };
            let same = ChamferSize::TwoDistances { d1: 2.0, d2: 2.0 };
            for size in [at_45, same] {
                let r = K.chamfer_asymmetric(&b, &[p], size).unwrap();
                assert_close(removed(&r), equal);
            }
        }
    }

    /// The one face of `s` lying in the plane `axis = value`.
    fn face_in_plane(s: &OcctShape, axis: usize, value: f64) -> (SubShape, BoundingBox) {
        let found: Vec<_> = (0..K.face_count(s).unwrap())
            .map(|i| sub(TopoKind::Face, i))
            .map(|f| (f, K.sub_shape_bounding_box(s, f).unwrap()))
            .filter(|(_, bb)| {
                (bb.min[axis] - value).abs() < 1e-6 && (bb.max[axis] - value).abs() < 1e-6
            })
            .collect();
        assert_eq!(found.len(), 1, "faces in plane {axis} = {value}");
        found[0]
    }

    #[test]
    fn asymmetric_chamfer_setbacks_follow_the_reference_face() {
        // The edge where the top face (z = 30) meets the front face (y = 0).
        let b = box_10x20x30();
        let (top, _) = face_in_plane(&b, 2, 30.0);
        let (front, _) = face_in_plane(&b, 1, 0.0);
        let top_front = (0..12)
            .map(edge)
            .find(|&e| {
                let faces = K.adjacent(&b, e, TopoKind::Face).unwrap();
                faces.contains(&top) && faces.contains(&front)
            })
            .unwrap();
        let tan30 = (PI / 6.0).tan();
        let two = ChamferSize::TwoDistances { d1: 2.0, d2: 4.0 };
        let da = ChamferSize::DistanceAngle {
            distance: 2.0,
            angle: PI / 6.0,
        };
        // (size, reference face, top face's y.min, front face's z.max)
        let cases = [
            (two, top, 2.0, 26.0),
            (da, top, 2.0, 30.0 - 2.0 * tan30),
            (two, front, 4.0, 28.0),
            (da, front, 2.0 * tan30, 28.0),
        ];
        for (size, face, y_min, z_max) in cases {
            let p = EdgeOnFace {
                edge: top_front,
                face,
            };
            let r = K.chamfer_asymmetric(&b, &[p], size).unwrap();
            let (_, t) = face_in_plane(&r, 2, 30.0);
            assert_close(t.min[1], y_min);
            assert_close(t.max[1], 20.0);
            let (_, f) = face_in_plane(&r, 1, 0.0);
            assert_close(f.min[2], 0.0);
            assert_close(f.max[2], z_max);
            assert_bbox(&r, [0.0; 3], [10.0, 20.0, 30.0]);
        }
    }

    #[test]
    fn asymmetric_chamfer_rejects_bad_inputs() {
        let b = box_10x20x30();
        let invalid =
            |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        let pairs = edge_face_pairs(&b);
        let p = pairs[0];
        let two = |d1, d2| ChamferSize::TwoDistances { d1, d2 };
        let da = |distance, angle| ChamferSize::DistanceAngle { distance, angle };
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(invalid(K.chamfer_asymmetric(&b, &[p], two(bad, 2.0))));
            assert!(invalid(K.chamfer_asymmetric(&b, &[p], two(2.0, bad))));
            assert!(invalid(K.chamfer_asymmetric(&b, &[p], da(bad, 0.5))));
        }
        for bad in [0.0, -0.5, PI / 2.0, 2.0, f64::NAN, f64::INFINITY] {
            assert!(invalid(K.chamfer_asymmetric(&b, &[p], da(2.0, bad))));
        }
        let size = two(2.0, 4.0);
        assert!(invalid(K.chamfer_asymmetric(&b, &[], size)));
        for wrong in [sub(TopoKind::Face, 0), sub(TopoKind::Vertex, 0)] {
            let e = EdgeOnFace { edge: wrong, ..p };
            assert!(invalid(K.chamfer_asymmetric(&b, &[e], size)));
        }
        for wrong in [edge(0), sub(TopoKind::Vertex, 0)] {
            let e = EdgeOnFace { face: wrong, ..p };
            assert!(invalid(K.chamfer_asymmetric(&b, &[e], size)));
        }
        let out_of_range = [
            EdgeOnFace {
                edge: edge(12),
                ..p
            },
            EdgeOnFace {
                face: sub(TopoKind::Face, 6),
                ..p
            },
        ];
        for e in out_of_range {
            assert!(invalid(K.chamfer_asymmetric(&b, &[e], size)));
        }
        let not_adjacent = (0..6)
            .map(|i| sub(TopoKind::Face, i))
            .find(|f| !K.adjacent(&b, p.edge, TopoKind::Face).unwrap().contains(f))
            .unwrap();
        let e = EdgeOnFace {
            face: not_adjacent,
            ..p
        };
        assert!(invalid(K.chamfer_asymmetric(&b, &[e], size)));
        // The same edge with both of its faces.
        assert_eq!(pairs[1].edge, p.edge);
        assert!(invalid(K.chamfer_asymmetric(&b, &[p, pairs[1]], size)));
        // An identical entry twice counts once.
        let once = K.chamfer_asymmetric(&b, &[p], size).unwrap();
        let twice = K.chamfer_asymmetric(&b, &[p, p], size).unwrap();
        assert_close(K.volume(&twice).unwrap(), K.volume(&once).unwrap());
    }

    #[test]
    fn asymmetric_chamfer_too_large_is_an_error_and_keeps_the_input() {
        // A 35 setback is wider than every face of the box.
        let b = box_10x20x30();
        let size = ChamferSize::TwoDistances { d1: 2.0, d2: 35.0 };
        for p in edge_face_pairs(&b) {
            let r = K.chamfer_asymmetric(&b, &[p], size);
            assert!(matches!(r, Err(KernelError::Backend(_))), "{p:?}");
        }
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert!(K
            .chamfer_asymmetric(
                &b,
                &edge_face_pairs(&b)[..1],
                ChamferSize::TwoDistances { d1: 2.0, d2: 4.0 }
            )
            .is_ok());
    }

    /// The #90 plate's loops: the 40 × 25 rectangle, then the r5 hole.
    fn tracked_plate_face() -> (OcctShape, Vec<Vec<SubShape>>) {
        K.make_face_tracked(
            &Workplane::XY,
            &rect(40.0, 25.0),
            &[circle([20.0, 12.5], 5.0)],
        )
        .unwrap()
    }

    fn assert_sub_bbox(s: &OcctShape, of: SubShape, min: Point3, max: Point3) {
        let bb = K.sub_shape_bounding_box(s, of).unwrap();
        for axis in 0..3 {
            assert_close(bb.min[axis], min[axis]);
            assert_close(bb.max[axis], max[axis]);
        }
    }

    /// Whether faces `a` and `b` of `s` share an edge.
    fn faces_touch(s: &OcctShape, a: SubShape, b: SubShape) -> bool {
        let ea = K.adjacent(s, a, TopoKind::Edge).unwrap();
        let eb = K.adjacent(s, b, TopoKind::Edge).unwrap();
        ea.iter().any(|e| eb.contains(e))
    }

    #[test]
    fn make_face_tracked_maps_each_segment_to_its_edge() {
        let (face, loops) = tracked_plate_face();
        assert_eq!(loops.iter().map(Vec::len).collect::<Vec<_>>(), [4, 1]);
        let mut all: Vec<usize> = loops.iter().flatten().map(|e| e.index).collect();
        assert!(loops.iter().flatten().all(|e| e.kind == TopoKind::Edge));
        all.sort_unstable();
        assert_eq!(all, (0..5).collect::<Vec<_>>());
        assert_eq!(K.sub_shape_count(&face, TopoKind::Edge).unwrap(), 5);
        assert_sub_bbox(&face, loops[0][0], [0.0; 3], [40.0, 0.0, 0.0]);
        assert_sub_bbox(&face, loops[0][1], [40.0, 0.0, 0.0], [40.0, 25.0, 0.0]);
        assert_sub_bbox(&face, loops[0][2], [0.0, 25.0, 0.0], [40.0, 25.0, 0.0]);
        assert_sub_bbox(&face, loops[0][3], [0.0; 3], [0.0, 25.0, 0.0]);
        assert_sub_bbox(&face, loops[1][0], [15.0, 7.5, 0.0], [25.0, 17.5, 0.0]);
        let untracked = plate_face(&Workplane::XY);
        for kind in KINDS {
            assert_eq!(
                K.sub_shape_count(&face, kind).unwrap(),
                K.sub_shape_count(&untracked, kind).unwrap()
            );
        }
    }

    #[test]
    fn make_face_tracked_follows_the_callers_order_on_a_clockwise_loop() {
        // Clockwise input: the kernel reverses the loop, but segment k must
        // still map to the edge built from segment k.
        let cw = polygon(&[[0.0, 0.0], [0.0, 25.0], [40.0, 25.0], [40.0, 0.0]]);
        let (face, loops) = K.make_face_tracked(&Workplane::XY, &cw, &[]).unwrap();
        assert_eq!(loops.len(), 1);
        assert_sub_bbox(&face, loops[0][0], [0.0; 3], [0.0, 25.0, 0.0]);
        assert_sub_bbox(&face, loops[0][1], [0.0, 25.0, 0.0], [40.0, 25.0, 0.0]);
        assert_sub_bbox(&face, loops[0][2], [40.0, 0.0, 0.0], [40.0, 25.0, 0.0]);
        assert_sub_bbox(&face, loops[0][3], [0.0; 3], [40.0, 0.0, 0.0]);
    }

    #[test]
    fn extrude_tracked_plate_caps_and_sides() {
        let (face, loops) = tracked_plate_face();
        let (solid, h) = K.extrude_tracked(&face, Extent::Distance(10.0)).unwrap();
        assert_close(K.volume(&solid).unwrap(), PLATE_VOLUME);
        assert_close(PLATE_VOLUME, 9214.6018);
        assert_sub_bbox(&solid, h.start_cap, [0.0; 3], [40.0, 25.0, 0.0]);
        assert_sub_bbox(&solid, h.end_cap, [0.0, 0.0, 10.0], [40.0, 25.0, 10.0]);
        assert_eq!(h.sides.len(), 5);
        let side_of = |edge: SubShape| h.sides.iter().find(|(e, _)| *e == edge).unwrap().1;
        let first = side_of(loops[0][0]);
        assert_sub_bbox(&solid, first, [0.0; 3], [40.0, 0.0, 10.0]);
        let hole = side_of(loops[1][0]);
        assert_sub_bbox(&solid, hole, [15.0, 7.5, 0.0], [25.0, 17.5, 10.0]);
        assert_eq!(K.adjacent(&solid, hole, TopoKind::Edge).unwrap().len(), 3);

        let mut faces: Vec<SubShape> = h.sides.iter().map(|(_, f)| *f).collect();
        faces.extend([h.start_cap, h.end_cap]);
        faces.sort_unstable();
        faces.dedup();
        assert_eq!(faces.len(), 7, "caps and sides are 7 distinct faces");
        for (_, side) in &h.sides {
            assert!(faces_touch(&solid, *side, h.start_cap));
            assert!(faces_touch(&solid, *side, h.end_cap));
        }
        assert!(faces_touch(&solid, first, side_of(loops[0][1])));
        assert!(faces_touch(&solid, first, side_of(loops[0][3])));
        assert!(!faces_touch(&solid, first, side_of(loops[0][2])));
    }

    #[test]
    fn extrude_tracked_symmetric_caps() {
        let (face, _) = tracked_plate_face();
        let (solid, h) = K.extrude_tracked(&face, Extent::Symmetric(10.0)).unwrap();
        assert_sub_bbox(&solid, h.start_cap, [0.0, 0.0, -5.0], [40.0, 25.0, -5.0]);
        assert_sub_bbox(&solid, h.end_cap, [0.0, 0.0, 5.0], [40.0, 25.0, 5.0]);
    }

    #[test]
    fn tracked_and_untracked_extrudes_agree() {
        let (face, _) = tracked_plate_face();
        for extent in [
            Extent::Distance(10.0),
            Extent::Distance(-10.0),
            Extent::Symmetric(10.0),
        ] {
            let (tracked, _) = K.extrude_tracked(&face, extent).unwrap();
            let plain = K.extrude(&plate_face(&Workplane::XY), extent).unwrap();
            assert_close(K.volume(&tracked).unwrap(), K.volume(&plain).unwrap());
            for kind in KINDS {
                assert_eq!(
                    K.sub_shape_count(&tracked, kind).unwrap(),
                    K.sub_shape_count(&plain, kind).unwrap(),
                    "{extent:?} {kind:?}"
                );
            }
        }
    }

    #[test]
    fn tracked_calls_reject_bad_inputs() {
        let invalid = |e: KernelError| matches!(e, KernelError::InvalidInput(_));
        let mut open = rect(1.0, 1.0);
        open.segments[2] = line([1.0, 1.0 + 1e-6], [0.0, 1.0]);
        assert!(invalid(
            K.make_face_tracked(&Workplane::XY, &open, &[]).unwrap_err()
        ));
        let (face, _) = tracked_plate_face();
        assert!(invalid(
            K.extrude_tracked(&face, Extent::Distance(0.0)).unwrap_err()
        ));
        assert!(invalid(
            K.extrude_tracked(&face, Extent::ThroughAll(crate::ThroughDirection::Normal))
                .unwrap_err()
        ));
        let f0 = sub(TopoKind::Face, 0);
        for extent in [Extent::ToFace(f0), Extent::BetweenFaces(f0, f0)] {
            assert!(invalid(K.extrude_tracked(&face, extent).unwrap_err()));
        }
        let solid = unit_box([0.0; 3], 1.0);
        assert!(matches!(
            K.extrude_tracked(&solid, Extent::Distance(1.0)),
            Err(KernelError::Backend(_))
        ));
        assert!(K
            .sub_shape_bounding_box(&solid, sub(TopoKind::Face, 5))
            .is_ok());
        for (kind, n) in KINDS.into_iter().zip([6, 12, 8]) {
            assert!(invalid(
                K.sub_shape_bounding_box(&solid, sub(kind, n)).unwrap_err()
            ));
        }
    }

    // ---- M3-04d: tracked booleans ----

    fn face_boxes(s: &OcctShape) -> Vec<BoundingBox> {
        (0..K.sub_shape_count(s, TopoKind::Face).unwrap())
            .map(|i| K.sub_shape_bounding_box(s, sub(TopoKind::Face, i)).unwrap())
            .collect()
    }

    fn near(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-6
    }

    /// The one face of `s` lying in the plane `axis` = `value` (axis 0/1/2 = x/y/z).
    fn face_on(s: &OcctShape, axis: usize, value: f64) -> usize {
        let hits: Vec<usize> = face_boxes(s)
            .iter()
            .enumerate()
            .filter(|(_, b)| near(b.min[axis], value) && near(b.max[axis], value))
            .map(|(i, _)| i)
            .collect();
        assert_eq!(hits.len(), 1, "faces on axis {axis} = {value}: {hits:?}");
        hits[0]
    }

    fn images(h: &OpHistory, operand: Operand, index: usize) -> &[SubShape] {
        let (_, _, imgs) = h
            .modified
            .iter()
            .find(|(o, f, _)| *o == operand && f.index == index)
            .unwrap();
        imgs
    }

    fn assert_box(b: &BoundingBox, min: Point3, max: Point3) {
        for k in 0..3 {
            assert!(
                near(b.min[k], min[k]) && near(b.max[k], max[k]),
                "{b:?} != {min:?}..{max:?}"
            );
        }
    }

    /// Runs `op` tracked and untracked and checks what holds for every case:
    /// same volume and face count, one `modified` entry per input face in
    /// order, every result face the image of exactly one input face, sorted
    /// images, and no `generated` entries.
    fn tracked(op: BooleanOp, a: &OcctShape, b: &OcctShape) -> (OcctShape, OpHistory) {
        let (r, h) = K.boolean_tracked(op, a, b).unwrap();
        let plain = match op {
            BooleanOp::Union => K.union(a, b),
            BooleanOp::Cut => K.cut(a, b),
            BooleanOp::Intersect => K.intersect(a, b),
        }
        .unwrap();
        assert_close(K.volume(&r).unwrap(), K.volume(&plain).unwrap());
        let n = K.face_count(&r).unwrap();
        assert_eq!(n, K.face_count(&plain).unwrap());
        let n_a = K.face_count(a).unwrap();
        let n_b = K.face_count(b).unwrap();
        let expected: Vec<(Operand, SubShape)> = (0..n_a)
            .map(|i| (Operand::A, sub(TopoKind::Face, i)))
            .chain((0..n_b).map(|i| (Operand::B, sub(TopoKind::Face, i))))
            .collect();
        let got: Vec<(Operand, SubShape)> = h.modified.iter().map(|(o, f, _)| (*o, *f)).collect();
        assert_eq!(got, expected);
        let mut owners = vec![0usize; n];
        for (_, _, imgs) in &h.modified {
            assert!(imgs.windows(2).all(|w| w[0].index < w[1].index));
            for img in imgs {
                assert_eq!(img.kind, TopoKind::Face);
                owners[img.index] += 1;
            }
        }
        assert_eq!(owners, vec![1; n], "result faces owned once each");
        assert!(h.generated.is_empty());
        (r, h)
    }

    #[test]
    fn tracked_cut_splits_the_top_face() {
        let a = K.make_box([0.0; 3], [40.0, 25.0, 10.0]).unwrap();
        let b = K.make_box([18.0, -1.0, 6.0], [4.0, 27.0, 5.0]).unwrap();
        let (r, h) = tracked(BooleanOp::Cut, &a, &b);
        assert_close(K.volume(&r).unwrap(), 9600.0);
        assert_eq!(K.face_count(&r).unwrap(), 10);
        let boxes = face_boxes(&r);

        let top = images(&h, Operand::A, face_on(&a, 2, 10.0));
        assert_eq!(top.len(), 2);
        let mut halves: Vec<&BoundingBox> = top.iter().map(|f| &boxes[f.index]).collect();
        halves.sort_by(|p, q| p.min[0].total_cmp(&q.min[0]));
        assert_box(halves[0], [0.0, 0.0, 10.0], [18.0, 25.0, 10.0]);
        assert_box(halves[1], [22.0, 0.0, 10.0], [40.0, 25.0, 10.0]);
        assert_eq!(images(&h, Operand::A, face_on(&a, 2, 0.0)).len(), 1);

        for (axis, value) in [(2, 11.0), (1, -1.0), (1, 26.0)] {
            assert!(images(&h, Operand::B, face_on(&b, axis, value)).is_empty());
        }
        for (axis, value) in [(2, 6.0), (0, 18.0), (0, 22.0)] {
            assert_eq!(images(&h, Operand::B, face_on(&b, axis, value)).len(), 1);
        }
    }

    #[test]
    fn tracked_union_absorbs_an_inner_boss() {
        let a = K.make_box([0.0; 3], [40.0, 25.0, 20.0]).unwrap();
        let b = K.make_cylinder([20.0, 12.5, 10.0], 5.0, 5.0).unwrap();
        let (r, h) = tracked(BooleanOp::Union, &a, &b);
        assert_close(K.volume(&r).unwrap(), 20000.0);
        assert_eq!(K.face_count(&r).unwrap(), 6);
        for (o, _, imgs) in &h.modified {
            match o {
                Operand::A => assert_eq!(imgs.len(), 1),
                Operand::B => assert!(imgs.is_empty()),
            }
        }
    }

    #[test]
    fn tracked_union_with_an_embedded_boss() {
        let a = K.make_box([0.0; 3], [40.0, 25.0, 10.0]).unwrap();
        let b = K.make_cylinder([20.0, 12.5, 5.0], 5.0, 10.0).unwrap();
        let (r, h) = tracked(BooleanOp::Union, &a, &b);
        assert_close(K.volume(&r).unwrap(), 10000.0 + 125.0 * PI);
        assert_eq!(K.face_count(&r).unwrap(), 8);
        assert!(images(&h, Operand::B, face_on(&b, 2, 5.0)).is_empty());
        assert_eq!(images(&h, Operand::B, face_on(&b, 2, 15.0)).len(), 1);
        let n_b = K.face_count(&b).unwrap();
        let side = (0..n_b)
            .find(|&i| i != face_on(&b, 2, 5.0) && i != face_on(&b, 2, 15.0))
            .unwrap();
        assert_eq!(images(&h, Operand::B, side).len(), 1);
        assert_eq!(images(&h, Operand::A, face_on(&a, 2, 10.0)).len(), 1);
    }

    #[test]
    fn tracked_intersect_keeps_three_faces_of_each() {
        let a = K.make_box([0.0; 3], [40.0, 25.0, 10.0]).unwrap();
        let b = K.make_box([30.0, 15.0, 5.0], [20.0, 20.0, 20.0]).unwrap();
        let (r, h) = tracked(BooleanOp::Intersect, &a, &b);
        assert_close(K.volume(&r).unwrap(), 500.0);
        assert_eq!(K.face_count(&r).unwrap(), 6);
        for operand in [Operand::A, Operand::B] {
            let counts: Vec<usize> = h
                .modified
                .iter()
                .filter(|(o, _, _)| *o == operand)
                .map(|(_, _, imgs)| imgs.len())
                .collect();
            assert_eq!(counts.iter().filter(|&&n| n == 1).count(), 3, "{operand:?}");
            assert_eq!(counts.iter().filter(|&&n| n == 0).count(), 3, "{operand:?}");
        }
        for (axis, value) in [(0, 40.0), (1, 25.0), (2, 10.0)] {
            assert_eq!(images(&h, Operand::A, face_on(&a, axis, value)).len(), 1);
        }
        for (axis, value) in [(0, 30.0), (1, 15.0), (2, 5.0)] {
            assert_eq!(images(&h, Operand::B, face_on(&b, axis, value)).len(), 1);
        }
    }

    #[test]
    fn tracked_empty_result_matches_the_untracked_call() {
        let a = K.make_box([1.0; 3], [2.0, 2.0, 2.0]).unwrap();
        let b = K.make_box([0.0; 3], [5.0, 5.0, 5.0]).unwrap();
        let plain = K.cut(&a, &b);
        let tracked = K.boolean_tracked(BooleanOp::Cut, &a, &b);
        match (plain, tracked) {
            (Err(p), Err(t)) => assert_eq!(p, t),
            (Ok(p), Ok((t, h))) => {
                assert_eq!(K.face_count(&t).unwrap(), K.face_count(&p).unwrap());
                assert_eq!(K.face_count(&t).unwrap(), 0);
                assert_eq!(h.modified.len(), 12);
                assert!(h.modified.iter().all(|(_, _, imgs)| imgs.is_empty()));
            }
            (p, t) => panic!("untracked {p:?} vs tracked {:?}", t.map(|(_, h)| h)),
        }
    }

    // ---- M3-10b: tracked combine ----

    /// Runs `combine_tracked` and checks what holds for every case: the same
    /// solid as `combine` (volume, body count, face count), one entry per
    /// input face, and every image a sorted, distinct, in-range result face.
    fn combined(
        op: BooleanOp,
        target: &OcctShape,
        tools: &[OcctShape],
    ) -> (OcctShape, CombineHistory) {
        let (r, h) = K.combine_tracked(op, target, tools).unwrap();
        let plain = K.combine(op, target, tools).unwrap();
        assert_close(K.volume(&r).unwrap(), K.volume(&plain).unwrap());
        assert_eq!(K.body_count(&r).unwrap(), K.body_count(&plain).unwrap());
        let n = K.face_count(&r).unwrap();
        assert_eq!(n, K.face_count(&plain).unwrap());
        assert_eq!(h.target.len(), K.face_count(target).unwrap());
        assert_eq!(h.tools.len(), tools.len());
        for (t, tool) in h.tools.iter().zip(tools) {
            assert_eq!(t.len(), K.face_count(tool).unwrap());
        }
        for imgs in h.target.iter().chain(h.tools.iter().flatten()) {
            assert!(imgs.windows(2).all(|w| w[0].index < w[1].index), "{imgs:?}");
            assert!(imgs.iter().all(|f| f.kind == TopoKind::Face && f.index < n));
        }
        (r, h)
    }

    /// Bounding boxes of `imgs` in `r`, sorted by minimum x.
    fn image_boxes(r: &OcctShape, imgs: &[SubShape]) -> Vec<BoundingBox> {
        let mut boxes: Vec<BoundingBox> = imgs
            .iter()
            .map(|&f| K.sub_shape_bounding_box(r, f).unwrap())
            .collect();
        boxes.sort_by(|p, q| p.min[0].total_cmp(&q.min[0]));
        boxes
    }

    fn assert_box_tol(b: &BoundingBox, min: Point3, max: Point3, tol: f64) {
        for k in 0..3 {
            assert!(
                (b.min[k] - min[k]).abs() <= tol && (b.max[k] - max[k]).abs() <= tol,
                "{b:?} != {min:?}..{max:?}"
            );
        }
    }

    /// The lateral face of a cylinder: the one face not on either end plane.
    fn cylinder_wall(cyl: &OcctShape, z0: f64, z1: f64) -> usize {
        let ends = [face_on(cyl, 2, z0), face_on(cyl, 2, z1)];
        let sides: Vec<usize> = (0..K.face_count(cyl).unwrap())
            .filter(|i| !ends.contains(i))
            .collect();
        assert_eq!(sides.len(), 1);
        sides[0]
    }

    #[test]
    fn combine_tracked_with_one_tool_matches_boolean_tracked() {
        let a = unit_box([0.0; 3], 10.0);
        let b = unit_box([5.0, 0.0, 0.0], 10.0);
        let (r, h) = combined(BooleanOp::Cut, &a, std::slice::from_ref(&b));
        let (r1, h1) = K.boolean_tracked(BooleanOp::Cut, &a, &b).unwrap();
        assert_eq!(K.face_count(&r).unwrap(), K.face_count(&r1).unwrap());
        for (i, imgs) in h.target.iter().enumerate() {
            assert_eq!(
                imgs.as_slice(),
                images(&h1, Operand::A, i),
                "target face {i}"
            );
        }
        for (i, imgs) in h.tools[0].iter().enumerate() {
            assert_eq!(imgs.as_slice(), images(&h1, Operand::B, i), "tool face {i}");
        }
    }

    #[test]
    fn combine_tracked_join_follows_the_outer_end_faces() {
        let target = unit_box([0.0; 3], 10.0);
        let tools = [
            unit_box([10.0, 0.0, 0.0], 10.0),
            unit_box([20.0, 0.0, 0.0], 10.0),
        ];
        let (r, h) = combined(BooleanOp::Union, &target, &tools);
        assert_close(K.volume(&r).unwrap(), 3000.0);
        let start = image_boxes(&r, &h.target[face_on(&target, 0, 0.0)]);
        assert_eq!(start.len(), 1);
        assert_box_tol(&start[0], [0.0; 3], [0.0, 10.0, 10.0], 1e-6);
        assert!(h.target[face_on(&target, 0, 10.0)].is_empty());
        assert!(h.tools[0][face_on(&tools[0], 0, 10.0)].is_empty());
        assert!(h.tools[0][face_on(&tools[0], 0, 20.0)].is_empty());
        let end = image_boxes(&r, &h.tools[1][face_on(&tools[1], 0, 30.0)]);
        assert_eq!(end.len(), 1);
        assert_box_tol(&end[0], [30.0, 0.0, 0.0], [30.0, 10.0, 10.0], 1e-6);
        // The inputs are unchanged.
        assert_close(K.volume(&target).unwrap(), 1000.0);
    }

    #[test]
    fn combine_tracked_cut_follows_each_cylinder_wall() {
        let bar = K.make_box([0.0; 3], [30.0, 10.0, 10.0]).unwrap();
        let pins = [
            K.make_cylinder([5.0, 5.0, 0.0], 2.0, 10.0).unwrap(),
            K.make_cylinder([25.0, 5.0, 0.0], 2.0, 10.0).unwrap(),
        ];
        let (r, h) = combined(BooleanOp::Cut, &bar, &pins);
        assert_close(K.volume(&r).unwrap(), 3000.0 - 80.0 * PI);
        let top = image_boxes(&r, &h.target[face_on(&bar, 2, 10.0)]);
        assert_eq!(top.len(), 1);
        assert_box_tol(&top[0], [0.0, 0.0, 10.0], [30.0, 10.0, 10.0], 1e-6);
        for (i, x) in [5.0, 25.0].into_iter().enumerate() {
            let wall = image_boxes(&r, &h.tools[i][cylinder_wall(&pins[i], 0.0, 10.0)]);
            assert_eq!(wall.len(), 1, "pin {i}");
            assert_box_tol(&wall[0], [x - 2.0, 3.0, 0.0], [x + 2.0, 7.0, 10.0], 1e-3);
            for z in [0.0, 10.0] {
                assert!(
                    h.tools[i][face_on(&pins[i], 2, z)].is_empty(),
                    "pin {i} end {z}"
                );
            }
        }
    }

    #[test]
    fn combine_tracked_cut_that_splits_the_target() {
        let bar = K.make_box([0.0; 3], [30.0, 10.0, 10.0]).unwrap();
        let slab = K.make_box([10.0, -1.0, -1.0], [10.0, 12.0, 12.0]).unwrap();
        let (r, h) = combined(BooleanOp::Cut, &bar, std::slice::from_ref(&slab));
        assert_eq!(K.body_count(&r).unwrap(), 2);
        let top = image_boxes(&r, &h.target[face_on(&bar, 2, 10.0)]);
        assert_eq!(top.len(), 2);
        assert_box_tol(&top[0], [0.0, 0.0, 10.0], [10.0, 10.0, 10.0], 1e-6);
        assert_box_tol(&top[1], [20.0, 0.0, 10.0], [30.0, 10.0, 10.0], 1e-6);
        for x in [10.0, 20.0] {
            let wall = image_boxes(&r, &h.tools[0][face_on(&slab, 0, x)]);
            assert_eq!(wall.len(), 1, "slab face x = {x}");
            assert_box_tol(&wall[0], [x, 0.0, 0.0], [x, 10.0, 10.0], 1e-6);
        }
    }

    #[test]
    fn combine_tracked_rejects_bad_inputs_and_empty_results() {
        let target = unit_box([0.0; 3], 10.0);
        let tool = unit_box([5.0, 0.0, 0.0], 10.0);
        let square = polygon(&[[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]]);
        let face = K.make_face(&Workplane::XY, &square, &[]).unwrap();
        let face_tool = K.make_face(&Workplane::XY, &square, &[]).unwrap();
        let kind = |r: Result<(OcctShape, CombineHistory), KernelError>| r.map(|_| ()).unwrap_err();
        let invalid = |e: KernelError, what: &str| {
            assert!(matches!(e, KernelError::InvalidInput(_)), "{what}: {e:?}");
        };
        invalid(
            kind(K.combine_tracked(BooleanOp::Union, &target, &[])),
            "no tools",
        );
        invalid(
            kind(K.combine_tracked(BooleanOp::Cut, &face, std::slice::from_ref(&tool))),
            "face target",
        );
        invalid(
            kind(K.combine_tracked(BooleanOp::Union, &target, &[tool, face_tool])),
            "face tool",
        );
        let far = unit_box([50.0, 0.0, 0.0], 10.0);
        let e = kind(K.combine_tracked(BooleanOp::Intersect, &target, std::slice::from_ref(&far)));
        assert!(matches!(e, KernelError::EmptyResult(_)), "{e:?}");
        let tools = [
            unit_box([5.0, 0.0, 0.0], 10.0),
            unit_box([-20.0, 0.0, 0.0], 10.0),
        ];
        let e = kind(K.combine_tracked(BooleanOp::Intersect, &target, &tools));
        assert!(matches!(e, KernelError::EmptyResult(_)), "{e:?}");
        assert_close(K.volume(&target).unwrap(), 1000.0);
        assert_close(K.volume(&far).unwrap(), 1000.0);
    }

    /// The face of `s` that lies in the plane `axis = value` (both ends of
    /// its bounding box on that axis within 1e-6).
    fn planar_face(s: &OcctShape, axis: usize, value: f64) -> SubShape {
        let n = K.sub_shape_count(s, TopoKind::Face).unwrap();
        let hits: Vec<SubShape> = (0..n)
            .map(|index| SubShape {
                kind: TopoKind::Face,
                index,
            })
            .filter(|&f| {
                let b = K.sub_shape_bounding_box(s, f).unwrap();
                (b.min[axis] - value).abs() <= 1e-6 && (b.max[axis] - value).abs() <= 1e-6
            })
            .collect();
        assert_eq!(hits.len(), 1, "faces in plane {axis} = {value}");
        hits[0]
    }

    fn assert_box_close(b: BoundingBox, min: Point3, max: Point3) {
        for i in 0..3 {
            assert!((b.min[i] - min[i]).abs() <= 1e-6, "min {:?}", b.min);
            assert!((b.max[i] - max[i]).abs() <= 1e-6, "max {:?}", b.max);
        }
    }

    #[test]
    fn shell_box_with_the_top_open() {
        let b = box_10x20x30();
        let top = planar_face(&b, 2, 30.0);
        let shelled = K.shell(&b, &[top], 1.0).unwrap();
        assert_close(K.volume(&shelled).unwrap(), 6000.0 - 8.0 * 18.0 * 29.0);
        assert_box_close(
            K.bounding_box(&shelled).unwrap(),
            [0.0, 0.0, 0.0],
            [10.0, 20.0, 30.0],
        );
        assert_eq!(K.face_count(&shelled).unwrap(), 11);
        // The same face listed twice counts once.
        let twice = K.shell(&b, &[top, top], 1.0).unwrap();
        assert_close(K.volume(&twice).unwrap(), 1824.0);
        // The input solid is unchanged.
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
    }

    #[test]
    fn shell_box_with_top_and_bottom_open() {
        let b = box_10x20x30();
        let open = [planar_face(&b, 2, 30.0), planar_face(&b, 2, 0.0)];
        let shelled = K.shell(&b, &open, 1.0).unwrap();
        assert_close(K.volume(&shelled).unwrap(), 30.0 * (200.0 - 8.0 * 18.0));
        assert_eq!(K.face_count(&shelled).unwrap(), 10);
        assert_box_close(
            K.bounding_box(&shelled).unwrap(),
            [0.0, 0.0, 0.0],
            [10.0, 20.0, 30.0],
        );
    }

    #[test]
    fn shell_cylinder_cup() {
        let c = K.make_cylinder([0.0, 0.0, 0.0], 10.0, 20.0).unwrap();
        let before = K.bounding_box(&c).unwrap();
        let top = planar_face(&c, 2, 20.0);
        let cup = K.shell(&c, &[top], 2.0).unwrap();
        assert_close(K.volume(&cup).unwrap(), 848.0 * PI);
        assert_box_close(K.bounding_box(&cup).unwrap(), before.min, before.max);
    }

    #[test]
    fn shell_rejects_bad_inputs() {
        let b = box_10x20x30();
        let top = planar_face(&b, 2, 30.0);
        let invalid =
            |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        assert!(invalid(K.shell(&b, &[], 1.0)));
        let edge = SubShape {
            kind: TopoKind::Edge,
            index: 0,
        };
        assert!(invalid(K.shell(&b, &[top, edge], 1.0)));
        let out_of_range = SubShape {
            kind: TopoKind::Face,
            index: 6,
        };
        assert!(invalid(K.shell(&b, &[out_of_range], 1.0)));
        for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(invalid(K.shell(&b, &[top], bad)), "thickness {bad}");
        }
    }

    #[test]
    fn shell_too_thick_is_an_error_and_keeps_the_input() {
        let b = box_10x20x30();
        let top = planar_face(&b, 2, 30.0);
        let r = K.shell(&b, &[top], 5.0);
        assert!(
            matches!(
                r,
                Err(KernelError::Backend(_) | KernelError::InvalidInput(_))
            ),
            "thickness 5 on a 10 mm wide box should fail"
        );
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert!(K.shell(&b, &[top], 1.0).is_ok());
    }

    /// Neutral plane z = 0, pull +Z.
    const PULL_Z: Axis = Axis {
        origin: [0.0; 3],
        dir: [0.0, 0.0, 1.0],
    };

    /// 30 · tan 5°: how far a 5° draft moves the top edge of a 30 mm face.
    fn draft_offset() -> f64 {
        30.0 * 5f64.to_radians().tan()
    }

    fn assert_face_box(s: &OcctShape, f: SubShape, min: Point3, max: Point3) {
        let b = K.sub_shape_bounding_box(s, f).unwrap();
        for axis in 0..3 {
            assert_close(b.min[axis], min[axis]);
            assert_close(b.max[axis], max[axis]);
        }
    }

    #[test]
    fn draft_one_side_face_by_5_degrees() {
        let b = box_10x20x30();
        let side = planar_face(&b, 0, 10.0);
        let r = K.draft(&b, &[side], PULL_Z, 5f64.to_radians()).unwrap();
        let d = draft_offset();
        assert_close(K.volume(&r).unwrap(), 6000.0 - 15.0 * d * 20.0);
        assert_close(K.volume(&r).unwrap(), 5212.6020);
        assert_face_box(
            &r,
            planar_face(&r, 2, 30.0),
            [0.0, 0.0, 30.0],
            [10.0 - d, 20.0, 30.0],
        );
        assert_face_box(
            &r,
            planar_face(&r, 2, 0.0),
            [0.0, 0.0, 0.0],
            [10.0, 20.0, 0.0],
        );
        assert_eq!(K.sub_shape_count(&r, TopoKind::Face).unwrap(), 6);
        assert_close(K.volume(&b).unwrap(), 6000.0);
    }

    #[test]
    fn draft_four_side_faces_by_5_degrees() {
        let b = box_10x20x30();
        let sides = [
            planar_face(&b, 0, 0.0),
            planar_face(&b, 0, 10.0),
            planar_face(&b, 1, 0.0),
            planar_face(&b, 1, 20.0),
        ];
        let r = K.draft(&b, &sides, PULL_Z, 5f64.to_radians()).unwrap();
        let (d, t) = (draft_offset(), 5f64.to_radians().tan());
        // ∫₀³⁰ (10 − 2zt)(20 − 2zt) dz = 6000 − 27000t + 36000t².
        assert_close(
            K.volume(&r).unwrap(),
            6000.0 - 27000.0 * t + 36000.0 * t * t,
        );
        assert_close(K.volume(&r).unwrap(), 3913.3597);
        assert_face_box(
            &r,
            planar_face(&r, 2, 30.0),
            [d, d, 30.0],
            [10.0 - d, 20.0 - d, 30.0],
        );
        assert_eq!(K.sub_shape_count(&r, TopoKind::Face).unwrap(), 6);
        // Listing a face twice drafts it once.
        let twice = K
            .draft(&b, &[sides[1], sides[1]], PULL_Z, 5f64.to_radians())
            .unwrap();
        assert_close(K.volume(&twice).unwrap(), 6000.0 - 15.0 * d * 20.0);
    }

    #[test]
    fn draft_negative_angle_grows_the_solid() {
        let b = box_10x20x30();
        let side = planar_face(&b, 0, 10.0);
        let r = K.draft(&b, &[side], PULL_Z, -(5f64.to_radians())).unwrap();
        assert_close(K.volume(&r).unwrap(), 6000.0 + 15.0 * draft_offset() * 20.0);
        assert_close(K.volume(&r).unwrap(), 6787.3980);
        assert_close(K.volume(&b).unwrap(), 6000.0);
    }

    #[test]
    fn draft_rejects_bad_inputs() {
        let b = box_10x20x30();
        let side = planar_face(&b, 0, 10.0);
        let a = 5f64.to_radians();
        let invalid =
            |r: Result<OcctShape, KernelError>| matches!(r, Err(KernelError::InvalidInput(_)));
        assert!(invalid(K.draft(&b, &[], PULL_Z, a)));
        assert!(invalid(K.draft(&b, &[side, edge(0)], PULL_Z, a)));
        assert!(invalid(K.draft(&b, &[sub(TopoKind::Face, 6)], PULL_Z, a)));
        for dir in [[0.0; 3], [f64::NAN, 0.0, 1.0], [0.0, f64::INFINITY, 1.0]] {
            let neutral = Axis {
                origin: [0.0; 3],
                dir,
            };
            assert!(invalid(K.draft(&b, &[side], neutral, a)), "dir {dir:?}");
        }
        let neutral = Axis {
            origin: [f64::NAN, 0.0, 0.0],
            dir: [0.0, 0.0, 1.0],
        };
        assert!(invalid(K.draft(&b, &[side], neutral, a)));
        for bad in [0.0, f64::NAN, f64::INFINITY, FRAC_PI_2, -FRAC_PI_2, 2.0] {
            assert!(invalid(K.draft(&b, &[side], PULL_Z, bad)), "angle {bad}");
        }
    }

    #[test]
    fn draft_face_parallel_to_the_neutral_plane_is_an_error() {
        let b = box_10x20x30();
        let top = planar_face(&b, 2, 30.0);
        let r = K.draft(&b, &[top], PULL_Z, 5f64.to_radians());
        assert!(
            matches!(
                r,
                Err(KernelError::Backend(_) | KernelError::InvalidInput(_))
            ),
            "drafting the top face should fail"
        );
        assert_close(K.volume(&b).unwrap(), 6000.0);
    }

    fn hole_box() -> OcctShape {
        K.make_box([0.0; 3], [40.0, 30.0, 20.0]).unwrap()
    }

    const HOLE_DOWN: Axis = Axis {
        origin: [20.0, 15.0, 20.0],
        dir: [0.0, 0.0, -1.0],
    };

    const CBORE: HoleKind = HoleKind::Counterbore {
        diameter: 6.0,
        cb_diameter: 11.0,
        cb_depth: 6.0,
    };

    fn assert_hole(s: &OcctShape, volume: f64, faces: usize) {
        assert_close(K.volume(s).unwrap(), volume);
        assert_eq!(K.face_count(s).unwrap(), faces);
        assert_bbox(s, [0.0; 3], [40.0, 30.0, 20.0]);
    }

    #[test]
    fn hole_simple_blind_and_through() {
        let b = hole_box();
        let simple = HoleKind::Simple { diameter: 6.0 };
        let blind = K
            .hole(&b, HOLE_DOWN, simple, HoleDepth::Blind(10.0))
            .unwrap();
        assert_hole(&blind, 24000.0 - 90.0 * PI, 8);
        let through = K
            .hole(&b, HOLE_DOWN, simple, HoleDepth::ThroughAll)
            .unwrap();
        assert_hole(&through, 24000.0 - 180.0 * PI, 7);
        // The input solid is unchanged.
        assert_close(K.volume(&b).unwrap(), 24000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
    }

    #[test]
    fn hole_counterbore_through_and_blind() {
        let b = hole_box();
        let through = K.hole(&b, HOLE_DOWN, CBORE, HoleDepth::ThroughAll).unwrap();
        assert_hole(&through, 24000.0 - (181.5 + 126.0) * PI, 9);
        let blind = K
            .hole(&b, HOLE_DOWN, CBORE, HoleDepth::Blind(10.0))
            .unwrap();
        assert_hole(&blind, 24000.0 - (181.5 + 36.0) * PI, 10);
        assert_close(K.volume(&b).unwrap(), 24000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
    }

    #[test]
    fn hole_on_a_side_face() {
        let b = hole_box();
        let at = Axis {
            origin: [0.0, 15.0, 10.0],
            dir: [1.0, 0.0, 0.0],
        };
        let h = K
            .hole(
                &b,
                at,
                HoleKind::Simple { diameter: 6.0 },
                HoleDepth::Blind(10.0),
            )
            .unwrap();
        assert_hole(&h, 24000.0 - 90.0 * PI, 8);
        assert!(
            find_by_box(&h, TopoKind::Face, [10.0, 12.0, 7.0], [10.0, 18.0, 13.0]).is_some(),
            "the hole bottom should be the disk at x = 10"
        );
    }

    /// Ø6 hole with a Ø12 × 90° countersink: cone depth 3, cone volume 63π.
    const CSK: HoleKind = HoleKind::Countersink {
        diameter: 6.0,
        csk_diameter: 12.0,
        csk_angle: std::f64::consts::FRAC_PI_2,
    };

    #[test]
    fn hole_countersink_through_and_blind() {
        let b = hole_box();
        let through = K.hole(&b, HOLE_DOWN, CSK, HoleDepth::ThroughAll).unwrap();
        // 180π hole + 63π cone − 27π overlap.
        assert_hole(&through, 24000.0 - 216.0 * PI, 8);
        let blind = K.hole(&b, HOLE_DOWN, CSK, HoleDepth::Blind(10.0)).unwrap();
        // 90π hole + 63π cone − 27π overlap.
        assert_hole(&blind, 24000.0 - 126.0 * PI, 9);
        // The input solid is unchanged.
        assert_close(K.volume(&b).unwrap(), 24000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
    }

    #[test]
    fn hole_countersink_on_a_side_face() {
        let b = hole_box();
        let at = Axis {
            origin: [0.0, 15.0, 10.0],
            dir: [1.0, 0.0, 0.0],
        };
        let h = K.hole(&b, at, CSK, HoleDepth::Blind(10.0)).unwrap();
        assert_hole(&h, 24000.0 - 126.0 * PI, 9);
        assert!(
            find_by_box(&h, TopoKind::Face, [10.0, 12.0, 7.0], [10.0, 18.0, 13.0]).is_some(),
            "the hole bottom should be the disk at x = 10"
        );
    }

    #[test]
    fn hole_countersink_rejects_bad_inputs() {
        let b = hole_box();
        let csk = |diameter, csk_diameter, csk_angle| HoleKind::Countersink {
            diameter,
            csk_diameter,
            csk_angle,
        };
        let through = HoleDepth::ThroughAll;
        for (kind, depth) in [
            (csk(0.0, 12.0, PI / 2.0), through),
            (csk(6.0, f64::NAN, PI / 2.0), through),
            (csk(6.0, 6.0, PI / 2.0), through),
            (csk(6.0, 12.0, PI), through),
            (csk(6.0, 12.0, 0.0), through),
            (csk(6.0, 12.0, f64::INFINITY), through),
            (csk(6.0, 12.0, PI / 2.0), HoleDepth::Blind(3.0)),
        ] {
            assert!(
                matches!(
                    K.hole(&b, HOLE_DOWN, kind, depth),
                    Err(KernelError::InvalidInput(_))
                ),
                "{kind:?} {depth:?} should be InvalidInput"
            );
        }
    }

    fn tapped(designation: &str) -> HoleKind {
        HoleKind::Tapped {
            thread: crate::metric_coarse(designation).unwrap(),
        }
    }

    #[test]
    fn hole_tapped_through_and_blind() {
        let b = hole_box();
        let m6 = tapped("M6");
        // M6 tap drill Ø5.
        let through = K.hole(&b, HOLE_DOWN, m6, HoleDepth::ThroughAll).unwrap();
        assert_hole(&through, 24000.0 - 125.0 * PI, 7);
        let blind = K.hole(&b, HOLE_DOWN, m6, HoleDepth::Blind(10.0)).unwrap();
        assert_hole(&blind, 24000.0 - 62.5 * PI, 8);
        assert!(
            find_by_box(
                &blind,
                TopoKind::Face,
                [17.5, 12.5, 10.0],
                [22.5, 17.5, 10.0]
            )
            .is_some(),
            "the hole bottom should be the Ø5 disk at z = 10"
        );
        // M10 tap drill Ø8.5: 24000 − 361.25π ≈ 22865.0997.
        let m10 = K
            .hole(&b, HOLE_DOWN, tapped("M10"), HoleDepth::ThroughAll)
            .unwrap();
        assert_hole(&m10, 24000.0 - 361.25 * PI, 7);
        assert!((K.volume(&m10).unwrap() - 22865.0997).abs() < 1e-3);
        // The input solid is unchanged.
        assert_close(K.volume(&b).unwrap(), 24000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
    }

    #[test]
    fn hole_tapped_equals_simple_of_tap_drill_diameter() {
        let b = hole_box();
        for name in ["M6", "M10"] {
            let thread = crate::metric_coarse(name).unwrap();
            let simple = HoleKind::Simple {
                diameter: thread.tap_drill_diameter(),
            };
            for depth in [HoleDepth::Blind(10.0), HoleDepth::ThroughAll] {
                let t = K
                    .hole(&b, HOLE_DOWN, HoleKind::Tapped { thread }, depth)
                    .unwrap();
                let s = K.hole(&b, HOLE_DOWN, simple, depth).unwrap();
                let (vt, vs) = (K.volume(&t).unwrap(), K.volume(&s).unwrap());
                assert!(
                    (vt - vs).abs() <= 1e-9 * vs,
                    "{name} {depth:?}: {vt} vs {vs}"
                );
                assert_eq!(K.face_count(&t).unwrap(), K.face_count(&s).unwrap());
            }
        }
    }

    #[test]
    fn hole_tapped_rejects_bad_threads() {
        let b = hole_box();
        let t = |nominal, pitch| HoleKind::Tapped {
            thread: crate::MetricThread { nominal, pitch },
        };
        for kind in [
            t(0.0, 1.0),
            t(-6.0, 1.0),
            t(f64::NAN, 1.0),
            t(6.0, 0.0),
            t(6.0, f64::INFINITY),
            t(6.0, 6.0),
            t(6.0, 7.0),
        ] {
            assert!(
                matches!(
                    K.hole(&b, HOLE_DOWN, kind, HoleDepth::ThroughAll),
                    Err(KernelError::InvalidInput(_))
                ),
                "{kind:?} should be InvalidInput"
            );
        }
    }

    #[test]
    fn hole_rejects_bad_inputs_and_misses() {
        let b = hole_box();
        let blind = HoleDepth::Blind(10.0);
        let invalid = |r: Result<OcctShape, KernelError>| {
            assert!(matches!(r, Err(KernelError::InvalidInput(_))), "{r:?}");
        };
        invalid(K.hole(&b, HOLE_DOWN, HoleKind::Simple { diameter: 0.0 }, blind));
        invalid(K.hole(
            &b,
            HOLE_DOWN,
            HoleKind::Simple { diameter: 6.0 },
            HoleDepth::Blind(-1.0),
        ));
        let narrow = HoleKind::Counterbore {
            diameter: 6.0,
            cb_diameter: 5.0,
            cb_depth: 6.0,
        };
        invalid(K.hole(&b, HOLE_DOWN, narrow, HoleDepth::ThroughAll));
        invalid(K.hole(&b, HOLE_DOWN, CBORE, HoleDepth::Blind(6.0)));
        let zero_dir = Axis {
            dir: [0.0; 3],
            ..HOLE_DOWN
        };
        invalid(K.hole(&b, zero_dir, CBORE, HoleDepth::ThroughAll));
        let nan_origin = Axis {
            origin: [f64::NAN, 15.0, 20.0],
            ..HOLE_DOWN
        };
        invalid(K.hole(&b, nan_origin, CBORE, HoleDepth::ThroughAll));
        let outside = Axis {
            origin: [100.0, 15.0, 20.0],
            ..HOLE_DOWN
        };
        for depth in [blind, HoleDepth::ThroughAll] {
            let r = K.hole(&b, outside, HoleKind::Simple { diameter: 6.0 }, depth);
            assert!(
                matches!(
                    r,
                    Err(KernelError::InvalidInput(_)
                        | KernelError::Backend(_)
                        | KernelError::EmptyResult(_))
                ),
                "a hole that misses should be an error, got {r:?}"
            );
        }
        // Pointing away from the solid also misses.
        let away = Axis {
            dir: [0.0, 0.0, 1.0],
            ..HOLE_DOWN
        };
        invalid(K.hole(&b, away, CBORE, HoleDepth::ThroughAll));
        assert_close(K.volume(&b).unwrap(), 24000.0);
    }

    /// Square of side `side` centered on the z axis at height `z`.
    fn square_section(side: f64, z: f64) -> OcctShape {
        let h = side / 2.0;
        let plane = Workplane::XY.offset(z).unwrap();
        K.make_face(&plane, &polygon(&[[-h, -h], [h, -h], [h, h], [-h, h]]), &[])
            .unwrap()
    }

    /// Circle of radius `r` centered on the z axis at height `z`.
    fn circle_section(r: f64, z: f64) -> OcctShape {
        let plane = Workplane::XY.offset(z).unwrap();
        K.make_face(&plane, &circle([0.0, 0.0], r), &[]).unwrap()
    }

    /// Each section is still one face with its original area.
    fn assert_sections_unchanged(sections: &[OcctShape], areas: &[f64]) {
        for (s, area) in sections.iter().zip(areas) {
            assert_eq!(K.face_count(s).unwrap(), 1);
            assert_close(K.surface_area(s).unwrap(), *area);
        }
    }

    #[test]
    fn loft_ruled_square_frustum() {
        let sections = [square_section(20.0, 0.0), square_section(10.0, 30.0)];
        let s = K.loft(&sections, true).unwrap();
        // h/3 · (A1 + A2 + √(A1·A2)) = 10 · (400 + 100 + 200).
        assert_close(K.volume(&s).unwrap(), 7000.0);
        assert_eq!(K.sub_shape_count(&s, TopoKind::Face).unwrap(), 6);
        assert_eq!(K.sub_shape_count(&s, TopoKind::Edge).unwrap(), 12);
        assert_eq!(K.sub_shape_count(&s, TopoKind::Vertex).unwrap(), 8);
        assert_bbox(&s, [-10.0, -10.0, 0.0], [10.0, 10.0, 30.0]);
        assert_sections_unchanged(&sections, &[400.0, 100.0]);
    }

    #[test]
    fn loft_ruled_cone_frustum() {
        let sections = [circle_section(10.0, 0.0), circle_section(5.0, 20.0)];
        let s = K.loft(&sections, true).unwrap();
        // π h/3 · (R² + Rr + r²) = π · 20/3 · 175.
        assert_close(K.volume(&s).unwrap(), 3500.0 * PI / 3.0);
        assert_eq!(K.face_count(&s).unwrap(), 3);
        assert_bbox(&s, [-10.0, -10.0, 0.0], [10.0, 10.0, 20.0]);
        assert_sections_unchanged(&sections, &[100.0 * PI, 25.0 * PI]);
    }

    #[test]
    fn loft_three_circles_ruled_and_smooth() {
        let sections = [
            circle_section(10.0, 0.0),
            circle_section(5.0, 10.0),
            circle_section(10.0, 20.0),
        ];
        let ruled_volume = 2.0 * (PI * 10.0 / 3.0 * 175.0);
        let ruled = K.loft(&sections, true).unwrap();
        assert_close(K.volume(&ruled).unwrap(), ruled_volume);
        assert_bbox(&ruled, [-10.0, -10.0, 0.0], [10.0, 10.0, 20.0]);

        let smooth = K.loft(&sections, false).unwrap();
        assert_bbox(&smooth, [-10.0, -10.0, 0.0], [10.0, 10.0, 20.0]);
        let v = K.volume(&smooth).unwrap();
        assert!(
            v > 500.0 * PI && v < ruled_volume,
            "smooth volume {v} should be between {} and {ruled_volume}",
            500.0 * PI
        );
        assert_sections_unchanged(&sections, &[100.0 * PI, 25.0 * PI, 100.0 * PI]);
    }

    #[test]
    fn loft_rejects_bad_sections_before_the_backend() {
        let invalid = |r: Result<OcctShape, KernelError>| {
            assert!(
                matches!(r, Err(KernelError::InvalidInput(_))),
                "expected InvalidInput, got {r:?}"
            );
        };
        let a = square_section(20.0, 0.0);
        invalid(K.loft(&[], true));
        invalid(K.loft(std::slice::from_ref(&a), true));
        let solid = K.make_box([-5.0, -5.0, 10.0], [10.0, 10.0, 10.0]).unwrap();
        invalid(K.loft(&[square_section(20.0, 0.0), solid], false));
        let plane = Workplane::XY.offset(30.0).unwrap();
        let holed = K
            .make_face(
                &plane,
                &polygon(&[[-10.0, -10.0], [10.0, -10.0], [10.0, 10.0], [-10.0, 10.0]]),
                &[circle([0.0, 0.0], 3.0)],
            )
            .unwrap();
        invalid(K.loft(&[square_section(20.0, 0.0), holed], true));
        assert_sections_unchanged(&[a], &[400.0]);
    }

    #[test]
    fn loft_of_coincident_sections_is_an_error() {
        let sections = [square_section(20.0, 0.0), square_section(20.0, 0.0)];
        for ruled in [true, false] {
            let r = K.loft(&sections, ruled);
            assert!(r.is_err(), "expected an error, got {r:?}");
        }
        assert_sections_unchanged(&sections, &[400.0, 400.0]);
        // The inputs are still usable.
        let s = K.extrude(&sections[0], Extent::Distance(5.0)).unwrap();
        assert_close(K.volume(&s).unwrap(), 2000.0);
    }

    fn assert_vec9(actual: [f64; 3], expected: [f64; 3]) {
        for i in 0..3 {
            assert!(
                (actual[i] - expected[i]).abs() <= 1e-9,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    /// `dir` is ±`expected` within 1e-9.
    fn assert_dir_either_sign(dir: [f64; 3], expected: [f64; 3]) {
        let s = if dir.iter().zip(expected).map(|(a, b)| a * b).sum::<f64>() < 0.0 {
            -1.0
        } else {
            1.0
        };
        assert_vec9(dir.map(|c| c * s), expected);
    }

    /// The face of `s` whose `face_plane` normal is `normal` (within 1e-9).
    fn face_with_normal(s: &OcctShape, normal: [f64; 3]) -> SubShape {
        let n = K.sub_shape_count(s, TopoKind::Face).unwrap();
        (0..n)
            .map(|i| sub(TopoKind::Face, i))
            .find(|&f| match K.face_plane(s, f) {
                Ok((_, m)) => (0..3).all(|a| (m[a] - normal[a]).abs() <= 1e-9),
                Err(_) => false,
            })
            .expect("face with that normal")
    }

    /// The sub-shape of `kind` of `s` whose bounding box is `min`–`max`.
    fn sub_with_bbox(s: &OcctShape, kind: TopoKind, min: Point3, max: Point3) -> SubShape {
        let n = K.sub_shape_count(s, kind).unwrap();
        (0..n)
            .map(|i| sub(kind, i))
            .find(|&e| {
                let b = K.sub_shape_bounding_box(s, e).unwrap();
                (0..3)
                    .all(|a| (b.min[a] - min[a]).abs() <= 1e-6 && (b.max[a] - max[a]).abs() <= 1e-6)
            })
            .expect("sub-shape with that bounding box")
    }

    #[test]
    fn work_plane_on_box_faces() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        let top = K
            .work_plane_on_face(&b, face_with_normal(&b, [0.0, 0.0, 1.0]))
            .unwrap();
        assert_vec9(top.origin, [0.0, 0.0, 30.0]);
        assert_vec9(top.normal, [0.0, 0.0, 1.0]);
        assert_vec9(top.x_dir, [1.0, 0.0, 0.0]);
        let side = K
            .work_plane_on_face(&b, face_with_normal(&b, [1.0, 0.0, 0.0]))
            .unwrap();
        assert_vec9(side.origin, [10.0, 0.0, 0.0]);
        assert_vec9(side.normal, [1.0, 0.0, 0.0]);
        assert_vec9(side.x_dir, [0.0, 1.0, 0.0]);
        let bottom = K
            .work_plane_on_face(&b, face_with_normal(&b, [0.0, 0.0, -1.0]))
            .unwrap();
        assert_vec9(bottom.origin, [0.0, 0.0, 0.0]);
        assert_vec9(bottom.normal, [0.0, 0.0, -1.0]);
        // The input is unchanged.
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
    }

    #[test]
    fn work_axis_on_a_box_edge_and_a_cylinder_face() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        let edge = sub_with_bbox(&b, TopoKind::Edge, [0.0, 20.0, 30.0], [10.0, 20.0, 30.0]);
        let a = K.work_axis_on(&b, edge).unwrap();
        assert_vec9(a.origin, [0.0, 20.0, 30.0]);
        assert_dir_either_sign(a.dir, [1.0, 0.0, 0.0]);

        let c = K.make_cylinder([5.0, 5.0, 0.0], 3.0, 10.0).unwrap();
        let lateral = (0..K.sub_shape_count(&c, TopoKind::Face).unwrap())
            .map(|i| sub(TopoKind::Face, i))
            .find(|&f| K.face_plane(&c, f).is_err())
            .expect("lateral face");
        let a = K.work_axis_on(&c, lateral).unwrap();
        assert_vec9(a.origin, [5.0, 5.0, 0.0]);
        assert_dir_either_sign(a.dir, [0.0, 0.0, 1.0]);
        // The inputs are unchanged.
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
        assert_close(K.volume(&c).unwrap(), 90.0 * PI);
        assert_eq!(K.face_count(&c).unwrap(), 3);
    }

    #[test]
    fn vertex_points_are_the_box_corners() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        assert_eq!(K.sub_shape_count(&b, TopoKind::Vertex).unwrap(), 8);
        let mut got: Vec<[f64; 3]> = (0..8)
            .map(|i| K.vertex_point(&b, sub(TopoKind::Vertex, i)).unwrap())
            .collect();
        let mut want = Vec::new();
        for x in [0.0, 10.0] {
            for y in [0.0, 20.0] {
                for z in [0.0, 30.0] {
                    want.push([x, y, z]);
                }
            }
        }
        let key = |p: &[f64; 3]| {
            (
                p[0].round() as i64,
                p[1].round() as i64,
                p[2].round() as i64,
            )
        };
        got.sort_by_key(key);
        want.sort_by_key(key);
        for (g, w) in got.iter().zip(&want) {
            assert_vec9(*g, *w);
        }
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
    }

    #[test]
    fn work_features_on_geometry_reject_bad_sub_shapes() {
        let invalid = |r: Result<(), KernelError>, what: &str| {
            assert!(
                matches!(r, Err(KernelError::InvalidInput(_))),
                "{what}: {r:?}"
            );
        };
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        let c = K.make_cylinder([5.0, 5.0, 0.0], 3.0, 10.0).unwrap();
        // Wrong kind.
        invalid(
            K.work_plane_on_face(&b, sub(TopoKind::Vertex, 0)).map(drop),
            "plane on vertex",
        );
        invalid(
            K.work_plane_on_face(&b, sub(TopoKind::Edge, 0)).map(drop),
            "plane on edge",
        );
        invalid(
            K.work_axis_on(&b, sub(TopoKind::Vertex, 0)).map(drop),
            "axis on vertex",
        );
        invalid(
            K.vertex_point(&b, sub(TopoKind::Face, 0)).map(drop),
            "point on face",
        );
        invalid(
            K.vertex_point(&b, sub(TopoKind::Edge, 0)).map(drop),
            "point on edge",
        );
        // Index out of range.
        invalid(
            K.work_plane_on_face(&b, sub(TopoKind::Face, 6)).map(drop),
            "face 6",
        );
        invalid(
            K.work_axis_on(&b, sub(TopoKind::Edge, 12)).map(drop),
            "edge 12",
        );
        invalid(
            K.work_axis_on(&b, sub(TopoKind::Face, 6)).map(drop),
            "axis face 6",
        );
        invalid(
            K.vertex_point(&b, sub(TopoKind::Vertex, 8)).map(drop),
            "vertex 8",
        );
        // A non-planar face for a plane.
        let lateral = (0..3)
            .map(|i| sub(TopoKind::Face, i))
            .find(|&f| K.face_plane(&c, f).is_err())
            .expect("lateral face");
        invalid(
            K.work_plane_on_face(&c, lateral).map(drop),
            "plane on cylinder side",
        );
        // A circular edge and a planar face for an axis.
        let circle = sub_with_bbox(&c, TopoKind::Edge, [2.0, 2.0, 10.0], [8.0, 8.0, 10.0]);
        invalid(
            K.work_axis_on(&c, circle).map(drop),
            "axis on circular edge",
        );
        let cap = face_with_normal(&c, [0.0, 0.0, 1.0]);
        invalid(K.work_axis_on(&c, cap).map(drop), "axis on planar face");
        invalid(
            K.work_axis_on(&b, face_with_normal(&b, [0.0, 0.0, 1.0]))
                .map(drop),
            "axis on box face",
        );
        // The inputs are still usable.
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_close(K.volume(&c).unwrap(), 90.0 * PI);
    }

    /// The lateral (non-planar) face of a cylinder.
    fn lateral_face(c: &OcctShape) -> SubShape {
        (0..K.sub_shape_count(c, TopoKind::Face).unwrap())
            .map(|i| sub(TopoKind::Face, i))
            .find(|&f| K.face_plane(c, f).is_err())
            .expect("lateral face")
    }

    #[test]
    fn work_plane_tangent_to_a_cylinder_face() {
        let c = K.make_cylinder([5.0, 5.0, 0.0], 3.0, 10.0).unwrap();
        let side = lateral_face(&c);
        let p = K.work_plane_tangent(&c, side, [1.0, 0.0, 0.0]).unwrap();
        assert_vec9(p.origin, [8.0, 0.0, 0.0]);
        assert_vec9(p.normal, [1.0, 0.0, 0.0]);
        assert_vec9(p.x_dir, [0.0, 1.0, 0.0]);
        let p = K.work_plane_tangent(&c, side, [0.0, -1.0, 0.0]).unwrap();
        assert_vec9(p.origin, [0.0, 2.0, 0.0]);
        assert_vec9(p.normal, [0.0, -1.0, 0.0]);
        let s = 0.5f64.sqrt();
        let p = K.work_plane_tangent(&c, side, [1.0, 1.0, 0.0]).unwrap();
        assert_vec9(p.normal, [s, s, 0.0]);
        assert_vec9(p.origin, [5.0 + 3.0 * s, 5.0 + 3.0 * s, 0.0]);
        assert!((p.origin[0] - 7.121_320_3).abs() <= 1e-7);
        // The axial part of toward is ignored.
        let q = K.work_plane_tangent(&c, side, [1.0, 0.0, 5.0]).unwrap();
        assert_vec9(q.origin, [8.0, 0.0, 0.0]);
        assert_vec9(q.normal, [1.0, 0.0, 0.0]);
        assert_vec9(q.x_dir, [0.0, 1.0, 0.0]);
        // The input is unchanged.
        assert_close(K.volume(&c).unwrap(), 90.0 * PI);
        assert_eq!(K.face_count(&c).unwrap(), 3);
    }

    #[test]
    fn edge_midpoints_at_half_the_arc_length() {
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        let edge = sub_with_bbox(&b, TopoKind::Edge, [0.0, 20.0, 30.0], [10.0, 20.0, 30.0]);
        assert_vec9(K.edge_midpoint(&b, edge).unwrap(), [5.0, 20.0, 30.0]);

        let half_disc = K
            .make_face(
                &Workplane::XY,
                &ProfileLoop {
                    segments: vec![
                        line([-5.0, 0.0], [5.0, 0.0]),
                        Segment2::Arc {
                            center: [0.0, 0.0],
                            radius: 5.0,
                            start_angle: 0.0,
                            end_angle: PI,
                        },
                    ],
                },
                &[],
            )
            .unwrap();
        assert_eq!(K.sub_shape_count(&half_disc, TopoKind::Edge).unwrap(), 2);
        let arc = (0..2)
            .map(|i| sub(TopoKind::Edge, i))
            .find(|&e| K.work_axis_on(&half_disc, e).is_err())
            .expect("arc edge");
        assert_vec9(K.edge_midpoint(&half_disc, arc).unwrap(), [0.0, 5.0, 0.0]);

        let c = K.make_cylinder([5.0, 5.0, 0.0], 3.0, 10.0).unwrap();
        let top = sub_with_bbox(&c, TopoKind::Edge, [2.0, 2.0, 10.0], [8.0, 8.0, 10.0]);
        let m = K.edge_midpoint(&c, top).unwrap();
        assert!((m[2] - 10.0).abs() <= 1e-9, "{m:?}");
        let r = ((m[0] - 5.0).powi(2) + (m[1] - 5.0).powi(2)).sqrt();
        assert!((r - 3.0).abs() <= 1e-9, "{m:?} is {r} from the axis");
        // The inputs are unchanged.
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_eq!(K.face_count(&b).unwrap(), 6);
        assert_close(K.volume(&c).unwrap(), 90.0 * PI);
        assert_eq!(K.face_count(&c).unwrap(), 3);
    }

    #[test]
    fn tangent_plane_and_edge_midpoint_reject_bad_inputs() {
        let invalid = |r: Result<(), KernelError>, what: &str| {
            assert!(
                matches!(r, Err(KernelError::InvalidInput(_))),
                "{what}: {r:?}"
            );
        };
        let b = K.make_box([0.0; 3], [10.0, 20.0, 30.0]).unwrap();
        let c = K.make_cylinder([5.0, 5.0, 0.0], 3.0, 10.0).unwrap();
        let side = lateral_face(&c);
        let x = [1.0, 0.0, 0.0];
        // Tangent: wrong face type, wrong kind, index out of range.
        invalid(
            K.work_plane_tangent(&b, face_with_normal(&b, [0.0, 0.0, 1.0]), x)
                .map(drop),
            "tangent on box face",
        );
        invalid(
            K.work_plane_tangent(&c, sub(TopoKind::Edge, 0), x)
                .map(drop),
            "tangent on edge",
        );
        invalid(
            K.work_plane_tangent(&c, sub(TopoKind::Vertex, 0), x)
                .map(drop),
            "tangent on vertex",
        );
        invalid(
            K.work_plane_tangent(&c, sub(TopoKind::Face, 3), x)
                .map(drop),
            "tangent face 3",
        );
        // Tangent: bad toward.
        for (toward, what) in [
            ([0.0; 3], "zero toward"),
            ([f64::NAN, 0.0, 0.0], "NaN toward"),
            ([f64::INFINITY, 0.0, 0.0], "infinite toward"),
            ([0.0, 0.0, 1.0], "toward along the axis"),
        ] {
            invalid(K.work_plane_tangent(&c, side, toward).map(drop), what);
        }
        // Midpoint: wrong kind, index out of range.
        invalid(
            K.edge_midpoint(&b, sub(TopoKind::Face, 0)).map(drop),
            "midpoint of face",
        );
        invalid(
            K.edge_midpoint(&b, sub(TopoKind::Vertex, 0)).map(drop),
            "midpoint of vertex",
        );
        invalid(
            K.edge_midpoint(&b, sub(TopoKind::Edge, 12)).map(drop),
            "edge 12",
        );
        // The inputs are still usable.
        assert_close(K.volume(&b).unwrap(), 6000.0);
        assert_close(K.volume(&c).unwrap(), 90.0 * PI);
    }
}
