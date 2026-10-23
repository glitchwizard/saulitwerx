//! Saulitwerx QA tooling.
//!
//! `swx-qa render` draws a [`Mesh`](swx_kernel::Mesh) to a PNG on the CPU:
//! no GPU, no window, deterministic bytes. Builder QA reports (FACTORY §6b)
//! use it to show geometry.
//!
//! `swx-qa sketch` plots a solved [`Sketch`](swx_sketch::Sketch) to a PNG
//! with its dimension values; Builders can also call [`sketch::plot`]
//! directly.

pub mod fixtures;
pub mod render;
pub mod sketch;

use std::path::{Path, PathBuf};

use render::Options;

/// Builds the `--help` text, including the fixture lists.
pub fn usage() -> String {
    format!(
        "\
Usage: swx-qa render (--fixture <name> | --step <file.step>) -o <out.png> [options]
       swx-qa sketch --fixture <name> -o <out.png> [--size <W>x<H>]

render: draws a mesh as an isometric, flat-shaded PNG with feature edges.

Input (exactly one):
  --fixture <name>     built-in mesh: {}
  --step <file.step>   STEP file, imported and tessellated through OCCT
                       (needs a build with `--features occt`)

Options:
  -o, --output <file>  PNG to write (required)
  --size <W>x<H>       image size in pixels (default 800x600)
  --crease <deg>       crease angle for feature edges (default 30)
  -h, --help           show this help

sketch: solves a built-in sketch and plots it top-down (x right, y up)
with its geometry, point dots and dimension values.

  --fixture <name>     built-in sketch (required): {}
  -o, --output <file>  PNG to write (required)
  --size <W>x<H>       image size in pixels (default 800x600)",
        fixtures::FIXTURE_NAMES.join(", "),
        sketch::FIXTURE_NAMES.join(", ")
    )
}

/// Runs the CLI with `args` (without the program name). Returns the message
/// to print on success, or an error message.
pub fn run(args: &[String]) -> Result<String, String> {
    let (cmd, rest) = match args.split_first() {
        Some((c, r)) => (c.as_str(), r),
        None => return Err(format!("missing command\n\n{}", usage())),
    };
    match cmd {
        "-h" | "--help" | "help" => Ok(usage()),
        "render" => render_cmd(rest),
        "sketch" => sketch_cmd(rest),
        other => Err(format!("unknown command `{other}`\n\n{}", usage())),
    }
}

enum Input {
    Fixture(String),
    Step(PathBuf),
}

fn render_cmd(args: &[String]) -> Result<String, String> {
    let mut input = None;
    let mut output = None;
    let mut opts = Options::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("`{arg}` needs a value"))
        };
        match arg.as_str() {
            "-h" | "--help" => return Ok(usage()),
            "--fixture" | "--step" if input.is_some() => {
                return Err("give exactly one of --fixture and --step".into())
            }
            "--fixture" => input = Some(Input::Fixture(value()?)),
            "--step" => input = Some(Input::Step(PathBuf::from(value()?))),
            "-o" | "--output" => output = Some(PathBuf::from(value()?)),
            "--size" => (opts.width, opts.height) = parse_size(&value()?)?,
            "--crease" => {
                let v = value()?;
                opts.crease_deg = v
                    .parse()
                    .map_err(|_| format!("invalid --crease `{v}`: expected degrees"))?;
            }
            other => return Err(format!("unknown option `{other}`\n\n{}", usage())),
        }
    }
    let input = input.ok_or("missing input: give --fixture <name> or --step <file.step>")?;
    let output = output.ok_or("missing output: give -o <out.png>")?;
    let mesh = match &input {
        Input::Fixture(name) => fixtures::fixture(name).ok_or_else(|| {
            format!(
                "unknown fixture `{name}`; available: {}",
                fixtures::FIXTURE_NAMES.join(", ")
            )
        })?,
        Input::Step(path) => step_mesh(path)?,
    };
    let (img, _) = render::render(&mesh, &opts)?;
    let png = img.to_png()?;
    std::fs::write(&output, &png).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    Ok(format!(
        "wrote {} ({}x{}, {} bytes)",
        output.display(),
        img.width,
        img.height,
        png.len()
    ))
}

