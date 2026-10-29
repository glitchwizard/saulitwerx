//! Works out parameter expressions with units, such as `d1 / 2 + 5 mm`,
//! looking names up in a document's parameter table. Ends with one example
//! of each error an expression can give.
//!
//! Used by the user manual ("Work out an expression").
//!
//! Run: `cargo run -p swx-core --example expressions`

use swx_core::expr::{eval, parse, Quantity, Unit};
use swx_core::Document;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    doc.add_model_param(5.0, "mm"); // d0
    doc.add_model_param(40.0, "mm"); // d1
    doc.add_user_param("wall", 2.5, "mm")?;
    doc.add_user_param("angle", 30.0, "deg")?;

    // Parameters are stored as plain numbers with a unit label; the
    // expression engine works in millimeters and radians.
    let resolve = |name: &str| -> Option<Quantity> {
        let p = doc.parameters().iter().find(|p| p.name == name)?;
        let unit = Unit::from_suffix(&p.unit)?;
        Some(Quantity::new(p.value * unit.factor(), unit.dim()))
    };

    for src in [
        "d1 / 2 + 5 mm",
        "(d1 - 2 * wall) / 3",
        "2 in + d0",
        "90 deg - angle",
        "d1 * d0",
        "d1 / d0",
    ] {
        let q = eval(&parse(src)?, &resolve, Unit::Mm)?;
        let shown = match (q.dim.length, q.dim.angle) {
            (0, 1) => format!("{:.2} deg", q.in_unit(Unit::Deg)),
            (1, 0) => format!("{:.2} mm", q.value),
            (2, 0) => format!("{:.2} mm²", q.value),
            _ => format!("{:.2} ({})", q.value, q.dim),
        };
        println!("{src:<20} = {shown}");
    }

    for src in [
        "d1 + 5 deg",
        "5 mmm",
        "d9 * 2",
        "d1 / (d0 - 5 mm)",
        "d1 / (2",
    ] {
        let result = parse(src).and_then(|e| eval(&e, &resolve, Unit::Mm));
        if let Err(e) = result {
            println!("{src}\n  error: {e}");
        }
    }
    Ok(())
}
