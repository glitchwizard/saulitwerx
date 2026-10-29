//! Plots the sketch of `swx-sketch`'s `inferred_sketch` example twice: each
//! entity as it was drawn, and the sketch after inference has added its
//! constraints and solved.
//!
//! Used by the user manual for its picture of sketch inference.
//!
//! Run: `cargo run -p swx-qa --example inference_plots -- <out-dir> [<W>x<H>]`
//! Writes `sketch-infer-drawn.png` and `sketch-infer.png`.

use std::path::PathBuf;

use swx_qa::parse_size;
use swx_qa::sketch::{plot, PlotOptions};
use swx_sketch::infer::{add_inferred, InferOptions};
use swx_sketch::{Circle, GeomId, Geometry, Line, Point, Sketch};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Adds `geom` to both sketches, and infers its constraints in `solved` only.
fn draw(drawn: &mut Sketch, solved: &mut Sketch, geom: Geometry, tol: Option<f64>) -> Res<()> {
    drawn.add(geom)?;
    let id = solved.add(geom)?;
    let mut opts = InferOptions::for_sketch(solved);
    if let Some(t) = tol {
        opts.point_tol = t;
    }
    add_inferred(solved, id, &opts)?;
    Ok(())
}

fn end_of(s: &Sketch, id: u32) -> Res<(f64, f64)> {
    match s.get(GeomId(id))? {
        Geometry::Line(l) => Ok((l.end.x, l.end.y)),
        _ => Err("not a line".into()),
    }
}

/// The two sketches, drawn exactly as in `inferred_sketch`.
fn sketches() -> Res<(Sketch, Sketch)> {
    let (mut d, mut s) = (Sketch::new(), Sketch::new());
    let p = Point::new;
    let line = |a: Point, b: Point| -> Res<Geometry> { Ok(Geometry::Line(Line::new(a, b)?)) };
    draw(&mut d, &mut s, line(p(0.0, 0.0)?, p(40.0, 0.4)?)?, None)?;
    let (x, y) = end_of(&s, 0)?;
    draw(
        &mut d,
        &mut s,
        line(p(x + 0.01, y - 0.02)?, p(39.7, 25.0)?)?,
        None,
    )?;
    let (x, y) = end_of(&s, 1)?;
    draw(
        &mut d,
        &mut s,
        line(p(x - 0.02, y + 0.01)?, p(0.2, 25.2)?)?,
        None,
    )?;
    let (x, y) = end_of(&s, 2)?;
    let left = line(p(x + 0.01, y + 0.01)?, p(0.1, 1.5)?)?;
    draw(&mut d, &mut s, left, Some(2.0))?;
    let Geometry::Line(right) = s.get(GeomId(1))? else {
        return Err("not a line".into());
    };
    let center = p(20.0, (right.start.y + right.end.y) / 2.0)?;
    let circle = Geometry::Circle(Circle::new(center, right.length() / 2.0 - 0.01)?);
    draw(&mut d, &mut s, circle, None)?;
    Ok((d, s))
}

fn main() -> Res<()> {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(
        args.next()
            .ok_or("usage: inference_plots <out-dir> [<W>x<H>]")?,
    );
    let mut opts = PlotOptions::default();
    if let Some(size) = args.next() {
        (opts.width, opts.height) = parse_size(&size)?;
    }
    let (drawn, solved) = sketches()?;
    for (s, name) in [
        (drawn, "sketch-infer-drawn.png"),
        (solved, "sketch-infer.png"),
    ] {
        let path = dir.join(name);
        std::fs::write(&path, plot(&s, &opts)?.to_png()?)?;
        println!("wrote {}", path.display());
    }
    Ok(())
}
