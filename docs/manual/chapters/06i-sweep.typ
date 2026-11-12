#import "../template.typ": *

= Sweeping along a path <ch-sweep>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("sweep")
Some parts have the same cross-section all the way along, but don't run straight: a bent rod, a pipe with an elbow, a handle that curves over the top of a box, a rib that follows the edge of a plate. An extrude can only push straight, and a revolve only turns around one axis. A #term[sweep] moves a flat outline along a line you draw, and the outline leaves a solid behind it as it goes.

A sweep needs two things:

- *The profile:* the flat outline that gives the cross-section, such as a circle for a rod or a rectangle for a bar. It is a face from `make_face`, as for an extrude (@ch-extrude).
- *The #term[path]:* the line the profile travels along, made of straight lines and arcs joined end to end. It is drawn on a plane of its own, and it must start on the profile's plane, usually at the profile's middle.

The profile keeps its size and shape all the way. Through each bend it turns with the path, so it stays square to the direction of travel and the part doesn't twist.

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example sweep_pipe -- sweep
    // swx-qa render --step sweep/sweep-<name>.step --size 900x700 -o docs/manual/img/sweep-<name>.png
    ("img/sweep-rod.png", "Circle: a bent rod", "A round rod that rises straight up, bends over in a smooth quarter circle, and runs level to a flat round end"),
    ("img/sweep-bar.png", "Rectangle: a bent bar", "A flat rectangular bar following the same path: straight up, a smooth bend, then level"),
  ),
  caption: [Two profiles swept along one path: up 30 mm, a quarter circle of radius 20 mm, then 20 mm level. Left, a circle 10 mm across; right, a 12 × 6 mm rectangle.],
) <fig-sweep>

*In Saulitwerx today:* the geometry engine can sweep one flat profile, with no holes, along an open path of lines and arcs that all lie in one plane, or around a helix (@sec-helix). The path can't be a closed loop, a full circle, a spline or any other curve in 3D yet, and the profile can't twist or change size along the way. A sweep makes a new solid; to add it to a part or cut it from one, use `union` or `cut` (@sec-feature). It isn't recorded in a part's history.

== Sweep a profile along a path <sec-sweep>

#task-intro(
  goal: [Sweep a circle and a rectangle along a path with a bend, and see the errors.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `sweep`:
  #term-block("cargo run -p swx-kernel --features occt --example sweep_pipe -- sweep")
  It prints one line for each sweep, five errors on purpose, then the files it wrote:
  #term-block(label: "Output", "rod         6394.39 mm³  5 faces  box 45 × 10 × 55 mm\nbar         5861.95 mm³  14 faces  box 46 × 6 × 56 mm\nstarts above -> error: invalid input: sweep path starts 5 off the\n    profile's plane\ngap -> error: invalid input: sweep path: segment 1 doesn't meet segment 0\n    within 0.0000001\ncircle path -> error: invalid input: sweep path: circles (closed paths)\n    are not supported\nprofile with hole -> error: invalid input: sweep profile has a hole;\n    profiles with holes are not supported\ntight bend -> error: invalid input: sweep path: arc segment 1 has radius\n    4, not larger than the profile's distance 5.000000232604544 from the\n    path's start; the swept solid would pass through itself\n…  (the geometry engine's file reports)\nwrote sweep/sweep-rod.step\nwrote sweep/sweep-bar.step")
  Five long error lines are wrapped here to fit the page.]],
  [Read the `rod` line. The path is 30 mm + a quarter circle of 20 mm radius (31.42 mm) + 20 mm = 81.42 mm long. The circle's area is π × 5² = 78.54 mm², and 78.54 × 81.42 = 6394.39 mm³: a sweep holds its profile's area times its path's length.],
  [Read the `bar` line. The rectangle is 72 mm², so the bar holds 72 × 81.42 = 5861.95 mm³. Each of its four sides is split into three faces, one for each piece of the path, which with the two ends makes 14.],
  [Read the errors. The path must start on the profile's plane, its pieces must meet end to end, it can't be a circle, and the profile can't have a hole. The last one bends the rod round a 4 mm radius, tighter than the rod's own 5 mm radius, so the sweep refuses it (see the caution below).],
  [#block(breakable: false)[Draw the rod:
  #term-block("target/release/swx-qa render --step sweep/sweep-rod.step -o sweep-rod.png")
  Open `sweep-rod.png`. You should see the left-hand part of @fig-sweep.]],
)

#result[A bent rod and a bent bar, each from one call.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/sweep_pipe.rs`. It does three things.

