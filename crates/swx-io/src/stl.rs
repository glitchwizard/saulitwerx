//! STL writers for an indexed triangle mesh, in the binary and the ASCII
//! form.
//!
//! The mesh is plain data: `positions` holds the vertices in millimetres and
//! `indices` holds three vertex indices per triangle, counter-clockwise when
//! seen from outside the solid. Each facet's normal is computed from its
//! three vertices in that order.
//!
//! Both layouts follow the public description of the STL format in
//! M. Burns, *Automated Fabrication: Improving Productivity in
//! Manufacturing*, Prentice Hall, 1993, §6.5 "The STL Format", and the
//! Library of Congress *Sustainability of Digital Formats* descriptions of
//! STL (binary and ASCII):
//!
//! - **Binary:** an 80-byte header, a little-endian `u32` triangle count,
//!   then 50 bytes per triangle: the normal and the three vertices as twelve
//!   little-endian IEEE-754 `f32` values, and a `u16` attribute byte count,
//!   which is 0.
//! - **ASCII:** `solid <name>`, then per triangle `facet normal nx ny nz`,
//!   `outer loop`, three `vertex x y z` lines, `endloop` and `endfacet`, and
//!   finally `endsolid <name>`. Numbers are written in `e` notation
//!   (mantissa, `e`, exponent), the shortest form that reads back to the
//!   same `f32`.
//!
//! The input is checked completely before the first byte is written, so a
//! bad mesh never leaves a partial file behind. Nothing here panics on bad
//! input.
//!
//! [`export_stl`] connects these writers to the kernel: it tessellates a
//! shape at a chosen deflection ([`StlOptions`]) and writes the file.

use std::fmt;
use std::io::Write;
use std::path::Path;

use swx_kernel::{Kernel, KernelError};

/// The fixed 80-byte header of a binary STL file. It is constant text, not a
/// timestamp, so writing the same mesh twice gives byte-identical files. It
/// does not start with `solid`, which some readers take as a sign of an
/// ASCII file.
const BINARY_HEADER: &[u8; 80] =
    b"binary STL written by Saulitwerx                                                ";

/// Why an STL file could not be written.
#[derive(Debug, Clone, PartialEq)]
pub enum StlError {
    /// The number of indices is not a multiple of 3, so it does not describe
    /// whole triangles. Holds the index count.
    IndexCountNotMultipleOf3(usize),
    /// A triangle refers to a vertex that does not exist.
    IndexOutOfRange {
        /// The offending index.
        index: u32,
        /// The number of vertices in `positions`.
        vertex_count: usize,
    },
    /// A vertex has a coordinate that is NaN or infinite.
    NonFiniteCoordinate {
        /// The vertex's position in `positions`.
        vertex: usize,
    },
    /// The mesh has more triangles than the binary format's `u32` count can
    /// hold. Holds the triangle count.
    TooManyTriangles(usize),
    /// Writing to the output failed. Holds the I/O error's message.
    Io(String),
    /// [`StlOptions::deflection_mm`] is not finite or not greater than 0.
    /// Holds the rejected value.
    InvalidDeflection(f64),
    /// The kernel could not tessellate the shape.
    Kernel(KernelError),
}

impl fmt::Display for StlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StlError::IndexCountNotMultipleOf3(n) => {
                write!(f, "index count {n} is not a multiple of 3")
            }
            StlError::IndexOutOfRange {
                index,
                vertex_count,
            } => write!(
                f,
                "index {index} is out of range for a mesh with {vertex_count} vertices"
            ),
            StlError::NonFiniteCoordinate { vertex } => {
                write!(f, "vertex {vertex} has a non-finite coordinate")
            }
            StlError::TooManyTriangles(n) => {
                write!(f, "{n} triangles do not fit in a binary STL file")
            }
            StlError::Io(msg) => write!(f, "could not write STL: {msg}"),
            StlError::InvalidDeflection(d) => write!(
                f,
                "deflection {d} mm is invalid: it must be finite and greater than 0"
            ),
            StlError::Kernel(e) => write!(f, "could not tessellate the shape: {e}"),
        }
    }
}

