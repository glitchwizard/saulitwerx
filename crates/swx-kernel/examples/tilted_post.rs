//! Uses a work plane to place a feature at an angle: a 40 × 40 × 10 mm plate
//! with a round post standing at 30° on its top. The post's sketch plane is
//! the plate's top plane (the XY plane offset 10 mm) turned 30° about a work
//! axis across its middle. Prints the plane and the result, and writes the
//! part to a STEP file.
//!
//! Used by the user manual ("Work planes, axes and points").
//!
//! Run: `cargo run -p swx-kernel --features occt --example tilted_post -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    extrude_feature, Axis, Extent, ExtrudeMode, Kernel, OcctKernel, ProfileLoop, Segment2,
    Workplane,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let plate = k.make_box([0.0, 0.0, 0.0], [40.0, 40.0, 10.0])?;
    let top = Workplane::XY.offset(10.0)?;
    let across = Axis {
        origin: [0.0, 20.0, 10.0],
        dir: [1.0, 0.0, 0.0],
    };
    let tilted = top.angled(across, 30f64.to_radians())?;
    println!(
        "tilted plane: origin {:?}, normal [{:.3}, {:.3}, {:.3}]",
        tilted.origin, tilted.normal[0], tilted.normal[1], tilted.normal[2]
    );

    // A circle 12 mm across, centered on the plate's middle. Turning the
    // plane turned its origin too, so the hinge line is at v = 20, 20 mm
    // from the plane's origin as before. The circle starts 4 mm below the
    // tilted plane, so the whole foot of the post is inside the plate, and is
    // extruded 24 mm along the tilted normal (20 mm above the plane) and
    // joined to the plate.
    let foot = tilted.offset(-4.0)?;
    let circle = ProfileLoop {
        segments: vec![Segment2::Circle {
            center: [20.0, 20.0],
            radius: 6.0,
        }],
    };
    let face = k.make_face(&foot, &circle, &[])?;
    let part = extrude_feature(
        &k,
        &face,
        Extent::Distance(24.0),
        ExtrudeMode::Join,
        Some(&plate),
    )?;
    println!(
        "plate with post: {:.2} mm³, {} faces",
        k.volume(&part)?,
        k.face_count(&part)?
    );
    let path = dir.join("tilted-post.step");
    k.export_step(&part, &path)?;
    println!("wrote {}", path.display());
    Ok(())
}
