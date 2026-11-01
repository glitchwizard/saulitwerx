#import "../template.typ": *

= Revolving a profile <ch-revolve>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("revolve")
A #term[revolve] turns a flat shape around a line, the way a potter's wheel turns a lump of clay into a pot. Every point of the shape sweeps a circle around that line, so the result is round: a shaft, a knob, a washer, a bottle.

Where an extrude (@ch-extrude) pushes an outline straight out, a revolve swings it around. You draw only half of the part's cross-section, the half on one side of the center line, and the revolve makes the rest.

The line the shape turns around is the #term[axis of revolution]. The #term[revolve angle] says how far it turns: a full turn makes a closed, round part; less than a full turn makes a wedge, like a slice of a cake.

#image-equation(
  (
    // cargo run -p swx-kernel --features occt --example revolve_bushing -- revolve
    // swx-qa render --step revolve/bushing-270.step --size 900x700 -o docs/manual/img/revolve-bushing-270.png
    ("img/revolve-bushing-270.png", "Three quarters of a turn"),
    // swx-qa render --step revolve/bushing.step --size 900x700 -o docs/manual/img/revolve-bushing.png
    ("img/revolve-bushing.png", "A full turn"),
  ),
  ("→",),
  caption: [A flanged bushing made by one revolve. Stopped at three quarters of a turn (left), the open quarter shows the outline that was turned: an L-shape beside a hollow center. A full turn (right) closes the part.],
  alt: "Left: a round flanged tube with a quarter cut away, showing its L-shaped cross-section. Right: the same tube, complete.",
) <fig-revolve>

*In Saulitwerx today:* the geometry engine can revolve a flat face from `make_face` about any axis, by any angle up to a full turn. Like an extrude, the result can be a new solid, or be joined to a part, cut from it or intersected with it in the same step. You type the outline's pieces into a small program; a sketch's regions can't be handed to the revolve yet, and the revolve isn't recorded in a part's history. Saulitwerx doesn't yet check that the axis lies in the outline's plane or stays clear of the outline (@sec-revolve-mistakes).

== Revolve a profile <sec-revolve>

#task-intro(
  goal: [Make a flanged bushing from half its cross-section, with a full turn and with three quarters of a turn.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [10 minutes.],
)

#steps(
  [Run the sample program, and tell it to save its parts in a folder called `revolve`:
  #term-block("cargo run -p swx-kernel --features occt --example revolve_bushing -- revolve")
  It prints a line for each part, then three errors on purpose, then the files it wrote:
  #term-block(label: "Output", "full turn   8482.30 mm³   6 faces  x from -20 to 20\n270°        6361.73 mm³   8 faces  x from -20 to 20\ngrooved     8362.92 mm³  10 faces  x from -20 to 20\n360 -> error: invalid input: revolve angle must be in (0, 2π] radians, got 360\nzero axis -> error: invalid input: revolve axis direction must be non-zero\ncut, no target -> error: invalid input: revolve mode \"cut\" needs a target body\n…  (the geometry engine's file reports)\nwrote revolve/bushing.step\nwrote revolve/bushing-270.step\nwrote revolve/bushing-grooved.step")],
  [Read the `full turn` line. The outline reaches 20 mm from the axis, so the part runs from x = −20 to 20 mm: 40 mm across. The six faces are the bore, the tube's outside, the flange's rim, and the top, the flange's top and the bottom.],
  [Read the `270°` line. Three quarters of a turn gives three quarters of the volume, 6361.73 mm³. The two flat ends of the wedge add two faces.],
  [Draw the three-quarter part:
  #term-block("target/release/swx-qa render --step revolve/bushing-270.step -o bushing-270.png")
  Open `bushing-270.png`. You should see the left picture of @fig-revolve: the open quarter shows the L-shaped outline that was turned. Draw `revolve/bushing.step` the same way to see the full part.],
)

#result[A round part made from one flat outline. The bushing's bore, tube and flange came from a single six-sided shape.]

=== How the sample is built

The program is `crates/swx-kernel/examples/revolve_bushing.rs`. It does three things.

#steps(
  [*Draw half the cross-section.* The outline sits on the XZ workplane, so left–right is X, the distance from the axis, and up–down is Z, the height. It starts 5 mm from the axis, which leaves a 10 mm bore down the middle:
  ```rust
  let outline = polygon(&[
      [5.0, 0.0], [20.0, 0.0], [20.0, 4.0],
      [10.0, 4.0], [10.0, 20.0], [5.0, 20.0],
  ]);
  let section = k.make_face(&Workplane::XZ, &outline, &[])?;
  ```
  `polygon` is a small helper in the sample that joins the points with lines, the last back to the first.],
  [*Give the axis.* An `Axis` is a point on the line and the line's direction. Here it is the Z axis, through the origin and pointing up:
  ```rust
  let z_axis = Axis { origin: [0.0, 0.0, 0.0], dir: [0.0, 0.0, 1.0] };
  ```],
  [*Revolve it.* `revolve_feature` takes the face, the axis, the angle in radians, the mode and the target. `TAU` is a full turn, 2π; `1.5 * PI` is three quarters of one:
  ```rust
  let bushing = revolve_feature(&k, &section, z_axis, TAU,
      ExtrudeMode::NewSolid, None)?;
  let cutaway = revolve_feature(&k, &section, z_axis, 1.5 * PI,
      ExtrudeMode::NewSolid, None)?;
  ```],
)

