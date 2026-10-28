#import "../template.typ": *

= Parameters and history <ch-history>

#for-developers[This chapter uses the document model, `swx-core`, and the sketcher, `swx-sketch`, from Rust. It needs no OpenCascade.]

The document model holds what makes a part parametric: its named parameters and its ordered list of features (@sec-parameter, @sec-history). It doesn't make geometry itself. Its job is to keep the history valid and to work out what must be rebuilt after a change.

== See what a change rebuilds

#task-intro(
  goal: [Build a four-feature history and ask which features a change affects.],
  needs: [Saulitwerx built (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [Run the sample:
  #term-block("cargo run -p swx-core --example feature_history")
  #term-block(label: "Output", "history:\n  0  Sketch1\n  1  Extrude1\n  2  Sketch2\n  3  Revolve1\nchange thickness -> rebuild Extrude1\nedit Sketch2     -> rebuild Sketch2, Revolve1\nadd Extrude2 with depth `height` -> error: unknown parameter `height`")],
  [Read the history. It is the table in @sec-history: two sketches, an extrude that uses `thickness`, and a revolve that uses `sweep`.],
  [Read the two rebuild lines. Changing `thickness` touches only Extrude1, the one feature that uses it. Editing Sketch2 rebuilds Sketch2 and Revolve1, which consumes it; Sketch1 and Extrude1 are left alone.],
  [Read the last line. A feature may use only parameters that exist, so adding an extrude whose depth is `height` is refused.],
)

#result[You saw Saulitwerx decide, from the history alone, the smallest set of features a change needs rebuilt.]

== How the sample is built

The program is `crates/swx-core/examples/feature_history.rs`. A `Document` holds parameters and features:

```rust
let mut doc = Document::new();
doc.set_param("thickness", 5.0);
let plate_sketch = doc.add_feature("Sketch1", FeatureKind::Sketch)?;
doc.add_feature("Extrude1", FeatureKind::Extrude {
    profile: plate_sketch,
    depth: "thickness".into(),
})?;
```

Then two questions give the rebuild lists, in history order:

```rust
doc.dirty_after_param_change("thickness");   // [Extrude1]
doc.dirty_after_feature_edit(pin_sketch)?;    // [Sketch2, Revolve1]
```

#idx("rebuild")
The features that need rebuilding are called #term[dirty]. Today's feature kinds are `Sketch`, `Extrude` (with a depth parameter) and `Revolve` (with an angle parameter, in degrees). They are records in the history only; making their geometry comes with the part features on the roadmap.

== Name and rename parameters <sec-param-table>

Every part keeps its parameters in one #idx("parameter table")parameter table. Each row has a name, a value, a unit and an optional comment, and says where the parameter came from. A #term[model parameter] belongs to a dimension and gets the next free name in the series `d0`, `d1`, `d2`, … A #term[user parameter] is one you create yourself, with a name you choose, for a value you want to reuse or change in one place.

Names like `d0` are quick to make but say nothing. Renaming `d0` to `thickness` makes the part easier to read, and every feature that uses it follows the new name, so nothing breaks.

#task-intro(
  goal: [Build a small parameter table, rename a parameter a feature uses, and see which names are refused.],
  needs: [Saulitwerx built (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [Run the sample:
  #term-block("cargo run -p swx-core --example parameter_table")
  #term-block(label: "Output", "model parameters: d0, d1, d2\nafter renaming d0 to thickness:\n  name       kind   value  unit  comment\n  thickness  model      5  mm\n  d1         model     40  mm\n  d2         model     10  mm\n  clearance  user     0.2  mm    gap around the pin\nExtrude1 depth reads `thickness`\nnext model parameter: d3\nadd `2nd_hole` -> error: invalid parameter name `2nd_hole`: must start with a letter or `_` and contain only letters, digits and `_`\nrename d1 to thickness -> error: a parameter named `thickness` already exists")],
  [Read the first line. Three dimensions of a plate (its thickness, width and hole size) were given the model names `d0`, `d1` and `d2`, in the order they were made.],
  [Read the table. `d0` is now called `thickness`; it is still a model parameter. `clearance` is a user parameter, with a comment saying what it is for. The rows stay in the order they were made.],
  [Read the line about Extrude1. The extrude was made with depth `d0`. After the rename it reads `thickness`, with no change to the extrude itself.],
  [Read `next model parameter: d3`. The name `d0` became free when it was renamed, but it is not handed out again. A model name always means one parameter, for the life of the part.],
  [Read the last two lines. A name may not start with a digit, and two parameters may not share a name. In both cases the table is left as it was.],
)

#result[You have a parameter table with meaningful names, and a feature that kept working through a rename.]

#tip(title: [Name what you will change])[Rename the parameters you expect to edit later, such as a thickness or a hole size. Leave the rest as `d0`, `d1`, …]

=== How the sample is built

The program is `crates/swx-core/examples/parameter_table.rs`. Each call works on the document:

```rust
let depth = doc.add_model_param(5.0, "mm");        // "d0"
doc.add_user_param("clearance", 0.2, "mm")?;
doc.set_param_comment("clearance", "gap around the pin")?;
doc.rename_param(&depth, "thickness")?;
for p in doc.parameters() { /* name, kind, value, unit, comment */ }
```

The unit is free text, such as `"mm"` or `"deg"`, and may be empty; Saulitwerx doesn't check it yet. `set_param`, used in the history sample above, changes a parameter's value; given a new name, it makes a user parameter with no unit. @sec-ref-params lists every call.

*In Saulitwerx today:* parameters hold plain numbers. You can work out an expression such as `d1 / 2 + 5 mm` (@sec-expressions), but you can't yet store one in the table.

== Drive a sketch with parameters <sec-drive-sketch>

A sketch dimension can be tied to a parameter, so the parameter table becomes the place where you change the part's sizes. Each tied dimension makes its own model parameter, `d0`, `d1`, … in the order you add them. Change the parameter, and the dimension follows the next time you apply the table and solve. The tie is called a #term[binding].

#task-intro(
  goal: [Bind a rectangle's width and height to parameters, rename one, change it, and see the sketch follow.],
  needs: [Saulitwerx built (@ch-install). OpenCascade is not needed.],
  time: [5 minutes.],
)

#steps(
  [Run the sample:
  #term-block("cargo run -p swx-sketch --example driven_plate")
  #term-block(label: "Output", "dimension 40 mm -> parameter d0\ndimension 25 mm -> parameter d1\nsolved: 40.00 × 25.00 mm\nrenamed d0 to width (1 dimension follows)\nwidth = 60: 60.00 × 25.00 mm\nrename d1 without rename_bound -> error: a sketch dimension is bound to unknown parameter `d1`\nheight = 0 -> error: value must be > 0, got 0\nsketch still 60.00 × 25.00 mm")],
  [Read the first two lines. The width dimension, 40 mm, made parameter `d0`; the height, 25 mm, made `d1`.],
  [Read `renamed d0 to width`. The parameter got a meaningful name, and the sketch was told, so its width dimension now reads `width`.],
  [Read `width = 60`. Setting `width` to 60 and applying the table made the rectangle 60 mm wide. The height stayed 25 mm.],
  [Read the last three lines. Renaming `d1` in the table alone left the sketch looking for `d1`, which no longer exists. A height of 0 was refused, and the sketch kept its size.],
)

#result[A rectangle whose size you change from the parameter table.]

=== How the sample is built

The program is `crates/swx-sketch/examples/driven_plate.rs`. Instead of `add_constraint`, a dimension goes in with `bind_dimension`, which also adds its parameter:

```rust
use swx_sketch::params::{apply_params, bind_dimension, rename_bound};

let (_, name) = bind_dimension(&mut s, &mut doc, Constraint::Distance {
    a: PointRef::Start(bottom), b: PointRef::End(bottom), value: 40.0 })?;
// name == "d0"
doc.rename_param("d0", "width")?;
rename_bound(&mut s, "d0", "width");
doc.set_param("width", 60.0);
apply_params(&mut s, &doc)?;              // copy the values into the sketch
s.solve(SolveOptions::default())?;        // then move the geometry
```

#caution(title: [Rename in both places])[The document and the sketch keep their own lists. `doc.rename_param` renames the parameter and the features that use it, but not the sketch's bindings. Call `rename_bound` with the same two names straight after, or `apply_params` fails with `a sketch dimension is bound to unknown parameter`.]

#tip[Only distance, radius and diameter can be bound. A diameter's parameter holds the diameter, not the radius. `apply_params` checks every value before it changes anything, so one bad value leaves the whole sketch as it was.]

== Work out an expression <sec-expressions>

An expression lets one size depend on others: a hole that is always half the plate's width plus 5 mm is `d1 / 2 + 5 mm`. Saulitwerx reads the formula, looks up each name, and checks the units as it goes. Adding a length to an angle is an error, but a length times a length is an area.

#task-intro(
  goal: [Work out six expressions from a small parameter table, and see each error an expression can give.],
  needs: [Saulitwerx built (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [Run the sample:
  #term-block("cargo run -p swx-core --example expressions")
  The table holds `d0` = 5 mm, `d1` = 40 mm, `wall` = 2.5 mm and `angle` = 30 deg.
  #term-block(label: "Output", "d1 / 2 + 5 mm        = 25.00 mm\n(d1 - 2 * wall) / 3  = 11.67 mm\n2 in + d0            = 55.80 mm\n90 deg - angle       = 60.00 deg\nd1 * d0              = 200.00 mm²\nd1 / d0              = 8.00 (unitless)\nd1 + 5 deg\n  error: dimension mismatch at byte 3: cannot apply `+` to length and angle\n5 mmm\n  error: unknown unit `mmm` at byte 2 (expected mm, cm, m, in, ft, deg or rad)\nd9 * 2\n  error: unknown name `d9` at byte 0\nd1 / (d0 - 5 mm)\n  error: division by zero at byte 3\nd1 / (2\n  error: syntax error at byte 7: expected `)`")],
  [Read the first line. Half of 40 mm is 20 mm, plus 5 mm is 25 mm. Multiplication and division come before addition and subtraction, as in school arithmetic; brackets change the order, as in the second line.],
  [Read `2 in + d0`. Units can be mixed: 2 inches is 50.8 mm, plus 5 mm. Results are always worked out in millimeters and radians.],
  [Read the next three lines. An angle minus an angle is an angle. A length times a length is an area, in mm². A length divided by a length is a plain number with no unit.],
  [Read the errors. Each one says what is wrong and where, counting characters from 0: in `d1 + 5 deg`, byte 3 is the `+`.],
)

#result[You can work out a size from other sizes, in any of seven units, and trust that mixed-up units are caught.]

=== How the sample is built

The program is `crates/swx-core/examples/expressions.rs`. `parse` reads the text once; `eval` works it out, asking a function you supply for each name's value:

```rust
use swx_core::expr::{eval, parse, Quantity, Unit};

let resolve = |name: &str| -> Option<Quantity> {
    let p = doc.parameters().iter().find(|p| p.name == name)?;
    let unit = Unit::from_suffix(&p.unit)?;
    Some(Quantity::new(p.value * unit.factor(), unit.dim()))
};
let q = eval(&parse("d1 / 2 + 5 mm")?, &resolve, Unit::Mm)?;
// q.value == 25.0 (mm), q.dim == Dim::LENGTH
```

The parameter table stores a unit as plain text, so the sample turns it into a `Unit` itself. @sec-ref-expr lists the rules and every unit.

#tip(title: [Give every number its unit])[`d1 + 5` is an error, because `d1` is a length and `5` is a bare number. Write `d1 + 5 mm`. A bare number is right only where it scales something, as in `d1 / 2`.]


== Common mistakes

- *Using a parameter before it exists.* Call `set_param` first; otherwise `add_feature` fails with `unknown parameter`.
- *Naming a parameter badly.* A name starts with a letter or `_` and holds only letters, digits and `_`: `hole_2` works, `2nd_hole` and `hole-2` don't. Names are case-sensitive.
- *Reusing a name.* Every parameter needs its own name, whether it is a model or a user parameter. To swap two names, rename one of them to a spare name first.
- *Renaming a bound parameter in one place only.* Call `rename_bound` after `rename_param` (@sec-drive-sketch).
- *Forgetting to apply and solve.* Changing a parameter changes the table only. Call `apply_params`, then `solve`, to move the sketch.
- *Using a feature that comes later.* A feature may use only features already in the history. Otherwise `add_feature` fails with `feature N references feature M which is not earlier in the history`.
