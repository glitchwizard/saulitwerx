//! Lets Saulitwerx add constraints while a sketch is drawn: a rectangle of
//! about 40 × 25 mm, drawn one side at a time and a little crooked, and a
//! circle that almost touches its top and bottom. After each entity, `infer` lists the
//! constraints it already almost satisfies and `add_inferred` adds them and
//! solves. The last side stops about 1 mm short of the first corner, which the
//! default tolerance doesn't bridge, so it is tried again with a wider one.
//!
//! Used by the user manual ("Let Saulitwerx add constraints as you draw").
//!
//! Run: `cargo run -p swx-sketch --example inferred_sketch`

use swx_sketch::infer::{add_inferred, infer, InferOptions};
use swx_sketch::{Circle, Constraint, GeomId, Geometry, Line, Point, PointRef, Sketch};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

const NAMES: [&str; 5] = ["bottom", "right", "top", "left", "circle"];

fn point(p: PointRef) -> String {
    let (end, GeomId(id)) = match p {
        PointRef::Point(id) => ("point", id),
        PointRef::Start(id) => ("start", id),
        PointRef::End(id) => ("end", id),
        PointRef::Center(id) => ("center", id),
        PointRef::ArcStart(id) => ("arc start", id),
        PointRef::ArcEnd(id) => ("arc end", id),
    };
    format!("{end} of {}", NAMES[id as usize])
}

/// A suggestion in words, such as `coincident: start of right = end of bottom`.
fn describe(c: &Constraint) -> String {
    let name = |id: &GeomId| NAMES[id.0 as usize];
    match c {
        Constraint::Coincident(a, b) => format!("coincident: {} = {}", point(*a), point(*b)),
        Constraint::Horizontal(l) => format!("horizontal: {}", name(l)),
        Constraint::Vertical(l) => format!("vertical: {}", name(l)),
        Constraint::Parallel(a, b) => format!("parallel: {} and {}", name(a), name(b)),
        Constraint::Perpendicular(a, b) => format!("perpendicular: {} and {}", name(a), name(b)),
        Constraint::Tangent(a, b) => format!("tangent: {} and {}", name(a), name(b)),
        other => format!("{other:?}"),
    }
}

fn draw(s: &mut Sketch, geom: Geometry, opts: Option<InferOptions>) -> Res<GeomId> {
    let id = s.add(geom)?;
    let opts = opts.unwrap_or_else(|| InferOptions::for_sketch(s));
    let found = infer(s, id, &opts)?;
    println!("{}: {} suggested", NAMES[id.0 as usize], found.len());
    for c in &found {
        println!("  {}", describe(c));
    }
    add_inferred(s, id, &opts)?;
    Ok(id)
}

fn end_of(s: &Sketch, id: u32) -> Res<(f64, f64)> {
    match s.get(GeomId(id))? {
        Geometry::Line(l) => Ok((l.end.x, l.end.y)),
        _ => Err("not a line".into()),
    }
}

fn corners(s: &Sketch) -> Res<String> {
    let mut out = Vec::new();
    for id in 0..4 {
        if let Geometry::Line(l) = s.get(GeomId(id))? {
            out.push(format!("({:.2}, {:.2})", l.start.x, l.start.y));
        }
    }
    Ok(out.join(" "))
}

fn main() -> Res<()> {
    let mut s = Sketch::new();
    let p = Point::new;
    let line = |a: Point, b: Point| -> Res<Geometry> { Ok(Geometry::Line(Line::new(a, b)?)) };

    // Each side starts where the last one ended, give or take a few
    // hundredths of a millimeter, as a hand on a mouse would.
    draw(&mut s, line(p(0.0, 0.0)?, p(40.0, 0.4)?)?, None)?;
    let (x, y) = end_of(&s, 0)?;
    draw(&mut s, line(p(x + 0.01, y - 0.02)?, p(39.7, 25.0)?)?, None)?;
    let (x, y) = end_of(&s, 1)?;
    draw(&mut s, line(p(x - 0.02, y + 0.01)?, p(0.2, 25.2)?)?, None)?;

    // The last side stops about 1 mm short of the corner where the sketch began.
    let (x, y) = end_of(&s, 2)?;
    let left = line(p(x + 0.01, y + 0.01)?, p(0.1, 1.5)?)?;
    let gap = match (&left, s.get(GeomId(0))?) {
        (Geometry::Line(l), Geometry::Line(b)) => l.end.distance(&b.start),
        _ => 0.0,
    };
    let mut tried = s.clone();
    draw(&mut tried, left, None)?;
    let tol = InferOptions::for_sketch(&tried).point_tol;
    println!("  (missed the first corner by {gap:.2} mm; the default point_tol is {tol:.3} mm)");
    let wide = InferOptions {
        point_tol: 2.0,
        ..InferOptions::for_sketch(&s)
    };
    println!("again with point_tol 2 mm:");
    draw(&mut s, left, Some(wide))?;
    println!("corners: {}", corners(&s)?);

    // A circle drawn to almost touch the top and bottom.
    let mid_y = match s.get(GeomId(1))? {
        Geometry::Line(l) => (l.start.y + l.end.y) / 2.0,
        _ => 0.0,
    };
    let height = match s.get(GeomId(1))? {
        Geometry::Line(l) => l.length(),
        _ => 0.0,
    };
    draw(
        &mut s,
        Geometry::Circle(Circle::new(p(20.0, mid_y)?, height / 2.0 - 0.01)?),
        None,
    )?;
    Ok(())
}
