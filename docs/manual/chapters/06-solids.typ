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

#caution(title: [A number belongs to one shape])[Indices count the sub-shapes of one shape, so face 6 of this plate means nothing for another shape, even the same plate after a change. Ask again after every operation. Names that last through changes are on the way; see *Coming next*.]

*Common mistakes*

- *Asking for the same kind you start from,* such as the edges of an edge. That gives: \
  `invalid input: adjacent needs two different kinds, got edge and edge`.
- *Counting from 1.* The last face of a 7-face part is face 6. Face 7 gives: \
  `invalid input: face index 7 is out of range: the shape has 7 faces`.

== Open a STEP file from another program

`import_step` reads any STEP file, and `swx-qa render --step` draws one (@ch-pictures). Everything in the file comes in as one shape. Saulitwerx can't edit an imported shape yet; using it as a starting feature is on the roadmap.
