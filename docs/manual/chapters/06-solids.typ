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

== Save a part as an STL file <sec-stl-export>

#idx("STL")
A 3D printer, and the slicing program that prepares a print, don't want a solid's exact curved faces. They want a #term[mesh]: the surface broken into many small flat triangles, like a geodesic dome. The most common file for a mesh is #term[STL]. It lists the triangles one by one, each with its three corners and the direction it faces, called its #term[normal]. STL has two forms: ASCII, which is plain text you can read, and binary, which is smaller and quicker to load. Both hold the same triangles.

STL keeps only the shape's surface, more coarsely than a STEP file and with no units: by custom, and in Saulitwerx, its numbers are millimeters. Use it to print a part, or to hand it to a program that wants a mesh. Keep the document file (@ch-files) or a STEP file for anything you will edit or build on.

#idx("deflection")
A mesh only comes close to a curved face: a round wall becomes a ring of flat strips. How close is set by the #term[deflection], the farthest any triangle may stray from the true surface, in millimeters. A smaller deflection gives a smoother mesh, with more triangles and a bigger file. A larger one gives fewer, larger triangles, and a round part comes out visibly faceted, like a pencil (@fig-stl-fineness). Flat faces don't change: they need only a few triangles, however small the deflection. Saulitwerx uses 0.01 mm unless you choose otherwise, well below the smallest step most 3D printers can make, so the facets don't show in a print.

#image-row(
  (
    // cargo run -p swx-qa --features occt --example stl_fineness -- docs/manual/img
    ("img/stl-coarse.png", "Deflection 1 mm: 100 triangles", "A flat round disc whose curved side is made of 26 wide flat strips, so its outline is visibly a many-sided polygon"),
    ("img/stl-fine.png", "Deflection 0.01 mm: 560 triangles", "The same disc with 141 narrow strips around its side, so its outline looks smoothly round"),
  ),
  caption: [The triangles `export_stl` writes for a disc 40 mm across and 8 mm thick, at two deflections. Lines show where the strips of the round side meet. The flat top and bottom look the same in both.],
) <fig-stl-fineness>

*In Saulitwerx today:* `swx-io` saves any solid as an STL file in one step, binary or ASCII, at the deflection you choose. It needs OpenCascade, and for now a short program, since there is no app yet. STL files can't be read.

#task-intro(
  goal: [Save a round part as STL three ways, compare the files, and see the deflection error.],
  needs: [Saulitwerx built from source with OpenCascade (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to write into a folder called `stl`:
  #term-block("cargo run -p swx-io --features occt --example export_part_stl -- stl")
  It makes a disc 40 mm across and 8 mm thick, saves it three times, then tries a deflection of 0 on purpose:
  #term-block(label: "Output", "part volume 10053.10 mm³\nwrote stl/disc.stl (28084 bytes, 560 triangles)\nwrote stl/disc-coarse.stl (5084 bytes, 100 triangles)\nwrote stl/disc-text.stl (111154 bytes), first line: solid disc-text\ndeflection 0 -> error: deflection 0 mm is invalid: it must be finite and\n    greater than 0\nstl/disc-bad.stl exists: false")
  One long line is wrapped here to fit the page.]],
  [Read the first two `wrote` lines. `disc.stl` uses the defaults, binary at 0.01 mm: 560 triangles. `disc-coarse.stl` uses 1 mm: 100 triangles, in a file less than a fifth of the size. A binary file is always 84 bytes plus 50 for each triangle.],
  [Compare the two meshes in @fig-stl-fineness. At 1 mm the round side is 26 flat strips, and the outline is a polygon you can see. At 0.01 mm it is 141 strips, and looks round. The coarse facets stray only about 0.15 mm, well inside 1 mm: the geometry engine also limits how far the surface may turn within one triangle, so a round face is never cut into very few strips.],
  [Read the third `wrote` line. `disc-text.stl` is the fine mesh again, as ASCII text: about four times the size of the binary file. Its first line names the solid after the file, `disc-text`.],
  [Read the error. A deflection must be a number greater than 0. It is checked before anything else, so no file is made: `disc-bad.stl` doesn't exist.],
)

#result[A round part saved as STL, smooth or coarse as you chose, ready for a 3D printer's slicing program.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-io/examples/export_part_stl.rs`. It makes the disc with the geometry engine (@sec-ref-kernel), then saves it:

