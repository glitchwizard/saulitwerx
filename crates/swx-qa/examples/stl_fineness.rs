//! Renders the triangles `export_stl` writes for a disc 40 mm across and
//! 8 mm thick, at a coarse and at the default deflection, with the edges
//! between the triangles of its curved side drawn.
//!
//! Used by the user manual ("Save a part as an STL file").
//!
//! Run: `cargo run -p swx-qa --features occt --example stl_fineness -- <out-dir> [<W>x<H>]`
//! Writes `stl-coarse.png` (1 mm) and `stl-fine.png` (0.01 mm).

use std::path::PathBuf;

use swx_kernel::{Kernel, OcctKernel};
use swx_qa::parse_size;
use swx_qa::render::{render, Options};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let dir = PathBuf::from(args.next().unwrap_or_else(|| ".".into()));
    let (width, height) = match args.next() {
        Some(s) => parse_size(&s)?,
        None => (900, 700),
    };
    std::fs::create_dir_all(&dir)?;
    let k = OcctKernel;
    // A round disc, 40 mm across and 8 mm thick: its curved side shows how
    // fine the mesh is.
    let part = k.make_cylinder([0.0, 0.0, 0.0], 20.0, 8.0)?;

    // A crease angle of 0 draws every edge where neighbouring triangles meet
    // at an angle, so the facets of the curved side show.
    let opts = Options {
        width,
        height,
        crease_deg: 0.0,
    };
    for (name, deflection) in [("stl-coarse.png", 1.0), ("stl-fine.png", 0.01)] {
        let mesh = k.tessellate(&part, deflection)?;
        let (image, _) = render(&mesh, &opts)?;
        let path = dir.join(name);
        std::fs::write(&path, image.to_png()?)?;
        println!(
            "wrote {} ({} triangles at {deflection} mm)",
            path.display(),
            mesh.indices.len() / 3
        );
    }
    Ok(())
}
