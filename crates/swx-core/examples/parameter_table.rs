//! Builds a part's parameter table: model parameters named `d0`, `d1`, … in
//! turn, a user parameter with a comment, and a rename that the feature using
//! the parameter follows. Ends with the two errors a bad name gives.
//!
//! Used by the user manual ("Name and rename parameters").
//!
//! Run: `cargo run -p swx-core --example parameter_table`

use swx_core::{Document, FeatureKind, ParamKind};

/// Prints the parameter table in creation order.
fn print_table(doc: &Document) {
    println!("  name       kind   value  unit  comment");
    for p in doc.parameters() {
        let kind = match p.kind {
            ParamKind::Model => "model",
            ParamKind::User => "user",
        };
        println!(
            "  {:<10} {:<5} {:>6}  {:<4}  {}",
            p.name, kind, p.value, p.unit, p.comment
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();

    // Three dimensions of the plate get the next free model names.
    let depth = doc.add_model_param(5.0, "mm");
    let width = doc.add_model_param(40.0, "mm");
    let hole = doc.add_model_param(10.0, "mm");
    println!("model parameters: {depth}, {width}, {hole}");

    let sketch = doc.add_feature("Sketch1", FeatureKind::Sketch)?;
    let extrude = doc.add_feature(
        "Extrude1",
        FeatureKind::Extrude {
            profile: sketch,
            depth: depth.clone(),
        },
    )?;

    // A value of your own, with a note on what it is for.
    doc.add_user_param("clearance", 0.2, "mm")?;
    doc.set_param_comment("clearance", "gap around the pin")?;

    // Give `d0` a meaningful name. The extrude follows the rename.
    doc.rename_param(&depth, "thickness")?;
    println!("after renaming {depth} to thickness:");
    print_table(&doc);
    for f in doc.history().iter().filter(|f| f.id == extrude) {
        if let FeatureKind::Extrude { depth, .. } = &f.kind {
            println!("Extrude1 depth reads `{depth}`");
        }
    }

    // The freed name `d0` is not handed out again.
    println!("next model parameter: {}", doc.add_model_param(2.0, "mm"));

    // Two names that are refused.
    if let Err(e) = doc.add_user_param("2nd_hole", 6.0, "mm") {
        println!("add `2nd_hole` -> error: {e}");
    }
    if let Err(e) = doc.rename_param(&width, "thickness") {
        println!("rename {width} to thickness -> error: {e}");
    }
    Ok(())
}
