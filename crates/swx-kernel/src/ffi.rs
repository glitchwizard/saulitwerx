//! FFI boundary to the OCCT C ABI shim (`cpp/occt_shim.cpp`).
//!
//! This is the only module in the workspace allowed to use `unsafe`. Each
//! `unsafe` block documents why it is sound. The shim functions are `noexcept`
//! and catch all C++ exceptions, so nothing unwinds into Rust.

#![allow(unsafe_code)]

use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_float, c_int};
use std::ptr::{self, NonNull};

use crate::Mesh;

/// Opaque C++ shape record (`struct SwxShape` in the shim).
#[repr(C)]
struct RawShape {
    _private: [u8; 0],
}

/// Opaque C++ mesh record (`struct SwxMesh` in the shim).
#[repr(C)]
struct RawMesh {
    _private: [u8; 0],
}

extern "C" {
    fn swx_occt_make_box(
        x: c_double,
        y: c_double,
        z: c_double,
        dx: c_double,
        dy: c_double,
        dz: c_double,
        out: *mut *mut RawShape,
    ) -> c_int;
    fn swx_occt_make_cylinder(
        x: c_double,
        y: c_double,
        z: c_double,
        radius: c_double,
        height: c_double,
        out: *mut *mut RawShape,
    ) -> c_int;
    fn swx_occt_make_sphere(
        x: c_double,
        y: c_double,
        z: c_double,
        radius: c_double,
        out: *mut *mut RawShape,
    ) -> c_int;
    fn swx_occt_boolean(
        op: c_int,
        a: *const RawShape,
        b: *const RawShape,
        out: *mut *mut RawShape,
    ) -> c_int;
    fn swx_occt_make_face(
        plane: *const c_double,
        n_loops: usize,
        loop_sizes: *const usize,
        edges: *const c_double,
        out: *mut *mut RawShape,
    ) -> c_int;
    fn swx_occt_extrude(
        s: *const RawShape,
        offset: c_double,
        length: c_double,
        out: *mut *mut RawShape,
    ) -> c_int;
    fn swx_occt_volume(s: *const RawShape, out: *mut c_double) -> c_int;
    fn swx_occt_surface_area(s: *const RawShape, out: *mut c_double) -> c_int;
    fn swx_occt_bounding_box(s: *const RawShape, out: *mut c_double) -> c_int;
    fn swx_occt_shape_free(s: *mut RawShape);
    fn swx_occt_face_count(s: *const RawShape, out: *mut usize) -> c_int;
    fn swx_occt_export_step(s: *const RawShape, path: *const c_char) -> c_int;
    fn swx_occt_import_step(path: *const c_char, out: *mut *mut RawShape) -> c_int;
    fn swx_occt_tessellate(
        s: *const RawShape,
        deflection: c_double,
        out: *mut *mut RawMesh,
    ) -> c_int;
    fn swx_occt_mesh_sizes(m: *const RawMesh, vertices: *mut usize, indices: *mut usize) -> c_int;
    fn swx_occt_mesh_copy(
        m: *const RawMesh,
        positions: *mut c_float,
        normals: *mut c_float,
        indices: *mut u32,
    ) -> c_int;
    fn swx_occt_mesh_free(m: *mut RawMesh);
}

/// Owning handle to a shim-allocated shape. Freed exactly once on drop.
#[derive(Debug)]
pub(crate) struct ShapeHandle(NonNull<RawShape>);

impl Drop for ShapeHandle {
    fn drop(&mut self) {
        // SAFETY: the pointer came from a successful shim constructor, is
        // uniquely owned by this handle (no `Clone`), and is freed only here.
        unsafe { swx_occt_shape_free(self.0.as_ptr()) }
    }
}

/// Boolean operation codes understood by `swx_occt_boolean`.
#[derive(Debug, Clone, Copy)]
pub(crate) enum BooleanOp {
    Union = 0,
    Cut = 1,
    Intersect = 2,
}

