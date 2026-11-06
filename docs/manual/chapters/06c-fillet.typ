#import "../template.typ": *

= Rounding and bevelling edges <ch-fillet>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("fillet")#idx("chamfer")
Sharp edges are rare on real parts. They cut hands, chip easily, and crack under load. So designers soften them, in one of two ways:

- A #term[fillet] rounds an edge into a smooth curve, like the corners of a phone. Its size is a _radius_: the radius of the curve that replaces the edge.
- A #term[chamfer] cuts an edge off at an angle, leaving a narrow flat face, like the bevel on the edge of a tabletop. Its size is a _distance_: how far the cut reaches back along each of the two faces that meet at the edge. It can also reach back further on one face than the other (@sec-chamfer-sizes).

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

*In Saulitwerx today:* the geometry engine can fillet edges with one radius, and chamfer them with one distance, the same on both faces. A chamfer can also take two different distances, or a distance and an angle (@sec-chamfer-sizes). You give the edges, and for an uneven chamfer the face to measure from, by their numbers in the solid (@sec-topology), so your program finds them first. A fillet or a chamfer isn't recorded in a part's history yet.

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

== Chamfer with two sizes <sec-chamfer-sizes>

#idx("chamfer")#idx("reference face")
An even chamfer cuts back the same distance on both faces, at 45° where they meet square. Sometimes you want it uneven: a long, shallow lead-in on the top of a part so a mating part slides on, or a bevel at a set angle to match a tool. Then one number isn't enough, and the chamfer needs to know which face each number belongs to. That face is the #term[reference face]: one of the two faces that meet at the edge.

You can size an uneven chamfer in two ways:

- *Two distances.* The first is measured on the reference face, the second on the other face.
- *A distance and an angle.* The distance is measured on the reference face. The angle is between the reference face and the new sloping face, so a smaller angle makes a flatter bevel. Where the faces meet square, the setback on the other face is the distance times the tangent of the angle: 6 mm at 30° reaches 3.46 mm down the side.

Swap the reference face and the same two numbers give a different part (@fig-chamfer-sizes).

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example chamfer_sizes -- edges
    // swx-qa render --step edges/chamfer-<name>.step --size 900x700 -o docs/manual/img/chamfer-<name>.png
    ("img/chamfer-equal.png", "Even, 3 mm", "The block with its near top edge bevelled evenly, 3 mm on the top and 3 mm down the side"),
    ("img/chamfer-top.png", "6 and 3 mm, top first", "The block with a wide, shallow bevel: 6 mm across the top and 3 mm down the side"),
    ("img/chamfer-side.png", "6 and 3 mm, side first", "The block with a tall, steep bevel: 3 mm across the top and 6 mm down the side"),
    ("img/chamfer-angle.png", "6 mm at 30°, top first", "The block with a shallow bevel: 6 mm across the top at 30 degrees to it, 3.46 mm down the side"),
  ),
  caption: [Four chamfers on the same edge of the 40 × 25 × 10 mm block: where the top meets the long side nearest you. The middle two use the same two distances, 6 and 3 mm, measured from different reference faces.],
) <fig-chamfer-sizes>

#task-intro(
  goal: [Bevel one edge of a block evenly, with two distances from each face in turn, and with a distance and an angle.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `edges`:
  #term-block("cargo run -p swx-kernel --features occt --example chamfer_sizes -- edges")
  It prints the block, the faces and the edge it found, one line for each chamfer, four errors on purpose, then the files it wrote:
  #term-block(label: "Output", "block                  10000.00 mm³\ntop face 5, side face 3, edge 11\nequal 3                 9820.00 mm³  slope y 22..25  z 7..10\n6 and 3, top first      9640.00 mm³  slope y 19..25  z 7..10\n6 and 3, side first     9640.00 mm³  slope y 22..25  z 4..10\n6 at 30°, top first     9584.31 mm³  slope y 19..25  z 6.54..10\n6 and 30 -> error: kernel failure: chamfer failed\n6 at 90° -> error: invalid input: chamfer angle must be finite and\n    strictly between 0 and π/2, got 1.5707963267948966\nbottom face -> error: invalid input: face 4 does not bound edge 11\nboth faces -> error: invalid input: edge 11 is listed with two\n    different faces (3 and 5)\n…  (the geometry engine's file reports)\nwrote edges/chamfer-equal.step\nwrote edges/chamfer-top.step\nwrote edges/chamfer-side.step\nwrote edges/chamfer-angle.step")
  Two long error lines are wrapped here to fit the page.]],
  [Read the second line. The top of the block is face 5, the long side at y = 25 is face 3, and the edge where they meet is edge 11.],
  [Read the `slope` part of each chamfer line. It is where the new sloping face lies: how far it reaches across the top (in y, from the edge at 25) and down the side (in z, from the top at 10). The even chamfer reaches 3 mm each way.],
  [Compare the two `6 and 3` lines. With the top as reference face, the slope reaches 6 mm across the top (y 19..25) and 3 mm down the side (z 7..10). With the side as reference face, it is the other way round. Both take away the same 360 mm³: a triangle of 6 × 3 ÷ 2 = 9 mm² along the 40 mm edge.],
  [Read the `6 at 30°` line. The slope reaches 6 mm across the top and 3.46 mm down the side, to z = 6.54.],
  [Draw the chamfers:
  #term-block("target/release/swx-qa render --step edges/chamfer-top.step -o chamfer-top.png")
  Open `chamfer-top.png`. You should see the second block of @fig-chamfer-sizes. Draw the other files the same way.],
)

