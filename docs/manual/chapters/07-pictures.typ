#import "../template.typ": *

= Pictures of parts <ch-pictures>

`swx-qa render` draws a part to a PNG picture file. It needs no graphics card and no window, and the same input always gives the same picture, byte for byte. The project uses it to show its work in reviews; this manual's part pictures come from it too.

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
