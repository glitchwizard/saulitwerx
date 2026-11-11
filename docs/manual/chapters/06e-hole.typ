#import "../template.typ": *

= Making holes <ch-hole>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("hole")
Almost every part has holes: for screws and bolts, for pins that line parts up, for shafts and wires to pass through. You could make each one by sketching a circle and cutting it (@sec-through-all), but a hole is so common that it has its own feature. You say where it starts, which way it goes, how wide and how deep it is, and the geometry engine drills it.

A #term[hole] has three things to choose:

- *Where and which way.* A start point on the face of the part, and a direction pointing into the material, the way a drill would go. The direction can be any way at all, so a hole can go into the top, the side or a sloping face.
- *Its shape.* A #term(key: "simple hole")[simple hole] is a plain round hole of one diameter. A #term[counterbore] adds a wider, flat-bottomed recess at the mouth of the hole, so that a screw head sits flush with the surface or below it. A #term[countersink] adds a cone-shaped recess instead, for a screw with a cone-shaped (flat) head. A #term(key: "tapped hole")[tapped hole] is drilled at the right size for a screw thread (@sec-hole-tapped).
- *How deep.* A #term[blind hole] stops at a depth you give, with a flat bottom. A #term(key: "through hole")[through hole] goes all the way through the part.

// cargo run -p swx-kernel --features occt --example hole_plate -- holes
// swx-qa render --step holes/hole-plate.step --size 900x700 -o docs/manual/img/hole-plate.png
#shot(
  "img/hole-plate.png",
  width: 65%,
  alt: "A rectangular plate with three holes in its top, growing in size from the back to the front, and one small hole in its near end",
  caption: [A 60 × 40 × 10 mm plate with four holes. In its top, from the back: a 6 mm simple hole through the plate, an 8 mm blind hole 5 mm deep, and a 6 mm through hole with an 11 mm counterbore 6 mm deep. In the end nearest you, a 4 mm blind hole 20 mm deep, drilled sideways.],
) <fig-holes>

*In Saulitwerx today:* the geometry engine can drill simple, counterbored, countersunk and tapped holes, blind or through all, in any direction. Holes have flat bottoms, with no drill points. A tapped hole is drilled at its tap-drill size, but the thread itself isn't modeled. You give the start point and the direction as numbers, so your program works them out first. A hole isn't recorded in a part's history yet.

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

== Drill holes for screws <sec-hole-tapped>

#idx("thread") #idx("countersink") #idx("tapped hole")
A screw holds two parts together in one of two ways. Either it passes through a plain hole in the top part and screws into the part below, or it passes through both and a nut holds it. The part it screws into needs a hole with a #term[thread] cut inside it: a spiral ridge that matches the screw's. A hole with a thread inside it is a tapped hole, because the thread is cut with a tool called a tap.

Threads come in standard sizes. The commonest are the ISO metric threads, named `M` and the screw's outside diameter in mm: an M6 screw is 6 mm across. Each size has a #term[pitch], the distance from one ridge of the thread to the next. The "coarse" pitch is the one you get unless you ask for another; for M6 it is 1 mm. Before you tap a hole, you drill it narrower than the screw, so the tap has material to cut the thread into. That narrower size is the #term(key: "tap drill")[tap drill]. A common workshop rule gives it: the screw's diameter minus its pitch, so 5 mm for M6.

Saulitwerx has the metric coarse thread sizes from M2 to M24 built in (@sec-ref-threads). Ask it for a size by name, such as `"M6"`, and hand that size to a tapped hole. The hole is drilled at the tap-drill diameter.

A countersink is a cone at a hole's mouth, so a screw with a cone-shaped head sits flush with the surface. You give the hole's diameter, the cone's diameter at the surface, and the angle across the cone. Most metric flat-head screws have a 90° head.

