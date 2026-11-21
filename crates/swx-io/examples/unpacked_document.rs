//! Writes a small part document in the unpacked folder form
//! (`<dir>/plate.sltwx/`), reads it back, prints its manifest, and shows the
//! errors reading and writing can give.
//!
//! Used by the user manual ("Saulitwerx document files").
//!
//! Run: `cargo run -p swx-io --example unpacked_document -- <dir>`

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use swx_io::{
    read_manifest, read_unpacked, write_unpacked, Document, DocumentKind, Manifest, Units,
};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;

    // The manifest says what the document is. Its id is fixed for the
    // document's life.
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
    // model.json and properties.json are plain JSON objects for now.
    let doc = Document {
        manifest,
        model: json!({ "parameters": [{ "name": "thickness", "expr": "10 mm" }] }),
        properties: Some(json!({ "part_number": "PL-001", "material": "aluminum" })),
    };

    let folder = dir.join("plate.sltwx");
    write_unpacked(&folder, &doc)?;
    println!("wrote {}", folder.display());
    let mut names: Vec<String> = std::fs::read_dir(&folder)?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<_, _>>()?;
    names.sort();
    println!("files: {}", names.join(", "));

    let back = read_unpacked(&folder)?;
    let m = &back.manifest;
    println!(
        "read back: {} {} {}, units {} / {} / {}",
        m.kind.as_str(),
        m.format_version,
        m.document_id,
        m.units.length,
        m.units.angle,
        m.units.mass
    );
    println!("same as written: {}", back == doc);
    print!(
        "manifest.json:\n{}",
        std::fs::read_to_string(folder.join("manifest.json"))?
    );

    // The errors: a part saved under an assembly's extension, a folder that
    // doesn't exist, a manifest from a future major version, and a model
    // that is not valid JSON.
    let wrong = dir.join("plate.sltwxa");
    println!(
        "wrong extension -> error: {}",
        write_unpacked(&wrong, &doc).unwrap_err()
    );
    let missing = dir.join("missing.sltwx");
    println!(
        "missing folder -> error: {}",
        read_unpacked(&missing).unwrap_err()
    );
    let future =
        std::fs::read_to_string(folder.join("manifest.json"))?.replace("\"0.1.0\"", "\"1.0.0\"");
    println!(
        "version 1.0.0 -> error: {}",
        read_manifest(&future).unwrap_err()
    );
    let broken = dir.join("broken.sltwx");
    write_unpacked(&broken, &doc)?;
    std::fs::write(broken.join("model.json"), "{ \"features\": [ }")?;
    println!(
        "bad model -> error: {}",
        read_unpacked(&broken).unwrap_err()
    );
    Ok(())
}