impl std::error::Error for StlError {}

impl From<std::io::Error> for StlError {
    fn from(e: std::io::Error) -> Self {
        StlError::Io(e.to_string())
    }
}

/// Checks the mesh and returns its triangle count.
fn validate(positions: &[[f32; 3]], indices: &[u32]) -> Result<usize, StlError> {
    if indices.len() % 3 != 0 {
        return Err(StlError::IndexCountNotMultipleOf3(indices.len()));
    }
    if let Some(vertex) = positions
        .iter()
        .position(|p| p.iter().any(|c| !c.is_finite()))
    {
        return Err(StlError::NonFiniteCoordinate { vertex });
    }
    if let Some(&index) = indices
        .iter()
        .find(|&&i| usize::try_from(i).map_or(true, |i| i >= positions.len()))
    {
        return Err(StlError::IndexOutOfRange {
            index,
            vertex_count: positions.len(),
        });
    }
    Ok(indices.len() / 3)
}

/// The unit normal `normalize((b - a) × (c - a))` of a triangle, or
/// `[0, 0, 0]` when the triangle has zero area. Computed in `f64` so thin
/// but valid triangles still get a usable direction. Negative zeros are
/// written as `0`.
fn facet_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    let u = [0, 1, 2].map(|k| f64::from(b[k]) - f64::from(a[k]));
    let v = [0, 1, 2].map(|k| f64::from(c[k]) - f64::from(a[k]));
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    if len == 0.0 || !len.is_finite() {
        return [0.0; 3];
    }
    // `+ 0.0` turns a negative zero into a positive one.
    n.map(|x| (x / len) as f32 + 0.0)
}

/// The three vertices of each triangle. Only called after [`validate`].
fn triangles<'a>(
    positions: &'a [[f32; 3]],
    indices: &'a [u32],
) -> impl Iterator<Item = [[f32; 3]; 3]> + 'a {
    indices
        .chunks_exact(3)
        .map(|t| [0, 1, 2].map(|k| positions[t[k] as usize]))
}

