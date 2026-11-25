#import "../template.typ": *

= Mirroring and patterning bodies <ch-pattern>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("mirror")
#idx("pattern")
Many parts repeat themselves. A bracket is often the same on the left as on the right. A cover plate has a row of screw holes, a grille has a grid of slots, a flange has bolt holes spaced evenly round a circle. You could model every copy by hand, measuring where each one goes, but one wrong number spoils the part, and changing the hole size means changing every copy.

Two operations do the repeating for you:

- A #term[mirror] makes the reflection of a body across a flat plane, the #term(key: "mirror plane")[mirror plane], as a looking-glass would. Model half of a symmetric part, mirror it, and join the two halves: the part is symmetric by construction.
- A #term[pattern] makes evenly spaced copies of a body. A #term(key: "rectangular pattern")[rectangular pattern] places them in rows and columns. A #term(key: "circular pattern")[circular pattern] turns them round an axis, like the numbers on a clock face.

Both work on whole bodies (@sec-bodies). To pattern a hole, you pattern the solid that drills it, a pin, and cut all the copies from the part in one `combine` call. To pattern a boss, you pattern the boss and join the copies.

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example mirror_pattern -- shapes
    // swx-qa render --step shapes/<name>.step --size 900x700 -o docs/manual/img/<name>.png
    ("img/mirror-bracket.png", "Mirror: a bracket", "A U-shaped bracket: a flat base with an upright wall at each end, made by mirroring an L-shaped half"),
    ("img/pattern-grid.png", "Rectangular: a grid of holes", "A flat rectangular plate with six round holes in two rows of three"),
    ("img/pattern-flange.png", "Circular: a bolt circle", "A round flange with six holes spaced evenly round a circle near its edge"),
  ),
  caption: [What the three operations are for. The bracket is half a part and its mirror image, joined. The grid and the bolt circle are each one pin, patterned and cut from the part. Each picture is scaled to fill its frame.],
) <fig-patterns>

*In Saulitwerx today:* the geometry engine can mirror a body across any plane, and copy it in a straight line, a grid, or round an axis. Each call gives back new shapes and leaves the original alone; you join or cut the copies yourself with `combine` (@sec-bodies). You can't pattern a feature, such as a hole with its settings, or skip some copies, or space them along a curve yet. Mirrors and patterns aren't recorded in a part's history.

== Mirror a body <sec-mirror>

A mirror needs the body and the mirror plane. The plane is a `Workplane` (@sec-work): Saulitwerx uses its origin and its normal, the direction that points straight out of it, and ignores its X direction. A point on the plane stays where it is; a point 10 mm in front of the plane lands 10 mm behind it.

Mirroring keeps every size and turns the shape left for right, the way your left hand is the mirror image of your right. If half the part touches the mirror plane, the two halves meet there, and `union` joins them into one body.

== Pattern a body in rows and columns <sec-pattern-rect>

A rectangular pattern needs a direction, a count and a spacing, the distance from one copy to the next. That makes a row. Give a second direction, count and spacing, and each copy in the row is repeated along it too: a grid.

The counts include the original. A count of 3 gives the body where it is and two copies beyond it; a grid of 3 by 2 gives 6 bodies. Copy number 0 is always the original, unmoved. The copies come in rows: all of the second direction for the first place along the first direction, then all of it for the next.

The directions don't need to be at right angles, and their length doesn't matter: only the way they point counts. A negative spacing copies the other way.

== Pattern a body round an axis <sec-pattern-circ>

A circular pattern needs an axis (@sec-revolve), a count and a total angle, in radians. The copies turn about the axis by the right-hand rule: point your right thumb along the axis, and your fingers curl the way the copies go.

The total angle decides how they are spread:

- *A full turn*, 2π (360°), spaces the copies evenly round the whole circle, without the last one landing on the first. Six copies are 60° apart.
- *Less than a full turn* spreads them from the start to the total angle, with the last copy on it. Three copies over 90° land at 0°, 45° and 90°.

#image-row(
  (
    ("img/pattern-flange.png", "Full turn, 6 copies", "A round flange with six holes spaced evenly all round it"),
    ("img/pattern-quarter.png", "90°, 3 copies", "The same flange with only three holes, spread over a quarter of the circle, the first and last a quarter-turn apart"),
  ),
  caption: [The same hole, patterned about the flange's center with two total angles. A full turn spreads the copies evenly; a smaller angle puts the last copy on it.],
) <fig-pattern-angles>

== Mirror and pattern a part <sec-mirror-pattern-task>

