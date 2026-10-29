manual-synced-to: b1e60128e238ef93db9d5a98e8a8f967312e2408

# Manual coverage

Everything a user can do or set on `main`, found from the code, `docs/ROADMAP.md` and `docs/specs/PRODUCT.md`, with the manual section that covers it. Section numbers refer to the built PDF.

## Covered

| Capability (crate) | Where |
|---|---|
| Install from source; build with and without `--features occt` | 2, 9.3 |
| `swx-qa render`: `--fixture` (cube, cylinder, l-bracket), `--step`, `-o/--output`, `--size` (default 800x600, max 8192), `--crease` (default 30, 0–180), `-h/--help`, `help` | 3.1, 7, 9.1 |
| `swx-qa sketch`: `--fixture` (rectangle, triangle-345), `-o/--output`, `--size` (default 800x600, max 8192), `-h/--help`; plot conventions | 7.5, 9.2 |
| `swx-qa` success line, exit status, every error message (render and sketch) | 9.1, 9.2, 10.2 |
| `swx_qa::sketch::plot` (developers) | 7.5 |
| Kernel: `make_box`, `make_cylinder`, `make_sphere` | 6.1, 9.4 |
| Kernel: `union`, `cut`, `intersect` | 4.9, 6.2, 9.4 |
| Kernel: `volume`, `surface_area`, `bounding_box`, `face_count` | 3.2, 6.1, 9.4 |
| Kernel: `tessellate` (deflection) | 7.1, 9.4 |
| Kernel: `export_step`, `import_step` (mm) | 3.2, 6.1, 6.3, 9.4, 9.10 |
| Kernel errors (`InvalidInput`, `Backend`) | 10.2 |
| Sketch entities: point, line, circle, arc (solving changes arc angles; sweep kept in (0, 2π]) | 4.3, 9.5 |
| `PointRef`: Point, Start, End, Center, ArcStart, ArcEnd (arc ends in every point constraint) | 5.1, 5.6, 9.5 |
| Constraints: coincident, horizontal, horizontal points, vertical, vertical points, fixed, distance, radius, diameter | 4.4, 4.5, 5, 9.5 |
| Constraints: parallel, perpendicular, equal (lines; circles/arcs), midpoint, symmetric, tangent (line–circle/arc, circle/arc–circle/arc; side and internal/external rules; endpoint form when a line end is coincident with an arc end) | 4.4, 5.5, 5.6, 9.5 |
| Profiles: `profiles`, `profile_report`, `ProfileOptions` (weld_tolerance), `ProfileReport` (regions, open_edges), `Region` (outer, holes, `area()`), `Loop` (edges, signed_area), `OrientedEdge`, nesting rules | 4.7, 5.7, 9.7 |
| Profile errors (`InvalidTolerance`, `SelfIntersecting`, `Sketch`) | 5.7, 10.2 |
| Solver: `SolveOptions` (tolerance, max_iterations, initial_damping), report, stop reasons | 5.1, 9.6 |
| DOF analysis: `dof::analyze`, `DofOptions` (rank_tolerance, residual_tolerance), `DofReport` (dof, rank, redundant, residual_norm, state), states incl. `NonFinite`; equations per constraint | 4.6, 5.3, 9.8 |
| DOF by constraint: `Sketch::dof`, `SketchDofReport` (report, redundant_constraints), `Sketch::constraint_of_row` | 4.6, 5.3, 9.8 |
| Sketch errors | 5.4, 10.2 |
| Document model: parameters, features (sketch, extrude, revolve), history, dirty propagation | 4.8, 4.10, 8 |
| Parameter table: `add_model_param` (d0, d1, …), `add_user_param`, `rename_param` (features follow), `set_param_comment`, `parameters`, `is_valid_param_name`, `Parameter` (name, kind, value, unit, comment), `ParamKind` | 4.8, 8.3, 9.9 |
| Document model errors (incl. `DuplicateParameter`, `InvalidParameterName`) | 8.4, 10.2 |
| Sketch dimensions bound to parameters: `swx_sketch::params` `bind_dimension`, `apply_params`, `rename_bound`, `ParamError`; `Sketch::bindings`, `Sketch::set_dimension_value`; `SketchError::UnknownConstraint`, `NotADimension` | 4.8, 8.4, 9.9, 10.2 |
| Expressions: `swx_core::expr` `parse`, `eval`, `Expr` (`names`), `Quantity` (`in_unit`), `Dim`, `Unit` (mm, cm, m, in, ft, deg, rad; `from_suffix`, `factor`, `dim`), `ExprError` kinds; grammar and limits | 4.8, 8.5, 9.10, 10.2 |
| Units (mm; degrees for feature parameters, radians for arcs) | 4.2, 9.10 |

## Examples the manual runs (`crates/*/examples/`)

| Example | Chapter |
|---|---|
| `swx-sketch` `plate_sketch` | 5 |
| `swx-sketch` `sketch_dof` | 5.3 |
| `swx-qa` `plate_sketch_plot` | 4.3–4.5, 5.1, 7.5 figures |
| `swx-kernel` `plate_with_hole` (occt) | 3, 6 |
| `swx-kernel` `plate_stages` (occt) | 4.9 figure |
| `swx-sketch` `centered_plate` | 5.5, 5.7 |
| `swx-sketch` `slot_sketch` | 5.6, 5.7 |
| `swx-qa` `constraint_plots` | 4.7, 5.5, 5.6 figures |
| `swx-core` `feature_history` | 8.1, 8.2 |
| `swx-core` `parameter_table` | 8.3 |
| `swx-sketch` `driven_plate` | 8.4 |
| `swx-core` `expressions` | 8.5 |

## Backlog

- Nothing built is undocumented. Next: when M2-08b (#79) lets a parameter hold an expression, update 4.8, 8.3–8.5, 9.9 and the parameter limitation in 10.4, and drop *Parameters as formulas* from Coming next.
- Possible addition: a rounded-corner task (arc ends joined to two lines, M2-04a), once a part feature can use it.
