//! Sketches a slot: two straight sides joined by two half-circle ends,
//! 30 mm between the centers of the ends, radius 5 mm. Shows tangent and
//! equal, then finds the region to extrude.
//!
//! Constraints can't name the ends of an arc yet, only its center, so each
//! straight side's ends are lined up under the arc centers (vertical points)
//! and the arcs must be drawn as exact half circles to meet them. The last
//! stage draws one arc 10° short to show what happens when they don't. Used
//! by the user manual ("Draw a slot").
//!
//! Run: `cargo run -p swx-sketch --example slot_sketch`

use std::f64::consts::{FRAC_PI_2, PI};

use swx_sketch::dof::{analyze, DofOptions};
use swx_sketch::profile::{profile_report, ProfileOptions};
use swx_sketch::solver::SolveOptions;
use swx_sketch::{Arc, Constraint, Geometry, Line, Point, PointRef, Sketch};

/// Builds and solves the slot. `left_end` is the end angle of the left arc,
/// in radians; 3π/2 makes it an exact half circle.
fn slot(left_end: f64) -> Result<(Sketch, [swx_sketch::GeomId; 4]), Box<dyn std::error::Error>> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);

    // Roughly drawn: the ends are different sizes and the slot is tilted.
    let left = s.add(Geometry::Arc(Arc::new(
        p(0.0, 0.0)?,
        4.0,
        FRAC_PI_2,
        left_end,
    )?))?;
    let right = s.add(Geometry::Arc(Arc::new(
        p(28.0, 1.0)?,
        6.0,
        -FRAC_PI_2,
        FRAC_PI_2,
    )?))?;
    let bottom = s.add(Geometry::Line(Line::new(p(0.0, -4.0)?, p(28.0, -5.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(28.0, 7.0)?, p(0.0, 4.0)?)?))?;

    // Both sides touch both ends without crossing them.
    for (line, arc) in [(bottom, left), (bottom, right), (top, left), (top, right)] {
        s.add_constraint(Constraint::Tangent(line, arc))?;
    }
    // Both ends the same size, set once.
    s.add_constraint(Constraint::Equal(left, right))?;
    s.add_constraint(Constraint::Radius {
        geom: left,
        value: 5.0,
    })?;
    // Level, anchored, and 30 mm between the centers.
    s.add_constraint(Constraint::Horizontal(bottom))?;
    s.add_constraint(Constraint::Fixed {
        point: PointRef::Center(left),
        x: 0.0,
        y: 0.0,
    })?;
    s.add_constraint(Constraint::Distance {
        a: PointRef::Center(left),
        b: PointRef::Center(right),
        value: 30.0,
    })?;
    // Each side ends straight above or below an arc's center.
    for (end, arc) in [
        (PointRef::Start(bottom), left),
        (PointRef::End(bottom), right),
        (PointRef::Start(top), right),
        (PointRef::End(top), left),
    ] {
        s.add_constraint(Constraint::VerticalPoints(end, PointRef::Center(arc)))?;
    }
    s.solve(SolveOptions::default())?;
    Ok((s, [left, right, bottom, top]))
}

/// `v`, with solver round-off near zero shown as 0 rather than -0.00.
fn tidy(v: f64) -> f64 {
    if v.abs() < 1e-9 {
        0.0
    } else {
        v
    }
}

fn report(s: &Sketch, ids: [swx_sketch::GeomId; 4]) -> Result<(), Box<dyn std::error::Error>> {
    let dof = analyze(s, s.params(), DofOptions::default())?;
    println!("  dof {} {:?}", dof.dof, dof.state);
    let [left, right, bottom, _] = ids;
    for (name, id) in [("left", left), ("right", right)] {
        if let Geometry::Arc(a) = s.get(id)? {
            println!(
                "  {name:<6} arc center ({:6.2}, {:5.2})  radius {:.2} mm",
                tidy(a.center.x),
                tidy(a.center.y),
                a.radius
            );
        }
    }
    if let Geometry::Line(l) = s.get(bottom)? {
        println!(
            "  bottom ({:6.2}, {:5.2}) -> ({:6.2}, {:5.2})",
            l.start.x, l.start.y, l.end.x, l.end.y
        );
    }
    let profile = profile_report(s, &ProfileOptions::default())?;
    println!("  regions: {}", profile.regions.len());
    for r in &profile.regions {
        println!(
            "    outer edges {}, holes {}, area {:.2} mm²",
            r.outer.edges.len(),
            r.holes.len(),
            r.area()
        );
    }
    let open: Vec<u32> = profile.open_edges.iter().map(|g| g.0).collect();
    println!("  open edges: {open:?}");
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("1 half-circle ends");
    let (s, ids) = slot(3.0 * FRAC_PI_2)?;
    report(&s, ids)?;

    println!("2 left end drawn 10 degrees short");
    let (s, ids) = slot(3.0 * FRAC_PI_2 - PI / 18.0)?;
    report(&s, ids)?;
    Ok(())
}
