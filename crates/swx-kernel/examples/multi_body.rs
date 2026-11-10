//! Works with several solid bodies: drills two holes in a 60 × 40 × 6 mm
//! plate in one combine call (with face history), then cuts a 4 mm slot
//! across the middle, which leaves two separate bodies. Counts the bodies,
//! takes each one on its own, shows the errors a combine can give, and
//! writes the split plate to a STEP file.
//!
//! Used by the user manual ("Solids and STEP files").
//!
//! Run: `cargo run -p swx-kernel --features occt --example multi_body -- <dir>`

use std::path::PathBuf;

use swx_kernel::{BooleanOp, Kernel, OcctKernel, ProfileLoop, Segment2, Workplane};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // The plate, and two pins 8 mm across that pass right through it.
    let plate = k.make_box([0.0, 0.0, 0.0], [60.0, 40.0, 6.0])?;
    let pins = [
        k.make_cylinder([12.0, 20.0, -1.0], 4.0, 8.0)?,
        k.make_cylinder([48.0, 20.0, -1.0], 4.0, 8.0)?,
    ];

    // Both holes in one call, with the history of every face.
    let (drilled, history) = k.combine_tracked(BooleanOp::Cut, &plate, &pins)?;
    println!(
        "drilled plate: {:.2} mm³, {} faces, {} body",
        k.volume(&drilled)?,
        k.face_count(&drilled)?,
        k.body_count(&drilled)?
    );
    for (face, images) in history.target.iter().enumerate() {
        let idx: Vec<usize> = images.iter().map(|s| s.index).collect();
        println!("plate face {face} -> {idx:?}");
    }
    for (tool, faces) in history.tools.iter().enumerate() {
        let idx: Vec<Vec<usize>> = faces
            .iter()
            .map(|images| images.iter().map(|s| s.index).collect())
            .collect();
        println!("pin {tool} faces -> {idx:?}");
    }

    // A slot 4 mm wide right across the middle: the plate falls in two.
    let slot = k.make_box([28.0, -1.0, -1.0], [4.0, 42.0, 8.0])?;
    let split = k.combine(BooleanOp::Cut, &drilled, &[slot])?;
    println!(
        "slotted plate: {:.2} mm³, {} bodies",
        k.volume(&split)?,
        k.body_count(&split)?
    );
    for (i, body) in k.bodies(&split)?.iter().enumerate() {
        let b = k.bounding_box(body)?;
        println!(
            "body {i}: {:.2} mm³, x {}..{}",
            k.volume(body)?,
            b.min[0].round(),
            b.max[0].round()
        );
    }

    // The errors: no tools, a tool that is a flat face, and an intersect
    // with a tool that doesn't touch the plate.
    let square = ProfileLoop {
        segments: vec![
            Segment2::Line {
                start: [0.0, 0.0],
                end: [10.0, 0.0],
            },
            Segment2::Line {
                start: [10.0, 0.0],
                end: [10.0, 10.0],
            },
            Segment2::Line {
                start: [10.0, 10.0],
                end: [0.0, 10.0],
            },
            Segment2::Line {
                start: [0.0, 10.0],
                end: [0.0, 0.0],
            },
        ],
    };
    let flat = k.make_face(&Workplane::XY, &square, &[])?;
    let far = k.make_box([100.0, 0.0, 0.0], [10.0, 10.0, 10.0])?;
    for (label, r) in [
        ("no tools", k.combine(BooleanOp::Cut, &plate, &[])),
        ("flat tool", k.combine(BooleanOp::Cut, &plate, &[flat])),
        ("far tool", k.combine(BooleanOp::Intersect, &plate, &[far])),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    let file = dir.join("two-bodies.step");
    k.export_step(&split, &file)?;
    println!("wrote {}", file.display());
    Ok(())
}
