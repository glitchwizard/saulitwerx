# Architecture & decisions

These decisions are binding on all contributors, human or agent. Changing one needs an ADR entry at the bottom of this file and a PR labeled `needs-human`.

## Stack

| Concern | Decision | Why |
|---|---|---|
| Language | Rust (stable) | Memory safety in a large codebase, a first-class WASM target, and strict compiler feedback, which keeps unattended agent work honest. |
| Geometry kernel | OpenCascade (OCCT) 7.x behind a `swx-kernel` trait | The only mature open B-rep kernel: booleans, fillets, shelling, STEP. Writing a B-rep kernel is out of scope. The trait keeps a pure-Rust kernel (e.g. `truck`) possible later. |
| Kernel binding | Thin C ABI shim (`swx-kernel/cpp/occt_shim.cpp`, built with `cc`) linked dynamically against system OCCT 7.6+ (ADR 2026-10-13) | Smallest dependency surface, seconds-long builds, OCCT stays a separate shared library. `unsafe` confined to `swx-kernel::ffi`. |
| Sketch solver | Our own crate, `swx-sketch` | Newton/Levenberg–Marquardt on a sparse system, with DOF analysis. This keeps the core permissive and fully testable. PlaneGCS (LGPL) is a reference for behavior, not code to copy. |
| Document model | `swx-core`: parameters plus an ordered feature history, with dirty propagation | Kernel-agnostic, so rebuild logic is unit-testable without geometry. |
| Native file | `.sltwx` family (part `.sltwx`, assembly `.sltwxa` incl. its exploded views, drawing `.sltwxd`): ZIP container with `manifest.json` + `model.json` (source of truth) and an optional regenerable cache (OCCT BRep, glTF, PNG). Spec: `docs/specs/FILE_FORMAT.md` | Open formats can't store parametric history. The JSON model stays the source of truth; the cache makes large files open fast. The unpacked-folder form keeps git diffs useful. |
| Exchange | STEP AP242 and STL first, then 3MF, IGES and DXF | Open standards only (see CLEAN_ROOM.md). |
| UI | `egui` + `eframe` on `wgpu`, with a custom wgpu 3D viewport | Pure Rust, cross-platform, WASM-capable, permissive licenses. |

## Crate layout (target)

