//! Builds work features in between and around a part's geometry: a shaft
//! 16 mm across and 50 mm long. Takes a midplane halfway between its two
//! end faces and drills a cross hole through the shaft on it; takes a
//! plane tangent to its round side and cuts a flat for a set screw from it;
//! and takes the work point in the middle of the flat's top edge. Prints
//! each work feature and the result, shows the errors, and writes the part
//! to a STEP file.
//!
//! Used by the user manual ("Work planes, axes and points").
//!
//! Run: `cargo run -p swx-kernel --features occt --example work_midplane -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    extrude_feature, Axis, Extent, ExtrudeMode, HoleDepth, HoleKind, Kernel, OcctKernel, OcctShape,
    Point3, ProfileLoop, Segment2, SubShape, TopoKind, Workplane,
};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// A number rounded to 3 places, with no "-0".
fn n(x: f64) -> String {
    let r = (x * 1000.0).round() / 1000.0;
    format!("{}", if r == 0.0 { 0.0 } else { r })
}

fn p(v: Point3) -> String {
    format!("({}, {}, {})", n(v[0]), n(v[1]), n(v[2]))
}

/// The first sub-shape of `kind` in `s` whose bounding box is `min`–`max`.
fn find(k: &OcctKernel, s: &OcctShape, kind: TopoKind, min: Point3, max: Point3) -> Res<SubShape> {
    let near = |a: Point3, b: Point3| (0..3).all(|i| (a[i] - b[i]).abs() < 1e-6);
    for index in 0..k.sub_shape_count(s, kind)? {
        let at = SubShape { kind, index };
        let b = k.sub_shape_bounding_box(s, at)?;
        if near(b.min, min) && near(b.max, max) {
            return Ok(at);
        }
    }
    Err(format!("no {kind:?} with box {min:?}–{max:?}").into())
}

fn line(start: [f64; 2], end: [f64; 2]) -> Segment2 {
    Segment2::Line { start, end }
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // The shaft stands on the XY plane, centered on the Z axis.
    let shaft = k.make_cylinder([0.0, 0.0, 0.0], 8.0, 50.0)?;
    let bottom = find(
        &k,
        &shaft,
        TopoKind::Face,
        [-8.0, -8.0, 0.0],
        [8.0, 8.0, 0.0],
    )?;
    let top = find(
        &k,
        &shaft,
        TopoKind::Face,
        [-8.0, -8.0, 50.0],
        [8.0, 8.0, 50.0],
    )?;
    let side = find(
        &k,
        &shaft,
        TopoKind::Face,
        [-8.0, -8.0, 0.0],
        [8.0, 8.0, 50.0],
    )?;
    println!(
        "bottom face {}, top face {}, round face {}",
        bottom.index, top.index, side.index
    );

    // Halfway between the two ends.
    let a = k.work_plane_on_face(&shaft, bottom)?;
    let b = k.work_plane_on_face(&shaft, top)?;
    let mid = Workplane::midplane(&a, &b)?;
    println!(
        "midplane               origin {}  normal {}",
        p(mid.origin),
        p(mid.normal)
    );

    // Touching the round side, on the +Y side.
    let tangent = k.work_plane_tangent(&shaft, side, [0.0, 1.0, 0.0])?;
    println!(
        "tangent plane (+Y)     origin {}  normal {}  x_dir {}",
        p(tangent.origin),
        p(tangent.normal),
        p(tangent.x_dir)
    );

    // The errors: a midplane of two planes that cross, a tangent plane on a
    // flat face, a tangent side along the axis, and the midpoint of a face.
    println!(
        "midplane of XY and XZ -> error: {}",
        Workplane::midplane(&Workplane::XY, &Workplane::XZ).unwrap_err()
    );
    println!(
        "tangent on top face -> error: {}",
        k.work_plane_tangent(&shaft, top, [0.0, 1.0, 0.0])
            .unwrap_err()
    );
    println!(
        "tangent toward +Z -> error: {}",
        k.work_plane_tangent(&shaft, side, [0.0, 0.0, 1.0])
            .unwrap_err()
    );
    println!(
        "midpoint of a face -> error: {}",
        k.edge_midpoint(&shaft, side).unwrap_err()
    );

    // Use the midplane: a cross hole 5 mm across, square to the shaft,
    // along the line where the midplane meets the XZ plane. The hole starts
    // outside the shaft and points into it.
    let across = Axis::plane_intersection(&mid, &Workplane::XZ)?;
    let start = Axis {
        origin: [-10.0, across.origin[1], across.origin[2]],
        dir: [1.0, 0.0, 0.0],
    };
    let drilled = k.hole(
        &shaft,
        start,
        HoleKind::Simple { diameter: 5.0 },
        HoleDepth::ThroughAll,
    )?;

    // Use the tangent plane: a flat 2 mm deep along the top 20 mm of the
    // shaft. The plane's u runs along its x_dir, +X, and v along
    // normal × x_dir, which is -Z, so the top 20 mm is v from -51 to -30.
    // The rectangle is sketched on the plane moved 2 mm into the shaft and
    // cut outward, away from the axis.
    let floor = tangent.offset(-2.0)?;
    let (u0, u1, v0, v1) = (-10.0, 10.0, -51.0, -30.0);
    let rect = ProfileLoop {
        segments: vec![
            line([u0, v0], [u1, v0]),
            line([u1, v0], [u1, v1]),
            line([u1, v1], [u0, v1]),
            line([u0, v1], [u0, v0]),
        ],
    };
    let face = k.make_face(&floor, &rect, &[])?;
    let finished = extrude_feature(
        &k,
        &face,
        Extent::Distance(5.0),
        ExtrudeMode::Cut,
        Some(&drilled),
    )?;
    println!(
        "shaft with hole and flat: {:.2} mm³, {} faces",
        k.volume(&finished)?,
        k.face_count(&finished)?
    );

    // The flat's top edge is a straight line across the shaft's top at
    // y = 6, from x = -√28 to +√28. Its midpoint is on the Y axis.
    let half = 28f64.sqrt();
    let edge = find(
        &k,
        &finished,
        TopoKind::Edge,
        [-half, 6.0, 50.0],
        [half, 6.0, 50.0],
    )?;
    println!(
        "flat's top edge {}: midpoint {}",
        edge.index,
        p(k.edge_midpoint(&finished, edge)?)
    );

    let file = dir.join("work-midplane.step");
    k.export_step(&finished, &file)?;
    println!("wrote {}", file.display());
    Ok(())
}
