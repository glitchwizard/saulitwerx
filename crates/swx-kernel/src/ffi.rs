//! FFI boundary to the OCCT C ABI shim (`cpp/occt_shim.cpp`).
//!
//! This is the only module in the workspace allowed to use `unsafe`. Each
//! `unsafe` block documents why it is sound. The shim functions are `noexcept`
//! and catch all C++ exceptions, so nothing unwinds into Rust.

#![allow(unsafe_code)]

use std::os::raw::{c_double, c_int};
use std::ptr::{self, NonNull};

/// Opaque C++ shape record (`struct SwxShape` in the shim).
#[repr(C)]
struct RawShape {
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
    fn swx_occt_volume(s: *const RawShape, out: *mut c_double) -> c_int;
    fn swx_occt_surface_area(s: *const RawShape, out: *mut c_double) -> c_int;
    fn swx_occt_bounding_box(s: *const RawShape, out: *mut c_double) -> c_int;
    fn swx_occt_shape_free(s: *mut RawShape);
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
