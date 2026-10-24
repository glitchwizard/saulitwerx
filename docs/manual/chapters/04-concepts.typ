#import "../template.typ": *

= How parametric CAD works <ch-concepts>

Every parametric modeler, Saulitwerx included, is built on a handful of ideas. Once you know them, the commands make sense on their own. This chapter explains each idea in the order you meet it when you design a part. For each one it says what it is, why it matters, and where it stands in Saulitwerx today.

== Solids <sec-solid>

A #term[solid] is a closed 3D shape with an inside and an outside, like a real object. Because Saulitwerx knows what is inside, it can tell you how much material a part holds (its #term[volume]), how much surface it has to paint (its #term[surface area]), and how big a box it would fit in (its #term[bounding box]).

The outside of a solid is made of #term[faces]: flat or curved patches of surface. Where two faces meet there is an #term[edge]. The plate from @ch-first has 7 faces: top, bottom, four sides, and the round inside of the hole.

#shot(
  // swx-qa render --step plate.step --size 1600x1000 -o docs/manual/img/plate-with-hole.png
  "img/plate-with-hole.png",
  width: 62%,
  caption: [A solid. Its faces are shaded; its edges are outlined. Volume 4607.30 mm³, 7 faces.],
  alt: "The plate with a hole, used as an example of a solid",
)

*In Saulitwerx today:* the geometry engine makes boxes, cylinders and spheres, and measures volume, surface area, bounding box and face count. @ch-solids shows how.

== Units

Every length in a part has a #term[unit]. In Saulitwerx the default is the millimeter (mm), and all of today's building blocks work in millimeters: a box 40 long is 40 mm long, and volumes come out in mm³. Angles are in degrees where you type them, and in radians inside the sketch geometry.

*In Saulitwerx today:* everything is in millimeters. STEP files are written in millimeters too. Choosing units per part comes with part files.

== Sketches <sec-sketch>

Most solids start as a flat drawing. A #term[sketch] is a 2D drawing on a flat plane, made of simple pieces: points, straight lines, circles and arcs. Together these are called sketch #term[geometry] or #term[entities].

A sketch for the plate would be a rectangle (four lines) and a circle. You don't need to draw it accurately. A rough shape is fine, because the next two ideas, constraints and dimensions, make it exact.

#shot(
  // cargo run -p swx-qa --example plate_sketch_plot -- docs/manual/img 600x420
  "img/sketch-rough.png",
  width: 40%,
  caption: [A rough sketch of the plate: four lines and a circle, drawn by eye. The red dots are the ends of the lines.],
  alt: "A crooked four-sided outline with a circle inside, drawn roughly",
) <fig-sketch-rough>

*In Saulitwerx today:* sketches with points, lines, circles and arcs exist as a building block (@ch-sketching). There is no drawing window yet, so sketches are written as a short program, and `swx-qa sketch` draws them to a picture (@sec-picture-sketch).

== Constraints <sec-constraint>

A #term[constraint] is a rule about the shape of a sketch: "this line is horizontal," "these two corners touch," "this point stays at the origin." Constraints carry the *intent* of a design. If you later change a size, a line that must stay horizontal stays horizontal.

#ref-table(
  columns: (auto, 1fr),
  header: ("Constraint", "Rule it keeps"),
  [Coincident], [Two points are at the same place. Use it to join the end of one line to the start of the next.],
  [Horizontal], [A line runs left–right; or two points are at the same height.],
  [Vertical], [A line runs up–down; or two points are directly above each other.],
  [Fixed], [A point stays at exact coordinates. Use it to anchor a sketch.],
)

#idx("coincident")#idx("horizontal")#idx("vertical")#idx("fixed")
Saulitwerx satisfies constraints with a #term[solver]: a calculation that nudges every point until all the rules hold at once. You give it a rough drawing and the rules; it gives back the exact shape.

#shot(
  // cargo run -p swx-qa --example plate_sketch_plot -- docs/manual/img 600x420
  "img/sketch-shape.png",
  width: 40%,
  caption: [The same sketch after the solver applied coincident, horizontal, vertical and fixed. It is now a true rectangle, but its size is still whatever the rough drawing happened to be.],
  alt: "A clean rectangle with a circle inside, with no numbers on it",
) <fig-sketch-shape>

== Dimensions <sec-dimension>

A #term[dimension] is a constraint with a number: a length, a distance, a radius or a diameter. "The bottom line is 40 mm long." "The hole is 10 mm across." Dimensions fix the size of a sketch, the way the plain constraints above fix its shape.

#ref-table(
  columns: (auto, 1fr),
  header: ("Dimension", "Number it sets"),
  [Distance], [The straight-line distance between two points, for example the two ends of a line.],
  [Radius], [The radius of a circle or arc.],
  [Diameter], [The diameter of a circle or arc: twice the radius.],
)

#idx("distance")#idx("radius")#idx("diameter")
#shot(
  // cargo run -p swx-qa --example plate_sketch_plot -- docs/manual/img 600x420
  "img/sketch-sized.png",
  width: 40%,
  caption: [With dimensions added, the plate sketch is exactly 40 × 25 mm with a ⌀10 mm hole. Each green number is a dimension.],
  alt: "The rectangle labeled 40 along the bottom and 25 on the right, and its circle labeled with a diameter of 10",
) <fig-sketch-sized>

== Degrees of freedom <sec-dof>

