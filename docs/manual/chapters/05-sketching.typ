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
A `PointRef` names a point the constraint acts on: `Start(line)`, `End(line)`, `Center(circle)`, `ArcStart(arc)`, `ArcEnd(arc)` or `Point(point)`. @sec-ref-sketch lists every entity and constraint with its exact form.

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
  [Read the second stage 7. With 45 mm on the top line instead, no rectangle can meet every rule. The solver stops with `Stagnated`, and the report says `Conflicting` and names the 45 mm distance.],
)

#result[You can tell an unfinished sketch (`UnderConstrained`, with the number of degrees of freedom still open) from a finished one (`FullyConstrained`), and spot a repeated or contradicting rule.]

=== How the check works

The program is `crates/swx-sketch/examples/sketch_dof.rs`. One call on the sketch makes the report:

```rust
use swx_sketch::dof::DofOptions;

let r = s.dof(DofOptions::default())?;
println!("dof {} {:?}", r.report.dof, r.report.state);
for id in &r.redundant_constraints {
    println!("redundant: {}", names[id.0 as usize]);
}
```

`r.report` holds `dof` and the `state`. `r.redundant_constraints` lists the repeated rules as #idx("constraint id")constraint ids: the number `add_constraint` gave each one, counting from 0 in the order you added them. The sample keeps a list of names in the same order, so `names[id.0]` prints the rule in words. A rule that adds two equations, such as a repeated coincident, is listed once. @sec-ref-dof lists every field and setting.

#tip(title: [Solve first, then check])[The difference between over-constrained and conflicting is whether the rules are met. Run the check after `solve`: on a rough drawing that hasn't been solved yet, a harmless repeated rule can show as `Conflicting`.]

#caution(title: [Only the later rule is named])[When two rules repeat each other, the report names the one you added *later*, because the earlier one was still independent when it arrived. The fix may be to delete either one.]

== Change a dimension

#steps(
  [Open `crates/swx-sketch/examples/plate_sketch.rs` in a text editor.],
  [Find `value: 40.0` and change it to `value: 60.0`. Save the file.],
  [Run the sample again (step 1 above). The bottom and top lines now report `length 60.00 mm`; the right line moves to x = 60. The hole stays at (20, 12.5), because it is fixed there, not centered.],
  [Change the value back to `40.0` when you are done.],
)

#tip[The hole stayed put because it is fixed at (20, 12.5). To keep it centered whatever the width, use midpoint instead (@sec-centered).]

== Keep a hole centered <sec-centered>

A good sketch says what you *mean*. If the hole belongs in the middle of the plate, the sketch should say "in the middle", not "at x = 20". Then a wider plate keeps its hole centered with no extra work. The tool for this is the #term[midpoint] constraint, which keeps a point exactly halfway along a line.

#task-intro(
  goal: [Center a hole with midpoint constraints, then change the plate's width and watch the hole follow.],
  needs: [Saulitwerx built (@ch-install). OpenCascade is not needed.],
  time: [5 minutes.],
)

#steps(
  [Run the centered-plate sample:
  #term-block("cargo run -p swx-sketch --example centered_plate")
  It solves the sketch, checks its degrees of freedom, and prints the sizes and the region an extrude would use:
  #term-block(label: "Output", "solved: true (Converged), dof 0 FullyConstrained\nbottom length 40.00 mm\nright  length 25.00 mm\nhole   center (20.00, 12.50)  diameter 10.00 mm\nregions: 1\n  region 0: outer edges 4, holes 1, area 921.46 mm²\nopen edges: 0")],
  [Check the first line. `FullyConstrained` with `dof 0` means every piece is pinned down, including the hole, even though no rule gives the hole's coordinates.],
  [Run it again for a plate 60 mm wide. The two numbers after `--` are the width and the height:
  #term-block("cargo run -p swx-sketch --example centered_plate -- 60 25")
  #term-block(label: "Output", "solved: true (Converged), dof 0 FullyConstrained\nbottom length 60.00 mm\nright  length 25.00 mm\nhole   center (30.00, 12.50)  diameter 10.00 mm\nregions: 1\n  region 0: outer edges 4, holes 1, area 1421.46 mm²\nopen edges: 0")
  The hole has moved to x = 30, the middle of the new width.],
)

#result[A plate whose hole stays in the middle whatever its width or height, and a sketch that is fully constrained.]

#image-equation(
  (
    // cargo run -p swx-qa --example constraint_plots -- docs/manual/img 600x420
    ("img/sketch-centered-40.png", "40 mm wide"),
    ("img/sketch-centered-60.png", "60 mm wide"),
  ),
  ("→",),
  caption: [The same sketch with only the width changed. The red dots at the middle of the bottom and left sides are the helper points that center the hole.],
) <fig-centered>

