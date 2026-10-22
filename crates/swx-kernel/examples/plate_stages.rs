//! Writes the stages of `plate_with_hole` as separate STEP files, so the
//! user manual can show "block − cylinder = plate" ("Concepts: features").
//!
//! Run: `cargo run -p swx-kernel --features occt --example plate_stages -- <out-dir>`
//!
//! Writes `<out-dir>/plate-block.step`, `plate-tool.step` and `plate-cut.step`.

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    let k = OcctKernel;
    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 5.0])?;
    let tool = k.make_cylinder([20.0, 12.5, -1.0], 5.0, 7.0)?;
    let cut = k.cut(&block, &tool)?;
    for (name, shape) in [
        ("plate-block", &block),
        ("plate-tool", &tool),
        ("plate-cut", &cut),
    ] {
        let path = dir.join(format!("{name}.step"));
        k.export_step(shape, &path)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
