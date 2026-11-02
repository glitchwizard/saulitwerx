manual-synced-to: eb68508ea378708e4c1ad035f2f653e38fcaec4f

# Manual coverage

Everything a user can do or set on `main`, found from the code, `docs/ROADMAP.md` and `docs/specs/PRODUCT.md`, with the manual section that covers it. Section numbers refer to the built PDF.

## Covered

| Capability (crate) | Where |
|---|---|
| Install from source; build with and without `--features occt` | 2, 11.3 |
| `swx-qa render`: `--fixture` (cube, cylinder, l-bracket), `--step`, `-o/--output`, `--size` (default 800x600, max 8192), `--crease` (default 30, 0–180), `-h/--help`, `help` | 3.1, 9, 11.1 |
| `swx-qa sketch`: `--fixture` (rectangle, triangle-345), `-o/--output`, `--size` (default 800x600, max 8192), `-h/--help`; plot conventions | 9.5, 11.2 |
| `swx-qa` success line, exit status, every error message (render and sketch) | 11.1, 11.2, 12.2 |
| `swx_qa::sketch::plot` (developers) | 9.5 |
| Kernel: `make_box`, `make_cylinder`, `make_sphere` | 6.1, 11.4 |
| Kernel: `union`, `cut`, `intersect` | 4.9, 6.2, 11.4 |
| Kernel: `volume`, `surface_area`, `bounding_box`, `face_count` | 3.2, 6.1, 11.4 |
| Kernel: `tessellate` (deflection) | 9.1, 11.4 |
| Kernel: `export_step`, `import_step` (mm) | 3.2, 6.1, 6.3, 11.4, 11.13 |
| Kernel: `make_face`, `extrude`; `Workplane` (`XY`, `XZ`, `YZ`, custom), `Segment2` (Line, Arc, Circle), `ProfileLoop`, `Extent` (Distance, Symmetric), `LOOP_GAP_TOLERANCE`, `PERPENDICULAR_TOLERANCE` | 4.7, 4.9, 7, 11.5 |
| Kernel: `extrude_feature`, `ExtrudeMode` (NewSolid, Join, Cut, Intersect), `EMPTY_VOLUME_TOLERANCE` | 7.3, 11.5 |
| Kernel: `Extent::ThroughAll`, `ThroughDirection` (Normal, Reversed, Both); through-all only with `extrude_feature` and `Cut`; miss check | 7.4, 7.5, 11.5, 12.2 |
| Kernel: `revolve`, `revolve_feature`, `Axis` (origin, dir); angle in radians, (0, 2π]; right-hand rule; modes as extrude | 8, 11.5, 11.13, 12.2, 12.4 |
| Kernel errors (`InvalidInput`, `Backend`, `EmptyResult`), incl. every `make_face`, `extrude`, `extrude_feature`, through-all, `revolve` and `revolve_feature` message | 7.3, 7.5, 8.3, 12.2 |
| Sketch entities: point, line, circle, arc (solving changes arc angles; sweep kept in (0, 2π]) | 4.3, 11.6 |
| `PointRef`: Point, Start, End, Center, ArcStart, ArcEnd (arc ends in every point constraint) | 5.1, 5.6, 11.6 |
| Constraints: coincident, horizontal, horizontal points, vertical, vertical points, fixed, distance, radius, diameter | 4.4, 4.5, 5, 11.6 |
| Constraints: parallel, perpendicular, equal (lines; circles/arcs), midpoint, symmetric, tangent (line–circle/arc, circle/arc–circle/arc; side and internal/external rules; endpoint form when a line end is coincident with an arc end) | 4.4, 5.5, 5.6, 11.6 |
| Profiles: `profiles`, `profile_report`, `ProfileOptions` (weld_tolerance), `ProfileReport` (regions, open_edges), `Region` (outer, holes, `area()`), `Loop` (edges, signed_area), `OrientedEdge`, nesting rules | 4.7, 5.7, 11.8 |
| Profile errors (`InvalidTolerance`, `SelfIntersecting`, `Sketch`) | 5.7, 12.2 |
| Solver: `SolveOptions` (tolerance, max_iterations, initial_damping), report, stop reasons | 5.1, 11.7 |
| DOF analysis: `dof::analyze`, `DofOptions` (rank_tolerance, residual_tolerance), `DofReport` (dof, rank, redundant, residual_norm, state), states incl. `NonFinite`; equations per constraint | 4.6, 5.3, 11.9 |
| DOF by constraint: `Sketch::dof`, `SketchDofReport` (report, redundant_constraints), `Sketch::constraint_of_row` | 4.6, 5.3, 11.9 |
| Sketch errors | 5.4, 12.2 |
| Document model: parameters, features (sketch, extrude, revolve), history, dirty propagation | 4.8, 4.10, 10 |
| Parameter table: `add_model_param` (d0, d1, …), `add_user_param`, `rename_param` (features and formulas follow), `set_param_comment`, `parameters`, `is_valid_param_name`, `Parameter` (name, kind, value, unit, comment, expr), `ParamKind` | 4.8, 10.3, 11.11 |
| Parameter formulas: `set_param_expr`, `evaluation_order` (dependency order), `set_param` drops a formula and re-evaluates dependents (old values kept on failure), unit handling (result in the parameter's unit; unitless taken in it), `dirty_after_param_change` through formulas | 4.8, 10.6, 11.11, 12.1, 12.4 |
| Formula errors: `ModelError::Expr`, `ParameterCycle`, `ParameterDimension` | 10.6, 12.2 |
| Inference: `swx_sketch::infer` `infer`, `add_inferred`, `InferOptions` (point_tol, angle_tol_deg, `for_sketch`), suggestion order and rules, `InferError` (Sketch, Solve, NotConverged) | 4.4, 5.8, 11.10, 12.1, 12.2 |
| Document model errors (incl. `DuplicateParameter`, `InvalidParameterName`) | 10.4, 12.2 |
| Sketch dimensions bound to parameters: `swx_sketch::params` `bind_dimension`, `apply_params`, `rename_bound`, `ParamError`; `Sketch::bindings`, `Sketch::set_dimension_value`; `SketchError::UnknownConstraint`, `NotADimension` | 4.8, 10.4, 11.11, 12.2 |
| Expressions: `swx_core::expr` `parse`, `eval`, `Expr` (`names`), `Quantity` (`in_unit`), `Dim`, `Unit` (mm, cm, m, in, ft, deg, rad; `from_suffix`, `factor`, `dim`), `ExprError` kinds; grammar and limits | 4.8, 10.5, 11.12, 12.2 |
| Units (mm; degrees for feature parameters, radians for arcs and kernel revolves) | 4.2, 11.13 |

## Examples the manual runs (`crates/*/examples/`)

| Example | Chapter |
|---|---|
| `swx-sketch` `plate_sketch` | 5 |
| `swx-sketch` `sketch_dof` | 5.3 |
| `swx-qa` `plate_sketch_plot` | 4.3–4.5, 5.1, 9.5 figures |
| `swx-kernel` `plate_with_hole` (occt) | 3, 6 |
| `swx-kernel` `plate_stages` (occt) | 4.9 figure |
| `swx-sketch` `centered_plate` | 5.5, 5.7 |
| `swx-sketch` `slot_sketch` | 5.6, 5.7 |
| `swx-qa` `constraint_plots` | 4.7, 5.5, 5.6 figures |
| `swx-core` `feature_history` | 10.1, 10.2 |
| `swx-core` `parameter_table` | 10.3 |
| `swx-sketch` `driven_plate` | 10.4 |
| `swx-core` `expressions` | 10.5 |
| `swx-core` `parameter_formulas` | 10.6 |
| `swx-sketch` `inferred_sketch` | 5.8 |
| `swx-qa` `inference_plots` | 5.8 figure |
| `swx-kernel` `extrude_plate` (occt) | 7.1 |
| `swx-kernel` `extrude_slot` (occt) | 7.2 |
| `swx-kernel` `extrude_modes` (occt) | 7.3 |
| `swx-kernel` `extrude_through_all` (occt) | 7.4 |
| `swx-kernel` `revolve_bushing` (occt) | 8 |

## Not user-facing yet

- `swx_core::topo` (M3-04e, #103): `EntityRef`, `Role`, `FaceName`, `EdgeName`, `TopoRef`, `ResolveError`. Plain data with no way to resolve a name to a face or an edge yet, so nothing a user can do changes. Document it, with its `describe()` wording, when references can be resolved (M3-04b..d); *Coming next* lists it as *Lasting references*.

## Backlog

- Nothing built is undocumented. Next: when sketch regions reach the extrude and the revolve (M3-01c, #106), rewrite chapters 7 and 8 so their tasks start from a sketch, and update the known limitations.
- When the revolve axis checks land (M3-03b, #117), replace the two axis entries in 8.3 *Common mistakes*, the `kernel failure: revolve failed` row in 12.2, and the revolve limitation in 12.4 with the new messages.
- Possible addition: a rounded-corner task (arc ends joined to two lines, M2-04a), now that an extrude can use arcs: a plate with rounded corners.
