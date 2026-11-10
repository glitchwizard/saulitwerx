#import "../template.typ": *

= Work planes, axes and points <ch-work>

#for-developers[This chapter calls `swx-kernel` from Rust. Work features built from numbers need no geometry engine, so its first sample runs on a plain build; the others need OpenCascade and the `occt` build feature (@ch-install).]

#idx("work feature")
Every sketch sits on a plane, every revolve turns about an axis, and every hole starts at a point. Often a face of the part gives you what you need: you sketch on the top, or drill into the side. But sometimes the plane, the line or the point you need isn't on the part at all: a plane 10 mm above the top, a plane tilted 30° to hold a sloping post, the line where two faces would meet. So you build it.

That reference geometry is called a #term[work feature]. It has no material of its own, so it never adds to or takes from the part; it is there to place other features. There are three kinds:

- A #term[work plane]: a flat plane to sketch on, extrude from or measure to.
- A #term[work axis]: a straight line, to revolve about or line things up on.
- A #term[work point]: one point in space, to start a hole at or measure from.

Every part starts with a set of them, all meeting at the #term[part origin], the point (0, 0, 0): three planes, XY, XZ and YZ, and three axes, X, Y and Z, along their edges. You build new ones from those, or from each other (@fig-tilted-post).

// cargo run -p swx-kernel --features occt --example tilted_post -- work
// swx-qa render --step work/tilted-post.step --size 900x700 -o docs/manual/img/tilted-post.png
#shot(
  "img/tilted-post.png",
  width: 65%,
  alt: "A square plate with a round post rising from the middle of its top, leaning over at 30 degrees",
  caption: [A post that leans at 30°. Its circle is sketched on a work plane: the plate's top plane, made by offsetting the XY plane 10 mm, then turned 30° about a work axis across the middle of the top.],
) <fig-tilted-post>

*In Saulitwerx today:* work planes, axes and points can be built from numbers, from the part origin and its planes and axes, and from each other. A work plane can be used wherever a `Workplane` goes (@sec-ref-extrude), and a work axis wherever an `Axis` goes: a revolve, a draft or a hole. They can also be taken from a part's own geometry: a plane on a flat face or touching a round one, an axis along a straight edge or through a round face, a point at a corner or halfway along an edge (@sec-work-part, @sec-work-between). A plane can also sit halfway between two others. You name that face, edge or corner by its number (@sec-topology); it can't be picked with the mouse yet, and the number doesn't follow the part through later changes. Work features aren't recorded in a part's history.

== Build work features <sec-work>

The sample in this section builds one of each kind of work feature from the part origin and its planes, then shows the errors you can meet.

#task-intro(
  goal: [Build an offset plane, a plane through three points, a tilted plane, two axes and a point, and see the errors.],
  needs: [Saulitwerx built from source (@ch-install). OpenCascade isn't needed.],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program:
  #term-block("cargo run -p swx-kernel --example work_features")
  It prints each work feature it builds, then five errors on purpose:
  #term-block(label: "Output", "part origin            (0, 0, 0)\nZ axis                 through (0, 0, 0)  along (0, 0, 1)\nXY plane               origin (0, 0, 0)  normal (0, 0, 1)\nXY offset 10           origin (0, 0, 10)  normal (0, 0, 1)\nthrough 3 points       origin (0, 0, 10)  normal (0, -0.447, 0.894)\ntop turned 30°         origin (0, 2.679, 0)  normal (0, -0.5, 0.866)\nslope meets YZ         through (0, -4, 8)  along (0, 0.894, 0.447)\nthrough 2 points       through (0, 0, 0)  along (0.816, 0.408, 0.408)\ndiagonal meets top     (20, 10, 10)\ntwo parallel planes -> error: invalid input: the planes are parallel:\n    they have no line in common\n3 points in a line -> error: invalid input: the three plane points are\n    collinear\n2 points the same -> error: invalid input: axis direction (the points\n    coincide) must be non-zero\nX axis meets top -> error: invalid input: the axis is parallel to the plane\nturn about Z axis -> error: invalid input: the axis does not lie in the\n    plane")
  Four long error lines are wrapped here to fit the page.]],
  [Read the `XY offset 10` line. The plane has moved 10 mm up and still faces up. It is the top of a 10 mm plate.],
  [Read the `through 3 points` line. The points (0, 0, 10), (40, 0, 10) and (0, 20, 20) make a slope that rises 10 mm over 20 mm, so its normal leans back from straight up: (0, −0.447, 0.894).],
  [Read the `top turned 30°` line. The top plane turned 30° about a line along X through (0, 20, 10). Its normal now leans 30° from straight up: (0, −0.5, 0.866), since sin 30° = 0.5 and cos 30° = 0.866. Turning the plane moved its origin too, to (0, 2.679, 0); the turning line stays put.],
  [Read the `slope meets YZ` line. It is the line where the slope meets the YZ plane. Its point is the one on the line nearest the origin.],
  [Read the last two lines before the errors. The axis from (0, 0, 0) towards (40, 20, 20) crosses the top plane at (20, 10, 10), halfway along.],
  [Read the errors. Each says why the work feature can't be built.],
)