fn sketch_cmd(args: &[String]) -> Result<String, String> {
    let mut fixture = None;
    let mut output = None;
    let mut opts = sketch::PlotOptions::default();
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || {
            it.next()
                .cloned()
                .ok_or_else(|| format!("`{arg}` needs a value"))
        };
        match arg.as_str() {
            "-h" | "--help" => return Ok(usage()),
            "--fixture" => fixture = Some(value()?),
            "-o" | "--output" => output = Some(PathBuf::from(value()?)),
            "--size" => (opts.width, opts.height) = parse_size(&value()?)?,
            other => return Err(format!("unknown option `{other}`\n\n{}", usage())),
        }
    }
    let name = fixture.ok_or("missing input: give --fixture <name>")?;
    let output = output.ok_or("missing output: give -o <out.png>")?;
    let sk = sketch::fixture(&name).ok_or_else(|| {
        format!(
            "unknown sketch fixture `{name}`; available: {}",
            sketch::FIXTURE_NAMES.join(", ")
        )
    })??;
    let img = sketch::plot(&sk, &opts)?;
    let png = img.to_png()?;
    std::fs::write(&output, &png).map_err(|e| format!("cannot write {}: {e}", output.display()))?;
    Ok(format!(
        "wrote {} ({}x{}, {} bytes)",
        output.display(),
        img.width,
        img.height,
        png.len()
    ))
}

/// Parses `<W>x<H>` with both sides positive integers.
pub fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let err = || format!("invalid --size `{s}`: expected <W>x<H>, e.g. 800x600");
    let (w, h) = s.split_once('x').ok_or_else(err)?;
    let w: u32 = w.parse().map_err(|_| err())?;
    let h: u32 = h.parse().map_err(|_| err())?;
    if w == 0 || h == 0 {
        return Err(format!(
            "invalid --size `{s}`: width and height must be > 0"
        ));
    }
    Ok((w, h))
}

/// Imports a STEP file and tessellates it at 0.1 % of its bounding-box diagonal.
#[cfg(feature = "occt")]
pub fn step_mesh(path: &Path) -> Result<swx_kernel::Mesh, String> {
    use swx_kernel::{Kernel, OcctKernel};
    let k = OcctKernel;
    let shape = k
        .import_step(path)
        .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
    let bb = k.bounding_box(&shape).map_err(|e| e.to_string())?;
    let d = (0..3)
        .map(|i| (bb.max[i] - bb.min[i]).powi(2))
        .sum::<f64>()
        .sqrt();
    let deflection = if d > 0.0 { d * 1e-3 } else { 1e-3 };
    k.tessellate(&shape, deflection).map_err(|e| e.to_string())
}

