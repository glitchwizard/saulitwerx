//! Sketches a 40 × 25 mm plate with rounded corners: four straight sides and
//! four quarter-circle corners of radius 5 mm. Shows coincident and tangent on
//! an arc's ends, and equal to set all four corners with one radius, then
//! finds the region to extrude.
//!
//! The plate is drawn roughly on purpose: the corners are different sizes,
//! the sides lean, and the ends don't meet. Solving closes the outline.
//! Then the radius changes to 8 mm and the sketch is solved again: the plate
//! keeps its 40 × 25 mm size, because the dimensions are on the outline.
//! Used by the user manual ("Round the corners of a plate").
//!
//! Run: `cargo run -p swx-sketch --example rounded_plate`

use std::f64::consts::{FRAC_PI_2, PI};

use swx_sketch::dof::{analyze, DofOptions};
use swx_sketch::profile::{profile_report, ProfileOptions};
use swx_sketch::solver::SolveOptions;
use swx_sketch::{Arc, Constraint, ConstraintId, GeomId, Geometry, Line, Point, PointRef, Sketch};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// The plate's pieces: the four sides, then the four corners, each corner
/// named after where it sits (bottom right, top right, top left, bottom left).
struct Plate {
    sides: [(&'static str, GeomId); 4],
    corners: [(&'static str, GeomId); 4],
    radius: ConstraintId,
}

/// Draws the plate roughly and adds its constraints, without solving.
fn plate() -> Res<(Sketch, Plate)> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);

    // Roughly drawn: the sides lean a little and stop short of the corners,
    // and the corners have four different radii.
    let bottom = s.add(Geometry::Line(Line::new(p(6.0, 0.5)?, p(34.0, -0.5)?)?))?;
    let right = s.add(Geometry::Line(Line::new(p(40.5, 6.0)?, p(39.5, 19.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(34.0, 26.0)?, p(6.0, 25.0)?)?))?;
    let left = s.add(Geometry::Line(Line::new(p(-0.5, 19.0)?, p(0.5, 6.0)?)?))?;
    // An arc runs counterclockwise from its start angle to its end angle, so
    // each corner starts where one side ends and ends where the next begins.
    let br = s.add(Geometry::Arc(Arc::new(
        p(35.0, 4.0)?,
        4.0,
        -FRAC_PI_2,
        0.0,
    )?))?;
    let tr = s.add(Geometry::Arc(Arc::new(
        p(34.0, 20.0)?,
        6.0,
        0.0,
        FRAC_PI_2,
    )?))?;
    let tl = s.add(Geometry::Arc(Arc::new(p(5.0, 21.0)?, 4.5, FRAC_PI_2, PI)?))?;
    let bl = s.add(Geometry::Arc(Arc::new(
        p(5.0, 5.0)?,
        5.5,
        PI,
        3.0 * FRAC_PI_2,
    )?))?;

    // Join the outline end to end, side, corner, side, corner, ...
    for (side, corner, next) in [
        (bottom, br, right),
        (right, tr, top),
        (top, tl, left),
        (left, bl, bottom),
    ] {
        s.add_constraint(Constraint::Coincident(
            PointRef::End(side),
            PointRef::ArcStart(corner),
        ))?;
        s.add_constraint(Constraint::Coincident(
            PointRef::ArcEnd(corner),
            PointRef::Start(next),
        ))?;
        // No corner where a side meets a rounded corner.
        s.add_constraint(Constraint::Tangent(side, corner))?;
        s.add_constraint(Constraint::Tangent(next, corner))?;
    }
    // All four corners the same size, set once.
    for c in [tr, tl, bl] {
        s.add_constraint(Constraint::Equal(br, c))?;
    }
    let radius = s.add_constraint(Constraint::Radius {
        geom: tr,
        value: 5.0,
    })?;
    // Square to the axes.
    for c in [
        Constraint::Horizontal(bottom),
        Constraint::Horizontal(top),
        Constraint::Vertical(left),
        Constraint::Vertical(right),
    ] {
        s.add_constraint(c)?;
    }
    // Overall size, 40 × 25 mm. Each side's ends are where it meets the
    // rounding, so the left side's start and the right side's end sit
    // straight across from each other, the full width apart.
    s.add_constraint(Constraint::Distance {
        a: PointRef::Start(left),
        b: PointRef::End(right),
        value: 40.0,
    })?;
    s.add_constraint(Constraint::Distance {
        a: PointRef::End(bottom),
        b: PointRef::Start(top),
        value: 25.0,
    })?;
    // Anchored: a fixed point at the origin, with the left side straight
    // above it and the bottom side level with it.
    let origin = s.add(Geometry::Point(p(0.0, 0.0)?))?;
    s.add_constraint(Constraint::Fixed {
        point: PointRef::Point(origin),
        x: 0.0,
        y: 0.0,
    })?;
    s.add_constraint(Constraint::VerticalPoints(
        PointRef::Point(origin),
        PointRef::Start(left),
    ))?;
    s.add_constraint(Constraint::HorizontalPoints(
        PointRef::Point(origin),
        PointRef::Start(bottom),
    ))?;
    let plate = Plate {
        sides: [
            ("bottom", bottom),
            ("right", right),
            ("top", top),
            ("left", left),
        ],
        corners: [("br", br), ("tr", tr), ("tl", tl), ("bl", bl)],
        radius,
    };
    Ok((s, plate))
}

/// `v`, with solver round-off near zero shown as 0 rather than -0.00.
fn tidy(v: f64) -> f64 {
    if v.abs() < 1e-9 {
        0.0
    } else {
        v
    }
}

fn report(s: &Sketch, plate: &Plate) -> Res<()> {
    for (name, id) in plate.corners {
        if let Geometry::Arc(a) = s.get(id)? {
            println!(
                "  {name:<6} arc center ({:5.2}, {:5.2})  radius {:.2} mm",
                tidy(a.center.x),
                tidy(a.center.y),
                a.radius,
            );
        }
    }
    for (name, id) in plate.sides {
        if let Geometry::Line(l) = s.get(id)? {
            println!(
                "  {name:<6} ({:5.2}, {:5.2}) -> ({:5.2}, {:5.2})",
                tidy(l.start.x),
                tidy(l.start.y),
                tidy(l.end.x),
                tidy(l.end.y)
            );
        }
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

fn main() -> Res<()> {
    let (mut s, plate) = plate()?;
    println!("1 as drawn");
    report(&s, &plate)?;

    let solved = s.solve(SolveOptions::default())?;
    println!("2 solved: {}", solved.converged);
    let dof = analyze(&s, s.params(), DofOptions::default())?;
    println!("  dof {} {:?}", dof.dof, dof.state);
    report(&s, &plate)?;

    s.set_dimension_value(plate.radius, 8.0)?;
    let solved = s.solve(SolveOptions::default())?;
    println!("3 radius 8 mm, solved: {}", solved.converged);
    report(&s, &plate)?;
    Ok(())
}
