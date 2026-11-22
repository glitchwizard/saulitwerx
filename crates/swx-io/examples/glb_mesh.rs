//! Writes a small triangle mesh, a corner cut off a 10 mm cube, as a binary
//! glTF 2.0 (GLB) file, reads back its header and the main parts of its JSON chunk, and shows the
//! errors a bad mesh gives.
//!
//! Used by the user manual ("Write a mesh as a glTF file").
//!
//! Run: `cargo run -p swx-io --example glb_mesh -- <dir>`

use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use swx_io::write_glb;

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

    let path = dir.join("corner.glb");
    let mut out = BufWriter::new(File::create(&path)?);
    write_glb(&positions, &indices, &mut out)?;
    drop(out);

    // Read the file back: a 12-byte header, then the JSON chunk.
    let bytes = std::fs::read(&path)?;
    println!("wrote {} ({} bytes)", path.display(), bytes.len());
    let word =
        |at: usize| u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
    println!(
        "header: magic {}, version {}, length {}",
        String::from_utf8_lossy(&bytes[0..4]),
        word(4),
        word(8)
    );
    let json_len = word(12) as usize;
    let json: serde_json::Value = serde_json::from_slice(&bytes[20..20 + json_len])?;
    let bin_at = 20 + json_len;
    println!(
        "chunks: JSON {} bytes, BIN {} bytes",
        json_len,
        word(bin_at)
    );
    // The parts of the JSON chunk worth reading: the node, which scales
    // millimetres to metres, and the corners' accessor, with their bounds.
    println!("node: {}", json["nodes"][0]);
    println!("corners: {}", json["accessors"][0]);
    let index_count = json["accessors"][1]["count"].as_u64().unwrap_or(0);
    println!(
        "triangles: {} indices, {} triangles",
        index_count,
        index_count / 3
    );

    // Bad meshes are refused before anything is written.
    let mut sink = Vec::new();
    let tries: [Try; 4] = [
        ("half a triangle", &positions, &[0, 1]),
        ("no such corner", &positions, &[0, 1, 7]),
        (
            "not a number",
            &[[0.0, 0.0, 0.0], [f32::NAN, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &[0, 1, 2],
        ),
        ("no triangles", &positions, &[]),
    ];
    for (what, p, i) in tries {
        let e = write_glb(p, i, &mut sink).unwrap_err();
        println!("{what} -> error: {e}");
    }
    println!("bytes written by the failed tries: {}", sink.len());
    Ok(())
}
