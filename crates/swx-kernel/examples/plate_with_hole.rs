//! Builds a 40 × 25 × 5 mm plate with a 10 mm through hole, prints its
//! measurements, and writes it to a STEP file.
//!
//! Used by the user manual ("Make a solid and save it as STEP").
//!
//! Run: `cargo run -p swx-kernel --features occt --example plate_with_hole -- plate.step`

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "plate.step".into()),
    );
    let k = OcctKernel;

    let plate = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 5.0])?;
    // A tool cylinder taller than the plate, so the cut goes all the way through.
    let tool = k.make_cylinder([20.0, 12.5, -1.0], 5.0, 7.0)?;
    let part = k.cut(&plate, &tool)?;

    let bb = k.bounding_box(&part)?;
    println!("volume        {:.2} mm³", k.volume(&part)?);
    println!("surface area  {:.2} mm²", k.surface_area(&part)?);
    println!("faces         {}", k.face_count(&part)?);
    println!(
        "bounding box  {:?} to {:?}",
        bb.min.map(|v| (v * 100.0).round() / 100.0),
        bb.max.map(|v| (v * 100.0).round() / 100.0)
    );

    k.export_step(&part, &out)?;
    let back = k.import_step(&out)?;
    println!(
        "wrote {} (read back: {:.2} mm³, {} faces)",
        out.display(),
        k.volume(&back)?,
        k.face_count(&back)?
    );
    Ok(())
}
