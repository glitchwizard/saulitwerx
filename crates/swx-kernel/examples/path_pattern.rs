//! Patterns a slot along a path with a bend, twice: once with every copy
//! keeping the slot's direction, once with each copy turning to follow the
//! path. Cuts each set from a plate, shows the errors, and writes both
//! plates to STEP files.
//!
//! Used by the user manual ("Mirroring, patterning and splitting bodies").
//!
//! Run: `cargo run -p swx-kernel --features occt --example path_pattern -- <dir>`

use std::f64::consts::FRAC_PI_2;
use std::path::PathBuf;

use swx_kernel::{BooleanOp, Kernel, OcctKernel, Segment2, Workplane};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // An 80 × 80 × 5 mm plate, and a slot tool 8 mm long and 3 mm wide,
    // centred on (10, 15), the start of the path.
    let plate = k.make_box([0.0, 0.0, 0.0], [80.0, 80.0, 5.0])?;
    let slot = k.make_box([6.0, 13.5, -1.0], [8.0, 3.0, 7.0])?;

    // The path, on the XY plane: 35 mm along X, a quarter turn of 20 mm
    // radius, then 35 mm along Y.
    let path = [
        Segment2::Line {
            start: [10.0, 15.0],
            end: [45.0, 15.0],
        },
        Segment2::Arc {
            center: [45.0, 35.0],
            radius: 20.0,
            start_angle: -FRAC_PI_2,
            end_angle: 0.0,
        },
        Segment2::Line {
            start: [65.0, 35.0],
            end: [65.0, 70.0],
        },
    ];
    println!("path {:.2} mm long", 70.0 + 10.0 * std::f64::consts::PI);

    // Where each copy sits, and how much room it takes along X and Y.
    let describe = |copies: &[<OcctKernel as Kernel>::Shape]| -> Vec<String> {
        copies
            .iter()
            .map(|c| {
                let b = k.bounding_box(c).unwrap();
                format!(
                    "({:.1},{:.1}) {:.1}×{:.1}",
                    (b.min[0] + b.max[0]) / 2.0,
                    (b.min[1] + b.max[1]) / 2.0,
                    b.max[0] - b.min[0],
                    b.max[1] - b.min[1]
                )
            })
            .collect()
    };

    let straight = k.pattern_path(&slot, &Workplane::XY, &path, 7)?;
    println!("same direction: {} copies", straight.len());
    for line in describe(&straight) {
        println!("  {line}");
    }
    let plate_straight = k.combine(BooleanOp::Cut, &plate, &straight)?;
    println!("plate {:.2} mm³", k.volume(&plate_straight)?);

    let aligned = k.pattern_path_aligned(&slot, &Workplane::XY, &path, 7)?;
    println!("following the path: {} copies", aligned.len());
    for line in describe(&aligned) {
        println!("  {line}");
    }
    let plate_aligned = k.combine(BooleanOp::Cut, &plate, &aligned)?;
    println!("plate {:.2} mm³", k.volume(&plate_aligned)?);

    // The errors.
    let gap = [
        Segment2::Line {
            start: [10.0, 15.0],
            end: [45.0, 15.0],
        },
        Segment2::Line {
            start: [50.0, 15.0],
            end: [50.0, 70.0],
        },
    ];
    let circle = [Segment2::Circle {
        center: [40.0, 40.0],
        radius: 20.0,
    }];
    let errors = [
        (
            "count 0",
            k.pattern_path(&slot, &Workplane::XY, &path, 0).map(|_| ()),
        ),
        (
            "empty path",
            k.pattern_path(&slot, &Workplane::XY, &[], 3).map(|_| ()),
        ),
        (
            "gap",
            k.pattern_path(&slot, &Workplane::XY, &gap, 3).map(|_| ()),
        ),
        (
            "circle path",
            k.pattern_path_aligned(&slot, &Workplane::XY, &circle, 3)
                .map(|_| ()),
        ),
    ];
    for (label, r) in errors {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, shape) in [
        ("path-pattern-straight", &plate_straight),
        ("path-pattern-aligned", &plate_aligned),
    ] {
        let file = dir.join(format!("{name}.step"));
        k.export_step(shape, &file)?;
        println!("wrote {}", file.display());
    }
    Ok(())
}
