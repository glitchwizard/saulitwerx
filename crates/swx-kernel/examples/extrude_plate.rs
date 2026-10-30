//! Draws a 40 × 25 mm rectangle with a 10 mm circle inside it on the XY
//! workplane, extrudes it 5 mm, prints the solid's measurements, and writes
//! it to a STEP file.
//!
//! Used by the user manual ("Extrude a profile").
//!
//! Run: `cargo run -p swx-kernel --features occt --example extrude_plate -- extruded-plate.step`

use std::path::PathBuf;

use swx_kernel::{Extent, Kernel, OcctKernel, ProfileLoop, Segment2, Workplane};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "extruded-plate.step".into()),
    );
    let k = OcctKernel;

    // The outer loop: four lines, each starting where the last one ended.
    let corners = [[0.0, 0.0], [40.0, 0.0], [40.0, 25.0], [0.0, 25.0]];
    let outer = ProfileLoop {
        segments: (0..4)
            .map(|i| Segment2::Line {
                start: corners[i],
                end: corners[(i + 1) % 4],
            })
            .collect(),
    };
    // One hole: a full circle is a loop on its own.
    let hole = ProfileLoop {
        segments: vec![Segment2::Circle {
            center: [20.0, 12.5],
            radius: 5.0,
        }],
    };

    let face = k.make_face(&Workplane::XY, &outer, &[hole])?;
    let part = k.extrude(&face, Extent::Distance(5.0))?;

    let bb = k.bounding_box(&part)?;
    println!("volume        {:.2} mm³", k.volume(&part)?);
    println!("faces         {}", k.face_count(&part)?);
    println!(
        "bounding box  {:?} to {:?}",
        bb.min.map(|v| (v * 100.0).round() / 100.0),
        bb.max.map(|v| (v * 100.0).round() / 100.0)
    );
    k.export_step(&part, &out)?;
    println!("wrote {}", out.display());
    Ok(())
}
