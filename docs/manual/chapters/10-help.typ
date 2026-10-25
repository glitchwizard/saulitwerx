#import "../template.typ": *

= Help <ch-help>

== Troubleshooting

#ref-table(
  columns: (1fr, 1.4fr),
  header: ("Symptom", "What to do"),
  [`cargo: command not found`], [Rust isn't installed, or this terminal started before it was. Open a new terminal; if that doesn't help, repeat the Rust step in @ch-install.],
  [`could not find Cargo.toml`], [The terminal isn't in the `saulitwerx` folder. Run `cd saulitwerx` and try again.],
  [A build with `--features occt` fails with errors about `.hxx` files or `TK…` libraries], [OpenCascade isn't installed. Install the `libocct-…-dev` packages from @ch-install.],
  [`swx-qa: cannot read … STEP input needs OCCT`], [`swx-qa` was built without OpenCascade. Rebuild it: `cargo build --release -p swx-qa --features occt`.],
  [`target/release/swx-qa: No such file or directory`], [The tool isn't built yet. Run the build step in @ch-install.],
  [Rows of `*` and colored text appear when saving or reading STEP], [Normal. The geometry engine prints its own report. Saulitwerx's own lines come before and after it.],
  [The picture looks small or blurry when printed], [Use a larger `--size`: about 80 pixels per centimeter of printed width.],
  [A curved part is covered in lines], [Raise `--crease`, for example to 45.],
  [The numbers in a sketch picture are too small to read], [Use a smaller `--size`. Labels have a fixed size in pixels, so they take up more of a small picture.],
  [A sketch solves, but its sizes aren't what you expect], [Some of it is still free to move. Check the degrees of freedom (@sec-check-dof) and add the missing dimensions.],
  [A sketch looks closed, but has no region], [Some ends don't meet. The open-edge list names the pieces (@sec-profiles). Join line ends with coincident, and check that each arc is drawn with the angles that reach its neighbors.],
  [An arc's ends stay where they were drawn after solving], [Expected today: no constraint can hold an arc's ends, so the solver never changes its angles (@sec-slot).],
  [The solver stops with `Stagnated`], [Two or more rules contradict each other. Check the degrees of freedom: the report says `Conflicting` and gives the clashing equation's number.],
)

== Error messages <sec-errors>

=== From `swx-qa`

Every message starts with `swx-qa: `. No picture is written when there is an error.

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`missing command`], [Nothing after `swx-qa`. Add `render`, `sketch`, or `--help`.],
  [#raw("unknown command `x`")], [Only `render`, `sketch` and `help` exist.],
  [#raw("unknown option `x`")], [A misspelled option, or one that belongs to the other command (`--crease` and `--step` are for `render` only). The help text follows the message.],
  [#raw("`--size` needs a value")], [An option was last on the line without its value. The same goes for the other options.],
  [`missing input: give --fixture <name> or --step <file.step>`], [Say what to draw.],
  [`give exactly one of --fixture and --step`], [Two inputs were given. Keep one.],
  [`missing input: give --fixture <name>`], [`sketch` needs the name of a built-in sketch.],
  [`missing output: give -o <out.png>`], [Say where to save the picture.],
  [#raw("unknown fixture `x`; available: cube, cylinder, l-bracket")], [Use one of the listed names.],
  [#raw("unknown sketch fixture `x`; available: rectangle, triangle-345")], [Use one of the listed names.],
  [#raw("invalid --size `x`: expected <W>x<H>, e.g. 800x600")], [Write the size as two whole numbers joined by `x`, with no spaces.],
  [#raw("invalid --size `x`: width and height must be > 0")], [Neither side may be 0.],
  [`image size WxH must be between 1x1 and 8192x8192`], [Use a smaller size.],
  [#raw("invalid --crease `x`: expected degrees")], [Give a number, such as `45`.],
  [`crease angle x must be between 0 and 180 degrees`], [Use a value from 0 to 180.],
  [`cannot read f: invalid input: no such file: f`], [The STEP file isn't there. Check the name and the folder.],
  [`cannot read f: kernel failure: import_step from f failed`], [The file isn't valid STEP. The geometry engine prints the reason just above.],
  [`cannot write f: …`], [The picture can't be saved there, for example because the folder doesn't exist.],
  [`mesh is empty`, `mesh has no visible extent`], [The STEP file holds nothing that can be drawn.],
)

=== From the geometry kernel

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`invalid input: <name> must be finite and > 0, got <v>`], [A size, radius or height is zero, negative or not a number.],
  [`invalid input: <name> must have finite coordinates, got …`], [A position contains an infinite value or not-a-number.],
  [`invalid input: no such file: f`], [The STEP file to read doesn't exist.],
  [`kernel failure: <operation> failed`], [OpenCascade couldn't complete the operation. For a cut, check that the tool solid overlaps the part (@ch-solids).],
)

=== From sketches

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`geometry values must be finite`], [A coordinate, radius or angle is infinite or not a number.],
  [`radius must be > 0, got r`], [A circle or arc needs a positive radius.],
  [`value must be > 0, got v`], [A distance, radius or diameter constraint needs a positive value.],
  [`unknown geometry N`], [The constraint names an entity that isn't in this sketch.],
  [`geometry N has a different kind`], [Wrong kind of point or entity, such as `Center` of a line, a circle in parallel, or a line made equal to a circle (@ch-sketching).],
)

=== From profiles

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [`geometry A and B intersect`], [Pieces A and B cross, overlap, or touch away from their ends. Move or shorten one so they meet only end to end (@sec-profiles).],
  [`weld tolerance must be > 0, got t`], [Give a `weld_tolerance` greater than zero, or `None` for the default.],
)

Any sketch message above can also come from profiles, when the sketch itself is at fault.

=== From the document model

#ref-table(
  columns: (1fr, 1fr),
  header: ("Message", "Meaning and fix"),
  [#raw("unknown parameter `p`")], [Set the parameter before a feature uses it.],
  [`unknown feature N`], [No feature with that number is in the history.],
  [`feature N references feature M which is not earlier in the history`], [A feature can use only features made before it.],
)

== Questions

*Is there a version with windows and buttons?* Not yet. The desktop app is a later milestone; see *Coming next*.

*Can I open files from other CAD programs?* Yes, if you save them as STEP first. Saulitwerx doesn't read any program's own native file format, and won't.

*Can I save my design and edit it later?* Not yet. Today you can save solids as STEP, which keeps the shape but not the history. Saulitwerx part files come later.

*Does it cost anything?* No. Saulitwerx is free and open source, under the MIT or Apache-2.0 license, at your choice.

*Why millimeters?* Mechanical design works mostly in millimeters, so it is the default. Choosing other units per part is planned.

== Known limitations

- No desktop app, 3D view, mouse or keyboard controls yet.
- No features that turn sketches into solids yet: no extrude, revolve, fillet or hole. Solids are made from boxes, cylinders and spheres.
- No native part files; work can't be saved with its history.
- The degree-of-freedom report gives redundant and conflicting rules as equation numbers, not as constraints; your program has to match them up (@sec-check-dof).
- `swx-qa sketch` draws only its two built-in sketches. Your own sketches need a short program (@sec-picture-sketch). Sketch pictures show no dimension arrows and no marks for constraints such as horizontal.
- A constraint can't name an arc's two ends, only its center. Arcs must be drawn with the angles that reach their neighbors; the solver never changes them (@sec-slot).
- There are no angle dimensions and no point-to-line distance yet.
- Profiles don't split pieces where they cross; crossing pieces are an error.
- Only built on and tested on Ubuntu Linux.
- An imported STEP file becomes one shape, and can't be edited.