#steps(
  [*Make the profile.* A circle 10 mm across on the XY plane, centered on the origin:
  ```rust
  let round = k.make_face(&Workplane::XY, &circle, &[])?;
  ```],
  [*Draw the path.* The path goes on the XZ plane, where u runs along X and v up along Z, so it starts at the origin, in the middle of the profile, and rises square to it. It is a list of `Segment2` pieces, as an outline is (@sec-ref-extrude), but open:
  ```rust
  let path = [
      Segment2::Line { start: [0.0, 0.0], end: [0.0, 30.0] },
      Segment2::Arc { center: [20.0, 30.0], radius: 20.0,
                      start_angle: FRAC_PI_2, end_angle: PI },
      Segment2::Line { start: [20.0, 50.0], end: [40.0, 50.0] },
  ];
  ```
  An arc always runs counterclockwise from its start angle to its end angle, so this one runs from (20, 50) back to (0, 30). That's fine: the sweep follows the path from the first piece's start, and takes each next piece whichever way round meets the end of the one before.],
  [*Sweep.* `sweep` takes the profile, the path's plane and the path:
  ```rust
  let rod = k.sweep(&round, &Workplane::XZ, &path)?;
  ```],
)
]

#tip(title: [Start square to the profile])[Start the path at the profile's middle, heading straight out of the profile's plane, as the sample does. A path that starts at a slant leans the whole part over; one that starts flat along the profile's plane gives nothing to sweep, and fails.]

#caution(title: [Keep bends wider than the profile])[Picture bending a thick rod round a tight corner: the inside of the bend gets squeezed until it would have to pass through itself. A sweep can't build that, so it refuses any arc whose radius isn't larger than the profile's reach: the distance from the path's start to the profile's farthest point, measured in the profile's plane. The message names the arc (pieces are counted from 0), its radius and that distance. In the sample, the rod's reach is its 5 mm radius (shown as 5.000000232604544, because it is measured on a fine mesh of the profile), so the 20 mm bend passes and the 4 mm bend is refused. For the 12 × 6 mm bar, the reach is its corner, √(6² + 3²) = 6.71 mm from the middle, so its bends must be wider than that. The check uses the farthest point in every direction, so it can refuse a bend that would just fit on the profile's narrow side. Make the bend wider, or the profile smaller.]

== Sweep around a helix: springs and coils <sec-helix>

#idx("helical sweep")
A coil spring is a round wire wound round and round while it climbs. The line its middle follows is a #term[helix]: the shape of a spiral staircase's handrail, which circles a center post and rises the same amount each time it goes round. How far it rises in one full turn is its #term[pitch].

A #term[helical sweep] moves a flat profile along such a helix. You don't draw the path: you give the axis the helix turns about, the pitch and the number of turns, and the profile itself says how far from the axis the helix runs. As it goes round, the profile turns with it and stays upright beside the axis, the way a door stays square to its hinge, so it doesn't twist. Springs, coils, and the winding ridge of a screw thread are all helical sweeps.

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example helical_spring -- helix
    // swx-qa render --step helix/helix-<name>.step --size 900x700 -o docs/manual/img/helix-<name>.png
    ("img/helix-spring.png", "Circle: a coil spring", "A round wire wound into a coil spring of six turns, standing upright, with a flat round cut at each end of the wire"),
    ("img/helix-ridge.png", "Rectangle: a square ridge", "A flat rectangular strip wound into four close turns, like a squared-off screw thread with nothing inside it"),
  ),
  caption: [Two profiles swept around the upright Z axis. Left, a circle 2 mm across, 10 mm from the axis: six turns at 5 mm a turn. Right, a 2 × 1.5 mm rectangle, 8 to 10 mm from the axis: four turns at 3 mm a turn.],
) <fig-helix>

*In Saulitwerx today:* the helix keeps one radius and one pitch all the way, and always winds right-handed, like a common screw. The profile is one flat face without holes, beside the axis. A helical sweep makes a new solid; to add it to a part or cut it from one, use `union` or `cut` (@sec-feature). Real screw threads, on a shaft or in a hole, aren't built yet (*Coming next*).

