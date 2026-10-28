//! Sketches a slot: two straight sides joined by two half-circle ends,
//! 30 mm between the centers of the ends, radius 5 mm. Shows coincident on
//! an arc's ends, tangent and equal, then finds the region to extrude.
//!
//! The slot is drawn roughly on purpose: the ends are different sizes, the
//! slot is tilted, and the left arc stops 10° short, so the outline has gaps.
//! Coincident constraints join each side's ends to the arcs' ends
//! (`PointRef::ArcStart` / `ArcEnd`), and solving closes the gaps. Used by the
//! user manual ("Draw a slot").
//!
//! Run: `cargo run -p swx-sketch --example slot_sketch`

use std::f64::consts::{FRAC_PI_2, PI};

use swx_sketch::dof::{analyze, DofOptions};
use swx_sketch::profile::{profile_report, ProfileOptions};
use swx_sketch::solver::SolveOptions;
use swx_sketch::{Arc, Constraint, GeomId, Geometry, Line, Point, PointRef, Sketch};

/// Draws the slot roughly and adds its constraints, without solving.
fn slot() -> Result<(Sketch, [GeomId; 4]), Box<dyn std::error::Error>> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);

    // Roughly drawn: the ends are different sizes, the slot is tilted, and
    // the left arc stops 10° before the bottom side.
    let left = s.add(Geometry::Arc(Arc::new(
        p(0.0, 0.0)?,
        4.0,
        FRAC_PI_2,
        3.0 * FRAC_PI_2 - PI / 18.0,
    )?))?;
    let right = s.add(Geometry::Arc(Arc::new(
        p(28.0, 1.0)?,
        6.0,
        -FRAC_PI_2,
        FRAC_PI_2,
    )?))?;
    let bottom = s.add(Geometry::Line(Line::new(p(0.0, -4.0)?, p(28.0, -5.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(28.0, 7.0)?, p(0.0, 4.0)?)?))?;

    // Join the outline end to end. An arc runs counterclockwise from its
    // start to its end, so the left arc ends at the bottom side.
    for (a, b) in [
        (PointRef::Start(bottom), PointRef::ArcEnd(left)),
        (PointRef::End(bottom), PointRef::ArcStart(right)),
        (PointRef::Start(top), PointRef::ArcEnd(right)),
        (PointRef::End(top), PointRef::ArcStart(left)),
    ] {
        s.add_constraint(Constraint::Coincident(a, b))?;
    }
    // Both sides run smoothly into both ends.
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

fn report(s: &Sketch, ids: [GeomId; 4]) -> Result<(), Box<dyn std::error::Error>> {
    let [left, right, bottom, _] = ids;
    for (name, id) in [("left", left), ("right", right)] {
        if let Geometry::Arc(a) = s.get(id)? {
            println!(
                "  {name:<6} arc center ({:6.2}, {:5.2})  radius {:.2} mm  from {:4.0}° to {:4.0}°",
                tidy(a.center.x),
                tidy(a.center.y),
                a.radius,
                tidy(a.start_angle.to_degrees()),
                tidy(a.end_angle.to_degrees())
            );
        }
    }
    if let Geometry::Line(l) = s.get(bottom)? {
        println!(
            "  bottom ({:6.2}, {:5.2}) -> ({:6.2}, {:5.2})",
            tidy(l.start.x),
            tidy(l.start.y),
            tidy(l.end.x),
            tidy(l.end.y)
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
    let (mut s, ids) = slot()?;
    println!("1 as drawn");
    report(&s, ids)?;

    let solved = s.solve(SolveOptions::default())?;
    println!("2 solved: {}", solved.converged);
    let dof = analyze(&s, s.params(), DofOptions::default())?;
    println!("  dof {} {:?}", dof.dof, dof.state);
    report(&s, ids)?;
    Ok(())
}
