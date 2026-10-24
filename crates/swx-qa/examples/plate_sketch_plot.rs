//! Plots the plate sketch of `swx-sketch`'s `plate_sketch` example in three
//! stages: the rough drawing, the shape constraints solved, and the fully
//! dimensioned sketch solved.
//!
//! Used by the user manual for its sketch pictures.
//!
//! Run: `cargo run -p swx-qa --example plate_sketch_plot -- <out-dir> [<W>x<H>]`
//! Writes `sketch-rough.png`, `sketch-shape.png` and `sketch-sized.png`.

use std::path::PathBuf;

use swx_qa::parse_size;
use swx_qa::sketch::{plot, PlotOptions};
use swx_sketch::solver::SolveOptions;
use swx_sketch::{Circle, Constraint, Geometry, Line, Point, PointRef, Sketch};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(
        args.next()
            .ok_or("usage: plate_sketch_plot <out-dir> [<W>x<H>]")?,
    );
    let mut opts = PlotOptions::default();
    if let Some(size) = args.next() {
        (opts.width, opts.height) = parse_size(&size)?;
    }
    let write = |s: &Sketch, name: &str| -> Result<(), Box<dyn std::error::Error>> {
        let path = dir.join(name);
        std::fs::write(&path, plot(s, &opts)?.to_png()?)?;
        println!("wrote {}", path.display());
        Ok(())
    };

    // The same rough drawing as `plate_sketch`.
    let mut s = Sketch::new();
    let p = |x, y| Point::new(x, y);
    let bottom = s.add(Geometry::Line(Line::new(p(0.0, 0.0)?, p(38.0, 1.0)?)?))?;
    let right = s.add(Geometry::Line(Line::new(p(38.0, 1.0)?, p(39.0, 22.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(39.0, 22.0)?, p(1.0, 24.0)?)?))?;
    let left = s.add(Geometry::Line(Line::new(p(1.0, 24.0)?, p(0.0, 0.0)?)?))?;
    let hole = s.add(Geometry::Circle(Circle::new(p(18.0, 11.0)?, 4.0)?))?;
    write(&s, "sketch-rough.png")?;

    // Shape only: closed, straight, anchored. No sizes yet.
    for (a, b) in [(bottom, right), (right, top), (top, left), (left, bottom)] {
        s.add_constraint(Constraint::Coincident(PointRef::End(a), PointRef::Start(b)))?;
    }
    s.add_constraint(Constraint::Horizontal(bottom))?;
    s.add_constraint(Constraint::Horizontal(top))?;
    s.add_constraint(Constraint::Vertical(left))?;
    s.add_constraint(Constraint::Vertical(right))?;
    s.add_constraint(Constraint::Fixed {
        point: PointRef::Start(bottom),
        x: 0.0,
        y: 0.0,
    })?;
    let mut shape = s.clone();
    shape.solve(SolveOptions::default())?;
    write(&shape, "sketch-shape.png")?;

    // Sizes: the rest of `plate_sketch`'s constraints.
    for c in [
        Constraint::Distance {
            a: PointRef::Start(bottom),
            b: PointRef::End(bottom),
            value: 40.0,
        },
        Constraint::Distance {
            a: PointRef::Start(right),
            b: PointRef::End(right),
            value: 25.0,
        },
        Constraint::Fixed {
            point: PointRef::Center(hole),
            x: 20.0,
            y: 12.5,
        },
        Constraint::Diameter {
            geom: hole,
            value: 10.0,
        },
    ] {
        s.add_constraint(c)?;
    }
    s.solve(SolveOptions::default())?;
    write(&s, "sketch-sized.png")?;
    Ok(())
}
