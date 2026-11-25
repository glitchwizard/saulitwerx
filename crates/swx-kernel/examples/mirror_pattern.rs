//! Mirrors and patterns bodies: mirrors half a bracket into a whole one,
//! drills a 3 × 2 grid of holes in a plate from one pin, and drills six
//! bolt holes round a flange, plus three holes spread over 90°. Shows the
//! errors the three calls give, and writes each part to a STEP file.
//!
//! Used by the user manual ("Mirroring and patterning bodies").
//!
//! Run: `cargo run -p swx-kernel --features occt --example mirror_pattern -- <dir>`

use std::f64::consts::{FRAC_PI_2, TAU};
use std::path::PathBuf;

use swx_kernel::{Axis, BooleanOp, Kernel, OcctKernel, Workplane};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // Mirror: half a bracket, a 30 × 20 × 5 mm base with an upright at its
    // left end, mirrored across the plane x = 30 and joined to itself.
    let base = k.make_box([0.0, 0.0, 0.0], [30.0, 20.0, 5.0])?;
    let upright = k.make_box([0.0, 0.0, 5.0], [5.0, 20.0, 20.0])?;
    let half = k.union(&base, &upright)?;
    let plane = Workplane {
        origin: [30.0, 0.0, 0.0],
        x_dir: [0.0, 1.0, 0.0],
        normal: [1.0, 0.0, 0.0],
    };
    let other = k.mirror(&half, &plane)?;
    let b = k.bounding_box(&other)?;
    println!(
        "half {:.2} mm³; mirror image {:.2} mm³, x {}..{}",
        k.volume(&half)?,
        k.volume(&other)?,
        b.min[0].round(),
        b.max[0].round()
    );
    let bracket = k.union(&half, &other)?;
    println!(
        "bracket {:.2} mm³, {} body",
        k.volume(&bracket)?,
        k.body_count(&bracket)?
    );

    // Rectangular pattern: one pin 6 mm across, copied 3 times along X
    // every 20 mm and twice along Y every 20 mm, cut from a plate.
    let plate = k.make_box([0.0, 0.0, 0.0], [60.0, 40.0, 5.0])?;
    let pin = k.make_cylinder([10.0, 10.0, -1.0], 3.0, 7.0)?;
    let pins = k.pattern_rectangular(
        &pin,
        [1.0, 0.0, 0.0],
        3,
        20.0,
        Some(([0.0, 1.0, 0.0], 2, 20.0)),
    )?;
    let centres: Vec<String> = pins
        .iter()
        .map(|p| {
            let b = k.bounding_box(p).unwrap();
            format!(
                "({},{})",
                ((b.min[0] + b.max[0]) / 2.0).round(),
                ((b.min[1] + b.max[1]) / 2.0).round()
            )
        })
        .collect();
    println!("grid: {} copies at {}", pins.len(), centres.join(" "));
    let grid = k.combine(BooleanOp::Cut, &plate, &pins)?;
    println!(
        "grid plate {:.2} mm³, {} faces",
        k.volume(&grid)?,
        k.face_count(&grid)?
    );

    // Circular pattern: a flange 80 mm across, with six holes 8 mm across
    // on a 60 mm circle, a full turn about the Z axis.
    let flange = k.make_cylinder([0.0, 0.0, 0.0], 40.0, 8.0)?;
    let bolt = k.make_cylinder([30.0, 0.0, -1.0], 4.0, 10.0)?;
    let z = Axis {
        origin: [0.0, 0.0, 0.0],
        dir: [0.0, 0.0, 1.0],
    };
    let bolts = k.pattern_circular(&bolt, z, 6, TAU)?;
    println!(
        "full turn: {} copies, every {:.0}°",
        bolts.len(),
        360.0 / 6.0
    );
    let ring = k.combine(BooleanOp::Cut, &flange, &bolts)?;
    println!(
        "flange {:.2} mm³, {} faces",
        k.volume(&ring)?,
        k.face_count(&ring)?
    );

    // Part of a turn: three copies over 90°, so the last lands on 90°.
    let arc = k.pattern_circular(&bolt, z, 3, FRAC_PI_2)?;
    let angles: Vec<String> = arc
        .iter()
        .map(|p| {
            let b = k.bounding_box(p).unwrap();
            let (x, y) = ((b.min[0] + b.max[0]) / 2.0, (b.min[1] + b.max[1]) / 2.0);
            format!("{:.0}°", y.atan2(x).to_degrees())
        })
        .collect();
    println!("quarter turn: {} copies at {}", arc.len(), angles.join(" "));
    let quarter = k.combine(BooleanOp::Cut, &flange, &arc)?;

    // The errors.
    let flat = Workplane {
        normal: [0.0, 0.0, 0.0],
        ..plane
    };
    let errors = [
        ("zero normal", k.mirror(&half, &flat).map(|_| ())),
        (
            "count 0",
            k.pattern_rectangular(&pin, [1.0, 0.0, 0.0], 0, 20.0, None)
                .map(|_| ()),
        ),
        (
            "zero direction",
            k.pattern_rectangular(&pin, [0.0, 0.0, 0.0], 3, 20.0, None)
                .map(|_| ()),
        ),
        (
            "spacing NaN",
            k.pattern_rectangular(&pin, [1.0, 0.0, 0.0], 3, f64::NAN, None)
                .map(|_| ()),
        ),
        (
            "angle 400°",
            k.pattern_circular(&bolt, z, 6, 400f64.to_radians())
                .map(|_| ()),
        ),
    ];
    for (label, r) in errors {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, shape) in [
        ("mirror-half", &half),
        ("mirror-bracket", &bracket),
        ("pattern-grid", &grid),
        ("pattern-flange", &ring),
        ("pattern-quarter", &quarter),
    ] {
        let file = dir.join(format!("{name}.step"));
        k.export_step(shape, &file)?;
        println!("wrote {}", file.display());
    }
    Ok(())
}
