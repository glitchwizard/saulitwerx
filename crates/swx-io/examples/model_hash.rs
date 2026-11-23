//! Records a part document's `model_hash`, checks it against the written
//! `model.json`, and shows how a cache built from that hash turns stale when
//! the model changes.
//!
//! Used by the user manual ("Tell whether a cache is up to date").
//!
//! Run: `cargo run -p swx-io --example model_hash -- <dir>`

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use swx_io::{read_unpacked, sha256, write_unpacked, Document, DocumentKind, Manifest, Units};

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
    // The bodies are out of ID order; the writers sort them.
    let mut doc = Document {
        manifest,
        model: json!({
            "bodies": [{ "id": "body-2" }, { "id": "body-1" }],
            "parameters": [{ "name": "thickness", "expr": "10 mm" }]
        }),
        properties: None,
    };

    // Record the hash of the model.json the writers will write.
    doc.refresh_model_hash();
    let recorded = doc.manifest.model_hash.clone().unwrap_or_default();
    println!("recorded in the manifest:\n  {recorded}");

    let folder = dir.join("hashed.sltwx");
    write_unpacked(&folder, &doc)?;
    let bytes = std::fs::read(folder.join("model.json"))?;
    let digest: String = sha256(&bytes).iter().map(|b| format!("{b:02x}")).collect();
    println!("hash of the written model.json:\n  sha256:{digest}");

    // A cache stamped with `recorded` is fresh until the model changes.
    let back = read_unpacked(&folder)?;
    println!(
        "fresh after reading back:   {}",
        back.cache_is_fresh(&recorded)
    );
    let mut edited = back.clone();
    edited.model["parameters"][0]["expr"] = json!("12 mm");
    println!(
        "fresh after an edit:        {}",
        edited.cache_is_fresh(&recorded)
    );
    println!(
        "fresh for a malformed hash: {}",
        back.cache_is_fresh("sha256:xyz")
    );
    Ok(())
}
