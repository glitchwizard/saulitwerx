manual-synced-to: 6f4a61ad36d4069a8b1fdfdbad6659fd3acd5c53

# Manual coverage

Everything a user can do or set on `main`, found from the code, `docs/ROADMAP.md` and `docs/specs/PRODUCT.md`, with the manual section that covers it. Section numbers refer to the built PDF.

## Covered

| Capability (crate) | Where |
|---|---|
| Install from source; build with and without `--features occt` | 2, 9.2 |
| `swx-qa render`: `--fixture` (cube, cylinder, l-bracket), `--step`, `-o/--output`, `--size` (default 800x600, max 8192), `--crease` (default 30, 0–180), `-h/--help`, `help` | 3.1, 7, 9.1 |
| `swx-qa sketch`: `--fixture` (rectangle, triangle-345), `-o/--output`, `--size` (default 800x600, max 8192), `-h/--help`; plot conventions | 7.5, 9.2 |
| `swx-qa` success line, exit status, every error message (render and sketch) | 9.1, 9.2, 10.2 |
| `swx_qa::sketch::plot` (developers) | 7.5 |
| Kernel: `make_box`, `make_cylinder`, `make_sphere` | 6.1, 9.3 |
| Kernel: `union`, `cut`, `intersect` | 4.7, 6.2, 9.3 |
| Kernel: `volume`, `surface_area`, `bounding_box`, `face_count` | 3.2, 6.1, 9.3 |
| Kernel: `tessellate` (deflection) | 7.1, 9.3 |
| Kernel: `export_step`, `import_step` (mm) | 3.2, 6.1, 6.3, 9.3, 9.6 |
| Kernel errors (`InvalidInput`, `Backend`) | 10.2 |
| Sketch entities: point, line, circle, arc | 4.3, 9.4 |
| Constraints: coincident, horizontal, horizontal points, vertical, vertical points, fixed, distance, radius, diameter | 4.4, 4.5, 5, 9.4 |
| Solver: `SolveOptions` (tolerance, max_iterations, initial_damping), report, stop reasons | 5.1, 9.5 |
| DOF analysis: `dof::analyze`, `DofOptions` (rank_tolerance, residual_tolerance), `DofReport` (dof, rank, redundant, residual_norm, state), states incl. `NonFinite` | 4.6, 5.3, 9.7 |
| Sketch errors | 5.4, 10.2 |
| Document model: parameters, features (sketch, extrude, revolve), history, dirty propagation | 4.6, 4.8, 8, 9.2 |
| Document model errors | 8.3, 10.2 |
| Units (mm; degrees for feature parameters, radians for arcs) | 4.2, 9.6 |

## Examples the manual runs (`crates/*/examples/`)

| Example | Chapter |
|---|---|
| `swx-sketch` `plate_sketch` | 5 |
| `swx-sketch` `sketch_dof` | 5.3 |
| `swx-qa` `plate_sketch_plot` | 4.3–4.5, 5.1, 7.5 figures |
| `swx-kernel` `plate_with_hole` (occt) | 3, 6 |
| `swx-kernel` `plate_stages` (occt) | 4.7 figure |
| `swx-core` `feature_history` | 8 |

## Backlog

- Nothing built is undocumented. Open PRs #65 (M2-04 constraints) and #66 (M2-06 profiles) are not on `main`; document them once merged.
