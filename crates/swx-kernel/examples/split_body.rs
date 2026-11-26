//! Splits bodies with a plane: a clamp block into two jaws, a U-shaped
//! bracket into three pieces, and a plane that misses. Then divides the
//! clamp block's faces along a plane without cutting it, shows the errors,
//! and writes the block and its jaws to STEP files.
//!
//! Used by the user manual ("Mirroring, patterning and splitting bodies").
//!
//! Run: `cargo run -p swx-kernel --features occt --example split_body -- <dir>`

use std::path::PathBuf;

use swx_kernel::{BooleanOp, Kernel, OcctKernel, Workplane};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// The plane through `origin` with normal `normal` (the X direction is not
/// used by a split, so any value will do).
fn plane(origin: [f64; 3], normal: [f64; 3]) -> Workplane {
    Workplane {
        origin,
        x_dir: [1.0, 0.0, 0.0],
        normal,
    }
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // A clamp block, 60 × 30 × 20 mm, with a bore 20 mm across through its
    // middle, split across the bore by the plane y = 15.
    let block = k.make_box([0.0, 0.0, 0.0], [60.0, 30.0, 20.0])?;
    let bore = k.make_cylinder([30.0, 15.0, -1.0], 10.0, 22.0)?;
    let clamp = k.combine(BooleanOp::Cut, &block, &[bore])?;
    println!("clamp {:.2} mm³", k.volume(&clamp)?);
    let (back, front) = k.split(&clamp, &plane([30.0, 15.0, 0.0], [0.0, 1.0, 0.0]))?;
    for (side, bodies) in [
        ("normal side (back)", &back),
        ("other side (front)", &front),
    ] {
        let volumes: Vec<String> = bodies
            .iter()
            .map(|b| format!("{:.2} mm³", k.volume(b).unwrap()))
            .collect();
        println!("{side}: {} body, {}", bodies.len(), volumes.join(", "));
    }

    // A U-shaped bracket, cut by the plane z = 10 through both uprights.
    let base = k.make_box([0.0, 0.0, 0.0], [60.0, 20.0, 5.0])?;
    let left = k.make_box([0.0, 0.0, 5.0], [5.0, 20.0, 20.0])?;
    let right = k.make_box([55.0, 0.0, 5.0], [5.0, 20.0, 20.0])?;
    let bracket = k.combine(BooleanOp::Union, &base, &[left, right])?;
    let (above, below) = k.split(&bracket, &plane([0.0, 0.0, 10.0], [0.0, 0.0, 1.0]))?;
    println!(
        "bracket at z = 10: {} above, {} below",
        above.len(),
        below.len()
    );

    // A plane that misses the clamp: everything is on one side.
    let (above, below) = k.split(&clamp, &plane([0.0, 0.0, 50.0], [0.0, 0.0, 1.0]))?;
    println!(
        "clamp at z = 50: {} above, {} below",
        above.len(),
        below.len()
    );

    // Divide the faces of the clamp at half its height, z = 10.
    let marked = k.split_faces(&clamp, &plane([0.0, 0.0, 10.0], [0.0, 0.0, 1.0]))?;
    println!(
        "faces {} -> {}; bodies {} -> {}",
        k.face_count(&clamp)?,
        k.face_count(&marked)?,
        k.body_count(&clamp)?,
        k.body_count(&marked)?
    );
    println!(
        "volume {:.2} -> {:.2} mm³; area {:.2} -> {:.2} mm²",
        k.volume(&clamp)?,
        k.volume(&marked)?,
        k.surface_area(&clamp)?,
        k.surface_area(&marked)?
    );

    // The errors.
    let errors = [
        (
            "zero normal",
            k.split(&clamp, &plane([0.0; 3], [0.0; 3])).map(|_| ()),
        ),
        (
            "origin NaN",
            k.split_faces(&clamp, &plane([f64::NAN, 0.0, 0.0], [0.0, 0.0, 1.0]))
                .map(|_| ()),
        ),
    ];
    for (label, r) in errors {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    // For the picture, move the back jaw 10 mm away from the front one. There
    // is no move operation yet, so this takes the second copy of a pattern
    // of two.
    let moved = k.pattern_rectangular(&back[0], [0.0, 1.0, 0.0], 2, 10.0, None)?;
    let apart = k.combine(BooleanOp::Union, &front[0], &moved[1..])?;
    println!("jaws apart: {} bodies", k.body_count(&apart)?);

    for (name, shape) in [("split-clamp", &clamp), ("split-jaws", &apart)] {
        let file = dir.join(format!("{name}.step"));
        k.export_step(shape, &file)?;
        println!("wrote {}", file.display());
    }
    Ok(())
}
