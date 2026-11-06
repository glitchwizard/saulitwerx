#import "../template.typ": *

= Rounding and bevelling edges <ch-fillet>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("fillet")#idx("chamfer")
Sharp edges are rare on real parts. They cut hands, chip easily, and crack under load. So designers soften them, in one of two ways:

- A #term[fillet] rounds an edge into a smooth curve, like the corners of a phone. Its size is a _radius_: the radius of the curve that replaces the edge.
- A #term[chamfer] cuts an edge off at an angle, leaving a narrow flat face, like the bevel on the edge of a tabletop. Its size is a _distance_: how far the cut reaches back along each of the two faces that meet at the edge.

Both work on edges of a solid you already have. You pick the edges, give one size, and get back a new solid with those edges softened (@fig-edges).

#image-row(
  (
    // modes-target.png: see chapter 7 (extrude_modes); the same 40 × 25 × 10 mm block
    ("img/modes-target.png", "Block: 40 × 25 × 10 mm", "A plain rectangular block with sharp edges"),
    // cargo run -p swx-kernel --features occt --example fillet_chamfer -- edges
    // swx-qa render --step edges/edges-<name>.step --size 900x700 -o docs/manual/img/edges-<name>.png
    ("img/edges-fillet.png", "Fillet 5 mm, 4 edges", "The block with its four upright corners rounded"),
    ("img/edges-chamfer.png", "Chamfer 5 mm, 4 edges", "The block with its four upright corners cut off at 45 degrees"),
    ("img/edges-all.png", "Fillet 2 mm, all edges", "The block with every edge softly rounded, like a bar of soap"),
  ),
  caption: [One block, three ways to soften its edges. The fillet and the chamfer take the same four upright corner edges and the same size, 5 mm. The last picture rounds every edge at once. Each picture is scaled to fill its frame.],
) <fig-edges>

*In Saulitwerx today:* the geometry engine can fillet edges with one radius, and chamfer them with one distance, the same on both faces. You give the edges by their numbers in the solid (@sec-topology), so your program finds them first. A fillet or a chamfer isn't recorded in a part's history yet, and there is no chamfer with two different distances or with a distance and an angle.

== Fillet and chamfer edges <sec-fillet>

#task-intro(
  goal: [Round the four corners of a block, bevel them instead, and round every edge at once.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `edges`:
  #term-block("cargo run -p swx-kernel --features occt --example fillet_chamfer -- edges")]
  It prints the block, the edges it picked, one line for each result, then four errors on purpose, then the files it wrote:
  #term-block(label: "Output", "block              10000.00 mm³   6 faces  12 edges\nupright edges: [0, 2, 4, 6]\nfillet r 5          9785.40 mm³  10 faces  24 edges\nchamfer 5           9500.00 mm³  10 faces  24 edges\nfillet all, r 2     9753.19 mm³  26 faces  56 edges\nr 15 -> error: kernel failure: fillet failed\nno edges -> error: invalid input: the edge list is empty\na face -> error: invalid input: expected an edge, got face 0\nr 0 -> error: invalid input: fillet radius must be finite and > 0, got 0\n…  (the geometry engine's file reports)\nwrote edges/edges-fillet.step\nwrote edges/edges-chamfer.step\nwrote edges/edges-all.step")],
  [Read the second line. Of the block's 12 edges, numbers 0, 2, 4 and 6 are the four that stand upright at its corners.],
  [Read the `fillet r 5` line. Each corner lost a 5 × 5 mm square less a quarter circle, 5.37 mm², over the block's 10 mm height. Four corners make 214.60 mm³. Each rounded corner is a new face, so the block now has 10.],
  [Read the `chamfer 5` line. Each corner lost a triangle with two 5 mm sides, 12.5 mm², so four corners take 500 mm³. The new faces are flat.],
  [Read the `r 15` line. A 15 mm fillet doesn't fit: on the block's 25 mm side, the two rounded corners would need 15 + 15 = 30 mm. The geometry engine can't build it and says so. The block is unchanged.],
  [Draw the filleted block:
  #term-block("target/release/swx-qa render --step edges/edges-fillet.step -o edges-fillet.png")
  Open `edges-fillet.png`. You should see the block of @fig-edges with its four corners rounded. Draw the other files the same way.],
)

#result[Three softened blocks, each made by one call. The block itself is left as it was.]

=== How the sample is built

The program is `crates/swx-kernel/examples/fillet_chamfer.rs`. It does two things.

#steps(
  [*Find the edges.* An edge is a `SubShape` of kind `Edge`, numbered from 0. To find the upright ones, the sample asks for each edge's bounding box (@sec-extrude-history). An upright edge's box has no width and no depth, only height:
  ```rust
  let bb = k.sub_shape_bounding_box(&block, edge)?;
  if bb.max[0] - bb.min[0] < 1e-6 && bb.max[1] - bb.min[1] < 1e-6 {
      corners.push(edge);
  }
  ```],
  [#block(breakable: false)[*Fillet or chamfer them.* `fillet` takes the solid, a list of edges and a radius; `chamfer` takes the same with a distance. Every edge in the list gets the same size:
  ```rust
  let rounded = k.fillet(&block, &corners, 5.0)?;
  let bevelled = k.chamfer(&block, &corners, 5.0)?;
  let soft = k.fillet(&block, &all, 2.0)?;
  ```]],
)

#tip(title: [Round many edges in one call])[Give all the edges that should be rounded in one list, as the sample does for `all`. Where the rounded edges meet at a corner, the geometry engine blends them into one smooth corner. Listing an edge twice is harmless; it counts once.]

#caution(title: [The numbers change])[A fillet or a chamfer makes new faces and edges, so the result is numbered afresh: the block had 12 edges, the filleted one has 24. Edge 0 of the result may be a different edge. To soften more edges later, find them again in the new solid.]

== Common mistakes <sec-fillet-mistakes>

#set par(justify: false)

- *A size too large for the part.* The rounding or the bevel has to fit on both faces next to each edge, and must not run into the next one: `kernel failure: fillet failed` (or `chamfer failed`). Make it smaller than the narrowest face it touches, and less than half of it where two softened edges share that face.
- *An empty list.* `invalid input: the edge list is empty`. Give at least one edge.
- *A face or a vertex in the list.* Only edges can be rounded: `invalid input: expected an edge, got face 0`. Ask for edges with `TopoKind::Edge`, or turn a face into its edges with `adjacent` (@sec-topology).
- *An edge number from another solid.* `invalid input: edge index 12 is out of range: the shape has 12 edges`, or worse, the wrong edge. Find the edges in the solid you are softening.
- *A size of zero or less.* `invalid input: fillet radius must be finite and > 0, got 0` (or `chamfer distance`).
