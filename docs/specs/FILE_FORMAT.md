# Native file format (`.sltwx` family)

Status: **v0 design**. Implemented starting at roadmap item M4-01. Changes to this file need an owner-merged PR.

## Why a native format

Open exchange formats (STEP, STL, 3MF, glTF) store only *results*: solid geometry or meshes. They can't hold what makes a model editable: parameters and expressions, sketches with constraints, the ordered feature history, assembly joints, and drawing views. So Saulitwerx has its own format for its documents. The format is **fully documented here and free for anyone to implement**. Open formats are used for everything else, both inside the container and for import/export.

## Extensions

| Kind | Extension | `kind` value |
|---|---|---|
| Part | `.sltwx` | `part` |
| Assembly | `.sltwxa` | `assembly` |
| Drawing | `.sltwxd` | `drawing` |

Exploded views are stored inside the assembly (see below), not as a separate kind.

The extension is a convenience for users and file browsers. **`manifest.json` → `kind` is authoritative.** A mismatch is a load error with a clear message.

## Container

A standard ZIP archive (deflate) with this layout:

```
<name>.sltwx
├── manifest.json        required
├── model.json           required: the source of truth
├── properties.json      optional: document properties
├── cache/               optional: derived data, always regenerable
│   ├── body-<id>.brep   solved solid per body (OCCT BRep text format)
│   └── display.glb      display mesh (glTF 2.0 binary)
├── thumbnail.png        optional: 256×256 preview
└── attachments/         optional: embedded images, part-family tables (CSV), etc.
```

Open standards used inside: ZIP, JSON (UTF-8, RFC 8259), PNG, glTF 2.0, and CSV. The OCCT BRep text format is documented by the open-source OCCT project.

### Unpacked (git-friendly) form

A directory named `<name>.sltwx/` with the identical layout is also a valid document. The app opens both forms. `sltwx pack` and `sltwx unpack` convert between them. The JSON writer emits **stable key order, 2-space indent, and one array element per line**, so version-control diffs are meaningful.

## `manifest.json`

```json
{
  "format": "sltwx",
  "format_version": "0.1.0",
  "kind": "part",
  "document_id": "6f1c2e0a-7b7d-4c55-9d3e-2a1f0e4b9c11",
  "units": { "length": "mm", "angle": "deg", "mass": "kg" },
  "created_by": "saulitwerx 0.3.0",
  "saved_by": "saulitwerx 0.4.1",
  "model_hash": "sha256:…"
}
```

- `document_id` is a UUID that stays fixed for the document's life. Assemblies reference parts by **relative path + document_id**, so a moved file can be found again and a substituted file is detected.
- `model_hash` is the SHA-256 of the canonical `model.json`. Each cache file records the hash it was built from. If they differ, the cache is stale: ignore it and rebuild.

## `model.json` (part)

The editable model. Everything in `cache/` can be regenerated from this file alone.

```json
{
  "parameters": [
    { "name": "d0", "kind": "model", "expr": "40 mm" },
    { "name": "thickness", "kind": "user", "expr": "d0 / 8", "comment": "plate thickness" }
  ],
  "origin": { "planes": ["XY", "XZ", "YZ"], "axes": ["X", "Y", "Z"], "point": "O" },
  "sketches": [{
    "id": "sk-7f3a",
    "placement": { "type": "plane", "ref": "origin:XY" },
    "entities": [
      { "id": "e1", "type": "line", "p1": [0, 0], "p2": [40, 0] }
    ],
    "constraints": [
      { "id": "c1", "type": "horizontal", "of": ["e1"] },
      { "id": "c2", "type": "distance", "of": ["e1"], "param": "d0" }
    ]
  }],
  "features": [
    { "id": "ft-91c2", "type": "extrude", "name": "Extrusion1",
      "profile": { "sketch": "sk-7f3a", "regions": ["*"] },
      "extent": { "type": "distance", "param": "thickness", "direction": "positive" },
      "op": "new_solid", "suppressed": false }
  ],
  "end_of_part": "ft-91c2",
  "bodies": [{ "id": "b-1", "created_by": "ft-91c2", "name": "Solid1" }]
}
```

Rules:

- **Stable IDs.** Every parameter, sketch, entity, constraint, feature, body and (later) joint has an ID that never changes once assigned. References always use IDs, never list positions or display names.
- **Geometry references** (faces, edges) use the persistent-naming scheme from M3-04. They are never raw kernel indices.
- **Values live in parameters.** Features and constraints reference parameter names. Literal numbers appear only inside parameter expressions.
- **Order matters** only for `features` (history order). All other arrays are sets, sorted by `id` on write.

## Assemblies and drawings

Same container. `model.json` changes shape:

- **Assembly:** `components` (each with `source: { path, document_id }`, `transform`, `grounded`, `visible`, and `bom_structure`), `joints` (type, two component references with geometry refs, and limits), `constraints`, `patterns`, and `exploded_views` (each with `id`, `name`, ordered `steps` → `tweaks` (component, transform, trail flag)). Tweaks are offsets applied on top of the solved assembly; they never change joints or constraints.
- **Drawing:** `sheets` → `views` (each with `source`, optional `exploded_view` ID, view type, scale, orientation, and section/detail definitions), `dimensions`, `annotations`, and `parts_lists`.

The detailed schemas are written as each milestone lands (M7–M9).

## Versioning

- `format_version` uses semver. **Minor and patch** versions only add fields. **Major** versions may restructure; the app migrates older majors forward on open and never silently downgrades.
- Readers keep fields they don't recognize and write them back unchanged, so a newer file round-trips through an older app without data loss.
- A JSON Schema per `format_version` is published in `docs/specs/schema/`.

## Import / export (not the native format)

STEP AP242 (main exchange), STL, 3MF, glTF, and DXF/PDF for drawings. Other vendors' native formats are out of scope permanently (see PRODUCT.md).
