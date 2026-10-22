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

== Common mistakes

- *Using a parameter before it exists.* Call `set_param` first; otherwise `add_feature` fails with `unknown parameter`.
- *Using a feature that comes later.* A feature may use only features already in the history. Otherwise `add_feature` fails with `feature N references feature M which is not earlier in the history`.
