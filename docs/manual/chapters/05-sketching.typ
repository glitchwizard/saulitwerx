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

#image-equation(
  (
    // cargo run -p swx-qa --example plate_sketch_plot -- docs/manual/img 600x420
    ("img/sketch-rough.png", "Before: the rough drawing"),
    ("img/sketch-sized.png", "After: solved"),
  ),
  ("→",),
  caption: [The sample sketch before and after solving. @sec-picture-sketch shows how to draw pictures like these.],
) <fig-sketch-solve>

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

== Check how fully a sketch is defined <sec-check-dof>

A sketch that solves is not necessarily finished: parts of it may still be free to move, or some rules may repeat each other. A #idx("degree-of-freedom report")degree-of-freedom report (@sec-dof) tells you which. In this task you run a program that builds the plate sketch one group of rules at a time and checks it after each.

#task-intro(
  goal: [Watch the degrees of freedom of a sketch fall to zero, then see what a repeated or contradicting dimension looks like.],
  needs: [Saulitwerx built (@ch-install). OpenCascade is not needed.],
  time: [5 minutes.],
)

#steps(
  [Run the sample:
  #term-block("cargo run -p swx-sketch --example sketch_dof")
  It prints one line per stage, with the number of equations the rules make, the degrees of freedom left, and the state:
  #term-block(label: "Output", "1 rough drawing            equations  0  dof 19  UnderConstrained\n2 + coincident x4          equations  8  dof 11  UnderConstrained\n3 + horizontal, vertical   equations 12  dof  7  UnderConstrained\n4 + fixed corner           equations 14  dof  5  UnderConstrained\n5 + two distances          equations 16  dof  3  UnderConstrained\n  solve: Converged\n6 + hole, then solve       equations 19  dof  0  FullyConstrained\n  solve: Converged\n7 + distance top 40        equations 20  dof  0  OverConstrained\n                           redundant: distance top 40\n  solve: Stagnated\n7 + distance top 45        equations 20  dof  0  Conflicting\n                           redundant: distance top 45")],
  [Read stages 1 to 6. The four lines and the circle start with 19 degrees of freedom. Each group of rules takes some away, and after the hole is placed and sized none are left: `FullyConstrained`.],
  [Read the first stage 7. A third distance, 40 mm on the top line, repeats what the bottom line and the horizontal and vertical rules already say. The solver still succeeds, but the report says `OverConstrained` and names the repeated rule.],
  [Read the second stage 7. With 45 mm on the top line instead, no rectangle can meet every rule. The solver stops with `Stagnated`, and the report says `Conflicting`.],
)

#result[You can tell an unfinished sketch (`UnderConstrained`, with the number of degrees of freedom still open) from a finished one (`FullyConstrained`), and spot a repeated or contradicting rule.]

=== How the check works

The program is `crates/swx-sketch/examples/sketch_dof.rs`. One call makes the report:

```rust
use swx_sketch::dof::{analyze, DofOptions};
use swx_sketch::solver::Residuals;

let report = analyze(&s, s.params(), DofOptions::default())?;
println!("dof {} {:?}", report.dof, report.state);
```

The report holds `dof`, the `state`, and `redundant`: the numbers of the equations that repeat earlier ones. Each constraint adds one or two equations, in the order you add the constraints: coincident and fixed add two, the others one. To turn an equation number back into a constraint, the sample notes how many equations each `add_constraint` call added, by reading `s.num_residuals()` before and after. @sec-ref-dof lists every field and setting.

#tip(title: [Solve first, then check])[The difference between over-constrained and conflicting is whether the rules are met. Run the check after `solve`: on a rough drawing that hasn't been solved yet, a harmless repeated rule can show as `Conflicting`.]

#caution(title: [Only the later rule is named])[When two rules repeat each other, the report names the one you added *later*, because the earlier one was still independent when it arrived. The fix may be to delete either one.]

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
- *Rules that fight each other.* Two fixed points 30 mm apart with a distance of 40 mm between them can't both hold. The solver then stops with `solved: false` and a reason of `Stagnated`, and moves the points as close to meeting the rules as it can. A degree-of-freedom check then says `Conflicting` and gives the number of the clashing equation (@sec-check-dof).
- *Too few rules.* A sketch can solve and still be under-constrained: the solver keeps unconstrained pieces near where you drew them. Check that the degrees of freedom reach zero before you rely on the sizes.
