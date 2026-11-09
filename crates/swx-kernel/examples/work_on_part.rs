//! Builds work features from a part's own geometry: a 40 × 30 × 20 mm block
//! with a round boss 12 mm across and 10 mm tall on its top. Takes a work
//! plane on the block's +X face, a work axis along one of its edges and
//! another along the boss's round face, and a work point at a corner. Then
//! uses two of them: a post sketched on the side face's plane, and a hole
//! drilled along the boss's axis. Prints each work feature and the result,
//! shows the errors, and writes the part to a STEP file.
//!
//! Used by the user manual ("Work planes, axes and points").
//!
//! Run: `cargo run -p swx-kernel --features occt --example work_on_part -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    extrude_feature, Extent, ExtrudeMode, HoleDepth, HoleKind, Kernel, OcctKernel, OcctShape,
    Point3, ProfileLoop, Segment2, SubShape, TopoKind,
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

fn sub(kind: TopoKind, index: usize) -> SubShape {
    SubShape { kind, index }
}

/// The first sub-shape of `kind` in `s` whose bounding box is `min`–`max`.
fn find(k: &OcctKernel, s: &OcctShape, kind: TopoKind, min: Point3, max: Point3) -> Res<SubShape> {
    let near = |a: Point3, b: Point3| (0..3).all(|i| (a[i] - b[i]).abs() < 1e-6);
    for index in 0..k.sub_shape_count(s, kind)? {
        let b = k.sub_shape_bounding_box(s, sub(kind, index))?;
        if near(b.min, min) && near(b.max, max) {
            return Ok(sub(kind, index));
        }
    }
    Err(format!("no {kind:?} with box {min:?}–{max:?}").into())
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 30.0, 20.0])?;
    let boss = k.make_cylinder([20.0, 15.0, 20.0], 6.0, 10.0)?;
    let part = k.union(&block, &boss)?;

    // The block's +X face, the top edge along it, the boss's round face,
    // its circular top edge, and the corner at (40, 30, 20).
    let side = find(
        &k,
        &part,
        TopoKind::Face,
        [40.0, 0.0, 0.0],
        [40.0, 30.0, 20.0],
    )?;
    let edge = find(
        &k,
        &part,
        TopoKind::Edge,
        [40.0, 0.0, 20.0],
        [40.0, 30.0, 20.0],
    )?;
    let round = find(
        &k,
        &part,
        TopoKind::Face,
        [14.0, 9.0, 20.0],
        [26.0, 21.0, 30.0],
    )?;
    let rim = find(
        &k,
        &part,
        TopoKind::Edge,
        [14.0, 9.0, 30.0],
        [26.0, 21.0, 30.0],
    )?;
    let corner = find(
        &k,
        &part,
        TopoKind::Vertex,
        [40.0, 30.0, 20.0],
        [40.0, 30.0, 20.0],
    )?;
    println!(
        "side face {}, top edge {}, round face {}, rim edge {}, corner vertex {}",
        side.index, edge.index, round.index, rim.index, corner.index
    );

    let plane = k.work_plane_on_face(&part, side)?;
    println!(
        "plane on side face     origin {}  normal {}  x_dir {}",
        p(plane.origin),
        p(plane.normal),
        p(plane.x_dir)
    );
    let along = k.work_axis_on(&part, edge)?;
    println!(
        "axis on top edge       through {}  along {}",
        p(along.origin),
        p(along.dir)
    );
    let mut boss_axis = k.work_axis_on(&part, round)?;
    println!(
        "axis on round face     through {}  along {}",
        p(boss_axis.origin),
        p(boss_axis.dir)
    );
    println!(
        "point at corner        {}",
        p(k.vertex_point(&part, corner)?)
    );

    // The errors: a plane on an edge, a plane on the round face, an axis on
    // a vertex, and an axis on the round top edge.
    println!(
        "plane on edge -> error: {}",
        k.work_plane_on_face(&part, edge).unwrap_err()
    );
    println!(
        "plane on round face -> error: {}",
        k.work_plane_on_face(&part, round).unwrap_err()
    );
    println!(
        "axis on corner -> error: {}",
        k.work_axis_on(&part, corner).unwrap_err()
    );
    println!(
        "axis on rim -> error: {}",
        k.work_axis_on(&part, rim).unwrap_err()
    );

    // Use the side plane: a circle 10 mm across in the middle of the face
    // (u runs along +Y, v along +Z), extruded 8 mm out and joined.
    let circle = ProfileLoop {
        segments: vec![Segment2::Circle {
            center: [15.0, 10.0],
            radius: 5.0,
        }],
    };
    let face = k.make_face(&plane, &circle, &[])?;
    let with_post = extrude_feature(
        &k,
        &face,
        Extent::Distance(8.0),
        ExtrudeMode::Join,
        Some(&part),
    )?;

    // Use the boss's axis: a 6 mm hole through the whole part along it. The
    // axis may point either way; the hole starts at its origin (on the
    // bottom face) and must point into the part, so point it up.
    if boss_axis.dir[2] < 0.0 {
        boss_axis.dir = boss_axis.dir.map(|c| -c);
    }
    let finished = k.hole(
        &with_post,
        boss_axis,
        HoleKind::Simple { diameter: 6.0 },
        HoleDepth::ThroughAll,
    )?;
    println!(
        "with post and hole: {:.2} mm³, {} faces",
        k.volume(&finished)?,
        k.face_count(&finished)?
    );

    let file = dir.join("work-on-part.step");
    k.export_step(&finished, &file)?;
    println!("wrote {}", file.display());
    Ok(())
}
