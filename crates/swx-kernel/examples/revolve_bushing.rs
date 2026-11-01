//! Revolves the outline of a flanged bushing about the Z axis: a full turn,
//! then three quarters of a turn to show the cross-section, then cuts a
//! groove into the full bushing with a second revolve. Prints each result's
//! measurements, writes the solids to STEP files, and shows the errors a
//! revolve can give.
//!
//! Used by the user manual ("Revolving a profile").
//!
//! Run: `cargo run -p swx-kernel --features occt --example revolve_bushing -- <dir>`

use std::f64::consts::{PI, TAU};
use std::path::PathBuf;

use swx_kernel::{
    revolve_feature, Axis, ExtrudeMode, Kernel, OcctKernel, OcctShape, ProfileLoop, Segment2,
    Workplane,
};

/// A closed loop of lines through `points`, in order.
fn polygon(points: &[[f64; 2]]) -> ProfileLoop {
    ProfileLoop {
        segments: (0..points.len())
            .map(|i| Segment2::Line {
                start: points[i],
                end: points[(i + 1) % points.len()],
            })
            .collect(),
    }
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Result<(), Box<dyn std::error::Error>> {
    let bb = k.bounding_box(s)?;
    let r = |v: f64| (v * 100.0).round() / 100.0 + 0.0;
    println!(
        "{label:<10} {:>8.2} mm³  {:>2} faces  x from {} to {}",
        k.volume(s)?,
        k.face_count(s)?,
        r(bb.min[0]),
        r(bb.max[0])
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // The axis: the Z axis, through the origin, pointing up.
    let z_axis = Axis {
        origin: [0.0, 0.0, 0.0],
        dir: [0.0, 0.0, 1.0],
    };

    // Half the bushing's cross-section, drawn on the XZ plane (left–right is
    // X, the distance from the axis; up–down is Z). A 10 mm bore, a 20 mm
    // tube 20 mm tall, and a 40 mm flange 4 mm thick at the bottom.
    let outline = polygon(&[
        [5.0, 0.0],
        [20.0, 0.0],
        [20.0, 4.0],
        [10.0, 4.0],
        [10.0, 20.0],
        [5.0, 20.0],
    ]);
    let section = k.make_face(&Workplane::XZ, &outline, &[])?;

    // A full turn.
    let bushing = revolve_feature(&k, &section, z_axis, TAU, ExtrudeMode::NewSolid, None)?;
    report(&k, "full turn", &bushing)?;

    // Three quarters of a turn leaves a quarter open, showing the section.
    let cutaway = revolve_feature(&k, &section, z_axis, 1.5 * PI, ExtrudeMode::NewSolid, None)?;
    report(&k, "270°", &cutaway)?;

    // A groove for a sealing ring: a 2 × 2 mm square straddling the tube's
    // outer wall (x = 10), revolved a full turn and cut from the bushing. The
    // groove is 1 mm deep and 2 mm wide.
    let groove = k.make_face(
        &Workplane::XZ,
        &polygon(&[[9.0, 11.0], [11.0, 11.0], [11.0, 13.0], [9.0, 13.0]]),
        &[],
    )?;
    let grooved = revolve_feature(&k, &groove, z_axis, TAU, ExtrudeMode::Cut, Some(&bushing))?;
    report(&k, "grooved", &grooved)?;

    // The errors a revolve can give.
    let degrees = revolve_feature(&k, &section, z_axis, 360.0, ExtrudeMode::NewSolid, None);
    println!("360 -> error: {}", degrees.unwrap_err());
    let no_axis = Axis {
        dir: [0.0, 0.0, 0.0],
        ..z_axis
    };
    let flat = revolve_feature(&k, &section, no_axis, TAU, ExtrudeMode::NewSolid, None);
    println!("zero axis -> error: {}", flat.unwrap_err());
    let no_target = revolve_feature(&k, &groove, z_axis, TAU, ExtrudeMode::Cut, None);
    println!("cut, no target -> error: {}", no_target.unwrap_err());

    for (name, s) in [
        ("bushing", &bushing),
        ("bushing-270", &cutaway),
        ("bushing-grooved", &grooved),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
