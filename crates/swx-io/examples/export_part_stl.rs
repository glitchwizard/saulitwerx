//! Builds a disc 40 mm across and 8 mm thick and saves it as
//! STL in one step with `export_stl`: binary at the default deflection, binary
//! at a coarse deflection, and ASCII. Then shows the deflection error.
//!
//! Used by the user manual ("Save a part as an STL file").
//!
//! Run: `cargo run -p swx-io --features occt --example export_part_stl -- <dir>`

use std::path::{Path, PathBuf};

use swx_io::{export_stl, StlFormat, StlOptions};
use swx_kernel::{Kernel, OcctKernel};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // A round disc, 40 mm across and 8 mm thick: its curved side shows how
    // fine the mesh is.
    let part = k.make_cylinder([0.0, 0.0, 0.0], 20.0, 8.0)?;
    println!("part volume {:.2} mm³", k.volume(&part)?);

    // The defaults: 0.01 mm, binary.
    let fine = dir.join("disc.stl");
    export_stl(&k, &part, &fine, &StlOptions::default())?;
    report(&fine)?;

    // A coarser mesh: fewer, larger triangles.
    let coarse = dir.join("disc-coarse.stl");
    let options = StlOptions {
        deflection_mm: 1.0,
        format: StlFormat::Binary,
    };
    export_stl(&k, &part, &coarse, &options)?;
    report(&coarse)?;

    // The same fine mesh as text.
    let text = dir.join("disc-text.stl");
    let options = StlOptions {
        deflection_mm: 0.01,
        format: StlFormat::Ascii,
    };
    export_stl(&k, &part, &text, &options)?;
    let first = std::fs::read_to_string(&text)?
        .lines()
        .next()
        .unwrap_or_default()
        .to_string();
    println!(
        "wrote {} ({} bytes), first line: {first}",
        text.display(),
        std::fs::metadata(&text)?.len()
    );

    // A deflection of 0 is refused before anything is made.
    let bad = dir.join("disc-bad.stl");
    let options = StlOptions {
        deflection_mm: 0.0,
        ..StlOptions::default()
    };
    let e = export_stl(&k, &part, &bad, &options).unwrap_err();
    println!("deflection 0 -> error: {e}");
    println!("{} exists: {}", bad.display(), bad.exists());
    Ok(())
}

/// Prints a binary STL file's size and triangle count (84 + 50 per triangle).
fn report(path: &Path) -> Res<()> {
    let len = std::fs::metadata(path)?.len();
    println!(
        "wrote {} ({} bytes, {} triangles)",
        path.display(),
        len,
        (len - 84) / 50
    );
    Ok(())
}
