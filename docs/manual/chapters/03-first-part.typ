#import "../template.typ": *

= Your first part <ch-first>

In this tutorial you draw a sample part, then build a real one, a plate with a hole through it, measure it, save it in a standard file format, and draw it. Along the way you will meet the ideas the rest of the manual builds on.

#task-intro(
  goal: [Draw a built-in sample part, build a plate with a hole, check its measurements, save it as a STEP file, and draw it.],
  needs: [Saulitwerx built with OpenCascade support (@ch-install), and a terminal in the `saulitwerx` folder.],
  time: [About 15 minutes.],
)

== Draw a sample part

Saulitwerx comes with three sample parts built in: a cube, a cylinder and an L-shaped bracket. They need nothing else, so they are a quick way to check that drawing works.

#steps(
  [Draw the cube to a picture file named `cube.png`:
  #term-block("target/release/swx-qa render --fixture cube -o cube.png")
  The tool answers with one line:
  #term-block(label: "Output", "wrote cube.png (800x600, 27669 bytes)")],
  [Open `cube.png` in any image viewer. On Ubuntu, type `xdg-open cube.png`. You should see a grey-blue cube on a light grey background, like @fig-cube.],
)

#figure(
  grid(columns: (1fr, 1fr, 1fr), column-gutter: 10pt,
    // swx-qa render --fixture cube --size 900x700 -o docs/manual/img/fixture-cube.png
    box(fill: render-bg, radius: 3pt, clip: true, stroke: 0.5pt + rule-c, image("../img/fixture-cube.png", width: 100%, alt: "A cube seen from above at an angle; three faces show in three shades")),
    // swx-qa render --fixture cylinder --size 900x700 -o docs/manual/img/fixture-cylinder.png
    box(fill: render-bg, radius: 3pt, clip: true, stroke: 0.5pt + rule-c, image("../img/fixture-cylinder.png", width: 100%, alt: "A short upright cylinder with a flat top")),
    // swx-qa render --fixture l-bracket --size 900x700 -o docs/manual/img/fixture-l-bracket.png
    box(fill: render-bg, radius: 3pt, clip: true, stroke: 0.5pt + rule-c, image("../img/fixture-l-bracket.png", width: 100%, alt: "An L-shaped bracket lying on its side")),
  ),
  caption: [The three built-in sample parts: `cube`, `cylinder` and `l-bracket`.],
  kind: image,
) <fig-cube>

The picture is always drawn from the same corner, looking down at an angle, so that three sides of a box show at once. Each flat face gets one even shade, and sharp edges get a dark outline. This kind of view is called #term[isometric].

#tip[Try `--fixture cylinder` and `--fixture l-bracket` too. Add `--size 1600x1200` for a bigger, sharper picture.]

== Build a plate with a hole

Now build a real part. Saulitwerx doesn't have a window to draw in yet, so this part comes from a small program that ships with Saulitwerx. The program asks the geometry engine for three things:

+ a #term[solid] block 40 mm long, 25 mm wide and 5 mm thick;
+ a cylinder 10 mm across, standing where the hole should go, a little taller than the plate;
+ the block *minus* the cylinder. Taking one solid away from another is called a _cut_.

#steps(
  [Run the program, and tell it to save the part as `plate.step`:
  #term-block("cargo run -p swx-kernel --features occt --example plate_with_hole -- plate.step")
  After a few seconds of building, it prints the part's measurements. The geometry engine also prints a short report about writing the file, framed in rows of `*`; you can ignore it.
  #term-block(label: "Output", "volume        4607.30 mm³\nsurface area  2650.00 mm²\nfaces         7\nbounding box  [0.0, 0.0, 0.0] to [40.0, 25.0, 5.0]\n…  (the geometry engine's file report)\nwrote plate.step (read back: 4607.30 mm³, 7 faces)")],
  [Check the numbers. The block holds 40 × 25 × 5 = 5000 mm³. The hole removes π × 5² × 5 ≈ 392.7 mm³. 5000 − 392.7 = 4607.3, which is what the program reports. The part has 7 faces: top, bottom, four sides and the inside of the hole.],
  [Draw the part:
  #term-block("target/release/swx-qa render --step plate.step --size 1600x1000 -o plate.png")
  The tool answers `wrote plate.png (1600x1000, …)`.],
  [Open `plate.png`. You should see the plate of @fig-plate, with the hole showing light background through it.],
)

#shot(
  // swx-qa render --step plate.step --size 1600x1000 -o docs/manual/img/plate-with-hole.png
  "img/plate-with-hole.png",
  width: 80%,
  caption: [The plate you built, drawn from the STEP file the program saved.],
  alt: "A rectangular plate with a round hole through its middle, seen from above at an angle",
) <fig-plate>

#result[You built a solid part, checked its volume, saved it as `plate.step`, and drew it as `plate.png`.]

== What you just did

The file `plate.step` is in the #term(key: "STEP")[STEP] format, an international standard (ISO 10303) for exchanging 3D parts. Almost every CAD program can open it, so you can take this plate into another tool, send it to a machine shop, or prepare it for 3D printing.

What you did *not* see yet is the parametric part of Saulitwerx: the numbers 40, 25, 5 and 10 are typed into the program, not stored as named, editable parameters. That comes with the part features and part files on the roadmap. @ch-concepts explains the ideas behind it, and @ch-history shows the piece of it that already works.

#for-developers[
  The program you ran is `crates/swx-kernel/examples/plate_with_hole.rs`. Change `5.0` (the hole radius) in the `make_cylinder` line to `8.0`, run step 1 again, and the volume drops to 3994.69 mm³. @ch-solids explains each call.
]
