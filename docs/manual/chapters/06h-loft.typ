#import "../template.typ": *

= Lofting between sections <ch-loft>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("loft")
An extrude pushes one outline straight, so the part has the same cross-section all the way along. A revolve turns one outline, so the part is round. Many real shapes are neither: a duct that starts square and ends round, a bottle that swells and narrows, a handle that tapers. For those you draw the shape at a few places along the part and let the computer fill in between.

That is a #term[loft]. Each outline you draw is a #term[section]: a flat face, like a slice through the finished part. The loft joins the sections in the order you give them, and closes the two ends with the first and last sections. It can join them two ways:

- *Ruled:* straight surfaces from each section to the next, like a lampshade made of straight strips. Each pair of neighbors gets its own surface, so you see a crease at every middle section.
- *Smooth:* one curved surface that passes through every section with no crease, like a vase thrown on a wheel.

With only two sections, ruled and smooth give the same shape.

#image-row(
  (
    // cargo run -p swx-kernel --features occt --example loft_sections -- loft
    // swx-qa render --step loft/loft-<name>.step --size 900x700 -o docs/manual/img/loft-<name>.png
    ("img/loft-transition.png", "Square to round", "A solid that is a square at the bottom and narrows to a circle at the top, with flat-looking sides that curve into the round top"),
    ("img/loft-ruled.png", "Three circles, ruled", "A vase shape made of two cones, wide at the bottom, narrow in the middle and wider at the top, with a sharp crease at the waist"),
    ("img/loft-smooth.png", "Three circles, smooth", "The same vase shape with one smooth curved side and no crease at the waist"),
  ),
  caption: [Three lofts. Left: a 40 × 40 mm square at the bottom, a circle 24 mm across 40 mm higher. Middle and right: circles 40, 20 and 32 mm across at heights 0, 30 and 60 mm, joined ruled and smooth. The faint ripples on the curved faces come from the picture's coarse mesh, not from the part.],
) <fig-loft>

*In Saulitwerx today:* the geometry engine can loft through two or more flat sections, ruled or smooth. Each section is one face from `make_face` (@ch-extrude), on any plane, with no holes. You give the sections in order; the loft has no guide curves yet, and no control over which point of one section joins which point of the next. A loft makes a new solid; to add it to a part, join it with `union` (@sec-feature). It isn't recorded in a part's history.

== Loft through sections <sec-loft>

#task-intro(
  goal: [Loft a square into a circle, then a vase through three circles, ruled and smooth.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `loft`:
  #term-block("cargo run -p swx-kernel --features occt --example loft_sections -- loft")
  It prints one line for each loft, four errors on purpose, then the files it wrote:
  #term-block(label: "Output", "square to round  38889.24 mm³  7 faces  height 40.00 mm\nvase, ruled      38201.77 mm³  4 faces  height 60.00 mm\nvase, smooth     31508.61 mm³  3 faces  height 60.00 mm\none section -> error: invalid input: a loft needs at least 2 sections,\n    got 1\nhole in top -> error: invalid input: loft section 1 has a hole; sections\n    with holes are not supported\nsolid section -> error: invalid input: loft section 1 is not a single\n    planar face\nsame section twice -> error: kernel failure: loft failed\n…  (the geometry engine's file reports)\nwrote loft/loft-transition.step\nwrote loft/loft-ruled.step\nwrote loft/loft-smooth.step")
  Four long error lines are wrapped here to fit the page.]],
  [Read the `square to round` line. The part is 40 mm tall, from the square up to the circle.],
  [Read the `vase, ruled` line. It is two cones stacked at the waist: 4 faces, a bottom, a top and one side for each pair of sections. Its volume is the two cones' volumes added up.],
  [Read the `vase, smooth` line. It has one side face, curving through all three circles. Because the side bows inward between the sections instead of running straight, it holds less: 31508.61 mm³ against 38201.77 mm³.],
  [Read the errors. A loft needs two sections or more; each section must be one flat face without a hole; and two sections in the same place give nothing to fill, so the geometry engine refuses them.],
  [#block(breakable: false)[Draw the lofts:
  #term-block("target/release/swx-qa render --step loft/loft-smooth.step -o loft-smooth.png")
  Open `loft-smooth.png`. You should see the right-hand part of @fig-loft. Draw the other files the same way.]],
)

#result[Three solids, each from one call, that no extrude or revolve could make.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/loft_sections.rs`. It does two things.

#steps(
  [*Make the sections.* Each section is a face from `make_face` (@ch-extrude): an outline on a workplane. The sample puts each one on the XY plane raised to its height with `offset` (@sec-work):
  ```rust
  let bottom = k.make_face(&Workplane::XY, &square, &[])?;
  let top = k.make_face(&Workplane::XY.offset(40.0)?, &circle, &[])?;
  ```],
  [*Loft them.* `loft` takes the sections in order, and `true` for ruled or `false` for smooth:
  ```rust
  let transition = k.loft(&[bottom, top], true)?;
  let vase = k.loft(&rings, false)?;
  ```],
)
]

#tip(title: [Sections don't have to be parallel])[Each section can sit on its own plane: a work plane tilted or turned (@ch-work) gives a loft that bends. Keep the sections in the order the part runs, and don't let them cross each other.]

#tip(title: [Ruled or smooth])[Choose ruled when the part really is made of straight pieces, such as a sheet-metal transition, and smooth for anything that should flow, such as a handle or a nozzle. With a smooth loft, add a section where you want to control the shape: the surface passes through every section you give it.]

== Common mistakes <sec-loft-mistakes>

#set par(justify: false)

- *One section, or none.* `invalid input: a loft needs at least 2 sections, got 1`.
- *A section with a hole.* A ring-shaped outline is refused: `invalid input: loft section 1 has a hole; sections with holes are not supported`. Loft the outer outline, then cut the hole out with a second loft or an extrude.
- *A solid in place of a face.* `invalid input: loft section 1 is not a single planar face`. Pass faces from `make_face`, not the solids made from them.
- *Two sections in the same place.* `kernel failure: loft failed`. Move one section, or leave it out.
- *Sections out of order.* The loft joins them in the order of the list, so a list out of order makes a part that folds back on itself, or fails. Sort them along the part.
- *Very different outlines.* A star into a circle, say, may twist or fail, because the loft has to decide which point of one joins which point of the next. Give the outlines a similar number of pieces, starting at similar places.