=== How the centered plate is built

The program is `crates/swx-sketch/examples/centered_plate.rs`. It starts from the same rough drawing as the plate sketch, and changes two things.

*Square corners without four direction rules.* Only the bottom is horizontal. The left side is #term[perpendicular] to it, and each side is #term[parallel] to the one opposite:

```rust
s.add_constraint(Constraint::Horizontal(bottom))?;
s.add_constraint(Constraint::Perpendicular(bottom, left))?;
s.add_constraint(Constraint::Parallel(top, bottom))?;
s.add_constraint(Constraint::Parallel(right, left))?;
```

*Center the hole with two helper points.* A point entity sits at the middle of the bottom side, and another at the middle of the left side. The hole's center is lined up straight above the first and level with the second:

```rust
let mid_bottom = s.add(Geometry::Point(p(19.0, 0.0)?))?;
let mid_left = s.add(Geometry::Point(p(0.0, 12.0)?))?;
s.add_constraint(Constraint::Midpoint { point: PointRef::Point(mid_bottom), line: bottom })?;
s.add_constraint(Constraint::Midpoint { point: PointRef::Point(mid_left), line: left })?;
s.add_constraint(Constraint::VerticalPoints(
    PointRef::Point(mid_bottom), PointRef::Center(hole)))?;
s.add_constraint(Constraint::HorizontalPoints(
    PointRef::Point(mid_left), PointRef::Center(hole)))?;
```

Helper points are safe to add: they are not part of any outline, so the profile ignores them.

#tip(title: [Why not a diagonal?])[Drawing the rectangle's diagonal and putting the hole's center at its midpoint would also work for the solver. But the diagonal would cut through the hole and the plate, and profiles can't be found from pieces that cross (@sec-profiles). Helper points avoid that.]

#idx("symmetric")
The #term[symmetric] constraint is the other way to say "centered": `Symmetric { a, b, line }` makes points `a` and `b` mirror images across `line`. It suits pairs, such as two holes placed evenly either side of a center line.

== Draw a slot <sec-slot>

A slot is a rounded-end opening, like the hole a bolt slides along. Its outline is two straight sides and two half circles, and the sides must run smoothly into the curves, with no corner. Three constraints say that. Coincident joins each side's ends to the arcs' ends, so the outline closes. The #term[tangent] constraint makes a line touch a circle or arc without cutting into it, so there is no corner where they meet. The #term[equal] constraint keeps both ends the same size, so you set the radius only once.

#task-intro(
  goal: [Close a roughly drawn slot with coincident, tangent and equal constraints, check that it is fully defined, and find its region.],
  needs: [Saulitwerx built (@ch-install). OpenCascade is not needed.],
  time: [5 minutes.],
)

