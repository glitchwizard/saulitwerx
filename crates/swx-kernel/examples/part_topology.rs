//! Counts the faces, edges and vertices of the 40 × 25 × 5 mm plate with a
//! 10 mm through hole, and walks from one sub-shape to its neighbours.
//!
//! Used by the user manual ("Faces, edges and vertices of a solid").
//!
//! Run: `cargo run -p swx-kernel --features occt --example part_topology`

use swx_kernel::{Kernel, OcctKernel, SubShape, TopoKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let k = OcctKernel;

    let plate = k.make_box([0.0, 0.0, 0.0], [40.0, 25.0, 5.0])?;
    let tool = k.make_cylinder([20.0, 12.5, -1.0], 5.0, 7.0)?;
    let part = k.cut(&plate, &tool)?;

    // 1. How many of each kind?
    for kind in [TopoKind::Face, TopoKind::Edge, TopoKind::Vertex] {
        println!("{kind:?} count: {}", k.sub_shape_count(&part, kind)?);
    }

    // 2. Going down: the edges of each face.
    println!();
    let faces = k.sub_shape_count(&part, TopoKind::Face)?;
    for index in 0..faces {
        let face = SubShape {
            kind: TopoKind::Face,
            index,
        };
        let edges = k.adjacent(&part, face, TopoKind::Edge)?;
        let list: Vec<usize> = edges.iter().map(|e| e.index).collect();
        println!("face {index}: {} edges {list:?}", edges.len());
    }

    // 3. Going up: the faces that meet at each edge of the hole wall, the
    //    one face with 3 edges.
    println!();
    for index in 0..faces {
        let face = SubShape {
            kind: TopoKind::Face,
            index,
        };
        let edges = k.adjacent(&part, face, TopoKind::Edge)?;
        if edges.len() != 3 {
            continue;
        }
        println!("hole wall is face {index}");
        for edge in edges {
            let around: Vec<usize> = k
                .adjacent(&part, edge, TopoKind::Face)?
                .iter()
                .map(|f| f.index)
                .collect();
            println!("  edge {}: faces {around:?}", edge.index);
        }
    }

    // 4. An index past the end is refused before it reaches the kernel.
    println!();
    let too_far = SubShape {
        kind: TopoKind::Face,
        index: faces,
    };
    match k.adjacent(&part, too_far, TopoKind::Edge) {
        Ok(_) => println!("unexpected: face {faces} was accepted"),
        Err(e) => println!("face {faces}: {e}"),
    }
    Ok(())
}
