#import "../template.typ": *

= Extruding a profile <ch-extrude>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("extrude")
An #term[extrude] takes a flat shape and pushes it straight out into 3D, the way a cookie cutter pressed into dough makes a shape with straight sides. It is the most common way to start a part: draw the outline, then give it a thickness.

Think back to the plate of @ch-first. There you made it in two steps: a block, then a cylinder cut out of it. With an extrude you draw the plate's outline once, rectangle and hole together, and push it up 5 mm. The hole comes with it, with no separate cut (@fig-extrude).

#image-equation(
  (
    // cargo run -p swx-qa --example constraint_plots -- docs/manual/img 600x420
    ("img/sketch-centered-40.png", "Profile: rectangle with a hole"),
    // cargo run -p swx-kernel --features occt --example extrude_plate -- extruded-plate.step
    // swx-qa render --step extruded-plate.step --size 900x700 -o docs/manual/img/extruded-plate.png
    ("img/extruded-plate.png", "Extruded 5 mm"),
  ),
  ("→",),
  caption: [An extrude pushes a flat profile straight out. The circle inside the rectangle becomes a hole all the way through.],
) <fig-extrude>

Two more words come with it:

- The #term[workplane] is the flat plane the outline is drawn on. It has its own left–right and up–down directions, and a third direction straight out of it, called its _normal_. The extrude goes along the normal.
- The #term[extent] says how far the extrude goes: a distance one way, or a length split evenly on both sides of the workplane.

*In Saulitwerx today:* the geometry engine can make a flat face from outlines of lines, arcs and circles on any workplane, and extrude it by a distance or symmetrically, as a new solid. You type the outline's pieces into a small program; a sketch's regions (@sec-profiles) can't be handed to the extrude yet, and the extrude isn't recorded in a part's history. Joining the result to a part or cutting it from one is done with the Boolean operations of @ch-solids.

== Extrude a profile <sec-extrude>