#steps(
  [*With the defaults,* binary at 0.01 mm:
  ```rust
  export_stl(&k, &part, &path, &StlOptions::default())?;
  ```],
  [*With your own settings,* a deflection in millimeters and a form:
  ```rust
  let options = StlOptions { deflection_mm: 1.0,
                             format: StlFormat::Binary };
  export_stl(&k, &part, &path, &options)?;
  ```],
)
]

#tip(title: [Choosing a deflection])[Keep the default, 0.01 mm, for printing. Choose about 0.1 mm for a quick look in another program or a smaller file: round faces still look round. Larger values suit only rough previews. A finer mesh than you need costs file size and slicing time, and adds nothing a printer can make.]

#caution(title: [Binary unless you need text])[Choose ASCII only to read a file yourself or for a program that asks for it: the same triangles take about four times the space.]

== Write your own mesh as an STL file <sec-stl>

#for-developers[When your program makes a mesh itself, rather than a solid, `swx-io` writes it as STL directly, in either form. This section needs no OpenCascade.]

#task-intro(
  goal: [Write a small mesh as an ASCII and a binary STL file, read the ASCII one, and see the errors.],
  needs: [Saulitwerx built from source (@ch-install). OpenCascade isn't needed.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to write into a folder called `stl`:
  #term-block("cargo run -p swx-io --example stl_mesh -- stl")
  It writes the two files, then tries three broken meshes on purpose:
  #term-block(label: "Output", "wrote stl/corner.stl (619 bytes)\nwrote stl/corner-binary.stl (284 bytes)\nhalf a triangle -> error: index count 2 is not a multiple of 3\nno such corner -> error: index 7 is out of range for a mesh with 4\n    vertices\nnot a number -> error: vertex 1 has a non-finite coordinate\nbytes written by the failed tries: 0")
  One long line is wrapped here to fit the page.]],
  [Read the first two lines. The mesh is a corner cut off a 10 mm cube: a small pyramid with four triangular faces. As text it takes 619 bytes; in binary, 284. A binary file is always 84 bytes plus 50 for each triangle, so a part's mesh of many thousands of triangles stays far smaller in binary.],
  [#block(breakable: false)[Look at the ASCII file:
  #term-block("cat stl/corner.stl")]
  #block(breakable: false)[It names the solid, then lists each triangle (`facet`) with its normal and its three corners (`vertex`), in millimeters:
  #term-block(label: "Output", "solid corner\n  facet normal 0e0 0e0 -1e0\n    outer loop\n      vertex 0e0 0e0 0e0\n      vertex 0e0 1e1 0e0\n      vertex 1e1 0e0 0e0\n    endloop\n  endfacet\n  …\n  facet normal 5.7735026e-1 5.7735026e-1 5.7735026e-1\n    outer loop\n      vertex 1e1 0e0 0e0\n      vertex 0e0 1e1 0e0\n      vertex 0e0 0e0 1e1\n    endloop\n  endfacet\nendsolid corner")]
  The middle two triangles are left out here. Numbers are in #term[e notation]: `1e1` is 1 × 10¹, which is 10, and `5.7735026e-1` is 0.57735026. The first triangle lies on the floor and faces straight down, −Z; the last is the sloping face, and faces up and out equally along X, Y and Z.],
  [Read the errors. Every triangle needs three corners, so two numbers are half a triangle. A triangle can't use corner 7 of a mesh with four corners. A corner whose position isn't a number is refused. The mesh is checked in full before the first byte is written, so a broken mesh never leaves half a file behind.],
)

#result[The same small mesh as a readable text STL and as a compact binary STL, ready for any program that reads STL.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-io/examples/stl_mesh.rs`. A mesh is two lists: the corners, as x, y, z in millimeters, and the triangles, as three corner numbers each, counting from 0.

#steps(
  [*List the corners and the triangles.* Give each triangle's corners counterclockwise as seen from outside the part; that is how the writer knows which way the triangle faces:
  ```rust
  let positions = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0],
                   [0.0, 10.0, 0.0], [0.0, 0.0, 10.0]];
  let indices = [0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
  ```],
  [*Write it.* Each writer takes anything you can write bytes to, such as an open file. The ASCII writer also takes the name it prints after `solid`:
  ```rust
  write_stl_ascii("corner", &positions, &indices, &mut out)?;
  write_stl_binary(&positions, &indices, &mut out)?;
  ```],
)
]

