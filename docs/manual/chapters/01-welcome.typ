#import "../template.typ": *

= Welcome to Saulitwerx <ch-welcome>

== What Saulitwerx is

Saulitwerx is a program for designing solid mechanical parts on a computer: brackets, plates, shafts, housings, and anything else you might machine, cast or 3D-print. Programs like this are called #term(key: "CAD")[CAD] (computer-aided design). Saulitwerx is free and open source, so anyone can use it, study it, change it, and build it into other software.

Saulitwerx is a #term[parametric] modeler. You don't sculpt a shape by pushing points around. Instead you describe how to build it, step by step, with measurements: "draw a 40 by 25 mm rectangle, push it up 5 mm, cut a 10 mm hole through the middle." Saulitwerx remembers those steps and their numbers. Change the hole to 12 mm later, and the part rebuilds itself to match. @ch-concepts explains how that works.

#shot(
  // cargo run -p swx-kernel --features occt --example plate_with_hole -- plate.step
  // swx-qa render --step plate.step --size 1600x1000 -o docs/manual/img/plate-with-hole.png
  "img/plate-with-hole.png",
  width: 78%,
  caption: [A plate with a hole, made by Saulitwerx's geometry engine and drawn by its renderer. You will make this part yourself in @ch-first.],
  alt: "A rectangular plate, seen from above at an angle, with a round hole through its middle",
)

== What it can do today

Saulitwerx is built in small, tested steps, in the order of a public plan (the _roadmap_). Here is where it stands for this edition.

#ref-table(
  columns: (auto, 1fr, auto),
  header: ("Area", "What exists", "Use it"),
  [Solids], [Make boxes, cylinders and spheres; combine them (add, subtract, keep the overlap); measure volume, surface area and size.], [#now],
  [Exchange], [Save a solid as a STEP file, and read STEP files back.], [#now],
  [Pictures], [Draw a solid or a STEP file to a PNG picture with `swx-qa render`, and a sketch with `swx-qa sketch`.], [#now],
  [Sketching], [Lines, circles, arcs and points, with thirteen kinds of constraint (from horizontal and coincident to tangent and symmetric), a solver that moves the shape to satisfy them, a check of how fully a sketch is defined, and the regions a sketch encloses, ready to become solids.], [#now],
  [History], [Keep features in order with named parameters, and work out which features a change affects.], [#now],
  [Extrude], [Push a flat outline of lines, arcs and circles, with holes, into a solid, on any plane, one way or both.], [#now],
  [Desktop app], [Windows, menus, a 3D view and mouse control.], [#later],
  [Part files], [Saving your design with its full history in a Saulitwerx part file.], [#later],
  [Features], [Extrude driven by sketches and recorded in the history; revolve, fillet, chamfer, hole, shell and more.], [#later],
  [Assemblies, drawings], [Putting parts together, and making 2D shop drawings.], [#later],
)

Everything marked #now is a building block that programmers can call today. The *Tasks* part shows each one with a real command or a short program you can run. Everything marked #later is described in *Coming next* at the back, and nowhere else.

== Who should use it now

- *If you are new to CAD,* read Parts I and II. You will learn the ideas every parametric modeler is built on, and you will be ready the day the desktop app arrives.
- *If you are comfortable in a terminal,* install Saulitwerx (@ch-install) and follow @ch-first. It takes about 15 minutes and ends with a picture of a part you built.
- *If you write software,* the *For developers* sections show how to call each building block from Rust.

#note[Saulitwerx reads and writes only open, documented file formats. It will never open another program's own native files. To bring in a design from elsewhere, export it as STEP first.]
