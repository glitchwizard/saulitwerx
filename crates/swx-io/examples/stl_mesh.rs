//! Writes a small triangle mesh, a corner cut off a 10 mm cube, as an ASCII
//! and a binary STL file, and shows the errors a bad mesh gives.
//!
//! Used by the user manual ("Write a mesh as an STL file").
//!
//! Run: `cargo run -p swx-io --example stl_mesh -- <dir>`

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use swx_io::{write_stl_ascii, write_stl_binary};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// A labelled mesh to try: what is wrong with it, its corners and its triangles.
type Try<'a> = (&'a str, &'a [[f32; 3]], &'a [u32]);

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;

    // Four corners, in millimetres.
    let positions = [
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, 10.0],
    ];
    // Four triangles, three corners each, counter-clockwise seen from outside.
    let indices = [0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];

    let ascii = dir.join("corner.stl");
    let mut out = BufWriter::new(File::create(&ascii)?);
    write_stl_ascii("corner", &positions, &indices, &mut out)?;
    drop(out);
    println!("wrote {} ({} bytes)", ascii.display(), size(&ascii)?);

    let binary = dir.join("corner-binary.stl");
    let mut out = BufWriter::new(File::create(&binary)?);
    write_stl_binary(&positions, &indices, &mut out)?;
    drop(out);
    println!("wrote {} ({} bytes)", binary.display(), size(&binary)?);

    // Bad meshes are refused before anything is written.
    let mut sink = Vec::new();
    let tries: [Try; 3] = [
        ("half a triangle", &positions, &[0, 1]),
        ("no such corner", &positions, &[0, 1, 7]),
        (
            "not a number",
            &[[0.0, 0.0, 0.0], [f32::NAN, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
        ),
    ];
    for (what, p, i) in tries {
        let e = write_stl_binary(p, i, &mut sink).unwrap_err();
        println!("{what} -> error: {e}");
    }
    println!("bytes written by the failed tries: {}", sink.len());
    Ok(())
}

fn size(path: &std::path::Path) -> Res<u64> {
    Ok(std::fs::metadata(path)?.len())
}
