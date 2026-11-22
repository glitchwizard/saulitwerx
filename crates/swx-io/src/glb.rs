//! Binary glTF 2.0 (GLB) writer for an indexed triangle mesh.
//!
//! The mesh is the same plain data the [`stl`](crate::stl) writers take:
//! `positions` holds the vertices in millimetres and `indices` holds three
//! vertex indices per triangle, counter-clockwise when seen from outside.
//!
//! The layout follows the Khronos *glTF 2.0 Specification*, section
//! "GLB File Format Specification" (and "Accessors", "Buffers and Buffer
//! Views", "Meshes"):
//!
//! - a 12-byte header: magic `0x46546C67` (`glTF`), version `2`, and the
//!   total file length, each a little-endian `u32`;
//! - a JSON chunk (chunk type `0x4E4F534A`), padded with spaces to a
//!   multiple of 4 bytes;
//! - a BIN chunk (chunk type `0x004E4942`), padded with zeros to a multiple
//!   of 4 bytes.
//!
//! Each chunk starts with its length (without the 8-byte chunk header) and
//! its type, both little-endian `u32`. The BIN chunk holds the positions
//! (`f32` × 3 per vertex) followed by the indices (`u32` per index). glTF
//! units are metres, so the positions are stored unchanged in millimetres
//! and the single node carries a `scale` of `0.001` on each axis.
//!
//! The input is checked completely before the first byte is written, so a
//! bad mesh never leaves a partial file behind. Nothing here panics on bad
//! input. The output is deterministic: the same mesh always gives the same
//! bytes.

use std::fmt;
use std::io::Write;

use serde_json::json;

/// GLB header magic, the ASCII bytes `glTF` read as a little-endian `u32`.
const MAGIC: u32 = 0x4654_6C67;
/// GLB container version.
const VERSION: u32 = 2;
/// Chunk type of the JSON chunk (`JSON`).
const CHUNK_JSON: u32 = 0x4E4F_534A;
/// Chunk type of the binary buffer chunk (`BIN\0`).
const CHUNK_BIN: u32 = 0x004E_4942;
/// glTF accessor component type for `f32`.
const FLOAT: u32 = 5126;
/// glTF accessor component type for `u32`.
const UNSIGNED_INT: u32 = 5125;
/// glTF bufferView target for vertex attributes.
const ARRAY_BUFFER: u32 = 34962;
/// glTF bufferView target for indices.
const ELEMENT_ARRAY_BUFFER: u32 = 34963;
/// glTF primitive mode for a triangle list.
const TRIANGLES: u32 = 4;

/// Why a GLB file could not be written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlbError {
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
    /// The mesh has no triangles.
    EmptyMesh,
    /// The file would be longer than the `u32` length field of the GLB
    /// header can hold. Holds the length in bytes.
    TooLarge(u64),
    /// Writing to the output failed. Holds the I/O error's message.
    Io(String),
}

impl fmt::Display for GlbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GlbError::IndexCountNotMultipleOf3(n) => {
                write!(f, "index count {n} is not a multiple of 3")
            }
            GlbError::IndexOutOfRange {
                index,
                vertex_count,
            } => write!(
                f,
                "index {index} is out of range for a mesh with {vertex_count} vertices"
            ),
            GlbError::NonFiniteCoordinate { vertex } => {
                write!(f, "vertex {vertex} has a non-finite coordinate")
            }
            GlbError::EmptyMesh => write!(f, "the mesh has no triangles"),
            GlbError::TooLarge(n) => write!(f, "{n} bytes do not fit in a GLB file"),
            GlbError::Io(msg) => write!(f, "could not write GLB: {msg}"),
        }
    }
}

impl std::error::Error for GlbError {}

impl From<std::io::Error> for GlbError {
    fn from(e: std::io::Error) -> Self {
        GlbError::Io(e.to_string())
    }
}

