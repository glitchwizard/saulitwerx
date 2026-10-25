//! Plots the sketches of `swx-sketch`'s `centered_plate` example at two
//! widths, and stage 2 of its `slot_sketch` example (one arc drawn short).
//!
//! Used by the user manual for its pictures of the parallel, perpendicular,
//! midpoint, tangent and equal constraints.
//!
//! Run: `cargo run -p swx-qa --example constraint_plots -- <out-dir> [<W>x<H>]`
//! Writes `sketch-centered-40.png`, `sketch-centered-60.png`
//! and `sketch-slot-gap.png`.

use std::f64::consts::{FRAC_PI_2, PI};
use std::path::PathBuf;

use swx_qa::parse_size;
use swx_qa::sketch::{plot, PlotOptions};
use swx_sketch::solver::SolveOptions;
use swx_sketch::{Arc, Circle, Constraint, Geometry, Line, Point, PointRef, Sketch};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// The `centered_plate` sketch at the given width, 25 mm high.
fn centered_plate(width: f64) -> Res<Sketch> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);
    let bottom = s.add(Geometry::Line(Line::new(p(0.0, 0.0)?, p(38.0, 1.0)?)?))?;
    let right = s.add(Geometry::Line(Line::new(p(38.0, 1.0)?, p(39.0, 22.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(39.0, 22.0)?, p(1.0, 24.0)?)?))?;
    let left = s.add(Geometry::Line(Line::new(p(1.0, 24.0)?, p(0.0, 0.0)?)?))?;
    let hole = s.add(Geometry::Circle(Circle::new(p(18.0, 11.0)?, 4.0)?))?;
    let mid_bottom = s.add(Geometry::Point(p(19.0, 0.0)?))?;
    let mid_left = s.add(Geometry::Point(p(0.0, 12.0)?))?;
    for (a, b) in [(bottom, right), (right, top), (top, left), (left, bottom)] {
        s.add_constraint(Constraint::Coincident(PointRef::End(a), PointRef::Start(b)))?;
    }
    for c in [
        Constraint::Horizontal(bottom),
        Constraint::Perpendicular(bottom, left),
        Constraint::Parallel(top, bottom),
        Constraint::Parallel(right, left),
        Constraint::Fixed {
            point: PointRef::Start(bottom),
            x: 0.0,
            y: 0.0,
        },
        Constraint::Distance {
            a: PointRef::Start(bottom),
            b: PointRef::End(bottom),
            value: width,
        },
        Constraint::Distance {
            a: PointRef::Start(right),
            b: PointRef::End(right),
            value: 25.0,
        },
        Constraint::Midpoint {
            point: PointRef::Point(mid_bottom),
            line: bottom,
        },
        Constraint::Midpoint {
            point: PointRef::Point(mid_left),
            line: left,
        },
        Constraint::VerticalPoints(PointRef::Point(mid_bottom), PointRef::Center(hole)),
        Constraint::HorizontalPoints(PointRef::Point(mid_left), PointRef::Center(hole)),
        Constraint::Diameter {
            geom: hole,
            value: 10.0,
        },
    ] {
        s.add_constraint(c)?;
    }
    s.solve(SolveOptions::default())?;
    Ok(s)
}

/// The `slot_sketch` sketch, with the left arc ending at `left_end`.
fn slot(left_end: f64) -> Res<Sketch> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);
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
    for (line, arc) in [(bottom, left), (bottom, right), (top, left), (top, right)] {
        s.add_constraint(Constraint::Tangent(line, arc))?;
    }
    for c in [
        Constraint::Equal(left, right),
        Constraint::Radius {
            geom: left,
            value: 5.0,
        },
        Constraint::Horizontal(bottom),
        Constraint::Fixed {
            point: PointRef::Center(left),
            x: 0.0,
            y: 0.0,
        },
        Constraint::Distance {
            a: PointRef::Center(left),
            b: PointRef::Center(right),
            value: 30.0,
        },
        Constraint::VerticalPoints(PointRef::Start(bottom), PointRef::Center(left)),
        Constraint::VerticalPoints(PointRef::End(bottom), PointRef::Center(right)),
        Constraint::VerticalPoints(PointRef::Start(top), PointRef::Center(right)),
        Constraint::VerticalPoints(PointRef::End(top), PointRef::Center(left)),
    ] {
        s.add_constraint(c)?;
    }
    s.solve(SolveOptions::default())?;
    Ok(s)
}

fn main() -> Res<()> {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(
        args.next()
            .ok_or("usage: constraint_plots <out-dir> [<W>x<H>]")?,
    );
    let mut opts = PlotOptions::default();
    if let Some(size) = args.next() {
        (opts.width, opts.height) = parse_size(&size)?;
    }
    for (s, name) in [
        (centered_plate(40.0)?, "sketch-centered-40.png"),
        (centered_plate(60.0)?, "sketch-centered-60.png"),
        (slot(3.0 * FRAC_PI_2 - PI / 18.0)?, "sketch-slot-gap.png"),
    ] {
        let path = dir.join(name);
        std::fs::write(&path, plot(&s, &opts)?.to_png()?)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
