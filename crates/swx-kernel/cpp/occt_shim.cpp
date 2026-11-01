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
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeVertex.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepLProp_SLProps.hxx>
#include <BRepMesh_IncrementalMesh.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <BRepTools.hxx>
#include <BRep_Tool.hxx>
#include <Bnd_Box.hxx>
#include <GProp_GProps.hxx>
#include <Geom_Plane.hxx>
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
#include <TopoDS_Vertex.hxx>
#include <TopoDS_Wire.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_ListIteratorOfListOfShape.hxx>
#include <gp_Ax2.hxx>
#include <gp_Ax3.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <gp_Pln.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>

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

// Planar face from oriented loops (OCCT reference documentation:
// BRepBuilderAPI_MakeEdge, BRepBuilderAPI_MakeWire, BRepBuilderAPI_MakeFace,
// BRepCheck_Analyzer).
//
// `plane` holds origin, x direction and normal (9 doubles, unit axes). The 2D
// point (u, v) maps to origin + u * x + v * (normal x x). `loop_sizes[l]` edges
// belong to loop l; loop 0 is the outer loop (counterclockwise about the
// normal), the rest are holes (clockwise). Each edge is 8 doubles: kind (0 line,
// 1/2 arc counter/clockwise, 3/4 circle counter/clockwise), from u v, to u v,
// center u v, radius. Edge i ends exactly where edge i + 1 starts, so the
// edges of a loop share one vertex at each junction. Fails if OCCT finds the
// face invalid, e.g. a self-intersecting loop or a hole outside the outer loop.
int swx_occt_make_face(const double* plane, size_t n_loops, const size_t* loop_sizes,
                       const double* edges, SwxShape** out) noexcept {
    if (plane == nullptr || loop_sizes == nullptr || edges == nullptr || out == nullptr ||
        n_loops == 0) {
        return -1;
    }
    try {
        const gp_Pnt origin(plane[0], plane[1], plane[2]);
        const gp_Dir x_dir(plane[3], plane[4], plane[5]);
        const gp_Dir normal(plane[6], plane[7], plane[8]);
        const gp_Ax3 axes(origin, normal, x_dir);
        const gp_Vec u_vec(axes.XDirection());
        const gp_Vec v_vec(axes.YDirection());
        auto to_3d = [&](double u, double v) { return origin.Translated(u_vec * u + v_vec * v); };

        std::vector<TopoDS_Wire> wires;
        const double* e = edges;
        for (size_t l = 0; l < n_loops; ++l) {
            const size_t n = loop_sizes[l];
            if (n == 0) {
                return -1;
            }
            std::vector<TopoDS_Vertex> vertices;
            for (size_t i = 0; i < n; ++i) {
                vertices.push_back(BRepBuilderAPI_MakeVertex(to_3d(e[8 * i + 1], e[8 * i + 2])));
            }
            BRepBuilderAPI_MakeWire wire;
            for (size_t i = 0; i < n; ++i, e += 8) {
                const int kind = static_cast<int>(e[0]);
                if (kind < 0 || kind > 4) {
                    return -1;
                }
                const TopoDS_Vertex& from = vertices[i];
                const TopoDS_Vertex& to = vertices[(i + 1) % n];
                const gp_Dir axis = (kind == 1 || kind == 3) ? normal : normal.Reversed();
                const gp_Circ circ(gp_Ax2(to_3d(e[5], e[6]), axis, x_dir), e[7]);
                BRepBuilderAPI_MakeEdge edge = [&]() {
                    switch (kind) {
                        case 0:
                            return BRepBuilderAPI_MakeEdge(from, to);
                        case 1:
                        case 2:
                            return BRepBuilderAPI_MakeEdge(circ, from, to);
                        default:
                            return BRepBuilderAPI_MakeEdge(circ);
                    }
                }();
                if (!edge.IsDone()) {
                    return -1;
                }
                wire.Add(edge.Edge());
                if (!wire.IsDone()) {
                    return -1;
                }
            }
            wires.push_back(wire.Wire());
        }

        BRepBuilderAPI_MakeFace face(gp_Pln(axes), wires[0], Standard_True);
        for (size_t l = 1; l < wires.size(); ++l) {
            face.Add(wires[l]);
        }
        if (!face.IsDone()) {
            return -1;
        }
        const TopoDS_Face result = face.Face();
        if (!BRepCheck_Analyzer(result).IsValid()) {
            return -1;
        }
        return emit(result, out);
    } catch (...) {
        return -1;
    }
}