#task-intro(
  goal: [Mirror half a bracket into a whole one, drill a grid of six holes from one pin, and drill six holes round a flange, then three over a quarter turn.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `shapes`:
  #term-block("cargo run -p swx-kernel --features occt --example mirror_pattern -- shapes")
  It prints one or two lines for each part, five errors on purpose, then the files it wrote:
  #term-block(label: "Output", "half 5000.00 mm³; mirror image 5000.00 mm³, x 30..60\nbracket 10000.00 mm³, 1 body\ngrid: 6 copies at (10,10) (10,30) (30,10) (30,30) (50,10) (50,30)\ngrid plate 11151.77 mm³, 12 faces\nfull turn: 6 copies, every 60°\nflange 37799.64 mm³, 9 faces\nquarter turn: 3 copies at 0° 45° 90°\nzero normal -> error: invalid input: mirror plane normal must be non-zero\ncount 0 -> error: invalid input: pattern count 1 must be at least 1\nzero direction -> error: invalid input: pattern direction 1 must be non-zero\nspacing NaN -> error: invalid input: pattern spacing 1 must be finite, got [NaN]\nangle 400° -> error: invalid input: pattern total angle must be in (0, 2π]\n    radians, got 6.981317007977318\n…  (the geometry engine's file reports)\nwrote shapes/mirror-half.step\nwrote shapes/mirror-bracket.step\nwrote shapes/pattern-grid.step\nwrote shapes/pattern-flange.step\nwrote shapes/pattern-quarter.step")
  One long line is wrapped here to fit the page.]],
  [Read the first two lines. The half bracket, a 30 mm base with an upright at its left end, runs from x = 0 to 30. The mirror plane stands at x = 30, so its mirror image runs from 30 to 60, with the upright at the far end. Joined, they make one body of twice the volume.],
  [Read the `grid` lines. The pin was at (10, 10). Three along X every 20 mm and two along Y every 20 mm give six places, listed row by row. Cut from the 60 × 40 × 5 mm plate, each hole 6 mm across takes π × 3² × 5 = 141.37 mm³: 12000 − 6 × 141.37 = 11151.77 mm³.],
  [Read the `full turn` and `flange` lines. Six copies of a hole 8 mm across, 30 mm from the center, land 60° apart. The flange, 80 mm across and 8 mm thick, keeps 37799.64 mm³.],
  [Read the `quarter turn` line. With 90° as the total angle, three copies land at 0°, 45° and 90°.],
  [Read the errors. Each one is caught before the geometry engine is called, and nothing is made.],
  [#block(breakable: false)[Draw the bracket:
  #term-block("target/release/swx-qa render --step shapes/mirror-bracket.step -o bracket.png")
  Open `bracket.png`. You should see the bracket of @fig-mirror. Draw the other files the same way.]],
)

#result[A symmetric bracket from half its shape, and three hole patterns from one pin each.]

#image-equation(
  (
    // swx-qa render --step shapes/mirror-half.step --size 900x700 -o docs/manual/img/mirror-half.png
    ("img/mirror-half.png", "Half: an L-shaped base and upright, 30 mm long"),
    ("img/mirror-bracket.png", "Half and its mirror image: a U-shaped bracket"),
  ),
  ("→",),
  caption: [Mirrored across the plane x = 30 at the open end, the half becomes a bracket 60 mm long, the same at both ends.],
) <fig-mirror>

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/mirror_pattern.rs`. It does three things.

#steps(
  [#block(breakable: false)[*Mirror the half and join it.* The plane is given by a point on it and its normal:
  ```rust
  let plane = Workplane { origin: [30.0, 0.0, 0.0],
                          x_dir: [0.0, 1.0, 0.0],
                          normal: [1.0, 0.0, 0.0] };
  let other = k.mirror(&half, &plane)?;
  let bracket = k.union(&half, &other)?;
  ```]],
  [#block(breakable: false)[*Pattern the pin in a grid and cut it.* The first direction, count and spacing come first; the second, as `(direction, count, spacing)`, is optional:
  ```rust
  let pins = k.pattern_rectangular(&pin, [1.0, 0.0, 0.0], 3, 20.0,
                                   Some(([0.0, 1.0, 0.0], 2, 20.0)))?;
  let grid = k.combine(BooleanOp::Cut, &plate, &pins)?;
  ```]],
  [#block(breakable: false)[*Pattern the bolt hole round the axis and cut it.* The total angle is in radians; `TAU` is a full turn:
  ```rust
  let z = Axis { origin: [0.0, 0.0, 0.0], dir: [0.0, 0.0, 1.0] };
  let bolts = k.pattern_circular(&bolt, z, 6, TAU)?;
  let ring = k.combine(BooleanOp::Cut, &flange, &bolts)?;
  ```]],
)
]

#tip(title: [Patterning a boss instead of a hole])[To add copies rather than cut them, join them: `combine(BooleanOp::Union, &part, &copies)`. Copies that don't touch the part, or each other, stay separate bodies (@sec-bodies).]

#caution(title: [Copies that overlap])[Nothing stops two copies from landing on top of each other, for example with a spacing smaller than the body. Cutting overlapping pins makes one bigger, odd-shaped hole. Check the spacing against the size of what you are copying.]

== Common mistakes <sec-pattern-mistakes>

#set par(justify: false)

- *A count of zero.* #raw("invalid input: pattern count 1 must be at least 1") (or `count 2`, or `pattern count` for a circular pattern). The count includes the original, so the smallest is 1, which gives the body alone.
- *A direction, normal or axis of length zero.* #raw("invalid input: pattern direction 1 must be non-zero"), #raw("mirror plane normal must be non-zero"), or #raw("pattern axis direction must be non-zero"). Give a direction that points somewhere, such as `[1.0, 0.0, 0.0]`.
- *A number that isn't a number.* #raw("invalid input: pattern spacing 1 must be finite, got [NaN]"). The same message names `mirror plane origin`, `pattern axis origin` or a direction when one of those holds an infinite value or NaN, usually the result of a division by zero earlier in your program.
- *An angle in degrees.* #raw("invalid input: pattern total angle must be in (0, 2π] radians, got 6.981317007977318"). The total angle is in radians: 360° is 2π, about 6.283. Convert with `to_radians()`. An angle of 0 or less is refused too.
- *Expecting the copies to be joined.* A pattern gives back a list of separate shapes. Cut or join them with `combine` to change your part.
