//! Extrudes a 40 × 25 mm plate with 5 mm rounded corners and a 10 mm hole in
//! its middle, 5 mm thick: the outline of `swx-sketch`'s `rounded_plate`
//! example, typed in as four lines and four quarter-circle arcs. Prints the
//! solid's measurements and writes it to a STEP file.
//!
//! Used by the user manual ("Extrude a plate with rounded corners").
//!
//! Run: `cargo run -p swx-kernel --features occt --example extrude_rounded_plate -- rounded-plate.step`

use std::f64::consts::{FRAC_PI_2, PI};
use std::path::PathBuf;

use swx_kernel::{Extent, Kernel, OcctKernel, ProfileLoop, Segment2, Workplane};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "rounded-plate.step".into()),
    );
    let k = OcctKernel;
    let r = 5.0;
    let line = |start, end| Segment2::Line { start, end };
    // Arcs run counterclockwise from start_angle to end_angle (radians).
    let corner = |center, start_angle: f64| Segment2::Arc {
        center,
        radius: r,
        start_angle,
        end_angle: start_angle + FRAC_PI_2,
    };

    // Side, corner, side, corner, ... counterclockwise from the bottom.
    let outer = ProfileLoop {
        segments: vec![
            line([r, 0.0], [40.0 - r, 0.0]),
            corner([40.0 - r, r], -FRAC_PI_2),
            line([40.0, r], [40.0, 25.0 - r]),
            corner([40.0 - r, 25.0 - r], 0.0),
            line([40.0 - r, 25.0], [r, 25.0]),
            corner([r, 25.0 - r], FRAC_PI_2),
            line([0.0, 25.0 - r], [0.0, r]),
            corner([r, r], PI),
        ],
    };
    let hole = ProfileLoop {
        segments: vec![Segment2::Circle {
            center: [20.0, 12.5],
            radius: 5.0,
        }],
    };
    let face = k.make_face(&Workplane::XY, &outer, &[hole])?;
    let part = k.extrude(&face, Extent::Distance(5.0))?;

    let bb = k.bounding_box(&part)?;
    let round = |v: [f64; 3]| v.map(|c| (c * 100.0).round() / 100.0);
    println!("volume        {:.2} mm³", k.volume(&part)?);
    println!("faces         {}", k.face_count(&part)?);
    println!("bounding box  {:?} to {:?}", round(bb.min), round(bb.max));
    k.export_step(&part, &out)?;
    println!("wrote {}", out.display());
    Ok(())
}
