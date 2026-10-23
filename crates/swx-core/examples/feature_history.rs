//! Builds a small feature history and shows which features a change would
//! rebuild.
//!
//! Used by the user manual ("See what a change rebuilds").
//!
//! Run: `cargo run -p swx-core --example feature_history`

use swx_core::{Document, FeatureId, FeatureKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    doc.set_param("thickness", 5.0);
    doc.set_param("sweep", 360.0);

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
    for f in doc.history() {
        println!("  {}  {}", f.id.0, f.name);
    }

    doc.set_param("thickness", 8.0);
    let names = |ids: Vec<FeatureId>| {
        ids.iter()
            .map(|id| doc.history()[id.0 as usize].name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!(
        "change thickness -> rebuild {}",
        names(doc.dirty_after_param_change("thickness"))
    );
    println!(
        "edit Sketch2     -> rebuild {}",
        names(doc.dirty_after_feature_edit(pin_sketch)?)
    );

    // A feature can only use parameters that exist.
    let err = doc
        .add_feature(
            "Extrude2",
            FeatureKind::Extrude {
                profile: plate_sketch,
                depth: "height".into(),
            },
        )
        .unwrap_err();
    println!("add Extrude2 with depth `height` -> error: {err}");
    Ok(())
}
