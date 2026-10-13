// C ABI shim between swx-kernel (Rust) and OpenCascade (C++).
//
// Every exported function is `noexcept` and catches all C++ exceptions, so no
// exception ever unwinds across the FFI boundary. Failures are reported as a
// negative status code.

#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Failure.hxx>
#include <TopoDS_Shape.hxx>

extern "C" {

// Builds an axis-aligned box with its corner at the origin and writes its
// volume to `*out_volume`. Returns 0 on success, -1 on any OCCT failure.
int swx_occt_box_volume(double dx, double dy, double dz, double* out_volume) noexcept {
    if (out_volume == nullptr) {
        return -1;
    }
    try {
        BRepPrimAPI_MakeBox maker(dx, dy, dz);
        const TopoDS_Shape& shape = maker.Shape();
        GProp_GProps props;
        BRepGProp::VolumeProperties(shape, props);
        *out_volume = props.Mass();
        return 0;
    } catch (...) {
        return -1;
    }
}

}  // extern "C"
