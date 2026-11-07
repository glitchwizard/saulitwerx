//! Moves the end-of-part marker of a four-feature history: rolls it back
//! above a sketch, inserts two features there, shows what a rolled-back
//! feature can't do, then rolls forward and lists what must be rebuilt.
//!
//! Used by the user manual ("Roll back and insert a feature").
//!
//! Run: `cargo run -p swx-core --example roll_back`

use swx_core::{Document, FeatureId, FeatureKind};

/// The history, one feature per line, with the end-of-part marker where it is.
fn show(doc: &Document) {
    for (pos, f) in doc.history().iter().enumerate() {
        if pos == doc.end_of_part() {
            println!("  -- end of part --");
        }
        println!("  {pos}  {}", f.name);
    }
    if doc.end_of_part() == doc.history().len() {
        println!("  -- end of part --");
    }
}

/// The names of `ids`, looked up in the history (ids and positions differ
/// once a feature has been inserted).
fn names(doc: &Document, ids: &[FeatureId]) -> String {
    let list: Vec<&str> = ids
        .iter()
        .filter_map(|id| doc.history().iter().find(|f| f.id == *id))
        .map(|f| f.name.as_str())
        .collect();
    if list.is_empty() {
        "nothing".into()
    } else {
        list.join(", ")
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    doc.set_param("thickness", 5.0);
    doc.set_param("sweep", 360.0);
    doc.set_param("boss", 12.0);

    let plate_sketch = doc.add_feature("Sketch1", FeatureKind::Sketch)?;
    doc.add_feature(
        "Extrude1",
        FeatureKind::Extrude {
            profile: plate_sketch,
            depth: "thickness".into(),
        },
    )?;
    let pin_sketch = doc.add_feature("Sketch2", FeatureKind::Sketch)?;
    doc.add_feature(
        "Revolve1",
        FeatureKind::Revolve {
            profile: pin_sketch,
            angle_deg: "sweep".into(),
        },
    )?;
    println!("history:");
    show(&doc);

    // Roll back: only the first two features stay active.
    let rebuilt = doc.set_end_of_part(2)?;
    println!("roll back to 2 -> rebuild {}", names(&doc, &rebuilt));
    println!("rolled back:");
    show(&doc);
    println!(
        "change sweep -> rebuild {}",
        names(&doc, &doc.dirty_after_param_change("sweep"))
    );

    // New features go in at the marker, above the rolled-back ones.
    let boss_sketch = doc.add_feature("Sketch3", FeatureKind::Sketch)?;
    doc.add_feature(
        "Extrude2",
        FeatureKind::Extrude {
            profile: boss_sketch,
            depth: "boss".into(),
        },
    )?;
    println!("after inserting Sketch3 and Extrude2:");
    show(&doc);

    // A new feature can't use a feature below the marker.
    let err = doc
        .add_feature(
            "Extrude3",
            FeatureKind::Extrude {
                profile: pin_sketch,
                depth: "thickness".into(),
            },
        )
        .unwrap_err();
    println!("add Extrude3 from Sketch2 -> error: {err}");

    // Roll forward to the end: the features that come back must be rebuilt.
    let end = doc.history().len();
    let rebuilt = doc.set_end_of_part(end)?;
    println!("roll forward to {end} -> rebuild {}", names(&doc, &rebuilt));
    println!(
        "edit Sketch1 -> rebuild {}",
        names(&doc, &doc.dirty_after_feature_edit(plate_sketch)?)
    );

    let err = doc.set_end_of_part(9).unwrap_err();
    println!("roll forward to 9 -> error: {err}");
    Ok(())
}
