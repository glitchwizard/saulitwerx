#import "../template.typ": *

= Work planes, axes and points <ch-work>

#for-developers[This chapter calls `swx-kernel` from Rust. The work features themselves need no geometry engine, so its first sample runs on a plain build; the second needs OpenCascade and the `occt` build feature (@ch-install).]

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

*In Saulitwerx today:* work planes, axes and points can be built from numbers, from the part origin and its planes and axes, and from each other. A work plane can be used wherever a `Workplane` goes (@sec-ref-extrude), and a work axis wherever an `Axis` goes: a revolve, a draft or a hole. No call builds one straight from a face or an edge of a part yet, though `face_plane` (@sec-extrude-to-face) gives the point and normal that `from_point_normal` takes. Work features aren't recorded in a part's history.

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

== Common mistakes <sec-work-mistakes>

#set par(justify: false)

- *Two parallel planes.* They never meet, so there's no line: `invalid input: the planes are parallel: they have no line in common`.
- *Three points in a line.* Many planes pass through them, so none is chosen: `invalid input: the three plane points are collinear`. The same if two of them are the same point.
- *Two points the same.* An axis needs two different points: `invalid input: axis direction (the points coincide) must be non-zero`.
- *An axis parallel to a plane.* It never crosses it: `invalid input: the axis is parallel to the plane`.
- *Turning a plane about an axis that isn't in it.* `invalid input: the axis does not lie in the plane`. The axis must lie on the plane: build it from two points on the plane, or where the plane meets another.
- *An angle in degrees.* `angled` takes radians: write `30f64.to_radians()`.