#result[Six new work features, built from the origin's planes and axes and from each other.]

Here are all the ways to build each kind. Every one checks its input and gives an error if the result wouldn't make sense, such as a plane through three points in a line.

#ref-table(
  columns: (auto, 1fr),
  header: ("To get", "Build it with"),
  [The origin and its planes and axes], [`ORIGIN`, `Workplane::XY`, `::XZ`, `::YZ`, `Axis::X`, `::Y`, `::Z`.],
  [A plane parallel to another], [`plane.offset(distance)`: moved along its normal by `distance` mm; a negative distance goes the other way.],
  [A plane through three points], [`Workplane::through_points(p0, p1, p2)`. Its origin is `p0`; its normal follows the right-hand rule from `p0 → p1` to `p0 → p2`.],
  [A plane through a point, square to a direction], [`Workplane::from_point_normal(p, n)`.],
  [A plane at an angle to another], [`plane.angled(axis, angle)`: turned about an axis that lies in the plane, by an angle in radians (right-hand rule).],
  [An axis through two points], [`Axis::through_points(p, q)`, pointing from `p` to `q`.],
  [The axis where two planes meet], [`Axis::plane_intersection(a, b)`.],
  [The point where an axis crosses a plane], [`plane_axis_intersection(plane, axis)`.],
  [A plane halfway between two parallel planes], [`Workplane::midplane(a, b)`, facing the way `a` faces (@sec-work-between).],
)


== Place a feature on a work plane <sec-work-use>

A work plane is used like any other plane. The sample in this section builds the leaning post of @fig-tilted-post: it sketches a circle on a tilted work plane and extrudes it into the plate.

#task-intro(
  goal: [Stand a post at 30° on a plate, using a tilted work plane.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its part in a folder called `work`:
  #term-block("cargo run -p swx-kernel --features occt --example tilted_post -- work")
  It prints the tilted plane and the finished part:
  #term-block(label: "Output", "tilted plane: origin [0.0, 2.6794919243112254, 1.7763568394002505e-15],\n    normal [0.000, -0.500, 0.866]\nplate with post: 18261.95 mm³, 8 faces\n…  (the geometry engine's file reports)\nwrote work/tilted-post.step")
  The long first line is wrapped here to fit the page. The tiny number at the end of the origin is zero, give or take rounding.]],
  [Read the `plate with post` line. The plate is 40 × 40 × 10 mm, 16000 mm³. The post is 12 mm across and stands 20 mm out of the plate along its leaning axis, which adds π × 6² × 20 = 2261.95 mm³.],
  [#block(breakable: false)[Draw the part:
  #term-block("target/release/swx-qa render --step work/tilted-post.step -o tilted-post.png")
  Open `tilted-post.png`. You should see the part of @fig-tilted-post.]],
)

#result[A post standing at an angle that no face of the plate could give you.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/tilted_post.rs`. It does three things.

#steps(
  [*Build the work plane.* The plate's top plane is the XY plane moved up 10 mm. The work axis runs along X across the middle of the top, at y = 20. The tilted plane is the top turned 30° about it:
  ```rust
  let top = Workplane::XY.offset(10.0)?;
  let across = Axis { origin: [0.0, 20.0, 10.0], dir: [1.0, 0.0, 0.0] };
  let tilted = top.angled(across, 30f64.to_radians())?;
  ```],
  [*Sketch on it.* The circle goes on the plane at (u, v) = (20, 20): 20 mm along X, and 20 mm from the plane's origin, which is where the turning line is. The sample starts it on a plane 4 mm below the tilted one, so the whole foot of the post is buried in the plate:
  ```rust
  let foot = tilted.offset(-4.0)?;
  let face = k.make_face(&foot, &circle, &[])?;
  ```],
  [*Extrude and join.* It extrudes 24 mm along the tilted normal and joins the post to the plate (@sec-extrude-modes):
  ```rust
  let part = extrude_feature(&k, &face, Extent::Distance(24.0),
                             ExtrudeMode::Join, Some(&plate))?;
  ```],
)
]