#task-intro(
  goal: [Wind a spring and a square ridge, and see the errors.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [Run the sample program, and tell it to save its parts in a folder called `helix`:
  #term-block("cargo run -p swx-kernel --features occt --example helical_spring -- helix")
  It prints one line for each part, five errors on purpose, then the files it wrote:
  #term-block(label: "Output", "spring   1184.35 mm³  3 faces  box 22 × 22 × 32.0 mm\nridge     678.58 mm³  6 faces  box 20 × 20 × 13.5 mm\npitch 0 -> error: invalid input: helix pitch must be finite and > 0, got 0\naxis along Y -> error: invalid input: helix axis (origin [0.0, 0.0, 0.0],\n    direction [0.0, 1.0, 0.0]) is not in the profile's plane (through\n    [0.0, 0.0, 0.0], normal [0.0, -1.0, 0.0])\non the axis -> error: invalid input: the profile crosses the helix axis:\n    its distance from the axis spans [-1, 1]\npitch 1.5 -> error: invalid input: the profile's extent along the helix\n    axis (2) must be less than the pitch (1.5), or neighboring turns overlap\ntube -> error: invalid input: helical sweep profile has a hole; profiles\n    with holes are not supported\n…  (the geometry engine's file reports)\nwrote helix/helix-spring.step\nwrote helix/helix-ridge.step")
  Four long error lines are wrapped here to fit the page.],
  [Read the `spring` line. The wire's middle is 10 mm from the axis, so one turn is about 2π × 10 = 62.83 mm long, and six turns about 377 mm. The wire's area is π × 1² = 3.14 mm², and 3.14 × 377 = 1184.35 mm³. The spring stands 6 × 5 = 30 mm tall, plus the wire's own 2 mm. It is 22 mm across: twice 10 mm, plus the wire.],
  [Read the `ridge` line. The rectangle's middle is 9 mm from the axis and its area is 3 mm², so four turns hold 4~×~2π~×~9~×~3 = 678.58 mm³. It stands 4 × 3 + 1.5 = 13.5 mm tall.],
  [Read the errors. The pitch must be more than 0. The axis must lie in the profile's plane: the profile is drawn on the XZ plane, which holds the Z axis but not the Y axis. The profile must sit to one side of the axis, not across it. It must be shorter, along the axis, than the pitch, or each turn would run into the next: the 2 mm wire can't climb only 1.5 mm a turn. And the profile can't have a hole.],
  [#block(breakable: false)[Draw the spring:
  #term-block("target/release/swx-qa render --step helix/helix-spring.step -o helix-spring.png")
  Open `helix-spring.png`. You should see the left-hand part of @fig-helix.]],
)

#result[A coil spring and a winding ridge, each from one call.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/helical_spring.rs`. It does three things.

#steps(
  [*Choose the axis.* An `Axis` is a point and a direction (@sec-ref-work). Here it is the Z axis, through the origin and pointing up:
  ```rust
  let z_axis = Axis { origin: [0.0, 0.0, 0.0], dir: [0.0, 0.0, 1.0] };
  ```],
  [*Make the profile beside it.* The XZ plane holds the Z axis, with u running along X and v up along Z. A circle 2 mm across, centered 10 mm out along X, sits beside the axis:
  ```rust
  let wire = k.make_face(&Workplane::XZ, &circle([10.0, 0.0], 1.0), &[])?;
  ```],
  [*Sweep.* `helical_sweep` takes the profile, the axis, the pitch in mm and the number of turns:
  ```rust
  let spring = k.helical_sweep(&wire, z_axis, 5.0, 6.0)?;
  ```],
)
]

#tip(title: [Which way it winds])[The helix starts at the profile and turns counterclockwise as you look down the axis from its tip, climbing towards the tip: a right-hand helix. To wind the other way up, point the axis the other way (`dir: [0.0, 0.0, -1.0]`); the part then climbs downwards from the profile. The number of turns needn't be whole: `0.25` makes a quarter turn.]

#note(title: [Why the pitch doesn't change the volume])[A spring with a steeper pitch has a longer wire, yet the same volume: 1184.35 mm³ for six turns whatever the pitch. The climb happens within the profile's own plane, so it sweeps out nothing; only going round the axis does. The volume is the number of turns, times 2π, times the profile's area, times the distance of its middle from the axis, as for a revolve.]

== Common mistakes <sec-sweep-mistakes>

#set par(justify: false)

- *A path that doesn't start on the profile.* `invalid input: sweep path starts 5 off the profile's plane`. The number is the distance, in mm. Start the path on the profile's plane, at its middle.
- *A gap in the path.* `invalid input: sweep path: segment 1 doesn't meet segment 0 within 0.0000001`. Segments are counted from 0. Reuse the same values for the end of one piece and the start of the next.
- *A closed path.* A circle is refused: `invalid input: sweep path: circles (closed paths) are not supported`. For a ring, revolve the profile instead (@ch-revolve).
- *An empty path.* `invalid input: sweep path is empty`.
- *A profile with a hole.* `invalid input: sweep profile has a hole; profiles with holes are not supported`. To make a tube, sweep the outer circle, sweep the inner one along the same path, and cut the second from the first (@sec-feature). For the sample's path, circles 10 and 6 mm across give a tube of 4092.41 mm³.
- *A solid in place of the profile.* `invalid input: sweep profile is not a single planar face`. Pass the face from `make_face`.
- *A path that starts along the profile's plane.* The profile slides within its own plane and sweeps out nothing: `empty result: the sweep encloses no volume`. Draw the path on a plane that crosses the profile's plane.
- *A bend tighter than the profile.* The message names the arc, its radius and the profile's reach: `invalid input: sweep path: arc segment 1 has radius 4,` `not larger than the profile's distance 5.000000232604544 from the path's start;` `the swept solid would pass through itself`. Give the arc a radius larger than the distance shown (see the caution above).
- *A helix axis that leaves the profile's plane.* `invalid input: helix axis (origin …, direction …) is not in the profile's plane (through …, normal …)`. Draw the profile on a plane that holds the axis: XZ or YZ for the Z axis.
- *A profile across or touching the helix axis.* `invalid input: the profile crosses the helix axis: its distance from the axis spans [-1, 1]` (or `touches`). Move the profile to one side of the axis.
- *A pitch no larger than the profile is tall.* `invalid input: the profile's extent along the helix axis (2)` `must be less than the pitch (1.5), or neighboring turns overlap`. Raise the pitch, or make the profile shorter along the axis.
- *A pitch or a number of turns of 0 or less.* `invalid input: helix pitch must be finite and > 0, got 0` (or `helix turns`).