/// Turns a status code plus out-pointer into an owned handle.
fn take(status: c_int, raw: *mut RawShape) -> Option<ShapeHandle> {
    if status == 0 {
        NonNull::new(raw).map(ShapeHandle)
    } else {
        None
    }
}

pub(crate) fn make_box(origin: [f64; 3], size: [f64; 3]) -> Option<ShapeHandle> {
    let mut raw = ptr::null_mut();
    // SAFETY: `raw` is a valid writable out-pointer on our stack; the shim
    // writes a fresh heap handle to it only on success and never throws.
    let status = unsafe {
        swx_occt_make_box(
            origin[0], origin[1], origin[2], size[0], size[1], size[2], &mut raw,
        )
    };
    take(status, raw)
}

pub(crate) fn make_cylinder(base: [f64; 3], radius: f64, height: f64) -> Option<ShapeHandle> {
    let mut raw = ptr::null_mut();
    // SAFETY: as in `make_box`: valid out-pointer, plain doubles by value.
    let status =
        unsafe { swx_occt_make_cylinder(base[0], base[1], base[2], radius, height, &mut raw) };
    take(status, raw)
}

pub(crate) fn make_sphere(center: [f64; 3], radius: f64) -> Option<ShapeHandle> {
    let mut raw = ptr::null_mut();
    // SAFETY: as in `make_box`: valid out-pointer, plain doubles by value.
    let status = unsafe { swx_occt_make_sphere(center[0], center[1], center[2], radius, &mut raw) };
    take(status, raw)
}

pub(crate) fn boolean(op: BooleanOp, a: &ShapeHandle, b: &ShapeHandle) -> Option<ShapeHandle> {
    let mut raw = ptr::null_mut();
    // SAFETY: `a` and `b` are live handles borrowed for the call; the shim
    // only reads them and does not retain the pointers. `raw` is a valid
    // out-pointer that receives a new, independently owned handle.
    let status = unsafe { swx_occt_boolean(op as c_int, a.0.as_ptr(), b.0.as_ptr(), &mut raw) };
    take(status, raw)
}

/// Builds a planar face. `plane` is origin, x direction and normal;
/// `loop_sizes` gives each loop's edge count and `edges` holds 8 doubles per
/// edge, in the layout of `PreparedFace::encode`.
pub(crate) fn make_face(
    plane: &[f64; 9],
    loop_sizes: &[usize],
    edges: &[f64],
) -> Option<ShapeHandle> {
    if loop_sizes.is_empty() || edges.len() != 8 * loop_sizes.iter().sum::<usize>() {
        return None;
    }
    let mut raw = ptr::null_mut();
    // SAFETY: `plane` points to the 9 doubles the shim reads; `loop_sizes` has
    // `n_loops` entries, and `edges` holds exactly 8 doubles for each edge they
    // count (checked above). The shim only reads them during the call and
    // retains nothing. `raw` is a valid out-pointer for a new owned handle.
    let status = unsafe {
        swx_occt_make_face(
            plane.as_ptr(),
            loop_sizes.len(),
            loop_sizes.as_ptr(),
            edges.as_ptr(),
            &mut raw,
        )
    };
    take(status, raw)
}

pub(crate) fn extrude(face: &ShapeHandle, offset: f64, length: f64) -> Option<ShapeHandle> {
    let mut raw = ptr::null_mut();
    // SAFETY: as in `boolean`: `face` is a live handle read-only for the call
    // and not retained; `raw` receives a new, independently owned handle.
    let status = unsafe { swx_occt_extrude(face.0.as_ptr(), offset, length, &mut raw) };
    take(status, raw)
}

pub(crate) fn volume(s: &ShapeHandle) -> Option<f64> {
    let mut out: c_double = 0.0;
    // SAFETY: `s` is a live handle read-only for the call; `out` is a valid
    // writable `f64` on our stack that outlives the call.
    let status = unsafe { swx_occt_volume(s.0.as_ptr(), &mut out) };
    (status == 0).then_some(out)
}

pub(crate) fn surface_area(s: &ShapeHandle) -> Option<f64> {
    let mut out: c_double = 0.0;
    // SAFETY: as in `volume`.
    let status = unsafe { swx_occt_surface_area(s.0.as_ptr(), &mut out) };
    (status == 0).then_some(out)
}