#result[One edge bevelled four ways. The block itself is left as it was.]

=== How the sample is built

The program is `crates/swx-kernel/examples/chamfer_sizes.rs`. It does three things.

#steps(
  [#block(breakable: false)[*Find the faces and the edge.* The sample finds the top and the side by their bounding boxes, the way @sec-fillet found the upright edges. The edge it wants is the one both faces list as theirs (@sec-topology):
  ```rust
  let top_edges = k.adjacent(&block, top, TopoKind::Edge)?;
  let edge = k.adjacent(&block, side, TopoKind::Edge)?
      .into_iter()
      .find(|e| top_edges.contains(e))
      .expect("shared edge");
  ```]],
  [*Pair the edge with its reference face.* An `EdgeOnFace` holds an edge and the face to measure from. The face must be one of the two that meet at the edge:
  ```rust
  let on_top = EdgeOnFace { edge, face: top };
  let on_side = EdgeOnFace { edge, face: side };
  ```],
  [#block(breakable: false)[*Give the size, and chamfer.* `ChamferSize::TwoDistances` takes `d1` (on the reference face) and `d2` (on the other face). `ChamferSize::DistanceAngle` takes a `distance` and an `angle` in radians, so convert degrees with `to_radians()`:
  ```rust
  let two = ChamferSize::TwoDistances { d1: 6.0, d2: 3.0 };
  let angled = ChamferSize::DistanceAngle {
      distance: 6.0,
      angle: 30f64.to_radians(),
  };
  let from_top = k.chamfer_asymmetric(&block, &[on_top], two)?;
  let from_side = k.chamfer_asymmetric(&block, &[on_side], two)?;
  ```]],
)

#tip(title: [Many edges, one size])[`chamfer_asymmetric` takes a list, like `chamfer`. Every edge in it gets the same size, each measured from its own reference face. Listing the same edge and face twice is harmless; it counts once.]

#caution(title: [Angles are in radians])[The angle must be strictly between 0 and π/2 (90°). Typing `30.0` means 30 radians and is refused; write `30f64.to_radians()`, or 0.5236.]

== Common mistakes <sec-fillet-mistakes>

#set par(justify: false)

- *A size too large for the part.* The rounding or the bevel has to fit on both faces next to each edge, and must not run into the next one: `kernel failure: fillet failed` (or `chamfer failed`). Make it smaller than the narrowest face it touches, and less than half of it where two softened edges share that face.
- *An empty list.* `invalid input: the edge list is empty`. Give at least one edge.
- *A face or a vertex in the list.* Only edges can be rounded: `invalid input: expected an edge, got face 0`. Ask for edges with `TopoKind::Edge`, or turn a face into its edges with `adjacent` (@sec-topology).
- *An edge number from another solid.* `invalid input: edge index 12 is out of range: the shape has 12 edges`, or worse, the wrong edge. Find the edges in the solid you are softening.
- *A size of zero or less.* `invalid input: fillet radius must be finite and > 0, got 0` (or `chamfer distance`, `chamfer d1`, `chamfer d2`).
- *An uneven chamfer that doesn't fit.* Each setback must fit on its own face: `kernel failure: chamfer failed`. The sample's 30 mm on the 10 mm side is far too much.
- *An angle of 90° or more, or in degrees.* `invalid input: chamfer angle must be finite and strictly between 0 and π/2, got 1.5707963267948966`. Give radians, between 0 and π/2.
- *A reference face that doesn't touch the edge.* `invalid input: face 4 does not bound edge 11`. Use one of the two faces that `adjacent` lists for the edge.
- *The same edge with both faces.* `invalid input: edge 11 is listed with two different faces (3 and 5)`. Each edge gets one reference face; pick the one to measure the first distance on.
