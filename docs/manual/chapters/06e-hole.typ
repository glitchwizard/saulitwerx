#import "../template.typ": *

= Making holes <ch-hole>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("hole")
Almost every part has holes: for screws and bolts, for pins that line parts up, for shafts and wires to pass through. You could make each one by sketching a circle and cutting it (@sec-through-all), but a hole is so common that it has its own feature. You say where it starts, which way it goes, how wide and how deep it is, and the geometry engine drills it.

A #term[hole] has three things to choose:

- *Where and which way.* A start point on the face of the part, and a direction pointing into the material, the way a drill would go. The direction can be any way at all, so a hole can go into the top, the side or a sloping face.
- *Its shape.* A #term(key: "simple hole")[simple hole] is a plain round hole of one diameter. A #term[counterbore] adds a wider, flat-bottomed recess at the mouth of the hole, so that a screw head sits flush with the surface or below it.
- *How deep.* A #term[blind hole] stops at a depth you give, with a flat bottom. A #term(key: "through hole")[through hole] goes all the way through the part.

// cargo run -p swx-kernel --features occt --example hole_plate -- holes
// swx-qa render --step holes/hole-plate.step --size 900x700 -o docs/manual/img/hole-plate.png
#shot(
  "img/hole-plate.png",
  width: 65%,
  alt: "A rectangular plate with three holes in its top, growing in size from the back to the front, and one small hole in its near end",
  caption: [A 60 × 40 × 10 mm plate with four holes. In its top, from the back: a 6 mm simple hole through the plate, an 8 mm blind hole 5 mm deep, and a 6 mm through hole with an 11 mm counterbore 6 mm deep. In the end nearest you, a 4 mm blind hole 20 mm deep, drilled sideways.],
) <fig-holes>

*In Saulitwerx today:* the geometry engine can drill simple and counterbored holes, blind or through all, in any direction. Holes have flat bottoms; there are no drill points, countersinks or threads yet. You give the start point and the direction as numbers, so your program works them out first. A hole isn't recorded in a part's history yet.

== Drill holes in a plate <sec-hole>

#task-intro(
  goal: [Drill a through hole, a blind hole, a counterbored hole and a sideways hole in a plate.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its part in a folder called `holes`:
  #term-block("cargo run -p swx-kernel --features occt --example hole_plate -- holes")
  It prints the plate, one line after each hole, four errors on purpose, then the file it wrote:
  #term-block(label: "Output", "plate              24000.00 mm³   6 faces\n6 through          23717.26 mm³   7 faces\n+ 8 blind 5        23465.93 mm³   9 faces\n+ counterbore      22782.63 mm³  12 faces\n+ 4 side, 20 deep  22531.31 mm³  14 faces\ncb 5 on hole 6 -> error: invalid input: counterbore diameter 5 must be\n    larger than the hole diameter 6\nblind 4, cb 6 -> error: invalid input: hole depth 4 must be larger than\n    the counterbore depth 6\noff the plate -> error: invalid input: the hole misses the solid: it\n    removes nothing\ndiameter 0 -> error: invalid input: hole diameter must be finite and > 0,\n    got 0\n…  (the geometry engine's file reports)\nwrote holes/hole-plate.step")
  Four long error lines are wrapped here to fit the page.]],
  [Read the `6 through` line. A 6 mm hole through the 10 mm plate takes away a cylinder of π × 3² × 10 = 282.74 mm³. It adds one face: the wall of the hole.],
  [Read the `8 blind 5` line. The 8 mm hole stops 5 mm down and takes away 251.33 mm³. It adds two faces: the wall and the flat bottom.],
  [Read the `counterbore` line. The 6 mm hole goes through, and the 11 mm recess is 6 mm deep, so 683.30 mm³ goes in all. It adds three faces: the recess wall, the flat step at the recess's bottom, and the wall of the narrow hole.],
  [Read the `4 side` line. This hole starts on the plate's end at x = 60 and points back along −X, so it goes in sideways, 20 mm deep.],
  [Read the four errors. Each one says what is wrong, and the plate is unchanged after each.],
  [Draw the plate:
  #term-block("target/release/swx-qa render --step holes/hole-plate.step -o hole-plate.png")
  Open `hole-plate.png`. You should see the plate of @fig-holes.],
)

#result[A plate with four holes, each made by one call. Each call gives a new solid; the one before it is left as it was.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/hole_plate.rs`. Each hole is one call to `hole`, with four things: the solid, where it starts and which way it goes, its shape, and its depth.

#steps(
  [*Say where it starts and which way it goes.* An `Axis` holds a point and a direction. The sample's top holes start on the top face, at z = 10, and point straight down. The direction doesn't need to be one unit long. The sample's helper `down_at(x, y)` builds this axis for any spot on the top:
  ```rust
  let at = Axis { origin: [12.0, 20.0, 10.0], dir: [0.0, 0.0, -1.0] };
  ```],
  [#block(breakable: false)[*Give its shape and depth, and drill.* `HoleKind::Simple` takes a diameter; `HoleKind::Counterbore` takes the hole's diameter, the recess's diameter and the recess's depth. `HoleDepth::Blind` takes a depth in mm; `HoleDepth::ThroughAll` takes nothing:
  ```rust
  let one = k.hole(&plate, at, HoleKind::Simple { diameter: 6.0 },
                   HoleDepth::ThroughAll)?;
  let cb = HoleKind::Counterbore {
      diameter: 6.0, cb_diameter: 11.0, cb_depth: 6.0,
  };
  let two = k.hole(&one, down_at(48.0, 20.0), cb, HoleDepth::ThroughAll)?;
  ```]],
)
]

#tip(title: [Sizes for a screw])[A counterbore is sized for the screw it holds: the hole a little wider than the screw, the recess a little wider than its head, and the recess at least as deep as the head is tall. The sample's 6 mm hole, 11 mm recess and 6 mm depth are close to the usual sizes for a 5 mm cap screw. Check the sizes your screws need.]

#caution(title: [Depths are measured from the start point])[A blind depth and a counterbore depth both run from the start point along the direction. Put the start point on the face where the hole begins: start it 2 mm above the part, and a 5 mm hole is only 3 mm deep in the material.]

== Common mistakes <sec-hole-mistakes>

#set par(justify: false)

- *A direction pointing away from the part.* The hole goes the wrong way and finds nothing: `invalid input: the hole misses the solid: it removes nothing`. Point the direction into the material.
- *A start point off the part.* The same message when the hole's line misses the part altogether. Check the coordinates against the part's bounding box (@sec-ref-kernel).
- *A counterbore no wider than its hole.* `invalid input: counterbore diameter 5 must be larger than the hole diameter 6`.
- *A blind hole no deeper than its counterbore.* `invalid input: hole depth 4 must be larger than the counterbore depth 6`. Make the hole deeper, or use a through hole.
- *A size of zero or less.* `invalid input: hole diameter must be finite and > 0, got 0` (or `hole depth`, `counterbore diameter`, `counterbore depth`).
- *No direction.* `invalid input: hole direction must be finite and non-zero, got [0.0, 0.0, 0.0]`.
- *A hole that takes the whole part.* `empty result: the hole removes the whole solid`: the hole is wider than the part.
