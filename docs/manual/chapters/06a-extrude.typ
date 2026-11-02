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
    // swx-qa render --step extruded-plate.step --size 1200x840 -o docs/manual/img/extruded-plate.png
    ("img/extruded-plate.png", "Extruded 5 mm"),
  ),
  ("→",),
  caption: [An extrude pushes a flat profile straight out. The circle inside the rectangle becomes a hole all the way through.],
) <fig-extrude>

Two more words come with it:

- The #term[workplane] is the flat plane the outline is drawn on. It has its own left–right and up–down directions, and a third direction straight out of it, called its _normal_. The extrude goes along the normal.
- The #term[extent] says how far the extrude goes: a distance one way, or a length split evenly on both sides of the workplane.

*In Saulitwerx today:* the geometry engine can make a flat face from outlines of lines, arcs and circles on any workplane, and extrude it by a distance or symmetrically. The result can be a new solid, or be joined to a part, cut from it or intersected with it in the same step (@sec-extrude-modes). A cut can also go through the whole part, however thick it is (@sec-through-all). You type the outline's pieces into a small program; a sketch's regions (@sec-profiles) can't be handed to the extrude yet, and the extrude isn't recorded in a part's history.

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

#shot(
  // cargo run -p swx-kernel --features occt --example extrude_slot -- extruded-slot.step
  // swx-qa render --step extruded-slot.step --size 1200x840 -o docs/manual/img/extruded-slot.png
  "img/extruded-slot.png",
  width: 36%,
  caption: [The slot you will make: two lines and two half-circles, extruded 4 mm.],
  alt: "A long flat bar with rounded ends, seen from above at an angle",
) <fig-extrude-slot>

#steps(
  [Run the slot sample:
  #term-block("cargo run -p swx-kernel --features occt --example extrude_slot -- extruded-slot.step")
  It extrudes the slot three ways, 4 mm each, and prints each result's volume and height:
  #term-block(label: "Output", "Distance(4.0)   1514.16 mm³, z from 0 to 4\nDistance(-4.0)  1514.16 mm³, z from -4 to 0\nSymmetric(4.0)  1514.16 mm³, z from -2 to 2\n…  (the geometry engine's file report)\nwrote extruded-slot.step")],
  [Read the three lines. The volume is the same each time; only the position changes. `Distance(4.0)` sits on the workplane and rises to 4 mm; `Distance(-4.0)` hangs below it; `Symmetric(4.0)` straddles it, 2 mm each side.],
  [Draw the first one:
  #term-block("target/release/swx-qa render --step extruded-slot.step -o extruded-slot.png")
  You should see the slot of @fig-extrude-slot: flat sides and round ends.],
)

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

== Join, cut or intersect with an extrude <sec-extrude-modes>

#idx("extrude mode")#idx("target body")
Most extrudes change a part that already exists. A boss stands up from a plate; a pocket sinks into it. You could extrude a new solid and then combine it with a Boolean operation (@ch-solids), but an extrude can do both in one step. Its #term[extrude mode] says what happens to the extruded shape:

#ref-table(
  columns: (auto, 1fr),
  header: ("Mode", "What you get"),
  [New solid], [The extruded shape on its own. The part is left as it was.],
  [Join], [The part with the extruded shape added to it.],
  [Cut], [The part with the extruded shape taken out of it.],
  [Intersect], [Only the part's material that lies inside the extruded shape.],
)

The part an extrude joins, cuts or intersects is its #term[target body]. A good habit: draw the outline on a face of the target, then extrude away from the part to add material, and into it to remove material. @fig-extrude-modes shows one block and the three combining modes.

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example extrude_modes -- modes
    // swx-qa render --step modes/modes-<name>.step --size 900x700 -o docs/manual/img/modes-<name>.png
    ("img/modes-target.png", "Target: 40 × 25 × 10 mm block", "A plain rectangular block"),
    ("img/modes-join.png", "Join: circle, 5 mm up", "The block with a short round post standing on top"),
    ("img/modes-cut.png", "Cut: rectangle, 4 mm down", "The block with a shallow rectangular pocket in its top"),
    ("img/modes-intersect.png", "Intersect: circle, 10 mm down", "A round disc with two straight flat sides"),
  ),
  caption: [One target and three extrude modes, each outline drawn on the block's top face. The intersect keeps the part of the block inside a 30 mm circle: the block's long sides trim the circle flat. Each picture is scaled to fill its frame.],
) <fig-extrude-modes>

#task-intro(
  goal: [Add a boss to a block, sink a pocket into it, and trim it to a circle, each with one extrude.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [10 minutes.],
)

