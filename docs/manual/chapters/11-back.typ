#import "../template.typ": *
#set heading(numbering: none)

= Glossary

#{
  let g = (
    ("Bounding box", [The smallest box, with sides parallel to the axes, that holds a solid.]),
    ("Boolean operation", [Combining two solids by union, cut or intersect.]),
    ("CAD", [Computer-aided design: using a computer to design physical objects.]),
    ("Coincident", [A constraint that puts two points at the same place.]),
    ("Conflicting", [Said of a sketch whose rules contradict each other, so no shape can meet them all.]),
    ("Constraint", [A rule about a sketch's shape, such as "horizontal" or "these points touch".]),
    ("Cut", [A Boolean operation that removes one solid from another.]),
    ("Degree of freedom (DOF)", [One way a piece of a sketch can still move. A free point has two.]),
    ("Diameter", [The width of a circle through its center; twice the radius.]),
    ("Dimension", [A constraint with a number: a distance, radius or diameter.]),
    ("Dirty", [Said of a feature that must be rebuilt after a change.]),
    ("Edge", [A curve where two faces of a solid meet.]),
    ("Equal", [A constraint that makes two lines the same length, or two circles or arcs the same size.]),
    ("Entity", [One piece of sketch geometry: a point, line, circle or arc.]),
    ("Face", [One flat or curved patch of a solid's surface.]),
    ("Feature", [One step in building a part, such as a sketch or a cut. Features are kept in the history.]),
    ("Feature tree", [Another name for a part's history.]),
    ("Fixed", [A constraint that keeps a point at exact coordinates.]),
    ("Fully constrained", [A sketch whose constraints and dimensions leave nothing free to move.]),
    ("Geometry kernel", [The part of a CAD program that does the 3D math. Saulitwerx uses OpenCascade.]),
    ("History", [A part's features, in the order they were made.]),
    ("Intersect", [A Boolean operation that keeps only what two solids share.]),
    ("Island", [A region inside a hole of another region.]),
    ("Isometric", [A view from a corner, looking down at an angle, so that three sides of a box show equally.]),
    ("Loop", [A closed chain of sketch pieces, each ending where the next begins.]),
    ("Midpoint", [A constraint that keeps a point halfway along a line.]),
    ("Model parameter", [A parameter that belongs to a dimension, named `d0`, `d1`, … in turn.]),
    ("Open edge", [A sketch piece that isn't part of any closed loop.]),
    ("OpenCascade (OCCT)", [The open-source geometry kernel Saulitwerx uses.]),
    ("Over-constrained", [Said of a sketch with a rule that repeats what other rules already say.]),
    ("Parallel", [A constraint that makes two lines point the same way.]),
    ("Parameter", [A named number that features and dimensions refer to.]),
    ("Parametric", [Built from steps and numbers that can be changed later, with the design rebuilding to match.]),
    ("Perpendicular", [A constraint that makes two lines meet at a right angle.]),
    ("Pre-alpha", [Early, unfinished software: the parts work, but it isn't ready for real use.]),
    ("Profile", [The regions of a sketch: the areas a feature can turn into a solid.]),
    ("Redundant", [Said of a rule that repeats what other rules already say.]),
    ("Region", [A flat area enclosed by an outer loop, minus any holes inside it.]),
    ("Sketch", [A 2D drawing on a plane, made of points, lines, circles and arcs.]),
    ("Solid", [A closed 3D shape with an inside and an outside.]),
    ("Solver", [The calculation that moves sketch geometry until every constraint holds.]),
    ("Symmetric", [A constraint that makes two points mirror images across a line.]),
    ("STEP", [An international standard file format (ISO 10303) for exchanging 3D parts.]),
    ("Surface area", [The total area of a solid's faces.]),
    ("Tangent", [Touching without cutting in: a line and a circle, or two circles, that meet at one point.]),
    ("Under-constrained", [Said of a sketch with at least one degree of freedom left.]),
    ("Union", [A Boolean operation that joins two solids into one.]),
    ("Unit", [What a number measures in, such as millimeters.]),
    ("User parameter", [A parameter you create yourself, with a name you choose.]),
    ("Volume", [How much space a solid fills.]),
  )
  set par(first-line-indent: 0pt, justify: false)
  columns(2, gutter: 18pt, for (t, d) in g {
    block(spacing: 0.45em, breakable: false, [#text(font: sans, weight: "semibold", size: 9pt, t) \ #text(size: 9.2pt, d)])
  })
}

= What's new

== 2026-10-26: parameter table, and named conflicts

- *New:* a part's parameters form a table: model parameters named `d0`, `d1`, … in turn, user parameters with names you choose, units and comments. Renaming a parameter updates every feature that uses it (@sec-parameter, @sec-param-table, @sec-ref-params).
- *Changed:* the degree-of-freedom report now names the redundant or conflicting constraints, not just equation numbers (@sec-check-dof, @sec-ref-dof). The matching known limitation is gone.
- Help: two new parameter error messages, and two new common mistakes.

== 2026-10-25: more constraints, and profiles

- *New:* six constraints: parallel, perpendicular, equal, midpoint, symmetric and tangent (@sec-constraint, @sec-ref-sketch).
- *New:* keep a hole centered whatever the plate's size (@sec-centered), and draw a slot with tangent, equal-sized ends (@sec-slot).
- *New:* find the regions a sketch encloses, with their holes and areas, and the pieces left open (@sec-profile, @sec-profiles, @sec-ref-profile).
- *Pictures:* the centered plate at two widths, and a slot with a gap, ringed by the template.
- Help: profile error messages, two new troubleshooting rows, and known limitations updated, including arcs whose ends can't be constrained.
- Corrected: midpoint and symmetric add two equations each to the degree-of-freedom count.

== 2026-10-24: degrees of freedom and sketch pictures

- *New:* check how fully a sketch is defined: degrees of freedom left, and whether it is under-, fully or over-constrained, or conflicting (@sec-dof, @sec-check-dof, @sec-ref-dof).
- *New:* draw a sketch to a PNG with `swx-qa sketch` (@sec-picture-sketch, @sec-ref-qa-sketch).
- *Pictures:* the sketch sections of *Concepts* and *Sketching* now show the plate sketch, rough and solved.
- Help: new troubleshooting rows and `swx-qa sketch` error messages; known limitations updated.

== 2026-10-22: first edition

The first edition of this manual. It covers everything on `main` today:

- Installing from source, and a first tutorial that builds, measures, saves and draws a plate with a hole.
- The ideas of parametric CAD: solids, units, sketches, constraints, dimensions, parameters, features and history.
- Sketching with the coincident, horizontal, vertical, fixed, distance, radius and diameter constraints, and the solver.
- Making and combining solids, and saving and reading STEP files.
- Drawing parts with `swx-qa render`.
- The feature history, and finding what a change rebuilds.
- Reference tables, every error message, troubleshooting, and known limitations.

= Coming next

These are the next items on the public roadmap. None of them works yet. When one arrives, it moves into the chapters of this manual and into *What's new*.

#ref-table(
  columns: (auto, 1fr),
  header: ("Soon", "What you'll be able to do"),
  [Parameter expressions], [Write a parameter as a formula with units, such as `d1 / 2 + 5 mm`.],
  [Dimension parameters], [Every sketch dimension makes its own model parameter, `d0`, `d1`, …, by itself.],
  [Sketch inference], [Constraints such as horizontal, tangent and coincident added for you while you draw.],
  [Part features], [Extrude, revolve, fillet, chamfer, shell and hole, driven by sketches.],
  [Part files], [Save a part with its full history, and open it again.],
  [Desktop app], [A window with a 3D view, a model browser, sketch mode and property panels.],
)

Later milestones add assemblies, 2D drawings and exploded views. The full plan is in `docs/ROADMAP.md` in the source code.

#page(header: none)[
  #set par(first-line-indent: 0pt, justify: false)
  #set text(size: 8.6pt)
  #text(font: sans, size: 18pt, weight: "bold", "Quick reference")
  #h(1fr) #text(font: sans, size: 8pt, fill: muted, "Saulitwerx, pre-alpha")
  #v(-2pt)
  #line(length: 100%, stroke: 1.5pt + accent)
  #grid(columns: (1.3fr, 1fr), column-gutter: 16pt, row-gutter: 12pt,
    [
      #text(font: sans, weight: "bold", fill: accent, "BUILD")
      #term-block("cargo test --workspace\ncargo build --release -p swx-qa \\\n  --features occt")
      #text(font: sans, weight: "bold", fill: accent, "DRAW A PART")
      #term-block("swx-qa render --step part.step -o part.png\nswx-qa render --fixture cube -o cube.png\n  --size 1600x1000   (default 800x600)\n  --crease 45     (default 30)")
      #text(font: sans, weight: "bold", fill: accent, "DRAW A SKETCH")
      #term-block("swx-qa sketch --fixture rectangle -o r.png\n  (or triangle-345)")
      #text(font: sans, weight: "bold", fill: accent, "SAMPLES")
      #term-block("cargo run -p swx-sketch \\\n  --example plate_sketch\ncargo run -p swx-sketch \\\n  --example sketch_dof\ncargo run -p swx-sketch \\\n  --example centered_plate -- 60 25\ncargo run -p swx-sketch \\\n  --example slot_sketch\ncargo run -p swx-kernel --features occt \\\n  --example plate_with_hole -- plate.step\ncargo run -p swx-core \\\n  --example feature_history\ncargo run -p swx-core \\\n  --example parameter_table")
    ],
    [
      #text(font: sans, weight: "bold", fill: accent, "SKETCH CONSTRAINTS")
      #ref-table(columns: (auto, 1fr), header: ("Constraint", "Keeps"),
        [Coincident], [two points together],
        [Horizontal], [a line level],
        [Vertical], [a line upright],
        [Fixed], [a point at (x, y)],
        [Distance], [two points a length apart],
        [Radius / Diameter], [a circle's size],
        [Parallel · Perpendicular], [two lines' directions],
        [Equal], [two lengths or radii alike],
        [Midpoint], [a point mid-line],
        [Symmetric], [two points mirrored],
        [Tangent], [a line touching a curve],
      )
      #text(font: sans, weight: "bold", fill: accent, "SKETCH STATES")
      #ref-table(columns: (auto, 1fr), header: ("State", "Means"),
        [Under-constrained], [can still move: add rules],
        [Fully constrained], [done: 0 DOF left],
        [Over-constrained], [a rule repeats others],
        [Conflicting], [rules contradict],
      )
      #text(font: sans, weight: "bold", fill: accent, "SOLIDS")
      #ref-table(columns: (auto, 1fr), header: ("Call", "Gives"),
        [`make_box`], [box from corner + sizes],
        [`make_cylinder`], [cylinder on +Z],
        [`make_sphere`], [sphere],
        [`union` · `cut` · `intersect`], [Boolean result],
        [`export_step` · `import_step`], [STEP file out / in],
      )
      #text(font: sans, weight: "bold", fill: accent, "UNITS")
      \ Millimeters. Angles: degrees in feature parameters, radians in sketch arcs.
    ],
  )
]

= Credits and licenses

Saulitwerx is written by its contributors and is licensed under either the MIT license or the Apache License 2.0, at your choice. It is an independent project, not affiliated with or derived from any commercial CAD product.

#ref-table(
  columns: (1.3fr, 1fr, 1.5fr),
  header: ("Component", "License", "Notes"),
  [OpenCascade Technology (OCCT) 7.6], [LGPL-2.1 with the OCCT exception], [The geometry kernel. Used as a separate, dynamically linked library; not included in Saulitwerx's source.],
  [`png`, `flate2`, `miniz_oxide`, `fdeflate`, `crc32fast`, `adler2`, `simd-adler32`, `bitflags`, `cfg-if`], [MIT, Apache-2.0, Zlib or 0BSD], [Write the PNG pictures of `swx-qa`.],
  [`cc`, `shlex`, `find-msvc-tools`], [MIT or Apache-2.0], [Build the bridge to OpenCascade.],
  [Typst 0.15.1], [Apache-2.0], [Typesets this manual.],
  [Inter 4.0, by The Inter Project Authors], [SIL Open Font License 1.1], [Headings and labels in this manual. License in `docs/manual/fonts/OFL-Inter.txt`.],
  [Libertinus Serif, DejaVu Sans Mono], [SIL Open Font License 1.1; Bitstream Vera license], [Body text and code in this manual, as bundled with Typst.],
)

Every picture of a part in this manual was drawn by Saulitwerx's own renderer, `swx-qa render`, and every picture of a sketch by `swx-qa sketch`'s plotter, from geometry made by Saulitwerx.

= Index

#print-index()
