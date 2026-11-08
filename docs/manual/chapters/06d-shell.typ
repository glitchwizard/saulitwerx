#import "../template.typ": *

= Hollowing a part <ch-shell>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("shell")
Many parts are hollow: a box for electronics, a tray, a cup, a cover. Making them solid would waste material, add weight, and in a molded or printed part cause sink marks and warping. You could model the inside as a second, smaller shape and cut it away, but there is a quicker way.

A #term[shell] hollows a solid in one step. It leaves walls of one thickness all round and takes away everything inside. The faces you choose to remove become the openings; they are the #term(key: "open face")[open faces]. A box with its top as the open face becomes a tray. Open the top and the bottom, and it becomes a frame. Open the top of a cylinder, and it becomes a cup (@fig-shells).

The walls grow _inward_ from the faces of the solid. The outside of the part stays where it was, at the size you drew, and only the inside changes. That is usually what a design needs: the outside fits a space, and the walls are as thick as the material requires.

#image-row(
  (
    // modes-target.png: see chapter 7 (extrude_modes); the same 40 × 25 × 10 mm block
    ("img/modes-target.png", "Block: 40 × 25 × 10 mm", "A plain rectangular block"),
    // cargo run -p swx-kernel --features occt --example shell_parts -- shells
    // swx-qa render --step shells/shell-<name>.step --size 900x700 -o docs/manual/img/shell-<name>.png
    ("img/shell-tray.png", "Top open: a tray", "The block hollowed out with 2 mm walls and an open top, like a tray"),
    ("img/shell-frame.png", "Top and bottom open: a frame", "The block hollowed out with 2 mm walls, open at both top and bottom, so only a rectangular frame of four walls is left"),
    ("img/shell-cup.png", "Cylinder, top open: a cup", "A cylinder 30 mm across hollowed out with 2 mm walls and an open top, like a cup"),
  ),
  caption: [Shells with 2 mm walls. The tray and the frame are made from the same block, opened at one face and at two. The cup is a cylinder opened at its top. Each part keeps its outside size. Each picture is scaled to fill its frame.],
) <fig-shells>

*In Saulitwerx today:* the geometry engine can shell a solid with one wall thickness, open at one or more faces. You give the open faces by their numbers in the solid (@sec-topology), so your program finds them first. Every wall has the same thickness, and a shell must have at least one opening. A shell isn't recorded in a part's history yet.

== Shell a solid <sec-shell>

#task-intro(
  goal: [Hollow a block into a tray and a frame, and a cylinder into a cup, all with 2 mm walls.],
  needs: [Saulitwerx built with OpenCascade, and `swx-qa` built with it too (@ch-install).],
  time: [5 minutes.],
)

#steps(
  [#block(breakable: false)[Run the sample program, and tell it to save its parts in a folder called `shells`:
  #term-block("cargo run -p swx-kernel --features occt --example shell_parts -- shells")
  It prints the block and the faces it found, one line for each solid, then five errors on purpose, then the files it wrote:
  #term-block(label: "Output", "block            10000.00 mm³   6 faces  outside 40 × 25 × 10 mm\ntop face 5, bottom face 4\ntray, wall 2      3952.00 mm³  11 faces  outside 40 × 25 × 10 mm\nframe, wall 2     2440.00 mm³  10 faces  outside 40 × 25 × 10 mm\ncylinder         21205.75 mm³   3 faces  outside 30 × 30 × 30 mm\ncup, wall 2       6339.73 mm³   5 faces  outside 30 × 30 × 30 mm\nwall 13 -> error: kernel failure: shell failed\nno faces -> error: invalid input: the face list is empty\nan edge -> error: invalid input: expected a face, got edge 0\nface 6 -> error: invalid input: face index 6 is out of range: the shape has 6 faces\nwall 0 -> error: invalid input: shell thickness must be finite and > 0, got 0\n…  (the geometry engine's file reports)\nwrote shells/shell-tray.step\nwrote shells/shell-frame.step\nwrote shells/shell-cup.step")]],
  [Read the second line. The block's top is face 5 and its bottom is face 4.],
  [Read the `tray` line. The outside is still 40 × 25 × 10 mm. The 2 mm walls leave a hollow 36 × 21 mm across and 8 mm deep, which is 6048 mm³ taken away. The tray has 11 faces: five outside, five inside, and the rim on top.],
  [Read the `frame` line. With the bottom open too, the hollow goes all the way through, 36 × 21 × 10 mm, and only the four walls are left.],
  [Read the `cup` line. The cylinder is 30 mm across and 30 mm tall. Its walls and floor are 2 mm thick, so the hollow is 26 mm across and 28 mm deep. The cup has 5 faces: the outside, the bottom, the inside wall, the floor and the rim.],
  [Read the `wall 13` line. The block is 25 mm wide, and two walls of 13 mm would need 26 mm. The geometry engine can't build it and says so. The block is unchanged.],
  [Draw the tray:
  #term-block("target/release/swx-qa render --step shells/shell-tray.step -o shell-tray.png")
  Open `shell-tray.png`. You should see the tray of @fig-shells. Draw the other files the same way.],
)

#result[A tray, a frame and a cup, each made by one call. The solids you started from are left as they were.]

#block(breakable: false)[
=== How the sample is built

The program is `crates/swx-kernel/examples/shell_parts.rs`. It does two things.

#steps(
  [#block(breakable: false)[*Find the faces to open.* A face is a `SubShape` of kind `Face`, numbered from 0. The sample looks for the flat face whose outward normal points the way it wants, up for the top and down for the bottom. `face_plane` gives each flat face's normal (@sec-extrude-to-face), and refuses curved faces, which the sample skips:
  ```rust
  for index in 0..k.sub_shape_count(s, TopoKind::Face)? {
      let face = SubShape { kind: TopoKind::Face, index };
      if let Ok((_, n)) = k.face_plane(s, face) {
          if dot(n, normal) > 0.999_999 {
              return Ok(face);
          }
      }
  }
  ```]],
  [#block(breakable: false)[*Shell the solid.* `shell` takes the solid, a list of open faces and the wall thickness in mm:
  ```rust
  let tray = k.shell(&block, &[top], 2.0)?;
  let frame = k.shell(&block, &[top, bottom], 2.0)?;
  let cup = k.shell(&can, &[rim], 2.0)?;
  ```]],
)
]

#tip(title: [Choosing a wall thickness])[Pick the thickness your material and process need, then check that the part still has room for it: every wall comes off the inside, so a part 25 mm wide with 2 mm walls is 21 mm wide inside. Listing a face twice is harmless; it counts once.]

#caution(title: [The numbers change])[A shell makes new faces and edges, so the result is numbered afresh: the block had 6 faces, the tray has 11. To round the tray's rim or open another face later, find the faces again in the new solid.]

== Common mistakes <sec-shell-mistakes>

#set par(justify: false)

- *A wall too thick for the part.* The walls from opposite sides must not meet: `kernel failure: shell failed`. Keep the thickness under half the part's narrowest width.
- *No open face.* `invalid input: the face list is empty`. Give at least one face to open. A part hollow inside with no opening can't be made yet.
- *An edge or a vertex in the list.* Only faces can be opened: `invalid input: expected a face, got edge 0`. Ask for faces with `TopoKind::Face`, or turn an edge into its faces with `adjacent` (@sec-topology).
- *A face number from another solid.* `invalid input: face index 6 is out of range: the shape has 6 faces`, or worse, the wrong face opened. Find the faces in the solid you are shelling.
- *A thickness of zero or less.* `invalid input: shell thickness must be finite and > 0, got 0`.