#steps(
  [Run the sample program, and tell it to save its parts in a folder called `modes`:
  #term-block("cargo run -p swx-kernel --features occt --example extrude_modes -- modes")
  It prints a line for the block and one for each mode, then two errors on purpose, then the files it wrote:
  #term-block(label: "Output", "target     10000.00 mm³   6 faces  z from 0 to 10\njoin       10392.70 mm³   8 faces  z from 0 to 15\ncut         9200.00 mm³  11 faces  z from 0 to 10\nintersect   6505.89 mm³   7 faces  z from 0 to 10\njoin, no target -> error: invalid input: extrude mode \"join\" needs a target body\ncut, all of it -> error: empty result: extrude cut leaves no solid (volume 0e0)\n…  (the geometry engine's file reports)\nwrote modes/modes-target.step\nwrote modes/modes-join.step\nwrote modes/modes-cut.step\nwrote modes/modes-intersect.step")],
  [Read the `join` line. The block grew by the boss: a 10 mm circle 5 mm tall adds 392.70 mm³, and the part now reaches 15 mm up.],
  [Read the `cut` line. A 20 × 10 mm rectangle cut 4 mm deep removes 800 mm³. The height stays 10 mm, and the pocket's floor and four walls bring the faces to 11.],
  [Read the `intersect` line. What is left is the block's material inside a 30 mm circle: less than either one on its own.],
  [Draw the cut result:
  #term-block("target/release/swx-qa render --step modes/modes-cut.step -o modes-cut.png")
  Open `modes-cut.png`. You should see the block with a pocket in its top, as in @fig-extrude-modes. Draw the other files the same way.],
)

#result[A boss, a pocket and a trimmed block, each made by one extrude against the same target. The block itself is never changed: each mode returns a new solid.]

=== How the sample is built

The program is `crates/swx-kernel/examples/extrude_modes.rs`. The outlines sit on the block's top face, a workplane at z = 10:

```rust
const TOP: Workplane = Workplane {
    origin: [0.0, 0.0, 10.0],
    x_dir: [1.0, 0.0, 0.0],
    normal: [0.0, 0.0, 1.0],
};
```

Each mode is one call to `extrude_feature`. It takes the face, the extent, the mode and the target. The join goes up, +5 mm, away from the block; the cut goes down, −4 mm, into it:

```rust
let joined = extrude_feature(&k, &boss, Extent::Distance(5.0),
    ExtrudeMode::Join, Some(&block))?;
let cut = extrude_feature(&k, &pocket, Extent::Distance(-4.0),
    ExtrudeMode::Cut, Some(&block))?;
```

For `ExtrudeMode::NewSolid` the target isn't used, so pass `None`. That gives the same solid as `extrude` (@sec-extrude).

#tip(title: [Which way is "into" the part?])[The extrude goes along the workplane's normal for a positive distance. On a top face whose normal points up, a cut needs a negative distance. If a cut leaves the part unchanged, it probably went the wrong way, into empty space.]

#caution(title: [A result must have something left])[A cut that removes the whole target, or an intersect whose shapes don't overlap, leaves nothing. That is an error, `empty result: …`, not an empty part. The sample's last line shows one: a rectangle larger than the block, cut all the way through.]

== Cut through the whole part <sec-through-all>

#idx("through all")
Holes for bolts and pins usually go all the way through a part. You could measure the part and cut exactly that deep, but if the part later gets thicker, the hole would stop short. A #term[through-all] cut has no depth of its own: it goes right through the target body, whatever its size.

A through-all cut still needs a direction. It can go along the workplane's normal, against it, or both ways at once. Both ways is the one to use when the outline sits on a plane in the middle of the part (@fig-through-all).

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example extrude_through_all -- through
    // swx-qa render --step through/through-<name>.step --size 900x700 -o docs/manual/img/through-<name>.png
    ("img/through-down.png", "Down from the top face (reversed)", "A rectangular block with one round hole straight down through it"),
    ("img/through-both.png", "Then both ways from the middle", "The same block with a second, smaller hole running sideways through it"),
  ),
  caption: [Two through-all cuts in a 40 × 25 × 10 mm block. The first hole is drawn on the top face and cut against its normal, down. The second is drawn on a plane through the middle of the block and cut both ways, out of the front and the back.],
) <fig-through-all>

