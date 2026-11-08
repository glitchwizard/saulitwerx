//! Rounds one edge of a 40 × 25 × 10 mm block with a fillet whose radius
//! changes along the edge: 2 mm at one end, 8 mm at the other. Compares it
//! with a constant 5 mm fillet of the same edge, starts from the other end to
//! show that the radii swap, prints each result's measurements, shows the
//! errors a variable fillet can give, and writes the solids to STEP files.
//!
//! Used by the user manual ("Rounding and bevelling edges").
//!
//! Run: `cargo run -p swx-kernel --features occt --example fillet_variable -- <dir>`

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel, OcctShape, SubShape, TopoKind};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn sub(kind: TopoKind, index: usize) -> SubShape {
    SubShape { kind, index }
}

/// The edge of `s` that runs along X (its box has no depth or height) at
/// height `z` and depth `y`.
fn edge_along_x(k: &OcctKernel, s: &OcctShape, y: f64, z: f64) -> Res<SubShape> {
    for index in 0..k.sub_shape_count(s, TopoKind::Edge)? {
        let e = sub(TopoKind::Edge, index);
        let b = k.sub_shape_bounding_box(s, e)?;
        let flat = (b.max[1] - b.min[1]).abs() < 1e-6 && (b.max[2] - b.min[2]).abs() < 1e-6;
        if flat && (b.min[1] - y).abs() < 1e-6 && (b.min[2] - z).abs() < 1e-6 {
            return Ok(e);
        }
    }
    Err("edge not found".into())
}

/// The end of `edge` whose x coordinate is `x`.
fn end_at_x(k: &OcctKernel, s: &OcctShape, edge: SubShape, x: f64) -> Res<SubShape> {
    for v in k.adjacent(s, edge, TopoKind::Vertex)? {
        if (k.sub_shape_bounding_box(s, v)?.min[0] - x).abs() < 1e-6 {
            return Ok(v);
        }
    }
    Err("vertex not found".into())
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Res<()> {
    println!(
        "{label:<22} {:>8.2} mm³  {:>2} faces",
        k.volume(s)?,
        k.face_count(s)?
    );
    Ok(())
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    report(&k, "block", &block)?;
    // The top edge along the long side nearest you, from x = 0 to x = 40.
    let edge = edge_along_x(&k, &block, 25.0, 10.0)?;
    let left = end_at_x(&k, &block, edge, 0.0)?;
    let right = end_at_x(&k, &block, edge, 40.0)?;
    println!(
        "edge {}: vertex {} at x = 0, vertex {} at x = 40",
        edge.index, left.index, right.index
    );

    let constant = k.fillet(&block, &[edge], 5.0)?;
    report(&k, "constant r 5", &constant)?;
    let growing = k.fillet_variable(&block, edge, left, 2.0, 8.0)?;
    report(&k, "2 at x = 0, 8 at 40", &growing)?;
    let shrinking = k.fillet_variable(&block, edge, right, 2.0, 8.0)?;
    report(&k, "2 at x = 40, 8 at 0", &shrinking)?;

    // The errors: a radius too large for the 10 mm side, a vertex that is
    // not on the edge, a face given as the edge, and a radius of zero.
    let far_vertex = k
        .adjacent(
            &block,
            edge_along_x(&k, &block, 0.0, 0.0)?,
            TopoKind::Vertex,
        )?
        .remove(0);
    for (label, r) in [
        ("2 to 12", k.fillet_variable(&block, edge, left, 2.0, 12.0)),
        (
            "wrong vertex",
            k.fillet_variable(&block, edge, far_vertex, 2.0, 8.0),
        ),
        (
            "a face",
            k.fillet_variable(&block, sub(TopoKind::Face, 0), left, 2.0, 8.0),
        ),
        ("0 to 8", k.fillet_variable(&block, edge, left, 0.0, 8.0)),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [
        ("fillet-constant", &constant),
        ("fillet-variable", &growing),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