#steps(
  [Run the slot sample:
  #term-block("cargo run -p swx-sketch --example slot_sketch")
  It prints the slot twice: as drawn, then after solving.
  #term-block(label: "Output", "1 as drawn\n  left   arc center (  0.00,  0.00)  radius 4.00 mm  from   90° to  260°\n  right  arc center ( 28.00,  1.00)  radius 6.00 mm  from  -90° to   90°\n  bottom (  0.00, -4.00) -> ( 28.00, -5.00)\n  regions: 0\n  open edges: [0, 1, 2, 3]\n2 solved: true\n  dof 0 FullyConstrained\n  left   arc center (  0.00,  0.00)  radius 5.00 mm  from   90° to  270°\n  right  arc center ( 30.00,  0.00)  radius 5.00 mm  from  -90° to   90°\n  bottom (  0.00, -5.00) -> ( 30.00, -5.00)\n  regions: 1\n    outer edges 4, holes 0, area 378.54 mm²\n  open edges: []")],
  [Read stage 1. The slot was drawn carelessly on purpose: the ends have radii 4 and 6, the slot is tilted, and the left arc stops at 260°, 10° short of the bottom side (@fig-slot). Because of that one gap the outline doesn't close, so there is no region and all four pieces are listed as open edges.],
  [Read stage 2. Both radii are now 5 mm, the centers are 30 mm apart and level, and the left arc runs the full half circle from 90° to 270°. The solver turned the arc's end until it met the bottom side.],
  [Read `dof 0 FullyConstrained`. Every piece of the slot is pinned down: its size, its shape and where it sits. Nothing is left to chance.],
  [Read the region. The closed slot encloses one region of 378.54 mm²: a 30 × 10 mm rectangle plus two half circles of radius 5 mm.],
)

#result[A fully defined 30 mm slot with 5 mm round ends, and one region ready to become a solid.]

#image-equation(
  (
    // cargo run -p swx-qa --example constraint_plots -- docs/manual/img 600x420
    ("img/sketch-slot-rough.png", "As drawn"),
    ("img/sketch-slot.png", "Solved"),
  ),
  ("→",),
  marks: (((18%, 64%, [gap]),), ()),
  caption: [The slot before and after solving. As drawn, the ends differ in size, the slot is tilted, and the left arc stops short of the bottom side (ringed). Solved, it is level, with equal 5 mm ends joined all the way round.],
) <fig-slot>

=== How the slot is built

The program is `crates/swx-sketch/examples/slot_sketch.rs`. An arc's two ends are named with `PointRef::ArcStart(arc)` and `PointRef::ArcEnd(arc)`. An arc always runs counterclockwise from its start to its end, so the left arc, drawn from the top (90°) round to the bottom (270°), ends where the bottom side starts:

```rust
for (a, b) in [
    (PointRef::Start(bottom), PointRef::ArcEnd(left)),
    (PointRef::End(bottom), PointRef::ArcStart(right)),
    (PointRef::Start(top), PointRef::ArcEnd(right)),
    (PointRef::End(top), PointRef::ArcStart(left)),
] {
    s.add_constraint(Constraint::Coincident(a, b))?;
}
for (line, arc) in [(bottom, left), (bottom, right), (top, left), (top, right)] {
    s.add_constraint(Constraint::Tangent(line, arc))?;
}
s.add_constraint(Constraint::Equal(left, right))?;
s.add_constraint(Constraint::Radius { geom: left, value: 5.0 })?;
```

Then the bottom side is made horizontal, the left center is fixed at the origin, and the centers are set 30 mm apart. That is all the slot needs.

#idx("arc ends")
#tip(title: [Which end is which])[Picture walking round the arc counterclockwise, the way the angles grow. Where you set off is `ArcStart`; where you stop is `ArcEnd`. If you join the wrong end, the solver still tries to obey, and you get a different shape from the one you meant. Check the angles in the output, or plot the sketch (@sec-picture-sketch).]

#tip[A tangent line stays on the side of the circle it started on, and two tangent circles stay touching on the outside or the inside, whichever they were closer to when you added the constraint. Draw roughly the right arrangement and the solver keeps it. When a line's end is joined to an arc's end, tangent there means the line meets the arc square to its radius, which is exactly "no corner".]

== Find the region to extrude <sec-profiles>

Before a sketch can become a solid, Saulitwerx has to know which areas it encloses (@sec-profile). Both samples above end by asking for exactly that. This section shows the call, and how to read the answer.

#task-intro(
  goal: [Get the regions of a sketch, with their holes and areas, and find the open edges when a sketch doesn't close.],
  needs: [The two samples above, run once each.],
  time: [5 minutes.],
)

#steps(
  [Look at the end of the centered-plate output (@sec-centered): `regions: 1`, `holes 1`, `area 921.46 mm²`. The rectangle is the region's outside edge and the circle its hole; the area is the rectangle's 1000 mm² minus the hole's 78.54 mm².],
  [Look at stage 2 of the slot output (@sec-slot): one region with `outer edges 4` (two lines, two arcs) and no holes.],
  [Look at stage 1: `regions: 0` and `open edges: [0, 1, 2, 3]`. The numbers are the pieces in the order they were added to the sketch: 0 and 1 are the arcs, 2 and 3 the lines. When a sketch has no region, the open edges show where to look for the gap.],
)

#result[You can tell whether a sketch is ready to become a solid, and how much area it encloses. @ch-extrude shows how an outline like this becomes a solid.]

=== How it works

One call returns every region and the open edges:

```rust
use swx_sketch::profile::{profile_report, ProfileOptions};

let profile = profile_report(&s, &ProfileOptions::default())?;
for r in &profile.regions {
    println!("holes {}, area {:.2} mm²", r.holes.len(), r.area());
}
println!("open edges: {:?}", profile.open_edges);
```

`profiles(&s, &options)` returns only the regions. Ends of pieces closer together than a small tolerance count as joined; @sec-ref-profile lists the setting, every field, and the rules for loops inside loops.

#caution(title: [Crossing pieces stop the search])[If two pieces cross, overlap or touch away from their ends, `profile_report` returns an error naming both, such as `geometry 2 and 5 intersect`, and no regions. Saulitwerx doesn't split pieces where they cross. Move or shorten one of them so they meet only end to end.]

== Let Saulitwerx add constraints as you draw <sec-infer>

Most of a sketch's constraints say what your drawing already almost shows: this corner joins that one, this line is level, this circle touches that line. Adding each of them by hand is slow. #term[Inference] does it for you. After you add a piece, Saulitwerx compares it with everything already in the sketch, suggests the constraints it nearly satisfies, adds them and solves. It never adds a dimension: sizes stay yours to set.

#image-equation(
  (
    // cargo run -p swx-qa --example inference_plots -- docs/manual/img 600x420
    ("img/sketch-infer-drawn.png", "As drawn"),
    ("img/sketch-infer.png", "After inference"),
  ),
  ("→",),
  marks: (((10%, 84%, [gap]),), ()),
  caption: [The sketch before and after inference. As drawn, the sides lean slightly and the left side stops short of the bottom-left corner (ringed). After inference and solving, the sides are level and upright, every corner is joined, and the circle touches the top and bottom.],
) <fig-infer>

