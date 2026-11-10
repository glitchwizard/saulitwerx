#import "../template.typ": *

= Solids and STEP files <ch-solids>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("geometry kernel")
The part of a CAD program that does the 3D math is its #term[geometry kernel]. Saulitwerx talks to its kernel through one interface, the `Kernel` trait, so the rest of the program never depends on which kernel is underneath. Today that kernel is OpenCascade, reached through `OcctKernel`.

== Make a solid and save it as STEP

#task-intro(
  goal: [Build the plate with a hole, measure it, save it as STEP and read it back.],
  needs: [Saulitwerx built with OpenCascade (@ch-install).],
  time: [5 minutes.],
)

You already ran this program in @ch-first. Here is what each part of it does. The file is `crates/swx-kernel/examples/plate_with_hole.rs`.

#steps(
  [*Make the two starting solids.* A box is given by its lowest corner and its three sizes; a cylinder stands on +Z, given by the center of its base, its radius and its height:
  ```rust
  let plate = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 5.0])?;
  let tool  = k.make_cylinder([20.0, 12.5, -1.0], 5.0, 7.0)?;
  ```],
  [*Cut one from the other.* The cylinder starts 1 mm below the plate and ends 1 mm above it, so the hole goes cleanly all the way through:
  ```rust
  let part = k.cut(&plate, &tool)?;
  ```],
  [*Measure the result* with `volume`, `surface_area`, `face_count` and `bounding_box`. The program prints them; @ch-first checks them by hand.],
  [*Save and read back.* `export_step` writes the file, in millimeters; `import_step` reads it back as one solid. Matching volume and face count show nothing was lost:
  ```rust
  k.export_step(&part, &out)?;
  let back = k.import_step(&out)?;
  ```],
)

#result[A STEP file of the plate, checked by reading it back: `wrote plate.step (read back: 4607.30 mm³, 7 faces)`.]

#tip(title: [Make the tool overlap])[When you cut a hole, make the tool solid a little longer than the part on both sides. A tool that ends exactly on a face can leave a paper-thin skin, or confuse the kernel.]

== Combine solids

The kernel has three Boolean operations (@sec-feature). Each takes two solids and returns a new one; the inputs are left unchanged.

#ref-table(
  columns: (auto, 1fr),
  header: ("Call", "Result"),
  [`k.union(&a, &b)`], [Everything in `a` or `b`.],
  [`k.cut(&a, &b)`], [`a` minus whatever `b` overlaps.],
  [`k.intersect(&a, &b)`], [Only what is in both.],
)

#caution[Sizes must be finite and greater than zero, and positions finite. A box with a zero size is refused before it reaches the kernel, with `invalid input: box dx must be finite and > 0, got 0`.]

== Faces, edges and vertices of a solid <sec-topology>

#idx("sub-shape") #idx("adjacent")
A solid's faces, edges and vertices (@sec-solid) are its #term[sub-shapes]. Later features will need them: a fillet rounds an edge, a sketch sits on a face. Today you can count them and walk from one to its neighbours. Two sub-shapes are #term[adjacent] when one bounds the other: a face and the edges around it, or an edge and the two vertices at its ends.

The kernel numbers the sub-shapes of each kind from 0. `SubShape { kind, index }` names one of them, such as face 6, and `TopoKind` is the kind: `Face`, `Edge` or `Vertex`.

#task-intro(
  goal: [Count the plate's faces, edges and vertices, and find the faces around the hole.],
  needs: [Saulitwerx built with OpenCascade (@ch-install).],
  time: [5 minutes.],
)

The program is `crates/swx-kernel/examples/part_topology.rs`. It builds the same plate as above, then asks the kernel about its parts.

