#import "../template.typ": *

= Help <ch-help>

== Troubleshooting

#ref-table(
  columns: (1fr, 1.4fr),
  header: ("Symptom", "What to do"),
  [`cargo: command not found`], [Rust isn't installed, or this terminal started before it was. Open a new terminal; if that doesn't help, repeat the Rust step in @ch-install.],
  [`could not find Cargo.toml`], [The terminal isn't in the `saulitwerx` folder. Run `cd saulitwerx` and try again.],
  [A build with `--features occt` fails with errors about `.hxx` files or `TK…` libraries], [OpenCascade isn't installed. Install the `libocct-…-dev` packages from @ch-install.],
  [`swx-qa: cannot read … STEP input needs OCCT`], [`swx-qa` was built without OpenCascade. Rebuild it: `cargo build --release -p swx-qa --features occt`.],
  [`target/release/swx-qa: No such file or directory`], [The tool isn't built yet. Run the build step in @ch-install.],
  [Rows of `*` and colored text appear when saving or reading STEP], [Normal. The geometry engine prints its own report. Saulitwerx's own lines come before and after it.],
  [The picture looks small or blurry when printed], [Use a larger `--size`: about 80 pixels per centimeter of printed width.],
  [A curved part is covered in lines], [Raise `--crease`, for example to 45.],
  [The numbers in a sketch picture are too small to read], [Use a smaller `--size`. Labels have a fixed size in pixels, so they take up more of a small picture.],
  [A sketch solves, but its sizes aren't what you expect], [Some of it is still free to move. Check the degrees of freedom (@sec-check-dof) and add the missing dimensions.],
  [A sketch looks closed, but has no region], [Some ends don't meet. The open-edge list names the pieces (@sec-profiles). Join every end, line or arc, to its neighbor with coincident (@sec-slot).],
  [An arc flips or bulges the wrong way after solving], [An arc runs counterclockwise from `ArcStart` to `ArcEnd`. Check that each line end is joined to the right end of the arc (@sec-slot).],
  [A sketch dimension ignores a new parameter value], [Copy the values into the sketch with `apply_params`, then solve again (@sec-drive-sketch).],
  [Inference left a corner open], [The ends were farther apart than `point_tol`. Draw the new piece from where the last one ended, or widen `point_tol` (@sec-infer).],
  [Inference made a line level that you meant to lean], [It was within `angle_tol_deg` of level. Lower the setting, or don't add the suggestion (@sec-infer).],
  [A parameter stopped following its formula], [`set_param` replaced the formula with a plain number. Give it the formula again with `set_param_expr` (@sec-param-formulas).],
  [A formula's value didn't change after its inputs did], [The new inputs made the formula fail, for example by dividing by zero, so it kept its old value. Change the inputs back, or give a formula that works (@sec-param-formulas).],
  [The solver stops with `Stagnated`], [Two or more rules contradict each other. Check the degrees of freedom: the report says `Conflicting` and gives the clashing equation's number.],
)

== Error messages <sec-errors>

=== From `swx-qa`

Every message starts with `swx-qa: `. No picture is written when there is an error.

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`missing command`], [Nothing after `swx-qa`. Add `render`, `sketch`, `thumbnail`, or `--help`.],
  [#raw("unknown command `x`")], [Only `render`, `sketch`, `thumbnail` and `help` exist.],
  [#raw("unknown option `x`")], [A misspelled option, or one that belongs to the other command (`--crease` and `--step` are for `render` only). The help text follows the message.],
  [#raw("`--size` needs a value")], [An option was last on the line without its value. The same goes for the other options.],
  [`missing input: give --fixture <name> or --step <file.step>`], [Say what to draw.],
  [`give exactly one of --fixture and --step`], [Two inputs were given. Keep one.],
  [`missing input: give --fixture <name>`], [`sketch` needs the name of a built-in sketch.],
  [`missing output: give -o <out.png>`], [Say where to save the picture.],
  [#raw("unknown fixture `x`; available: cube, cylinder, l-bracket")], [Use one of the listed names.],
  [#raw("unknown sketch fixture `x`; available: rectangle, triangle-345")], [Use one of the listed names.],
  [#raw("invalid --size `x`: expected <W>x<H>, e.g. 800x600")], [Write the size as two whole numbers joined by `x`, with no spaces.],
  [#raw("invalid --size `x`: width and height must be > 0")], [Neither side may be 0.],
  [`image size WxH must be between 1x1 and 8192x8192`], [Use a smaller size.],
  [#raw("invalid --crease `x`: expected degrees")], [Give a number, such as `45`.],
  [`crease angle x must be between 0 and 180 degrees`], [Use a value from 0 to 180.],
  [`cannot read f: invalid input: no such file: f`], [The STEP file isn't there. Check the name and the folder.],
  [`cannot read f: kernel failure: import_step from f failed`], [The file isn't valid STEP. The geometry engine prints the reason just above.],
  [`cannot write f: …`], [The picture can't be saved there, for example because the folder doesn't exist.],
  [`mesh is empty`, `mesh has no visible extent`], [The STEP file holds nothing that can be drawn.],
  [`thumbnail takes exactly one document path`], [`thumbnail` was given no document, two, or an option. Give one folder or `.sltwx` file. The help text follows the message.],
  [`f has no fresh body BRep (cache/body-<id>.brep): the cache must be built first`], [The part has no body files in its cache, or they are out of date. Build the cache (@sec-files-part-cache), then try again. Nothing is written.],
  [`cannot read f: …`, `cannot write f: …` (thumbnail)], [The document can't be read or saved; the reason is a document-file message (@sec-errors) or the system's.],
  [#raw("cannot make a thumbnail of f: reading body BReps needs OCCT; rebuild with `cargo build -p swx-qa --features occt`")], [`swx-qa` was built without OpenCascade. Rebuild it with `--features occt`.],
)

=== From the geometry kernel

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`invalid input: <name> must be finite and > 0, got <v>`], [A size, radius or height is zero, negative or not a number.],
  [`invalid input: <name> must have finite coordinates, got …`], [A position contains an infinite value or not-a-number.],
  [`invalid input: no such file: f`], [The STEP file to read doesn't exist.],
  [`invalid input: adjacent needs two different kinds, got edge and edge`], [`adjacent` was asked for the same kind it starts from. Ask for another kind: the faces or vertices of an edge (@sec-topology).],
  [`invalid input: face index 7 is out of range: the shape has 7 faces`], [The index is past the last sub-shape of its kind. Indices start at 0, so the last is the count minus 1. An index from another shape, or from before a change, can also be out of range (@sec-topology).],
  [`kernel failure: <operation> failed`], [OpenCascade couldn't complete the operation. For a cut, check that the tool solid overlaps the part (@ch-solids). For a `fillet` or a `chamfer`, the size is too large for the faces next to the edges: make it smaller (@sec-fillet-mistakes). For a `shell`, the walls are too thick for the part: keep them under half its narrowest width (@sec-shell-mistakes). For a `draft`, a face can't be tilted, such as one parallel to the neutral plane (@sec-draft-mistakes). For a `loft`, two sections may be in the same place (@sec-loft-mistakes).],
  [`invalid input: BRep text is empty`], [`read_brep` was given no text, or only spaces. Check that the file was read, and isn't empty (@sec-files-brep).],
  [`kernel failure: read_brep: not valid BRep text`], [The text isn't BRep, or was cut short. Write it again from the solid with `write_brep`. A cache can always be rebuilt from the model, so a damaged one can simply be replaced.],
  [`invalid input: the edge list is empty`], [A fillet or a chamfer was given no edges. Give at least one (@ch-fillet).],
  [`invalid input: the face list is empty`], [A shell was given no faces to open, or a draft no faces to tilt. Give at least one (@ch-shell, @ch-draft).],
  [`invalid input: vertex 1 is not on edge 11`], [A variable fillet's start vertex isn't one of its edge's two ends. Ask `adjacent` for the edge's vertices (@sec-fillet-variable).],
  [`invalid input: a variable fillet needs an edge and a vertex, got Face 0 and Vertex 2`], [The first sub-shape must be an edge, the second a vertex (@sec-fillet-variable).],
  [`invalid input: counterbore diameter 5 must be larger than the hole diameter 6`], [A counterbore's recess must be wider than its hole (@sec-hole-mistakes).],
  [`invalid input: hole depth 4 must be larger than the counterbore depth 6`], [A blind hole must go deeper than its recess. Make it deeper, or go through all.],
  [`invalid input: countersink diameter 6 must be larger than the hole diameter 6.4`], [A countersink's cone must be wider than its hole (@sec-hole-mistakes).],
  [`invalid input: countersink angle <v> (<d>°) must be strictly between 0 and π radians`], [The angle across the cone is 0, 180° or more, not a number, or given in degrees. `<d>` is your angle in degrees. Give radians: `90f64.to_radians()` (@sec-hole-tapped).],
  [`invalid input: hole depth 2 must be larger than the countersink depth 2.8`], [A blind hole must go deeper than its cone. The cone depth is rounded to 6 decimal places. Make the hole deeper, or go through all.],
  [`invalid input: no metric coarse thread of size M7 (sizes: M2 to M24)`], [The size isn't in the thread table (@sec-ref-threads). Pick one that is.],
  [`invalid input: thread designation "6" must be M followed by a size, e.g. "M6"`], [The name is missing its `M`, or isn't a number after it.],
  [`invalid input: thread pitch 6 must be smaller than the nominal diameter 6`], [A thread typed by hand has a pitch as large as its size. Take sizes from `metric_coarse`.],
  [`invalid input: hole direction must be finite and non-zero, got …`], [The hole's direction is missing or not a number. Point it into the material.],
  [`invalid input: the hole misses the solid: it removes nothing`], [The hole's line misses the part, or points away from it (@sec-hole-mistakes).],
  [`kernel failure: hole failed: the result is not a valid solid`], [The geometry engine made the hole, but the result isn't a sound solid. Try a slightly different position or size.],
  [`empty result: the hole removes the whole solid`], [The hole is so large that nothing is left.],
  [`invalid input: draft angle must be finite, non-zero and below π/2 in size, got <v>`], [A draft angle is zero, 90° or more, not a number, or given in degrees. Give radians: `5f64.to_radians()` (@sec-draft).],
  [`invalid input: draft pull direction must be finite and non-zero, got …`], [The neutral plane's direction is missing or not a number.],
  [`invalid input: a loft needs at least 2 sections, got 1`], [A loft was given one section, or none (@sec-loft-mistakes).],
  [`invalid input: loft section 1 has a hole; sections with holes are not supported`], [A loft section is ring-shaped. Loft the outer outline, and cut the hole separately.],
  [`invalid input: loft section 1 is not a single planar face`], [A loft section is a solid, or more than one face. Pass faces from `make_face`.],
  [`empty result: the loft encloses no volume`], [The sections enclose nothing between them.],
  [`invalid input: sweep path is empty`], [A sweep was given no path (@sec-sweep-mistakes).],
  [`invalid input: sweep path: circles (closed paths) are not supported`], [The path holds a full circle. A sweep path must be open; for a ring, revolve instead (@ch-revolve).],
  [`invalid input: sweep path: segment 1 doesn't meet segment 0 within 0.0000001`], [Two neighboring pieces of the path don't touch. Reuse the same end values (@sec-sweep-mistakes).],
  [`invalid input: sweep path starts 5 off the profile's plane`], [The path's start isn't on the profile's plane; the number is how far off it is, in mm. Start the path at the profile's middle.],
  [`invalid input: sweep profile is not a single planar face`, `sweep profile has a hole; profiles with holes are not supported`], [The profile is a solid, or has a hole. Pass one face from `make_face`; for a tube, cut an inner sweep from an outer one.],
  [`invalid input: sweep path: arc segment 1 has radius 4, not larger than the profile's distance 5.000000232604544 from the path's start; the swept solid would pass through itself`], [A bend of the path is tighter than the profile is wide: the inside of the part would fold through itself. Make the bend's radius larger than the distance shown, or the profile smaller (@sec-sweep-mistakes).],
  [`empty result: the sweep encloses no volume`], [The path runs along the profile's own plane, so the profile sweeps out nothing. Draw the path on a plane that crosses the profile's.],
  [`invalid input: helix pitch must be finite and > 0, got 0`, `helix turns must be finite and > 0, got …`], [A helical sweep was given a pitch or a number of turns of 0 or less, or not a number (@sec-helix).],
  [`invalid input: helix axis direction must be non-zero`, `helix axis direction must be finite, got …`, `helix axis origin must be finite, got …`], [The helix axis has no direction (all zeros), or a value that isn't a number.],
  [`invalid input: helix axis (origin …, direction …) is not in the profile's plane (through …, normal …)`], [The axis must lie in the profile's plane. Draw the profile on a plane that holds the axis (@sec-sweep-mistakes).],
  [`invalid input: the profile crosses the helix axis: its distance from the axis spans [-1, 1]`, `the profile touches the helix axis: …`], [The profile isn't wholly on one side of the axis. Move it away from the axis.],
  [`invalid input: the profile's extent along the helix axis (2) must be less than the pitch (1.5), or neighboring turns overlap`], [The profile is as tall as the pitch or taller, along the axis, so each turn would run into the next. Raise the pitch.],
  [`invalid input: helical sweep profile is not a single planar face`, `helical sweep profile has a hole; profiles with holes are not supported`], [Pass one flat face without holes from `make_face`.],
  [`invalid input: work_axis_on needs an edge or a face, got a vertex`], [A work axis was asked of a corner. Give a straight edge or a round face (@sec-work-part).],
  [`invalid input: Edge 13 is not a straight edge or a cylindrical face` (or `Face …`)], [The edge is curved, or the face isn't round like a cylinder, so it has no single line.],
  [`invalid input: vertex_point needs a vertex, got a Edge sub-shape` (or `Face`)], [A work point was asked of an edge or a face. Give a vertex.],
  [`invalid input: the planes are not parallel: they have no midplane`], [A midplane needs two parallel planes (@sec-work-mistakes).],
  [`invalid input: face 1 is not cylindrical`], [A tangent plane was asked of a face that isn't round like a cylinder. Give the round side of a shaft, a boss or a hole (@sec-work-between).],
  [`invalid input: toward is parallel to the cylinder axis: it picks no side`], [The direction given for a tangent plane runs along the cylinder, so it doesn't say which side to touch. Point it out sideways from the axis.],
  [`invalid input: work_plane_tangent needs a face, got a Edge sub-shape` (or `Vertex`), `edge_midpoint needs an edge, got a Face sub-shape` (or `Vertex`)], [A tangent plane needs a face, and a midpoint an edge.],
  [`invalid input: combine needs at least one tool body`], [A combine was given an empty list of tools (@sec-bodies).],
  [`invalid input: pattern count 1 must be at least 1` (or `pattern count 2`, `pattern count`)], [A pattern was asked for no copies. The count includes the original, so the least is 1 (@sec-pattern-mistakes).],
  [`invalid input: mirror plane normal must be non-zero`, `pattern direction 1 must be non-zero` (or `2`), `pattern axis direction must be non-zero`], [A mirror plane, pattern direction or pattern axis of length zero. Give a direction that points somewhere.],
  [`invalid input: pattern spacing 1 must be finite, got [NaN]`, `mirror plane origin must be finite, got …`, `pattern axis origin must be finite, got …`], [A mirror or a pattern was given an infinite value or NaN.],
  [`invalid input: pattern total angle must be in (0, 2π] radians, got <v>`], [A circular pattern's total angle is 0 or less, more than a full turn, not a number, or given in degrees. Give radians: 2π for a full turn (@sec-pattern-circ).],
  [`invalid input: pattern count must be at least 1` from a path pattern, or `sweep path is empty`, `sweep path: segment 1 doesn't meet segment 0 within 0.0000001`, `sweep path: circles (closed paths) are not supported`], [A path pattern was asked for no copies, or its path is empty, has a gap or is a circle. A path pattern checks its path as a sweep does, so the message says "sweep path" (@sec-pattern-path).],
  [`invalid input: split plane normal must be non-zero`, `split plane origin must be finite, got …`], [A split plane of length zero, or with an infinite value or NaN (@sec-split).],
  [`kernel failure: mirror failed`, `pattern_rectangular failed`, `pattern_circular failed`, `pattern_path failed`, `pattern_path_aligned failed`], [The geometry engine couldn't move a copy. The original is unchanged.],
  [`kernel failure: split failed`, `split_faces failed`], [The geometry engine couldn't cut the body or divide its faces, or, for `split_faces`, the shape holds no solid. The original is unchanged.],
  [`invalid input: the combine target has no solid body`, `combine tool 0 has no solid body`], [The target, or a tool (counted from 0), is a flat face, not a solid. Extrude it first.],
  [`empty result: the combined result encloses no volume`], [A cut removed everything, or an intersect found no common volume.],
  [`invalid input: the planes are parallel: they have no line in common`], [Two parallel planes have no line where they meet (@sec-work-mistakes).],
  [`invalid input: the three plane points are collinear`], [Three points in a line don't fix a plane. Move one off the line.],
  [`invalid input: axis direction (the points coincide) must be non-zero`, `plane points (two coincide) must be non-zero`], [An axis or a plane was given the same point twice.],
  [`invalid input: the axis is parallel to the plane`], [The axis never crosses the plane, so there is no point (@sec-work-mistakes).],
  [`invalid input: the axis does not lie in the plane`], [A plane can turn only about an axis on it (@sec-work-mistakes).],
  [`invalid input: <name> must be finite, got …`, `<name> must be non-zero`], [A work feature was given a number that isn't finite, or a zero direction: for example `plane normal`, `axis direction` or `offset distance`.],
  [`invalid input: shell thickness must be finite and > 0, got <v>`], [A shell's wall thickness is zero, negative or not a number. Give a thickness in mm (@sec-shell).],
  [`invalid input: chamfer angle must be finite and strictly between 0 and π/2, got <v>`], [An uneven chamfer's angle is 0, 90° or more, not a number, or given in degrees. Give radians: `30f64.to_radians()` (@sec-chamfer-sizes).],
  [`invalid input: face 4 does not bound edge 11`], [An uneven chamfer's reference face isn't one of the two faces that meet at its edge. Ask `adjacent` for the edge's faces and pick one (@sec-fillet-mistakes).],
  [`invalid input: edge 11 is listed with two different faces (3 and 5)`], [An uneven chamfer lists the same edge twice, with different reference faces. Keep one.],
  [`invalid input: expected a face, got edge 0` (or `vertex`)], [An uneven chamfer's reference face, or one of a shell's open faces, is an edge or a vertex. Give a face (@sec-shell-mistakes).],
  [`invalid input: expected an edge, got face 0` (or `vertex`)], [A fillet's or a chamfer's list holds something that isn't an edge. Give edges only (@sec-fillet-mistakes).],
  [`invalid input: edge index 12 is out of range: the shape has 12 edges`], [As for faces: the edge number is past the last edge, or comes from another solid. Find the edges in the solid you are softening.],
  [`invalid input: face index 14 is out of range (the shape has 14 faces)`], [`face_plane`, or a face extent, was given a face number past the last face of the target.],
  [`invalid input: face_plane needs a face, got a Edge sub-shape` (or `Vertex`)], [`face_plane` or `work_plane_on_face` was given an edge or a vertex. Give a face.],
  [`invalid input: face 2 is not planar`], [The face is curved, so it has no plane: a to-face extrude can't end on it, and no work plane can sit on it. Pick a flat face (@sec-extrude-to-face, @sec-work-part).],
  [`invalid input: outer loop: consecutive segments don't meet within 0.0000001` (or `hole N loop: …`)], [Two pieces of the loop, or its last and first, don't meet. Reuse the same corner values for both ends (@ch-extrude).],
  [`invalid input: outer loop is empty`, `… loop encloses no area`], [The loop has no pieces, or they enclose nothing: a line there and back, or a figure eight whose halves are the same size.],
  [`invalid input: … loop of one segment must be a circle`, `… a circle must be the only segment of its loop`], [A loop of one piece must be a full circle, and a circle can't share a loop. Use arcs to round corners.],
  [`invalid input: line from … to … has no length`, `arc from … to … rad has no sweep; use a circle`], [A piece starts and ends at the same point.],
  [`invalid input: radius must be > 0, got <v>`], [An arc or circle has a zero or negative radius.],
  [`invalid input: line must be finite, got …` (also `arc`, `circle`, `workplane origin`)], [A number is infinite or not a number.],
  [`invalid input: workplane x_dir must be non-zero` (or `normal`), `workplane x_dir and normal must be perpendicular (dot …)`], [The workplane's directions are missing or not at right angles (@sec-ref-extrude).],
  [`kernel failure: make_face failed: the face is invalid (e.g. a self-intersecting loop or a hole outside the outer loop)`], [The loops pass the checks above, but the outline crosses itself, or a hole is outside the outline or touches its edge.],
  [`invalid input: extrude distance must be finite and non-zero, got <v>` (or `symmetric extrude length`)], [The extent is zero or not a number.],
  [`kernel failure: extrude failed`], [The shape given to `extrude` or `extrude_tracked` isn't a face from `make_face` or `make_face_tracked`.],
  [`invalid input: extrude mode "join" needs a target body` (or `"cut"`, `"intersect"`)], [The mode combines with a part, but none was given. Pass the part as the target, or use `NewSolid` (@sec-extrude-modes).],
  [`invalid input: a through-all extent works only with the cut mode, not "join"` (or another mode)], [Only a cut can go through all. Give a distance instead (@sec-through-all).],
  [`invalid input: a through-all extent needs a target body to measure; use extrude_feature with the cut mode`], [A through-all extent was given to `extrude` or `extrude_tracked`, which have no target to measure. Use `extrude_feature` with `ExtrudeMode::Cut`.],
  [`invalid input: a to-face or between-faces extent needs a target body: it works with the join, cut and intersect modes, not "new solid"`], [A face extent was given with the new-solid mode. Use join, cut or intersect, with the part as the target (@sec-extrude-to-face).],
  [`invalid input: a to-face or between-faces extent needs a target body to measure; use extrude_feature with the join, cut or intersect mode`], [A face extent was given to `extrude` or `extrude_tracked`. Use `extrude_feature`.],
  [`invalid input: target face 11 (normal …) is not parallel to the workplane (normal …)`], [The face stands at an angle to the outline's plane. Pick a face parallel to the workplane (@sec-extrude-to-face).],
  [`invalid input: face 5 lies in the workplane: the to-face extrude has zero length`, `faces 7 and 9 are coplanar: the between-faces extrude has zero length`], [The face is on the outline's own plane, or the two faces are on one plane, so the extrude has no length. Pick another face.],
  [`invalid input: extrude start and end must be finite, got …`, `extrude length (end − start) must be finite and non-zero, got 0`], [A from–to extent has a number that isn't finite, or the same start and end.],
  [`invalid input: the through-all cut misses the target: it removes nothing`], [The cut went the wrong way, or the outline isn't over the part. Change the direction or move the outline (@sec-through-all).],
  [`invalid input: revolve angle must be in (0, 2π] radians, got <v>`], [The angle is zero, negative, more than a full turn, not a number, or given in degrees. Give radians: a full turn is 2π (@sec-revolve).],
  [`invalid input: revolve axis direction must be non-zero` (or `must be finite, got …`), `revolve axis origin must be finite, got …`], [The axis is missing its direction, or has a number that isn't finite.],
  [`invalid input: the profile crosses the revolve axis (origin …, direction …): it reaches <a> on one side and <b> on the other`], [The axis runs through the outline. Move the axis or the outline so the outline is all on one side; touching the axis is fine (@sec-revolve-mistakes).],
  [`invalid input: revolve axis (origin …, direction …) is not in the profile's plane (through …, normal …)`], [The axis is tilted out of the outline's plane, stands straight out of it, or runs beside it. Use an axis that lies in the plane (@sec-revolve-mistakes).],
  [`invalid input: revolve needs a single planar face`], [The shape given to the revolve isn't one flat face. Pass a face from `make_face`.],
  [`kernel failure: revolve failed`], [The geometry engine couldn't make the revolve, although its input passed every check above.],
  [`invalid input: revolve mode "join" needs a target body` (or `"cut"`, `"intersect"`)], [As for an extrude: pass the part as the target, or use `NewSolid` (@sec-revolve-modes).],
  [`empty result: revolve new solid leaves no solid (volume …)` (or `join`, `cut`, `intersect`)], [Nothing is left: for a cut or an intersect, as for an extrude.],
  [`empty result: extrude cut leaves no solid (volume …)` (or `intersect`, `join`)], [Nothing is left: the cut removed the whole part, or the intersect found no overlap. Make the outline smaller, or move it over the part.],
)

=== From sketches

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`geometry values must be finite`], [A coordinate, radius or angle is infinite or not a number.],
  [`radius must be > 0, got r`], [A circle or arc needs a positive radius.],
  [`value must be > 0, got v`], [A distance, radius or diameter constraint needs a positive value.],
  [`unknown geometry N`], [The constraint names an entity that isn't in this sketch.],
  [`geometry N has a different kind`], [Wrong kind of point or entity, such as `Center` of a line, `ArcStart` of a circle, a circle in parallel, or a line made equal to a circle (@ch-sketching).],
  [`unknown constraint N`], [No constraint with that number is in this sketch.],
  [`constraint N is not a dimension`], [Only a distance, radius or diameter has a value to change.],
  [#raw("only distance, radius and diameter can be bound, got …")], [`bind_dimension` was given another kind of constraint. Add it with `add_constraint` instead (@sec-drive-sketch).],
  [#raw("a sketch dimension is bound to unknown parameter `p`")], [The parameter was renamed or never existed. After `doc.rename_param`, call `rename_bound` with the same names (@sec-drive-sketch).],
)

=== From profiles

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`geometry A and B intersect`], [Pieces A and B cross, overlap, or touch away from their ends. Move or shorten one so they meet only end to end (@sec-profiles).],
  [`weld tolerance must be > 0, got t`], [Give a `weld_tolerance` greater than zero, or `None` for the default.],
)

Any sketch message above can also come from profiles, when the sketch itself is at fault.

=== From the document model

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [#raw("unknown parameter `p`")], [Set the parameter before a feature uses it. Renaming or commenting on a parameter that doesn't exist also gives this.],
  [#raw("a parameter named `p` already exists")], [Every parameter needs its own name. Pick another, or rename the other parameter first (@sec-param-table).],
  [#raw("invalid parameter name `p`: must start with a letter or `_` and contain only letters, digits and `_`")], [Use a name like `hole_2`: no leading digit, no spaces, dashes, dots or accents.],
  [`unknown feature N`], [No feature with that number is in the history.],
  [`feature N references feature M which is not earlier in the history`], [A feature can use only features above the end-of-part marker. `M` is a feature number, not a place in the history: it may be rolled back (@sec-roll-back), or not made yet.],
  [`end-of-part position P is past the end of the history (N features)`], [The marker can go anywhere from 0 to the number of features, `N`. It stayed where it was (@sec-roll-back).],
  [#raw("expression for `p`: …")], [The formula for `p` doesn't parse, names an unknown parameter, or can't be worked out. The rest is an expression message, below.],
  [#raw("parameter expressions form a cycle: a -> b -> a")], [The formulas would depend on each other in a loop, traced in the message. Make one of them a plain number (@sec-param-formulas).],
  [#raw("expression for `p` gives length^2, but its unit expects length")], [The formula's result is the wrong kind of quantity for the parameter's unit, here an area for a length. A parameter with no unit, or a unit outside the seven, expects `unitless`.],
)

=== From expressions

#block(breakable: false)[
Each message gives the byte where the problem is, counting from 0 at the first character (@sec-expressions).

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`syntax error at byte n: …`], [The text isn't a formula: an unexpected character, a missing `)` or number, or an expression too long or too deeply nested. The rest of the message says which.],
  [#raw("unknown unit `u` at byte n (expected mm, cm, m, in, ft, deg or rad)")], [A word after a number isn't one of the seven units. Check the spelling.],
  [#raw("unknown name `p` at byte n")], [No parameter has that name. Names are case-sensitive.],
  [#raw("dimension mismatch at byte n: cannot apply `+` to length and angle")], [Both sides of `+` or `-` must measure the same thing. Give a bare number its unit, as in `d1 + 5 mm`.],
  [`division by zero at byte n`], [The part after `/` works out to zero.],
)
]

=== From inference

For an unknown entity, `infer` and `add_inferred` give the sketch messages above. They can also give:

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`geometry values must be finite`], [A setting is infinite or not a number (@sec-ref-infer).],
  [`value must be > 0, got v`], [A setting is zero or less. Both tolerances must be greater than zero.],
  [`solve did not converge (residual r)`], [From `add_inferred` only. The suggested constraints were added, but couldn't all be met. Check the degrees of freedom for a conflict (@sec-check-dof). A constraint can't be removed yet, so build the sketch again and add only the suggestions you want (@sec-infer).],
)

#block(breakable: false)[
=== From document files

`swx-io` messages have no prefix (@ch-files). Fields are named by their path, such as `units.length`.

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`invalid JSON: model.json: expected value at line 1 column 17`], [The file isn't valid JSON. The message names the file (not for `read_manifest` alone) and where reading stopped.],
  [#raw("missing required field `units.length`")], [A required manifest field is missing (@sec-ref-files).],
  [#raw("invalid field `kind`: must be \"part\", \"assembly\" or \"drawing\", got \"widget\"")], [A manifest field has a value that isn't allowed, or the wrong type. The reason names what is allowed. The same form covers `format`, `format_version`, `document_id`, `model_hash` and `units`.],
  [#raw("invalid field `model.json`: must be a JSON object")], [The model or the properties isn't a JSON object (also `properties.json`, and `manifest` for a manifest that isn't an object).],
  [`unsupported format_version "1.0.0": only major version 0 is supported`], [The document comes from a newer major version. Use a newer Saulitwerx.],
  [#raw("missing required file `model.json`")], [The folder or file has no `model.json` or no `manifest.json` (@sec-files-mistakes).],
  [#raw("folder extension `.sltwxa` does not match the manifest kind `part` (expected `.sltwx`)"), #raw("folder has no extension; the manifest kind `part` needs `.sltwx`")], [Name the folder for its kind: `.sltwx` for a part, `.sltwxa` for an assembly, `.sltwxd` for a drawing.],
  [#raw("file extension `.sltwxa` does not match the manifest kind `part` (expected `.sltwx`)"), #raw("file has no extension; the manifest kind `part` needs `.sltwx`")], [The same check for a document kept as one file. Name the file for its kind, as for a folder.],
  [`files/missing.sltwx: No such file or directory (os error 2)`, `…: not a directory`], [The folder or a file can't be read, written or removed. The message starts with the path and ends with the system's reason.],
  [`malformed ZIP archive: no end-of-central-directory record`], [The file isn't a ZIP, or was cut short (a copy or download that didn't finish). Copy it again (@sec-files-mistakes).],
  [#raw("malformed ZIP archive: entry `model.json` is truncated"), #raw("… has corrupt deflate data: …"), #raw("… is 88 bytes, its header says 87"), `… ZIP64 archives are not supported`], [The file is damaged, or uses a ZIP feature Saulitwerx doesn't read (ZIP64, encryption, several disks). The message names what is wrong. Use an earlier copy, or re-zip the files with default settings.],
  [#raw("archive entry `model.json` fails its CRC-32 check")], [A byte in that file changed after it was written. Use an earlier copy.],
  [#raw("archive entry `model.json` uses unsupported compression method 12 (only 0 and 8 are supported)")], [The ZIP was made by a tool set to another compression. Re-zip with default settings, or unpack it to a folder.],
  [#raw("duplicate id `body-2` in `bodies`")], [Two items in one of a part's ID arrays share an ID. Give one a new ID (@sec-files-ids).],
  [#raw("`bodies[1]` needs a non-empty string `id`")], [That item of a part's ID array has no `id`, an empty one, or one that isn't text. The array is named by its path, such as #raw("sketches[0].constraints") (@sec-files-ids).],
  [#raw("invalid field `bodies`: must be an array")], [One of a part's ID arrays isn't a list. Write it in square brackets.],
  [#raw("invalid field `cache`: \"cache/meshes/display.glb\" is not `thumbnail.png` or `cache/<name>`")], [A cache file's path has a folder in it, or isn't in `cache/`. Nothing is written. Keep cache files directly in `cache/` (@sec-files-cache). A malformed `model_hash` in a cache entry is refused the same way.],
  [`deflection 0 mm is invalid: it must be finite and greater than 0` (from `build_part_cache`)], [The display mesh's deflection is 0, less than 0, or not a number. Pass `DISPLAY_DEFLECTION_MM` (@sec-files-part-cache). The cache is unchanged.],
  [`body ID "a/b" can't name a cache file`], [A body's ID is empty, or holds `/` or `\`, so it can't name its BRep file. The cache is unchanged.],
  [`could not build the body cache: …`, `could not write the display mesh: …`], [The geometry engine couldn't write a body or break it into triangles, or the merged mesh couldn't be written; the reason follows. The cache is unchanged.],
  [#raw("invalid field `attachments`: …")], [An embedded file's name isn't a plain file name: it is empty, holds a `/` or `\`, or is `.` or `..`. Nothing is written.],
)
]

=== From importing STEP files

The first five come from `embed_step`, when you import, and leave the part as it was. The last three come from `rebuild_imported`, when the solid is read back from the part; the part itself still opens (@sec-files-import).

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [#raw("cannot read STEP file `import/missing.step`: No such file or directory (os error 2)")], [The file isn't there, or can't be read. Check the path.],
  [#raw("`import/notes.txt` is not a STEP file (ISO 10303-21 header missing)")], [The file doesn't start with `ISO-10303-21;`, as every STEP file does. Ask for the file again, saved as STEP.],
  [#raw("STEP import of `import/cut-short.step` failed: kernel failure: import_step from import/cut-short.step failed")], [The geometry engine couldn't read the file: it is damaged or cut short. Its reason is printed just above.],
  [#raw("`part.step` contains no solid")], [The file holds only surfaces, curves or points, with no volume. Ask for a STEP file of the solid.],
  [#raw("cannot import into this document: only a part can import a STEP file")], [Import into a part (`.sltwx`), not an assembly or a drawing. The same start covers a `model.json` whose `features` or `bodies` isn't a list.],
  [#raw("no `import_step` feature with id `ft-1`")], [The part has no imported feature with that ID. Use the `id` that `embed_step` returned.],
  [#raw("Imported1: embedded STEP file `attachments/ft-ec796e41.step` is missing"), #raw("… is damaged (sha256 does not match)")], [The copy inside the document was deleted or changed. Use an earlier copy of the document, or import the STEP file again.],
  [#raw("Imported1: STEP import failed: …")], [The geometry engine couldn't read the copy, though its hash matches. The reason follows the colon.],
)

=== From `sltwx`

Every message starts `sltwx: `. Any document-file message above can follow it, when reading or writing fails.

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`sltwx: plate.sltwx: already exists`], [The file or folder to write is already there. `sltwx` never overwrites: delete it first, or pick another name (@sec-files-sltwx).],
  [`sltwx: copy/plate.sltwx: parent folder does not exist`], [`unpack` makes only the last folder of the path. Make the ones above it first, for example with `mkdir -p copy`.],
  [The usage, starting `usage: sltwx pack`], [The words after `sltwx` aren't one of its forms: a missing or extra path, or a misspelled `pack` or `unpack`. Exit status 2 (@sec-ref-sltwx).],
)

// Keeps the heading with its table.
#pagebreak(weak: true)
=== From STL files

`swx_io::stl` messages have no prefix (@sec-ref-stl).

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`index count 2 is not a multiple of 3`], [The triangle list doesn't hold whole triangles. Each triangle needs exactly three corner numbers.],
  [`index 7 is out of range for a mesh with 4 vertices`], [A triangle names a corner that isn't in the corner list. Corners count from 0, so the last of 4 is 3.],
  [`vertex 1 has a non-finite coordinate`], [A corner's position is infinite or not a number, often from dividing by zero. Fix the program that made the mesh.],
  [`… triangles do not fit in a binary STL file`], [More than 4 294 967 295 triangles. Use a coarser mesh.],
  [`could not write STL: …`], [Writing failed, for example a full disk. The rest is the system's reason.],
  [`deflection 0 mm is invalid: it must be finite and greater than 0`], [From `export_stl`: the deflection is 0, less than 0, or not a number. Use 0.01 mm unless you need otherwise (@sec-stl-export). No file is made.],
  [`could not tessellate the shape: …`], [From `export_stl`: the geometry engine couldn't break the shape into triangles. The rest is the kernel message, below.],
)

=== From glTF files

`swx_io::glb` messages have no prefix (@sec-ref-glb). The first three are the same as for STL, above.

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`index count 2 is not a multiple of 3`], [The triangle list doesn't hold whole triangles.],
  [`index 7 is out of range for a mesh with 4 vertices`], [A triangle names a corner that isn't in the corner list.],
  [`vertex 1 has a non-finite coordinate`], [A corner's position is infinite or not a number.],
  [`the mesh has no triangles`], [The triangle list is empty, so there is nothing to show. Check that the program that made the mesh produced one.],
  [`… bytes do not fit in a GLB file`], [The file would be over 4 GB. Use a coarser mesh.],
  [`could not write GLB: …`], [Writing failed, for example a full disk. The rest is the system's reason.],
)

== Questions

*Is there a version with windows and buttons?* Not yet. The desktop app is a later milestone; see *Coming next*.

*Can I open files from other CAD programs?* Yes, if you save them as STEP first. A STEP file can also be brought into a part and kept inside it (@sec-files-import). Saulitwerx doesn't read any program's own native file format, and won't.

*Can I 3D-print a part?* Yes. Save it as an STL file with `export_stl`, which every 3D-printing program reads (@sec-stl-export). For now that takes a short program; the desktop app will make it a menu choice.

*Can I save my design and edit it later?* Not yet. Today you can save solids as STEP, which keeps the shape but not the history. Saulitwerx can write and read its own document files, as a folder or as one file (@ch-files), but can't yet store a part's history in them or rebuild a part from one.

*Does it cost anything?* No. Saulitwerx is free and open source, under the MIT or Apache-2.0 license, at your choice.

*Why millimeters?* Mechanical design works mostly in millimeters, so it is the default. Choosing other units per part is planned.

== Known limitations

- No desktop app, 3D view, mouse or keyboard controls yet.
- A sketch's regions can't be extruded or revolved directly yet: type the outline's pieces into a program (@ch-extrude, @ch-revolve).
- An extrude can end on a face (@sec-extrude-to-face) only when the face is flat and parallel to the workplane, and it ends on the face's whole plane: it doesn't stop at the face's edges, or follow a curved face.
- A fillet has one radius for all its edges, and a chamfer one size for all its edges: one distance, two distances, or a distance and an angle (@ch-fillet). A fillet whose radius changes along the edge rounds one edge per call, changing evenly from one end to the other (@sec-fillet-variable).
- A shell has one wall thickness for every wall, and needs at least one open face (@ch-shell). A part hollow inside with no opening, or with walls of different thicknesses, can't be made yet.
- A revolve turns one way from the outline's plane by an angle; it can't yet turn both ways or symmetrically.
- An extrude's or a revolve's target is a solid you hold in your program. Faces and edges can be counted and numbered (@sec-topology), a plain extrude says which face each piece of the outline became (@sec-extrude-history), and a Boolean what became of each face (@sec-boolean-history), but they can't be picked, and their numbers don't last through a change; so the outline's workplane is given by its numbers (@sec-extrude-modes).
- Holes have flat bottoms, with no drill points. A tapped hole is drilled at its tap-drill size, but its thread isn't modeled, and only metric coarse sizes from M2 to M24 are built in (@sec-hole-tapped).
- A draft tilts flat faces by one angle about one neutral plane; it can't split a face or step its neutral line (@ch-draft).
- Mirrors and patterns copy whole bodies, not features; the copies are separate shapes that your program joins or cuts with `combine`. A pattern can't skip copies, follow a path that leaves its plane, or change from copy to copy (@ch-pattern). A split cuts with a flat plane only, not a surface or a sketch.
- A part with several bodies (@sec-bodies) is one shape: its bodies can be counted and taken apart, but not named, moved or kept apart in a part's history.
- Work planes, axes and points can be taken from a part's faces, edges and corners only by their numbers, not picked; an axis taken from the part may point either way (@sec-work-part).
- A loft goes through flat sections without holes, in the order given, with no guide curves and no control over how the sections line up (@ch-loft).
- A sweep moves one flat profile without holes along an open path of lines and arcs in one plane, or around a helix of one radius and one pitch: no closed, spline or other 3D paths, no twist or change of size. A bend must be wider than the profile: the check uses the profile's farthest point from the path's start, so it can refuse a bend that would just fit on the profile's narrow side (@ch-sweep).
- Extrudes, revolves, lofts, sweeps, fillets, chamfers, shells, holes, drafts, mirrors, patterns, splits and work features aren't recorded in a part's history, so they don't rebuild when a parameter changes.
- `model.json` isn't yet filled from a part or read back into one, so work can't be saved with its history (@ch-files). A single file can't hold more than 4 GB (no ZIP64).
- A solid can be saved as STL in one step (@sec-stl-export), but STL can't be read. GLB files are written only from a mesh your program gives (@sec-glb), and hold only the shape, with no colors or materials.
- A document can hold a cache (@sec-files-cache), built from bodies your program hands over (@sec-files-part-cache), with a thumbnail from `swx-qa thumbnail` (@sec-files-thumbnail). Nothing builds a part's bodies from `model.json` yet, so the cache can't be filled from a saved part alone.
- `swx-qa sketch` draws only its two built-in sketches. Your own sketches need a short program (@sec-picture-sketch). Sketch pictures show no dimension arrows and no marks for constraints such as horizontal.
- A parameter's unit is free text. Only the seven units of @sec-ref-expr are understood by formulas; any other unit counts as none.
- When a new value breaks a formula that depends on it, the formula keeps its old value without an error (@sec-param-formulas).
- A constraint can't be removed from a sketch once added; build the sketch again without it.
- Inference needs a program: there is no drawing with the mouse yet, so nothing is inferred as you move the pointer (@sec-infer).
- Renaming a parameter doesn't update the sketches bound to it by itself; call `rename_bound` too (@sec-drive-sketch).
- There are no angle dimensions and no point-to-line distance yet.
- Profiles don't split pieces where they cross; crossing pieces are an error.
- Only built on and tested on Ubuntu Linux.
- An imported STEP file becomes one shape, and one body however many solids it holds. Its sizes can't be edited, it can't be moved or turned as it comes in, and its faces and edges can't be picked. It can't be linked to the original file, or its file replaced (@sec-files-import).
