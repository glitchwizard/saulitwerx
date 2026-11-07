//! Cuts a 10 mm wide, 5 mm deep groove across a 40 × 25 × 10 mm block with
//! history, and prints what became of every face of the block and of the
//! cutting box: kept, split into several faces, or deleted. Then finds the
//! groove's floor through the history and writes the part to a STEP file.
//!
//! Used by the user manual ("Find the faces a Boolean made").
//!
//! Run: `cargo run -p swx-kernel --features occt --example boolean_history -- <out.step>`

use swx_kernel::{BooleanOp, BoundingBox, Kernel, OcctKernel, Operand};

/// Rounds a box to 0.01 mm and prints it as `x a..b  y c..d  z e..f`.
fn span(b: &BoundingBox) -> String {
    let r = |v: f64| (v * 100.0).round() / 100.0 + 0.0;
    format!(
        "x {}..{}  y {}..{}  z {}..{}",
        r(b.min[0]),
        r(b.max[0]),
        r(b.min[1]),
        r(b.max[1]),
        r(b.min[2]),
        r(b.max[2])
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "groove.step".into());
    let k = OcctKernel;

    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    // The tool: 10 mm wide in x, longer than the block in y, from z = 5 to
    // 1 mm above the top, so the groove runs right across.
    let tool = k.make_box([15.0, -1.0, 5.0], [10.0, 27.0, 6.0])?;

    let (part, history) = k.boolean_tracked(BooleanOp::Cut, &block, &tool)?;
    println!(
        "block {} faces, tool {} faces, part {} faces",
        k.face_count(&block)?,
        k.face_count(&tool)?,
        k.face_count(&part)?
    );

    let mut floor = None;
    for (operand, face, images) in &history.modified {
        let (name, input) = match operand {
            Operand::A => ("block", &block),
            Operand::B => ("tool ", &tool),
        };
        let what = match images.len() {
            0 => "deleted".to_string(),
            1 => "one face".to_string(),
            n => format!("split in {n}"),
        };
        let list: Vec<usize> = images.iter().map(|f| f.index).collect();
        println!(
            "{name} face {}  {:<28} {what:<10} -> {list:?}",
            face.index,
            span(&k.sub_shape_bounding_box(input, *face)?)
        );
        // The tool's bottom (flat at z = 5) becomes the groove's floor.
        let b = k.sub_shape_bounding_box(input, *face)?;
        if *operand == Operand::B && b.max[2] - b.min[2] < 1e-6 && b.min[2] == 5.0 {
            floor = images.first().copied();
        }
    }

    if let Some(f) = floor {
        println!(
            "groove floor is part face {}  {}",
            f.index,
            span(&k.sub_shape_bounding_box(&part, f)?)
        );
    }
    println!("part volume {:.2} mm³", k.volume(&part)?);
    k.export_step(&part, std::path::Path::new(&out))?;
    println!("wrote {out}");
    Ok(())
}