#steps(
  [*Count each kind* with `sub_shape_count`:
  ```rust
  for kind in [TopoKind::Face, TopoKind::Edge, TopoKind::Vertex] {
      println!("{kind:?} count: {}", k.sub_shape_count(&part, kind)?);
  }
  ```
  A shared sub-shape counts once: the edge between two faces is one edge.],
  [*Go down from a face to its edges* with `adjacent`. Give it the sub-shape to start from and the kind you want back:
  ```rust
  let face = SubShape { kind: TopoKind::Face, index };
  let edges = k.adjacent(&part, face, TopoKind::Edge)?;
  ```
  The list comes back sorted by index, with no repeats.],
  [*Go up from an edge to its faces* the same way, asking for `TopoKind::Face`. The program does this for each edge of the hole's wall, the only face with 3 edges.],
  [*Run it:*
  #term-block("cargo run -p swx-kernel --features occt --example part_topology")
  You should see:
  #term-block(label: "Output", "Face count: 7
Edge count: 15
Vertex count: 10

face 0: 4 edges [0, 1, 2, 3]
face 1: 4 edges [0, 4, 5, 6]
face 2: 5 edges [1, 6, 7, 8, 9]
face 3: 4 edges [2, 7, 10, 11]
face 4: 5 edges [3, 4, 10, 12, 13]
face 5: 4 edges [5, 8, 11, 12]
face 6: 3 edges [9, 13, 14]

hole wall is face 6
  edge 9: faces [2, 6]
  edge 13: faces [4, 6]
  edge 14: faces [6]

face 7: invalid input: face index 7 is out of range: the shape has 7 faces")],
)

#result[The plate has 7 faces, 15 edges and 10 vertices. Faces 2 and 4 have 5 edges each, four sides plus the rim of the hole: they are the top and the bottom, the two faces the hole passes through. The hole's wall, face 6, meets them along edges 9 and 13.]

#note(title: [Why 15 edges, and an edge with one face])[A box has 12 edges; the hole adds its two rims, which makes 14. The fifteenth is a #term[seam]: the kernel makes a closed round face, like the hole's wall, from a sheet rolled up until its two sides meet, and the line where they meet is an edge. It doesn't show in pictures. Because the wall lies on both sides of it, the seam, edge 14, lists only one face.]

#caution(title: [A number belongs to one shape])[Indices count the sub-shapes of one shape, so face 6 of this plate means nothing for another shape, even the same plate after a change. Ask again after every operation. An extrude and a Boolean can tell you which of their faces is which (@sec-extrude-history, @sec-boolean-history); names that last through changes are on the way, see *Coming next*.]

*Common mistakes*

- *Asking for the same kind you start from,* such as the edges of an edge. That gives: \
  `invalid input: adjacent needs two different kinds, got edge and edge`.
- *Counting from 1.* The last face of a 7-face part is face 6. Face 7 gives: \
  `invalid input: face index 7 is out of range: the shape has 7 faces`.

== Find the faces a Boolean made <sec-boolean-history>

#idx("Boolean history")
A Boolean renumbers the faces of its result (@sec-topology), so face 5 of a block need not be face 5 after a cut. To keep track of a face, such as the top you want to round next, ask for the Boolean _with history_. Next to the new solid you get one entry for each face of both inputs: first every face of `a`, then every face of `b`, each with the faces of the result it became.

An entry can list:

- *One face.* The face lives on in the result. It may be trimmed: the cutting tool's bottom becomes a groove's floor, cut to the block's width.
- *Several faces.* The face was #term(key: "split face")[split]: a groove across the top leaves two pieces of top, one each side.
- *No faces.* The face was deleted: nothing of it is left on the result's surface.

#shot(
  // cargo run -p swx-kernel --features occt --example boolean_history -- groove.step
  // swx-qa render --step groove.step --size 1200x840 -o docs/manual/img/groove.png
  // Rings drawn by the template; face numbers from the same run.
  "img/groove.png",
  width: 62%,
  alt: "A block with a groove across its middle. Rings mark the two pieces of the top, one each side of the groove, and the groove's floor",
  marks: (
    (25%, 52%, [top: face 2]),
    (63%, 23%, [top: face 6]),
    (54%, 50%, [floor: face 8]),
  ),
  caption: [A groove cut across a block, with history. The block's top, face 5, was split into faces 2 and 6 of the result. The tool's bottom became the groove's floor, face 8.],
) <fig-boolean-history>

#task-intro(
  goal: [Cut a groove across a block with history, and find what became of each face.],
  needs: [Saulitwerx built with OpenCascade (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [Run the sample program, and tell it to save the part as `groove.step`:
  #term-block("cargo run -p swx-kernel --features occt --example boolean_history -- groove.step")
  #block(breakable: false)[It prints the face counts, one line for each face of the block and of the tool, then the groove's floor:
  #term-block(label: "Output", "block 6 faces, tool 6 faces, part 10 faces\nblock face 0  x 0..0  y 0..25  z 0..10     one face   -> [0]\nblock face 1  x 40..40  y 0..25  z 0..10   one face   -> [5]\nblock face 2  x 0..40  y 0..0  z 0..10     one face   -> [1]\nblock face 3  x 0..40  y 25..25  z 0..10   one face   -> [3]\nblock face 4  x 0..40  y 0..25  z 0..0     one face   -> [4]\nblock face 5  x 0..40  y 0..25  z 10..10   split in 2 -> [2, 6]\ntool  face 0  x 15..15  y -1..26  z 5..11  one face   -> [9]\ntool  face 1  x 25..25  y -1..26  z 5..11  one face   -> [7]\ntool  face 2  x 15..25  y -1..-1  z 5..11  deleted    -> []\ntool  face 3  x 15..25  y 26..26  z 5..11  deleted    -> []\ntool  face 4  x 15..25  y -1..26  z 5..5   one face   -> [8]\ntool  face 5  x 15..25  y -1..26  z 11..11 deleted    -> []\ngroove floor is part face 8  x 15..25  y 0..25  z 5..5\npart volume 8750.00 mm³\n…  (the geometry engine's file report)\nwrote groove.step")]],
  [Read the line `block face 5`. It lies flat at z = 10: the block's top. The groove split it in two, faces 2 and 6 of the part.],
  [Read the `tool` lines. The tool's two walls at x = 15 and x = 25 became the groove's walls, faces 9 and 7. Its bottom at z = 5 became the floor, face 8. Its two ends and its top stuck out of the block, so nothing of them is left.],
  [Count the part's faces: the block's 6, plus one more piece of top, plus the tool's 3 that survive, make 10.],
)

#result[Every face of the grooved block traced back to the face it came from.]

The sample calls `boolean_tracked` with the operation to do, `BooleanOp::Union`, `Cut` or `Intersect`, and the two solids. It returns the solid, the same as `union`, `cut` or `intersect` would, and an `OpHistory`:

```rust
let (part, history) = k.boolean_tracked(BooleanOp::Cut, &block, &tool)?;
for (operand, face, images) in &history.modified {
    // operand: Operand::A (the block) or Operand::B (the tool)
    // face: a face of that input; images: the faces of `part` it became
}
```

*In Saulitwerx today:* history covers faces only. Edges are found through their faces (@sec-topology). `history.generated` is always empty for a Boolean, because a Boolean makes no new faces, only the edges where the two solids meet. A cut or join made by an extrude (@sec-extrude-modes) doesn't report a history yet, and the part's history doesn't keep it.

== Parts with several bodies <sec-bodies>

#idx("multi-body part")
A solid doesn't have to be in one piece. Cut a slot right across a plate and you get two pieces that don't touch, but they are still one result, one shape. Each separate piece is a #term[body]. A part with more than one is a #term[multi-body part]. You meet them when a cut splits a part, and you can build them on purpose: to make two halves that fit together, or to shape several pieces side by side and pull them apart at the end.

Three calls work with bodies:

- `body_count` says how many bodies a shape has.
- `bodies` hands you each body as its own shape, to measure, save or use on its own.
- `combine` applies one Boolean to a target with a whole list of tools, in order: one call drills every hole, or joins every boss. `combine_tracked` does the same and says what became of every face, like a Boolean with history (@sec-boolean-history).


#task-intro(
  goal: [Drill two holes in one call, see what became of each face, split the plate in two, and take the two bodies apart.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its part in a folder called `bodies`:
  #term-block("cargo run -p swx-kernel --features occt --example multi_body -- bodies")
  It prints the drilled plate and the history of its faces, the slotted plate and its two bodies, three errors on purpose, and the file it wrote:
  #term-block(label: "Output", "drilled plate: 13796.81 mm³, 8 faces, 1 body\nplate face 0 -> [0]\nplate face 1 -> [5]\nplate face 2 -> [1]\nplate face 3 -> [3]\nplate face 4 -> [4]\nplate face 5 -> [2]\npin 0 faces -> [[6], [], []]\npin 1 faces -> [[7], [], []]\nslotted plate: 12836.81 mm³, 2 bodies\nbody 0: 6418.41 mm³, x 0..28\nbody 1: 6418.41 mm³, x 32..60\nno tools -> error: invalid input: combine needs at least one tool body\nflat tool -> error: invalid input: combine tool 0 has no solid body\nfar tool -> error: empty result: the combined result encloses no volume\n…  (the geometry engine's file report)\nwrote bodies/two-bodies.step")]],
  [Read the first line. The plate is 60 × 40 × 6 = 14400 mm³, and each hole 8 mm across takes π × 4² × 6 = 301.59 mm³: 14400 − 603.19 = 13796.81 mm³. Still one body.],
  [Read the `plate face` lines. Each of the plate's 6 faces lives on, renumbered. The top and the bottom now have holes in them, but each is still one face.],
  [Read the `pin` lines. Each pin is a cylinder with three faces: its round side, then its two ends. The round side became the wall of its hole, face 6 or 7. The ends were outside the plate, so nothing of them is left.],
  [Read the `slotted plate` lines. The slot removed 4 × 40 × 6 = 960 mm³, and the plate fell into 2 bodies. `bodies` gives them in order along X: the left half, from x = 0 to 28, then the right half, from 32 to 60. Each holds half the volume.],
  [Read the errors. A combine needs at least one tool, every tool must be a solid (not a flat face), and an intersect with a tool that doesn't touch the plate leaves nothing.],
  [#block(breakable: false)[Draw the part:
  #term-block("target/release/swx-qa render --step bodies/two-bodies.step -o two-bodies.png")
  Open `two-bodies.png`. You should see the two halves of @fig-two-bodies.]],
)

#result[Two holes from one call, a part split into two bodies, and each body in hand on its own.]

// cargo run -p swx-kernel --features occt --example multi_body -- bodies
// swx-qa render --step bodies/two-bodies.step --size 900x700 -o docs/manual/img/two-bodies.png
#shot(
  "img/two-bodies.png",
  width: 65%,
  alt: "Two flat rectangular plates side by side with a narrow gap between them, each with one round hole through its middle",
  caption: [A 60 × 40 × 6 mm plate with two holes, drilled in one combine call, then cut by a 4 mm slot across the middle. The result is one shape with two bodies.],
) <fig-two-bodies>

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/multi_body.rs`. It does three things.

#steps(
  [*Combine with history.* The tools go in a list. The result is the same as cutting with each pin in turn:
  ```rust
  let (drilled, history) = k.combine_tracked(BooleanOp::Cut, &plate, &pins)?;
  // history.target[f]: the faces of `drilled` that face f of the plate became
  // history.tools[t][f]: the same for face f of tool t
  ```],
  [*Split.* A plain `combine` with one tool, the slot, cuts the plate in two:
  ```rust
  let split = k.combine(BooleanOp::Cut, &drilled, &[slot])?;
  ```],
  [*Count and take apart:*
  ```rust
  let n = k.body_count(&split)?;      // 2
  for body in k.bodies(&split)? { /* measure, save or use each one */ }
  ```],
)
]

#tip(title: [One call for a row of features])[To drill ten holes or join ten bosses, build the ten tools and pass them to one `combine`. It's shorter than ten Booleans, and `combine_tracked` still tells you which hole wall came from which tool.]

#caution(title: [Count the bodies after a cut])[A cut can split a part without warning: a slot or a hole that runs right across leaves two pieces in one shape. If a later feature should touch the whole part, check `body_count` first.]

*In Saulitwerx today:* the bodies of a shape can be counted, taken apart and combined, but not named, moved or picked, and a part's history doesn't keep them. `bodies` orders them by the lowest corner of each one's bounding box, so the order follows where they are, not how they were made.

== Open a STEP file from another program

`import_step` reads any STEP file, and `swx-qa render --step` draws one (@ch-pictures). Everything in the file comes in as one shape. Saulitwerx can't edit an imported shape yet; using it as a starting feature is on the roadmap.
