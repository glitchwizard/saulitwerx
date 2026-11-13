manual-synced-to: f72947756d5489deed0505031c2482cfa28c7342

# Manual coverage

Everything a user can do or set on `main`, found from the code, `docs/ROADMAP.md` and `docs/specs/PRODUCT.md`, with the manual section that covers it. Section numbers refer to the built PDF.

## Covered

| Capability (crate) | Where |
|---|---|
| Install from source; build with and without `--features occt` | 2, 19.3 |
| `swx-qa render`: `--fixture` (cube, cylinder, l-bracket), `--step`, `-o/--output`, `--size` (default 800x600, max 8192), `--crease` (default 30, 0–180), `-h/--help`, `help` | 3.1, 16, 19.1 |
| `swx-qa sketch`: `--fixture` (rectangle, triangle-345), `-o/--output`, `--size` (default 800x600, max 8192), `-h/--help`; plot conventions | 16.5, 19.2 |
| `swx-qa` success line, exit status, every error message (render and sketch) | 19.1, 19.2, 20.2 |
| `swx_qa::sketch::plot` (developers) | 16.5 |
| Kernel: `make_box`, `make_cylinder`, `make_sphere` | 6.1, 19.4 |
| Kernel: `union`, `cut`, `intersect` | 4.9, 6.2, 19.4 |
| Kernel: `boolean_tracked`, `BooleanOp` (Union, Cut, Intersect), `OpHistory` (modified, generated), `Operand` (A, B); faces only, `generated` empty | 6.4, 19.4 |
| Kernel: `volume`, `surface_area`, `bounding_box`, `face_count` | 3.2, 6.1, 19.4 |
| Kernel: `sub_shape_count`, `adjacent`, `TopoKind` (Face, Edge, Vertex), `SubShape` (kind, index); index rules and both `adjacent` errors; seam edge | 4.1, 6.3, 19.4, 20.2, 20.4 |
| Kernel: `tessellate` (deflection) | 16.1, 19.4 |
| Kernel: `export_step`, `import_step` (mm) | 3.2, 6.1, 6.5, 19.4, 19.16 |
| Kernel: `make_face`, `extrude`; `Workplane` (`XY`, `XZ`, `YZ`, custom), `Segment2` (Line, Arc, Circle), `ProfileLoop`, `Extent` (Distance, Symmetric), `LOOP_GAP_TOLERANCE`, `PERPENDICULAR_TOLERANCE` | 4.7, 4.9, 7, 19.6 |
| Kernel: `make_face_tracked` (segment → edge, caller's order), `extrude_tracked`, `ExtrudeHistory` (start_cap, end_cap, sides); through-all refused | 7.6, 7.8, 19.6, 20.2 |
| Kernel: `sub_shape_bounding_box` | 7.6, 19.4 |
| Kernel: `face_plane` (point and unit outward normal of a planar face; kind, range and not-planar errors) | 7.7, 19.4, 20.2 |
| Kernel: `Extent::ToFace`, `Extent::BetweenFaces` (planar faces parallel to the workplane, either side; join, cut and intersect only; refused by `extrude` and `extrude_tracked`), `Extent::FromTo`, `PARALLEL_FACE_TOLERANCE`, `FACE_DISTANCE_TOLERANCE`; every message | 7.7, 7.8, 19.6, 20.2, 20.4 |
| Kernel: `fillet` (constant radius), `chamfer` (equal distance); edge-list checks (empty, not an edge, out of range), duplicates once, too-large size fails and keeps the input; every message | 9, 19.4, 20.2, 20.4 |
| Kernel: `chamfer_asymmetric`, `EdgeOnFace` (edge, reference face), `ChamferSize` (TwoDistances d1/d2, DistanceAngle distance/angle in radians, (0, π/2)); face-bounds-edge and one-face-per-edge checks; every message | 9.2, 9.4, 19.4, 20.2 |
| Kernel: `shell` (one inward wall thickness, outside size kept; open-face list: not empty, faces only, in range, duplicates once; thickness > 0; too thick fails and keeps the input); every message | 10, 19.4, 20.2, 20.4 |
| Kernel: `fillet_variable` (one edge, start vertex, radius at each end, linear change; edge/vertex kind, range and on-edge checks; larger radius must fit as a constant fillet; tangent chains refused); every message | 9.3, 9.4, 19.4, 20.2, 20.4 |
| Kernel: `hole`, `HoleKind` (Simple, Counterbore: diameter, cb_diameter, cb_depth; Countersink: diameter, csk_diameter, csk_angle in radians (0, π), cone depth rule; Tapped: thread, cut at the tap-drill diameter, thread not modeled), `HoleDepth` (Blind, ThroughAll); flat bottoms; any direction; miss, empty-result and invalid-result checks; every message (countersink depth rounded to 6 places, angle also in degrees, M3-12d) | 11, 11.2, 19.4, 20.2, 20.4 |
| Kernel: `MetricThread` (nominal, pitch, `designation()`, `minor_diameter()`, `tap_drill_diameter()`), `METRIC_COARSE` (M2 to M24), `metric_coarse` (name lookup, case, whitespace); every message | 11.2, 19.5, 20.2 |
| Kernel: `draft` (faces, neutral plane as an `Axis`: point and pull direction, angle in radians, sign rule, (0, π/2) in size; face-list checks; faces parallel to the neutral plane fail); every message | 12, 19.4, 20.2, 20.4 |
| Work features: `ORIGIN`, `Axis::X`/`Y`/`Z`, `Workplane::offset`, `through_points`, `from_point_normal`, `angled`, `Axis::through_points`, `Axis::plane_intersection`, `plane_axis_intersection`, `DATUM_TOLERANCE`; every message | 15, 19.7, 20.2, 20.4 |
| Kernel: `loft` (sections in order, ruled or smooth; `MIN_LOFT_SECTIONS`; single planar faces without holes, any plane; coincident sections fail; empty result); every message | 13, 19.4, 20.2, 20.4 |
| Kernel: `sweep` (planar profile without holes, open path of lines and arcs on a workplane, start on the profile's plane, segments either way round within `LOOP_GAP_TOLERANCE`, no circles; path along the profile's plane is an empty result; an arc no wider than the profile's reach from the path's start is refused); every message | 14, 19.4, 20.2, 20.4 |
| Kernel: `helical_sweep` (planar profile without holes, `Axis` in the profile's plane, profile wholly on one side of it; pitch and turns finite and > 0, turns may be fractional; right-handed about `axis.dir`; extent along the axis < pitch; volume independent of the pitch); every message | 14.2, 14.3, 19.4, 20.2, 20.4 |
| Work features from geometry: `work_plane_on_face`, `work_axis_on` (straight edge or cylindrical face; either direction), `vertex_point`; origin nearest the part origin; every message | 15.3, 19.7, 20.2, 20.4 |
| Work features between and around faces: `Workplane::midplane` (parallel planes, keeps `a`'s normal), `work_plane_tangent` (cylindrical face, side from `toward`), `edge_midpoint` (half the arc length); every message | 15.4, 19.7, 20.2 |
| Kernel: multi-body shapes: `body_count`, `bodies` (ordered by bounding-box minimum), `combine` (target with several tools, in order; tool, solid and empty checks), `combine_tracked`, `CombineHistory` (target, tools); every message | 6.5, 19.4, 20.2, 20.4 |
| Kernel: `extrude_feature`, `ExtrudeMode` (NewSolid, Join, Cut, Intersect), `EMPTY_VOLUME_TOLERANCE` | 7.4, 19.6 |
| Kernel: `Extent::ThroughAll`, `ThroughDirection` (Normal, Reversed, Both); through-all only with `extrude_feature` and `Cut`; miss check | 7.5, 7.8, 19.6, 20.2 |
| Kernel: `revolve`, `revolve_feature`, `Axis` (origin, dir); angle in radians, (0, 2π]; right-hand rule; modes as extrude; axis-in-plane and profile-crossing checks, `REVOLVE_AXIS_TOLERANCE` | 8, 19.6, 19.16, 20.2, 20.4 |
| Kernel errors (`InvalidInput`, `Backend`, `EmptyResult`), incl. every `make_face`, `extrude`, `extrude_feature`, through-all, `revolve` and `revolve_feature` message | 7.4, 7.8, 8.3, 20.2 |
| Sketch entities: point, line, circle, arc (solving changes arc angles; sweep kept in (0, 2π]) | 4.3, 19.8 |
| `PointRef`: Point, Start, End, Center, ArcStart, ArcEnd (arc ends in every point constraint) | 5.1, 5.6, 19.8 |
| Constraints: coincident, horizontal, horizontal points, vertical, vertical points, fixed, distance, radius, diameter | 4.4, 4.5, 5, 19.8 |
| Constraints: parallel, perpendicular, equal (lines; circles/arcs), midpoint, symmetric, tangent (line–circle/arc, circle/arc–circle/arc; side and internal/external rules; endpoint form when a line end is coincident with an arc end) | 4.4, 5.5, 5.6, 5.8, 19.8 |
| Profiles: `profiles`, `profile_report`, `ProfileOptions` (weld_tolerance), `ProfileReport` (regions, open_edges), `Region` (outer, holes, `area()`), `Loop` (edges, signed_area), `OrientedEdge`, nesting rules | 4.7, 5.7, 19.10 |
| Profile errors (`InvalidTolerance`, `SelfIntersecting`, `Sketch`) | 5.7, 20.2 |
| Solver: `SolveOptions` (tolerance, max_iterations, initial_damping), report, stop reasons | 5.1, 19.9 |
| DOF analysis: `dof::analyze`, `DofOptions` (rank_tolerance, residual_tolerance), `DofReport` (dof, rank, redundant, residual_norm, state), states incl. `NonFinite`; equations per constraint | 4.6, 5.3, 19.11 |
| DOF by constraint: `Sketch::dof`, `SketchDofReport` (report, redundant_constraints), `Sketch::constraint_of_row` | 4.6, 5.3, 19.11 |
| Sketch errors | 5.4, 20.2 |
| Document model: parameters, features (sketch, extrude, revolve), history, dirty propagation | 4.8, 4.10, 17, 19.13 |
| End-of-part marker: `end_of_part`, `set_end_of_part` (returns the features to rebuild), `active_features`, `add_feature` inserts at the marker, rolled-back features never dirty, `FeatureId` vs history place, `ModelError::MarkerOutOfRange`, forward reference to a rolled-back feature | 4.10, 17.3, 17.8, 19.13, 20.2 |
| Parameter table: `add_model_param` (d0, d1, …), `add_user_param`, `rename_param` (features and formulas follow), `set_param_comment`, `parameters`, `is_valid_param_name`, `Parameter` (name, kind, value, unit, comment, expr), `ParamKind` | 4.8, 17.4, 19.14 |
| Parameter formulas: `set_param_expr`, `evaluation_order` (dependency order), `set_param` drops a formula and re-evaluates dependents (old values kept on failure), unit handling (result in the parameter's unit; unitless taken in it), `dirty_after_param_change` through formulas | 4.8, 17.7, 19.14, 20.1, 20.4 |
| Formula errors: `ModelError::Expr`, `ParameterCycle`, `ParameterDimension` | 17.7, 20.2 |
| Inference: `swx_sketch::infer` `infer`, `add_inferred`, `InferOptions` (point_tol, angle_tol_deg, `for_sketch`), suggestion order and rules, `InferError` (Sketch, Solve, NotConverged) | 4.4, 5.9, 19.12, 20.1, 20.2 |
| Document model errors (incl. `DuplicateParameter`, `InvalidParameterName`) | 17.5, 20.2 |
| Sketch dimensions bound to parameters: `swx_sketch::params` `bind_dimension`, `apply_params`, `rename_bound`, `ParamError`; `Sketch::bindings`, `Sketch::set_dimension_value`; `SketchError::UnknownConstraint`, `NotADimension` | 4.8, 17.5, 19.14, 20.2 |
| Expressions: `swx_core::expr` `parse`, `eval`, `Expr` (`names`), `Quantity` (`in_unit`), `Dim`, `Unit` (mm, cm, m, in, ft, deg, rad; `from_suffix`, `factor`, `dim`), `ExprError` kinds; grammar and limits | 4.8, 17.6, 19.15, 20.2 |
| Document files (`swx-io`): `write_unpacked`, `read_unpacked`, `Document` (manifest, model, properties), `read_manifest`, `write_manifest`, `to_canonical_json`, `Manifest` (every field, `extra`), `Units` (`extra`), `DocumentKind` (`as_str`, extensions), `FormatError` (every kind and message); folder-name check, properties removal, unknown entries left alone | 18, 19.17, 20.2, 20.3, 20.4 |
| Units (mm; degrees for feature parameters, radians for arcs and kernel revolves) | 4.2, 19.16 |

## Examples the manual runs (`crates/*/examples/`)

| Example | Chapter |
|---|---|
| `swx-sketch` `plate_sketch` | 5 |
| `swx-sketch` `sketch_dof` | 5.3 |
| `swx-qa` `plate_sketch_plot` | 4.3–4.5, 5.1, 16.5 figures |
| `swx-kernel` `plate_with_hole` (occt) | 3, 6 |
| `swx-kernel` `plate_stages` (occt) | 4.9 figure |
| `swx-sketch` `centered_plate` | 5.5, 5.7 |
| `swx-sketch` `slot_sketch` | 5.6, 5.7 |
| `swx-sketch` `rounded_plate` | 5.8 |
| `swx-qa` `constraint_plots` | 4.7, 5.5, 5.6, 5.8 figures |
| `swx-core` `feature_history` | 17.1, 17.2 |
| `swx-core` `parameter_table` | 17.4 |
| `swx-sketch` `driven_plate` | 17.5 |
| `swx-core` `expressions` | 17.6 |
| `swx-core` `parameter_formulas` | 17.7 |
| `swx-sketch` `inferred_sketch` | 5.9 |
| `swx-qa` `inference_plots` | 5.9 figure |
| `swx-kernel` `extrude_plate` (occt) | 7.1 |
| `swx-kernel` `extrude_slot` (occt) | 7.2 |
| `swx-kernel` `extrude_rounded_plate` (occt) | 7.3 |
| `swx-kernel` `extrude_modes` (occt) | 7.4 |
| `swx-kernel` `extrude_through_all` (occt) | 7.5 |
| `swx-kernel` `revolve_bushing` (occt) | 8 |
| `swx-kernel` `part_topology` (occt) | 6.3 |
| `swx-kernel` `extrude_to_face` (occt) | 7.7 |
| `swx-kernel` `fillet_chamfer` (occt) | 9.1 |
| `swx-kernel` `chamfer_sizes` (occt) | 9.2 |
| `swx-kernel` `boolean_history` (occt) | 6.4 |
| `swx-kernel` `shell_parts` (occt) | 10 |
| `swx-core` `roll_back` | 17.3 |
| `swx-kernel` `fillet_variable` (occt) | 9.3 |
| `swx-kernel` `hole_plate` (occt) | 11 |
| `swx-kernel` `hole_screw` (occt) | 11.2 |
| `swx-kernel` `thread_table` | 11.2 |
| `swx-kernel` `draft_block` (occt) | 12 |
| `swx-kernel` `work_features` | 15.1 |
| `swx-kernel` `tilted_post` (occt) | 15.2 |
| `swx-kernel` `loft_sections` (occt) | 13 |
| `swx-kernel` `sweep_pipe` (occt) | 14 |
| `swx-kernel` `work_on_part` (occt) | 15.3 |
| `swx-kernel` `work_midplane` (occt) | 15.4 |
| `swx-kernel` `multi_body` (occt) | 6.5 |
| `swx-kernel` `helical_spring` (occt) | 14.2 |
| `swx-io` `unpacked_document` | 18 |

## Not user-facing yet

- `swx_core::topo` (M3-04e, #103): `EntityRef`, `Role`, `FaceName`, `EdgeName`, `TopoRef`, `ResolveError`. Plain data with no way to resolve a name to a face or an edge yet, so nothing a user can do changes. M3-04b (#116) added the sub-shape queries it will resolve to (documented in 6.3), and M3-04c/d the face histories of extrudes and Booleans (7.6, 6.4), but not the resolving itself. Document it, with its `describe()` wording, when references can be resolved (later M3-04 slices); *Coming next* lists it as *Lasting references*.

## Backlog

- Nothing built is undocumented. Next: when shells take walls of different thicknesses or join the history (later M3-06 slices), extend chapter 10. When sketch regions reach the extrude and the revolve (M3-01c, #106), rewrite chapters 7, 8, 13 and 14 so their tasks start from a sketch, and update the known limitations. When fillets and chamfers take lasting edge references or join the history (later M3-05 slices), rewrite chapter 9's tasks around them. When extrudes and revolves with join, cut or intersect report a face history built on `boolean_tracked`, add it to 7.6 and 6.4. When holes gain drill points, other thread series, or modeled threads, or join the history, extend chapter 11. When work features can be picked on a part, use lasting references, or join the history (later M3-08 slices), extend chapter 15. When bodies can be named, moved or kept in a part's history, extend 6.5. When lofts gain guide curves or sections with holes, or sweeps gain closed or 3D paths, twist or guide rails (later M3-11 slices), extend chapters 13 and 14. When helical sweeps gain a varying radius or pitch, a left hand, or modeled threads arrive (later MB-03 slices), extend 14.2 and chapter 11. When `model.json` gets its typed schema, a part can be saved and opened, the ZIP form or `sltwx pack`/`unpack` arrive, or the cache and `model_hash` check (M4-01, M4-04, M4-05), extend chapter 18, and move the first-part tutorial's save step onto it. The sweep's bend check (M3-11c) is documented in 14.
- Possible improvement: the variable-fillet picture (9.3) and the square-to-round loft (13) show faint tessellation ripples on curved faces, because `swx-qa render` has no finer-mesh option. Regenerate them if the renderer gains one.
