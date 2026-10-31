#import "../template.typ": *

= Reference <ch-reference>

== `swx-qa render` <sec-ref-qa>

#idx("swx-qa render")
Draws a part as an isometric, flat-shaded PNG with its sharp edges outlined.

#term-block(label: "Form", "swx-qa render (--fixture <name> | --step <file.step>) -o <out.png> [options]")

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Option", "What it does", "Default"),
  [`--fixture <name>`], [Draws a built-in part: `cube`, `cylinder` or `l-bracket`. Give this or `--step`, not both.], [—],
  [`--step <file>`], [Draws a STEP file. Needs a build with `--features occt`.], [—],
  [`-o`, `--output <file>`], [The PNG to write. Required. An existing file is replaced.], [—],
  [`--size <W>x<H>`], [Picture size in pixels. Each side from 1 to 8192.], [`800x600`],
  [`--crease <deg>`], [Outline edges where the surface bends by more than this many degrees. 0 to 180.], [`30`],
  [`-h`, `--help`], [Prints the help text and does nothing else.], [—],
)

On success it prints `wrote <file> (<W>x<H>, <bytes> bytes)` and exits with status 0. On an error it prints `swx-qa: ` and the message (@sec-errors) and exits with status 1, without writing a file.

#ref-table(
  columns: (auto, 1fr),
  header: ("Built-in part", "Shape"),
  [`cube`], [A 10 mm cube with one corner at the origin.],
  [`cylinder`], [Radius 5 mm, height 10 mm, standing on +Z; 32 flat segments around.],
  [`l-bracket`], [An L profile with 30 mm legs and 8 mm thickness, 20 mm deep.],
)

== `swx-qa sketch` <sec-ref-qa-sketch>

#idx("swx-qa sketch")
Solves a built-in sketch and draws it from above as a PNG, with its dimension values.

#term-block(label: "Form", "swx-qa sketch --fixture <name> -o <out.png> [--size <W>x<H>]")

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Option", "What it does", "Default"),
  [`--fixture <name>`], [The built-in sketch to draw: `rectangle` or `triangle-345`. Required.], [—],
  [`-o`, `--output <file>`], [The PNG to write. Required. An existing file is replaced.], [—],
  [`--size <W>x<H>`], [Picture size in pixels. Each side from 1 to 8192.], [`800x600`],
  [`-h`, `--help`], [Prints the help text and does nothing else.], [—],
)

Success and errors work as for `render`. `--step` and `--crease` belong to `render` only.

#ref-table(
  columns: (auto, 1fr),
  header: ("Built-in sketch", "Shape"),
  [`rectangle`], [40 × 25 mm, lower-left corner fixed at the origin; sides horizontal and vertical; bottom and right dimensioned.],
  [`triangle-345`], [A right triangle with its right angle fixed at the origin; a horizontal side of 4 and a vertical side of 3, both dimensioned. The long side comes out 5.],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("In the picture", "Drawn as"),
  [Lines, circles, arcs], [Dark blue lines, 1 pixel wide.],
  [Points, line ends], [Red dots, 5 pixels across.],
  [Distance], [Its value, in green, beside the middle of the two points, on the side away from the sketch's center.],
  [Radius, diameter], [Its value with `R` or `Ø` in front, outside the circle at the upper right, or outside an arc at its middle.],
  [Values], [Up to 3 decimals, with trailing zeros dropped: `40`, `12.5`, `R2.125`.],
)

== Build options

