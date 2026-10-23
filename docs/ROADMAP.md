# Roadmap

The Planner turns the **first unchecked items** into GitHub issues. Each item ID (e.g. `M1-02`) is copied into its issue title, so this file and the issue queue stay linked. Check an item off only when its issue's PR has merged.

Milestones are sequential. Don't start M(n+1) items while M(n) has unchecked items, unless the item is explicitly marked _(parallel ok)_.

## M0: Foundation

- [x] M0-01 Cargo workspace, CI (fmt, clippy, test), licenses, factory docs
- [x] M0-02 `swx-core`: parameters, feature history, dirty propagation

## M1: Kernel spike (OCCT)

- [x] M1-01 Spike: choose the OCCT binding approach. Compare build time, CI cacheability and dynamic linking. Record an ADR in ARCHITECTURE.md. Deliverable: a `swx-kernel` crate that makes a box and reports its volume in a test. OCCT code sits behind the cargo feature `occt`, off by default, so the current CI stays green until M1-02.
- [x] M1-02 CI job for `swx-kernel` with OCCT cached (separate workflow job, feature `occt`). Touches `.github/`, so the owner merges it (FACTORY §8).
- [x] M1-03 `Kernel` trait: primitives (box, cylinder, sphere), boolean union/cut/intersect, and volume/area/bbox queries. Regression tests with tolerances.
- [x] M1-04 Tessellation: `Shape → Mesh` (positions, normals, indices) with deflection control and tests.
- [x] M1-05 STEP export plus round-trip import. Golden tests compare volume and face count.

## QA tooling

Tools the Builder uses for the visuals in its QA reports (FACTORY §6b).

- [x] QA-01 _(parallel ok)_ `swx-qa render`: a headless CPU renderer (no GPU, deterministic) that renders a built-in fixture or a STEP file to an isometric, flat-shaded PNG with feature edges.
- [x] QA-02 _(parallel ok, after M2-03)_ `swx-qa sketch`: plot a solved sketch (geometry, points, dimension values) to a PNG.

## M2: Sketcher

- [x] M2-01 `swx-sketch` geometry: point, line segment, circle, arc. Parameter vector layout.
- [x] M2-02 Solver core: residual/Jacobian assembly, damped Newton / LM, convergence reporting.
- [x] M2-03 Constraints: coincident, horizontal, vertical, fixed, distance, radius/diameter.
- [ ] M2-04 Constraints: parallel, perpendicular, tangent (line–arc, arc–arc), equal, midpoint, symmetric.
- [x] M2-05 DOF analysis: report under-, fully- and over-constrained states, and conflicting constraints.
- [ ] M2-06 Profile extraction: closed loops → regions with holes, ready for the kernel.
- [ ] M2-07 Dimensions bound to document parameters (`swx-core` integration). Every dimension auto-creates a model parameter `d0`, `d1`, … (PRODUCT §2.2).
- [ ] M2-08 Parameter expressions with units (`d3 = d1 / 2 + 5 mm`), user parameters, dependency ordering and cycle detection.
- [ ] M2-09 Sketch inference: auto-apply coincident/horizontal/vertical/tangent/perpendicular/parallel while drawing (solver-level API, no UI).

## M3: Part features

- [ ] M3-01 Sketch → kernel face; extrude (distance, symmetric) with join / cut / intersect / new solid.
- [ ] M3-02 Extrude extents: through all, to face, between faces.
- [ ] M3-03 Revolve and revolve cut.
- [ ] M3-04 Topological naming v1: persistent face/edge references across rebuilds. Write a design doc first in `docs/specs/`.
- [ ] M3-05 Fillet and chamfer on referenced edges.
- [ ] M3-06 Shell, draft, and hole (simple, counterbore).
- [ ] M3-07 Full rebuild pipeline: parameter edit → dirty features → regenerate → regression test.
- [ ] M3-08 Part origin (3 planes, 3 axes, center point) and work features: work planes, axes, points.
- [ ] M3-09 End-of-part marker: roll back and insert features above it.
- [ ] M3-10 Multi-body parts and combine (join/cut/intersect between bodies).
- [ ] M3-11 Sweep and loft.
- [ ] M3-12 Hole feature v2: countersink, tapped, thread table data.

## M4: Files

- [ ] M4-01 Native `.sltwx` part files per `docs/specs/FILE_FORMAT.md`: ZIP container, `manifest.json`, `model.json`, stable IDs, round-trip tests, plus the unpacked-folder form.
- [ ] M4-04 Derived cache (`cache/*.brep`, `display.glb`, `thumbnail.png`) with `model_hash` staleness check.
- [ ] M4-05 `sltwx pack` / `unpack` CLI and the published JSON Schema for format v0.1.
- [ ] M4-02 STL export with tolerance options.
- [ ] M4-03 STEP import as a dumb solid feature.

## M5: Desktop app

