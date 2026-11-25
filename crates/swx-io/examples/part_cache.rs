//! Builds a part's cache from its solved bodies: a plate with a hole and a
//! separate pin, each kept as `cache/body-<id>.brep`, plus one merged
//! display mesh, `cache/display.glb`. Writes the part as a folder, reads
//! it back, loads each body from its BRep, shows that an edit makes the
//! bodies stale, and shows the two mistakes `build_part_cache` refuses.
//!
//! Used by the user manual ("Build a part's cache from its bodies").
//!
//! Run: `cargo run -p swx-io --features occt --example part_cache -- <dir>`

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use swx_io::{
    build_part_cache, load_body, read_unpacked, write_unpacked, Document, DocumentKind, Manifest,
    Units, DISPLAY_DEFLECTION_MM,
};
use swx_kernel::{Kernel, OcctKernel};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    let mut doc = Document {
        manifest: Manifest {
            format: "sltwx".into(),
            format_version: "0.1.0".into(),
            kind: DocumentKind::Part,
            document_id: "7c1e9a40-2b5d-4e8f-a613-0f4d2c9b8e71".into(),
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
        model: json!({
            "parameters": [{ "name": "thickness", "expr": "10 mm" }],
            "bodies": [{ "id": "plate" }, { "id": "pin" }],
        }),
        properties: None,
        attachments: BTreeMap::new(),
        cache: None,
    };

    // The part's two bodies: a plate with a hole, and a pin beside it.
    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    let drill = k.make_cylinder([20.0, 12.5, -1.0], 5.0, 12.0)?;
    let plate = k.cut(&block, &drill)?;
    let pin = k.make_cylinder([55.0, 12.5, 0.0], 4.5, 20.0)?;
    let bodies = [("plate".to_owned(), &plate), ("pin".to_owned(), &pin)];

    doc.refresh_model_hash();
    build_part_cache(&k, &mut doc, &bodies, DISPLAY_DEFLECTION_MM)?;
    let cache = doc.cache.as_ref().expect("built");
    for (path, entry) in &cache.entries {
        println!("{path:<24} {:>6} bytes", entry.bytes.len());
    }

    let folder = dir.join("pinned-plate.sltwx");
    write_unpacked(&folder, &doc)?;
    println!("wrote {}", folder.display());

    let back = read_unpacked(&folder)?;
    for id in ["plate", "pin"] {
        let body = load_body(&k, &back, id).expect("fresh body");
        println!(
            "loaded {id:<5} {:>9.2} mm³, {} faces",
            k.volume(&body)?,
            k.face_count(&body)?
        );
    }
    println!("loaded bolt: {}", load_body(&k, &back, "bolt").is_some());

    let mut edited = back.clone();
    edited.model["parameters"][0]["expr"] = json!("12 mm");
    println!(
        "after an edit: plate {}, display mesh fresh {}",
        if load_body(&k, &edited, "plate").is_some() {
            "loaded"
        } else {
            "stale"
        },
        edited.fresh_cache_file("cache/display.glb").is_some()
    );

    let mut scratch = back.clone();
    for (label, r) in [
        (
            "deflection 0",
            build_part_cache(&k, &mut scratch, &bodies, 0.0),
        ),
        (
            "body a/b",
            build_part_cache(
                &k,
                &mut scratch,
                &[("a/b".to_owned(), &pin)],
                DISPLAY_DEFLECTION_MM,
            ),
        ),
    ] {
        println!("{label} -> error: {}", r.unwrap_err());
    }
    println!("cache unchanged: {}", scratch.cache == back.cache);
    Ok(())
}
