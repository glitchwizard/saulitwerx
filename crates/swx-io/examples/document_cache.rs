//! Keeps a cache in a part document: a display mesh (`cache/display.glb`)
//! stamped with the model hash and listed in `cache/index.json`. Writes it,
//! reads it back, and shows that an edit makes it stale, that a damaged
//! cache file or index is dropped without an error, and that a bad cache
//! path is refused on write.
//!
//! Used by the user manual ("Keep a cache in a document").
//!
//! Run: `cargo run -p swx-io --example document_cache -- <dir>`

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use swx_io::{
    read_unpacked, write_glb, write_unpacked, Cache, CacheEntry, Document, DocumentKind, Manifest,
    Units,
};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

const GLB: &str = "cache/display.glb";

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;

    let mut doc = Document {
        manifest: Manifest {
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
        },
        model: json!({ "parameters": [{ "name": "thickness", "expr": "10 mm" }] }),
        properties: None,
        attachments: BTreeMap::new(),
        cache: None,
    };

    // A display mesh: a corner cut off a 10 mm cube, as binary glTF.
    let positions = [
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [0.0, 10.0, 0.0],
        [0.0, 0.0, 10.0],
    ];
    let indices = [0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
    let mut glb = Vec::new();
    write_glb(&positions, &indices, &mut glb)?;

    // Stamp it with the hash of the model it shows, and keep it.
    doc.refresh_model_hash();
    let hash = doc.manifest.model_hash.clone().unwrap_or_default();
    let mut cache = Cache::default();
    cache.entries.insert(
        GLB.into(),
        CacheEntry {
            model_hash: hash,
            bytes: glb,
        },
    );
    doc.cache = Some(cache);

    let folder = dir.join("cached.sltwx");
    write_unpacked(&folder, &doc)?;
    println!("wrote {}", folder.display());

    // Read it back: the mesh is there and fresh.
    let back = read_unpacked(&folder)?;
    println!("fresh mesh after reading back: {}", fresh(&back));

    // Change the model: the mesh is still held, but stale.
    let mut edited = back.clone();
    edited.model["parameters"][0]["expr"] = json!("12 mm");
    println!(
        "after an edit: fresh mesh {}, entries kept {}",
        fresh(&edited),
        entries(&edited)
    );

    // Damage one byte of the mesh in a second copy: reading works, the entry
    // is gone.
    let copy = dir.join("damaged.sltwx");
    write_unpacked(&copy, &doc)?;
    let glb_path = copy.join(GLB);
    let mut bytes = std::fs::read(&glb_path)?;
    bytes[20] ^= 0x01;
    std::fs::write(&glb_path, &bytes)?;
    let damaged = read_unpacked(&copy)?;
    println!(
        "damaged mesh: read ok, fresh mesh {}, entries kept {}",
        fresh(&damaged),
        entries(&damaged)
    );

    // Break the index: reading works, and the document has no cache.
    std::fs::write(copy.join("cache/index.json"), "{")?;
    let no_index = read_unpacked(&copy)?;
    println!(
        "broken index: read ok, cache {}",
        if no_index.cache.is_some() {
            "kept"
        } else {
            "none"
        }
    );

    // A cache path that isn't `thumbnail.png` or `cache/<name>` is refused,
    // and nothing is written.
    let mut bad = doc.clone();
    if let Some(c) = bad.cache.as_mut() {
        let entry = c.entries.remove(GLB).expect("inserted above");
        c.entries.insert("cache/meshes/display.glb".into(), entry);
    }
    let target = dir.join("bad.sltwx");
    match write_unpacked(&target, &bad) {
        Ok(()) => println!("bad path -> written"),
        Err(e) => println!("bad path -> error: {e}"),
    }
    println!("{} exists: {}", target.display(), target.exists());
    Ok(())
}

/// Whether the display mesh is there and fresh.
fn fresh(doc: &Document) -> bool {
    doc.fresh_cache_file(GLB).is_some()
}

/// How many cache entries the document holds.
fn entries(doc: &Document) -> usize {
    doc.cache.as_ref().map_or(0, |c| c.entries.len())
}
