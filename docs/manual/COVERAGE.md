manual-synced-to: e04eaf4ad675a928472336b4e9dd85e3accfebdc

# Manual coverage

Everything a user can do or set on `main`, found from the code, `docs/ROADMAP.md` and `docs/specs/PRODUCT.md`, with the manual section that covers it. Section numbers refer to the built PDF.

## Covered

| Capability (crate) | Where |
|---|---|
| Install from source; build with and without `--features occt` | 2, 10.3 |
| `swx-qa render`: `--fixture` (cube, cylinder, l-bracket), `--step`, `-o/--output`, `--size` (default 800x600, max 8192), `--crease` (default 30, 0–180), `-h/--help`, `help` | 3.1, 8, 10.1 |
| `swx-qa sketch`: `--fixture` (rectangle, triangle-345), `-o/--output`, `--size` (default 800x600, max 8192), `-h/--help`; plot conventions | 8.5, 10.2 |
| `swx-qa` success line, exit status, every error message (render and sketch) | 10.1, 10.2, 11.2 |
| `swx_qa::sketch::plot` (developers) | 8.5 |
| Kernel: `make_box`, `make_cylinder`, `make_sphere` | 6.1, 10.4 |
| Kernel: `union`, `cut`, `intersect` | 4.9, 6.2, 10.4 |
| Kernel: `volume`, `surface_area`, `bounding_box`, `face_count` | 3.2, 6.1, 10.4 |
| Kernel: `tessellate` (deflection) | 8.1, 10.4 |
| Kernel: `export_step`, `import_step` (mm) | 3.2, 6.1, 6.3, 10.4, 10.13 |
| Kernel: `make_face`, `extrude`; `Workplane` (`XY`, `XZ`, `YZ`, custom), `Segment2` (Line, Arc, Circle), `ProfileLoop`, `Extent` (Distance, Symmetric), `LOOP_GAP_TOLERANCE`, `PERPENDICULAR_TOLERANCE` | 4.7, 4.9, 7, 10.5 |
| Kernel errors (`InvalidInput`, `Backend`), incl. every `make_face` and `extrude` message | 7.3, 11.2 |
| Sketch entities: point, line, circle, arc (solving changes arc angles; sweep kept in (0, 2π]) | 4.3, 10.6 |
| `PointRef`: Point, Start, End, Center, ArcStart, ArcEnd (arc ends in every point constraint) | 5.1, 5.6, 10.6 |
| Constraints: coincident, horizontal, horizontal points, vertical, vertical points, fixed, distance, radius, diameter | 4.4, 4.5, 5, 10.6 |
| Constraints: parallel, perpendicular, equal (lines; circles/arcs), midpoint, symmetric, tangent (line–circle/arc, circle/arc–circle/arc; side and internal/external rules; endpoint form when a line end is coincident with an arc end) | 4.4, 5.5, 5.6, 10.6 |
| Profiles: `profiles`, `profile_report`, `ProfileOptions` (weld_tolerance), `ProfileReport` (regions, open_edges), `Region` (outer, holes, `area()`), `Loop` (edges, signed_area), `OrientedEdge`, nesting rules | 4.7, 5.7, 10.8 |
| Profile errors (`InvalidTolerance`, `SelfIntersecting`, `Sketch`) | 5.7, 11.2 |
| Solver: `SolveOptions` (tolerance, max_iterations, initial_damping), report, stop reasons | 5.1, 10.7 |
| DOF analysis: `dof::analyze`, `DofOptions` (rank_tolerance, residual_tolerance), `DofReport` (dof, rank, redundant, residual_norm, state), states incl. `NonFinite`; equations per constraint | 4.6, 5.3, 10.9 |
| DOF by constraint: `Sketch::dof`, `SketchDofReport` (report, redundant_constraints), `Sketch::constraint_of_row` | 4.6, 5.3, 10.9 |
| Sketch errors | 5.4, 11.2 |
| Document model: parameters, features (sketch, extrude, revolve), history, dirty propagation | 4.8, 4.10, 9 |
| Parameter table: `add_model_param` (d0, d1, …), `add_user_param`, `rename_param` (features and formulas follow), `set_param_comment`, `parameters`, `is_valid_param_name`, `Parameter` (name, kind, value, unit, comment, expr), `ParamKind` | 4.8, 9.3, 10.11 |
| Parameter formulas: `set_param_expr`, `evaluation_order` (dependency order), `set_param` drops a formula and re-evaluates dependents (old values kept on failure), unit handling (result in the parameter's unit; unitless taken in it), `dirty_after_param_change` through formulas | 4.8, 9.6, 10.11, 11.1, 11.4 |
| Formula errors: `ModelError::Expr`, `ParameterCycle`, `ParameterDimension` | 9.6, 11.2 |
| Inference: `swx_sketch::infer` `infer`, `add_inferred`, `InferOptions` (point_tol, angle_tol_deg, `for_sketch`), suggestion order and rules, `InferError` (Sketch, Solve, NotConverged) | 4.4, 5.8, 10.10, 11.1, 11.2 |
| Document model errors (incl. `DuplicateParameter`, `InvalidParameterName`) | 9.4, 11.2 |
| Sketch dimensions bound to parameters: `swx_sketch::params` `bind_dimension`, `apply_params`, `rename_bound`, `ParamError`; `Sketch::bindings`, `Sketch::set_dimension_value`; `SketchError::UnknownConstraint`, `NotADimension` | 4.8, 9.4, 10.11, 11.2 |
| Expressions: `swx_core::expr` `parse`, `eval`, `Expr` (`names`), `Quantity` (`in_unit`), `Dim`, `Unit` (mm, cm, m, in, ft, deg, rad; `from_suffix`, `factor`, `dim`), `ExprError` kinds; grammar and limits | 4.8, 9.5, 10.12, 11.2 |
| Units (mm; degrees for feature parameters, radians for arcs) | 4.2, 10.13 |

## Examples the manual runs (`crates/*/examples/`)

| Example | Chapter |
|---|---|
| `swx-sketch` `plate_sketch` | 5 |
| `swx-sketch` `sketch_dof` | 5.3 |
| `swx-qa` `plate_sketch_plot` | 4.3–4.5, 5.1, 8.5 figures |
| `swx-kernel` `plate_with_hole` (occt) | 3, 6 |
| `swx-kernel` `plate_stages` (occt) | 4.9 figure |
| `swx-sketch` `centered_plate` | 5.5, 5.7 |
| `swx-sketch` `slot_sketch` | 5.6, 5.7 |
| `swx-qa` `constraint_plots` | 4.7, 5.5, 5.6 figures |
| `swx-core` `feature_history` | 9.1, 9.2 |
| `swx-core` `parameter_table` | 9.3 |
| `swx-sketch` `driven_plate` | 9.4 |
| `swx-core` `expressions` | 9.5 |
| `swx-core` `parameter_formulas` | 9.6 |
| `swx-sketch` `inferred_sketch` | 5.8 |
| `swx-qa` `inference_plots` | 5.8 figure |
| `swx-kernel` `extrude_plate` (occt) | 7.1 |
| `swx-kernel` `extrude_slot` (occt) | 7.2 |

## Backlog

- Nothing built is undocumented. Next: when M3-01b (#91) lands, add a task for the join, cut and intersect modes to chapter 7, drop *Extrude modes* from Coming next, and update the known limitations.
- Possible addition: a rounded-corner task (arc ends joined to two lines, M2-04a), now that an extrude can use arcs: a plate with rounded corners.