// cargo run -p swx-kernel --features occt --example hole_screw -- holes
// swx-qa render --step holes/hole-screw.step --size 900x700 -o docs/manual/img/hole-screw.png
#shot(
  "img/hole-screw.png",
  width: 65%,
  alt: "A rectangular plate with three holes in a row along its top: a wide cone-shaped recess around a hole at the back, a small hole in the middle, and a larger hole at the front",
  caption: [A 60 × 30 × 12 mm plate with holes for screws. From the back: a 6.4 mm hole through the plate with a 90° countersink 12 mm across, a tapped M6 hole 10 mm deep (drilled 5 mm), and a tapped M8 hole through the plate (drilled 6.75 mm).],
) <fig-hole-screw>

#task-intro(
  goal: [Drill a countersunk hole and two tapped holes in a plate, and look up thread sizes.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install). The thread table needs no OpenCascade.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Print the thread table:
  #term-block("cargo run -p swx-kernel --example thread_table")
  It prints each size with its pitch, its minor diameter (the inside of the thread's ridges) and its tap drill, then four lookups by name:
  #term-block(label: "Output", "size   pitch  minor   tap drill\nM2      0.40   1.57   1.60\n…\nM6      1.00   4.92   5.00\nM8      1.25   6.65   6.75\n…\nM24     3.00  20.75  21.00\n\"M6\" -> M6, pitch 1 mm, tap drill 5 mm\n\"m10\" -> M10, pitch 1.5 mm, tap drill 8.5 mm\n\"M7\" -> error: invalid input: no metric coarse thread of size M7\n    (sizes: M2 to M24)\n\"6\" -> error: invalid input: thread designation \"6\" must be M\n    followed by a size, e.g. \"M6\"")
  Rows are left out here (…); the whole table is in @sec-ref-threads. Two long error lines are wrapped to fit the page.]],
  [Read the lookups. A small `m` works as well as a capital. M7 isn't a standard coarse size, and a size needs its `M`.],
  [#block(breakable: false)[Run the plate sample, and tell it to save its part in a folder called `holes`:
  #term-block("cargo run -p swx-kernel --features occt --example hole_screw -- holes")
  It prints the plate, one line after each hole, four errors on purpose, then the file it wrote:
  #term-block(label: "Output", "plate            21600.00 mm³   6 faces\ncountersink      21112.16 mm³   8 faces\n+ M6 blind 10     20915.81 mm³  10 faces\n+ M8 through      20486.39 mm³  11 faces\ncsk 6 on hole 6.4 -> error: invalid input: countersink diameter 6 must\n    be larger than the hole diameter 6.4\nangle 180° -> error: invalid input: countersink angle 3.141592653589793\n    must be strictly between 0 and π radians\nblind 2 -> error: invalid input: hole depth 2 must be larger than the\n    countersink depth 2.8000000000000003\nM7 -> error: invalid input: no metric coarse thread of size M7 (sizes:\n    M2 to M24)\n…  (the geometry engine's file reports)\nwrote holes/hole-screw.step")
  Four long error lines are wrapped here to fit the page.]],
  [Read the `countersink` line. The cone narrows from 12 mm to 6.4 mm at 90°, so it is (12 − 6.4) ÷ 2 = 2.8 mm deep. With the hole below it, 487.84 mm³ goes. It adds two faces: the cone and the hole's wall.],
  [Read the `M6 blind 10` line. The hole is 5 mm across, M6's tap drill, not 6 mm: π × 2.5² × 10 = 196.35 mm³ goes. It adds two faces, the wall and the flat bottom.],
  [Read the `M8 through` line. The hole is 6.75 mm across, 8 mm minus the 1.25 mm pitch, and adds one face.],
  [Read the errors. The last number in the third one is 2.8, printed with every digit the computer holds. The plate is unchanged after each.],
  [Draw the plate:
  #term-block("target/release/swx-qa render --step holes/hole-screw.step -o hole-screw.png")
  Open `hole-screw.png`. You should see the plate of @fig-hole-screw.],
)

#result[A plate with a countersunk hole for a flat-head screw and two holes ready to be tapped, each made by one call.]

#block(breakable: false)[
=== How the sample is built

The programs are `crates/swx-kernel/examples/thread_table.rs` and `crates/swx-kernel/examples/hole_screw.rs`. They use the same `hole` call as @sec-hole, with two more shapes.

#steps(
  [*Make a countersink.* `HoleKind::Countersink` takes the hole's diameter, the cone's diameter at the start point, and the angle across the cone in radians. `FRAC_PI_2` is π/2, that is 90°:
  ```rust
  let countersink = HoleKind::Countersink {
      diameter: 6.4, csk_diameter: 12.0, csk_angle: FRAC_PI_2,
  };
  let csk = k.hole(&plate, down_at(12.0, 15.0), countersink,
                   HoleDepth::ThroughAll)?;
  ```],
  [*Look up a thread and tap a hole.* `metric_coarse` turns a name into a `MetricThread`, which knows its `pitch` and its `tap_drill_diameter()`. `HoleKind::Tapped` takes it:
  ```rust
  let m6 = metric_coarse("M6")?;
  let tapped = k.hole(&csk, down_at(30.0, 15.0),
                      HoleKind::Tapped { thread: m6 },
                      HoleDepth::Blind(10.0))?;
  ```],
)
]