#ref-table(
  columns: (auto, 1fr),
  header: ("Command", "What it builds"),
  [`cargo test --workspace`], [Everything without OpenCascade, and runs every test.],
  [`cargo build --release -p swx-qa --features occt`], [The `swx-qa` tool with STEP input, at `target/release/swx-qa`.],
  [`cargo run -p swx-sketch --example plate_sketch`], [Runs the sample sketch (@ch-sketching).],
  [`cargo run -p swx-sketch --example sketch_dof`], [Runs the degree-of-freedom sample (@sec-check-dof).],
  [`cargo run -p swx-sketch --example centered_plate -- [<width> <height>]`], [Runs the centered-hole sample (@sec-centered). Defaults: 40 and 25.],
  [`cargo run -p swx-sketch --example slot_sketch`], [Runs the slot sample (@sec-slot).],
  [`cargo run -p swx-qa --example plate_sketch_plot -- <dir> [<W>x<H>]`], [Draws the sample sketch in three stages, as PNGs in `<dir>` (@sec-picture-sketch).],
  [`cargo run -p swx-qa --example constraint_plots -- <dir> [<W>x<H>]`], [Draws the centered-plate and slot sketches as PNGs in `<dir>`.],
  [`cargo run -p swx-kernel --features occt --example plate_with_hole -- <file.step>`], [Builds the plate and saves it (@ch-solids). The file defaults to `plate.step`.],
  [`cargo run -p swx-kernel --features occt --example plate_stages -- <dir>`], [Saves the block, the cylinder and the plate as three STEP files in `<dir>`.],
  [`cargo run -p swx-core --example feature_history`], [Runs the history sample (@ch-history).],
  [`cargo run -p swx-core --example parameter_table`], [Runs the parameter-table sample (@sec-param-table).],
  [`cargo run -p swx-sketch --example driven_plate`], [Runs the bound-dimension sample (@sec-drive-sketch).],
  [`cargo run -p swx-core --example expressions`], [Runs the expression sample (@sec-expressions).],
  [`cargo run -p swx-core --example parameter_formulas`], [Runs the stored-formula sample (@sec-param-formulas).],
  [`cargo run -p swx-sketch --example inferred_sketch`], [Runs the inference sample (@sec-infer).],
  [`cargo run -p swx-qa --example inference_plots -- <dir> [<W>x<H>]`], [Draws the inference sketch as drawn and after inference, as PNGs in `<dir>`.],
)

#idx("occt feature")
The `occt` feature switches on OpenCascade. It is off by default, so a plain build needs only Rust.

== Geometry kernel operations

#ref-table(
  columns: (auto, 1fr),
  header: ("Operation", "What it does and what it accepts"),
  [`make_box(corner, size)`], [A box with its lowest corner at `corner` and sides `size` = [x, y, z]. Sizes > 0.],
  [`make_cylinder(base, r, h)`], [A cylinder along +Z with its base circle centered at `base`. Radius and height > 0.],
  [`make_sphere(center, r)`], [A sphere. Radius > 0.],
  [`union(a, b)`, `cut(a, b)`, `intersect(a, b)`], [Boolean operations (@sec-feature). Inputs unchanged.],
  [`volume(s)`, `surface_area(s)`], [In mm³ and mm².],
  [`bounding_box(s)`], [The smallest box, aligned with the axes, that holds the solid: its `min` and `max` corners.],
  [`face_count(s)`], [The number of faces.],
  [`tessellate(s, deflection)`], [Breaks the surface into triangles, no farther than `deflection` (mm, > 0) from the true surface.],
  [`export_step(s, path)`], [Writes a STEP file in millimeters, replacing any existing file.],
  [`import_step(path)`], [Reads everything in a STEP file as one shape.],
)

All positions must be finite numbers; all sizes finite and greater than zero.

== Profiles and extrude <sec-ref-extrude>

`make_face(plane, outer, holes)` makes a flat face on `plane`, bounded by the `outer` loop, with each loop in `holes` cut out. `extrude(face, extent)` sweeps that face along the plane's normal into a new solid (@ch-extrude).

