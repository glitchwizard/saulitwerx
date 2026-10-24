#import "../template.typ": *

= Pictures of parts and sketches <ch-pictures>

`swx-qa render` draws a part to a PNG picture file, and `swx-qa sketch` draws a sketch. They need no graphics card and no window, and the same input always gives the same picture, byte for byte. The project uses them to show its work in reviews; this manual's pictures come from them too.

== Draw a STEP file

#task-intro(
  goal: [Draw any STEP file to a PNG, at the size you want.],
  needs: [`swx-qa` built with OpenCascade (@ch-install), and a STEP file. `plate.step` from @ch-first works.],
)

#steps(
  [Run `render` with the STEP file, a size and an output name:
  #term-block("target/release/swx-qa render --step plate.step --size 1600x1000 -o plate.png")
  It answers `wrote plate.png (1600x1000, …)`, with the file size in bytes.],
  [Open `plate.png`. The part fills the picture, seen from the same corner as every other render.],
)

#result[A PNG of the part, ready to share or to put in a document.]

The renderer chooses how finely to break curved faces into flat pieces for you: the mesh stays within 0.1 % of the part's overall size of the true surface.

== Choose the size

`--size` takes the width and height in pixels, written `W` `x` `H`, for example `1600x1000`. The default is `800x600`; the largest is `8192x8192`. For a picture you will print, allow about 80 pixels per centimeter (200 per inch) of printed width: a picture printed 10 cm wide needs about 800 pixels.

== Choose which edges are outlined

The renderer outlines the edges where faces meet at a sharp angle. `--crease` sets how sharp, in degrees, from 0 to 180; the default is 30. Where the surface bends by more than this angle from one face to the next, the edge gets a dark line. The edges of a box bend by 90°, so they are always outlined; the many small facets of a curved face bend by only a few degrees, so they are not. A smaller value outlines more edges; a larger value outlines fewer.

#tip[On a curved part that looks striped with lines, raise `--crease` (try 45). If an edge you want to see has no outline, lower it.]

== Draw a built-in sample

`--fixture` draws one of three built-in parts instead of a file: `cube` (10 mm), `cylinder` (5 mm radius, 10 mm tall) or `l-bracket` (30 mm legs, 8 mm thick, 20 mm deep). They need no OpenCascade, so they are a quick way to test the tool.

#term-block("target/release/swx-qa render --fixture l-bracket -o bracket.png")

@sec-ref-qa lists every option.

== Draw a sketch <sec-picture-sketch>

`swx-qa sketch` solves a built-in sketch and draws it from above, with x to the right and y up. Lines, circles and arcs are dark blue; points and the ends of lines are red dots; each dimension's value is written in green beside what it measures.

#task-intro(
  goal: [Draw a built-in sketch to a PNG.],
  needs: [`swx-qa` built (@ch-install). OpenCascade is not needed.],
  time: [2 minutes.],
)

#steps(
  [Run `sketch` with a sample name and an output name:
  #term-block("target/release/swx-qa sketch --fixture rectangle -o rectangle.png")
  It answers `wrote rectangle.png (800x600, …)`, with the file size in bytes.],
  [Open `rectangle.png`. You should see a 40 × 25 mm rectangle, labeled `40` along the bottom and `25` up the right side.],
  [Try the other sample, a right triangle with sides of 4 and 3:
  #term-block("target/release/swx-qa sketch --fixture triangle-345 -o triangle.png")],
)

#result[A PNG of a solved sketch with its dimension values (@fig-sketch-fixtures).]

#image-equation(
  (
    // swx-qa sketch --fixture rectangle --size 600x420 -o docs/manual/img/sketch-rectangle.png
    ("img/sketch-rectangle.png", "rectangle"),
    // swx-qa sketch --fixture triangle-345 --size 600x420 -o docs/manual/img/sketch-triangle-345.png
    ("img/sketch-triangle-345.png", "triangle-345"),
  ),
  ("",),
  caption: [The two built-in sketches. The triangle's long side has no number: it has no dimension of its own, and comes out 5 because the other two sides are 4 and 3.],
) <fig-sketch-fixtures>

`--size` works as it does for parts; the default is `800x600`. The sketch is scaled to fill the picture with a margin, so a small sketch and a large one come out the same size. Labels are drawn at a fixed size in pixels, so they look smaller in a large picture.

#for-developers[To draw your own sketch, call `swx_qa::sketch::plot(&sketch, &PlotOptions::default())` and save the result with `to_png()`. It draws the sketch as it is, solved or not. The manual's sketch pictures come from `crates/swx-qa/examples/plate_sketch_plot.rs`, which does exactly this:
#term-block("cargo run -p swx-qa --example plate_sketch_plot -- <dir> 600x420")
It writes `sketch-rough.png`, `sketch-shape.png` and `sketch-sized.png` in `<dir>`.]

Only distances, radii and diameters are labeled. Other constraints, such as horizontal or fixed, don't show in the picture, and there are no dimension arrows yet.