#tip(title: [A part's own mesh])[To save a solid, use `export_stl` (@sec-stl-export): it breaks the solid into triangles and calls these writers for you. The geometry engine's `tessellate` (@sec-ref-kernel) gives the same mesh, if your program needs to change it before writing.]

#caution(title: [Get the winding right])[The writer works out each triangle's normal from the order of its corners. Corners listed clockwise give a normal that points into the part, and many programs then show the triangle as a hole or turn the part inside out. If a mesh looks inside out in another program, swap two corners of every triangle.]

== Write a mesh as a glTF file <sec-glb>

STL suits a 3D printer, but a web page, a phone or a design-review viewer that shows a part in 3D usually wants #term(key: "glTF")[glTF] instead: an open standard for 3D scenes, made to load quickly. Its one-file form, #term(key: "GLB")[GLB], holds a short text description of the scene followed by the mesh's numbers in binary.

Two things set it apart from STL. It stores each corner once and lists each triangle by its three corner numbers, the same way Saulitwerx keeps a mesh, so a large mesh takes less room. And it has units: glTF measures in meters. Saulitwerx keeps the corners in millimeters and tells the viewer to scale them by 0.001, so a 10 mm part shows at its true size of 0.01 m.

*In Saulitwerx today:* `swx-io` writes any triangle mesh your program gives it as a GLB file, with no colors and no normals; viewers work out the shading from the triangles. Keeping a part's mesh as a GLB file inside its document file, so the part shows at once when opened, is planned (*Coming next*). This section needs no OpenCascade.

#task-intro(
  goal: [Write the same small mesh as a GLB file, read what it holds, and see the errors.],
  needs: [Saulitwerx built from source (@ch-install). OpenCascade isn't needed.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to write into a folder called `glb`:
  #term-block("cargo run -p swx-io --example glb_mesh -- glb")
  It writes the file, reads back what is in it, then tries four broken meshes on purpose:
  #term-block(label: "Output", "wrote glb/corner.glb (688 bytes)\nheader: magic glTF, version 2, length 688\nchunks: JSON 564 bytes, BIN 96 bytes\nnode: {\"mesh\":0,\"scale\":[0.001,0.001,0.001]}\ncorners: {\"bufferView\":0,\"componentType\":5126,\"count\":4,\n    \"max\":[10.0,10.0,10.0],\"min\":[0.0,0.0,0.0],\"type\":\"VEC3\"}\ntriangles: 12 indices, 4 triangles\nhalf a triangle -> error: index count 2 is not a multiple of 3\nno such corner -> error: index 7 is out of range for a mesh with 4\n    vertices\nnot a number -> error: vertex 1 has a non-finite coordinate\nno triangles -> error: the mesh has no triangles\nbytes written by the failed tries: 0")
  Two long lines are wrapped here to fit the page.]],
  [Read the first three lines. The mesh is the corner cut off a 10 mm cube from @sec-stl. The file starts with the letters `glTF` and the version, 2, so any program can tell what it is. Then come two #term(key: "chunk")[chunks]: the scene description in JSON, and the corners and triangles in binary, 96 bytes of them.],
  [Read the `node` line. The scene has one node, which holds the mesh and scales it by 0.001 on each axis: millimeters to meters.],
  [Read the `corners` line. There are 4 corners (`count`), each three numbers (`VEC3`), and the file states the smallest and largest of them: from 0 to 10 mm on every axis. A viewer uses these to frame the part before it reads a single triangle. The 12 corner numbers make 4 triangles.],
  [Read the errors. They are the same checks as for STL, plus one: a mesh with no triangles at all is refused, because there would be nothing to show. As for STL, the mesh is checked in full before the first byte is written.],
)

#result[A GLB file of the mesh, at its true size, ready for a 3D viewer, a web page or a game engine.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-io/examples/glb_mesh.rs`. The mesh is the same two lists as for STL (@sec-stl), and the call takes anything you can write bytes to:

```rust
write_glb(&positions, &indices, &mut out)?;
```
]

#tip(title: [STL or GLB?])[For a 3D printer, STL: every printing program reads it. To show a part on screen, GLB: it is smaller for a large mesh, carries its units, and loads quickly. Binary STL takes 50 bytes for each triangle; GLB takes 12 for each triangle and 12 for each corner, and in a part's mesh most corners are shared by several triangles. In this tiny sample the scene description outweighs the saving, so the GLB is the larger file.]
