#import "../template.typ": *

= Sketching <ch-sketching>

#for-developers[Until the desktop app's sketch mode arrives, sketches are written as a short Rust program against the `swx-sketch` library. This chapter runs one that ships with Saulitwerx.]

== Solve a sketch

The sketch for the plate of @ch-first is a 40 × 25 mm rectangle with a 10 mm circle in the middle. In this task you run a program that draws that sketch *roughly*, adds the constraints and dimensions, and asks the solver for the exact shape.

#task-intro(
  goal: [Run the sample sketch, read the solver's report, and see the rough shape become exact.],
  needs: [Saulitwerx built (@ch-install). OpenCascade is not needed.],
  time: [5 minutes.],
)

#steps(
  [Run the sample sketch:
  #term-block("cargo run -p swx-sketch --example plate_sketch")
  It prints the solver's report, then each line and the circle:
  #term-block(label: "Output", "solved: true (Converged, 3 iterations, residual 5.3e-14)\nbottom (  0.00,   0.00) -> ( 40.00,   0.00)  length 40.00 mm\nright  ( 40.00,   0.00) -> ( 40.00,  25.00)  length 25.00 mm\ntop    ( 40.00,  25.00) -> (  0.00,  25.00)  length 40.00 mm\nleft   (  0.00,  25.00) -> (  0.00,   0.00)  length 25.00 mm\nhole   center (20.00, 12.50)  diameter 10.00 mm")],
  [Read the first line. `solved: true` and `Converged` mean the solver found a shape that meets every rule. `residual` is how far off it still is, here about 0.00000000000005 mm: exact for any practical purpose.],
  [Compare the result with the rough drawing in the program (below). The bottom line was drawn 38 mm long and tilted; it is now exactly 40 mm and level.],
)

#result[A fully sized, square-cornered sketch: a 40 × 25 mm rectangle with its lower-left corner at the origin, and a 10 mm circle centered at (20, 12.5).]

== How the sample sketch is built

The program is `crates/swx-sketch/examples/plate_sketch.rs`. It works in four stages, the same ones you will use in the app. The code below is shortened from the file.

*1. Draw roughly.* Four lines and a circle, deliberately off-size:

```rust
let bottom = s.add(Geometry::Line(Line::new(p(0.0, 0.0)?, p(38.0, 1.0)?)?))?;
let right = s.add(Geometry::Line(Line::new(p(38.0, 1.0)?, p(39.0, 22.0)?)?))?;
let top = s.add(Geometry::Line(Line::new(p(39.0, 22.0)?, p(1.0, 24.0)?)?))?;
let left = s.add(Geometry::Line(Line::new(p(1.0, 24.0)?, p(0.0, 0.0)?)?))?;
let hole = s.add(Geometry::Circle(Circle::new(p(18.0, 11.0)?, 4.0)?))?;
```

*2. Fix the shape.* Join each line's end to the next line's start with *coincident*, then make two lines horizontal and two vertical:

```rust
for (a, b) in [(bottom, right), (right, top), (top, left), (left, bottom)] {
    s.add_constraint(Constraint::Coincident(PointRef::End(a), PointRef::Start(b)))?;
}
s.add_constraint(Constraint::Horizontal(bottom))?;
s.add_constraint(Constraint::Vertical(left))?;   // and likewise top, right
```

*3. Anchor and size it.* Pin the lower-left corner to the origin with *fixed*, and set two *distances* and the hole's *diameter*:

```rust
s.add_constraint(Constraint::Fixed { point: PointRef::Start(bottom), x: 0.0, y: 0.0 })?;
s.add_constraint(Constraint::Distance {
    a: PointRef::Start(bottom), b: PointRef::End(bottom), value: 40.0 })?;
s.add_constraint(Constraint::Diameter { geom: hole, value: 10.0 })?;
```

*4. Solve.* One call moves everything into place and returns a report:

```rust
let report = s.solve(SolveOptions::default())?;
```

#idx("PointRef")
A `PointRef` names a point the constraint acts on: `Start(line)`, `End(line)`, `Center(circle)` or `Point(point)`. @sec-ref-sketch lists every entity and constraint with its exact form.

== Change a dimension

#steps(
  [Open `crates/swx-sketch/examples/plate_sketch.rs` in a text editor.],
  [Find `value: 40.0` and change it to `value: 60.0`. Save the file.],
  [Run the sample again (step 1 above). The bottom and top lines now report `length 60.00 mm`; the right line moves to x = 60. The hole stays at (20, 12.5), because it is fixed there, not centered.],
  [Change the value back to `40.0` when you are done.],
)

#tip[To keep the hole centered whatever the width, the sketch would need a *midpoint* or *symmetric* constraint. Those are next on the roadmap.]

== Common mistakes

- *A value of zero or less.* Distance, radius and diameter must be greater than zero. Adding `value: 0.0` fails with `value must be > 0, got 0`.
- *The wrong kind of point.* `PointRef::Start` and `End` work only on lines, and `Center` only on circles and arcs. Using `Center` on a line fails with `geometry 0 has a different kind`.
- *Rules that fight each other.* Two fixed points 30 mm apart with a distance of 40 mm between them can't both hold. The solver then stops with `solved: false` and a reason of `Stagnated`, and moves the points as close to meeting the rules as it can. Saulitwerx can't yet say *which* rules conflict; that is on the roadmap.
