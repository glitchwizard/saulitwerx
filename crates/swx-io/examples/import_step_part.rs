//! Brings a STEP file into a part document as an imported solid: makes a
//! block with a hole and saves it as STEP (standing in for a file from
//! another program), embeds it with `embed_step`, saves the part as one
//! file, reads it back and rebuilds the solid with `rebuild_imported`. Then
//! shows the import and rebuild errors.
//!
//! Used by the user manual ("Bring a STEP file into a part").
//!
//! Run: `cargo run -p swx-io --features occt --example import_step_part -- <dir>`

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde_json::json;
use swx_io::{
    embed_step, read_sltwx, rebuild_imported, to_canonical_json, write_sltwx, Document,
    DocumentKind, Manifest, Units,
};
use swx_kernel::{Kernel, OcctKernel};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // The file from "another program": a 40 x 25 x 10 mm block with a hole
    // 8 mm across straight through it.
    let block = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 10.0])?;
    let hole = k.make_cylinder([20.0, 12.5, 0.0], 4.0, 10.0)?;
    let block = k.cut(&block, &hole)?;
    let step = dir.join("block.step");
    k.export_step(&block, &step)?;
    println!("wrote {}: {:.2} mm³", step.display(), k.volume(&block)?);

    // A new, empty part, and the file brought into it.
    let mut doc = new_part();
    let feature = embed_step(&k, &mut doc, &step)?;
    println!(
        "imported as feature {} from {}",
        feature.name, feature.original_name
    );
    println!(
        "attachments: {:?}",
        doc.attachments.keys().collect::<Vec<_>>()
    );
    println!("model.json:\n{}", to_canonical_json(&doc.model));

    // Save the part as one file, read it back, and rebuild the solid from
    // the copy inside it.
    let file = dir.join("bracket.sltwx");
    write_sltwx(&file, &doc)?;
    let back = read_sltwx(&file)?;
    let solid = rebuild_imported(&k, &back, &feature.id)?;
    let bb = k.bounding_box(&solid)?;
    println!(
        "rebuilt from {}: {:.2} mm³, {:.0} x {:.0} x {:.0} mm",
        file.display(),
        k.volume(&solid)?,
        bb.max[0] - bb.min[0],
        bb.max[1] - bb.min[1],
        bb.max[2] - bb.min[2]
    );
    k.export_step(&solid, &dir.join("rebuilt.step"))?;

    // Errors when importing: nothing is added to the part.
    let text = std::fs::read_to_string(&step)?;
    let notes = dir.join("notes.txt");
    std::fs::write(&notes, "Bracket for the shelf, rev B\n")?;
    let cut_short = dir.join("cut-short.step");
    std::fs::write(&cut_short, &text[..text.len() / 2])?;
    for (label, path) in [
        ("missing file", dir.join("missing.step")),
        ("not STEP", notes),
        ("cut short", cut_short),
    ] {
        let mut part = new_part();
        match embed_step(&k, &mut part, &path) {
            Ok(_) => println!("{label} -> imported"),
            Err(e) => println!("{label} -> error: {e}"),
        }
    }

    // Errors when rebuilding: the part still opens; the feature fails.
    let name = format!("{}.step", feature.id);
    let mut damaged = back.clone();
    if let Some(bytes) = damaged.attachments.get_mut(&name) {
        bytes[200] ^= 0x01;
    }
    report("damaged copy", rebuild_imported(&k, &damaged, &feature.id));
    let mut missing = back.clone();
    missing.attachments.remove(&name);
    report("missing copy", rebuild_imported(&k, &missing, &feature.id));
    Ok(())
}

/// A part document with one parameter and nothing else in its model.
fn new_part() -> Document {
    Document {
        manifest: Manifest {
            format: "sltwx".into(),
            format_version: "0.1.0".into(),
            kind: DocumentKind::Part,
            document_id: "6f1c2e8a-93b4-4d2a-8c55-0e7a41d9b3f2".into(),
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
        model: json!({}),
        properties: None,
        attachments: BTreeMap::new(),
        cache: None,
    }
}

/// Prints a rebuild's outcome.
fn report<S>(label: &str, r: Result<S, swx_io::StepImportError>) {
    match r {
        Ok(_) => println!("{label} -> rebuilt"),
        Err(e) => println!("{label} -> error: {e}"),
    }
}