#ref-table(
  columns: (auto, 1fr),
  header: ("Type", "Fields and rules"),
  [`Workplane`], [`origin` (a point), `x_dir` (the plane's left–right direction) and `normal` (straight out of the plane). Both directions non-zero and at right angles; their length doesn't matter. A 2D point (u, v) sits at `origin + u·x_dir + v·(normal × x_dir)`.],
  [`Workplane::XY`, `::XZ`, `::YZ`], [Built-in planes through the origin, with normals +Z, −Y and +X (@sec-extrude-direction).],
  [`Segment2::Line { start, end }`], [A straight piece. Its ends must be apart.],
  [`Segment2::Arc { center, radius, start_angle, end_angle }`], [A circular arc, counterclockwise from `start_angle` to `end_angle`, in radians from the +u axis. Radius > 0; the two angles must give different end points.],
  [`Segment2::Circle { center, radius }`], [A full circle. Radius > 0. It must be the only piece of its loop.],
  [`ProfileLoop { segments }`], [Pieces in chain order, each meeting the next and the last meeting the first. Any piece may run backwards, and the loop may go either way round.],
  [`Extent::Distance(d)`], [`d` mm along the normal; negative goes the other way. Not zero.],
  [`Extent::Symmetric(l)`], [`l` mm in total, half on each side of the plane. Not zero.],
  [`LOOP_GAP_TOLERANCE`], [1e-7 mm: the largest gap between two ends that still counts as joined. Ends that close are moved to meet exactly.],
  [`PERPENDICULAR_TOLERANCE`], [1e-9: how far from a right angle `x_dir` and `normal` may be.],
)

All numbers must be finite. A face is refused if a loop is empty, has no area, doesn't close, or crosses itself, or if a hole isn't wholly inside the outer loop. Every refusal is an error (@sec-errors), never a crash.

`extrude_feature(k, face, extent, mode, target)` extrudes `face` like `extrude`, then combines the result with `target` as `mode` says (@sec-extrude-modes). It returns a new solid; `target` is left unchanged.

#ref-table(
  columns: (auto, 1fr),
  header: ("Mode or value", "Result and rules"),
  [`ExtrudeMode::NewSolid`], [The extrusion alone. `target` is ignored and may be `None`.],
  [`ExtrudeMode::Join`], [Target ∪ extrusion. Needs a target.],
  [`ExtrudeMode::Cut`], [Target − extrusion. Needs a target.],
  [`ExtrudeMode::Intersect`], [Target ∩ extrusion. Needs a target.],
  [`EMPTY_VOLUME_TOLERANCE`], [1e-9 mm³: a result with less volume than this counts as empty and is refused with `EmptyResult`.],
)

A missing target is checked first, before any geometry is made. Errors from `extrude` and from the Boolean operations come through unchanged.

== Sketch entities and constraints <sec-ref-sketch>

