# Architecture & decisions

These decisions are binding on all contributors, human or agent. Changing one needs an ADR entry at the bottom of this file and a PR labeled `needs-human`.

## Stack

| Concern | Decision | Why |
|---|---|---|
| Language | Rust (stable) | Memory safety in a large codebase, a first-class WASM target, and strict compiler feedback, which keeps unattended agent work honest. |
| Geometry kernel | OpenCascade (OCCT) 7.x behind a `swx-kernel` trait | The only mature open B-rep kernel: booleans, fillets, shelling, STEP. Writing a B-rep kernel is out of scope. The trait keeps a pure-Rust kernel (e.g. `truck`) possible later. |
| Kernel binding | Decided in roadmap item M1-01 (spike) | Candidates: `opencascade-sys` (bundled build), the same crate against a system OCCT, or a thin `cxx` bridge. Requirements: CI under 15 minutes with caching, and dynamic linking for license reasons. |
| Sketch solver | Our own crate, `swx-sketch` | Newton/Levenberg–Marquardt on a sparse system, with DOF analysis. This keeps the core permissive and fully testable. PlaneGCS (LGPL) is a reference for behavior, not code to copy. |
| Document model | `swx-core`: parameters plus an ordered feature history, with dirty propagation | Kernel-agnostic, so rebuild logic is unit-testable without geometry. |
| Native file | `.swx`: UTF-8 RON/JSON of parameters and history (no B-rep cache) | Diffable and mergeable. Geometry is always regenerated. |
| Exchange | STEP AP242 and STL first, then 3MF, IGES and DXF | Open standards only (see CLEAN_ROOM.md). |
| UI | `egui` + `eframe` on `wgpu`, with a custom wgpu 3D viewport | Pure Rust, cross-platform, WASM-capable, permissive licenses. |

## Crate layout (target)

```
crates/
  swx-core     document model, parameters, feature history, rebuild ordering   (exists)
  swx-sketch   2D geometry + constraint solver
  swx-kernel   Kernel trait + OCCT implementation (feature "occt")
  swx-io       .swx, STEP, STL
  swx-render   tessellation upload, camera, picking (wgpu)
  swx-app      desktop application (eframe)
```

Dependency direction: `app → render, io → kernel, sketch → core`. `core` and `sketch` must never depend on the kernel.

## Testing strategy

- **Unit tests** for every crate. The solver and document model must be fully testable without OCCT.
- **Geometry regression tests** (`swx-kernel`, `swx-io`): build a model and assert volume, surface area, bounding box, and face/edge counts against tolerances. Golden STEP files live in `tests/golden/`.
- **Topological naming**: every feature that references faces or edges needs a test that edits an upstream parameter and checks the reference still resolves.
- CI must stay green on `main`. A red `main` is the top-priority issue.

## Licensing

- Saulitwerx code: MIT OR Apache-2.0.
- OCCT: LGPL-2.1 with the OCCT exception. It is used as a separately, dynamically linked library. Binary releases ship OCCT as shared libraries alongside its license and a pointer to its source. This keeps Saulitwerx itself permissive.
- Allowed dependency licenses: MIT, Apache-2.0, BSD, Zlib, ISC, MPL-2.0 (file-level), and LGPL (dynamic only). GPL and AGPL are forbidden.

### Third-party dependency register

| Dependency | License | Used by | Notes |
|---|---|---|---|
| _(none yet)_ | | | |

## ADR log

- 2026-10-12: initial stack chosen (this document).