/// Without the `occt` feature, STEP input is unavailable.
#[cfg(not(feature = "occt"))]
pub fn step_mesh(path: &Path) -> Result<swx_kernel::Mesh, String> {
    Err(format!(
        "cannot read {}: STEP input needs OCCT; rebuild with `cargo build -p swx-qa --features occt`",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(s: &str) -> Vec<String> {
        s.split_whitespace().map(String::from).collect()
    }

    fn out_path(name: &str) -> String {
        std::env::temp_dir()
            .join(format!("swx-qa-test-{}-{name}", std::process::id()))
            .display()
            .to_string()
    }

    #[test]
    fn help_lists_fixtures_and_options() {
        let h = run(&args("render --help")).unwrap();
        for word in fixtures::FIXTURE_NAMES.iter().chain(&[
            "--fixture",
            "--step",
            "--size",
            "--crease",
            "800x600",
            "30",
        ]) {
            assert!(h.contains(word), "help lacks {word}");
        }
    }

    #[test]
    fn renders_fixture_to_png() {
        let out = out_path("cube.png");
        run(&args(&format!(
            "render --fixture cube --size 64x48 -o {out}"
        )))
        .unwrap();
        let bytes = std::fs::read(&out).unwrap();
        std::fs::remove_file(&out).unwrap();
        assert_eq!(&bytes[1..4], b"PNG");
    }

    #[test]
    fn invalid_input_is_an_error() {
        let out = out_path("bad.png");
        let bad = [
            "render --fixture sphere -o OUT",
            "render --fixture cube --size 0x10 -o OUT",
            "render --fixture cube --size 10 -o OUT",
            "render --fixture cube --size axb -o OUT",
            "render --fixture cube --crease x -o OUT",
            "render --step does/not/exist.step -o OUT",
            "render --fixture cube",
            "render -o OUT",
            "render --fixture",
            "frobnicate",
        ];
        for a in bad {
            assert!(run(&args(&a.replace("OUT", &out))).is_err(), "{a}");
        }
        assert!(!std::path::Path::new(&out).exists());
    }

    #[test]
    fn sketch_help_lists_fixtures() {
        let h = run(&args("sketch --help")).unwrap();
        for word in sketch::FIXTURE_NAMES
            .iter()
            .chain(&["sketch", "--size", "-o"])
        {
            assert!(h.contains(word), "help lacks {word}");
        }
    }

    #[test]
    fn plots_sketch_fixtures_to_png() {
        for name in sketch::FIXTURE_NAMES {
            let out = out_path(&format!("{name}.png"));
            let msg = run(&args(&format!(
                "sketch --fixture {name} --size 120x90 -o {out}"
            )))
            .unwrap();
            assert!(msg.contains("120x90"), "{msg}");
            let bytes = std::fs::read(&out).unwrap();
            std::fs::remove_file(&out).unwrap();
            assert_eq!(&bytes[1..4], b"PNG");
        }
    }

    #[test]
    fn invalid_sketch_input_is_an_error() {
        let out = out_path("bad-sketch.png");
        let bad = [
            "sketch --fixture hexagon -o OUT",
            "sketch --fixture rectangle",
            "sketch -o OUT",
            "sketch --fixture rectangle --size 0x10 -o OUT",
            "sketch --fixture rectangle --size 400 -o OUT",
            "sketch --fixture rectangle --size 400x -o OUT",
            "sketch --fixture rectangle --size 9000x10 -o OUT",
            "sketch --fixture rectangle --crease 30 -o OUT",
            "sketch --fixture",
        ];
        for a in bad {
            assert!(run(&args(&a.replace("OUT", &out))).is_err(), "{a}");
        }
        assert!(!std::path::Path::new(&out).exists());
    }

    #[cfg(not(feature = "occt"))]
    #[test]
    fn step_without_occt_says_to_rebuild() {
        let e = run(&args("render --step box.step -o x.png")).unwrap_err();
        assert!(e.contains("--features occt"), "{e}");
    }

    #[cfg(feature = "occt")]
    #[test]
    fn golden_step_box_renders_three_face_shades() {
        use render::{BACKGROUND, EDGE};
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/box_10x20x30.step");
        let mesh = step_mesh(&path).unwrap();
        let opts = Options {
            width: 200,
            height: 200,
            ..Options::default()
        };
        let (img, _) = render::render(&mesh, &opts).unwrap();
        for (x, y) in [(0, 0), (199, 0), (0, 199), (199, 199)] {
            assert_eq!(img.get(x, y), BACKGROUND);
        }
        let shades: std::collections::BTreeSet<_> = img
            .pixels
            .iter()
            .filter(|&&c| c != BACKGROUND && c != EDGE)
            .collect();
        assert_eq!(shades.len(), 3, "{shades:?}");
    }
}
