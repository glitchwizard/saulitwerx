//! Sweeps flat profiles along a path drawn on the XZ plane: a circle 10 mm
//! across along a path that rises 30 mm, turns a quarter circle of radius
//! 20 mm, and runs 20 mm level (a bent rod), and a 12 × 6 mm rectangle along
//! the same path (a bent bar). Prints each result, shows the errors a sweep
//! can give (including a bend tighter than the profile), and writes the
//! solids to STEP files.
//!
//! Used by the user manual ("Sweeping along a path").
//!
//! Run: `cargo run -p swx-kernel --features occt --example sweep_pipe -- <dir>`

use std::f64::consts::{FRAC_PI_2, PI};
use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel, OcctShape, ProfileLoop, Segment2, Workplane};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn line(start: [f64; 2], end: [f64; 2]) -> Segment2 {
    Segment2::Line { start, end }
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Res<()> {
    let b = k.bounding_box(s)?;
    println!(
        "{label:<10} {:>8.2} mm³  {} faces  box {:.0} × {:.0} × {:.0} mm",
        k.volume(s)?,
        k.face_count(s)?,
        b.max[0] - b.min[0],
        b.max[1] - b.min[1],
        b.max[2] - b.min[2]
    );
    Ok(())
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // The profiles lie on the XY plane, centered on the origin, where the
    // path starts.
    let circle = ProfileLoop {
        segments: vec![Segment2::Circle {
            center: [0.0, 0.0],
            radius: 5.0,
        }],
    };
    let rectangle = ProfileLoop {
        segments: vec![
            line([-6.0, -3.0], [6.0, -3.0]),
            line([6.0, -3.0], [6.0, 3.0]),
            line([6.0, 3.0], [-6.0, 3.0]),
            line([-6.0, 3.0], [-6.0, -3.0]),
        ],
    };
    let round = k.make_face(&Workplane::XY, &circle, &[])?;
    let flat = k.make_face(&Workplane::XY, &rectangle, &[])?;

    // The path, on the XZ plane (u along X, v along Z): up 30 mm, a quarter
    // circle of radius 20 mm about (20, 30) that turns towards +X, then
    // 20 mm level. The arc runs counterclockwise from (20, 50) to (0, 30);
    // the sweep takes it the other way, starting where the line ends.
    let path = [
        line([0.0, 0.0], [0.0, 30.0]),
        Segment2::Arc {
            center: [20.0, 30.0],
            radius: 20.0,
            start_angle: FRAC_PI_2,
            end_angle: PI,
        },
        line([20.0, 50.0], [40.0, 50.0]),
    ];
    let xz = &Workplane::XZ;
    let rod = k.sweep(&round, xz, &path)?;
    report(&k, "rod", &rod)?;
    let bar = k.sweep(&flat, xz, &path)?;
    report(&k, "bar", &bar)?;

    // The errors: a path that starts 5 mm above the profile, a gap in the
    // path, a closed path (a circle), a profile with a hole, and a bend of
    // 4 mm radius, tighter than the rod's 5 mm radius.
    let washer = k.make_face(
        &Workplane::XY,
        &circle,
        &[ProfileLoop {
            segments: vec![Segment2::Circle {
                center: [0.0, 0.0],
                radius: 2.0,
            }],
        }],
    )?;
    let ring = [Segment2::Circle {
        center: [0.0, 20.0],
        radius: 20.0,
    }];
    let tight = [
        path[0],
        Segment2::Arc {
            center: [4.0, 30.0],
            radius: 4.0,
            start_angle: FRAC_PI_2,
            end_angle: PI,
        },
    ];
    for (label, r) in [
        (
            "starts above",
            k.sweep(&round, xz, &[line([0.0, 5.0], [0.0, 30.0])]),
        ),
        ("gap", k.sweep(&round, xz, &[path[0], path[2]])),
        ("circle path", k.sweep(&round, xz, &ring)),
        ("profile with hole", k.sweep(&washer, xz, &path)),
        ("tight bend", k.sweep(&round, xz, &tight)),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [("sweep-rod", &rod), ("sweep-bar", &bar)] {
        let file = dir.join(format!("{name}.step"));
        k.export_step(s, &file)?;
        println!("wrote {}", file.display());
    }
    Ok(())
}
