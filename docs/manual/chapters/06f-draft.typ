#import "../template.typ": *

= Tilting faces with draft <ch-draft>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("draft")
Plastic and cast parts are made in molds. Once the material has set, the part has to come out. If its sides are perfectly upright, they rub against the mold all the way out, and the part sticks or scrapes. So designers tilt the sides a little, a degree or a few, so that the part comes free as soon as it starts to move. That tilt is called #term[draft].

Three things set a draft:

- *The #term[pull direction]:* the way the part leaves the mold.
- *The #term[neutral plane]:* a flat plane across the part, at right angles to the pull direction. Where each tilted face crosses it, the face stays where it was; the face turns about that line. On a part that sits on a mold's floor, the neutral plane is usually its bottom, so the bottom keeps its size.
- *The angle,* and which faces to tilt. A positive angle makes the part smaller as you go along the pull direction, which is what lets it come out. A negative angle makes it larger.

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example draft_block -- draft
    // swx-qa render --step draft/draft-<name>.step --size 900x700 -o docs/manual/img/draft-<name>.png
    ("img/draft-four.png", "Four sides, 5°", "The block with all four sides tilted inward, so its top is smaller than its bottom, like a tapered loaf"),
    ("img/draft-one.png", "One side, 5°", "The block with only its right-hand end tilted inward; the other sides stay upright"),
    ("img/draft-flared.png", "Four sides, −5°", "The block with all four sides tilted outward, so its top is larger than its bottom"),
  ),
  caption: [A 40 × 25 × 20 mm block, drafted three ways. The neutral plane is the bottom and the part is pulled up, so the bottom keeps its size and the top changes: 1.75 mm in from each tilted side at 5°, 1.75 mm out at −5°.],
) <fig-draft>

*In Saulitwerx today:* the geometry engine can draft flat faces of a solid by one angle, about one neutral plane. You give the faces by their numbers in the solid (@sec-topology), and the neutral plane as a point and the pull direction. Every face in one call gets the same angle. A draft isn't recorded in a part's history yet.

== Draft the sides of a block <sec-draft>

#task-intro(
  goal: [Tilt the four sides of a block by 5°, then one side only, then the four sides outward.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `draft`:
  #term-block("cargo run -p swx-kernel --features occt --example draft_block -- draft")
  It prints the block and its faces, one line for each draft with the size of the bottom and the top, four errors on purpose, then the files it wrote:
  #term-block(label: "Output", "block          20000.00 mm³  bottom 40.00 × 25.00 mm  top 40.00 × 25.00 mm\nside faces [1, 0, 3, 2], top face 5\n4 sides, 5°    17806.94 mm³  bottom 40.00 × 25.00 mm  top 36.50 × 21.50 mm\n+X side, 5°    19562.56 mm³  bottom 40.00 × 25.00 mm  top 38.25 × 25.00 mm\n4 sides, -5°   22356.35 mm³  bottom 40.00 × 25.00 mm  top 43.50 × 28.50 mm\ntop face -> error: kernel failure: draft failed\n90° -> error: invalid input: draft angle must be finite, non-zero and\n    below π/2 in size, got 1.5707963267948966\n0° -> error: invalid input: draft angle must be finite, non-zero and\n    below π/2 in size, got 0\nno faces -> error: invalid input: the face list is empty\n…  (the geometry engine's file reports)\nwrote draft/draft-four.step\nwrote draft/draft-one.step\nwrote draft/draft-flared.step")
  Two long error lines are wrapped here to fit the page.]],
  [Read the second line. The four upright sides are faces 0 to 3, and the top is face 5.],
  [Read the `4 sides, 5°` line. The bottom is still 40 × 25 mm. Each side leans in by 20 × tan 5° = 1.75 mm over the block's 20 mm height, so the top is 3.5 mm smaller each way: 36.50 × 21.50 mm.],
  [Read the `+X side` line. Only the end facing +X leans, so only the length shrinks at the top.],
  [Read the `-5°` line. A negative angle leans the sides out, so the top is 3.5 mm larger each way.],
  [Read the `top face` line. The top is parallel to the neutral plane, so it doesn't cross it and has no line to turn about. The geometry engine can't draft it and says so. The block is unchanged.],
  [Draw the drafted block:
  #term-block("target/release/swx-qa render --step draft/draft-four.step -o draft-four.png")
  Open `draft-four.png`. You should see the first block of @fig-draft. Draw the other files the same way.],
)

#result[A block drafted three ways, each by one call. The block itself is left as it was.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/draft_block.rs`. It does two things.

#steps(
  [*Find the faces.* The sample finds each side by the way its outward normal points, as @sec-shell found the top of a block, with `face_plane` (@sec-extrude-to-face).],
  [#block(breakable: false)[*Draft them.* `draft` takes the solid, the list of faces, the neutral plane and the angle in radians. The neutral plane is an `Axis`: a point on the plane, and the pull direction, which is also the plane's normal:
  ```rust
  let pull_up = Axis { origin: [0.0, 0.0, 0.0], dir: [0.0, 0.0, 1.0] };
  let tapered = k.draft(&block, &sides, pull_up, 5f64.to_radians())?;
  ```]],
)
]

#tip(title: [How much draft])[One or two degrees is common for plastic parts, and more for textured surfaces or deep parts. Ask whoever makes the mold. Draft the sides before you round their edges: a fillet made first would have to be tilted too.]

#caution(title: [Angles are in radians])[`5.0` means 5 radians, which is refused. Write `5f64.to_radians()`. The angle must not be zero, and must be less than π/2 (90°) either way.]

== Common mistakes <sec-draft-mistakes>

#set par(justify: false)

- *A face parallel to the neutral plane.* A top or a bottom can't be drafted: `kernel failure: draft failed`. Draft the faces that run along the pull direction.
- *An angle of zero, 90° or more, or in degrees.* `invalid input: draft angle must be finite, non-zero and below π/2 in size, got 0`.
- *No faces.* `invalid input: the face list is empty`.
- *An edge or a vertex in the list, or a face number from another solid.* `invalid input: expected a face, got edge 0`, or `face index 6 is out of range: the shape has 6 faces`. Find the faces in the solid you are drafting.
- *No pull direction.* `invalid input: draft pull direction must be finite and non-zero, got [0.0, 0.0, 0.0]`.
- *The wrong sign.* If the part grows where it should shrink, the pull direction points the wrong way, or the angle's sign is wrong. Flip one of them.
