#import "../template.typ": *

= Parameters and history <ch-history>

#for-developers[This chapter uses the document model, `swx-core`, from Rust. It needs no OpenCascade.]

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

*In Saulitwerx today:* you create model parameters yourself with `add_model_param`. Sketch dimensions don't make them for you yet, and parameters hold plain numbers, not expressions such as `d1 / 2 + 5 mm`. Both are on the roadmap.

== Common mistakes

- *Using a parameter before it exists.* Call `set_param` first; otherwise `add_feature` fails with `unknown parameter`.
- *Naming a parameter badly.* A name starts with a letter or `_` and holds only letters, digits and `_`: `hole_2` works, `2nd_hole` and `hole-2` don't. Names are case-sensitive.
- *Reusing a name.* Every parameter needs its own name, whether it is a model or a user parameter. To swap two names, rename one of them to a spare name first.
- *Using a feature that comes later.* A feature may use only features already in the history. Otherwise `add_feature` fails with `feature N references feature M which is not earlier in the history`.
