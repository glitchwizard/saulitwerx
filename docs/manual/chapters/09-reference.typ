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

== Build options

#ref-table(
  columns: (auto, 1fr),
  header: ("Command", "What it builds"),
  [`cargo test --workspace`], [Everything without OpenCascade, and runs every test.],
  [`cargo build --release -p swx-qa --features occt`], [The `swx-qa` tool with STEP input, at `target/release/swx-qa`.],
  [`cargo run -p swx-sketch --example plate_sketch`], [Runs the sample sketch (@ch-sketching).],
  [`cargo run -p swx-kernel --features occt --example plate_with_hole -- <file.step>`], [Builds the plate and saves it (@ch-solids). The file defaults to `plate.step`.],
  [`cargo run -p swx-kernel --features occt --example plate_stages -- <dir>`], [Saves the block, the cylinder and the plate as three STEP files in `<dir>`.],
  [`cargo run -p swx-core --example feature_history`], [Runs the history sample (@ch-history).],
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

== Sketch entities and constraints <sec-ref-sketch>

#ref-table(
  columns: (auto, 1fr),
  header: ("Entity", "Defined by"),
  [`Point`], [x, y.],
  [`Line`], [A start point and an end point.],
  [`Circle`], [A center point and a radius > 0.],
  [`Arc`], [A center, a radius > 0, and start and end angles in radians, running counter-clockwise; the end angle must be after the start.],
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
)

A point `p` is written `PointRef::Point(id)` for a point entity, `Start(id)` or `End(id)` for a line's ends, or `Center(id)` for a circle's or arc's center.

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

== File formats and units

#ref-table(
  columns: (auto, auto, 1fr),
  header: ("Format", "Read / write", "Notes"),
  [STEP (`.step`, `.stp`), ISO 10303-21], [Both], [Solids only. Written in millimeters.],
  [PNG (`.png`)], [Write], [Pictures from `swx-qa render`.],
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
