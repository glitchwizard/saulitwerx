//! Measures a part: an angle bracket's mass, center of mass and moments of
//! inertia in aluminium and in steel, then the gap between the bracket and
//! a block beside it, and the wall left between its hole and its end.
//! Shows the errors on purpose, and writes the bracket and the block to a
//! STEP file.
//!
//! Used by the user manual ("Measure a part").
//!
//! Run: `cargo run -p swx-kernel --features occt --example measure_part -- <dir>`

use std::path::PathBuf;

use swx_kernel::{
    BooleanOp, Kernel, OcctKernel, OcctShape, ProfileLoop, Segment2, SubShape, TopoKind, Workplane,
};

type Res<T> = Result<T, Box<dyn std::error::Error>>;

/// Aluminium, 2.70 g/cm³, in grams per mm³.
const ALUMINIUM: f64 = 2.70e-3;
/// Steel, 7.85 g/cm³, in grams per mm³.
const STEEL: f64 = 7.85e-3;

fn face(index: usize) -> SubShape {
    SubShape {
        kind: TopoKind::Face,
        index,
    }
}

/// The first face of `s` whose bounding box passes `test`.
fn find_face(
    k: &OcctKernel,
    s: &OcctShape,
    test: impl Fn([f64; 3], [f64; 3]) -> bool,
) -> Res<usize> {
    for i in 0..k.sub_shape_count(s, TopoKind::Face)? {
        let b = k.sub_shape_bounding_box(s, face(i))?;
        if test(b.min, b.max) {
            return Ok(i);
        }
    }
    Err("no such face".into())
}

fn p3(p: [f64; 3]) -> String {
    format!("({}, {}, {})", n(p[0]), n(p[1]), n(p[2]))
}

/// `x` to 2 decimal places, with no minus sign on a value that rounds to 0.
fn n(x: f64) -> String {
    let x = if x.abs() < 0.005 { 0.0 } else { x };
    format!("{x:.2}")
}

fn main() -> Res<()> {
    let dir = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| ".".into()));
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;

    // An angle bracket: a 60 × 40 × 6 mm foot, a 6 mm upright 40 mm tall,
    // and a hole 10 mm across through the foot, 15 mm from its end.
    let foot = k.make_box([0.0, 0.0, 0.0], [60.0, 40.0, 6.0])?;
    let upright = k.make_box([0.0, 0.0, 6.0], [6.0, 40.0, 34.0])?;
    let hole = k.make_cylinder([45.0, 20.0, -1.0], 5.0, 8.0)?;
    let solid = k.union(&foot, &upright)?;
    let bracket = k.combine(BooleanOp::Cut, &solid, &[hole])?;

    for (name, density) in [("aluminium", ALUMINIUM), ("steel", STEEL)] {
        let m = k.mass_properties(&bracket, density)?;
        let i = m.inertia;
        println!("{name} ({density} g/mm³):");
        println!("  volume {:.2} mm³, area {:.2} mm²", m.volume, m.area);
        println!("  mass {:.2} g", m.mass);
        println!("  center of mass {}", p3(m.center));
        println!(
            "  Ixx {}  Iyy {}  Izz {} g·mm²",
            n(i[0][0]),
            n(i[1][1]),
            n(i[2][2])
        );
        println!(
            "  Ixy {}  Ixz {}  Iyz {} g·mm²",
            n(i[0][1]),
            n(i[0][2]),
            n(i[1][2])
        );
        let p = m.principal;
        println!("  principal {}, {}, {} g·mm²", n(p[0]), n(p[1]), n(p[2]));
    }

    // A 20 × 40 × 20 mm block standing 10 mm from the bracket's foot.
    let block = k.make_box([70.0, 0.0, 0.0], [20.0, 40.0, 20.0])?;
    let gap = k.min_distance(&bracket, None, &block, None)?;
    println!(
        "bracket to block: {:.2} mm, from {} to {}",
        gap.value,
        p3(gap.on_a),
        p3(gap.on_b)
    );

    // The wall between the hole and the end of the foot: from the hole's
    // wall (the face around x = 40..50) to the end face (x = 60).
    let wall = find_face(&k, &bracket, |lo, hi| {
        (lo[0] - 40.0).abs() < 0.01 && (hi[0] - 50.0).abs() < 0.01
    })?;
    let end = find_face(&k, &bracket, |lo, hi| {
        (lo[0] - 60.0).abs() < 0.01 && (hi[0] - 60.0).abs() < 0.01
    })?;
    let d = k.min_distance(&bracket, Some(face(wall)), &bracket, Some(face(end)))?;
    println!(
        "hole wall (face {wall}) to end (face {end}): {:.2} mm, from {} to {}",
        d.value,
        p3(d.on_a),
        p3(d.on_b)
    );

    // Touching shapes: the foot and the upright share a face.
    let touch = k.min_distance(&foot, None, &upright, None)?;
    println!("foot to upright: {:.2} mm", touch.value);

    // Errors, on purpose.
    let disc = k.make_face(
        &Workplane::XY,
        &ProfileLoop {
            segments: vec![Segment2::Circle {
                center: [0.0, 0.0],
                radius: 10.0,
            }],
        },
        &[],
    )?;
    let faces = k.sub_shape_count(&bracket, TopoKind::Face)?;
    for (what, result) in [
        ("density 0", k.mass_properties(&bracket, 0.0).map(|_| ())),
        ("flat disc", k.mass_properties(&disc, ALUMINIUM).map(|_| ())),
        (
            "face 99",
            k.min_distance(&bracket, Some(face(99)), &block, None)
                .map(|_| ()),
        ),
    ] {
        match result {
            Ok(()) => println!("{what} -> ok"),
            Err(e) => println!("{what} -> error: {e}"),
        }
    }
    println!("(the bracket has {faces} faces)");

    let both = k.combine(BooleanOp::Union, &bracket, &[block])?;
    let path = dir.join("bracket-and-block.step");
    k.export_step(&both, &path)?;
    println!("wrote {}", path.display());
    Ok(())
}