#task-intro(
  goal: [Cut two holes right through a block, one from its top face and one from a plane in its middle.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [Run the sample program, and tell it to save its parts in a folder called `through`:
  #term-block("cargo run -p swx-kernel --features occt --example extrude_through_all -- through")
  It prints a line for the block and one for each cut, then two errors on purpose, then the files it wrote:
  #term-block(label: "Output", "block            10000.00 mm³   6 faces\ndown (reversed)   9497.35 mm³   7 faces\nboth ways         8790.49 mm³   8 faces\nup (normal) -> error: invalid input: the through-all cut misses the target: it removes nothing\njoin -> error: invalid input: a through-all extent works only with the cut mode, not \"join\"\n…  (the geometry engine's file reports)\nwrote through/through-down.step\nwrote through/through-both.step")],
  [Read the `down` line. An 8 mm hole through 10 mm of block removes 502.65 mm³, and adds one face: the hole's wall.],
  [Read the `both ways` line. A 6 mm hole through the block's 25 mm width removes another 706.86 mm³. The cut started in the middle of the block and went out through both sides.],
  [Read the first error. From the top face, the normal points up, away from the block. A through-all cut that way finds nothing to remove, and says so.],
  [Draw the result with both holes:
  #term-block("target/release/swx-qa render --step through/through-both.step -o through-both.png")
  Open `through-both.png`. You should see the block of @fig-through-all, with a hole down through its top and a smaller one through its side.],
)

#result[Two holes that go right through the block. If the block were 50 mm thick instead of 10, the same two cuts would still go through it.]

=== How the sample is built

The program is `crates/swx-kernel/examples/extrude_through_all.rs`. The extent is `Extent::ThroughAll`, with a direction:

#ref-table(
  columns: (auto, 1fr),
  header: ("Direction", "The cut goes"),
  [`ThroughDirection::Normal`], [Along the workplane's normal.],
  [`ThroughDirection::Reversed`], [Against the normal.],
  [`ThroughDirection::Both`], [Both ways from the workplane.],
)

The first hole is drawn on the top face, so it goes against the normal, down into the block:

```rust
let drilled = extrude_feature(&k, &top_hole,
    Extent::ThroughAll(ThroughDirection::Reversed),
    ExtrudeMode::Cut, Some(&block))?;
```

The second is drawn on a workplane through the middle of the block, at y = 12.5, and cut from the result of the first:

```rust
let cross = extrude_feature(&k, &cross_hole,
    Extent::ThroughAll(ThroughDirection::Both),
    ExtrudeMode::Cut, Some(&drilled))?;
```

#note(title: [How far is "all"?])[Saulitwerx measures the outline and the target, and makes the cut a little longer than the distance needed to clear the whole target. That is why a through-all extent works only with `extrude_feature` and the cut mode: it needs a target to measure. `extrude` on its own refuses it.]

#tip(title: [A missed cut is an error])[If a through-all cut removes nothing, Saulitwerx refuses it instead of handing back the part unchanged, so a cut that went the wrong way never passes unnoticed. Change the direction, or move the outline over the part.]

== Common mistakes

#set par(justify: false)

- *The ends don't quite meet.* A loop whose last piece stops short of the first is refused with `outer loop: consecutive segments don't meet within 0.0000001`. Compute shared corners once and reuse them, as `extrude_plate` does with `corners`.
- *A circle mixed with other pieces.* A full circle must be a loop of its own. To round a corner, use an arc.
- *A hole outside the outline, or touching its edge.* The geometry engine refuses the face: `make_face failed: the face is invalid …`. Keep every hole wholly inside the outer loop.
- *An outline that crosses itself,* like a figure eight. Same message; or, when the two halves are the same size, `outer loop encloses no area`, because their areas cancel out. Split it into separate outlines.
- *A distance of zero.* An extrude has to go somewhere: `extrude distance must be finite and non-zero, got 0`.
- *Join, cut or intersect with no target.* These modes need a part to work on: `invalid input: extrude mode "cut" needs a target body`. Pass the part as `Some(&part)`, or use `ExtrudeMode::NewSolid`.
- *A cut that goes the wrong way.* It misses the part, and the part comes back unchanged. Check the sign of the distance (@sec-extrude-modes).
- *A cut that removes everything,* or an intersect that doesn't overlap: `empty result: extrude cut leaves no solid …`. Make the outline smaller than the face it cuts, or move it over the part.
- *A through-all extent with another mode.* Only a cut can go through all: `a through-all extent works only with the cut mode, not "join"`. Give a distance for a join or an intersect.
- *A through-all cut the wrong way.* It misses the part: `the through-all cut misses the target: it removes nothing`. On a top face, use `ThroughDirection::Reversed`, or `Both` (@sec-through-all).