Each way a piece of a sketch can still move is called a #term[degree of freedom], or DOF for short. A free point on a sheet has two (left–right and up–down). A free line has four (two for each end); a free circle has three (its center's two, and its size). Each constraint takes some away: fixed takes two from a point, horizontal takes one from a line.

Counting what is left tells you how finished a sketch is. A sketch is in one of four states: #term[under-constrained], #term[fully constrained], #term[over-constrained] or #term[conflicting].

#ref-table(
  columns: (auto, 1fr),
  header: ("State", "What it means"),
  [Under-constrained], [Something can still move. Add constraints or dimensions until the count reaches zero.],
  [Fully constrained], [Nothing can move, and no rule repeats another. This is the goal.],
  [Over-constrained], [At least one rule repeats what other rules already say, but they all agree. The repeated rule is #term[redundant]: harmless today, but it will fight you as soon as you change one of the sizes it repeats.],
  [Conflicting], [Redundant rules that disagree, such as two sides of a rectangle dimensioned 40 and 45 mm. No shape can satisfy them, so the solver stops short.],
)

The plate sketch starts with 19 degrees of freedom: four lines and a circle. @fig-sketch-shape still has 5 (the rectangle's width and height, and the circle's position and size); @fig-sketch-sized has none. @sec-check-dof shows how to get these numbers.

#tip[Constrain the shape first (coincident, horizontal, vertical), then add dimensions. Then anchor one point with fixed, so the whole sketch can't slide around.]

*In Saulitwerx today:* all of the constraints and dimensions in these tables work, and the solver reports whether it found a shape that meets them. A degree-of-freedom report gives the count and the state, and points to each redundant rule as a numbered equation (@sec-check-dof). Naming the constraint behind each equation is up to your program for now.

== Parameters <sec-parameter>

A #term[parameter] is a named number, like `thickness = 5` or `hole_dia = 10`. Features and dimensions refer to parameters by name instead of holding a bare number. Change the parameter, and everything that uses it follows.

This is the "parametric" in parametric CAD. One plate design, with its thickness as a parameter, is really a whole family of plates: set `thickness` to 3, 5 or 8 and rebuild.

*In Saulitwerx today:* parts hold named parameters, and features refer to them by name (@ch-history). Expressions such as `d3 = d1 / 2 + 5 mm`, and automatic parameters for every dimension, are on the roadmap.

== Features <sec-feature>

A #term[feature] is one step in building a part: a sketch, or something done to the solid, such as pushing a sketch out into 3D or cutting a hole. Each feature records *what* it does and *which* sketch and parameters it uses, not just the shape that resulted.

@fig-cut shows the idea of a cut, done directly with the geometry engine: a block, minus a cylinder, gives the plate. A future *extrude* feature will make the block from a rectangle sketch, and a *cut* feature will make the hole from the circle.

#image-equation(
  (
    // cargo run -p swx-kernel --features occt --example plate_stages -- <dir>
    // swx-qa render --step <dir>/plate-block.step --size 900x700 -o docs/manual/img/plate-block.png
    ("img/plate-block.png", "Block, 40 × 25 × 5 mm"),
    // swx-qa render --step <dir>/plate-tool.step --size 900x700 -o docs/manual/img/plate-tool.png
    ("img/plate-tool.png", "Cylinder, ⌀10 × 7 mm"),
    // swx-qa render --step <dir>/plate-cut.step --size 900x700 -o docs/manual/img/plate-cut.png
    ("img/plate-cut.png", "Result: the plate"),
  ),
  ("−", "="),
  caption: [A cut takes one solid away from another. Each picture is scaled to fill its frame, so the cylinder looks larger than it is.],
) <fig-cut>

#idx("cut")#idx("union")#idx("intersect")
There are three ways to combine two solids. Together they are called #term[Boolean operations]:

- *Union* (join): keep everything that is in either solid.
- *Cut* (subtract): keep the first solid, minus anything the second one overlaps.
- *Intersect*: keep only the part that is in both.

== History <sec-history>

A part's #term[history], also called its #term[feature tree], is its list of features in the order they were made. Each feature can use only what came before it: a cut needs a solid to cut into, and an extrude needs a sketch to push.

#ref-table(
  columns: (auto, auto, 1fr),
  header: ("#", "Feature", "Uses"),
  [0], [Sketch1], [nothing: a rectangle on a plane],
  [1], [Extrude1], [Sketch1, and the parameter `thickness`],
  [2], [Sketch2], [nothing: a profile for a pin],
  [3], [Revolve1], [Sketch2, and the parameter `sweep`],
)

Because the history remembers how each feature was made, Saulitwerx can *rebuild* a part after a change. It doesn't rebuild everything. It works out which features depend on what changed, and redoes only those. Change `thickness`, and only Extrude1 rebuilds; edit Sketch2, and Sketch2 and Revolve1 rebuild.

*In Saulitwerx today:* the history and the "what needs rebuilding" logic exist (@ch-history), using the exact example in the table above. Running the rebuild on real geometry comes with the part features.

== Putting it together

Here is how the ideas line up when you design a part, once the app exists:

#steps(
  [Make a *sketch* on a plane, roughly the shape you want.],
  [Add *constraints* to fix its shape, and *dimensions* to fix its size, until no degrees of freedom are left.],
  [Turn the sketch into a *solid* with a *feature*, such as extrude.],
  [Add more features (cuts, holes, rounded edges) to finish the part. They form its *history*.],
  [Change a *parameter* or a dimension later, and the part rebuilds to match.],
)
