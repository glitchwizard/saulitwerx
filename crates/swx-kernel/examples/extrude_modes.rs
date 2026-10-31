//! Extrudes three outlines from the top of a 40 × 25 × 10 mm block, one in
//! each combining mode (join, cut, intersect), prints each result's
//! measurements, writes every solid to a STEP file, and shows the two errors
//! a mode can give.
//!
//! Used by the user manual ("Join, cut or intersect with an extrude").
//!
//! Run: `cargo run -p swx-kernel --features occt --example extrude_modes -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    extrude_feature, Extent, ExtrudeMode, Kernel, OcctKernel, OcctShape, ProfileLoop, Segment2,
    Workplane,
};

/// The block's top face: the XY plane lifted to z = 10.
const TOP: Workplane = Workplane {
    origin: [0.0, 0.0, 10.0],
    x_dir: [1.0, 0.0, 0.0],
    normal: [0.0, 0.0, 1.0],
};

/// A loop of four lines through `corners`, in order.
fn rectangle(corners: [[f64; 2]; 4]) -> ProfileLoop {
    ProfileLoop {
        segments: (0..4)
            .map(|i| Segment2::Line {
                start: corners[i],
                end: corners[(i + 1) % 4],
            })
            .collect(),
    }
}

/// A loop that is one full circle.
fn circle(center: [f64; 2], radius: f64) -> ProfileLoop {
    ProfileLoop {
        segments: vec![Segment2::Circle { center, radius }],
    }
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Result<(), Box<dyn std::error::Error>> {
    let bb = k.bounding_box(s)?;
    println!(
        "{label:<10} {:>8.2} mm³  {:>2} faces  z from {} to {}",
        k.volume(s)?,
        k.face_count(s)?,
        (bb.min[2] * 100.0).round() / 100.0,
        (bb.max[2] * 100.0).round() / 100.0
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // The target: the body each mode combines with.
    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    report(&k, "target", &block)?;

    // Join: a 10 mm circle, 5 mm up from the top face, becomes a boss.
    let boss = k.make_face(&TOP, &circle([20.0, 12.5], 5.0), &[])?;
    let joined = extrude_feature(
        &k,
        &boss,
        Extent::Distance(5.0),
        ExtrudeMode::Join,
        Some(&block),
    )?;
    report(&k, "join", &joined)?;

    // Cut: a 20 × 10 mm rectangle, 4 mm down into the block, leaves a pocket.
    let pocket = k.make_face(
        &TOP,
        &rectangle([[10.0, 7.5], [30.0, 7.5], [30.0, 17.5], [10.0, 17.5]]),
        &[],
    )?;
    let cut = extrude_feature(
        &k,
        &pocket,
        Extent::Distance(-4.0),
        ExtrudeMode::Cut,
        Some(&block),
    )?;
    report(&k, "cut", &cut)?;

    // Intersect: a 30 mm circle, 10 mm down through the block, keeps only
    // the overlap: a disc with two flats where the block's sides trim it.
    let disc = k.make_face(&TOP, &circle([20.0, 12.5], 15.0), &[])?;
    let common = extrude_feature(
        &k,
        &disc,
        Extent::Distance(-10.0),
        ExtrudeMode::Intersect,
        Some(&block),
    )?;
    report(&k, "intersect", &common)?;

    // The two errors a mode adds.
    let no_target = extrude_feature(&k, &boss, Extent::Distance(5.0), ExtrudeMode::Join, None);
    println!("join, no target -> error: {}", no_target.unwrap_err());
    let everything = k.make_face(
        &TOP,
        &rectangle([[-5.0, -5.0], [45.0, -5.0], [45.0, 30.0], [-5.0, 30.0]]),
        &[],
    )?;
    let nothing_left = extrude_feature(
        &k,
        &everything,
        Extent::Distance(-10.0),
        ExtrudeMode::Cut,
        Some(&block),
    );
    println!("cut, all of it -> error: {}", nothing_left.unwrap_err());

    for (name, s) in [
        ("modes-target", &block),
        ("modes-join", &joined),
        ("modes-cut", &cut),
        ("modes-intersect", &common),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
