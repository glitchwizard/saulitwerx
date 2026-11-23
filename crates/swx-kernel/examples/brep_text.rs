//! Writes the 40 × 25 × 5 mm plate with a 10 mm through hole as BRep text,
//! reads it back, compares the two, and shows the errors bad text gives.
//!
//! Used by the user manual ("Keep a solid's exact shape as text").
//!
//! Run: `cargo run -p swx-kernel --features occt --example brep_text -- plate.brep`

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(
        std::env::args()
            .nth(1)
            .unwrap_or_else(|| "plate.brep".into()),
    );
    let k = OcctKernel;
    let plate = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 5.0])?;
    let tool = k.make_cylinder([20.0, 12.5, -1.0], 5.0, 7.0)?;
    let part = k.cut(&plate, &tool)?;

    let text = k.write_brep(&part)?;
    std::fs::write(&out, &text)?;
    println!(
        "wrote {} ({} bytes, {} lines)",
        out.display(),
        text.len(),
        text.lines().count()
    );
    let tri = text
        .lines()
        .find(|l| l.starts_with("Triangulations"))
        .unwrap_or("(none)");
    println!("{tri}");

    let back = k.read_brep(&std::fs::read_to_string(&out)?)?;
    println!(
        "original:  {:.2} mm³, {} faces",
        k.volume(&part)?,
        k.face_count(&part)?
    );
    println!(
        "read back: {:.2} mm³, {} faces",
        k.volume(&back)?,
        k.face_count(&back)?
    );
    println!("same text again: {}", k.write_brep(&back)? == text);

    for (what, bad) in [("empty", ""), ("not BRep", "not a brep")] {
        let e = k.read_brep(bad).unwrap_err();
        println!("{what} -> error: {e}");
    }
    Ok(())
}