#caution(title: [Leave no gap under a leaning feature])[A circle on a tilted plane rises on one side above the plane it turned from. Started right on the tilted plane, the post's high side would float above the plate, and the join would leave a gap. Start the sketch a little below, as the sample does, so the foot is inside the part.]

== Take work features from a part <sec-work-part>

Often the plane or line you need is already on the part: the face you want to sketch on, the edge you want to turn about, the middle of a round boss you want to drill down. Instead of working out its numbers, ask the geometry engine for it. The sample in this section takes all three kinds from a block with a round boss on top, then uses two of them (@fig-work-on-part).

// cargo run -p swx-kernel --features occt --example work_on_part -- work
// swx-qa render --step work/work-on-part.step --size 900x700 -o docs/manual/img/work-on-part.png
#shot(
  "img/work-on-part.png",
  width: 65%,
  alt: "A rectangular block with a round boss on its top, drilled through its middle, and a short round post sticking out of the middle of one side",
  caption: [A 40 × 30 × 20 mm block with a boss 12 mm across on top. The post on the side is sketched on a work plane taken from that face; the 6 mm hole runs down a work axis taken from the boss's round face.],
) <fig-work-on-part>

#task-intro(
  goal: [Take a work plane, two work axes and a work point from a part, then sketch on the plane and drill along an axis.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its part in a folder called `work`:
  #term-block("cargo run -p swx-kernel --features occt --example work_on_part -- work")
  It prints the numbers of the faces, edges and corner it uses, the work features it takes from them, four errors on purpose, and the finished part:
  #term-block(label: "Output", "side face 5, top edge 8, round face 6, rim edge 13, corner vertex 6\nplane on side face     origin (40, 0, 0)  normal (1, 0, 0)  x_dir (0, 1, 0)\naxis on top edge       through (40, 0, 20)  along (0, 1, 0)\naxis on round face     through (20, 15, 0)  along (0, 0, 1)\npoint at corner        (40, 30, 20)\nplane on edge -> error: invalid input: face_plane needs a face, got a\n    Edge sub-shape\nplane on round face -> error: invalid input: face 6 is not planar\naxis on corner -> error: invalid input: work_axis_on needs an edge or a\n    face, got a vertex\naxis on rim -> error: invalid input: Edge 13 is not a straight edge or a\n    cylindrical face\nwith post and hole: 24911.06 mm³, 11 faces\n…  (the geometry engine's file reports)\nwrote work/work-on-part.step")
  Three long error lines are wrapped here to fit the page.]],
  [Read the `plane on side face` line. The plane lies on the block's end at x = 40 and faces out of the part, along +X. Its origin is the point of the plane nearest the part origin, (40, 0, 0), and its u direction runs along +Y.],
  [Read the two `axis` lines. The first runs along the top edge of that face, in the Y direction. The second is the boss's center line: straight up through (20, 15). Each passes through its point nearest the part origin, which for the boss is down at z = 0, on the bottom of the block.],
  [Read the `point at corner` line: the top corner at the far end of the block.],
  [Read the errors. A plane needs a flat face, and an axis needs a straight edge or a round (cylindrical) face. A corner has no line, and the boss's round top edge is a circle, not a line.],
  [Read the last line. The block (24000 mm³) and the boss (1130.97 mm³) gained a post 10 mm across and 8 mm long (628.32 mm³), and lost a hole 6 mm across through 30 mm (848.23 mm³): 24911.06 mm³.],
  [#block(breakable: false)[Draw the part:
  #term-block("target/release/swx-qa render --step work/work-on-part.step -o work-on-part.png")
  Open `work-on-part.png`. You should see the part of @fig-work-on-part.]],
)

#result[Work features that sit exactly on the part, with no numbers worked out by hand, and two features placed on them.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/work_on_part.rs`. It does three things.

#steps(
  [*Find the face, edge or corner.* Each one is a `SubShape`: its kind and its number in the part (@sec-topology). The sample finds each by its bounding box, with `sub_shape_bounding_box`.],
  [*Take the work features.* One call for each kind:
  ```rust
  let plane = k.work_plane_on_face(&part, side)?;
  let along = k.work_axis_on(&part, edge)?;
  let boss_axis = k.work_axis_on(&part, round)?;
  let corner_point = k.vertex_point(&part, corner)?;
  ```],
  [*Use them.* The post is a circle sketched on `plane` at (u, v) = (15, 10), the middle of the face, extruded 8 mm out and joined (@sec-extrude-modes). The hole is drilled through all along `boss_axis` (@ch-hole):
  ```rust
  let face = k.make_face(&plane, &circle, &[])?;
  let with_post = extrude_feature(&k, &face, Extent::Distance(8.0),
                                  ExtrudeMode::Join, Some(&part))?;
  let finished = k.hole(&with_post, boss_axis,
                        HoleKind::Simple { diameter: 6.0 }, HoleDepth::ThroughAll)?;
  ```],
)
]

