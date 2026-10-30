//! Extrudes a slot (two lines and two half-circle arcs) 4 mm three ways:
//! up, down, and split evenly about the workplane. Prints where each
//! result sits, and writes the first one to a STEP file.
//!
//! Used by the user manual ("Choose the direction and the workplane").
//!
//! Run: `cargo run -p swx-kernel --features occt --example extrude_slot -- extruded-slot.step`

use std::f64::consts::FRAC_PI_2;
use std::path::PathBuf;

use swx_kernel::{Extent, Kernel, OcctKernel, ProfileLoop, Segment2, Workplane};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "extruded-slot.step".into()),
    );
    let k = OcctKernel;

    // Arcs run counterclockwise from start_angle to end_angle (radians).
    let slot = ProfileLoop {
        segments: vec![
            Segment2::Line {
                start: [0.0, 0.0],
                end: [30.0, 0.0],
            },
            Segment2::Arc {
                center: [30.0, 5.0],
                radius: 5.0,
                start_angle: -FRAC_PI_2,
                end_angle: FRAC_PI_2,
            },
            Segment2::Line {
                start: [30.0, 10.0],
                end: [0.0, 10.0],
            },
            Segment2::Arc {
                center: [0.0, 5.0],
                radius: 5.0,
                start_angle: FRAC_PI_2,
                end_angle: 3.0 * FRAC_PI_2,
            },
        ],
    };
    let face = k.make_face(&Workplane::XY, &slot, &[])?;

    let round = |v: [f64; 3]| v.map(|c| (c * 100.0).round() / 100.0);
    let mut first = None;
    for (label, extent) in [
        ("Distance(4.0)  ", Extent::Distance(4.0)),
        ("Distance(-4.0) ", Extent::Distance(-4.0)),
        ("Symmetric(4.0) ", Extent::Symmetric(4.0)),
    ] {
        let solid = k.extrude(&face, extent)?;
        let bb = k.bounding_box(&solid)?;
        println!(
            "{label} {:.2} mm³, z from {} to {}",
            k.volume(&solid)?,
            round(bb.min)[2],
            round(bb.max)[2]
        );
        first.get_or_insert(solid);
    }
    if let Some(solid) = first {
        k.export_step(&solid, &out)?;
        println!("wrote {}", out.display());
    }
    Ok(())
}
