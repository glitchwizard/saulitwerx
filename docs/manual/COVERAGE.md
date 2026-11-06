manual-synced-to: 5c7abad180caf8e37e78e2252efae1da78e86489

# Manual coverage

Everything a user can do or set on `main`, found from the code, `docs/ROADMAP.md` and `docs/specs/PRODUCT.md`, with the manual section that covers it. Section numbers refer to the built PDF.

## Covered

| Capability (crate) | Where |
|---|---|
| Install from source; build with and without `--features occt` | 2, 12.3 |
| `swx-qa render`: `--fixture` (cube, cylinder, l-bracket), `--step`, `-o/--output`, `--size` (default 800x600, max 8192), `--crease` (default 30, 0–180), `-h/--help`, `help` | 3.1, 10, 12.1 |
| `swx-qa sketch`: `--fixture` (rectangle, triangle-345), `-o/--output`, `--size` (default 800x600, max 8192), `-h/--help`; plot conventions | 10.5, 12.2 |
| `swx-qa` success line, exit status, every error message (render and sketch) | 12.1, 12.2, 13.2 |
| `swx_qa::sketch::plot` (developers) | 10.5 |
| Kernel: `make_box`, `make_cylinder`, `make_sphere` | 6.1, 12.4 |
| Kernel: `union`, `cut`, `intersect` | 4.9, 6.2, 12.4 |
| Kernel: `boolean_tracked`, `BooleanOp` (Union, Cut, Intersect), `OpHistory` (modified, generated), `Operand` (A, B); faces only, `generated` empty | 6.4, 12.4 |
| Kernel: `volume`, `surface_area`, `bounding_box`, `face_count` | 3.2, 6.1, 12.4 |
| Kernel: `sub_shape_count`, `adjacent`, `TopoKind` (Face, Edge, Vertex), `SubShape` (kind, index); index rules and both `adjacent` errors; seam edge | 4.1, 6.3, 12.4, 13.2, 13.4 |
| Kernel: `tessellate` (deflection) | 10.1, 12.4 |
| Kernel: `export_step`, `import_step` (mm) | 3.2, 6.1, 6.5, 12.4, 12.13 |
| Kernel: `make_face`, `extrude`; `Workplane` (`XY`, `XZ`, `YZ`, custom), `Segment2` (Line, Arc, Circle), `ProfileLoop`, `Extent` (Distance, Symmetric), `LOOP_GAP_TOLERANCE`, `PERPENDICULAR_TOLERANCE` | 4.7, 4.9, 7, 12.5 |
| Kernel: `make_face_tracked` (segment → edge, caller's order), `extrude_tracked`, `ExtrudeHistory` (start_cap, end_cap, sides); through-all refused | 7.5, 7.7, 12.5, 13.2 |
| Kernel: `sub_shape_bounding_box` | 7.5, 12.4 |
| Kernel: `face_plane` (point and unit outward normal of a planar face; kind, range and not-planar errors) | 7.6, 12.4, 13.2 |
| Kernel: `Extent::ToFace`, `Extent::BetweenFaces` (planar faces parallel to the workplane, either side; join, cut and intersect only; refused by `extrude` and `extrude_tracked`), `Extent::FromTo`, `PARALLEL_FACE_TOLERANCE`, `FACE_DISTANCE_TOLERANCE`; every message | 7.6, 7.7, 12.5, 13.2, 13.4 |
| Kernel: `fillet` (constant radius), `chamfer` (equal distance); edge-list checks (empty, not an edge, out of range), duplicates once, too-large size fails and keeps the input; every message | 9, 12.4, 13.2, 13.4 |
| Kernel: `chamfer_asymmetric`, `EdgeOnFace` (edge, reference face), `ChamferSize` (TwoDistances d1/d2, DistanceAngle distance/angle in radians, (0, π/2)); face-bounds-edge and one-face-per-edge checks; every message | 9.2, 9.3, 12.4, 13.2 |
| Kernel: `extrude_feature`, `ExtrudeMode` (NewSolid, Join, Cut, Intersect), `EMPTY_VOLUME_TOLERANCE` | 7.3, 12.5 |
| Kernel: `Extent::ThroughAll`, `ThroughDirection` (Normal, Reversed, Both); through-all only with `extrude_feature` and `Cut`; miss check | 7.4, 7.7, 12.5, 13.2 |
| Kernel: `revolve`, `revolve_feature`, `Axis` (origin, dir); angle in radians, (0, 2π]; right-hand rule; modes as extrude; axis-in-plane and profile-crossing checks, `REVOLVE_AXIS_TOLERANCE` | 8, 12.5, 12.13, 13.2, 13.4 |
| Kernel errors (`InvalidInput`, `Backend`, `EmptyResult`), incl. every `make_face`, `extrude`, `extrude_feature`, through-all, `revolve` and `revolve_feature` message | 7.3, 7.7, 8.3, 13.2 |
| Sketch entities: point, line, circle, arc (solving changes arc angles; sweep kept in (0, 2π]) | 4.3, 12.6 |
| `PointRef`: Point, Start, End, Center, ArcStart, ArcEnd (arc ends in every point constraint) | 5.1, 5.6, 12.6 |
| Constraints: coincident, horizontal, horizontal points, vertical, vertical points, fixed, distance, radius, diameter | 4.4, 4.5, 5, 12.6 |
| Constraints: parallel, perpendicular, equal (lines; circles/arcs), midpoint, symmetric, tangent (line–circle/arc, circle/arc–circle/arc; side and internal/external rules; endpoint form when a line end is coincident with an arc end) | 4.4, 5.5, 5.6, 12.6 |
| Profiles: `profiles`, `profile_report`, `ProfileOptions` (weld_tolerance), `ProfileReport` (regions, open_edges), `Region` (outer, holes, `area()`), `Loop` (edges, signed_area), `OrientedEdge`, nesting rules | 4.7, 5.7, 12.8 |
| Profile errors (`InvalidTolerance`, `SelfIntersecting`, `Sketch`) | 5.7, 13.2 |
| Solver: `SolveOptions` (tolerance, max_iterations, initial_damping), report, stop reasons | 5.1, 12.7 |
| DOF analysis: `dof::analyze`, `DofOptions` (rank_tolerance, residual_tolerance), `DofReport` (dof, rank, redundant, residual_norm, state), states incl. `NonFinite`; equations per constraint | 4.6, 5.3, 12.9 |
| DOF by constraint: `Sketch::dof`, `SketchDofReport` (report, redundant_constraints), `Sketch::constraint_of_row` | 4.6, 5.3, 12.9 |
| Sketch errors | 5.4, 13.2 |
| Document model: parameters, features (sketch, extrude, revolve), history, dirty propagation | 4.8, 4.10, 11 |
| Parameter table: `add_model_param` (d0, d1, …), `add_user_param`, `rename_param` (features and formulas follow), `set_param_comment`, `parameters`, `is_valid_param_name`, `Parameter` (name, kind, value, unit, comment, expr), `ParamKind` | 4.8, 11.3, 12.11 |
| Parameter formulas: `set_param_expr`, `evaluation_order` (dependency order), `set_param` drops a formula and re-evaluates dependents (old values kept on failure), unit handling (result in the parameter's unit; unitless taken in it), `dirty_after_param_change` through formulas | 4.8, 11.6, 12.11, 13.1, 13.4 |
| Formula errors: `ModelError::Expr`, `ParameterCycle`, `ParameterDimension` | 11.6, 13.2 |
| Inference: `swx_sketch::infer` `infer`, `add_inferred`, `InferOptions` (point_tol, angle_tol_deg, `for_sketch`), suggestion order and rules, `InferError` (Sketch, Solve, NotConverged) | 4.4, 5.8, 12.10, 13.1, 13.2 |
| Document model errors (incl. `DuplicateParameter`, `InvalidParameterName`) | 11.4, 13.2 |
| Sketch dimensions bound to parameters: `swx_sketch::params` `bind_dimension`, `apply_params`, `rename_bound`, `ParamError`; `Sketch::bindings`, `Sketch::set_dimension_value`; `SketchError::UnknownConstraint`, `NotADimension` | 4.8, 11.4, 12.11, 13.2 |
| Expressions: `swx_core::expr` `parse`, `eval`, `Expr` (`names`), `Quantity` (`in_unit`), `Dim`, `Unit` (mm, cm, m, in, ft, deg, rad; `from_suffix`, `factor`, `dim`), `ExprError` kinds; grammar and limits | 4.8, 11.5, 12.12, 13.2 |
| Units (mm; degrees for feature parameters, radians for arcs and kernel revolves) | 4.2, 12.13 |

## Examples the manual runs (`crates/*/examples/`)

| Example | Chapter |
|---|---|
| `swx-sketch` `plate_sketch` | 5 |
| `swx-sketch` `sketch_dof` | 5.3 |
| `swx-qa` `plate_sketch_plot` | 4.3–4.5, 5.1, 10.5 figures |
| `swx-kernel` `plate_with_hole` (occt) | 3, 6 |
| `swx-kernel` `plate_stages` (occt) | 4.9 figure |
| `swx-sketch` `centered_plate` | 5.5, 5.7 |
| `swx-sketch` `slot_sketch` | 5.6, 5.7 |
| `swx-qa` `constraint_plots` | 4.7, 5.5, 5.6 figures |
| `swx-core` `feature_history` | 11.1, 11.2 |
| `swx-core` `parameter_table` | 11.3 |
| `swx-sketch` `driven_plate` | 11.4 |
| `swx-core` `expressions` | 11.5 |
| `swx-core` `parameter_formulas` | 11.6 |
| `swx-sketch` `inferred_sketch` | 5.8 |
| `swx-qa` `inference_plots` | 5.8 figure |
| `swx-kernel` `extrude_plate` (occt) | 7.1 |
| `swx-kernel` `extrude_slot` (occt) | 7.2 |
| `swx-kernel` `extrude_modes` (occt) | 7.3 |
| `swx-kernel` `extrude_through_all` (occt) | 7.4 |
| `swx-kernel` `revolve_bushing` (occt) | 8 |
| `swx-kernel` `part_topology` (occt) | 6.3 |
| `swx-kernel` `extrude_to_face` (occt) | 7.6 |
| `swx-kernel` `fillet_chamfer` (occt) | 9.1 |
| `swx-kernel` `chamfer_sizes` (occt) | 9.2 |
| `swx-kernel` `boolean_history` (occt) | 6.4 |

## Not user-facing yet

- `swx_core::topo` (M3-04e, #103): `EntityRef`, `Role`, `FaceName`, `EdgeName`, `TopoRef`, `ResolveError`. Plain data with no way to resolve a name to a face or an edge yet, so nothing a user can do changes. M3-04b (#116) added the sub-shape queries it will resolve to (documented in 6.3), and M3-04c/d the face histories of extrudes and Booleans (7.5, 6.4), but not the resolving itself. Document it, with its `describe()` wording, when references can be resolved (later M3-04 slices); *Coming next* lists it as *Lasting references*.

## Backlog

- Nothing built is undocumented. Next: when sketch regions reach the extrude and the revolve (M3-01c, #106), rewrite chapters 7 and 8 so their tasks start from a sketch, and update the known limitations. When fillets and chamfers take lasting edge references or join the history (later M3-05 slices), rewrite chapter 9's tasks around them. When extrudes and revolves with join, cut or intersect report a face history built on `boolean_tracked`, add it to 7.5 and 6.4.
- Possible addition: a rounded-corner task (arc ends joined to two lines, M2-04a), now that an extrude can use arcs: a plate with rounded corners.
