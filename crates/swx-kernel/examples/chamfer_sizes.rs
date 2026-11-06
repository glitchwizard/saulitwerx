//! Bevels the edge between the top and a long side of a 40 × 25 × 10 mm
//! block four ways: an equal chamfer, two distances measured from the top,
//! the same two distances measured from the side, and a distance and an
//! angle. Prints each result's volume and where its new sloping face lies,
//! shows the errors an asymmetric chamfer can give, and writes the solids to
//! STEP files.
//!
//! Used by the user manual ("Chamfer with two sizes").
//!
//! Run: `cargo run -p swx-kernel --features occt --example chamfer_sizes -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    BoundingBox, ChamferSize, EdgeOnFace, Kernel, OcctKernel, OcctShape, SubShape, TopoKind,
};

/// Rounds a box to 0.01 mm and prints it as `y a..b  z c..d` (the block's
/// edge runs along x, so x adds nothing).
fn span(b: &BoundingBox) -> String {
    let r = |v: f64| (v * 100.0).round() / 100.0 + 0.0;
    format!(
        "y {}..{}  z {}..{}",
        r(b.min[1]),
        r(b.max[1]),
        r(b.min[2]),
        r(b.max[2])
    )
}

/// The one face of `s` whose box lies flat at `value` on `axis`.
fn face_at(k: &OcctKernel, s: &OcctShape, axis: usize, value: f64) -> SubShape {
    (0..k.face_count(s).unwrap())
        .map(|index| SubShape {
            kind: TopoKind::Face,
            index,
        })
        .find(|&f| {
            let b = k.sub_shape_bounding_box(s, f).unwrap();
            (b.min[axis] - value).abs() < 1e-6 && (b.max[axis] - value).abs() < 1e-6
        })
        .expect("face in plane")
}

/// Prints the volume and the sloping face: the one face whose normal points
/// along neither y nor z.
fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Result<(), Box<dyn std::error::Error>> {
    let mut slope = None;
    for index in 0..k.face_count(s)? {
        let f = SubShape {
            kind: TopoKind::Face,
            index,
        };
        let (_, n) = k.face_plane(s, f)?;
        if n[1].abs() > 1e-6 && n[2].abs() > 1e-6 {
            slope = Some(k.sub_shape_bounding_box(s, f)?);
        }
    }
    let slope = slope.map(|b| span(&b)).unwrap_or_default();
    println!("{label:<22} {:>8.2} mm³  slope {slope}", k.volume(s)?);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    println!("block                  {:>8.2} mm³", k.volume(&block)?);

    // The top (flat at z = 10), the long side that faces the viewer in
    // `swx-qa render` pictures (flat at y = 25), and the edge where they
    // meet: the one edge both faces list.
    let top = face_at(&k, &block, 2, 10.0);
    let side = face_at(&k, &block, 1, 25.0);
    let bottom = face_at(&k, &block, 2, 0.0);
    let top_edges = k.adjacent(&block, top, TopoKind::Edge)?;
    let edge = k
        .adjacent(&block, side, TopoKind::Edge)?
        .into_iter()
        .find(|e| top_edges.contains(e))
        .expect("shared edge");
    println!(
        "top face {}, side face {}, edge {}",
        top.index, side.index, edge.index
    );

    let on_top = EdgeOnFace { edge, face: top };
    let on_side = EdgeOnFace { edge, face: side };
    let two = ChamferSize::TwoDistances { d1: 6.0, d2: 3.0 };
    let angled = ChamferSize::DistanceAngle {
        distance: 6.0,
        angle: 30f64.to_radians(),
    };

    let equal = k.chamfer(&block, &[edge], 3.0)?;
    report(&k, "equal 3", &equal)?;
    let from_top = k.chamfer_asymmetric(&block, &[on_top], two)?;
    report(&k, "6 and 3, top first", &from_top)?;
    let from_side = k.chamfer_asymmetric(&block, &[on_side], two)?;
    report(&k, "6 and 3, side first", &from_side)?;
    let angle = k.chamfer_asymmetric(&block, &[on_top], angled)?;
    report(&k, "6 at 30°, top first", &angle)?;

    // The errors: a setback wider than the face, an angle of 90°, a face
    // that doesn't touch the edge, and the edge listed with both faces.
    let off = EdgeOnFace { edge, face: bottom };
    for (label, r) in [
        (
            "6 and 30",
            k.chamfer_asymmetric(
                &block,
                &[on_top],
                ChamferSize::TwoDistances { d1: 6.0, d2: 30.0 },
            ),
        ),
        (
            "6 at 90°",
            k.chamfer_asymmetric(
                &block,
                &[on_top],
                ChamferSize::DistanceAngle {
                    distance: 6.0,
                    angle: 90f64.to_radians(),
                },
            ),
        ),
        ("bottom face", k.chamfer_asymmetric(&block, &[off], two)),
        (
            "both faces",
            k.chamfer_asymmetric(&block, &[on_top, on_side], two),
        ),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [
        ("chamfer-equal", &equal),
        ("chamfer-top", &from_top),
        ("chamfer-side", &from_side),
        ("chamfer-angle", &angle),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
