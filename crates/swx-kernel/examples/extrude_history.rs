//! Makes the 40 × 25 mm plate with a 10 mm hole, extrudes it 5 mm with
//! history, and prints which face of the plate each piece of the outline
//! became: the two caps, and the side face swept from every edge.
//!
//! Used by the user manual ("Find the faces an extrude made").
//!
//! Run: `cargo run -p swx-kernel --features occt --example extrude_history`

use swx_kernel::{BoundingBox, Extent, Kernel, OcctKernel, ProfileLoop, Segment2, Workplane};

/// Rounds a box to 0.01 mm and prints it as `x a..b  y c..d  z e..f`.
fn span(b: &BoundingBox) -> String {
    let r = |v: f64| (v * 100.0).round() / 100.0 + 0.0;
    format!(
        "x {}..{}  y {}..{}  z {}..{}",
        r(b.min[0]),
        r(b.max[0]),
        r(b.min[1]),
        r(b.max[1]),
        r(b.min[2]),
        r(b.max[2])
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let k = OcctKernel;

    // The outline of `extrude_plate`: four lines and a round hole.
    let corners = [[0.0, 0.0], [40.0, 0.0], [40.0, 25.0], [0.0, 25.0]];
    let outer = ProfileLoop {
        segments: (0..4)
            .map(|i| Segment2::Line {
                start: corners[i],
                end: corners[(i + 1) % 4],
            })
            .collect(),
    };
    let hole = ProfileLoop {
        segments: vec![Segment2::Circle {
            center: [20.0, 12.5],
            radius: 5.0,
        }],
    };

    // 1. The face, and the edge each segment became.
    let (face, edges) = k.make_face_tracked(&Workplane::XY, &outer, &[hole])?;
    // 2. The solid, and the face each edge became.
    let (part, history) = k.extrude_tracked(&face, Extent::Distance(5.0))?;

    println!("start cap     face {}", history.start_cap.index);
    println!("end cap       face {}", history.end_cap.index);
    for (lp, name) in edges.iter().zip(["outer", "hole"]) {
        for (seg, edge) in lp.iter().enumerate() {
            // 3. Follow the edge to the side face it swept.
            let side = history
                .sides
                .iter()
                .find(|(e, _)| e == edge)
                .map(|(_, f)| *f)
                .ok_or("edge not in history")?;
            let bb = k.sub_shape_bounding_box(&part, side)?;
            println!(
                "{name} {seg}  ->  edge {}  ->  face {}   {}",
                edge.index,
                side.index,
                span(&bb)
            );
        }
    }
    let top = k.sub_shape_bounding_box(&part, history.end_cap)?;
    println!("end cap is    {}", span(&top));
    Ok(())
}
