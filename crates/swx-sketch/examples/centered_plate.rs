//! Sketches the 40 × 25 mm plate with its 10 mm hole held at the center,
//! whatever the plate's size, then finds the region to extrude.
//!
//! The rectangle is squared up with perpendicular and parallel instead of
//! horizontal and vertical on every side. Two helper points sit at the
//! middles of the bottom and left sides (midpoint), and the hole's center is
//! lined up with them. Used by the user manual ("Keep a hole centered" and
//! "Find the region to extrude").
//!
//! Run: `cargo run -p swx-sketch --example centered_plate [-- <width> <height>]`

use swx_sketch::dof::{analyze, DofOptions};
use swx_sketch::profile::{profile_report, ProfileOptions};
use swx_sketch::solver::SolveOptions;
use swx_sketch::{Circle, Constraint, Geometry, Line, Point, PointRef, Sketch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let width: f64 = args.next().map_or(Ok(40.0), |a| a.parse())?;
    let height: f64 = args.next().map_or(Ok(25.0), |a| a.parse())?;

    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);

    // The same rough drawing as `plate_sketch`, plus two helper points.
    let bottom = s.add(Geometry::Line(Line::new(p(0.0, 0.0)?, p(38.0, 1.0)?)?))?;
    let right = s.add(Geometry::Line(Line::new(p(38.0, 1.0)?, p(39.0, 22.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(39.0, 22.0)?, p(1.0, 24.0)?)?))?;
    let left = s.add(Geometry::Line(Line::new(p(1.0, 24.0)?, p(0.0, 0.0)?)?))?;
    let hole = s.add(Geometry::Circle(Circle::new(p(18.0, 11.0)?, 4.0)?))?;
    let mid_bottom = s.add(Geometry::Point(p(19.0, 0.0)?))?;
    let mid_left = s.add(Geometry::Point(p(0.0, 12.0)?))?;

    // Close the outline.
    for (a, b) in [(bottom, right), (right, top), (top, left), (left, bottom)] {
        s.add_constraint(Constraint::Coincident(PointRef::End(a), PointRef::Start(b)))?;
    }
    // Square it up: one level side, then square corners and opposite sides
    // parallel.
    s.add_constraint(Constraint::Horizontal(bottom))?;
    s.add_constraint(Constraint::Perpendicular(bottom, left))?;
    s.add_constraint(Constraint::Parallel(top, bottom))?;
    s.add_constraint(Constraint::Parallel(right, left))?;
    // Anchor and size it.
    s.add_constraint(Constraint::Fixed {
        point: PointRef::Start(bottom),
        x: 0.0,
        y: 0.0,
    })?;
    s.add_constraint(Constraint::Distance {
        a: PointRef::Start(bottom),
        b: PointRef::End(bottom),
        value: width,
    })?;
    s.add_constraint(Constraint::Distance {
        a: PointRef::Start(right),
        b: PointRef::End(right),
        value: height,
    })?;
    // Center the hole: helper points at the middles of two sides, and the
    // hole's center straight above one and level with the other.
    s.add_constraint(Constraint::Midpoint {
        point: PointRef::Point(mid_bottom),
        line: bottom,
    })?;
    s.add_constraint(Constraint::Midpoint {
        point: PointRef::Point(mid_left),
        line: left,
    })?;
    s.add_constraint(Constraint::VerticalPoints(
        PointRef::Point(mid_bottom),
        PointRef::Center(hole),
    ))?;
    s.add_constraint(Constraint::HorizontalPoints(
        PointRef::Point(mid_left),
        PointRef::Center(hole),
    ))?;
    s.add_constraint(Constraint::Diameter {
        geom: hole,
        value: 10.0,
    })?;

    let report = s.solve(SolveOptions::default())?;
    let dof = analyze(&s, s.params(), DofOptions::default())?;
    println!(
        "solved: {} ({:?}), dof {} {:?}",
        report.converged, report.reason, dof.dof, dof.state
    );
    for (name, id) in [("bottom", bottom), ("right", right)] {
        if let Geometry::Line(l) = s.get(id)? {
            println!("{name:<6} length {:.2} mm", l.length());
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

    // The region an extrude would use: the outline, minus the hole.
    let profile = profile_report(&s, &ProfileOptions::default())?;
    println!("regions: {}", profile.regions.len());
    for (i, r) in profile.regions.iter().enumerate() {
        println!(
            "  region {i}: outer edges {}, holes {}, area {:.2} mm²",
            r.outer.edges.len(),
            r.holes.len(),
            r.area()
        );
    }
    println!("open edges: {}", profile.open_edges.len());
    Ok(())
}
