//! Extrudes up to faces of a stepped block instead of by a typed distance:
//! a post joined up to the block's top face, a pocket cut down to its step
//! face, and a slot cut between the two. Also shows the from–to extent, and
//! the two errors a face extent most often gives. Writes the solids to STEP
//! files.
//!
//! Used by the user manual ("Extrude up to a face").
//!
//! Run: `cargo run -p swx-kernel --features occt --example extrude_to_face -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    extrude_feature, Extent, ExtrudeMode, Kernel, OcctKernel, OcctShape, Point3, ProfileLoop,
    Segment2, SubShape, TopoKind, Workplane,
};

/// The XY plane lifted to height `z`, normal pointing up.
fn plane_at(z: f64) -> Workplane {
    Workplane {
        origin: [0.0, 0.0, z],
        ..Workplane::XY
    }
}

/// A rectangle from corner `a` to corner `b`, in workplane coordinates.
fn rectangle(a: [f64; 2], b: [f64; 2]) -> ProfileLoop {
    let c = [a, [b[0], a[1]], b, [a[0], b[1]]];
    ProfileLoop {
        segments: (0..4)
            .map(|i| Segment2::Line {
                start: c[i],
                end: c[(i + 1) % 4],
            })
            .collect(),
    }
}

/// The planar face of `s` whose outward normal is `normal` and whose plane
/// lies `at` mm along it from the origin. `face_plane` reports each planar
/// face's plane; curved faces are skipped.
fn find_face(
    k: &OcctKernel,
    s: &OcctShape,
    normal: Point3,
    at: f64,
) -> Result<SubShape, Box<dyn std::error::Error>> {
    let dot = |a: Point3, b: Point3| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    for index in 0..k.sub_shape_count(s, TopoKind::Face)? {
        let face = SubShape {
            kind: TopoKind::Face,
            index,
        };
        if let Ok((p, n)) = k.face_plane(s, face) {
            if dot(n, normal) > 0.999_999 && (dot(p, normal) - at).abs() < 1e-6 {
                return Ok(face);
            }
        }
    }
    Err(format!("no face with normal {normal:?} at {at}").into())
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Result<(), Box<dyn std::error::Error>> {
    let bb = k.bounding_box(s)?;
    println!(
        "{label:<14} {:>8.2} mm³  {:>2} faces  z from {} to {}",
        k.volume(s)?,
        k.face_count(s)?,
        bb.min[2],
        bb.max[2]
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // The target: a 40 × 25 × 10 mm block with a 20 × 25 × 30 mm block on
    // its left half. Its step face is at z = 10, its top face at z = 30.
    let low = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    let high = k.make_box([0.0, 0.0, 0.0], [20.0, 25.0, 30.0])?;
    let stepped = k.union(&low, &high)?;
    let up = [0.0, 0.0, 1.0];
    let step = find_face(&k, &stepped, up, 10.0)?;
    let top = find_face(&k, &stepped, up, 30.0)?;
    report(&k, "stepped block", &stepped)?;
    println!(
        "step face is face {}, top face is face {}",
        step.index, top.index
    );

    // A 10 mm post drawn on the step face, joined up to the top face.
    let post = k.make_face(
        &plane_at(10.0),
        &ProfileLoop {
            segments: vec![Segment2::Circle {
                center: [30.0, 12.5],
                radius: 5.0,
            }],
        },
        &[],
    )?;
    let joined = extrude_feature(
        &k,
        &post,
        Extent::ToFace(top),
        ExtrudeMode::Join,
        Some(&stepped),
    )?;
    report(&k, "post to top", &joined)?;

    // A 10 × 10 mm pocket drawn on the top face, cut down to the step face.
    // The step face is behind the workplane, so the cut goes down.
    let pocket = k.make_face(&plane_at(30.0), &rectangle([5.0, 7.5], [15.0, 17.5]), &[])?;
    let pocketed = extrude_feature(
        &k,
        &pocket,
        Extent::ToFace(step),
        ExtrudeMode::Cut,
        Some(&stepped),
    )?;
    report(&k, "pocket to step", &pocketed)?;

    // A 7 mm slot across the tall part, drawn on the XY plane at z = 0 and
    // cut between the step face and the top face.
    let slot = k.make_face(&Workplane::XY, &rectangle([0.0, 9.0], [20.0, 16.0]), &[])?;
    let slotted = extrude_feature(
        &k,
        &slot,
        Extent::BetweenFaces(step, top),
        ExtrudeMode::Cut,
        Some(&stepped),
    )?;
    report(&k, "slot between", &slotted)?;

    // The from–to extent: from 10 mm to 30 mm along the normal, with no
    // target at all.
    let block = k.extrude(&slot, Extent::FromTo(10.0, 30.0))?;
    report(&k, "from 10 to 30", &block)?;

    // The two errors a face extent most often gives.
    let alone = extrude_feature(&k, &post, Extent::ToFace(top), ExtrudeMode::NewSolid, None);
    println!("new solid -> error: {}", alone.unwrap_err());
    let side = find_face(&k, &stepped, [1.0, 0.0, 0.0], 40.0)?;
    let tilted = extrude_feature(
        &k,
        &post,
        Extent::ToFace(side),
        ExtrudeMode::Join,
        Some(&stepped),
    );
    println!("side face -> error: {}", tilted.unwrap_err());

    for (name, s) in [
        ("toface-target", &stepped),
        ("toface-post", &joined),
        ("toface-pocket", &pocketed),
        ("toface-slot", &slotted),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
