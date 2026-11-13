//! Plots the sketches of `swx-sketch`'s `centered_plate` example at two
//! widths, its `slot_sketch` example as drawn and solved, and its
//! `rounded_plate` example as drawn and solved with 5 mm and 8 mm corners.
//!
//! Used by the user manual for its pictures of the parallel, perpendicular,
//! midpoint, tangent and equal constraints.
//!
//! Run: `cargo run -p swx-qa --example constraint_plots -- <out-dir> [<W>x<H>]`
//! Writes `sketch-centered-40.png`, `sketch-centered-60.png`,
//! `sketch-slot-rough.png`, `sketch-slot.png`, `sketch-rounded-rough.png`,
//! `sketch-rounded-5.png` and `sketch-rounded-8.png`.

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

/// The `slot_sketch` sketch: as drawn (with gaps) when `solve` is false,
/// closed by its coincident and tangent constraints when it is true.
fn slot(solve: bool) -> Res<Sketch> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);
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
    for (a, b) in [
        (PointRef::Start(bottom), PointRef::ArcEnd(left)),
        (PointRef::End(bottom), PointRef::ArcStart(right)),
        (PointRef::Start(top), PointRef::ArcEnd(right)),
        (PointRef::End(top), PointRef::ArcStart(left)),
    ] {
        s.add_constraint(Constraint::Coincident(a, b))?;
    }
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
    ] {
        s.add_constraint(c)?;
    }
    if solve {
        s.solve(SolveOptions::default())?;
    }
    Ok(s)
}

/// The `rounded_plate` sketch: as drawn (with gaps) when `radius` is `None`,
/// solved with that corner radius otherwise.
fn rounded(radius: Option<f64>) -> Res<Sketch> {
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);
    let bottom = s.add(Geometry::Line(Line::new(p(6.0, 0.5)?, p(34.0, -0.5)?)?))?;
    let right = s.add(Geometry::Line(Line::new(p(40.5, 6.0)?, p(39.5, 19.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(34.0, 26.0)?, p(6.0, 25.0)?)?))?;
    let left = s.add(Geometry::Line(Line::new(p(-0.5, 19.0)?, p(0.5, 6.0)?)?))?;
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
        s.add_constraint(Constraint::Tangent(side, corner))?;
        s.add_constraint(Constraint::Tangent(next, corner))?;
    }
    let origin = s.add(Geometry::Point(p(0.0, 0.0)?))?;
    for c in [
        Constraint::Equal(br, tr),
        Constraint::Equal(br, tl),
        Constraint::Equal(br, bl),
        Constraint::Radius {
            geom: tr,
            value: radius.unwrap_or(5.0),
        },
        Constraint::Horizontal(bottom),
        Constraint::Horizontal(top),
        Constraint::Vertical(left),
        Constraint::Vertical(right),
        Constraint::Distance {
            a: PointRef::Start(left),
            b: PointRef::End(right),
            value: 40.0,
        },
        Constraint::Distance {
            a: PointRef::End(bottom),
            b: PointRef::Start(top),
            value: 25.0,
        },
        Constraint::Fixed {
            point: PointRef::Point(origin),
            x: 0.0,
            y: 0.0,
        },
        Constraint::VerticalPoints(PointRef::Point(origin), PointRef::Start(left)),
        Constraint::HorizontalPoints(PointRef::Point(origin), PointRef::Start(bottom)),
    ] {
        s.add_constraint(c)?;
    }
    if radius.is_some() {
        s.solve(SolveOptions::default())?;
    }
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
        (slot(false)?, "sketch-slot-rough.png"),
        (slot(true)?, "sketch-slot.png"),
        (rounded(None)?, "sketch-rounded-rough.png"),
        (rounded(Some(5.0))?, "sketch-rounded-5.png"),
        (rounded(Some(8.0))?, "sketch-rounded-8.png"),
    ] {
        let path = dir.join(name);
        std::fs::write(&path, plot(&s, &opts)?.to_png()?)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
