//! Drills holes for screws in a 60 × 30 × 12 mm plate: a countersunk hole
//! for a flat-head screw, a blind tapped M6 hole and a tapped M8 hole through
//! the plate. Prints the volume after each hole, shows the errors these holes
//! can give, and writes the result to a STEP file.
//!
//! Used by the user manual ("Making holes").
//!
//! Run: `cargo run -p swx-kernel --features occt --example hole_screw -- <dir>`

use std::f64::consts::FRAC_PI_2;
use std::path::PathBuf;

use swx_kernel::{metric_coarse, Axis, HoleDepth, HoleKind, Kernel, OcctKernel, OcctShape};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Straight down into the plate's top face (z = 12) at (`x`, `y`).
fn down_at(x: f64, y: f64) -> Axis {
    Axis {
        origin: [x, y, 12.0],
        dir: [0.0, 0.0, -1.0],
    }
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Res<()> {
    println!(
        "{label:<16} {:>8.2} mm³  {:>2} faces",
        k.volume(s)?,
        k.face_count(s)?
    );
    Ok(())
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let plate = k.make_box([0.0, 0.0, 0.0], [60.0, 30.0, 12.0])?;
    report(&k, "plate", &plate)?;

    // A 90° countersink, 12 mm across at the top, on a 6.4 mm hole.
    let countersink = HoleKind::Countersink {
        diameter: 6.4,
        csk_diameter: 12.0,
        csk_angle: FRAC_PI_2,
    };
    let csk = k.hole(
        &plate,
        down_at(12.0, 15.0),
        countersink,
        HoleDepth::ThroughAll,
    )?;
    report(&k, "countersink", &csk)?;

    let m6 = metric_coarse("M6")?;
    let tapped6 = k.hole(
        &csk,
        down_at(30.0, 15.0),
        HoleKind::Tapped { thread: m6 },
        HoleDepth::Blind(10.0),
    )?;
    report(&k, "+ M6 blind 10", &tapped6)?;

    let m8 = metric_coarse("M8")?;
    let tapped8 = k.hole(
        &tapped6,
        down_at(48.0, 15.0),
        HoleKind::Tapped { thread: m8 },
        HoleDepth::ThroughAll,
    )?;
    report(&k, "+ M8 through", &tapped8)?;

    // The errors: a countersink no wider than its hole, a flat 180° cone,
    // a blind hole no deeper than its cone, and a size not in the table.
    let narrow = HoleKind::Countersink {
        diameter: 6.4,
        csk_diameter: 6.0,
        csk_angle: FRAC_PI_2,
    };
    let flat = HoleKind::Countersink {
        diameter: 6.4,
        csk_diameter: 12.0,
        csk_angle: 2.0 * FRAC_PI_2,
    };
    for (label, r) in [
        (
            "csk 6 on hole 6.4",
            k.hole(&plate, down_at(30.0, 15.0), narrow, HoleDepth::ThroughAll),
        ),
        (
            "angle 180°",
            k.hole(&plate, down_at(30.0, 15.0), flat, HoleDepth::ThroughAll),
        ),
        (
            "blind 2",
            k.hole(
                &plate,
                down_at(30.0, 15.0),
                countersink,
                HoleDepth::Blind(2.0),
            ),
        ),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }
    match metric_coarse("M7") {
        Ok(t) => println!("M7 -> {}", t.designation()),
        Err(e) => println!("M7 -> error: {e}"),
    }

    let path = dir.join("hole-screw.step");
    k.export_step(&tapped8, &path)?;
    println!("wrote {}", path.display());
    Ok(())
}
