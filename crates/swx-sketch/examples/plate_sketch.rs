//! Sketches a 40 × 25 mm rectangle with a 10 mm hole and solves it.
//!
//! The rough shape is drawn off-size on purpose; the constraints pull it
//! into place. Used by the user manual ("Solve a sketch").
//!
//! Run: `cargo run -p swx-sketch --example plate_sketch`

use swx_sketch::solver::SolveOptions;
use swx_sketch::{Circle, Constraint, Geometry, Line, Point, PointRef, Sketch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);

    // A rough, slightly crooked rectangle and a circle, as if drawn by hand.
    let bottom = s.add(Geometry::Line(Line::new(p(0.0, 0.0)?, p(38.0, 1.0)?)?))?;
    let right = s.add(Geometry::Line(Line::new(p(38.0, 1.0)?, p(39.0, 22.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(39.0, 22.0)?, p(1.0, 24.0)?)?))?;
    let left = s.add(Geometry::Line(Line::new(p(1.0, 24.0)?, p(0.0, 0.0)?)?))?;
    let hole = s.add(Geometry::Circle(Circle::new(p(18.0, 11.0)?, 4.0)?))?;

    // Close the outline: each line ends where the next one starts.
    for (a, b) in [(bottom, right), (right, top), (top, left), (left, bottom)] {
        s.add_constraint(Constraint::Coincident(PointRef::End(a), PointRef::Start(b)))?;
    }
    // Straighten it.
    s.add_constraint(Constraint::Horizontal(bottom))?;
    s.add_constraint(Constraint::Horizontal(top))?;
    s.add_constraint(Constraint::Vertical(left))?;
    s.add_constraint(Constraint::Vertical(right))?;
    // Anchor the lower-left corner at the origin.
    s.add_constraint(Constraint::Fixed {
        point: PointRef::Start(bottom),
        x: 0.0,
        y: 0.0,
    })?;
    // Size it.
    s.add_constraint(Constraint::Distance {
        a: PointRef::Start(bottom),
        b: PointRef::End(bottom),
        value: 40.0,
    })?;
    s.add_constraint(Constraint::Distance {
        a: PointRef::Start(right),
        b: PointRef::End(right),
        value: 25.0,
    })?;
    // Place and size the hole.
    s.add_constraint(Constraint::Fixed {
        point: PointRef::Center(hole),
        x: 20.0,
        y: 12.5,
    })?;
    s.add_constraint(Constraint::Diameter {
        geom: hole,
        value: 10.0,
    })?;

    let report = s.solve(SolveOptions::default())?;
    println!(
        "solved: {} ({:?}, {} iterations, residual {:.1e})",
        report.converged, report.reason, report.iterations, report.final_residual_norm
    );
    for (name, id) in [
        ("bottom", bottom),
        ("right", right),
        ("top", top),
        ("left", left),
    ] {
        if let Geometry::Line(l) = s.get(id)? {
            println!(
                "{name:<6} ({:6.2}, {:6.2}) -> ({:6.2}, {:6.2})  length {:.2} mm",
                l.start.x,
                l.start.y,
                l.end.x,
                l.end.y,
                l.length()
            );
        }
    }
    if let Geometry::Circle(c) = s.get(hole)? {
        println!(
            "hole   center ({:.2}, {:.2})  diameter {:.2} mm",
            c.center.x,
            c.center.y,
            2.0 * c.radius
        );
    }
    Ok(())
}
