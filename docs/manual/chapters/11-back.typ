#import "../template.typ": *
#set heading(numbering: none)

= Glossary

#{
  let g = (
    ("Bounding box", [The smallest box, with sides parallel to the axes, that holds a solid.]),
    ("Boolean operation", [Combining two solids by union, cut or intersect.]),
    ("CAD", [Computer-aided design: using a computer to design physical objects.]),
    ("Coincident", [A constraint that puts two points at the same place.]),
    ("Constraint", [A rule about a sketch's shape, such as "horizontal" or "these points touch".]),
    ("Cut", [A Boolean operation that removes one solid from another.]),
    ("Degree of freedom", [One way a piece of a sketch can still move. A free point has two.]),
    ("Diameter", [The width of a circle through its center; twice the radius.]),
    ("Dimension", [A constraint with a number: a distance, radius or diameter.]),
    ("Dirty", [Said of a feature that must be rebuilt after a change.]),
    ("Edge", [A curve where two faces of a solid meet.]),
    ("Entity", [One piece of sketch geometry: a point, line, circle or arc.]),
    ("Face", [One flat or curved patch of a solid's surface.]),
    ("Feature", [One step in building a part, such as a sketch or a cut. Features are kept in the history.]),
    ("Feature tree", [Another name for a part's history.]),
    ("Fixed", [A constraint that keeps a point at exact coordinates.]),
    ("Fully constrained", [A sketch whose constraints and dimensions leave nothing free to move.]),
    ("Geometry kernel", [The part of a CAD program that does the 3D math. Saulitwerx uses OpenCascade.]),
    ("History", [A part's features, in the order they were made.]),
    ("Intersect", [A Boolean operation that keeps only what two solids share.]),
    ("Isometric", [A view from a corner, looking down at an angle, so that three sides of a box show equally.]),
    ("OpenCascade (OCCT)", [The open-source geometry kernel Saulitwerx uses.]),
    ("Parameter", [A named number that features and dimensions refer to.]),
    ("Parametric", [Built from steps and numbers that can be changed later, with the design rebuilding to match.]),
    ("Pre-alpha", [Early, unfinished software: the parts work, but it isn't ready for real use.]),
    ("Sketch", [A 2D drawing on a plane, made of points, lines, circles and arcs.]),
    ("Solid", [A closed 3D shape with an inside and an outside.]),
    ("Solver", [The calculation that moves sketch geometry until every constraint holds.]),
    ("STEP", [An international standard file format (ISO 10303) for exchanging 3D parts.]),
    ("Surface area", [The total area of a solid's faces.]),
    ("Union", [A Boolean operation that joins two solids into one.]),
    ("Unit", [What a number measures in, such as millimeters.]),
    ("Volume", [How much space a solid fills.]),
  )
  set par(first-line-indent: 0pt, justify: false)
  columns(2, gutter: 18pt, for (t, d) in g {
    block(spacing: 0.75em, breakable: false, [#text(font: sans, weight: "semibold", size: 9pt, t) \ #text(size: 9.5pt, d)])
  })
}

= What's new

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
  [More constraints], [Parallel, perpendicular, tangent, equal, midpoint and symmetric.],
  [Degree-of-freedom report], [See whether a sketch is under-, fully or over-constrained, and which constraints conflict.],
  [Profiles], [Turn closed loops in a sketch into regions, with holes, ready to become solids.],
  [Pictures of sketches], [Draw a solved sketch, with its points and dimension values, to a PNG.],
  [Dimension parameters], [Every dimension becomes a named parameter, `d0`, `d1`, …, with expressions and units.],
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
      #text(font: sans, weight: "bold", fill: accent, "SAMPLES")
      #term-block("cargo run -p swx-sketch \\\n  --example plate_sketch\ncargo run -p swx-kernel --features occt \\\n  --example plate_with_hole -- plate.step\ncargo run -p swx-core \\\n  --example feature_history")
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

Every picture of a part in this manual was drawn by Saulitwerx's own renderer, `swx-qa render`, from geometry made by Saulitwerx.

= Index

#print-index()
