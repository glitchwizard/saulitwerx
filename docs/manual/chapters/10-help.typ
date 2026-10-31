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
  [`kernel failure: <operation> failed`], [OpenCascade couldn't complete the operation. For a cut, check that the tool solid overlaps the part (@ch-solids).],
  [`invalid input: outer loop: consecutive segments don't meet within 0.0000001` (or `hole N loop: …`)], [Two pieces of the loop, or its last and first, don't meet. Reuse the same corner values for both ends (@ch-extrude).],
  [`invalid input: outer loop is empty`, `… loop encloses no area`], [The loop has no pieces, or they enclose nothing: a line there and back, or a figure eight whose halves are the same size.],
  [`invalid input: … loop of one segment must be a circle`, `… a circle must be the only segment of its loop`], [A loop of one piece must be a full circle, and a circle can't share a loop. Use arcs to round corners.],
  [`invalid input: line from … to … has no length`, `arc from … to … rad has no sweep; use a circle`], [A piece starts and ends at the same point.],
  [`invalid input: radius must be > 0, got <v>`], [An arc or circle has a zero or negative radius.],
  [`invalid input: line must be finite, got …` (also `arc`, `circle`, `workplane origin`)], [A number is infinite or not a number.],
  [`invalid input: workplane x_dir must be non-zero` (or `normal`), `workplane x_dir and normal must be perpendicular (dot …)`], [The workplane's directions are missing or not at right angles (@sec-ref-extrude).],
  [`kernel failure: make_face failed: the face is invalid (e.g. a self-intersecting loop or a hole outside the outer loop)`], [The loops pass the checks above, but the outline crosses itself, or a hole is outside the outline or touches its edge.],
  [`invalid input: extrude distance must be finite and non-zero, got <v>` (or `symmetric extrude length`)], [The extent is zero or not a number.],
  [`kernel failure: extrude failed`], [The shape given to `extrude` isn't a face from `make_face`.],
  [`invalid input: extrude mode "join" needs a target body` (or `"cut"`, `"intersect"`)], [The mode combines with a part, but none was given. Pass the part as the target, or use `NewSolid` (@sec-extrude-modes).],
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
  [`feature N references feature M which is not earlier in the history`], [A feature can use only features made before it.],
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
- A sketch's regions can't be extruded directly yet: type the outline's pieces into a program (@ch-extrude). There is no revolve, fillet or hole yet.
- An extrude goes a set distance only: not "up to a face" or "through all". To cut through a part, give a distance at least as deep as the part.
- An extrude's target is a solid you hold in your program. Faces and edges can't be picked or named yet, so the outline's workplane is given by its numbers (@sec-extrude-modes).
- Extrudes aren't recorded in a part's history, so they don't rebuild when a parameter changes.
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
