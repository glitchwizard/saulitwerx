// C ABI shim between swx-kernel (Rust) and OpenCascade (C++).
//
// Every exported function is `noexcept` and catches all C++ exceptions, so no
// exception ever unwinds across the FFI boundary. Failures are reported as a
// negative status code.
//
// Shapes cross the boundary as opaque `SwxShape*` handles that own a heap
// `TopoDS_Shape`. Rust frees each handle exactly once with `swx_occt_shape_free`.

#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBndLib.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <Standard_Failure.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>

struct SwxShape {
    TopoDS_Shape shape;
};

namespace {

// Stores a successfully built, non-null shape in a new handle.
int emit(const TopoDS_Shape& shape, SwxShape** out) {
    if (shape.IsNull()) {
        return -1;
    }
    *out = new SwxShape{shape};
    return 0;
}

}  // namespace

extern "C" {

// Axis-aligned box with its minimum corner at (x, y, z).
int swx_occt_make_box(double x, double y, double z, double dx, double dy, double dz,
                      SwxShape** out) noexcept {
    if (out == nullptr) {
        return -1;
    }
    try {
        BRepPrimAPI_MakeBox maker(gp_Pnt(x, y, z), dx, dy, dz);
        return emit(maker.Shape(), out);
    } catch (...) {
        return -1;
    }
}

// Cylinder along +Z with its base circle centered at (x, y, z).
int swx_occt_make_cylinder(double x, double y, double z, double radius, double height,
                           SwxShape** out) noexcept {
    if (out == nullptr) {
        return -1;
    }
    try {
        gp_Ax2 axis(gp_Pnt(x, y, z), gp_Dir(0.0, 0.0, 1.0));
        BRepPrimAPI_MakeCylinder maker(axis, radius, height);
        return emit(maker.Shape(), out);
    } catch (...) {
        return -1;
    }
}

// Sphere centered at (x, y, z).
int swx_occt_make_sphere(double x, double y, double z, double radius, SwxShape** out) noexcept {
    if (out == nullptr) {
        return -1;
    }
    try {
        BRepPrimAPI_MakeSphere maker(gp_Pnt(x, y, z), radius);
        return emit(maker.Shape(), out);
    } catch (...) {
        return -1;
    }
}

// Boolean operation: 0 = union (fuse), 1 = cut (a minus b), 2 = intersect (common).
int swx_occt_boolean(int op, const SwxShape* a, const SwxShape* b, SwxShape** out) noexcept {
    if (a == nullptr || b == nullptr || out == nullptr) {
        return -1;
    }
    try {
        TopoDS_Shape result;
        bool ok = false;
        switch (op) {
            case 0: {
                BRepAlgoAPI_Fuse algo(a->shape, b->shape);
                ok = algo.IsDone() && !algo.HasErrors();
                if (ok) result = algo.Shape();
                break;
            }
            case 1: {
                BRepAlgoAPI_Cut algo(a->shape, b->shape);
                ok = algo.IsDone() && !algo.HasErrors();
                if (ok) result = algo.Shape();
                break;
            }
            case 2: {
                BRepAlgoAPI_Common algo(a->shape, b->shape);
                ok = algo.IsDone() && !algo.HasErrors();
                if (ok) result = algo.Shape();
                break;
            }
            default:
                return -1;
        }
        if (!ok) {
            return -1;
        }
        return emit(result, out);
    } catch (...) {
        return -1;
    }
}

// Volume by OCCT mass-properties integration.
int swx_occt_volume(const SwxShape* s, double* out) noexcept {
    if (s == nullptr || out == nullptr) {
        return -1;
    }
    try {
        GProp_GProps props;
        BRepGProp::VolumeProperties(s->shape, props);
        *out = props.Mass();
        return 0;
    } catch (...) {
        return -1;
    }
}

// Total face area by OCCT mass-properties integration.
int swx_occt_surface_area(const SwxShape* s, double* out) noexcept {
    if (s == nullptr || out == nullptr) {
        return -1;
    }
    try {
        GProp_GProps props;
        BRepGProp::SurfaceProperties(s->shape, props);
        *out = props.Mass();
        return 0;
    } catch (...) {
        return -1;
    }
}

// Tight axis-aligned bounding box (no tolerance gap, exact geometry rather
// than triangulation). Writes xmin, ymin, zmin, xmax, ymax, zmax to out[0..6].
int swx_occt_bounding_box(const SwxShape* s, double* out) noexcept {
    if (s == nullptr || out == nullptr) {
        return -1;
    }
    try {
        Bnd_Box box;
        BRepBndLib::AddOptimal(s->shape, box, Standard_False, Standard_False);
        if (box.IsVoid()) {
            return -1;
        }
        box.Get(out[0], out[1], out[2], out[3], out[4], out[5]);
        return 0;
    } catch (...) {
        return -1;
    }
}

// Frees a handle returned by any constructor above. Null is a no-op.
void swx_occt_shape_free(SwxShape* s) noexcept {
    delete s;
}

}  // extern "C"