/// Checks the mesh before anything is written.
fn validate(positions: &[[f32; 3]], indices: &[u32]) -> Result<(), GlbError> {
    if indices.len() % 3 != 0 {
        return Err(GlbError::IndexCountNotMultipleOf3(indices.len()));
    }
    if indices.is_empty() {
        return Err(GlbError::EmptyMesh);
    }
    if let Some(vertex) = positions
        .iter()
        .position(|p| p.iter().any(|c| !c.is_finite()))
    {
        return Err(GlbError::NonFiniteCoordinate { vertex });
    }
    if let Some(&index) = indices
        .iter()
        .find(|&&i| usize::try_from(i).map_or(true, |i| i >= positions.len()))
    {
        return Err(GlbError::IndexOutOfRange {
            index,
            vertex_count: positions.len(),
        });
    }
    Ok(())
}

/// Exact per-axis minimum and maximum of `positions` (non-empty, finite).
fn bounds(positions: &[[f32; 3]]) -> ([f32; 3], [f32; 3]) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in positions {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    (min, max)
}

/// Rounds `n` up to the next multiple of 4.
fn pad4(n: usize) -> usize {
    n.div_ceil(4) * 4
}

/// Writes the mesh as a binary glTF 2.0 (GLB) file.
///
/// `positions` are vertices in millimetres; `indices` hold three vertex
/// indices per triangle, counter-clockwise from outside. The file has one
/// scene with one node (scaled by `0.001` to metres), one mesh with one
/// triangle primitive, a `POSITION` accessor with exact `min`/`max`, and a
/// `u32` index accessor. See the module documentation for the byte layout.
///
/// The mesh is checked first: on error nothing is written to `out`.
pub fn write_glb(
    positions: &[[f32; 3]],
    indices: &[u32],
    out: &mut impl Write,
) -> Result<(), GlbError> {
    validate(positions, indices)?;
    let (min, max) = bounds(positions);

    let position_bytes = positions.len() * 12;
    let index_bytes = indices.len() * 4;
    let bin_len = position_bytes + index_bytes;

    let doc = json!({
        "asset": { "version": "2.0", "generator": "Saulitwerx" },
        "scene": 0,
        "scenes": [ { "nodes": [0] } ],
        "nodes": [ { "mesh": 0, "scale": [0.001, 0.001, 0.001] } ],
        "meshes": [ {
            "primitives": [ {
                "attributes": { "POSITION": 0 },
                "indices": 1,
                "mode": TRIANGLES
            } ]
        } ],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": FLOAT,
                "count": positions.len(),
                "type": "VEC3",
                "min": min,
                "max": max
            },
            {
                "bufferView": 1,
                "componentType": UNSIGNED_INT,
                "count": indices.len(),
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            {
                "buffer": 0,
                "byteOffset": 0,
                "byteLength": position_bytes,
                "target": ARRAY_BUFFER
            },
            {
                "buffer": 0,
                "byteOffset": position_bytes,
                "byteLength": index_bytes,
                "target": ELEMENT_ARRAY_BUFFER
            }
        ],
        "buffers": [ { "byteLength": bin_len } ]
    });
    let json_text = serde_json::to_vec(&doc).map_err(|e| GlbError::Io(e.to_string()))?;

    let json_chunk_len = pad4(json_text.len());
    let bin_chunk_len = pad4(bin_len);
    let total = 12u64 + 8 + json_chunk_len as u64 + 8 + bin_chunk_len as u64;
    let too_large = || GlbError::TooLarge(total);
    let total_u32 = u32::try_from(total).map_err(|_| too_large())?;
    let json_chunk_u32 = u32::try_from(json_chunk_len).map_err(|_| too_large())?;
    let bin_chunk_u32 = u32::try_from(bin_chunk_len).map_err(|_| too_large())?;

    let mut buf = Vec::with_capacity(usize::try_from(total).map_err(|_| too_large())?);
    for word in [MAGIC, VERSION, total_u32, json_chunk_u32, CHUNK_JSON] {
        buf.extend_from_slice(&word.to_le_bytes());
    }
    buf.extend_from_slice(&json_text);
    buf.resize(buf.len() + (json_chunk_len - json_text.len()), b' ');
    for word in [bin_chunk_u32, CHUNK_BIN] {
        buf.extend_from_slice(&word.to_le_bytes());
    }
    for c in positions.iter().flatten() {
        buf.extend_from_slice(&c.to_le_bytes());
    }
    for i in indices {
        buf.extend_from_slice(&i.to_le_bytes());
    }
    buf.resize(buf.len() + (bin_chunk_len - bin_len), 0);

    out.write_all(&buf)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 10 × 20 × 30 mm box: 8 vertices, 12 triangles, outward winding.
    fn box_mesh() -> (Vec<[f32; 3]>, Vec<u32>) {
        let (x, y, z) = (10.0, 20.0, 30.0);
        let positions = vec![
            [0.0, 0.0, 0.0],
            [x, 0.0, 0.0],
            [x, y, 0.0],
            [0.0, y, 0.0],
            [0.0, 0.0, z],
            [x, 0.0, z],
            [x, y, z],
            [0.0, y, z],
        ];
        let indices = vec![
            0, 2, 1, 0, 3, 2, // bottom (-z)
            4, 5, 6, 4, 6, 7, // top (+z)
            0, 1, 5, 0, 5, 4, // front (-y)
            2, 3, 7, 2, 7, 6, // back (+y)
            0, 4, 7, 0, 7, 3, // left (-x)
            1, 2, 6, 1, 6, 5, // right (+x)
        ];
        (positions, indices)
    }

    fn u32_at(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
    }

    fn glb(positions: &[[f32; 3]], indices: &[u32]) -> Vec<u8> {
        let mut out = Vec::new();
        write_glb(positions, indices, &mut out).unwrap();
        out
    }

    /// Splits a GLB file into its JSON value and BIN chunk bytes, checking
    /// the header and chunk framing on the way.
    fn split(bytes: &[u8]) -> (serde_json::Value, &[u8]) {
        assert_eq!(u32_at(bytes, 0), MAGIC);
        assert_eq!(u32_at(bytes, 4), 2);
        assert_eq!(u32_at(bytes, 8) as usize, bytes.len());
        let json_len = u32_at(bytes, 12) as usize;
        assert_eq!(u32_at(bytes, 16), CHUNK_JSON);
        assert_eq!(json_len % 4, 0);
        let json_bytes = &bytes[20..20 + json_len];
        let bin_header = 20 + json_len;
        let bin_len = u32_at(bytes, bin_header) as usize;
        assert_eq!(u32_at(bytes, bin_header + 4), CHUNK_BIN);
        assert_eq!(bin_len % 4, 0);
        let bin = &bytes[bin_header + 8..bin_header + 8 + bin_len];
        assert_eq!(bin_header + 8 + bin_len, bytes.len());
        let text = std::str::from_utf8(json_bytes).unwrap();
        (
            serde_json::from_str(text.trim_end_matches(' ')).unwrap(),
            bin,
        )
    }

    #[test]
    fn box_layout_json_and_bin_are_exact() {
        let (positions, indices) = box_mesh();
        let bytes = glb(&positions, &indices);
        let (doc, bin) = split(&bytes);

        assert_eq!(doc["asset"]["version"], "2.0");
        assert_eq!(doc["scenes"].as_array().unwrap().len(), 1);
        assert_eq!(doc["scenes"][0]["nodes"], json!([0]));
        assert_eq!(doc["nodes"].as_array().unwrap().len(), 1);
        assert_eq!(doc["nodes"][0]["scale"], json!([0.001, 0.001, 0.001]));
        assert_eq!(doc["meshes"].as_array().unwrap().len(), 1);
        let prim = &doc["meshes"][0]["primitives"];
        assert_eq!(prim.as_array().unwrap().len(), 1);
        assert_eq!(prim[0]["mode"], 4);
        assert_eq!(prim[0]["attributes"]["POSITION"], 0);
        assert_eq!(prim[0]["indices"], 1);

        let pos = &doc["accessors"][0];
        assert_eq!(pos["componentType"], 5126);
        assert_eq!(pos["type"], "VEC3");
        assert_eq!(pos["count"], 8);
        assert_eq!(pos["min"], json!([0.0, 0.0, 0.0]));
        assert_eq!(pos["max"], json!([10.0, 20.0, 30.0]));
        let idx = &doc["accessors"][1];
        assert_eq!(idx["componentType"], 5125);
        assert_eq!(idx["type"], "SCALAR");
        assert_eq!(idx["count"], 36);

        assert_eq!(doc["buffers"].as_array().unwrap().len(), 1);
        assert_eq!(doc["buffers"][0]["byteLength"], 240);
        let views = doc["bufferViews"].as_array().unwrap();
        assert_eq!(views.len(), 2);
        assert_eq!(views[0]["byteOffset"], 0);
        assert_eq!(views[0]["byteLength"], 96);
        assert_eq!(views[0]["target"], 34962);
        assert_eq!(views[1]["byteOffset"], 96);
        assert_eq!(views[1]["byteLength"], 144);
        assert_eq!(views[1]["target"], 34963);

        assert_eq!(bin.len(), 240);
        let floats: Vec<f32> = bin[..96]
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let expected: Vec<f32> = positions.iter().flatten().copied().collect();
        assert_eq!(floats, expected);
        let read_indices: Vec<u32> = bin[96..]
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        assert_eq!(read_indices, indices);
    }

    #[test]
    fn json_chunk_is_space_padded() {
        let (positions, indices) = box_mesh();
        let bytes = glb(&positions, &indices);
        let json_len = u32_at(&bytes, 12) as usize;
        let json_bytes = &bytes[20..20 + json_len];
        let text_end = json_bytes.iter().rposition(|&b| b == b'}').unwrap() + 1;
        assert!(json_bytes[text_end..].iter().all(|&b| b == b' '));
        assert!(json_len - text_end < 4);
    }

    #[test]
    fn min_max_are_exact_for_non_round_coordinates() {
        let positions = [[0.1, -2.5, 3.3], [-0.7, 4.25, 1e-3], [5.5, 0.0, -9.9]];
        let bytes = glb(&positions, &[0, 1, 2]);
        let (doc, _) = split(&bytes);
        let read = |key: &str| -> Vec<f32> {
            doc["accessors"][0][key]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap() as f32)
                .collect()
        };
        assert_eq!(read("min"), vec![-0.7, -2.5, -9.9]);
        assert_eq!(read("max"), vec![5.5, 4.25, 3.3]);
    }

    #[test]
    fn double_write_is_byte_identical() {
        let (positions, indices) = box_mesh();
        assert_eq!(glb(&positions, &indices), glb(&positions, &indices));
    }

    fn fails(positions: &[[f32; 3]], indices: &[u32]) -> GlbError {
        let mut out = Vec::new();
        let err = write_glb(positions, indices, &mut out).unwrap_err();
        assert!(out.is_empty(), "output written on error: {err}");
        err
    }

    #[test]
    fn index_count_not_multiple_of_3() {
        let (positions, _) = box_mesh();
        assert_eq!(
            fails(&positions, &[0, 1, 2, 3]),
            GlbError::IndexCountNotMultipleOf3(4)
        );
    }

    #[test]
    fn index_out_of_range() {
        let (positions, _) = box_mesh();
        assert_eq!(
            fails(&positions, &[0, 1, 99]),
            GlbError::IndexOutOfRange {
                index: 99,
                vertex_count: 8
            }
        );
    }

    #[test]
    fn non_finite_coordinate() {
        let (mut positions, indices) = box_mesh();
        positions[5][1] = f32::NAN;
        assert_eq!(
            fails(&positions, &indices),
            GlbError::NonFiniteCoordinate { vertex: 5 }
        );
        positions[5][1] = 20.0;
        positions[2][0] = f32::INFINITY;
        assert_eq!(
            fails(&positions, &indices),
            GlbError::NonFiniteCoordinate { vertex: 2 }
        );
    }

    #[test]
    fn empty_mesh() {
        let (positions, _) = box_mesh();
        assert_eq!(fails(&positions, &[]), GlbError::EmptyMesh);
        assert_eq!(fails(&[], &[]), GlbError::EmptyMesh);
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
        let (positions, indices) = box_mesh();
        let err = write_glb(&positions, &indices, &mut Broken).unwrap_err();
        assert_eq!(err, GlbError::Io("disk full".to_string()));
    }
}
