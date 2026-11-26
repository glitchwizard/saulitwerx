#import "../template.typ": *

= Mirroring, patterning and splitting bodies <ch-pattern>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("mirror")
#idx("pattern")
#idx("split")
Many parts repeat themselves. A bracket is often the same on the left as on the right. A cover plate has a row of screw holes, a grille has a grid of slots, a flange has bolt holes spaced evenly round a circle. You could model every copy by hand, measuring where each one goes, but one wrong number spoils the part, and changing the hole size means changing every copy.

Two kinds of operation do the repeating for you:

- A #term[mirror] makes the reflection of a body across a flat plane, the #term(key: "mirror plane")[mirror plane], as a looking-glass would. Model half of a symmetric part, mirror it, and join the two halves: the part is symmetric by construction.
- A #term[pattern] makes evenly spaced copies of a body. A #term(key: "rectangular pattern")[rectangular pattern] places them in rows and columns. A #term(key: "circular pattern")[circular pattern] turns them round an axis, like the numbers on a clock face. A #term(key: "path pattern")[path pattern] spreads them along a path of lines and arcs, such as a row of slots that follows a bend.

A third operation goes the other way. A #term[split] cuts one body into pieces along a flat plane, so that parts which must match, such as the two jaws of a clamp, can be made as one and then divided.

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

*In Saulitwerx today:* the geometry engine can mirror a body across any plane; copy it in a straight line, a grid, round an axis or along a flat path; and split it with a plane, or divide its faces along one. Each call gives back new shapes and leaves the original alone; you join or cut the copies yourself with `combine` (@sec-bodies). You can't pattern a feature, such as a hole with its settings, or skip some copies, or follow a path that leaves its plane yet. Mirrors, patterns and splits aren't recorded in a part's history.

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

== Pattern a body along a path <sec-pattern-path>

A path pattern needs the body, a path, the workplane the path is drawn on, and a count. The path is the same kind a sweep follows (@sec-sweep): a chain of lines and arcs, end to end, on one workplane.

The copies are spread at equal distances measured _along_ the path, round its bends, not in a straight line. The count includes the original: copy 0 is the body where it is, and the last copy sits at the path's end. Seven copies on a path 101.42 mm long are 101.42 ÷ 6 = 16.90 mm apart.

Each copy is moved by the distance from the path's start to its place on the path. So put the body at the start of the path, and every copy lands on the path. A body 5 mm to one side of the start gives copies 5 mm to the same side of the path.

There are two calls, and they differ in which way the copies face:

- `pattern_path` only moves the copies. Every copy faces the same way as the original, however the path turns.
- `pattern_path_aligned` also turns each copy as the path turns: by as much as the path has turned since its start, about the workplane's normal. A slot that lies along the path at its start lies along it at every copy.

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example path_pattern -- shapes
    // swx-qa render --step shapes/<name>.step --size 900x700 -o docs/manual/img/<name>.png
    ("img/path-pattern-straight.png", "pattern_path: same direction", "A square plate with seven small slots in an L-shaped row round one corner; every slot points the same way"),
    ("img/path-pattern-aligned.png", "pattern_path_aligned: following the path", "The same plate and row of slots, but each slot lies along the row: the slots on the second leg are turned a quarter turn, and the one on the bend half that"),
  ),
  caption: [One slot, patterned seven times along a path with a bend, and cut from a plate. On the left the copies keep the slot's direction; on the right they turn with the path.],
) <fig-path-pattern>