#caution(title: [An axis from the part can point either way])[An axis taken from an edge or a round face runs along it, but which of its two directions you get isn't fixed. A hole or a revolve cares which way the axis points, so check it and flip it when needed, as the sample does before drilling: it makes the boss's axis point up, into the part, from its start on the bottom face.]

#tip(title: [A plane on a face faces out])[A plane taken from a face always faces out of the part, like the face itself. Sketch on it and extrude a positive distance to build outward from the face; extrude a negative distance, or cut, to go into the part.]

== Midplanes, tangent planes and midpoints <sec-work-between>

Some places on a part are easy to describe but awkward to work out by hand. The middle of a shaft. The flat that touches its round side. The middle of an edge. Three calls give you each one exactly:

- A #term[midplane] is the plane halfway between two parallel planes. Features placed on it sit in the middle of the part, such as a cross hole halfway along a shaft.
- A #term[tangent plane] touches a round face along one straight line without cutting into it, on the side you choose. Sketch on it to put a flat, a pad or a label on a shaft or a boss.
- An #term[edge midpoint] is the point halfway along an edge, measured along the edge. On a curved edge it lies on the curve.

The sample in this section uses all three on a shaft 16 mm across and 50 mm long (@fig-work-midplane).

// cargo run -p swx-kernel --features occt --example work_midplane -- work
// swx-qa render --step work/work-midplane.step --size 600x900 -o docs/manual/img/work-midplane.png
// Numbered rings drawn by the template.
#shot(
  "img/work-midplane.png",
  width: 40%,
  alt: "An upright round shaft with a small round hole through its side halfway up, and a flat cut into the other side along its top end",
  marks: (
    (68pt, 161pt, [1]),
    (119pt, 93pt, [2]),
    (119pt, 51pt, [3]),
  ),
  caption: [A shaft with (1) a cross hole on the midplane between its ends, (2) a flat cut from a plane tangent to its side, and (3) the midpoint of the flat's top edge.],
) <fig-work-midplane>

#task-intro(
  goal: [Take a midplane, a tangent plane and an edge midpoint from a shaft, and drill and cut on them.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its part in a folder called `work`:
  #term-block("cargo run -p swx-kernel --features occt --example work_midplane -- work")
  It prints the faces it uses, the two planes, four errors on purpose, the finished part and the midpoint:
  #term-block(label: "Output", "bottom face 2, top face 1, round face 0\nmidplane               origin (0, 0, 25)  normal (0, 0, -1)\ntangent plane (+Y)     origin (0, 8, 0)  normal (0, 1, 0)  x_dir (1, 0, 0)\nmidplane of XY and XZ -> error: invalid input: the planes are not\n    parallel: they have no midplane\ntangent on top face -> error: invalid input: face 1 is not cylindrical\ntangent toward +Z -> error: invalid input: toward is parallel to the\n    cylinder axis: it picks no side\nmidpoint of a face -> error: invalid input: edge_midpoint needs an edge,\n    got a Face sub-shape\nshaft with hole and flat: 9452.69 mm³, 6 faces\nflat's top edge 12: midpoint (0, 6, 50)\n…  (the geometry engine's file report)\nwrote work/work-midplane.step")
  Three long error lines are wrapped here to fit the page.]],
  [Read the `midplane` line. It lies at z = 25, halfway up the 50 mm shaft. It was built from the planes of the bottom and the top faces, and it faces the way the first one does: the bottom face faces out of the part, down, so the normal is (0, 0, −1).],
  [Read the `tangent plane` line. Asked for the +Y side, the plane touches the shaft at y = 8, its radius, and faces out along +Y.],
  [Read the errors. A midplane needs two parallel planes; XY and XZ cross. A tangent plane needs a round face, and a side that points away from the axis, not along it. A midpoint needs an edge.],
  [Read the `shaft with hole and flat` line. The shaft holds π × 8² × 50 = 10053.10 mm³. The flat, 2 mm deep along the top 20 mm, removes 290.12 mm³. The rest of the difference, 310.29 mm³, is the cross hole 5 mm across: a little less than π × 2.5² × 16 = 314.16 mm³, because the shaft's round side trims the ends of the hole.],
  [Read the last line before the file. The flat's top edge runs straight across the shaft's top at y = 6. Halfway along, it crosses the Y axis: (0, 6, 50).],
  [#block(breakable: false)[Draw the part:
  #term-block("target/release/swx-qa render --step work/work-midplane.step -o work-midplane.png")
  Open `work-midplane.png`. You should see the shaft of @fig-work-midplane.]],
)