#ref-table(
  columns: (auto, 1fr),
  header: ("Entity", "Defined by"),
  [`Point`], [x, y.],
  [`Line`], [A start point and an end point.],
  [`Circle`], [A center point and a radius > 0.],
  [`Arc`], [A center, a radius > 0, and start and end angles in radians, running counter-clockwise; the end angle must be after the start. Solving can change all five; it keeps the arc's sweep more than 0 and at most one full turn.],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("Constraint", "Rule"),
  [`Coincident(p, q)`], [Points `p` and `q` are at the same place.],
  [`Horizontal(line)`], [The line's two ends are at the same height.],
  [`HorizontalPoints(p, q)`], [`p` and `q` are at the same height.],
  [`Vertical(line)`], [The line's two ends are directly above each other.],
  [`VerticalPoints(p, q)`], [`p` is directly above or below `q`.],
  [`Fixed { point, x, y }`], [The point stays at (x, y).],
  [`Distance { a, b, value }`], [Points `a` and `b` are `value` apart. Value > 0.],
  [`Radius { geom, value }`], [The circle or arc has this radius. Value > 0.],
  [`Diameter { geom, value }`], [The circle or arc has this diameter. Value > 0.],
  [`Parallel(l1, l2)`], [Lines `l1` and `l2` point the same way (or exactly opposite ways).],
  [`Perpendicular(l1, l2)`], [Lines `l1` and `l2` are at 90°.],
  [`Equal(g1, g2)`], [Two lines of equal length, or two circles or arcs of equal radius (any mix of circle and arc). A line and a circle can't be made equal.],
  [`Midpoint { point, line }`], [The point is halfway between the line's two ends.],
  [`Symmetric { a, b, line }`], [Points `a` and `b` are mirror images across the line: the middle of `a`–`b` lies on the line (or on its extension), and `a`–`b` is perpendicular to it.],
  [`Tangent(g1, g2)`], [A line and a circle or arc, in either order, touch: the distance from the center to the line (extended without end) equals the radius. Or two circles or arcs touch, from outside or inside.],
)

A point `p` is written `PointRef::Point(id)` for a point entity, `Start(id)` or `End(id)` for a line's ends, `Center(id)` for a circle's or arc's center, or `ArcStart(id)` or `ArcEnd(id)` for the point of an arc at its start or end angle. Every constraint that takes a point accepts all six.

#ref-table(
  columns: (auto, 1fr),
  header: ("Tangent detail", "How it is decided"),
  [Line and circle], [The circle stays on the side of the line its center was on when the constraint was added.],
  [Two circles], [Touching from outside (centers `r1 + r2` apart) or inside (`|r1 − r2|` apart), whichever the drawing was closer to when the constraint was added.],
  [Line end joined to arc end], [When a coincident joins one of the line's ends directly to one of the arc's ends, tangent means the line is square to the arc's radius at that shared point. This holds whichever of the two constraints is added first; a join made through a separate point entity is not recognized.],
  [Where they touch], [Otherwise anywhere along the line. Tangent alone doesn't say where a line stops; add other constraints for its ends.],
)

#idx("tangent")
Tangency uses the standard distance conditions for lines and circles; see A. Bowyer and J. Woodwark, _A Programmer's Geometry_, chapters 3 and 5.

== Solver settings and report

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Setting", "Meaning", "Default"),
  [`tolerance`], [Done when the remaining error is at most this.], [`1e-10`],
  [`max_iterations`], [Give up after this many attempts.], [`100`],
  [`initial_damping`], [How cautious the first steps are.], [`1e-6`],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("Stop reason", "Meaning"),
  [`Converged`], [Every rule holds, within the tolerance. `converged` is `true`.],
  [`MaxIterations`], [The attempts ran out first.],
  [`Stagnated`], [No move makes it better: usually rules that conflict.],
  [`NonFinite`], [A value became infinite or not a number.],
)

#idx("Levenberg–Marquardt")
The solver uses the Levenberg–Marquardt method, a standard way to solve many equations at once; see J. Nocedal and S. J. Wright, _Numerical Optimization_, 2nd ed., chapter 10.

== Profiles <sec-ref-profile>

#idx("profile_report")
`swx_sketch::profile::profile_report(&sketch, &options)` finds the regions a sketch encloses and the pieces left open (@sec-profile). `profiles(&sketch, &options)` returns the regions alone. Point entities are ignored.

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Setting", "Meaning", "Default"),
  [`weld_tolerance`], [Ends of pieces closer than this count as one corner. `None` uses 0.000001 × the diagonal of the box around the sketch. Must be > 0 when given.], [`None`],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("Result", "Meaning"),
  [`regions`], [Every region found. Each has an `outer` loop and a list of `holes`.],
  [`Region::area()`], [The region's area: the outer loop's area minus its holes', in mm².],
  [`Loop`], [The `edges` around it in order, each a piece's id and whether it is walked backwards (`reversed`), and its `signed_area`.],
  [`signed_area`], [Positive for an outer loop, which runs counter-clockwise; negative for a hole, which runs clockwise. Exact for arcs.],
  [`open_edges`], [Ids of the pieces in no loop: free-ended pieces, pieces that only link two loops, and pieces of zero length. Smallest id first.],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("Rule", "What happens"),
  [Circle], [A loop on its own. So is an arc that goes all the way round.],
  [Loop inside a loop], [Becomes a hole of the smallest loop around it. A loop inside that hole is a new region (an island), and so on, alternating.],
  [Pieces that cross, overlap or touch away from their ends], [An error, `SelfIntersecting`, naming the first two such pieces. No regions are returned.],
  [No closed loop], [No regions, and no error.],
)

#idx("doubly connected edge list")
Loops are found by walking around each corner in angle order, the standard method for the faces of a flat network of lines; see M. de Berg, O. Cheong, M. van Kreveld and M. Overmars, _Computational Geometry_, 3rd ed., §2.2. Areas use Green's theorem (J. Stewart, _Calculus_, 8th ed., §16.4).

== Degree-of-freedom report <sec-ref-dof>

#idx("analyze")
#idx("dof")
`sketch.dof(options)` checks a sketch as it is now (@sec-dof) and returns a `SketchDofReport`. Run it after `solve`. The lower-level `swx_sketch::dof::analyze(&sketch, sketch.params(), options)` returns the `DofReport` alone.

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Setting", "Meaning", "Default"),
  [`rank_tolerance`], [How independent an equation must be, compared with the largest, to count as a new rule rather than a repeat.], [`1e-9`],
  [`residual_tolerance`], [A sketch with a repeated rule is `Conflicting`, not `OverConstrained`, when its remaining error is more than this.], [`1e-8`],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("Report field", "Meaning"),
  [`dof`], [Degrees of freedom left: the sketch's numbers minus its independent equations.],
  [`rank`], [The number of independent equations.],
  [`redundant`], [The numbers of the equations that repeat earlier ones, smallest first, counting from 0 in the order the constraints were added.],
  [`residual_norm`], [The remaining error: how far the rules are from being met, overall.],
  [`state`], [One of the states below.],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("State", "When"),
  [`UnderConstrained`], [No repeated rule, and `dof` is more than 0.],
  [`FullyConstrained`], [No repeated rule, and `dof` is 0.],
  [`OverConstrained`], [At least one repeated rule, and every rule is met.],
  [`Conflicting`], [At least one repeated rule, and the remaining error is more than `residual_tolerance`.],
  [`NonFinite`], [A value is infinite or not a number. Nothing else was worked out.],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("SketchDofReport field", "Meaning"),
  [`report`], [The `DofReport` above.],
  [`redundant_constraints`], [The constraints that own a repeated equation, as `ConstraintId`s, smallest first, each listed once. Of two rules that repeat each other, the one added later is listed.],
)

`sketch.constraint_of_row(n)` gives the constraint that made equation `n`, or `None` when there is no such equation.

#ref-table(
  columns: (auto, 1fr),
  header: ("Entity or constraint", "Numbers or equations"),
  [Point · Line · Circle · Arc], [2 · 4 · 3 · 5 numbers (degrees of freedom when free).],
  [Coincident · Fixed · Midpoint · Symmetric], [2 equations each.],
  [Every other constraint], [1 equation.],
)

#idx("Householder QR")
The count uses a Householder QR factorization, taking the equations in the order they were added; see G. H. Golub and C. F. Van Loan, _Matrix Computations_, 4th ed., §5.2.

== Inference <sec-ref-infer>

#idx("inference")
`swx_sketch::infer` suggests the constraints a newly added entity already almost satisfies (@sec-infer). `infer(&sketch, new, &options)` returns them and changes nothing. `add_inferred(&mut sketch, new, &options)` adds them in order, then solves, and returns the new constraint ids. Both fail if `new` doesn't exist or a setting is invalid.

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Setting", "Meaning", "Default"),
  [`point_tol`], [How close, in mm, two points must be to be joined, and how near a line or circle must come to touching to be made tangent. Finite and > 0.], [diagonal ÷ 1000],
  [`angle_tol_deg`], [How close, in degrees, a line must be to level, upright, parallel or perpendicular. Finite and > 0.], [1],
)

`InferOptions::for_sketch(&sketch)` gives the defaults for that sketch. The diagonal is that of the smallest axis-aligned box holding the whole sketch, with circles and arcs counted as full circles; for an empty sketch `point_tol` is 0.001. The suggestions come in this order:

#ref-table(
  columns: (auto, 1fr),
  header: ("Suggestion", "When"),
  [Coincident], [For each point of the new entity (a line's start, then end; a circle's center; an arc's center, start, then end), the nearest other point within `point_tol`, if any: a point, a line end, an arc end or a center. Ties go to the entity added first.],
  [Horizontal, vertical], [The new entity is a line within `angle_tol_deg` of level or upright.],
  [Parallel, perpendicular], [The new line is within `angle_tol_deg` of parallel or square to another line. At most one, the closest. When the new line got horizontal or vertical, lines that are themselves level or upright are skipped.],
  [Tangent], [A line whose distance to a circle's or arc's center is within `point_tol` of the radius (the line counted as endless); or two circles or arcs whose centers are within `point_tol` of touching, outside or inside. Circles with the same center are skipped.],
)

A constraint the sketch already holds, in either order, is never suggested, so inferring twice adds nothing. Inference never suggests a dimension.

#ref-table(
  columns: (auto, 1fr),
  header: ("Error from add_inferred", "Meaning"),
  [`Sketch`], [The entity doesn't exist, or a setting is not finite or not > 0. Nothing is added.],
  [`Solve`], [The solver could not start.],
  [`NotConverged`], [The constraints were added, but the solve did not finish; the geometry is the solver's best attempt.],
)

== Parameters <sec-ref-params>

#idx("Document")
A `swx_core::Document` keeps its parameters in creation order (@sec-param-table). Every call that can fail leaves the table unchanged when it does.

#ref-table(
  columns: (auto, 1fr),
  header: ("Call", "What it does"),
  [`add_model_param(value, unit)`], [Makes a model parameter named `d<N>` and returns the name. `N` starts at 0 and only goes up; a name already taken is skipped.],
  [`add_user_param(name, value, unit)`], [Makes a user parameter. Fails if the name is invalid or taken.],
  [`rename_param(old, new)`], [Renames a parameter, every feature that uses it, and every formula that names it (whole names only: renaming `d1` leaves `d10` alone). The kind is kept. Renaming to the same name does nothing.],
  [`set_param_comment(name, comment)`], [Sets the comment. Fails if there is no such parameter.],
  [`set_param(name, value)`], [Sets the value and drops the parameter's formula, if it had one, then works out every formula that depends on it. If one of those fails, they keep their old values, with no error. An unknown name makes a user parameter with an empty unit; this name is not checked.],
  [`set_param_expr(name, text)`], [Gives the parameter a formula (@sec-ref-expr) and works out the whole table. The result is stored in the parameter's unit; a unitless result is taken in that unit. Fails if the parameter or a name in the formula is unknown, the formula doesn't parse or can't be worked out, its result is the wrong kind of quantity for the unit, or formulas would form a loop.],
  [`evaluation_order()`], [Every parameter name, each after the parameters its formula names; ties keep creation order.],
  [`dirty_after_param_change(name)`], [The features to rebuild when `name` changes, including those that use a parameter whose formula depends on it, directly or through other formulas.],
  [`param(name)`], [The value, or `None`.],
  [`parameters()`], [Every parameter, in creation order.],
  [`is_valid_param_name(name)`], [Whether a name is allowed.],
)

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Parameter field", "Meaning", "Allowed values"),
  [`name`], [Unique within the part. Case-sensitive.], [a letter or `_`, then letters, digits or `_` (ASCII)],
  [`kind`], [Where it came from.], [`Model`, `User`],
  [`value`], [The number; for a parameter with a formula, its latest result.], [any number],
  [`unit`], [Free text; not checked. Formulas understand only the units in @sec-ref-expr.], [any, may be empty],
  [`comment`], [A note for people.], [any; empty by default],
  [`expr`], [The formula as typed, or `None` for a plain number.], [`None` by default],
)

#idx("Kahn's algorithm")
The dependency order uses Kahn's algorithm, always taking the earliest-made parameter that is ready; see A. B. Kahn, "Topological sorting of large networks", _Communications of the ACM_ 5(11), 1962, and T. H. Cormen et al., _Introduction to Algorithms_, 3rd ed., §22.4.

A unit that is not one of the seven in @sec-ref-expr, including an empty one, counts as unitless when a formula reads or stores the parameter.

#idx("bind_dimension")
`swx_sketch::params` ties a sketch's dimensions to a document's parameters (@sec-drive-sketch). A sketch remembers each binding as a pair of constraint and parameter name; `sketch.bindings()` lists them in the order they were made.

#ref-table(
  columns: (auto, 1fr),
  header: ("Call", "What it does"),
  [`bind_dimension(sketch, doc, c)`], [Adds dimension `c` (a `Distance`, `Radius` or `Diameter`) to the sketch, and a model parameter with the same value and unit `mm` to the document; returns the constraint id and the parameter's name. A diameter's parameter holds the diameter. On error neither changes.],
  [`apply_params(sketch, doc)`], [Copies each bound parameter's current value into its dimension. Checks every value first, so on error nothing changes. Solve afterwards to move the geometry.],
  [`rename_bound(sketch, old, new)`], [Points the sketch's bindings from `old` to `new`; returns how many changed. Call it with every `doc.rename_param`.],
  [`sketch.set_dimension_value(id, value)`], [Changes one dimension's value directly. Value finite and > 0.],
)

== Expressions <sec-ref-expr>

#idx("expression")
`swx_core::expr` reads and works out expressions (@sec-expressions). `parse(text)` turns the text into an `Expr`; `eval(&expr, &resolve, default_length)` works it out, asking `resolve(name)` for each name's value. `expr.names()` lists the names an expression uses, each once, in first-use order. The result is a `Quantity`: a `value` in millimeters and radians and a `dim` (its powers of length and angle). `q.in_unit(unit)` converts the value for display. `default_length` is the unit for showing lengths; it doesn't change the result.

#ref-table(
  columns: (auto, 1fr),
  header: ("Part", "Rule"),
  [Numbers], [`5`, `2.5`, `.5`, `1e-3`. A bare number has no unit.],
  [Names], [A letter or `_`, then letters, digits or `_`, as for parameters.],
  [Operators], [`+ - * /`, unary `-`, and `( )`. `*` and `/` before `+` and `-`; left to right otherwise.],
  [Units], [Written straight after a number or a `)`: `5 mm`, `(2 + 3) mm`. After a name, a word is not a unit.],
  [`+` and `-`], [Both sides must have the same dimension: two lengths, two angles, or two bare numbers.],
  [`*` and `/`], [Any dimensions; they multiply or divide. A length times a length is an area.],
  [Limits], [At most 1024 pieces and 64 levels of brackets or minus signs.],
)

#ref-table(
  columns: (auto, auto, auto),
  header: ("Unit", "Measures", "Size"),
  [`mm`], [length], [1 mm],
  [`cm`], [length], [10 mm],
  [`m`], [length], [1000 mm],
  [`in`], [length], [25.4 mm],
  [`ft`], [length], [304.8 mm],
  [`deg`], [angle], [π/180 rad],
  [`rad`], [angle], [1 rad],
)

== File formats and units

#ref-table(
  columns: (auto, auto, 1fr),
  header: ("Format", "Read / write", "Notes"),
  [STEP (`.step`, `.stp`), ISO 10303-21], [Both], [Solids only. Written in millimeters.],
  [PNG (`.png`)], [Write], [Pictures from `swx-qa render` and `swx-qa sketch`.],
  [Saulitwerx part (`.sltwx`)], [Not yet], [Planned: the part with its full history.],
)

#ref-table(
  columns: (auto, 1fr),
  header: ("Quantity", "Unit"),
  [Length], [millimeter (mm)],
  [Area, volume], [mm², mm³],
  [Angle], [degrees for feature parameters such as a revolve's sweep; radians for arc angles in sketches],
)

== Keyboard and mouse

There are no keyboard or mouse controls yet, because there is no app window. They arrive with the desktop app and will be listed here.
