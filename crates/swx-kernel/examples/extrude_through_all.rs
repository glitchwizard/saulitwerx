//! Cuts two holes through a 40 × 25 × 10 mm block with the "through all"
//! extent: one straight down from the top face, one both ways from a
//! workplane in the middle of the block. Prints each result's measurements,
//! writes the solids to STEP files, and shows the two errors the extent can
//! give.
//!
//! Used by the user manual ("Cut through the whole part").
//!
//! Run: `cargo run -p swx-kernel --features occt --example extrude_through_all -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    extrude_feature, Extent, ExtrudeMode, Kernel, OcctKernel, OcctShape, ProfileLoop, Segment2,
    ThroughDirection, Workplane,
};

/// The block's top face: the XY plane lifted to z = 10. Its normal points up,
/// out of the block.
const TOP: Workplane = Workplane {
    origin: [0.0, 0.0, 10.0],
    x_dir: [1.0, 0.0, 0.0],
    normal: [0.0, 0.0, 1.0],
};

/// A plane through the middle of the block, parallel to its long side
/// (y = 12.5). Left–right is +X and up–down is +Z.
const MIDDLE: Workplane = Workplane {
    origin: [0.0, 12.5, 0.0],
    x_dir: [1.0, 0.0, 0.0],
    normal: [0.0, -1.0, 0.0],
};

/// A loop that is one full circle.
fn circle(center: [f64; 2], radius: f64) -> ProfileLoop {
    ProfileLoop {
        segments: vec![Segment2::Circle { center, radius }],
    }
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{label:<16} {:>8.2} mm³  {:>2} faces",
        k.volume(s)?,
        k.face_count(s)?
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    report(&k, "block", &block)?;

    // An 8 mm hole drawn on the top face, cut through all, against the
    // normal: down into the block and out of the bottom.
    let top_hole = k.make_face(&TOP, &circle([12.0, 12.5], 4.0), &[])?;
    let drilled = extrude_feature(
        &k,
        &top_hole,
        Extent::ThroughAll(ThroughDirection::Reversed),
        ExtrudeMode::Cut,
        Some(&block),
    )?;
    report(&k, "down (reversed)", &drilled)?;

    // A 6 mm hole drawn on the middle plane, cut through all both ways: out
    // through the front and the back of the block.
    let cross_hole = k.make_face(&MIDDLE, &circle([30.0, 5.0], 3.0), &[])?;
    let cross = extrude_feature(
        &k,
        &cross_hole,
        Extent::ThroughAll(ThroughDirection::Both),
        ExtrudeMode::Cut,
        Some(&drilled),
    )?;
    report(&k, "both ways", &cross)?;

    // The two errors the extent adds.
    let missed = extrude_feature(
        &k,
        &top_hole,
        Extent::ThroughAll(ThroughDirection::Normal),
        ExtrudeMode::Cut,
        Some(&block),
    );
    println!("up (normal) -> error: {}", missed.unwrap_err());
    let joined = extrude_feature(
        &k,
        &top_hole,
        Extent::ThroughAll(ThroughDirection::Normal),
        ExtrudeMode::Join,
        Some(&block),
    );
    println!("join -> error: {}", joined.unwrap_err());

    for (name, s) in [("through-down", &drilled), ("through-both", &cross)] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
