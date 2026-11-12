//! Sweeps flat profiles along a helix about the Z axis: a circle 2 mm across,
//! 10 mm from the axis, six turns at a 5 mm pitch (a coil spring), and a
//! 2 × 1.5 mm rectangle, 8 to 10 mm from the axis, four turns at a 3 mm
//! pitch (a square ridge that winds around like a thread). Prints each
//! result, shows the errors a helical sweep can give, and writes the solids
//! to STEP files.
//!
//! Used by the user manual ("Sweeping along a path", the helix section).
//!
//! Run: `cargo run -p swx-kernel --features occt --example helical_spring -- <dir>`

use std::path::PathBuf;

use swx_kernel::{Axis, Kernel, OcctKernel, OcctShape, ProfileLoop, Segment2, Workplane};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn line(start: [f64; 2], end: [f64; 2]) -> Segment2 {
    Segment2::Line { start, end }
}

fn circle(center: [f64; 2], radius: f64) -> ProfileLoop {
    ProfileLoop {
        segments: vec![Segment2::Circle { center, radius }],
    }
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Res<()> {
    let b = k.bounding_box(s)?;
    println!(
        "{label:<7} {:>8.2} mm³  {} faces  box {:.0} × {:.0} × {:.1} mm",
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

    // The helix turns about the Z axis through the origin.
    let z_axis = Axis {
        origin: [0.0, 0.0, 0.0],
        dir: [0.0, 0.0, 1.0],
    };

    // The profiles lie on the XZ plane (u along X, v along Z), which holds
    // the Z axis, off to one side of it.
    let xz = &Workplane::XZ;
    let wire = k.make_face(xz, &circle([10.0, 0.0], 1.0), &[])?;
    let square = ProfileLoop {
        segments: vec![
            line([8.0, 0.0], [10.0, 0.0]),
            line([10.0, 0.0], [10.0, 1.5]),
            line([10.0, 1.5], [8.0, 1.5]),
            line([8.0, 1.5], [8.0, 0.0]),
        ],
    };
    let ridge_profile = k.make_face(xz, &square, &[])?;

    // Pitch (mm per turn), then the number of turns.
    let spring = k.helical_sweep(&wire, z_axis, 5.0, 6.0)?;
    report(&k, "spring", &spring)?;
    let ridge = k.helical_sweep(&ridge_profile, z_axis, 3.0, 4.0)?;
    report(&k, "ridge", &ridge)?;

    // The errors: a zero pitch, an axis that leaves the profile's plane, a
    // profile that crosses the axis, a pitch smaller than the profile is
    // tall (the turns would overlap), and a profile with a hole.
    let y_axis = Axis {
        origin: [0.0, 0.0, 0.0],
        dir: [0.0, 1.0, 0.0],
    };
    let on_axis = k.make_face(xz, &circle([0.0, 0.0], 1.0), &[])?;
    let tube = k.make_face(xz, &circle([10.0, 0.0], 1.0), &[circle([10.0, 0.0], 0.5)])?;
    for (label, r) in [
        ("pitch 0", k.helical_sweep(&wire, z_axis, 0.0, 6.0)),
        ("axis along Y", k.helical_sweep(&wire, y_axis, 5.0, 6.0)),
        ("on the axis", k.helical_sweep(&on_axis, z_axis, 5.0, 6.0)),
        ("pitch 1.5", k.helical_sweep(&wire, z_axis, 1.5, 6.0)),
        ("tube", k.helical_sweep(&tube, z_axis, 5.0, 6.0)),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [("helix-spring", &spring), ("helix-ridge", &ridge)] {
        let file = dir.join(format!("{name}.step"));
        k.export_step(s, &file)?;
        println!("wrote {}", file.display());
    }
    Ok(())
}
