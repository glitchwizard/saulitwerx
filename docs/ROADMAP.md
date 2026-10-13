# Roadmap

The Planner turns the **first unchecked items** into GitHub issues. Each item ID (e.g. `M1-02`) is copied into its issue title, so this file and the issue queue stay linked. Check an item off only when its issue's PR has merged.

Milestones are sequential. Don't start M(n+1) items while M(n) has unchecked items, unless the item is explicitly marked _(parallel ok)_.

## M0: Foundation

- [x] M0-01 Cargo workspace, CI (fmt, clippy, test), licenses, factory docs
- [x] M0-02 `swx-core`: parameters, feature history, dirty propagation

## M1: Kernel spike (OCCT)

- [ ] M1-01 Spike: choose the OCCT binding approach. Compare build time, CI cacheability and dynamic linking. Record an ADR in ARCHITECTURE.md. Deliverable: a `swx-kernel` crate that makes a box and reports its volume in a test. OCCT code sits behind the cargo feature `occt`, off by default, so the current CI stays green until M1-02.
- [ ] M1-02 CI job for `swx-kernel` with OCCT cached (separate workflow job, feature `occt`). Touches `.github/`, so the owner merges it (FACTORY §8).
- [ ] M1-03 `Kernel` trait: primitives (box, cylinder, sphere), boolean union/cut/intersect, and volume/area/bbox queries. Regression tests with tolerances.
- [ ] M1-04 Tessellation: `Shape → Mesh` (positions, normals, indices) with deflection control and tests.
- [ ] M1-05 STEP export plus round-trip import. Golden tests compare volume and face count.

## M2: Sketcher

- [ ] M2-01 `swx-sketch` geometry: point, line segment, circle, arc. Parameter vector layout.
- [ ] M2-02 Solver core: residual/Jacobian assembly, damped Newton / LM, convergence reporting.
- [ ] M2-03 Constraints: coincident, horizontal, vertical, fixed, distance, radius/diameter.
- [ ] M2-04 Constraints: parallel, perpendicular, tangent (line–arc, arc–arc), equal, midpoint, symmetric.
- [ ] M2-05 DOF analysis: report under-, fully- and over-constrained states, and conflicting constraints.
- [ ] M2-06 Profile extraction: closed loops → regions with holes, ready for the kernel.
- [ ] M2-07 Dimensions bound to document parameters (`swx-core` integration).

## M3: Part features

- [ ] M3-01 Sketch → kernel face; extrude (blind, symmetric) as a new body.
- [ ] M3-02 Extrude cut, through-all, and up-to-depth.
- [ ] M3-03 Revolve and revolve cut.
- [ ] M3-04 Topological naming v1: persistent face/edge references across rebuilds. Write a design doc first in `docs/specs/`.
- [ ] M3-05 Fillet and chamfer on referenced edges.
- [ ] M3-06 Shell, draft, and hole (simple, counterbore).
- [ ] M3-07 Full rebuild pipeline: parameter edit → dirty features → regenerate → regression test.

## M4: Files

- [ ] M4-01 `.swx` native format (versioned, RON or JSON) with save/load round-trip tests.
- [ ] M4-02 STL export with tolerance options.
- [ ] M4-03 STEP import as a dumb solid feature.

## M5: Desktop app

- [ ] M5-01 `swx-app` shell: eframe window, menus, open/save.
- [ ] M5-02 wgpu viewport: orbit/pan/zoom, shaded + edges, view cube.
- [ ] M5-03 Feature tree panel: select, rename, suppress, edit (opens the property panel), rollback bar.
- [ ] M5-04 Sketch mode: draw tools, snapping, constraint glyphs, dimension editing.
- [ ] M5-05 Face/edge picking, wired to feature inputs.
- [ ] M5-06 Property panel for extrude, revolve and fillet, with live preview.

## M6: Productivity

- [ ] M6-01 Linear/circular pattern and mirror.
- [ ] M6-02 Reference geometry: planes, axes, points.
- [ ] M6-03 Equations / global variables editor.
- [ ] M6-04 Measure tool and mass properties.

## Later (not yet scheduled)

Assemblies with mates, 2D drawings with dimensions, a WASM/browser build, sheet metal, and configurations.