#task-intro(
  goal: [Make the plate with a hole from its outline, in one extrude, and save it as STEP.],
  needs: [Saulitwerx built with OpenCascade (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [Run the sample program, and tell it to save the part as `extruded-plate.step`:
  #term-block("cargo run -p swx-kernel --features occt --example extrude_plate -- extruded-plate.step")
  It prints the solid's measurements, then the geometry engine's report about writing the file:
  #term-block(label: "Output", "volume        4607.30 mm³\nfaces         7\nbounding box  [0.0, 0.0, 0.0] to [40.0, 25.0, 5.0]\n…  (the geometry engine's file report)\nwrote extruded-plate.step")],
  [Compare the numbers with the plate you made in @ch-first. They are the same: 4607.30 mm³, 7 faces, 40 × 25 × 5 mm. The region's area, 921.46 mm² (@sec-profiles), times the 5 mm height gives the volume.],
  [Draw the part:
  #term-block("target/release/swx-qa render --step extruded-plate.step -o extruded-plate.png")
  Open `extruded-plate.png`. You should see the plate of @fig-extrude, with the hole showing light background through it.],
)

#result[The plate with a hole, made from its outline in one step instead of a block and a cut.]

=== How the sample is built

The program is `crates/swx-kernel/examples/extrude_plate.rs`. It does three things.

#steps(
  [*Describe each loop.* A `ProfileLoop` is a list of pieces, `Segment2`, in workplane coordinates. Each piece starts where the one before it ended, and the last one ends where the first began. A full circle is a loop on its own:
  ```rust
  let corners = [[0.0, 0.0], [40.0, 0.0], [40.0, 25.0], [0.0, 25.0]];
  let outer = ProfileLoop {
      segments: (0..4)
          .map(|i| Segment2::Line { start: corners[i], end: corners[(i + 1) % 4] })
          .collect(),
  };
  let hole = ProfileLoop {
      segments: vec![Segment2::Circle { center: [20.0, 12.5], radius: 5.0 }],
  };
  ```],
  [*Make the face.* `make_face` takes the workplane, the outer loop, and a list of holes. The hole loops must lie inside the outer one:
  ```rust
  let face = k.make_face(&Workplane::XY, &outer, &[hole])?;
  ```],
  [*Extrude it.* `Extent::Distance(5.0)` pushes the face 5 mm along the workplane's normal, which for the XY plane is up, +Z:
  ```rust
  let part = k.extrude(&face, Extent::Distance(5.0))?;
  ```],
)

#tip(title: [Direction doesn't matter])[You can list a loop's pieces clockwise or counterclockwise, and any piece may run backwards. Saulitwerx puts them in order before the geometry engine sees them. What matters is that the ends meet: closer than 0.0000001 mm.]

== Choose the direction and the workplane <sec-extrude-direction>

An extrude doesn't have to go up. A negative distance goes the other way, below the workplane. A symmetric extent splits the length evenly, half on each side, which is handy for a part whose middle should sit on the plane, like a bracket centered on its mounting plate.

#task-intro(
  goal: [Extrude a slot up, down and symmetrically, and see where each result sits.],
  needs: [Saulitwerx built with OpenCascade.],
  time: [5 minutes.],
)

#steps(
  [Run the slot sample:
  #term-block("cargo run -p swx-kernel --features occt --example extrude_slot -- extruded-slot.step")
  It extrudes the same slot outline three ways, 4 mm each time, and prints each one's volume and how high it reaches:
  #term-block(label: "Output", "Distance(4.0)   1514.16 mm³, z from 0 to 4\nDistance(-4.0)  1514.16 mm³, z from -4 to 0\nSymmetric(4.0)  1514.16 mm³, z from -2 to 2\n…  (the geometry engine's file report)\nwrote extruded-slot.step")],
  [Read the three lines. The volume is the same each time; only the position changes. `Distance(4.0)` sits on the workplane and rises to 4 mm; `Distance(-4.0)` hangs below it; `Symmetric(4.0)` straddles it, 2 mm each side.],
  [Draw the first one:
  #term-block("target/release/swx-qa render --step extruded-slot.step -o extruded-slot.png")
  You should see the slot of @fig-extrude-slot: flat sides and round ends.],
)

#shot(
  // cargo run -p swx-kernel --features occt --example extrude_slot -- extruded-slot.step
  // swx-qa render --step extruded-slot.step --size 900x700 -o docs/manual/img/extruded-slot.png
  "img/extruded-slot.png",
  width: 55%,
  caption: [The slot outline, two lines and two half-circles, extruded 4 mm.],
  alt: "A long flat bar with rounded ends, seen from above at an angle",
) <fig-extrude-slot>

#result[You can place an extrude above, below or across its workplane.]

The slot's outline mixes lines and arcs. An arc, `Segment2::Arc`, runs counterclockwise from `start_angle` to `end_angle`, in radians measured from the workplane's left–right axis, just as in a sketch (@sec-slot). The right-hand end of the slot is the arc from −π/2 to π/2 around (30, 5).

Three workplanes are built in. To draw on another plane, give your own `Workplane` with an `origin`, an `x_dir` and a `normal` at right angles to it (@sec-ref-extrude).

#ref-table(
  columns: (auto, auto, auto, auto),
  header: ("Workplane", "Left–right", "Up–down", "Extrude goes"),
  [`Workplane::XY`], [+X], [+Y], [+Z],
  [`Workplane::XZ`], [+X], [+Z], [−Y],
  [`Workplane::YZ`], [+Y], [+Z], [+X],
)

#caution(title: [XZ extrudes toward −Y])[Looking down on the XZ plane so that X runs right and Z runs up, you are looking from the −Y side, and that is where a positive distance goes. If a part comes out on the wrong side, use a negative distance.]

== Common mistakes

- *The ends don't quite meet.* A loop whose last piece stops short of the first is refused with `outer loop: consecutive segments don't meet within 0.0000001`. Compute shared corners once and reuse them, as `extrude_plate` does with `corners`.
- *A circle mixed with other pieces.* A full circle must be a loop of its own. To round a corner, use an arc.
- *A hole outside the outline, or touching its edge.* The geometry engine refuses the face: `make_face failed: the face is invalid …`. Keep every hole wholly inside the outer loop.
- *An outline that crosses itself,* like a figure eight. Same message; or, when the two halves are the same size, `outer loop encloses no area`, because their areas cancel out. Split it into separate outlines.
- *A distance of zero.* An extrude has to go somewhere: `extrude distance must be finite and non-zero, got 0`.