#task-intro(
  goal: [Pattern a slot along a path with a bend, once keeping its direction and once following the path, cut both from a plate, and see the errors.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `shapes`:
  #term-block("cargo run -p swx-kernel --features occt --example path_pattern -- shapes")
  It prints the path's length, where each copy landed and how much room it takes along X and Y, four errors on purpose, then the files it wrote:
  #term-block(label: "Output", "path 101.42 mm long\nsame direction: 7 copies\n  (10.0,15.0) 8.0×3.0\n  (26.9,15.0) 8.0×3.0\n  (43.8,15.0) 8.0×3.0\n  (59.1,20.9) 8.0×3.0\n  (65.0,36.2) 8.0×3.0\n  (65.0,53.1) 8.0×3.0\n  (65.0,70.0) 8.0×3.0\nplate 31160.00 mm³\nfollowing the path: 7 copies\n  (10.0,15.0) 8.0×3.0\n  (26.9,15.0) 8.0×3.0\n  (43.8,15.0) 8.0×3.0\n  (59.1,20.9) 7.8×7.8\n  (65.0,36.2) 3.0×8.0\n  (65.0,53.1) 3.0×8.0\n  (65.0,70.0) 3.0×8.0\nplate 31160.00 mm³\ncount 0 -> error: invalid input: pattern count must be at least 1\nempty path -> error: invalid input: sweep path is empty\ngap -> error: invalid input: sweep path: segment 1 doesn't meet segment 0\n    within 0.0000001\ncircle path -> error: invalid input: sweep path: circles (closed paths)\n    are not supported\n…  (the geometry engine's file reports)\nwrote shapes/path-pattern-straight.step\nwrote shapes/path-pattern-aligned.step")
  Two long lines are wrapped here to fit the page.]],
  [Read the `path` line. The path runs 35 mm along X, a quarter circle of 20 mm radius (31.42 mm), then 35 mm along Y: 101.42 mm in all.],
  [Read the `same direction` lines. The slot, 8 × 3 mm, sits at the path's start, (10, 15). Three copies fall on the first straight, one halfway round the bend at (59.1, 20.9), and three on the second straight, the last at the path's end, (65, 70). Every copy is still 8.0 mm along X and 3.0 mm along Y: none has turned.],
  [Read the `following the path` lines. The copies sit in the same places, but the one on the bend is turned 45°, so the box round it is square (8 × cos 45° + 3 × sin 45° = 7.8 mm each way), and the last three are turned a quarter turn: 3.0 × 8.0 mm.],
  [Read the two `plate` lines. Each slot cuts 8 × 3 × 5 = 120 mm³ from the 80 × 80 × 5 mm plate, turned or not: 32000 − 7 × 120 = 31160 mm³.],
  [Read the errors. They are caught before the geometry engine is called, and nothing is made. The path messages say "sweep path", because a pattern's path is checked by the same rules as a sweep's (@sec-sweep-mistakes).],
  [#block(breakable: false)[Draw the two plates:
  #term-block("target/release/swx-qa render --step shapes/path-pattern-aligned.step -o aligned.png")
  Open `aligned.png`. You should see the right-hand plate of @fig-path-pattern. Draw `path-pattern-straight.step` the same way.]],
)

#result[A row of slots round a bend, from one slot, in two styles.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/path_pattern.rs`. The path is a list of `Segment2` pieces on the XY plane, as for a sweep:

```rust
let path = [
    Segment2::Line { start: [10.0, 15.0], end: [45.0, 15.0] },
    Segment2::Arc { center: [45.0, 35.0], radius: 20.0,
                    start_angle: -FRAC_PI_2, end_angle: 0.0 },
    Segment2::Line { start: [65.0, 35.0], end: [65.0, 70.0] },
];
let straight = k.pattern_path(&slot, &Workplane::XY, &path, 7)?;
let aligned = k.pattern_path_aligned(&slot, &Workplane::XY, &path, 7)?;
let plate_aligned = k.combine(BooleanOp::Cut, &plate, &aligned)?;
```
]

#tip(title: [Copies on a corner])[Where two pieces of the path meet at a sharp corner, a copy that lands exactly on the corner faces along the piece that ends there, not the one that starts there. A count of 2 puts the copies on the path's two ends only.]

== Split a body with a plane <sec-split>

A split needs the body and the #term(key: "split plane")[split plane], a `Workplane`. As for a mirror, Saulitwerx uses the plane's origin and its normal, and ignores its X direction. The plane has no edges: it reaches as far as the body does.

`split` gives back two lists of bodies. The first holds the pieces on the side the normal points to, the second the pieces on the other side. Nothing is lost: the pieces' volumes add up to the whole.

A side can hold more than one piece. Cut a U-shaped bracket across both uprights and the side above holds two pieces, the tops of the uprights, while the side below holds one, the base with the stubs. Within a list, the pieces come in the order `bodies` gives them (@sec-bodies). A plane that misses the body leaves one list empty.

#image-equation(
  (
    // cargo run -p swx-kernel --features occt --example split_body -- shapes
    // swx-qa render --step shapes/split-clamp.step --size 900x700 -o docs/manual/img/split-clamp.png
    ("img/split-clamp.png", "A clamp block with a bore"),
    // swx-qa render --step shapes/split-jaws.step --size 900x700 -o docs/manual/img/split-jaws.png
    ("img/split-jaws.png", "Its two jaws, moved apart"),
  ),
  ("→",),
  caption: [A clamp block, 60 × 30 × 20 mm with a bore 20 mm across, split by the plane through the bore's axis. The sample moves one jaw 10 mm away so you can see the cut; the split itself leaves both where they were.],
) <fig-split>

