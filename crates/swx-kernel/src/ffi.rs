//! FFI boundary to the OCCT C ABI shim (`cpp/occt_shim.cpp`).
//!
//! This is the only module in the workspace allowed to use `unsafe`. Each
//! `unsafe` block documents why it is sound. The shim functions are `noexcept`
//! and catch all C++ exceptions, so nothing unwinds into Rust.

#![allow(unsafe_code)]

use std::os::raw::{c_double, c_int};

extern "C" {
    fn swx_occt_box_volume(dx: c_double, dy: c_double, dz: c_double, out: *mut c_double) -> c_int;
}

/// Safe wrapper: builds a box and returns its volume, or `None` on failure.
pub(crate) fn box_volume(dx: f64, dy: f64, dz: f64) -> Option<f64> {
    let mut out: c_double = 0.0;
    // SAFETY: `out` is a valid, aligned, writable `f64` on our stack that
    // outlives the call. The shim only writes through it, never retains the
    // pointer, takes plain doubles by value, and never throws across the ABI.
    let status = unsafe { swx_occt_box_volume(dx, dy, dz, &mut out) };
    (status == 0).then_some(out)
}
