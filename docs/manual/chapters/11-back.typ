#import "../template.typ": *
#set heading(numbering: none)

= Glossary

#{
  let g = (
    ("Adjacent", [Said of two sub-shapes when one bounds the other, such as a face and one of its edges.]),
    ("Binding", [The link between a sketch dimension and the parameter that drives it.]),
    ("Axis of revolution", [The line a revolve turns its outline around.]),
    ("Blind hole", [A hole that stops at a set depth, with a flat bottom.]),
    ("Bounding box", [The smallest box, with sides parallel to the axes, that holds a solid.]),
    ("Body", [One separate piece of solid. A shape can hold several bodies that don't touch.]),
    ("Multi-body part", [A part made of more than one body, such as a plate cut right across into two pieces.]),
    ("Midplane", [The work plane halfway between two parallel planes.]),
    ("Tangent plane", [A work plane that touches a round face along one straight line without cutting into it.]),
    ("Edge midpoint", [The work point halfway along an edge, measured along the edge.]),
    ("Boolean operation", [Combining two solids by union, cut or intersect.]),
    ("CAD", [Computer-aided design: using a computer to design physical objects.]),
    ("Between faces", [An extrude extent that runs from one flat face of the target body to another.]),
    ("Cap", [One of the two flat faces at the ends of an extrude: where the push starts and where it ends.]),
    ("Chamfer", [A flat, angled face cut along an edge in place of the sharp edge. It is set back the same distance on both faces, or by two distances, or by a distance and an angle.]),
    ("Coincident", [A constraint that puts two points at the same place.]),
    ("Conflicting", [Said of a sketch whose rules contradict each other, so no shape can meet them all.]),
    ("Constraint", [A rule about a sketch's shape, such as "horizontal" or "these points touch".]),
    ("Counterbore", [A wider, flat-bottomed recess at the mouth of a hole, so a screw head sits flush or below the surface.]),
    ("Countersink", [A cone-shaped recess at the mouth of a hole, so a screw with a cone-shaped head sits flush with the surface.]),
    ("Pitch", [How far a helix rises in one full turn; for a screw thread, the distance from one ridge to the next.]),
    ("Helix", [A curve that circles an axis at a fixed distance while rising the same amount each turn, like the handrail of a spiral staircase.]),
    ("Helical sweep", [A sweep whose path is a helix, for springs, coils and threads.]),
    ("Attachment", [A file made elsewhere, such as a STEP file, kept byte for byte inside a Saulitwerx document, in its `attachments/` folder.]),
    ("Dumb solid", [A solid with no history of its own, such as one brought in from a STEP file. Later features can build on it, but its own sizes can't be changed.]),
    ("Cache index", [In a document's cache, the file `cache/index.json` that lists every cache file with its own hash and the model hash it was built from.]),
    ("Document file", [A Saulitwerx file that keeps a design with everything needed to edit it: a part (`.sltwx`), an assembly (`.sltwxa`) or a drawing (`.sltwxd`).]),
    ("JSON", [A plain-text way of writing down named values, readable by people and programs. Saulitwerx document files are written in it.]),
    ("Manifest", [The small file in a Saulitwerx document, `manifest.json`, that says what the document is, which format version wrote it, its id and its units.]),
    ("ID", [In a part's `model.json`, the short text name, such as `sketch-1`, that a sketch, feature, body, sketch piece or rule keeps for as long as it exists. No two in the same list are alike.]),
    ("Set", [A list where only what is in it counts, not the order. Saulitwerx writes a part's sets sorted by ID.]),
    ("sltwx command", [The command that turns a Saulitwerx document folder into one file (`pack`) and back (`unpack`).]),
    ("Deflection", [For a mesh, the farthest any triangle may stray from the true surface, in mm. Smaller gives a smoother mesh with more triangles.]),
    ("Cache", [Results worked out from a document's model and stored with it, such as each body's solid and a preview, so the part shows at once. It can always be rebuilt.]),
    ("Hash", [A short code worked out from every byte of a file. Change one byte and the hash changes completely, so equal hashes mean the same file.]),
    ("SHA-256", [A published standard hash, 64 hexadecimal digits long. Saulitwerx writes it as `sha256:` and the digits.]),
    ("Stale", [Said of a cache built from an older version of the model, whose hash no longer matches. Ignore it and rebuild.]),
    ("BRep", [Boundary representation: a solid described by its faces, edges and corners and the exact surfaces and curves they lie on. Also the text file OpenCascade keeps one in.]),
    ("Mesh", [A surface broken into many small flat triangles, as 3D printers and many viewers want it.]),
    ("STL", [A common file format for a mesh, listing each triangle's corners and normal. It comes as ASCII text or as binary.]),
    ("Normal", [The direction straight out of a plane or a face, at right angles to it. Each triangle in an STL file has one.]),
    ("glTF", [An open standard file format for 3D scenes and meshes, made to load quickly in viewers and web pages. It measures in meters.]),
    ("GLB", [The one-file, binary form of glTF: a header, a JSON chunk describing the scene, and a binary chunk with the mesh's numbers.]),
    ("Chunk", [One labelled block of a file, such as the JSON or the binary part of a GLB file.]),
    ("JSON Schema", [A standard way of writing down, in JSON, which fields a JSON file must or may have and what values they can take, so a program can check a file against it.]),
    ("E notation", [A way of writing numbers with a power of ten: `1e1` is 1 × 10¹, which is 10, and `2.5e-1` is 0.25.]),
    ("Packed form", [A Saulitwerx document kept as one file, a ZIP archive of its files, rather than as a folder.]),
    ("ZIP", [A standard way of squeezing several files into one smaller file, called an archive, that almost any computer can open.]),
    ("CRC-32", [A short number worked out from a file's bytes and stored with it. Working it out again on reading shows whether any byte has changed.]),
    ("Unpacked form", [A Saulitwerx document kept as a folder of files, named like the document, rather than as one file.]),
    ("UUID", [A 32-digit hexadecimal id, written in groups of 8, 4, 4, 4 and 12, made so that no two are ever the same.]),
    ("Tap drill", [The drill a hole is made with before a thread is cut in it: narrower than the screw, so there is material for the thread.]),
    ("Tapped hole", [A hole with a screw thread cut inside it, or drilled ready for one.]),
    ("Thread", [The spiral ridge on a screw, or the matching groove inside a hole. Metric threads are named M and the diameter in mm, such as M6.]),
    ("Cut", [A Boolean operation that removes one solid from another.]),
    ("Degree of freedom (DOF)", [One way a piece of a sketch can still move. A free point has two.]),
    ("Dependency order", [The order formulas are worked out in: each parameter after the parameters its formula uses.]),
    ("Diameter", [The width of a circle through its center; twice the radius.]),
    ("Dimension", [A constraint with a number: a distance, radius or diameter.]),
    ("Dirty", [Said of a feature that must be rebuilt after a change.]),
    ("Draft", [A small tilt given to the sides of a molded or cast part so it comes out of the mold.]),
    ("Edge", [A curve where two faces of a solid meet, or where one round face meets itself (a seam).]),
    ("Equal", [A constraint that makes two lines the same length, or two circles or arcs the same size.]),
    ("Entity", [One piece of sketch geometry: a point, line, circle or arc.]),
    ("Extent", [How far an extrude goes: a distance one way, a length split evenly on both sides of the workplane, through all, or up to a face.]),
    ("Extrude", [Pushing a flat outline straight out into a solid with straight sides.]),
    ("Extrude mode", [What an extrude does with its shape: keep it as a new solid, or join, cut or intersect it with a target body.]),
    ("Face", [One flat or curved patch of a solid's surface.]),
    ("Expression", [A small formula with units that works out a value, such as `d1 / 2 + 5 mm`.]),
    ("Feature", [One step in building a part, such as a sketch or a cut. Features are kept in the history.]),
    ("Feature tree", [Another name for a part's history.]),
    ("End-of-part marker", [The point in a part's history after which features are set aside (rolled back). New features go in at the marker.]),
    ("Hole", [A feature that drills a round hole into a solid from a start point, in a direction, to a depth or through all.]),
    ("Loft", [A feature that makes a solid through two or more flat sections, joining them in order, ruled or smooth.]),
    ("Path", [The line a sweep moves its profile along: straight lines and arcs joined end to end.]),
    ("Ruled", [Said of a loft joined by straight surfaces from each section to the next, with a crease at each middle section.]),
    ("Section", [One of the flat outlines a loft passes through, like a slice through the finished part.]),
    ("Smooth loft", [A loft with one curved surface through all its sections, with no creases.]),
    ("Sweep", [A feature that moves a flat profile along a path, leaving a solid behind it.]),
    ("Neutral plane", [The plane a draft turns faces about. Where a drafted face crosses it, the face stays put.]),
    ("Open face", [A face that a shell removes, leaving an opening in the hollowed part.]),
    ("Part origin", [The point (0, 0, 0) of a part, where its three origin planes and axes meet.]),
    ("Pull direction", [The way a part leaves its mold. Draft makes the part smaller along it.]),
    ("Rolling back", [Moving the end-of-part marker up the history, so the features below it are kept but not used.]),
    ("Shell", [A feature that hollows a solid, leaving walls of one thickness and openings where its open faces were.]),
    ("Fillet", [A rounded face along an edge in place of the sharp edge, with a set radius.]),
    ("Fixed", [A constraint that keeps a point at exact coordinates.]),
    ("Fully constrained", [A sketch whose constraints and dimensions leave nothing free to move.]),
    ("From–to", [An extrude extent given by a start and an end distance along the workplane's normal.]),
    ("Geometry kernel", [The part of a CAD program that does the 3D math. Saulitwerx uses OpenCascade.]),
    ("History", [A part's features, in the order they were made.]),
    ("Inference", [Saulitwerx adding the constraints a newly drawn piece already almost satisfies, such as coincident or horizontal.]),
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
    ("Reference face", [For an uneven chamfer, the one of an edge's two faces that its first distance, or its distance and angle, is measured on.]),
    ("Region", [A flat area enclosed by an outer loop, minus any holes inside it.]),
    ("Revolve", [Turning a flat outline around an axis into a round solid.]),
    ("Revolve angle", [How far a revolve turns: a full turn (2π radians, 360°) or part of one.]),
    ("Seam", [An edge where a closed round face, such as the wall of a hole, meets itself.]),
    ("Side face", [A face of an extrude swept by one edge of its outline: a flat wall from a line, a round one from an arc or circle.]),
    ("Simple hole", [A plain round hole of one diameter.]),
    ("Sketch", [A 2D drawing on a plane, made of points, lines, circles and arcs.]),
    ("Split face", [A face that an operation cut into several faces of the result, such as a top divided by a groove.]),
    ("Solid", [A closed 3D shape with an inside and an outside.]),
    ("Solver", [The calculation that moves sketch geometry until every constraint holds.]),
    ("Sub-shape", [One face, edge or vertex of a solid.]),
    ("Symmetric", [A constraint that makes two points mirror images across a line.]),
    ("Mirror", [The reflection of a body across a flat plane, as in a looking-glass; or the operation that makes it.]),
    ("Mirror plane", [The flat plane a mirror reflects a body across. Points on it stay where they are.]),
    ("Pattern", [Evenly spaced copies of a body, made in one step.]),
    ("Rectangular pattern", [A pattern in a row, or in rows and columns, each copy a set spacing from the last.]),
    ("Circular pattern", [A pattern turned about an axis, spread over a full turn or a set angle.]),
    ("Path pattern", [A pattern spread at equal distances along a path of lines and arcs, either keeping the body's direction or turning with the path.]),
    ("Split", [Cutting one body into pieces along a flat plane, with the pieces on each side given back separately.]),
    ("Split plane", [The flat plane a split cuts along. It has no edges: it reaches as far as the body does.]),
    ("Thumbnail", [The small preview picture of a part kept in its document, `thumbnail.png`.]),
    ("STEP", [An international standard file format (ISO 10303) for exchanging 3D parts.]),
    ("Surface area", [The total area of a solid's faces.]),
    ("Tangent", [Touching without cutting in: a line and a circle, or two circles, that meet at one point.]),
    ("Target body", [The solid an extrude or a revolve joins to, cuts from or intersects with.]),
    ("Through hole", [A hole that goes all the way through a solid.]),
    ("To face", [An extrude extent that ends on the plane of a flat face of the target body.]),
    ("Through all", [An extrude cut with no depth of its own, which goes right through the target body.]),
    ("Under-constrained", [Said of a sketch with at least one degree of freedom left.]),
    ("Union", [A Boolean operation that joins two solids into one.]),
    ("Unit", [What a number measures in, such as millimeters.]),
    ("User parameter", [A parameter you create yourself, with a name you choose.]),
    ("Variable fillet", [A fillet whose radius changes evenly from one end of its edge to the other.]),
    ("Vertex", [A corner point of a solid, where its edges meet. Plural: vertices.]),
    ("Volume", [How much space a solid fills.]),
    ("Work axis", [A straight line built to place features, such as an axis to revolve about. It has no material.]),
    ("Work feature", [Reference geometry built to place other features: a work plane, a work axis or a work point. It adds no material.]),
    ("Work plane", [A flat plane built to sketch on or place features from, such as one offset from a face or tilted at an angle. Used as a workplane.]),
    ("Work point", [One point in space, built to place features, such as where an axis crosses a plane.]),
    ("Workplane", [The flat plane an outline is drawn on, with its own directions and a normal pointing straight out of it.]),
  )
  set par(first-line-indent: 0pt, justify: false)
  // Printed in alphabetical order, whatever the order above.
  columns(2, gutter: 18pt, for (t, d) in g.sorted(key: e => lower(e.at(0))) {
    block(spacing: 0.45em, breakable: false, [#text(font: sans, weight: "semibold", size: 9pt, t) \ #text(size: 9.2pt, d)])
  })
}

= What's new

== 2026-11-26: patterns along a path, and splitting a body

- *New for developers:* copy a body along a path of lines and arcs, either keeping its direction or turning with the path, such as a row of slots round a bend (@sec-pattern-path).
- *New for developers:* split a body in two with a plane, such as a clamp block into its jaws, or divide its faces along a plane without cutting it (@sec-split).
- Chapter 15 now covers splitting too, and is renamed to match (@ch-pattern). Reference: `pattern_path`, `pattern_path_aligned`, `split`, `split_faces`, and two new samples. Help: the path-pattern and split messages, and the known limitations updated. Glossary: path pattern, split, split plane.

== 2026-11-25: mirrors and patterns, and a part's cache from its bodies

- *New for developers:* mirror a body across a plane, and copy it in rows and columns or round an axis. A new chapter shows a bracket made from half its shape, a grid of holes and a bolt circle, each from one pin (@ch-pattern).
- *New for developers:* build a part's cache in one call: each body's exact shape and one display mesh for them all, and load a body back while it is still up to date (@sec-files-part-cache).
- *New:* `swx-qa thumbnail` gives a part its preview picture, in a folder or in one file (@sec-files-thumbnail).
- Reference: `swx-qa thumbnail` (@sec-ref-qa-thumbnail), `mirror`, `pattern_rectangular`, `pattern_circular`, `build_part_cache`, `load_body`, `DISPLAY_DEFLECTION_MM`, `PartCacheError`, and two new samples. Help: the pattern, part-cache and thumbnail messages, and the known limitations updated. Glossary: mirror, mirror plane, pattern, rectangular pattern, circular pattern and thumbnail.

== 2026-11-24: STEP files inside a part, and a document's cache

- *New for developers:* bring a STEP file into a part as an imported feature. The file is kept inside the document, so the part works wherever the original goes, and the solid is read back from that copy (@sec-files-import).
- *New for developers:* keep a cache in a document: stored results stamped with the model hash, handed out only while they still match the model, and set aside without an error when they are damaged (@sec-files-cache).
- *New for developers:* a part's `model.json` has a published JSON Schema (@sec-files-rules).
- *Changed:* reading and writing a document now keeps its embedded files and its cache (@sec-files-rules). How a STEP file's length unit is read is now described (@sec-files-import).
- Reference: `embed_step`, `rebuild_imported`, `fresh_cache_file`, `Cache`, `CacheEntry`, `ImportStepFeature`, the cache rules, and two new samples. Help: the STEP-import and cache messages, and the known limitations updated. Glossary: attachment, cache index and dumb solid.

== 2026-11-23: save a part as STL, and the model hash

- *New:* save a solid as an STL file in one step, binary or ASCII, with the deflection you choose, 0.01 mm unless you say otherwise (@sec-stl-export). A new picture compares a coarse and a fine mesh.
- *New for developers:* record the hash of a part's model in its manifest, and tell whether results built from the model are still up to date (@sec-files-hash).
- *New for developers:* keep a solid's exact shape as BRep text and read it back unchanged (@sec-files-brep).
- *Changed:* writing your own mesh as STL is now a section for developers (@sec-stl).
- Reference: `export_stl` and its options, `model_hash`, `sha256`, `refresh_model_hash`, `cache_is_fresh`, `write_brep`, `read_brep`, the BRep format row, and four new samples. Help: the deflection, tessellation and BRep messages, the 3D-printing answer, and the known limitations updated. Glossary: BRep, cache, deflection, hash, SHA-256 and stale.

== 2026-11-22: glTF files and the manifest schema

- *New for developers:* write any triangle mesh as a binary glTF file (GLB), at its true size, for 3D viewers and web pages (@sec-glb).
- *New for developers:* the rules of `manifest.json` are published as a JSON Schema, so other programs can check a manifest (@sec-files-rules).
- Reference: glTF files (@sec-ref-glb), the GLB format row, and a new sample. Help: the glTF messages, and the known limitations updated. Glossary: chunk, GLB, glTF and JSON Schema.

== 2026-11-21: the `sltwx` command, IDs in a part, and STL files

- *New:* the `sltwx` command packs a document folder into one file and unpacks a file into a folder, never overwriting anything (@sec-files-sltwx).
- *New:* a part's sketches, features and bodies, and each sketch's pieces and rules, must carry their own ID; files are written with these lists sorted by ID, the features kept in history order (@sec-files-ids).
- *New for developers:* write any triangle mesh as an ASCII or a binary STL file (@sec-stl).
- Reference: `sltwx` (@sec-ref-sltwx), the ID rules and the `InvalidId` and `DuplicateId` errors, STL files (@sec-ref-stl), and three samples. Help: the ID, `sltwx` and STL messages, a new question, new common mistakes, and the known limitations updated. Glossary: seven new terms, from e notation to STL.

== 2026-11-20: clearer file-name messages

- *Changed:* when a document kept as one file has the wrong extension, or none, the message now says _file_ instead of _folder_ (@sec-files-mistakes). Messages about folders are unchanged.
- Reference: `DocumentForm`, and the `form` that `ExtensionMismatch` now holds. Help: the two file messages are listed on their own.

== 2026-11-17: a document in one file

- *New:* write and read a Saulitwerx document as one file, a ZIP of the same files, the same bytes every time; turn a folder into a file and back (@sec-files-packed).
- *Changed:* chapter 18 now explains both forms, the folder and the file. Reference: `write_sltwx`, `read_sltwx` and the three new error kinds. Help: the archive error messages, new common mistakes, and the known limitations updated. Glossary: CRC-32, packed form and ZIP.

== 2026-11-13: plates with rounded corners

- *New task:* draw a plate with four rounded corners, set every corner with one radius, and change it without changing the plate's size (@sec-rounded).
- *New task:* extrude the rounded plate with a hole, and compare it with the sharp-cornered plate (@sec-extrude-rounded).
- *Pictures:* the rounded plate as drawn and solved, with 5 mm and 8 mm corners, and the extruded plate.

== 2026-11-12: springs and coils, and Saulitwerx document files

- *New:* sweep a flat profile around a helix, by pitch and number of turns, for coil springs, coils and thread-like ridges (@sec-helix).
- *New:* write and read Saulitwerx document folders (`.sltwx`), with a checked manifest and files written the same way every time (@ch-files). The design itself can't be stored in them yet.
- *Changed:* hole errors now print the countersink depth without stray digits (`2.8`, not `2.8000000000000003`), and a bad countersink angle in degrees as well as radians (@sec-hole-mistakes).
- *Pictures:* a coil spring and a square ridge wound around a helix.
- New chapter 18, Saulitwerx document files; Reference and Help are now chapters 19 and 20. Reference: `helical_sweep`, and the document file calls and manifest fields. Help: the helical sweep's and the document files' error messages, new common mistakes, and the known limitations updated. Glossary: seven new terms, from document file to UUID.

== 2026-11-11: countersunk and tapped holes, and a thread table

- *New:* drill a countersunk hole for a flat-head screw, and a tapped hole at the right drill size for a metric screw thread, blind or through all (@sec-hole-tapped).
- *New:* look up metric coarse thread sizes from M2 to M24 by name, with their pitch, minor diameter and tap drill (@sec-ref-threads).
- *Pictures:* a plate with a countersunk hole and two tapped holes.
- Reference: the new hole shapes and the thread table. Help: six new error messages, new common mistakes, and the known limitations updated. Glossary: five new terms, from countersink to thread.

== 2026-11-10: several bodies, midplanes and tangent planes, safer sweeps

- *New:* count the separate bodies in a part, take each one on its own, and join, cut or intersect a part with a whole list of tools in one call, with the history of every face (@sec-bodies).
- *New:* a work plane halfway between two parallel planes, a work plane touching a round face, and a work point halfway along an edge (@sec-work-between).
- *Changed:* a sweep now refuses a bend tighter than its profile, with an error naming the bend, instead of returning a part that passes through itself (@ch-sweep).
- *Pictures:* a plate drilled in one call and split into two bodies; a shaft with a cross hole on its midplane and a flat cut from a tangent plane.
- Reference: the new calls, `CombineHistory`, and the sweep's bend rule. Help: ten new error messages, new common mistakes, and the known limitations updated. Glossary: five new terms, from body to tangent plane.

== 2026-11-09: loft, sweep, and work features from the part

- *New:* blend a solid through several flat sections, ruled or smooth, such as a square into a circle (@ch-loft).
- *New:* move a flat outline along a path of lines and arcs, for bent rods, bars and pipes (@ch-sweep).
- *New:* take a work plane from a flat face, a work axis from a straight edge or a round face, and a work point from a corner of a part (@sec-work-part).
- *Pictures:* a square-to-round transition and a vase lofted ruled and smooth; a rod and a bar swept along one bent path; a block with a post on a face's plane and a hole down its boss's axis.
- Reference: the new calls and samples. Help: thirteen new error messages, new common mistakes, and the known limitations updated. Glossary: six new terms, from loft to sweep. Chapters 13 to 19 are renumbered: Work planes, axes and points is now @ch-work.

== 2026-11-08: holes, draft, work features, and a fillet that changes size

- *New:* drill simple and counterbored holes, blind or through all, in any direction (@ch-hole).
- *New:* tilt the sides of a part with draft, so it comes out of a mold (@ch-draft).
- *New:* build work planes, axes and points from the part origin and from each other, and place features on them (@ch-work).
- *New:* round one edge with a radius that changes from one end to the other (@sec-fillet-variable).
- *Pictures:* a plate with four holes; a block drafted three ways; a post leaning on a tilted work plane; a constant and a variable fillet side by side.
- Reference: the new calls and types, and a work-features section (@sec-ref-work). Help: sixteen new error messages, new common mistakes, and the known limitations updated. Glossary: fourteen new terms, from blind hole to work point.

== 2026-11-07: shells, and rolling back the history

- *New:* hollow a solid into walls of one thickness, open at the faces you choose: a tray, a frame or a cup in one call (@ch-shell).
- *New:* move the history's end-of-part marker to roll a part back, insert features in the middle of its history, and roll forward again (@sec-roll-back). A new reference section lists the history calls (@sec-ref-history).
- *Pictures:* a block shelled into a tray and a frame, and a cylinder into a cup.
- Help: three new error messages, seven new common mistakes, and the known limitations updated. Glossary: end-of-part marker, open face, rolling back, shell.

== 2026-11-06: uneven chamfers, and faces through a Boolean

- *New:* chamfer an edge by two different distances, or by a distance and an angle, measured from a face you choose (@sec-chamfer-sizes).
- *New:* combine two solids with history, and find out which faces of the result each input face became: kept, split or deleted (@sec-boolean-history).
- *Pictures:* one edge chamfered four ways; a grooved block with its faces named.
- Help: four new error messages, four new common mistakes, and the known limitations updated. Glossary: reference face, split face.

== 2026-11-05: up to a face, fillets and chamfers

- *New:* extrude up to a flat face of the part, or between two of its faces, instead of by a distance; and give an extrude a start and an end distance (@sec-extrude-to-face).
- *New:* round edges with a fillet, or bevel them with a chamfer, in a new chapter (@ch-fillet).
- *New:* ask for the plane of a flat face (@sec-ref-kernel).
- *Pictures:* a stepped block extruded up to its faces; a block with filleted and chamfered edges.
- Help: thirteen new error messages, eight new common mistakes, and the known limitations updated. Glossary: between faces, chamfer, fillet, from–to, to face.

== 2026-11-04: which face is which

- *New:* extrude with history, and find out which face of the solid each piece of your outline became: the two caps and every side face (@sec-extrude-history, @sec-ref-extrude).
- *New:* ask for the bounding box of a single face, edge or vertex (@sec-ref-kernel).
- *Pictures:* the extruded plate, with its faces named by their history.
- Help: two new common mistakes; two error messages and a known limitation updated. Glossary: cap, side face.

== 2026-11-03: safer revolves

- *New:* a revolve now checks its axis first. An axis that runs through the outline, or doesn't lie in the outline's plane, gets a clear message saying which, instead of a broken solid or a vague failure (@sec-revolve-mistakes).
- The revolve sample shows both new errors (@sec-revolve).
- Help: three new error messages; the revolve rows and the known limitation about unchecked axes updated.

== 2026-11-02: faces, edges and vertices

- *New:* count a solid's faces, edges and vertices, and list which ones touch: the edges around a face, the faces along an edge (@sec-topology, @sec-ref-kernel).
- Concepts: vertices join faces and edges in @sec-solid, with the plate's figure now marking one of each.
- Help: two new error messages; the known limitation about faces and edges updated.

== 2026-11-01: revolve, and cuts through all

- *New:* revolve a flat outline around an axis into a round solid, by a full turn or part of one, as a new solid or joined to, cut from or intersected with a part (@ch-revolve, @sec-ref-extrude).
- *New:* an extrude cut can go right through a part, one way or both, however thick the part is (@sec-through-all).
- *Pictures:* a flanged bushing at three quarters of a turn and at a full turn, the bushing with a groove, and a block with two through holes.
- Help: nine new error messages, eight new common mistakes, and known limitations updated. The limitations that there is no revolve and no "through all" are gone.

== 2026-10-31: extrude modes

- *New:* an extrude can join its shape to a part, cut it out, or keep only the overlap, in one step (@sec-extrude-modes, @sec-ref-extrude).
- *Pictures:* one block with a boss, a pocket and a trimmed shape made from it.
- Help: two new error messages, three new common mistakes, and known limitations updated. The limitation that an extrude always makes a new solid is gone.

== 2026-10-30: extrude

- *New:* extrude a flat outline into a solid. Outlines are made of lines, arcs and circles, can have holes, and can sit on any plane; the extrude goes a distance one way or splits evenly on both sides (@ch-extrude, @sec-ref-extrude).
- *Pictures:* a profile and the plate extruded from it, and an extruded slot.
- Concepts: the profile and feature sections now point to the extrude.
- Help: eleven new error messages, and known limitations updated. The limitation that nothing turns outlines into solids is gone.

== 2026-10-29: formulas in parameters, and inference

- *New:* a parameter can hold a formula, such as `d0 / 2 + 5 mm`, and follows the parameters it names. Formulas are worked out in dependency order, loops and wrong units are refused, and renaming a parameter rewrites the formulas that use it (@sec-param-formulas, @sec-ref-params).
- *New:* inference adds the coincident, horizontal, vertical, parallel, perpendicular and tangent constraints a newly drawn piece already almost satisfies (@sec-infer, @sec-ref-infer).
- *Pictures:* a rough rectangle and circle before and after inference.
- Help: new error messages for formulas and inference, four troubleshooting rows, new common mistakes, and known limitations updated. The limitation that parameters hold only plain numbers is gone.

== 2026-10-28: dimensions as parameters, expressions, and joined arcs

- *New:* constraints can name an arc's two ends (`ArcStart`, `ArcEnd`), so lines and arcs join into closed outlines. The slot task is rewritten: it now starts from a careless drawing and solves to a fully defined slot (@sec-slot, @sec-ref-sketch).
- *New:* a sketch's distance, radius and diameter dimensions can each make a model parameter, `d0`, `d1`, …, and follow it when it changes (@sec-drive-sketch).
- *New:* work out expressions with units, such as `d1 / 2 + 5 mm`, with unit checking (@sec-expressions, @sec-ref-expr).
- *Pictures:* the slot before and after solving replaces the slot with a gap.
- Help: new error messages for bound dimensions and expressions; troubleshooting and known limitations updated. The limitation on arc ends is gone.

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
  [Extrude from a sketch], [Hand a sketch's region straight to the extrude and the revolve.],
  [More part features], [Shells with walls of different thicknesses; fillets, chamfers, shells, holes, drafts, lofts, sweeps and work features kept in a part's history.],
  [Threads], [Model real screw threads, outside on a shaft or inside a hole, built on the helical sweep.],
  [Lasting references], [A feature built on a face or an edge keeps finding it when earlier features change.],
  [More patterns], [Patterns and mirrors of features as well as bodies, and patterns that skip copies.],
  [Part files], [Save a part with its full history in a document file and open it again, with its cache rebuilt from the history when it is stale.],
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
      #text(font: sans, weight: "bold", fill: accent, "DOCUMENT FILES")
      #term-block("sltwx pack <folder> <file>\nsltwx unpack <file> <folder>\nswx-qa thumbnail <part>")
      #text(font: sans, weight: "bold", fill: accent, "SAMPLES")
      #term-block("cargo run -p swx-sketch \\\n  --example plate_sketch\ncargo run -p swx-sketch \\\n  --example centered_plate -- 60 25\ncargo run -p swx-kernel --features occt \\\n  --example plate_with_hole -- plate.step\ncargo run -p swx-kernel --features occt \\\n  --example extrude_plate\ncargo run -p swx-core \\\n  --example feature_history\ncargo run -p swx-sketch \\\n  --example inferred_sketch")
      #v(-4pt) Every sample, with what it does: @sec-ref-build.

      #text(font: sans, weight: "bold", fill: accent, "UNITS")
      \ Millimeters. Angles: degrees in feature parameters; radians in sketch arcs and kernel revolves.

      #text(font: sans, weight: "bold", fill: accent, "SKETCH STATES")
      #ref-table(columns: (auto, 1fr), header: ("State", "Means"),
        [Under-constrained], [can still move: add rules],
        [Fully constrained], [done: 0 DOF left],
        [Over-constrained], [a rule repeats others],
        [Conflicting], [rules contradict],
      )
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
      #text(font: sans, weight: "bold", fill: accent, "SOLIDS")
      #ref-table(columns: (auto, 1fr), header: ("Call", "Gives"),
        [`make_box`], [box: corner + sizes],
        [`make_cylinder`], [cylinder on +Z],
        [`make_sphere`], [sphere],
        [`union` · `cut` · `intersect`], [Boolean result],
        [`make_face` · `extrude`], [outline to solid],
        [`extrude_feature`], [extrude and combine],
        [`revolve_feature`], [revolve and combine],
        [`fillet` · `chamfer`], [round / bevel edges],
        [`chamfer_asymmetric`], [bevel by 2 sizes],
        [`shell`], [hollow, open at faces],
        [`fillet_variable`], [round, 2 radii],
        [`hole`], [drill any kind of hole],
        [`metric_coarse`], [thread size by name],
        [`mirror` · `pattern_…`], [copies of a body],
        [`split` · `split_faces`], [cut / divide by a plane],
        [`draft`], [tilt faces for a mold],
        [`loft` · `sweep`], [blend / follow a path],
        [`helical_sweep`], [spring, coil],
        [`offset` · `midplane`], [work planes],
        [`work_plane_on_face`], [plane on a flat face],
        [`work_axis_on`], [axis on edge, cylinder],
        [`boolean_tracked`], [Boolean + face history],
        [`export_step` · `import_step`], [STEP file out / in],
        [`export_stl`], [STL file for printing],
      )
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