// Solid swept from a planar face along its outward plane normal (OCCT
// reference documentation: BRepPrimAPI_MakePrism). The face is first moved
// `offset` along the normal, then swept `length` (> 0) along it, so the sweep
// always follows the face normal and the solid is never inside out.
int swx_occt_extrude(const SwxShape* s, double offset, double length, SwxShape** out) noexcept {
    if (s == nullptr || out == nullptr || !(length > 0.0)) {
        return -1;
    }
    try {
        if (s->shape.IsNull() || s->shape.ShapeType() != TopAbs_FACE) {
            return -1;
        }
        const TopoDS_Face& face = TopoDS::Face(s->shape);
        Handle(Geom_Plane) plane = Handle(Geom_Plane)::DownCast(BRep_Tool::Surface(face));
        if (plane.IsNull()) {
            return -1;
        }
        gp_Dir normal = plane->Axis().Direction();
        if (face.Orientation() == TopAbs_REVERSED) {
            normal.Reverse();
        }
        gp_Trsf shift;
        shift.SetTranslation(gp_Vec(normal) * offset);
        BRepPrimAPI_MakePrism prism(face.Moved(TopLoc_Location(shift)), gp_Vec(normal) * length);
        if (!prism.IsDone()) {
            return -1;
        }
        return emit(prism.Shape(), out);
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

// Maps a sub-shape kind code (0 face, 1 edge, 2 vertex) to its OCCT type.
static bool topo_kind(int kind, TopAbs_ShapeEnum* out) {
    switch (kind) {
        case 0: *out = TopAbs_FACE; return true;
        case 1: *out = TopAbs_EDGE; return true;
        case 2: *out = TopAbs_VERTEX; return true;
        default: return false;
    }
}

// Number of distinct sub-shapes of `kind` (shared ones counted once).
int swx_occt_sub_shape_count(const SwxShape* s, int kind, size_t* out) noexcept {
    TopAbs_ShapeEnum type;
    if (s == nullptr || out == nullptr || !topo_kind(kind, &type)) {
        return -1;
    }
    try {
        TopTools_IndexedMapOfShape map;
        TopExp::MapShapes(s->shape, type, map);
        *out = static_cast<size_t>(map.Extent());
        return 0;
    } catch (...) {
        return -1;
    }
}

// Indices (0-based, in the TopExp::MapShapes order of `kind`) of the
// sub-shapes of `kind` adjacent to sub-shape `of_index` of `of_kind`.
// Going up (e.g. faces of an edge) uses TopExp::MapShapesAndAncestors; going
// down (e.g. edges of a face) uses TopExp::MapShapes on the sub-shape. Writes
// at most `cap` indices to `out` and the number found to `len`; returns -1 if
// that number exceeds `cap`. May contain duplicates (e.g. a seam edge lists its
// face twice); the caller sorts and deduplicates.
int swx_occt_adjacent(const SwxShape* s, int of_kind, size_t of_index, int kind,
                      size_t* out, size_t cap, size_t* len) noexcept {
    TopAbs_ShapeEnum of_type;
    TopAbs_ShapeEnum type;
    if (s == nullptr || len == nullptr || (out == nullptr && cap > 0) ||
        !topo_kind(of_kind, &of_type) || !topo_kind(kind, &type) || of_kind == kind) {
        return -1;
    }
    try {
        TopTools_IndexedMapOfShape of_map;
        TopExp::MapShapes(s->shape, of_type, of_map);
        if (of_index >= static_cast<size_t>(of_map.Extent())) {
            return -1;
        }
        const TopoDS_Shape& of = of_map.FindKey(static_cast<int>(of_index) + 1);
        TopTools_IndexedMapOfShape kind_map;
        TopExp::MapShapes(s->shape, type, kind_map);

        size_t n = 0;
        auto push = [&](const TopoDS_Shape& sub) {
            const int i = kind_map.FindIndex(sub);
            if (i > 0) {
                if (n < cap) {
                    out[n] = static_cast<size_t>(i - 1);
                }
                ++n;
            }
        };
        if (kind < of_kind) {
            // Up: `kind` is the higher-dimensional one (face < edge < vertex).
            TopTools_IndexedDataMapOfShapeListOfShape ancestors;
            TopExp::MapShapesAndAncestors(s->shape, of_type, type, ancestors);
            const int key = ancestors.FindIndex(of);
            if (key > 0) {
                for (TopTools_ListIteratorOfListOfShape it(ancestors.FindFromIndex(key));
                     it.More(); it.Next()) {
                    push(it.Value());
                }
            }
        } else {
            TopTools_IndexedMapOfShape subs;
            TopExp::MapShapes(of, type, subs);
            for (int i = 1; i <= subs.Extent(); ++i) {
                push(subs.FindKey(i));
            }
        }
        *len = n;
        return n <= cap ? 0 : -1;
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
