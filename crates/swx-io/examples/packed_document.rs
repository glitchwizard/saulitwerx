//! Writes a small part document as a single `.sltwx` file (the packed ZIP
//! form), reads it back, turns a document folder into a file, and shows the
//! errors reading a damaged file can give.
//!
//! Used by the user manual ("Saulitwerx document files").
//!
//! Run: `cargo run -p swx-io --example packed_document -- <dir>`

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use swx_io::{
    read_sltwx, read_unpacked, write_sltwx, write_unpacked, Document, DocumentKind, Manifest, Units,
};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;

    let manifest = Manifest {
        format: "sltwx".into(),
        format_version: "0.1.0".into(),
        kind: DocumentKind::Part,
        document_id: "0b6d3a52-4c1e-4f7a-9e21-5d8c7f3b2a10".into(),
        units: Units {
            length: "mm".into(),
            angle: "deg".into(),
            mass: "kg".into(),
            extra: BTreeMap::new(),
        },
        created_by: "saulitwerx 0.0.1".into(),
        saved_by: "saulitwerx 0.0.1".into(),
        model_hash: None,
        extra: BTreeMap::new(),
    };
    let doc = Document {
        manifest,
        model: json!({ "parameters": [{ "name": "thickness", "expr": "10 mm" }] }),
        properties: Some(json!({ "part_number": "PL-001", "material": "aluminum" })),
    };

    // One file: write it, read it back.
    let file = dir.join("plate.sltwx");
    write_sltwx(&file, &doc)?;
    let bytes = std::fs::read(&file)?;
    println!("wrote {} ({} bytes)", file.display(), bytes.len());
    let back = read_sltwx(&file)?;
    println!("same as written: {}", back == doc);

    // The same document written again gives exactly the same bytes.
    let again = dir.join("plate-again.sltwx");
    write_sltwx(&again, &doc)?;
    println!(
        "written twice, identical: {}",
        std::fs::read(&again)? == bytes
    );

    // From a folder to a file: read the unpacked form, write the packed one.
    let folder = dir.join("folder").join("plate.sltwx");
    write_unpacked(&folder, &doc)?;
    let packed = dir.join("from-folder.sltwx");
    write_sltwx(&packed, &read_unpacked(&folder)?)?;
    println!(
        "folder {} -> file {}, identical to plate.sltwx: {}",
        folder.display(),
        packed.display(),
        std::fs::read(&packed)? == bytes
    );

    // The errors: a text file under a document's name, a file cut short, one
    // damaged byte inside the compressed model, and a part saved under an
    // assembly's extension.
    let text = dir.join("text.sltwx");
    std::fs::write(&text, "not a zip")?;
    println!("not a zip -> error: {}", read_sltwx(&text).unwrap_err());
    let cut = dir.join("cut.sltwx");
    std::fs::write(&cut, &bytes[..bytes.len() - 60])?;
    println!("cut short -> error: {}", read_sltwx(&cut).unwrap_err());
    let damaged = dir.join("damaged.sltwx");
    let mut flipped = bytes.clone();
    let at = model_data_offset(&bytes).ok_or("model.json not found")?;
    // Byte 40 of the compressed data: the stream still decompresses, but to
    // different text, which only the CRC-32 check catches.
    flipped[at + 40] ^= 0xff;
    std::fs::write(&damaged, &flipped)?;
    println!("damaged -> error: {}", read_sltwx(&damaged).unwrap_err());
    let wrong = dir.join("plate.sltwxa");
    println!(
        "wrong extension -> error: {}",
        write_sltwx(&wrong, &doc).unwrap_err()
    );
    Ok(())
}

/// The offset of the first byte of `model.json`'s compressed data: just
/// after its local file header (30 bytes plus the name, with no extra field
/// in files this crate writes; PKWARE APPNOTE 6.3.x, section 4.3.7).
fn model_data_offset(bytes: &[u8]) -> Option<usize> {
    let name = b"model.json";
    let at = bytes.windows(name.len()).position(|w| w == name)?;
    Some(at + name.len())
}
