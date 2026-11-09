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
  [`missing command`], [Nothing after `swx-qa`. Add `render`, `sketch`, or `--help`.],
  [#raw("unknown command `x`")], [Only `render`, `sketch` and `help` exist.],
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
  [`invalid input: the edge list is empty`], [A fillet or a chamfer was given no edges. Give at least one (@ch-fillet).],
  [`invalid input: the face list is empty`], [A shell was given no faces to open, or a draft no faces to tilt. Give at least one (@ch-shell, @ch-draft).],
  [`invalid input: vertex 1 is not on edge 11`], [A variable fillet's start vertex isn't one of its edge's two ends. Ask `adjacent` for the edge's vertices (@sec-fillet-variable).],
  [`invalid input: a variable fillet needs an edge and a vertex, got Face 0 and Vertex 2`], [The first sub-shape must be an edge, the second a vertex (@sec-fillet-variable).],
  [`invalid input: counterbore diameter 5 must be larger than the hole diameter 6`], [A counterbore's recess must be wider than its hole (@sec-hole-mistakes).],
  [`invalid input: hole depth 4 must be larger than the counterbore depth 6`], [A blind hole must go deeper than its recess. Make it deeper, or go through all.],
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
  [`empty result: the sweep encloses no volume`], [The path runs along the profile's own plane, so the profile sweeps out nothing. Draw the path on a plane that crosses the profile's.],
  [`invalid input: work_axis_on needs an edge or a face, got a vertex`], [A work axis was asked of a corner. Give a straight edge or a round face (@sec-work-part).],
  [`invalid input: Edge 13 is not a straight edge or a cylindrical face` (or `Face …`)], [The edge is curved, or the face isn't round like a cylinder, so it has no single line.],
  [`invalid input: vertex_point needs a vertex, got a Edge sub-shape` (or `Face`)], [A work point was asked of an edge or a face. Give a vertex.],
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

== Questions

*Is there a version with windows and buttons?* Not yet. The desktop app is a later milestone; see *Coming next*.

*Can I open files from other CAD programs?* Yes, if you save them as STEP first. Saulitwerx doesn't read any program's own native file format, and won't.

*Can I save my design and edit it later?* Not yet. Today you can save solids as STEP, which keeps the shape but not the history. Saulitwerx part files come later.

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
- Holes have flat bottoms and are simple or counterbored: no drill points, countersinks or threads yet (@ch-hole).
- A draft tilts flat faces by one angle about one neutral plane; it can't split a face or step its neutral line (@ch-draft).
- Work planes, axes and points can be taken from a part's faces, edges and corners only by their numbers, not picked; an axis taken from the part may point either way (@sec-work-part).
- A loft goes through flat sections without holes, in the order given, with no guide curves and no control over how the sections line up (@ch-loft).
- A sweep moves one flat profile without holes along an open path of lines and arcs in one plane: no closed, 3D or spline paths, no twist or change of size. A bend tighter than the profile gives a part that passes through itself, with no error (@ch-sweep).
- Extrudes, revolves, lofts, sweeps, fillets, chamfers, shells, holes, drafts and work features aren't recorded in a part's history, so they don't rebuild when a parameter changes.
- No native part files; work can't be saved with its history.
- `swx-qa sketch` draws only its two built-in sketches. Your own sketches need a short program (@sec-picture-sketch). Sketch pictures show no dimension arrows and no marks for constraints such as horizontal.
- A parameter's unit is free text. Only the seven units of @sec-ref-expr are understood by formulas; any other unit counts as none.
- When a new value breaks a formula that depends on it, the formula keeps its old value without an error (@sec-param-formulas).
- A constraint can't be removed from a sketch once added; build the sketch again without it.
- Inference needs a program: there is no drawing with the mouse yet, so nothing is inferred as you move the pointer (@sec-infer).
- Renaming a parameter doesn't update the sketches bound to it by itself; call `rename_bound` too (@sec-drive-sketch).
- There are no angle dimensions and no point-to-line distance yet.
- Profiles don't split pieces where they cross; crossing pieces are an error.
- Only built on and tested on Ubuntu Linux.
- An imported STEP file becomes one shape, and can't be edited.
