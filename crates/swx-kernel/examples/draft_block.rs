//! Drafts (tilts) the side faces of a 40 × 25 × 20 mm block, the way a part
//! is shaped so it can be pulled out of a mold: the four sides by 5° with the
//! bottom as the neutral plane, one side only, and the four sides by −5°.
//! Prints each result's size at the bottom and the top, shows the errors a
//! draft can give, and writes the solids to STEP files.
//!
//! Used by the user manual ("Tilting faces with draft").
//!
//! Run: `cargo run -p swx-kernel --features occt --example draft_block -- <dir>`

use std::path::PathBuf;

use swx_kernel::{Axis, Kernel, OcctKernel, OcctShape, Point3, SubShape, TopoKind};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn dot(a: Point3, b: Point3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The flat face of `s` whose outward normal is `normal`.
fn face_facing(k: &OcctKernel, s: &OcctShape, normal: Point3) -> Res<SubShape> {
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

/// Width × depth of the flat face of `s` that faces `normal`.
fn size_of(k: &OcctKernel, s: &OcctShape, normal: Point3) -> Res<String> {
    let b = k.sub_shape_bounding_box(s, face_facing(k, s, normal)?)?;
    Ok(format!(
        "{:.2} × {:.2}",
        b.max[0] - b.min[0],
        b.max[1] - b.min[1]
    ))
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Res<()> {
    println!(
        "{label:<14} {:>8.2} mm³  bottom {} mm  top {} mm",
        k.volume(s)?,
        size_of(k, s, [0.0, 0.0, -1.0])?,
        size_of(k, s, [0.0, 0.0, 1.0])?
    );
    Ok(())
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 20.0])?;
    report(&k, "block", &block)?;
    let mut sides = Vec::new();
    for n in [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
    ] {
        sides.push(face_facing(&k, &block, n)?);
    }
    let top = face_facing(&k, &block, [0.0, 0.0, 1.0])?;
    let indices: Vec<usize> = sides.iter().map(|f| f.index).collect();
    println!("side faces {indices:?}, top face {}", top.index);

    // The neutral plane is the bottom (z = 0); the part is pulled up, along +Z.
    let pull_up = Axis {
        origin: [0.0, 0.0, 0.0],
        dir: [0.0, 0.0, 1.0],
    };
    let five = 5f64.to_radians();
    let tapered = k.draft(&block, &sides, pull_up, five)?;
    report(&k, "4 sides, 5°", &tapered)?;
    let one = k.draft(&block, &sides[..1], pull_up, five)?;
    report(&k, "+X side, 5°", &one)?;
    let flared = k.draft(&block, &sides, pull_up, -five)?;
    report(&k, "4 sides, -5°", &flared)?;

    // The errors: the top face (parallel to the neutral plane), an angle of
    // 90°, an angle of zero, and no faces.
    for (label, r) in [
        ("top face", k.draft(&block, &[top], pull_up, five)),
        ("90°", k.draft(&block, &sides, pull_up, 90f64.to_radians())),
        ("0°", k.draft(&block, &sides, pull_up, 0.0)),
        ("no faces", k.draft(&block, &[], pull_up, five)),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [
        ("draft-four", &tapered),
        ("draft-one", &one),
        ("draft-flared", &flared),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
