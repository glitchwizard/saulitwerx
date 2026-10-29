//! Stores expressions in a parameter table: a plate whose hole size and rib
//! spacing follow its width. Shows the order Saulitwerx works them out in,
//! what changes when the width changes or is renamed, and the three ways
//! setting a formula can be refused (a loop, the wrong kind of unit, an
//! unknown name). Ends by typing a plain number over a formula.
//!
//! Used by the user manual ("Make one size follow another").
//!
//! Run: `cargo run -p swx-core --example parameter_formulas`

use swx_core::{Document, FeatureKind};

fn table(doc: &Document) {
    println!("  name   value  unit  formula");
    for p in doc.parameters() {
        let row = format!(
            "  {:<5} {:>6.2}  {:<4}  {}",
            p.name,
            p.value,
            p.unit,
            p.expr.as_deref().unwrap_or("")
        );
        println!("{}", row.trim_end());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let width = doc.add_model_param(40.0, "mm"); // d0
    let hole = doc.add_model_param(0.0, "mm"); // d1
    let rib = doc.add_model_param(0.0, "mm"); // d2
    doc.add_user_param("wall", 2.5, "mm")?;

    // The formulas may name a parameter made later, such as `wall`.
    doc.set_param_expr(&hole, "d0 / 2 + 5 mm")?;
    doc.set_param_expr(&rib, "(d0 - 2 * wall) / 3")?;
    table(&doc);
    println!("worked out in order: {}", doc.evaluation_order().join(", "));

    let sketch = doc.add_feature("Sketch1", FeatureKind::Sketch)?;
    doc.add_feature(
        "Extrude1",
        FeatureKind::Extrude {
            profile: sketch,
            depth: hole.clone(),
        },
    )?;
    let rebuild: Vec<&str> = doc
        .dirty_after_param_change(&width)
        .iter()
        .map(|id| doc.history()[id.0 as usize].name.as_str())
        .collect();
    println!("change d0 -> rebuild {}", rebuild.join(", "));

    doc.set_param(&width, 60.0);
    println!("after d0 = 60:");
    table(&doc);

    doc.rename_param(&width, "width")?;
    println!("after renaming d0 to width:");
    table(&doc);

    for (name, src) in [
        ("width", "d1 - 5 mm"),
        ("d1", "width * wall"),
        ("d1", "height / 2"),
    ] {
        if let Err(e) = doc.set_param_expr(name, src) {
            println!("{name} = {src} -> error: {e}");
        }
    }

    doc.set_param(&hole, 30.0);
    println!("after d1 = 30 (typed as a plain number):");
    table(&doc);
    Ok(())
}