#tip(title: [Which way does it turn?])[Point your right thumb along the axis direction: your fingers curl the way a positive angle turns. Here the axis points up, so the outline swings from +X towards +Y, counterclockwise seen from above. A full turn looks the same either way.]

#caution(title: [Radians, not degrees])[The geometry engine takes the angle in radians: a full turn is 2π, about 6.283, and 360 is refused. To work in degrees, multiply by π/180, or write `360.0_f64.to_radians()`.]

== Cut or join with a revolve <sec-revolve-modes>

#idx("revolve mode")
A revolve uses the same four modes as an extrude (@sec-extrude-modes): new solid, join, cut and intersect, with a target body for the last three. A revolve cut is how you make a round groove, such as the seat for a sealing ring around a shaft (@fig-revolve-groove).

#task-intro(
  goal: [Cut a groove around the bushing with a second revolve.],
  needs: [The sample run of @sec-revolve.],
  time: [5 minutes.],
)

#steps(
  [Read the `grooved` line of the output in @sec-revolve. The groove took 119.38 mm³ from the bushing, and its floor and two walls added three faces, bringing the bushing's six faces to nine, plus one more where the groove splits the tube's outside in two: ten.],
  [Draw the grooved part:
  #term-block("target/release/swx-qa render --step revolve/bushing-grooved.step -o bushing-grooved.png")
  Open `bushing-grooved.png`. You should see @fig-revolve-groove: a narrow groove around the tube, halfway up.],
)

#result[A groove around a round part, made by turning a small rectangle around the same axis as the part itself.]

#shot(
  // cargo run -p swx-kernel --features occt --example revolve_bushing -- revolve
  // swx-qa render --step revolve/bushing-grooved.step --size 900x700 -o docs/manual/img/revolve-bushing-grooved.png
  "img/revolve-bushing-grooved.png",
  width: 42%,
  caption: [The bushing with a groove around its tube: a small rectangle, revolved a full turn and cut from the part.],
  alt: "A flanged round tube with a narrow groove running around the outside of the tube",
) <fig-revolve-groove>

The groove's outline is a 2 × 2 mm square that straddles the tube's outer wall, at x = 10 mm. Only the half inside the tube removes material; the other half turns through empty space. Letting a cutting outline overlap the edge like this makes sure the groove opens cleanly onto the surface:

```rust
let groove = k.make_face(&Workplane::XZ,
    &polygon(&[[9.0, 11.0], [11.0, 11.0], [11.0, 13.0], [9.0, 13.0]]), &[])?;
let grooved = revolve_feature(&k, &groove, z_axis, TAU,
    ExtrudeMode::Cut, Some(&bushing))?;
```

The rules are the extrude's: join, cut and intersect need a target, a result with nothing left is an `empty result` error, and the target itself is never changed. `ExtrudeMode` is the name of the mode type for both features.

== Common mistakes <sec-revolve-mistakes>

#set par(justify: false)

- *Degrees instead of radians.* `revolve angle must be in (0, 2π] radians, got 360`. Convert the angle (@sec-revolve).
- *An angle of zero, or a negative one.* A revolve has to turn: the same message. To turn the other way, reverse the axis direction.
- *An axis with no direction.* `revolve axis direction must be non-zero`. Give the axis a direction, such as `[0.0, 0.0, 1.0]`.
- *The axis runs through the outline.* The outline would sweep through itself. Today the geometry engine refuses it with `kernel failure: revolve failed`. Keep the whole outline on one side of the axis; touching it is fine, and makes a solid center with no bore.
- *The axis isn't in the outline's plane.* If the axis stands straight out of the plane, the outline only spins in place and encloses nothing: `empty result: revolve new solid leaves no solid (volume 0e0)`. Other tilted or offset axes aren't checked yet and can give a broken solid. Draw the outline on a plane that contains the axis, as the sample does with the XZ plane and the Z axis.
- *Join, cut or intersect with no target.* `revolve mode "cut" needs a target body`. Pass the part as `Some(&part)`, or use `ExtrudeMode::NewSolid`.
