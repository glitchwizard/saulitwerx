//! Drills holes in a 60 × 40 × 10 mm plate: a simple hole through all, a
//! blind hole, a counterbored hole for a screw head, and a hole drilled into
//! the side. Prints the volume after each hole, shows the errors a hole can
//! give, and writes the results to STEP files.
//!
//! Used by the user manual ("Making holes").
//!
//! Run: `cargo run -p swx-kernel --features occt --example hole_plate -- <dir>`

use std::path::PathBuf;

use swx_kernel::{Axis, HoleDepth, HoleKind, Kernel, OcctKernel, OcctShape};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Straight down into the plate's top face (z = 10) at (`x`, `y`).
fn down_at(x: f64, y: f64) -> Axis {
    Axis {
        origin: [x, y, 10.0],
        dir: [0.0, 0.0, -1.0],
    }
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Res<()> {
    println!(
        "{label:<18} {:>8.2} mm³  {:>2} faces",
        k.volume(s)?,
        k.face_count(s)?
    );
    Ok(())
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let plate = k.make_box([0.0, 0.0, 0.0], [60.0, 40.0, 10.0])?;
    report(&k, "plate", &plate)?;

    let through = k.hole(
        &plate,
        down_at(12.0, 20.0),
        HoleKind::Simple { diameter: 6.0 },
        HoleDepth::ThroughAll,
    )?;
    report(&k, "6 through", &through)?;
    let blind = k.hole(
        &through,
        down_at(30.0, 20.0),
        HoleKind::Simple { diameter: 8.0 },
        HoleDepth::Blind(5.0),
    )?;
    report(&k, "+ 8 blind 5", &blind)?;
    let counterbore = HoleKind::Counterbore {
        diameter: 6.0,
        cb_diameter: 11.0,
        cb_depth: 6.0,
    };
    let cbored = k.hole(
        &blind,
        down_at(48.0, 20.0),
        counterbore,
        HoleDepth::ThroughAll,
    )?;
    report(&k, "+ counterbore", &cbored)?;
    // Into the side at x = 60, pointing back along −X, at mid-height.
    let side = Axis {
        origin: [60.0, 10.0, 5.0],
        dir: [-1.0, 0.0, 0.0],
    };
    let drilled = k.hole(
        &cbored,
        side,
        HoleKind::Simple { diameter: 4.0 },
        HoleDepth::Blind(20.0),
    )?;
    report(&k, "+ 4 side, 20 deep", &drilled)?;

    // The errors: a counterbore no wider than the hole, a blind hole no
    // deeper than its counterbore, a hole that misses the plate, and a
    // diameter of zero.
    let narrow = HoleKind::Counterbore {
        diameter: 6.0,
        cb_diameter: 5.0,
        cb_depth: 6.0,
    };
    let miss = Axis {
        origin: [100.0, 20.0, 10.0],
        dir: [0.0, 0.0, -1.0],
    };
    for (label, r) in [
        (
            "cb 5 on hole 6",
            k.hole(&plate, down_at(30.0, 20.0), narrow, HoleDepth::ThroughAll),
        ),
        (
            "blind 4, cb 6",
            k.hole(
                &plate,
                down_at(30.0, 20.0),
                counterbore,
                HoleDepth::Blind(4.0),
            ),
        ),
        (
            "off the plate",
            k.hole(
                &plate,
                miss,
                HoleKind::Simple { diameter: 6.0 },
                HoleDepth::ThroughAll,
            ),
        ),
        (
            "diameter 0",
            k.hole(
                &plate,
                down_at(30.0, 20.0),
                HoleKind::Simple { diameter: 0.0 },
                HoleDepth::ThroughAll,
            ),
        ),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    let path = dir.join("hole-plate.step");
    k.export_step(&drilled, &path)?;
    println!("wrote {}", path.display());
    Ok(())
}