```
crates/
  swx-core     document model, parameters, feature history, rebuild ordering   (exists)
  swx-sketch   2D geometry + constraint solver
  swx-kernel   Kernel trait + OCCT implementation (feature "occt")
  swx-io       .sltwx family, STEP, STL
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
- Allowed dependency licenses: MIT, Apache-2.0, BSD (2- and 3-clause), 0BSD, ISC, Zlib, BSL-1.0, Unicode-3.0, Unicode-DFS-2016, CC0-1.0, MPL-2.0 (file-level), and LGPL (dynamic only). GPL, AGPL, proprietary licenses and unlicensed code are forbidden; FACTORY §8a says what to do instead.

### Third-party dependency register

| Dependency | License | Used by | Notes |
|---|---|---|---|
| _(none yet)_ | | | |
| `cc` 1.x | MIT OR Apache-2.0 | swx-kernel (build, feature `occt`) | Compiles the OCCT C ABI shim. |
| `shlex` 2.x | MIT OR Apache-2.0 | swx-kernel (build, via `cc`) | Transitive. |
| `find-msvc-tools` 0.1 | MIT OR Apache-2.0 | swx-kernel (build, via `cc`) | Transitive. |
| OpenCascade (OCCT) 7.6.3, system packages | LGPL-2.1 with OCCT exception | swx-kernel (feature `occt`) | Dynamic only (`libTK*.so`); not vendored. |
| `png` 0.17 | MIT OR Apache-2.0 | swx-qa | Writes QA render PNGs. |
| `flate2` 1.x | MIT OR Apache-2.0 | swx-qa | Transitive (via `png`). |
| `miniz_oxide` 0.8, 0.9 | MIT OR Zlib OR Apache-2.0 | swx-qa | Transitive (via `png`, `flate2`). |
| `fdeflate` 0.3 | MIT OR Apache-2.0 | swx-qa | Transitive (via `png`). |
| `crc32fast` 1.x | MIT OR Apache-2.0 | swx-qa | Transitive (via `png`, `flate2`). |
| `adler2` 2.x | 0BSD OR MIT OR Apache-2.0 | swx-qa | Transitive (via `miniz_oxide`). |
| `simd-adler32` 0.3 | MIT | swx-qa | Transitive (via `fdeflate`, `miniz_oxide`). |
| `bitflags` 1.x | MIT OR Apache-2.0 | swx-qa | Transitive (via `png`). |
| `cfg-if` 1.x | MIT OR Apache-2.0 | swx-qa | Transitive (via `crc32fast`). |
| Typst 0.15.1 (release binary, SHA-256 pinned) | Apache-2.0 | docs/manual (build tool) | Typesets the user manual; fetched by `docs/manual/build.sh`, not vendored. |
| Inter 4.0 font (Regular, Italic, SemiBold, Bold) | OFL-1.1 | docs/manual | Headings and labels; files and license in `docs/manual/fonts/`. |
| `serde` 1.x | MIT OR Apache-2.0 | swx-io | Serialization traits; without the `derive` feature. |
| `serde_core` 1.x | MIT OR Apache-2.0 | swx-io | Transitive (via `serde`). |
| `serde_json` 1.x | MIT OR Apache-2.0 | swx-io | JSON parsing and values for `.sltwx` files. |
| `itoa` 1.x | MIT OR Apache-2.0 | swx-io | Transitive (via `serde_json`). |
| `memchr` 2.x | Unlicense OR MIT (used under MIT) | swx-io | Transitive (via `serde_json`). |
| `zmij` 1.x | MIT | swx-io | Transitive (via `serde_json`). |
| `serde_derive` 1.x | MIT OR Apache-2.0 | none (Cargo.lock only) | Optional dependency of `serde`, recorded in `Cargo.lock` but not built: the `derive` feature is off. |
| `syn` 3.x, `quote` 1.x, `proc-macro2` 1.x | MIT OR Apache-2.0 | none (Cargo.lock only) | Transitive (via `serde_derive`); not built. |
| `unicode-ident` 1.x | (MIT OR Apache-2.0) AND Unicode-3.0 | none (Cargo.lock only) | Transitive (via `proc-macro2`); not built. Unicode-3.0 is not on the allowed list, which is why `swx-io` leaves `serde`'s `derive` feature off. |
| `flate2` 1.x | MIT OR Apache-2.0 | swx-io | Deflate for the `.sltwx` ZIP container (M4-01d). |
| `crc32fast` 1.x | MIT OR Apache-2.0 | swx-io | CRC-32 of ZIP entries (M4-01d). |
| `miniz_oxide` 0.9 | MIT OR Zlib OR Apache-2.0 | swx-io | Transitive (via `flate2`). |
| `adler2` 2.x | 0BSD OR MIT OR Apache-2.0 | swx-io | Transitive (via `miniz_oxide`). |
| `simd-adler32` 0.3 | MIT | swx-io | Transitive (via `miniz_oxide`). |
| `cfg-if` 1.x | MIT OR Apache-2.0 | swx-io | Transitive (via `crc32fast`). |

## ADR log

- 2026-10-12: initial stack chosen (this document).
- 2026-10-13 (M1-01): OCCT binding = thin `extern "C"` shim compiled by `cc`, linked with `cargo:rustc-link-lib=dylib=TK*` against system OCCT (Ubuntu 24.04 `libocct-*-dev` 7.6.3; overridable via `OCCT_INCLUDE_DIR`/`OCCT_LIB_DIR`). Measured clean build of `swx-kernel --features occt`: ~2.2 s (only the shim compiles; OCCT is prebuilt). Cache strategy: apt packages + standard cargo `target/` cache; nothing large to cache. Rejected (not measured in this spike): bundled-source `opencascade-sys` (compiles all of OCCT from source on a cold cache, linked statically by default) and `cxx` (extra proc-macro deps; revisit if the API surface grows). Shim functions are `noexcept` and catch all C++ exceptions.
- 2026-10-13: native format changed from text-only `.swx` to the `.sltwx` family (ZIP + JSON + optional cache, unpacked-folder form for git). See `docs/specs/FILE_FORMAT.md`.
- 2026-10-18 (QA-01): new crate `swx-qa` (binary `swx-qa`, `publish = false`): headless CPU renderer for QA evidence. Depends only on `swx-kernel` among workspace crates (feature `occt` forwards to `swx-kernel/occt` for STEP input); pure-Rust PNG output via `png`, no GPU or system graphics libraries.
- 2026-10-23 (QA-02): `swx-qa` gains a path dependency on `swx-sketch` (both workspace crates; `swx-sketch` stays kernel-free) for `swx-qa sketch`, which plots solved sketches to PNG. No new third-party dependencies: dimension text uses a small bitmap font written in `swx-qa`.
- 2026-10-27 (M2-07b): `swx-sketch` gains a path dependency on `swx-core` (`sketch → core`, allowed direction; neither depends on `swx-kernel`) so sketch dimensions can be bound to the document parameter table (`swx_sketch::params`). No new third-party dependencies.
- 2026-11-11 (M4-01a): new crate `swx-io` (`publish = false`) for the `.sltwx` family, starting with `manifest.json` and the canonical JSON writer. No workspace dependencies. Third-party: `serde` (without `derive`) and `serde_json`, so the manifest is built as a `serde_json::Value` by hand rather than derived.
- 2026-11-12 (M4-01c): `swx-io` now builds `serde` with its `derive` feature, and `manifest.json` is written through derived `Serialize`. So `serde_derive`, `syn`, `quote`, `proc-macro2` and `unicode-ident` (rows already in the register) are now built, not only recorded in `Cargo.lock`. Unicode-3.0 has been on the allowed list since #198. No other new crates.
