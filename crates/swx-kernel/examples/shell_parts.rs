//! Hollows solids with the shell operation: a 40 × 25 × 10 mm block opened
//! at the top (a tray), the same block opened at top and bottom (a frame),
//! and a cylinder opened at the top (a cup). Finds the faces to open by
//! their planes, prints each result's measurements, shows the errors a shell
//! can give, and writes the solids to STEP files.
//!
//! Used by the user manual ("Hollowing a part").
//!
//! Run: `cargo run -p swx-kernel --features occt --example shell_parts -- <dir>`

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel, OcctShape, Point3, SubShape, TopoKind};

/// The flat face of `s` whose outward normal is `normal`. `face_plane`
/// reports each flat face's plane; curved faces give an error and are
/// skipped.
fn face_facing(
    k: &OcctKernel,
    s: &OcctShape,
    normal: Point3,
) -> Result<SubShape, Box<dyn std::error::Error>> {
    let dot = |a: Point3, b: Point3| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    for index in 0..k.sub_shape_count(s, TopoKind::Face)? {
        let face = SubShape {
            kind: TopoKind::Face,
            index,
        };
        if let Ok((_, n)) = k.face_plane(s, face) {
            if dot(n, normal) > 0.999_999 {
                return Ok(face);
            }
        }
    }
    Err(format!("no flat face facing {normal:?}").into())
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Result<(), Box<dyn std::error::Error>> {
    let bb = k.bounding_box(s)?;
    let size: Vec<String> = (0..3)
        .map(|i| format!("{}", bb.max[i] - bb.min[i]))
        .collect();
    println!(
        "{label:<16} {:>8.2} mm³  {:>2} faces  outside {} mm",
        k.volume(s)?,
        k.face_count(s)?,
        size.join(" × ")
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    report(&k, "block", &block)?;
    let top = face_facing(&k, &block, [0.0, 0.0, 1.0])?;
    let bottom = face_facing(&k, &block, [0.0, 0.0, -1.0])?;
    println!("top face {}, bottom face {}", top.index, bottom.index);

    let tray = k.shell(&block, &[top], 2.0)?;
    report(&k, "tray, wall 2", &tray)?;
    let frame = k.shell(&block, &[top, bottom], 2.0)?;
    report(&k, "frame, wall 2", &frame)?;

    let can = k.make_cylinder([0.0, 0.0, 0.0], 15.0, 30.0)?;
    report(&k, "cylinder", &can)?;
    let rim = face_facing(&k, &can, [0.0, 0.0, 1.0])?;
    let cup = k.shell(&can, &[rim], 2.0)?;
    report(&k, "cup, wall 2", &cup)?;

    // The errors: a wall too thick for the block, no faces, an edge given as
    // a face, a face number the block doesn't have, and a wall of zero.
    let edge = SubShape {
        kind: TopoKind::Edge,
        index: 0,
    };
    let face_6 = SubShape {
        kind: TopoKind::Face,
        index: 6,
    };
    for (label, r) in [
        ("wall 13", k.shell(&block, &[top], 13.0)),
        ("no faces", k.shell(&block, &[], 2.0)),
        ("an edge", k.shell(&block, &[edge], 2.0)),
        ("face 6", k.shell(&block, &[face_6], 2.0)),
        ("wall 0", k.shell(&block, &[top], 0.0)),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [
        ("shell-tray", &tray),
        ("shell-frame", &frame),
        ("shell-cup", &cup),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