#task-intro(
  goal: [Draw a rough rectangle and a circle one piece at a time, and see which constraints inference adds after each.],
  needs: [Saulitwerx built (@ch-install). OpenCascade is not needed.],
  time: [5 minutes.],
)

#steps(
  [Run the inference sample:
  #term-block("cargo run -p swx-sketch --example inferred_sketch")
  It draws four sides and then a circle, and after each one lists what inference suggested.
  #term-block(label: "Output", "bottom: 1 suggested\n  horizontal: bottom\nright: 2 suggested\n  coincident: start of right = end of bottom\n  vertical: right\ntop: 2 suggested\n  coincident: start of top = end of right\n  horizontal: top\nleft: 2 suggested\n  coincident: start of left = end of top\n  vertical: left\n  (missed the first corner by 1.31 mm; the default point_tol is 0.047 mm)\nagain with point_tol 2 mm:\nleft: 3 suggested\n  coincident: start of left = end of top\n  coincident: end of left = start of bottom\n  vertical: left\ncorners: (0.13, 0.52) (39.90, 0.52) (39.90, 25.07) (0.13, 25.07)\ncircle: 2 suggested\n  tangent: circle and bottom\n  tangent: circle and top")],
  [Read the first three pieces. The bottom was drawn 0.6° off level, so it got horizontal. The right side starts a few hundredths of a millimeter from the bottom's end, so its start was joined to that end, and it got vertical. The top went the same way. No piece was made parallel or perpendicular to another: once a line is horizontal or vertical, those would only repeat the rule.],
  [Read the first try at the left side. Its start was joined to the top, but its end stopped 1.31 mm short of the first corner. That is far more than the default reach of 0.047 mm, so that corner was left open.],
  [Read the second try. With a reach of 2 mm the gap was closed, and the rectangle is complete. Its corners are now exactly square (@fig-infer).],
  [Read the circle. It was drawn 0.01 mm short of touching the top and bottom, so it got tangent to both.],
)

#result[A closed rectangle with a circle touching its top and bottom, held by ten constraints you didn't have to add yourself.]

Inference doesn't make a sketch fully defined. This rectangle has no sizes yet, and its corner is not fixed, so it still has degrees of freedom (@sec-check-dof). Add a fixed point and the dimensions next.

=== How the sample is built

The program is `crates/swx-sketch/examples/inferred_sketch.rs`. After each `add`, one call infers, adds and solves:

```rust
use swx_sketch::infer::{add_inferred, infer, InferOptions};

let right = s.add(Geometry::Line(Line::new(start, end)?))?;
let opts = InferOptions::for_sketch(&s);
let suggested = infer(&s, right, &opts)?;      // look only
let added = add_inferred(&mut s, right, &opts)?; // add and solve
```

`infer` only looks and changes nothing, so you can show the suggestions first and let the user choose. `add_inferred` adds every suggestion and solves. Two settings decide how near is near enough. `point_tol` is the reach for joining points and for tangency; by default it is a thousandth of the sketch's overall size, its diagonal. `angle_tol_deg` is the reach for directions; by default 1°. @sec-ref-infer gives the full rules.

#tip(title: [Draw from where you stopped])[Inference joins only points that are very close by default. Start each new piece where the last one ended, as precisely as you can, and the corners join themselves. When a drawing is coarser, widen `point_tol`, but not so far that it joins points you meant to keep apart.]

#caution(title: [It adds what it sees])[A line drawn half a degree off level becomes level, even if you meant it to lean, and a constraint can't be taken out of a sketch yet. When that matters, call `infer` first, and add only the suggestions you want with `add_constraint`. A constraint already in the sketch is never suggested again.]

== Common mistakes

- *A value of zero or less.* Distance, radius and diameter must be greater than zero. Adding `value: 0.0` fails with `value must be > 0, got 0`.
- *The wrong kind of point.* `PointRef::Start` and `End` work only on lines, `ArcStart` and `ArcEnd` only on arcs, and `Center` only on circles and arcs. Using `Center` on a line fails with `geometry 0 has a different kind`.
- *Rules that fight each other.* Two fixed points 30 mm apart with a distance of 40 mm between them can't both hold. The solver then stops with `solved: false` and a reason of `Stagnated`, and moves the points as close to meeting the rules as it can. A degree-of-freedom check then says `Conflicting` and names the clashing constraint (@sec-check-dof).
- *The wrong kind of entity.* Parallel and perpendicular take two lines. Equal takes two lines or two round pieces (circles and arcs), not one of each. Tangent takes a line and a round piece, or two round pieces, never two lines. Each of these fails with `geometry N has a different kind`.
- *An outline with a gap.* Pieces that look joined but aren't leave no region. Join every corner with coincident, including where a line meets an arc (@sec-slot).
- *Expecting inference to set sizes.* Inference adds coincident, horizontal, vertical, parallel, perpendicular and tangent only. Add distances, radii and diameters yourself (@sec-infer).
- *Too few rules.* A sketch can solve and still be under-constrained: the solver keeps unconstrained pieces near where you drew them. Check that the degrees of freedom reach zero before you rely on the sizes.
