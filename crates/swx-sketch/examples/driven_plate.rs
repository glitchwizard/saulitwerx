//! Drives a sketch from the parameter table: a 40 × 25 mm rectangle whose
//! width and height are bound to model parameters `d0` and `d1`. Renames
//! `d0` to `width`, changes it, and solves again; then shows the two
//! mistakes the manual warns about (a rename the sketch wasn't told about,
//! and a value of zero).
//!
//! Used by the user manual ("Drive a sketch with parameters").
//!
//! Run: `cargo run -p swx-sketch --example driven_plate`

use swx_core::Document;
use swx_sketch::params::{apply_params, bind_dimension, rename_bound};
use swx_sketch::solver::SolveOptions;
use swx_sketch::{Constraint, GeomId, Geometry, Line, Point, PointRef, Sketch};

/// Width and height of the rectangle, from its bottom and right sides.
fn size(s: &Sketch, bottom: GeomId, right: GeomId) -> Result<String, Box<dyn std::error::Error>> {
    let len = |id| -> Result<f64, Box<dyn std::error::Error>> {
        match s.get(id)? {
            Geometry::Line(l) => Ok(l.length()),
            _ => Err("not a line".into()),
        }
    };
    Ok(format!("{:.2} × {:.2} mm", len(bottom)?, len(right)?))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = Document::new();
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);

    // A rough rectangle, joined at the corners, level and upright.
    let corners = [(0.0, 0.0), (38.0, 1.0), (39.0, 24.0), (1.0, 23.0)];
    let mut sides = Vec::new();
    for i in 0..4 {
        let (a, b) = (corners[i], corners[(i + 1) % 4]);
        sides.push(s.add(Geometry::Line(Line::new(p(a.0, a.1)?, p(b.0, b.1)?)?))?);
    }
    let [bottom, right, top, left] = [sides[0], sides[1], sides[2], sides[3]];
    for i in 0..4 {
        s.add_constraint(Constraint::Coincident(
            PointRef::End(sides[i]),
            PointRef::Start(sides[(i + 1) % 4]),
        ))?;
    }
    s.add_constraint(Constraint::Horizontal(bottom))?;
    s.add_constraint(Constraint::Horizontal(top))?;
    s.add_constraint(Constraint::Vertical(right))?;
    s.add_constraint(Constraint::Vertical(left))?;
    s.add_constraint(Constraint::Fixed {
        point: PointRef::Start(bottom),
        x: 0.0,
        y: 0.0,
    })?;

    // Each dimension becomes a model parameter: d0, then d1.
    for (side, value) in [(bottom, 40.0), (right, 25.0)] {
        let (_, name) = bind_dimension(
            &mut s,
            &mut doc,
            Constraint::Distance {
                a: PointRef::Start(side),
                b: PointRef::End(side),
                value,
            },
        )?;
        println!("dimension {value} mm -> parameter {name}");
    }
    s.solve(SolveOptions::default())?;
    println!("solved: {}", size(&s, bottom, right)?);

    // Rename d0 in the table, and tell the sketch.
    doc.rename_param("d0", "width")?;
    let moved = rename_bound(&mut s, "d0", "width");
    println!("renamed d0 to width ({moved} dimension follows)");

    // Change the parameter, copy it into the sketch, solve again.
    doc.set_param("width", 60.0);
    apply_params(&mut s, &doc)?;
    s.solve(SolveOptions::default())?;
    println!("width = 60: {}", size(&s, bottom, right)?);

    // Mistake 1: rename in the table only.
    doc.rename_param("d1", "height")?;
    if let Err(e) = apply_params(&mut s, &doc) {
        println!("rename d1 without rename_bound -> error: {e}");
    }
    rename_bound(&mut s, "d1", "height");

    // Mistake 2: a value of zero. Nothing changes.
    doc.set_param("height", 0.0);
    if let Err(e) = apply_params(&mut s, &doc) {
        println!("height = 0 -> error: {e}");
    }
    println!("sketch still {}", size(&s, bottom, right)?);
    Ok(())
}
