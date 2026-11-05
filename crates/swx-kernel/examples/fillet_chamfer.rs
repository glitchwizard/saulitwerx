//! Rounds and bevels the edges of a 40 × 25 × 10 mm block: a fillet and a
//! chamfer on its four upright corner edges, and a small fillet on all twelve
//! edges. Finds the corner edges by their bounding boxes, prints each
//! result's measurements, shows the errors a fillet can give, and writes the
//! solids to STEP files.
//!
//! Used by the user manual ("Rounding and bevelling edges").
//!
//! Run: `cargo run -p swx-kernel --features occt --example fillet_chamfer -- <dir>`

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel, OcctShape, SubShape, TopoKind};

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{label:<18} {:>8.2} mm³  {:>2} faces  {:>2} edges",
        k.volume(s)?,
        k.face_count(s)?,
        k.sub_shape_count(s, TopoKind::Edge)?
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    report(&k, "block", &block)?;

    // Every edge of the block, and the four upright ones: an edge whose box
    // has no width in x or y runs straight up.
    let all: Vec<SubShape> = (0..k.sub_shape_count(&block, TopoKind::Edge)?)
        .map(|index| SubShape {
            kind: TopoKind::Edge,
            index,
        })
        .collect();
    let mut corners = Vec::new();
    for &edge in &all {
        let bb = k.sub_shape_bounding_box(&block, edge)?;
        if bb.max[0] - bb.min[0] < 1e-6 && bb.max[1] - bb.min[1] < 1e-6 {
            corners.push(edge);
        }
    }
    let list: Vec<usize> = corners.iter().map(|e| e.index).collect();
    println!("upright edges: {list:?}");

    let rounded = k.fillet(&block, &corners, 5.0)?;
    report(&k, "fillet r 5", &rounded)?;
    let bevelled = k.chamfer(&block, &corners, 5.0)?;
    report(&k, "chamfer 5", &bevelled)?;
    let soft = k.fillet(&block, &all, 2.0)?;
    report(&k, "fillet all, r 2", &soft)?;

    // The errors: a radius too large for the block, no edges, a face given
    // as an edge, and a radius of zero.
    let face = SubShape {
        kind: TopoKind::Face,
        index: 0,
    };
    for (label, r) in [
        ("r 15", k.fillet(&block, &corners, 15.0)),
        ("no edges", k.fillet(&block, &[], 2.0)),
        ("a face", k.fillet(&block, &[face], 2.0)),
        ("r 0", k.fillet(&block, &corners, 0.0)),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [
        ("edges-fillet", &rounded),
        ("edges-chamfer", &bevelled),
        ("edges-all", &soft),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