/// Returns `[xmin, ymin, zmin, xmax, ymax, zmax]`.
pub(crate) fn bounding_box(s: &ShapeHandle) -> Option<[f64; 6]> {
    let mut out = [0.0 as c_double; 6];
    // SAFETY: `s` is a live handle read-only for the call; `out` is a valid
    // writable array of exactly the 6 doubles the shim writes.
    let status = unsafe { swx_occt_bounding_box(s.0.as_ptr(), out.as_mut_ptr()) };
    (status == 0).then_some(out)
}

pub(crate) fn face_count(s: &ShapeHandle) -> Option<usize> {
    let mut out = 0usize;
    // SAFETY: `s` is a live handle read-only for the call; `out` is a valid
    // writable `usize` on our stack (`size_t` and `usize` share a layout).
    let status = unsafe { swx_occt_face_count(s.0.as_ptr(), &mut out) };
    (status == 0).then_some(out)
}

/// Writes `s` as STEP to `path`. Returns `false` on any failure.
pub(crate) fn export_step(s: &ShapeHandle, path: &CStr) -> bool {
    // SAFETY: `s` is a live handle read-only for the call; `path` is a valid
    // NUL-terminated string that outlives the call and is not retained.
    unsafe { swx_occt_export_step(s.0.as_ptr(), path.as_ptr()) == 0 }
}

pub(crate) fn import_step(path: &CStr) -> Option<ShapeHandle> {
    let mut raw = ptr::null_mut();
    // SAFETY: `path` is a valid NUL-terminated string that outlives the call
    // and is not retained; `raw` is a valid out-pointer that receives a new,
    // independently owned handle on success.
    let status = unsafe { swx_occt_import_step(path.as_ptr(), &mut raw) };
    take(status, raw)
}

/// Owning handle to a shim-allocated mesh, freed on drop.
struct MeshHandle(NonNull<RawMesh>);

impl Drop for MeshHandle {
    fn drop(&mut self) {
        // SAFETY: the pointer came from a successful `swx_occt_tessellate`, is
        // uniquely owned by this handle, and is freed only here.
        unsafe { swx_occt_mesh_free(self.0.as_ptr()) }
    }
}

pub(crate) fn tessellate(s: &ShapeHandle, deflection: f64) -> Option<Mesh> {
    let mut raw = ptr::null_mut();
    // SAFETY: `s` is a live handle for the call; the shim only replaces its
    // cached triangulation and does not retain the pointer. `raw` is a valid
    // out-pointer that receives a new, independently owned mesh on success.
    let status = unsafe { swx_occt_tessellate(s.0.as_ptr(), deflection, &mut raw) };
    if status != 0 {
        return None;
    }
    let handle = MeshHandle(NonNull::new(raw)?);

    let (mut nv, mut ni) = (0usize, 0usize);
    // SAFETY: `handle` is live; both out-pointers are valid `usize`s on our
    // stack (`size_t` and `usize` have the same layout on supported targets).
    let status = unsafe { swx_occt_mesh_sizes(handle.0.as_ptr(), &mut nv, &mut ni) };
    if status != 0 {
        return None;
    }

    let mut mesh = Mesh {
        positions: vec![[0.0; 3]; nv],
        normals: vec![[0.0; 3]; nv],
        indices: vec![0; ni],
    };
    // SAFETY: `handle` is live. Each buffer is allocated with exactly the
    // length the shim writes: `[f32; 3]` is three contiguous `f32`s, so
    // `positions`/`normals` hold `3 * nv` floats and `indices` holds `ni` u32s.
    let status = unsafe {
        swx_occt_mesh_copy(
            handle.0.as_ptr(),
            mesh.positions.as_mut_ptr().cast::<c_float>(),
            mesh.normals.as_mut_ptr().cast::<c_float>(),
            mesh.indices.as_mut_ptr(),
        )
    };
    (status == 0).then_some(mesh)
}
