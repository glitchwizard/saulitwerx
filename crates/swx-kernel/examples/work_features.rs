//! Builds work features, the reference geometry a part is laid out with:
//! the part origin and its three axes and planes, a plane offset from
//! another, a plane through three points, a plane turned about an axis, the
//! axis where two planes meet, an axis through two points, and the point
//! where an axis crosses a plane. Prints each one, then the errors work
//! features can give. Needs no geometry engine: it runs without `occt`.
//!
//! Used by the user manual ("Work planes, axes and points").
//!
//! Run: `cargo run -p swx-kernel --example work_features`

use swx_kernel::{plane_axis_intersection, Axis, Point3, Workplane, ORIGIN};

/// A number rounded to 3 decimals, with −0 shown as 0.
fn num(v: f64) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    let r = if r == 0.0 { 0.0 } else { r };
    format!("{r}")
}

fn vec3(v: Point3) -> String {
    format!("({}, {}, {})", num(v[0]), num(v[1]), num(v[2]))
}

fn plane(label: &str, p: &Workplane) {
    println!(
        "{label:<22} origin {}  normal {}",
        vec3(p.origin),
        vec3(p.normal)
    );
}

fn axis(label: &str, a: &Axis) {
    println!(
        "{label:<22} through {}  along {}",
        vec3(a.origin),
        vec3(a.dir)
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{:<22} {}", "part origin", vec3(ORIGIN));
    axis("Z axis", &Axis::Z);
    plane("XY plane", &Workplane::XY);

    // A plane 10 mm above the XY plane: the top of a 10 mm plate.
    let top = Workplane::XY.offset(10.0)?;
    plane("XY offset 10", &top);

    // A plane through three corners of a sloping face.
    let slope = Workplane::through_points([0.0, 0.0, 10.0], [40.0, 0.0, 10.0], [0.0, 20.0, 20.0])?;
    plane("through 3 points", &slope);

    // The top plane turned 30° about a line along X across its middle.
    let hinge = Axis {
        origin: [0.0, 20.0, 10.0],
        dir: [1.0, 0.0, 0.0],
    };
    let tilted = top.angled(hinge, 30f64.to_radians())?;
    plane("top turned 30°", &tilted);

    // Where the sloping plane meets the YZ plane.
    let edge = Axis::plane_intersection(&slope, &Workplane::YZ)?;
    axis("slope meets YZ", &edge);

    // An axis through two points, and where it crosses the top plane.
    let diagonal = Axis::through_points([0.0, 0.0, 0.0], [40.0, 20.0, 20.0])?;
    axis("through 2 points", &diagonal);
    let p = plane_axis_intersection(&top, &diagonal)?;
    println!("{:<22} {}", "diagonal meets top", vec3(p));

    // The errors: parallel planes, collinear points, coincident points, an
    // axis parallel to a plane, and an axis that does not lie in the plane.
    let errors = [
        (
            "two parallel planes",
            Axis::plane_intersection(&Workplane::XY, &top).map(|_| ()),
        ),
        (
            "3 points in a line",
            Workplane::through_points(ORIGIN, [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]).map(|_| ()),
        ),
        (
            "2 points the same",
            Axis::through_points([5.0, 5.0, 5.0], [5.0, 5.0, 5.0]).map(|_| ()),
        ),
        (
            "X axis meets top",
            plane_axis_intersection(&top, &Axis::X).map(|_| ()),
        ),
        ("turn about Z axis", top.angled(Axis::Z, 1.0).map(|_| ())),
    ];
    for (label, r) in errors {
        println!("{label} -> error: {}", r.unwrap_err());
    }
    Ok(())
}