#tip(title: [Sizes for a flat-head screw])[The hole should be a little wider than the screw, and the cone at least as wide as the screw's head. The sample's 6.4 mm hole and 12 mm cone are close to the usual sizes for an M6 flat-head screw. Check the sizes your screws need.]

#caution(title: [The tap drill comes from a rule of thumb])[The tap-drill diameter here is the screw's diameter minus its pitch. Workshops sometimes use a slightly different drill, such as 6.8 mm rather than 6.75 mm for M8. If your shop or the screw's maker gives a size, use a simple hole of that size instead.]

== Common mistakes <sec-hole-mistakes>

#set par(justify: false)

- *A direction pointing away from the part.* The hole goes the wrong way and finds nothing: `invalid input: the hole misses the solid: it removes nothing`. Point the direction into the material.
- *A start point off the part.* The same message when the hole's line misses the part altogether. Check the coordinates against the part's bounding box (@sec-ref-kernel).
- *A counterbore no wider than its hole.* `invalid input: counterbore diameter 5 must be larger than the hole diameter 6`.
- *A blind hole no deeper than its counterbore.* `invalid input: hole depth 4 must be larger than the counterbore depth 6`. Make the hole deeper, or use a through hole.
- *A size of zero or less.* `invalid input: hole diameter must be finite and > 0, got 0` (or `hole depth`, `counterbore diameter`, `counterbore depth`).
- *No direction.* `invalid input: hole direction must be finite and non-zero, got [0.0, 0.0, 0.0]`.
- *A hole that takes the whole part.* `empty result: the hole removes the whole solid`: the hole is wider than the part.
- *A countersink no wider than its hole.* `invalid input: countersink diameter 6 must be larger than the hole diameter 6.4`.
- *A countersink angle in degrees, or of 180° or more.* `invalid input: countersink angle 3.141592653589793 must be strictly between 0 and π radians`. Give the angle in radians, between 0 and π: `90f64.to_radians()`.
- *A blind hole no deeper than its countersink.* `invalid input: hole depth 2 must be larger than the countersink depth 2.8000000000000003`. The cone's depth is (cone diameter − hole diameter) ÷ 2, divided by the tangent of half the angle; for 90°, just (cone diameter − hole diameter) ÷ 2. Make the hole deeper, or use a through hole.
- *A thread size that isn't in the table.* `invalid input: no metric coarse thread of size M7 (sizes: M2 to M24)`, or `thread designation "6" must be M followed by a size, e.g. "M6"` when the `M` is missing.
- *A home-made thread with a pitch as big as its diameter.* `invalid input: thread pitch 6 must be smaller than the nominal diameter 6`. Take sizes from `metric_coarse` rather than typing them.
