// C ABI shim between swx-kernel (Rust) and OpenCascade (C++).
//
// Every exported function is `noexcept` and catches all C++ exceptions, so no
// exception ever unwinds across the FFI boundary. Failures are reported as a
// negative status code.
//
// Shapes cross the boundary as opaque `SwxShape*` handles that own a heap
// `TopoDS_Shape`. Rust frees each handle exactly once with `swx_occt_shape_free`.

#include <BRepAdaptor_Surface.hxx>
#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <BRepBndLib.hxx>
#include <BRepGProp.hxx>
#include <BRepLProp_SLProps.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <IFSelect_ReturnStatus.hxx>
#include <Poly_Triangulation.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_Writer.hxx>
#include <Standard_Failure.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <gp_Ax2.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <mutex>
#include <utility>
#include <vector>

struct SwxShape {
    TopoDS_Shape shape;
};

// Triangle mesh: xyz triples for positions and normals, three indices per triangle.
struct SwxMesh {
    std::vector<float> positions;
    std::vector<float> normals;
    std::vector<uint32_t> indices;
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

// OCCT's data-exchange layer keeps process-global state (static controllers
// and parameter tables), so STEP reads and writes are serialized.
std::mutex step_mutex;

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

// Triangulates every face with BRepMesh_IncrementalMesh at the given linear
// deflection (absolute, model units) and an angular deflection of 0.5 rad.
// Existing triangulations are cleared first so the result always reflects
// `deflection`. Nodes are not shared between faces. Normals come from the
// exact surface at each node's UV parameters, falling back to the
// triangle-averaged normal where the surface normal is undefined (e.g. at a
// sphere pole). Reversed faces get flipped winding and normals so that every
// triangle is counter-clockwise seen from outside.
int swx_occt_tessellate(const SwxShape* s, double deflection, SwxMesh** out) noexcept {
    if (s == nullptr || out == nullptr || !(deflection > 0.0)) {
        return -1;
    }
    try {
        BRepTools::Clean(s->shape);
        BRepMesh_IncrementalMesh mesher(s->shape, deflection, Standard_False, 0.5, Standard_False);
        if (!mesher.IsDone()) {
            return -1;
        }
        SwxMesh mesh;
        for (TopExp_Explorer ex(s->shape, TopAbs_FACE); ex.More(); ex.Next()) {
            const TopoDS_Face& face = TopoDS::Face(ex.Current());
            TopLoc_Location loc;
            Handle(Poly_Triangulation) tri = BRep_Tool::Triangulation(face, loc);
            if (tri.IsNull()) {
                return -1;
            }
            const bool reversed = face.Orientation() == TopAbs_REVERSED;
            const gp_Trsf trsf = loc.Transformation();
            tri->ComputeNormals();
            BRepAdaptor_Surface surface(face, Standard_False);
            BRepLProp_SLProps props(surface, 1, 1e-9);

            const size_t base = mesh.positions.size() / 3;
            if (base + tri->NbNodes() > std::numeric_limits<uint32_t>::max()) {
                return -1;
            }
            for (Standard_Integer i = 1; i <= tri->NbNodes(); ++i) {
                const gp_Pnt p = tri->Node(i).Transformed(trsf);
                gp_Dir n = tri->Normal(i).Transformed(trsf);
                if (tri->HasUVNodes()) {
                    const gp_Pnt2d uv = tri->UVNode(i);
                    props.SetParameters(uv.X(), uv.Y());
                    if (props.IsNormalDefined()) {
                        // BRepAdaptor_Surface already includes the face location.
                        n = props.Normal();
                    } else if (reversed) {
                        // Fallback normal is in triangle order; undo the flip below.
                        n.Reverse();
                    }
                }
                if (reversed) {
                    n.Reverse();
                }
                mesh.positions.insert(mesh.positions.end(),
                                      {static_cast<float>(p.X()), static_cast<float>(p.Y()),
                                       static_cast<float>(p.Z())});
                mesh.normals.insert(mesh.normals.end(),
                                    {static_cast<float>(n.X()), static_cast<float>(n.Y()),
                                     static_cast<float>(n.Z())});
            }
            for (Standard_Integer t = 1; t <= tri->NbTriangles(); ++t) {
                Standard_Integer a, b, c;
                tri->Triangle(t).Get(a, b, c);
                if (reversed) {
                    std::swap(b, c);
                }
                for (Standard_Integer k : {a, b, c}) {
                    mesh.indices.push_back(static_cast<uint32_t>(base + k - 1));
                }
            }
        }
        *out = new SwxMesh(std::move(mesh));
        return 0;
    } catch (...) {
        return -1;
    }
}

// Writes the vertex count and index count of a mesh.
int swx_occt_mesh_sizes(const SwxMesh* m, size_t* vertices, size_t* indices) noexcept {
    if (m == nullptr || vertices == nullptr || indices == nullptr) {
        return -1;
    }
    *vertices = m->positions.size() / 3;
    *indices = m->indices.size();
    return 0;
}

// Copies a mesh into caller buffers sized by swx_occt_mesh_sizes: positions and
// normals hold 3 * vertices floats each, indices holds `indices` values.
int swx_occt_mesh_copy(const SwxMesh* m, float* positions, float* normals,
                       uint32_t* indices) noexcept {
    if (m == nullptr || positions == nullptr || normals == nullptr || indices == nullptr) {
        return -1;
    }
    std::copy(m->positions.begin(), m->positions.end(), positions);
    std::copy(m->normals.begin(), m->normals.end(), normals);
    std::copy(m->indices.begin(), m->indices.end(), indices);
    return 0;
}

// Number of distinct faces (shared faces counted once).
int swx_occt_face_count(const SwxShape* s, size_t* out) noexcept {
    if (s == nullptr || out == nullptr) {
        return -1;
    }
    try {
        TopTools_IndexedMapOfShape faces;
        TopExp::MapShapes(s->shape, TopAbs_FACE, faces);
        *out = static_cast<size_t>(faces.Extent());
        return 0;
    } catch (...) {
        return -1;
    }
}

// Writes a shape to a STEP (ISO 10303-21) file at `path` (UTF-8, NUL-terminated)
// using OCCT's default application protocol and length unit (mm).
int swx_occt_export_step(const SwxShape* s, const char* path) noexcept {
    if (s == nullptr || path == nullptr) {
        return -1;
    }
    try {
        std::lock_guard<std::mutex> lock(step_mutex);
        STEPControl_Writer writer;
        if (writer.Transfer(s->shape, STEPControl_AsIs) != IFSelect_RetDone) {
            return -1;
        }
        return writer.Write(path) == IFSelect_RetDone ? 0 : -1;
    } catch (...) {
        return -1;
    }
}

// Reads a STEP file and returns all its root shapes as one shape (a compound
// when there are several). Fails if the file is unreadable or yields nothing.
int swx_occt_import_step(const char* path, SwxShape** out) noexcept {
    if (path == nullptr || out == nullptr) {
        return -1;
    }
    try {
        std::lock_guard<std::mutex> lock(step_mutex);
        STEPControl_Reader reader;
        if (reader.ReadFile(path) != IFSelect_RetDone) {
            return -1;
        }
        if (reader.TransferRoots() <= 0 || reader.NbShapes() <= 0) {
            return -1;
        }
        return emit(reader.OneShape(), out);
    } catch (...) {
        return -1;
    }
}

// Frees a mesh returned by swx_occt_tessellate. Null is a no-op.
void swx_occt_mesh_free(SwxMesh* m) noexcept {
    delete m;
}

// Frees a handle returned by any constructor above. Null is a no-op.
void swx_occt_shape_free(SwxShape* s) noexcept {
    delete s;
}

}  // extern "C"