#task-intro(
  goal: [Split a clamp block into two jaws, cut a bracket into three pieces, try a plane that misses, divide a block's faces without cutting it, and see the errors.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `shapes`:
  #term-block("cargo run -p swx-kernel --features occt --example split_body -- shapes")
  It prints what each split gave, two errors on purpose, then the files it wrote:
  #term-block(label: "Output", "clamp 29716.81 mm³\nnormal side (back): 1 body, 14858.41 mm³\nother side (front): 1 body, 14858.41 mm³\nbracket at z = 10: 2 above, 1 below\nclamp at z = 50: 0 above, 1 below\nfaces 7 -> 12; bodies 1 -> 1\nvolume 29716.81 -> 29716.81 mm³; area 7828.32 -> 7828.32 mm²\nzero normal -> error: invalid input: split plane normal must be non-zero\norigin NaN -> error: invalid input: split plane origin must be finite,\n    got [NaN, 0.0, 0.0]\njaws apart: 2 bodies\n…  (the geometry engine's file reports)\nwrote shapes/split-clamp.step\nwrote shapes/split-jaws.step")
  One long line is wrapped here to fit the page.]],
  [Read the first three lines. The plane passes through the bore's axis at y = 15 with its normal along +Y, so the first list holds the back jaw and the second the front one. Each is exactly half the block.],
  [Read the `bracket` line. The plane z = 10 cuts through both uprights of a U-shaped bracket: two pieces above it, one below.],
  [Read the `clamp at z = 50` line. The clamp is only 20 mm tall, so the plane misses it: nothing above, the whole clamp below.],
  [Read the `faces` and `volume` lines. They come from dividing the faces without cutting; see *Divide faces without cutting*, below.],
  [Read the errors. Both are caught before the geometry engine is called, and nothing is made.],
  [#block(breakable: false)[Draw the jaws:
  #term-block("target/release/swx-qa render --step shapes/split-jaws.step -o jaws.png")
  Open `jaws.png`. You should see the right-hand picture of @fig-split.]],
)

#result[Two matching jaws from one block, and the pieces of a bracket, each side in its own list.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/split_body.rs`. A split takes the body and the plane, and gives back the two lists:

```rust
let plane = Workplane { origin: [30.0, 15.0, 0.0],
                        x_dir: [1.0, 0.0, 0.0],   // not used
                        normal: [0.0, 1.0, 0.0] };
let (back, front) = k.split(&clamp, &plane)?;
```

Saulitwerx has no command to move a body yet, so for the picture the sample moves the back jaw with a rectangular pattern of two, 10 mm apart, and keeps the second copy.
]

=== Divide faces without cutting

Sometimes you want the line, not the cut. `split_faces` takes the same plane, and divides every face the plane crosses along the line where they meet. The body stays whole: one body, the same volume and the same area, only more faces. Each piece is a split face (@sec-boolean-history).

In the sample, the plane z = 10 runs round the clamp at half its height. It crosses the four sides and the bore, so those five faces become ten, and the block's 7 faces become 12. A plane that crosses no face, or only touches the body along a face or an edge it already has, leaves the faces as they were.

#tip(title: [What the line is for])[Divided faces can be given to a later step one side at a time. A draft, for example, takes a list of faces (@ch-draft): divide the sides at a mold's parting line first, and you can tilt only the faces below it. Find their numbers with the topology report (@sec-topology).]

== Common mistakes <sec-pattern-mistakes>

#set par(justify: false)

- *A count of zero.* #raw("invalid input: pattern count 1 must be at least 1") (or `count 2`, or `pattern count` for a circular pattern). The count includes the original, so the smallest is 1, which gives the body alone.
- *A direction, normal or axis of length zero.* #raw("invalid input: pattern direction 1 must be non-zero"), #raw("mirror plane normal must be non-zero"), or #raw("pattern axis direction must be non-zero"). Give a direction that points somewhere, such as `[1.0, 0.0, 0.0]`.
- *A number that isn't a number.* #raw("invalid input: pattern spacing 1 must be finite, got [NaN]"). The same message names `mirror plane origin`, `pattern axis origin` or a direction when one of those holds an infinite value or NaN, usually the result of a division by zero earlier in your program.
- *An angle in degrees.* #raw("invalid input: pattern total angle must be in (0, 2π] radians, got 6.981317007977318"). The total angle is in radians: 360° is 2π, about 6.283. Convert with `to_radians()`. An angle of 0 or less is refused too.
- *A path with a gap, a circle, or nothing in it.* #raw("invalid input: sweep path: segment 1 doesn't meet segment 0 within 0.0000001"), #raw("sweep path: circles (closed paths) are not supported"), or #raw("sweep path is empty"). A path pattern checks its path as a sweep does, so the message says "sweep path" (@sec-sweep-mistakes). Join the pieces end to end, and use an arc instead of a whole circle.
- *A body away from the path's start.* The copies keep the body's offset from the start, so they follow a line beside the path. Put the body at the start of the path.
- *A split plane of length zero, or not a number.* #raw("invalid input: split plane normal must be non-zero"), or #raw("split plane origin must be finite, got [NaN, 0.0, 0.0]").
- *Expecting one piece on each side.* A split gives a list for each side; a side can hold several pieces, or none when the plane misses the body. Check the lengths before taking a piece.
- *Expecting the copies to be joined.* A pattern gives back a list of separate shapes. Cut or join them with `combine` to change your part.
