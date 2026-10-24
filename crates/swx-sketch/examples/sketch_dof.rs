//! Builds the plate sketch of `plate_sketch` one group of constraints at a
//! time and reports its degrees of freedom after each group. Then adds a
//! repeated dimension, first agreeing and then disagreeing with the others.
//!
//! Used by the user manual ("Check how fully a sketch is defined").
//!
//! Run: `cargo run -p swx-sketch --example sketch_dof`

use swx_sketch::dof::{analyze, DofOptions};
use swx_sketch::solver::{Residuals, SolveOptions};
use swx_sketch::{Circle, Constraint, Geometry, Line, Point, PointRef, Sketch};

/// Adds `c` and remembers which residual rows it produced, so redundant rows
/// in a report can be traced back to the constraint that made them.
fn add(
    s: &mut Sketch,
    rows: &mut Vec<String>,
    name: &str,
    c: Constraint,
) -> Result<(), Box<dyn std::error::Error>> {
    let before = s.num_residuals();
    s.add_constraint(c)?;
    for _ in before..s.num_residuals() {
        rows.push(name.to_string());
    }
    Ok(())
}

/// Prints one line of the degree-of-freedom report for the sketch as it is.
fn report(stage: &str, s: &Sketch, rows: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let r = analyze(s, s.params(), DofOptions::default())?;
    println!(
        "{stage:<26} equations {:>2}  dof {:>2}  {:?}",
        s.num_residuals(),
        r.dof,
        r.state
    );
    for i in &r.redundant {
        println!("{:<26} redundant: {}", "", rows[*i]);
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut s = Sketch::new();
    let mut rows = Vec::new();
    let p = |x, y| Point::new(x, y);

    let bottom = s.add(Geometry::Line(Line::new(p(0.0, 0.0)?, p(38.0, 1.0)?)?))?;
    let right = s.add(Geometry::Line(Line::new(p(38.0, 1.0)?, p(39.0, 22.0)?)?))?;
    let top = s.add(Geometry::Line(Line::new(p(39.0, 22.0)?, p(1.0, 24.0)?)?))?;
    let left = s.add(Geometry::Line(Line::new(p(1.0, 24.0)?, p(0.0, 0.0)?)?))?;
    let hole = s.add(Geometry::Circle(Circle::new(p(18.0, 11.0)?, 4.0)?))?;
    report("1 rough drawing", &s, &rows)?;

    for (a, b) in [(bottom, right), (right, top), (top, left), (left, bottom)] {
        add(
            &mut s,
            &mut rows,
            "coincident",
            Constraint::Coincident(PointRef::End(a), PointRef::Start(b)),
        )?;
    }
    report("2 + coincident x4", &s, &rows)?;

    for (name, c) in [
        ("horizontal bottom", Constraint::Horizontal(bottom)),
        ("horizontal top", Constraint::Horizontal(top)),
        ("vertical left", Constraint::Vertical(left)),
        ("vertical right", Constraint::Vertical(right)),
    ] {
        add(&mut s, &mut rows, name, c)?;
    }
    report("3 + horizontal, vertical", &s, &rows)?;

    let fixed = Constraint::Fixed {
        point: PointRef::Start(bottom),
        x: 0.0,
        y: 0.0,
    };
    add(&mut s, &mut rows, "fixed corner", fixed)?;
    report("4 + fixed corner", &s, &rows)?;

    let width = |value| Constraint::Distance {
        a: PointRef::Start(bottom),
        b: PointRef::End(bottom),
        value,
    };
    add(&mut s, &mut rows, "distance bottom 40", width(40.0))?;
    add(
        &mut s,
        &mut rows,
        "distance right 25",
        Constraint::Distance {
            a: PointRef::Start(right),
            b: PointRef::End(right),
            value: 25.0,
        },
    )?;
    report("5 + two distances", &s, &rows)?;

    add(
        &mut s,
        &mut rows,
        "fixed hole center",
        Constraint::Fixed {
            point: PointRef::Center(hole),
            x: 20.0,
            y: 12.5,
        },
    )?;
    add(
        &mut s,
        &mut rows,
        "diameter 10",
        Constraint::Diameter {
            geom: hole,
            value: 10.0,
        },
    )?;
    let solved = s.solve(SolveOptions::default())?;
    println!("  solve: {:?}", solved.reason);
    report("6 + hole, then solve", &s, &rows)?;

    // The same sketch with the top line dimensioned too: once agreeing with
    // the bottom (40), once disagreeing (45).
    for value in [40.0, 45.0] {
        let mut t = s.clone();
        let mut t_rows = rows.clone();
        let name = format!("distance top {value}");
        add(
            &mut t,
            &mut t_rows,
            &name,
            Constraint::Distance {
                a: PointRef::Start(top),
                b: PointRef::End(top),
                value,
            },
        )?;
        let solved = t.solve(SolveOptions::default())?;
        println!("  solve: {:?}", solved.reason);
        report(&format!("7 + {name}"), &t, &t_rows)?;
    }
    Ok(())
}
