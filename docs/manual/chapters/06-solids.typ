#import "../template.typ": *

= Solids and STEP files <ch-solids>

#for-developers[This chapter calls the geometry engine, `swx-kernel`, from Rust. It needs OpenCascade and the `occt` build feature (@ch-install).]

#idx("geometry kernel")
The part of a CAD program that does the 3D math is its #term[geometry kernel]. Saulitwerx talks to its kernel through one interface, the `Kernel` trait, so the rest of the program never depends on which kernel is underneath. Today that kernel is OpenCascade, reached through `OcctKernel`.

== Make a solid and save it as STEP

#task-intro(
  goal: [Build the plate with a hole, measure it, save it as STEP and read it back.],
  needs: [Saulitwerx built with OpenCascade (@ch-install).],
  time: [5 minutes.],
)

You already ran this program in @ch-first. Here is what each part of it does. The file is `crates/swx-kernel/examples/plate_with_hole.rs`.

#steps(
  [*Make the two starting solids.* A box is given by its lowest corner and its three sizes; a cylinder stands on +Z, given by the center of its base, its radius and its height:
  ```rust
  let plate = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 5.0])?;
  let tool  = k.make_cylinder([20.0, 12.5, -1.0], 5.0, 7.0)?;
  ```],
  [*Cut one from the other.* The cylinder starts 1 mm below the plate and ends 1 mm above it, so the hole goes cleanly all the way through:
  ```rust
  let part = k.cut(&plate, &tool)?;
  ```],
  [*Measure the result* with `volume`, `surface_area`, `face_count` and `bounding_box`. The program prints them; @ch-first checks them by hand.],
  [*Save and read back.* `export_step` writes the file, in millimeters; `import_step` reads it back as one solid. Matching volume and face count show nothing was lost:
  ```rust
  k.export_step(&part, &out)?;
  let back = k.import_step(&out)?;
  ```],
)

#result[A STEP file of the plate, checked by reading it back: `wrote plate.step (read back: 4607.30 mm³, 7 faces)`.]

#tip(title: [Make the tool overlap])[When you cut a hole, make the tool solid a little longer than the part on both sides. A tool that ends exactly on a face can leave a paper-thin skin, or confuse the kernel.]

== Combine solids

The kernel has three Boolean operations (@sec-feature). Each takes two solids and returns a new one; the inputs are left unchanged.

#ref-table(
  columns: (auto, 1fr),
  header: ("Call", "Result"),
  [`k.union(&a, &b)`], [Everything in `a` or `b`.],
  [`k.cut(&a, &b)`], [`a` minus whatever `b` overlaps.],
  [`k.intersect(&a, &b)`], [Only what is in both.],
)

#caution[Sizes must be finite and greater than zero, and positions finite. A box with a zero size is refused before it reaches the kernel, with `invalid input: box dx must be finite and > 0, got 0`.]

== Open a STEP file from another program

`import_step` reads any STEP file, and `swx-qa render --step` draws one (@ch-pictures). Everything in the file comes in as one shape. Saulitwerx can't edit an imported shape yet; using it as a starting feature is on the roadmap.