- [ ] M5-01 `swx-app` shell: eframe window, menus, open/save.
- [ ] M5-02 wgpu viewport: shaded + edges, orientation cube, home view. Default mouse: wheel zooms to cursor, middle-drag pans, Shift + middle-drag orbits (PRODUCT §8).
- [ ] M5-03 Model browser: origin folder, features, end-of-part marker; select, rename, suppress, edit.
- [ ] M5-04 Sketch mode: draw tools, snapping, constraint glyphs, dimension editing.
- [ ] M5-05 Face/edge picking, wired to feature inputs.
- [ ] M5-06 Property panels for extrude, revolve and fillet, with live preview and OK / Apply / Cancel.
- [ ] M5-07 Ribbon with per-environment tabs (part, sketch) and a contextual sketch tab with Finish sketch.
- [ ] M5-08 Right-click radial menu and selection mini-toolbar.
- [ ] M5-09 Single-letter command aliases and a remappable shortcut/mouse-preset system.

## M6: Productivity

- [ ] M6-01 Rectangular / circular / path patterns and mirror (features and bodies).
- [ ] M6-02 Shell, face draft, thread, split.
- [ ] M6-03 Parameters dialog: model + user parameters, units, expressions, comments.
- [ ] M6-04 Measure tool and mass properties.

## MB: Benchmark parts through the UI (`docs/specs/BENCHMARK_PARTS.md`)

The project goal: an agent models three real catalog parts (an M8 hex nut and an M8 hex head screw, both with modeled threads, and an 8 mm clamp-on shaft collar) from scratch, using only the UI, checked in CI.

- [ ] MB-00 _(owner)_ Confirm the three catalog parts and their drawing dimensions in BENCHMARK_PARTS §2. The Planner skips this item and doesn't start MB-05 to MB-07 until it's checked.
- [ ] MB-01 _(parallel ok, after M5-01)_ UI test harness: run `swx-app` headless in a test, drive it only through UI input addressed by accessible label, and take screenshots. ADR for the harness crate, and add it to the dependency register.
- [ ] MB-02 _(parallel ok, after MB-01)_ Agent driver: a mode that takes one UI action per line on stdin and replies with an accessibility-tree summary and a screenshot path. It records the session as a replayable `benchmarks/<part>/build.ui.json`.
- [ ] MB-03 _(parallel ok, after M3-11)_ Helix feature and modeled threads (external on a cylindrical face, internal in a hole) with the ISO 68-1 profile and 6g/6H offsets from BENCHMARK_PARTS §3. Tests check the pitch diameter and the nut-on-screw interference test from §5.
- [ ] MB-04 CI job that replays every `benchmarks/*/build.ui.json`, runs the §5 checks, and uploads screenshots and STL as artifacts. Touches `.github/`, so the owner merges it (FACTORY §8).
- [ ] MB-05 BP-1 hex nut M8 × 1.25, modeled through the UI and passing BENCHMARK_PARTS §5.
- [ ] MB-06 BP-2 hex head screw M8 × 1.25 × 40, modeled through the UI and passing §5, including the nut-on-screw thread check.
- [ ] MB-07 BP-3 clamp-on shaft collar, 8 mm bore with a tapped M3 clamp hole, modeled through the UI and passing §5.
- [ ] MB-08 _(owner, stretch)_ Pick three random Thingiverse objects per BENCHMARK_PARTS §6 and write their briefs. The Planner skips this item. Once it's checked, the Planner adds one modeling item per object.

## M7: Assemblies (joint-first, PRODUCT §4)

- [ ] M7-01 Assembly document (`.sltwxa`): place parts and subassemblies; first component grounded.
- [ ] M7-02 Joints: rigid, rotational, slider; DOF solver for assemblies.
- [ ] M7-03 Joints: cylindrical, planar, ball; joint limits.
- [ ] M7-04 Constraints: mate/flush, angle, tangent, insert, symmetry.
- [ ] M7-05 Drag components within remaining DOF; interference check.
- [ ] M7-06 Component patterns/mirror, replace component, BOM structure.
- [ ] M7-07 Edit a part in place within the assembly.

## M8: Drawings (PRODUCT §5)

- [ ] M8-01 Drawing document (`.sltwxd`), sheets, template title block.
- [ ] M8-02 Base and projected views with hidden-line removal.
- [ ] M8-03 Section and detail views.
- [ ] M8-04 Retrieved model dimensions plus drawing dimensions, center marks and centerlines.
- [ ] M8-05 Parts list and balloons from the BOM.
- [ ] M8-06 PDF and DXF export.

## M9: Exploded views (PRODUCT §6, stored in the assembly)

- [ ] M9-01 Named exploded views in `.sltwxa`: tweaks (translate/rotate) with trail lines, applied on top of the solved assembly.
- [ ] M9-02 Ordered steps with animation playback; drawing views can use an exploded view.

## Later (not yet scheduled; see PRODUCT §7)

Parameters automation (rules), part families, sheet metal, structural frames, standard parts library, design calculators, simulation, photoreal rendering, and a WASM/browser build.