#result[A cross hole exactly halfway along the shaft, and a flat exactly 2 mm deep, with no positions worked out by hand.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/work_midplane.rs`. It does three things.

#steps(
  [*The midplane and the cross hole.* The two end faces give two planes (@sec-work-part), and `midplane` gives the one between them. The hole runs along the line where the midplane meets the XZ plane, starting outside the shaft at x = −10 and pointing into it (@ch-hole):
  ```rust
  let mid = Workplane::midplane(&a, &b)?;
  let across = Axis::plane_intersection(&mid, &Workplane::XZ)?;
  ```],
  [*The tangent plane and the flat.* `work_plane_tangent` takes the round face and the side to touch. The sample moves the plane 2 mm into the shaft, sketches a rectangle on it over the top 20 mm, and cuts outward 5 mm, away from the axis (@sec-extrude-modes):
  ```rust
  let tangent = k.work_plane_tangent(&shaft, side, [0.0, 1.0, 0.0])?;
  let floor = tangent.offset(-2.0)?;
  ```],
  [*The midpoint.* The sample finds the flat's top edge by its bounding box, then asks for its middle:
  ```rust
  let middle = k.edge_midpoint(&finished, edge)?;
  ```],
)
]

#caution(title: [Mind which way v runs])[On a plane, u runs along `x_dir` and v along the normal × `x_dir` (@sec-ref-extrude). For the tangent plane here, that is −Z: down the shaft. So the top 20 mm of the shaft is v from −51 to −30, not 30 to 51. When a sketch on a work plane lands in the wrong place, check the directions the program prints.]

#tip(title: [A midplane faces the way the first plane does])[`midplane(a, b)` keeps `a`'s normal and directions, and swapping the two planes flips it. When a feature on the midplane goes the wrong way, swap them, or extrude a negative distance.]

== Common mistakes <sec-work-mistakes>

#set par(justify: false)

- *Two parallel planes.* They never meet, so there's no line: `invalid input: the planes are parallel: they have no line in common`.
- *Three points in a line.* Many planes pass through them, so none is chosen: `invalid input: the three plane points are collinear`. The same if two of them are the same point.
- *Two points the same.* An axis needs two different points: `invalid input: axis direction (the points coincide) must be non-zero`.
- *An axis parallel to a plane.* It never crosses it: `invalid input: the axis is parallel to the plane`.
- *Turning a plane about an axis that isn't in it.* `invalid input: the axis does not lie in the plane`. The axis must lie on the plane: build it from two points on the plane, or where the plane meets another.
- *An angle in degrees.* `angled` takes radians: write `30f64.to_radians()`.
- *A plane on something other than a flat face.* An edge or a corner gives `invalid input: face_plane needs a face, got a Edge sub-shape` (or `Vertex`): the call is `work_plane_on_face`, though the message names the `face_plane` check it shares. A curved face gives `invalid input: face 6 is not planar`.
- *An axis on a corner, a curved edge or a face that isn't round.* `invalid input: work_axis_on needs an edge or a face, got a vertex`, or `Edge 13 is not a straight edge or a cylindrical face`. Pick a straight edge, or a round face such as a boss, a pin or a hole.
- *A point on something other than a corner.* `invalid input: vertex_point needs a vertex, got a Edge sub-shape` (or `Face`).
- *A midplane of planes that cross.* `invalid input: the planes are not parallel: they have no midplane`. Take the planes of two opposite faces.
- *A tangent plane on a face that isn't round.* `invalid input: face 1 is not cylindrical`. Give the round side of a shaft, a boss or a hole.
- *A tangent side along the axis.* `invalid input: toward is parallel to the cylinder axis: it picks no side`. Point `toward` out from the axis, to the side you want.
- *A midpoint of something other than an edge.* `invalid input: edge_midpoint needs an edge, got a Face sub-shape` (or `Vertex`).
- *A number from another part, or from before a change.* `invalid input: face index 14 is out of range (the shape has 14 faces)`, or a work feature on the wrong face. Find the face, edge or corner again in the part you are building on.