/// Writes the mesh as a binary STL file.
///
/// Layout (see the module documentation for the source): a fixed 80-byte
/// text header (no timestamp), the triangle count as a little-endian `u32`,
/// then 50 bytes per triangle: normal, vertex 1, vertex 2, vertex 3 as
/// little-endian `f32` triples, and a `u16` attribute byte count of 0. The
/// output is exactly `84 + 50 × triangles` bytes long.
///
/// # Errors
///
/// [`StlError::IndexCountNotMultipleOf3`], [`StlError::NonFiniteCoordinate`],
/// [`StlError::IndexOutOfRange`] and [`StlError::TooManyTriangles`] for a bad
/// mesh, checked before anything is written; [`StlError::Io`] when `out`
/// fails.
pub fn write_stl_binary(
    positions: &[[f32; 3]],
    indices: &[u32],
    out: &mut impl Write,
) -> Result<(), StlError> {
    let count = validate(positions, indices)?;
    let count32 = u32::try_from(count).map_err(|_| StlError::TooManyTriangles(count))?;
    out.write_all(BINARY_HEADER)?;
    out.write_all(&count32.to_le_bytes())?;
    let mut record = [0u8; 50];
    for [a, b, c] in triangles(positions, indices) {
        let normal = facet_normal(a, b, c);
        for (slot, value) in [normal, a, b, c].iter().flatten().enumerate() {
            record[slot * 4..slot * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        record[48..50].copy_from_slice(&0u16.to_le_bytes());
        out.write_all(&record)?;
    }
    Ok(())
}

/// Writes the mesh as an ASCII STL file named `name`.
///
/// Layout (see the module documentation for the source):
///
/// ```text
/// solid <name>
///   facet normal nx ny nz
///     outer loop
///       vertex x y z
///       vertex x y z
///       vertex x y z
///     endloop
///   endfacet
/// endsolid <name>
/// ```
///
/// Numbers use `e` notation, e.g. `1e0` or `-2.5e-1`. The name must fit on
/// one line of ASCII text, so control characters and non-ASCII characters in
/// it are replaced by `_`. An empty name gives `solid` and `endsolid` alone.
///
/// # Errors
///
/// [`StlError::IndexCountNotMultipleOf3`], [`StlError::NonFiniteCoordinate`]
/// and [`StlError::IndexOutOfRange`] for a bad mesh, checked before anything
/// is written; [`StlError::Io`] when `out` fails.
pub fn write_stl_ascii(
    name: &str,
    positions: &[[f32; 3]],
    indices: &[u32],
    out: &mut impl Write,
) -> Result<(), StlError> {
    validate(positions, indices)?;
    let name: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii() && !ch.is_ascii_control() {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let sep = if name.is_empty() { "" } else { " " };
    writeln!(out, "solid{sep}{name}")?;
    for [a, b, c] in triangles(positions, indices) {
        let [nx, ny, nz] = facet_normal(a, b, c);
        writeln!(out, "  facet normal {nx:e} {ny:e} {nz:e}")?;
        writeln!(out, "    outer loop")?;
        for [x, y, z] in [a, b, c] {
            writeln!(out, "      vertex {x:e} {y:e} {z:e}")?;
        }
        writeln!(out, "    endloop")?;
        writeln!(out, "  endfacet")?;
    }
    writeln!(out, "endsolid{sep}{name}")?;
    Ok(())
}

/// Which STL layout [`export_stl`] writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StlFormat {
    /// The compact binary layout ([`write_stl_binary`]).
    #[default]
    Binary,
    /// The text layout ([`write_stl_ascii`]).
    Ascii,
}

/// Options for [`export_stl`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StlOptions {
    /// The largest distance, in millimetres, the mesh may be from the true
    /// surface. Smaller values give more triangles. Must be finite and
    /// greater than 0.
    pub deflection_mm: f64,
    /// Binary or ASCII.
    pub format: StlFormat,
}

impl Default for StlOptions {
    /// Deflection 0.01 mm, binary.
    fn default() -> Self {
        StlOptions {
            deflection_mm: 0.01,
            format: StlFormat::Binary,
        }
    }
}

/// Returns `deflection` when it is finite and greater than 0, otherwise
/// [`StlError::InvalidDeflection`].
fn check_deflection(deflection: f64) -> Result<f64, StlError> {
    if deflection.is_finite() && deflection > 0.0 {
        Ok(deflection)
    } else {
        Err(StlError::InvalidDeflection(deflection))
    }
}

/// Tessellates `shape` with `kernel` at `options.deflection_mm` and writes
/// it to `path` as an STL file in `options.format`.
///
/// The deflection is checked before anything else, so an invalid value
/// returns [`StlError::InvalidDeflection`] without tessellating or creating
/// a file. Kernel failures are returned as [`StlError::Kernel`]. The whole
/// file is built in memory first, so a mesh the writers reject leaves no
/// file behind. The ASCII solid name is the file name without its
/// extension.
pub fn export_stl<K: Kernel>(
    kernel: &K,
    shape: &K::Shape,
    path: &Path,
    options: &StlOptions,
) -> Result<(), StlError> {
    let deflection = check_deflection(options.deflection_mm)?;
    let mesh = kernel
        .tessellate(shape, deflection)
        .map_err(StlError::Kernel)?;
    let mut bytes = Vec::new();
    match options.format {
        StlFormat::Binary => write_stl_binary(&mesh.positions, &mesh.indices, &mut bytes)?,
        StlFormat::Ascii => {
            let name = path
                .file_stem()
                .and_then(|n| n.to_str())
                .unwrap_or("saulitwerx");
            write_stl_ascii(name, &mesh.positions, &mesh.indices, &mut bytes)?;
        }
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit cube `[0, 1]³`: vertex `i` has `x = i & 1`, `y = (i >> 1) & 1`,
    /// `z = (i >> 2) & 1`. Twelve triangles, counter-clockwise from outside.
    fn cube() -> (Vec<[f32; 3]>, Vec<u32>) {
        let positions = (0..8u32)
            .map(|i| [(i & 1) as f32, ((i >> 1) & 1) as f32, ((i >> 2) & 1) as f32])
            .collect();
        let indices = vec![
            0, 2, 3, 0, 3, 1, // z = 0
            4, 5, 7, 4, 7, 6, // z = 1
            0, 1, 5, 0, 5, 4, // y = 0
            2, 6, 7, 2, 7, 3, // y = 1
            0, 4, 6, 0, 6, 2, // x = 0
            1, 3, 7, 1, 7, 5, // x = 1
        ];
        (positions, indices)
    }

    fn f32_at(bytes: &[u8], offset: usize) -> f32 {
        f32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    /// The twelve floats (normal + three vertices) of binary triangle `t`.
    fn binary_triangle(bytes: &[u8], t: usize) -> [f32; 12] {
        std::array::from_fn(|k| f32_at(bytes, 84 + 50 * t + 4 * k))
    }

    fn binary(positions: &[[f32; 3]], indices: &[u32]) -> Vec<u8> {
        let mut out = Vec::new();
        write_stl_binary(positions, indices, &mut out).unwrap();
        out
    }

    fn ascii(name: &str, positions: &[[f32; 3]], indices: &[u32]) -> String {
        let mut out = Vec::new();
        write_stl_ascii(name, positions, indices, &mut out).unwrap();
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn cube_binary_has_684_bytes_and_outward_axis_normals() {
        let (positions, indices) = cube();
        let bytes = binary(&positions, &indices);
        assert_eq!(bytes.len(), 684);
        assert_eq!(&bytes[..80], BINARY_HEADER);
        assert_eq!(u32::from_le_bytes(bytes[80..84].try_into().unwrap()), 12);
        for t in 0..12 {
            let f = binary_triangle(&bytes, t);
            let normal = [f[0], f[1], f[2]];
            // Exactly one component is ±1, the others are 0.
            let mut sorted = normal.map(f32::abs);
            sorted.sort_by(f32::total_cmp);
            assert_eq!(sorted, [0.0, 0.0, 1.0], "triangle {t}: {normal:?}");
            // Outward: points from the cube's centre towards the facet.
            let centroid: [f32; 3] =
                std::array::from_fn(|k| (f[3 + k] + f[6 + k] + f[9 + k]) / 3.0);
            let dot: f32 = (0..3).map(|k| normal[k] * (centroid[k] - 0.5)).sum();
            assert!(dot > 0.0, "triangle {t} normal {normal:?} points inward");
            assert_eq!(&bytes[84 + 50 * t + 48..84 + 50 * t + 50], &[0, 0]);
        }
    }

    #[test]
    fn ascii_single_triangle_is_exact() {
        let positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let text = ascii("tri", &positions, &[0, 1, 2]);
        let expected = "solid tri
  facet normal 0e0 0e0 1e0
    outer loop
      vertex 0e0 0e0 0e0
      vertex 1e0 0e0 0e0
      vertex 0e0 1e0 0e0
    endloop
  endfacet
endsolid tri
";
        assert_eq!(text, expected);
    }

    #[test]
    fn ascii_name_is_sanitised_and_may_be_empty() {
        let positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let text = ascii("a\nb é", &positions, &[0, 1, 2]);
        assert!(text.starts_with("solid a_b _\n"), "{text}");
        assert!(text.ends_with("endsolid a_b _\n"), "{text}");
        let text = ascii("", &positions, &[]);
        assert_eq!(text, "solid\nendsolid\n");
    }

    #[test]
    fn binary_and_ascii_agree_on_vertices_and_normals() {
        let (mut positions, indices) = cube();
        // Non-integer coordinates exercise the e-notation round trip.
        for p in &mut positions {
            for c in p.iter_mut() {
                *c = *c * 12.345 - 0.1;
            }
        }
        let bytes = binary(&positions, &indices);
        let text = ascii("cube", &positions, &indices);
        let numbers = |prefix: &str| -> Vec<f32> {
            text.lines()
                .filter_map(|l| l.trim().strip_prefix(prefix))
                .flat_map(|rest| rest.split(' ').map(|s| s.parse::<f32>().unwrap()))
                .collect()
        };
        let vertices = numbers("vertex ");
        let normals = numbers("facet normal ");
        assert_eq!(vertices.len(), 12 * 9);
        assert_eq!(normals.len(), 12 * 3);
        for t in 0..12 {
            let f = binary_triangle(&bytes, t);
            assert_eq!(&f[..3], &normals[3 * t..3 * t + 3], "normal of {t}");
            assert_eq!(&f[3..], &vertices[9 * t..9 * t + 9], "vertices of {t}");
        }
    }

    #[test]
    fn degenerate_triangle_gets_zero_normal_and_is_written() {
        // Collinear points and a repeated vertex.
        let positions = [[0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [2.0, 2.0, 2.0]];
        let indices = [0, 1, 2, 0, 0, 1];
        let bytes = binary(&positions, &indices);
        assert_eq!(bytes.len(), 84 + 2 * 50);
        for t in 0..2 {
            let f = binary_triangle(&bytes, t);
            assert_eq!(&f[..3], &[0.0, 0.0, 0.0]);
            // Positive zeros, not negative ones.
            assert!(f[..3].iter().all(|x| x.is_sign_positive()));
        }
        assert_eq!(binary_triangle(&bytes, 0)[9..], [2.0, 2.0, 2.0]);
        let text = ascii("d", &positions, &indices);
        assert_eq!(text.matches("facet normal 0e0 0e0 0e0").count(), 2);
    }

    #[test]
    fn double_write_is_byte_identical() {
        let (positions, indices) = cube();
        assert_eq!(binary(&positions, &indices), binary(&positions, &indices));
        assert_eq!(
            ascii("c", &positions, &indices),
            ascii("c", &positions, &indices)
        );
    }

    /// Runs both writers on a bad mesh: they must give the same error and
    /// write nothing.
    fn both_fail(positions: &[[f32; 3]], indices: &[u32]) -> StlError {
        let mut out = Vec::new();
        let e1 = write_stl_binary(positions, indices, &mut out).unwrap_err();
        assert!(out.is_empty());
        let e2 = write_stl_ascii("x", positions, indices, &mut out).unwrap_err();
        assert!(out.is_empty());
        assert_eq!(e1, e2);
        e1
    }

    #[test]
    fn index_count_not_multiple_of_3() {
        let (positions, _) = cube();
        assert_eq!(
            both_fail(&positions, &[0, 1, 2, 3]),
            StlError::IndexCountNotMultipleOf3(4)
        );
    }

    #[test]
    fn index_out_of_range_names_index_and_vertex_count() {
        let (positions, _) = cube();
        let e = both_fail(&positions, &[0, 1, 9]);
        assert_eq!(
            e,
            StlError::IndexOutOfRange {
                index: 9,
                vertex_count: 8
            }
        );
        assert_eq!(
            e.to_string(),
            "index 9 is out of range for a mesh with 8 vertices"
        );
    }

    #[test]
    fn non_finite_coordinate() {
        for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let positions = [[0.0, 0.0, 0.0], [1.0, bad, 0.0], [0.0, 1.0, 0.0]];
            assert_eq!(
                both_fail(&positions, &[0, 1, 2]),
                StlError::NonFiniteCoordinate { vertex: 1 }
            );
        }
    }

    #[test]
    fn io_error_is_reported() {
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk full"))
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let (positions, indices) = cube();
        let e = write_stl_binary(&positions, &indices, &mut Broken).unwrap_err();
        assert_eq!(e, StlError::Io("disk full".to_string()));
        let e = write_stl_ascii("c", &positions, &indices, &mut Broken).unwrap_err();
        assert_eq!(e, StlError::Io("disk full".to_string()));
    }

    #[test]
    fn stl_options_default_is_0_01_mm_binary() {
        let o = StlOptions::default();
        assert_eq!(o.deflection_mm, 0.01);
        assert_eq!(o.format, StlFormat::Binary);
        assert_eq!(StlFormat::default(), StlFormat::Binary);
    }

    #[test]
    fn deflection_must_be_finite_and_positive() {
        assert_eq!(check_deflection(0.01), Ok(0.01));
        assert_eq!(check_deflection(1e-9), Ok(1e-9));
        for bad in [0.0, -0.0, -1.0, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(check_deflection(bad), Err(StlError::InvalidDeflection(bad)));
        }
        // NaN never equals itself, so match the variant instead.
        assert!(matches!(
            check_deflection(f64::NAN),
            Err(StlError::InvalidDeflection(d)) if d.is_nan()
        ));
    }

    #[test]
    fn new_errors_name_the_value_and_the_kernel_message() {
        let msg = StlError::InvalidDeflection(0.0).to_string();
        assert!(msg.contains("deflection 0 mm"), "{msg}");
        let msg = StlError::Kernel(KernelError::Backend("boom".into())).to_string();
        assert!(msg.contains("boom"), "{msg}");
    }

    #[cfg(feature = "occt")]
    mod occt {
        use super::super::*;
        use std::collections::HashMap;
        use std::path::PathBuf;
        use swx_kernel::OcctKernel;

        /// A fresh path in the system temp directory.
        fn scratch(name: &str) -> PathBuf {
            use std::sync::atomic::{AtomicUsize, Ordering};
            static N: AtomicUsize = AtomicUsize::new(0);
            let n = N.fetch_add(1, Ordering::Relaxed);
            let p =
                std::env::temp_dir().join(format!("swx-io-stl-{}-{n}-{name}", std::process::id()));
            let _ = std::fs::remove_file(&p);
            p
        }

        /// Reads a binary STL file back as triangles (three vertices each),
        /// checking that the stored count matches the file length.
        fn read_binary(path: &Path) -> Vec<[[f32; 3]; 3]> {
            let bytes = std::fs::read(path).unwrap();
            let count = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
            assert_eq!(bytes.len(), 84 + 50 * count);
            let f = |o: usize| f32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
            (0..count)
                .map(|t| {
                    let base = 84 + 50 * t + 12; // skip the facet normal
                    [0, 1, 2].map(|v| [0, 1, 2].map(|k| f(base + 12 * v + 4 * k)))
                })
                .collect()
        }

        /// Sum of the signed volumes of the tetrahedra (origin, a, b, c).
        fn signed_volume(tris: &[[[f32; 3]; 3]]) -> f64 {
            tris.iter()
                .map(|t| {
                    let [a, b, c] = t.map(|p| p.map(f64::from));
                    (a[0] * (b[1] * c[2] - b[2] * c[1]) - a[1] * (b[0] * c[2] - b[2] * c[0])
                        + a[2] * (b[0] * c[1] - b[1] * c[0]))
                        / 6.0
                })
                .sum()
        }

        fn bbox_size(tris: &[[[f32; 3]; 3]]) -> [f64; 3] {
            let mut lo = [f64::INFINITY; 3];
            let mut hi = [f64::NEG_INFINITY; 3];
            for p in tris.iter().flatten() {
                for k in 0..3 {
                    lo[k] = lo[k].min(f64::from(p[k]));
                    hi[k] = hi[k].max(f64::from(p[k]));
                }
            }
            [0, 1, 2].map(|k| hi[k] - lo[k])
        }

        /// True when every undirected edge, after merging bit-identical
        /// positions, is used by exactly two triangles.
        fn is_closed(tris: &[[[f32; 3]; 3]]) -> bool {
            let mut ids: HashMap<[u32; 3], usize> = HashMap::new();
            let mut edges: HashMap<(usize, usize), usize> = HashMap::new();
            for t in tris {
                let v = t.map(|p| {
                    let key = p.map(|c| (c + 0.0).to_bits());
                    let next = ids.len();
                    *ids.entry(key).or_insert(next)
                });
                for (a, b) in [(v[0], v[1]), (v[1], v[2]), (v[2], v[0])] {
                    *edges.entry((a.min(b), a.max(b))).or_insert(0) += 1;
                }
            }
            !edges.is_empty() && edges.values().all(|&n| n == 2)
        }

        fn export(shape: &swx_kernel::OcctShape, deflection_mm: f64, name: &str) -> PathBuf {
            let path = scratch(name);
            let options = StlOptions {
                deflection_mm,
                format: StlFormat::Binary,
            };
            export_stl(&OcctKernel, shape, &path, &options).unwrap();
            path
        }

        #[test]
        fn invalid_deflection_creates_no_file() {
            let k = OcctKernel;
            let shape = k.make_box([0.0; 3], [20.0, 10.0, 5.0]).unwrap();
            for bad in [0.0, -1.0, f64::NAN, f64::INFINITY] {
                let path = scratch("bad.stl");
                let options = StlOptions {
                    deflection_mm: bad,
                    format: StlFormat::Binary,
                };
                let e = export_stl(&k, &shape, &path, &options).unwrap_err();
                assert!(matches!(e, StlError::InvalidDeflection(_)), "{e:?}");
                assert!(!path.exists());
            }
        }

        #[test]
        fn box_exports_with_its_size_and_volume() {
            let k = OcctKernel;
            let shape = k.make_box([0.0; 3], [20.0, 10.0, 5.0]).unwrap();
            let path = export(&shape, 0.01, "box.stl");
            let tris = read_binary(&path);
            assert!(tris.len() >= 12, "{} triangles", tris.len());
            let size = bbox_size(&tris);
            for (got, want) in size.iter().zip([20.0, 10.0, 5.0]) {
                assert!((got - want).abs() <= 1e-4, "{size:?}");
            }
            let vol = signed_volume(&tris);
            assert!((vol - 1000.0).abs() <= 1e-3, "volume {vol}");
            assert!(is_closed(&tris));
            let _ = std::fs::remove_file(path);
        }

        #[test]
        fn finer_deflection_gives_more_triangles_and_a_closer_volume() {
            let k = OcctKernel;
            let shape = k.make_cylinder([0.0; 3], 5.0, 10.0).unwrap();
            let coarse = read_binary(&export(&shape, 0.1, "cyl-coarse.stl"));
            let fine = read_binary(&export(&shape, 0.01, "cyl-fine.stl"));
            assert!(
                fine.len() > coarse.len(),
                "{} vs {}",
                fine.len(),
                coarse.len()
            );
            let exact = std::f64::consts::PI * 25.0 * 10.0;
            let vol = signed_volume(&fine);
            assert!(
                ((vol - exact) / exact).abs() <= 1e-3,
                "volume {vol}, exact {exact}"
            );
            assert!(is_closed(&coarse));
            assert!(is_closed(&fine));
        }

        #[test]
        fn ascii_export_names_the_solid_after_the_file() {
            let k = OcctKernel;
            let shape = k.make_box([0.0; 3], [20.0, 10.0, 5.0]).unwrap();
            let path = scratch("plate.stl");
            let options = StlOptions {
                deflection_mm: 0.01,
                format: StlFormat::Ascii,
            };
            export_stl(&k, &shape, &path, &options).unwrap();
            let text = std::fs::read_to_string(&path).unwrap();
            let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
            assert!(
                text.starts_with(&format!("solid {stem}\n")),
                "{}",
                &text[..40]
            );
            assert!(text.trim_end().ends_with(&format!("endsolid {stem}")));
            assert!(text.matches("facet normal").count() >= 12);
            let _ = std::fs::remove_file(path);
        }
    }
}
