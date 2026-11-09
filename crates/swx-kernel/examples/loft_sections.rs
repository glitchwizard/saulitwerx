//! Lofts solids through flat sections: a 40 × 40 mm square at the bottom to
//! a circle 24 mm across 40 mm above it (a square-to-round transition), and
//! a vase through three circles (40, 20 and 32 mm across, at heights 0, 30
//! and 60 mm), once ruled and once smooth. Prints each result, shows the
//! errors a loft can give, and writes the solids to STEP files.
//!
//! Used by the user manual ("Lofting between sections").
//!
//! Run: `cargo run -p swx-kernel --features occt --example loft_sections -- <dir>`

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel, OcctShape, ProfileLoop, Segment2, Workplane};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn line(start: [f64; 2], end: [f64; 2]) -> Segment2 {
    Segment2::Line { start, end }
}

/// A square of side `side` centered on the plane's origin.
fn square(side: f64) -> ProfileLoop {
    let h = side / 2.0;
    ProfileLoop {
        segments: vec![
            line([-h, -h], [h, -h]),
            line([h, -h], [h, h]),
            line([h, h], [-h, h]),
            line([-h, h], [-h, -h]),
        ],
    }
}

/// A circle of `diameter` centered on the plane's origin.
fn circle(diameter: f64) -> ProfileLoop {
    ProfileLoop {
        segments: vec![Segment2::Circle {
            center: [0.0, 0.0],
            radius: diameter / 2.0,
        }],
    }
}

/// A section face: `outline` on the XY plane raised to `height`.
fn section(k: &OcctKernel, outline: &ProfileLoop, height: f64) -> Res<OcctShape> {
    Ok(k.make_face(&Workplane::XY.offset(height)?, outline, &[])?)
}

fn report(k: &OcctKernel, label: &str, s: &OcctShape) -> Res<()> {
    let b = k.bounding_box(s)?;
    println!(
        "{label:<15} {:>9.2} mm³  {} faces  height {:.2} mm",
        k.volume(s)?,
        k.face_count(s)?,
        b.max[2] - b.min[2]
    );
    Ok(())
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // A square bottom to a round top.
    let bottom = || section(&k, &square(40.0), 0.0);
    let top = section(&k, &circle(24.0), 40.0)?;
    let transition = k.loft(&[bottom()?, top], true)?;
    report(&k, "square to round", &transition)?;

    // A vase through three circles: ruled (straight between sections) and
    // smooth (one surface through all three).
    let rings = [
        section(&k, &circle(40.0), 0.0)?,
        section(&k, &circle(20.0), 30.0)?,
        section(&k, &circle(32.0), 60.0)?,
    ];
    let ruled = k.loft(&rings, true)?;
    report(&k, "vase, ruled", &ruled)?;
    let smooth = k.loft(&rings, false)?;
    report(&k, "vase, smooth", &smooth)?;

    // The errors: one section, a section with a hole, a solid as a section,
    // and two equal sections in the same plane.
    let washer = k.make_face(&Workplane::XY.offset(40.0)?, &circle(24.0), &[circle(10.0)])?;
    let cube = k.make_box([0.0, 0.0, 0.0], [10.0, 10.0, 10.0])?;
    for (label, r) in [
        ("one section", k.loft(&[bottom()?], true)),
        ("hole in top", k.loft(&[bottom()?, washer], true)),
        ("solid section", k.loft(&[bottom()?, cube], true)),
        ("same section twice", k.loft(&[bottom()?, bottom()?], true)),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }

    for (name, s) in [
        ("loft-transition", &transition),
        ("loft-ruled", &ruled),
        ("loft-smooth", &smooth),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(s, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
