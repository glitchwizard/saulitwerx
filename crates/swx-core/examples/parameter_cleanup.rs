//! Tidies a parameter table: changes the units a plate's sizes are shown
//! in, deletes a user parameter nothing reads, and shows the four ways a
//! delete or a unit change can be refused.
//!
//! Used by the user manual ("Tidy the parameter table").
//!
//! Run: `cargo run -p swx-core --example parameter_cleanup`

use swx_core::{Document, FeatureKind};

fn table(doc: &Document) {
    println!("  name    value     unit  formula");
    for p in doc.parameters() {
        let row = format!(
            "  {:<6} {:>9.4}  {:<4}  {}",
            p.name,
            p.value,
            p.unit,
            p.expr.as_deref().unwrap_or("")
        );
        println!("{}", row.trim_end());
    }
}

fn report(what: &str, result: Result<(), swx_core::ModelError>) {
    match result {
        Ok(()) => println!("{what} -> ok"),
        Err(e) => println!("{what} -> error: {e}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let width = doc.add_model_param(50.8, "mm"); // d0
    doc.add_user_param("depth", 6.0, "mm")?;
    doc.add_user_param("half", 0.0, "mm")?;
    doc.set_param_expr("half", "d0 / 2")?;
    doc.add_user_param("spare", 3.0, "mm")?;
    doc.add_user_param("tilt", 90.0, "deg")?;
    let sketch = doc.add_feature("Sketch1", FeatureKind::Sketch)?;
    doc.add_feature(
        "Extrude1",
        FeatureKind::Extrude {
            profile: sketch,
            depth: "depth".into(),
        },
    )?;
    println!("start:");
    table(&doc);

    // Show the width in inches and the tilt in radians. The sizes they
    // stand for stay the same.
    doc.set_param_unit(&width, "in")?;
    doc.set_param_unit("tilt", "rad")?;
    // A parameter with a formula keeps it, and is worked out in its new unit.
    doc.set_param_unit("half", "cm")?;
    println!("after the unit changes:");
    table(&doc);

    // Nothing reads `spare`, so it can go.
    doc.delete_param("spare")?;
    println!("after deleting spare:");
    table(&doc);

    // Refused, each leaving the table as it was.
    report("delete d0", doc.delete_param(&width));
    report("delete depth", doc.delete_param("depth"));
    report("depth in furlong", doc.set_param_unit("depth", "furlong"));
    report("depth in deg", doc.set_param_unit("depth", "deg"));
    Ok(())
}
