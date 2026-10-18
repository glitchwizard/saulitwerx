//! Built-in test meshes, built in pure Rust so renders work without OCCT.
//!
//! Like kernel tessellation output, flat faces get their own vertices, so a
//! position shared by two faces appears once per face.

use swx_kernel::Mesh;

/// Names accepted by [`fixture`], in the order `--help` lists them.
pub const FIXTURE_NAMES: [&str; 3] = ["cube", "cylinder", "l-bracket"];

/// Returns the named built-in mesh, or `None` for an unknown name.
pub fn fixture(name: &str) -> Option<Mesh> {
    match name {
        "cube" => Some(cube()),
        "cylinder" => Some(cylinder()),
        "l-bracket" => Some(l_bracket()),
        _ => None,
    }
}

/// 10 mm cube spanning `[0, 10]³`.
pub fn cube() -> Mesh {
    let square = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    extrude(&square, &[[0, 1, 2], [0, 2, 3]], 10.0)
}

/// L profile with 30 mm legs and 8 mm thickness, extruded 20 mm along +Y.
pub fn l_bracket() -> Mesh {
    let profile = [
        [0.0, 0.0],
        [30.0, 0.0],
        [30.0, 8.0],
        [8.0, 8.0],
        [8.0, 30.0],
        [0.0, 30.0],
    ];
    // Fan from the reflex corner (index 3), which sees every other vertex.
    let caps = [[3, 4, 5], [3, 5, 0], [3, 0, 1], [3, 1, 2]];
    extrude(&profile, &caps, 20.0)
}

/// Seam angle of [`cylinder`]: the +X+Y diagonal, which faces the isometric camera.
pub const CYLINDER_SEAM_ANGLE: f64 = std::f64::consts::FRAC_PI_4;
/// Segment count of [`cylinder`].
pub const CYLINDER_SEGMENTS: usize = 32;

/// Capped cylinder along +Z, radius 5 mm, height 10 mm, 32 segments.
///
/// The side is one strip whose first and last columns coincide at
/// [`CYLINDER_SEAM_ANGLE`], with smooth radial normals, as a kernel would
/// tessellate a periodic face.
pub fn cylinder() -> Mesh {
    let (r, h, n) = (5.0f64, 10.0f64, CYLINDER_SEGMENTS);
    let angle = |i: usize| CYLINDER_SEAM_ANGLE + std::f64::consts::TAU * i as f64 / n as f64;
    let mut m = Mesh::default();

    // Side strip: columns 0..=n, bottom then top vertex per column.
    for i in 0..=n {
        let (s, c) = angle(i % n).sin_cos();
        for z in [0.0, h] {
            m.positions.push([(r * c) as f32, (r * s) as f32, z as f32]);
            m.normals.push([c as f32, s as f32, 0.0]);
        }
    }
    for i in 0..n as u32 {
        let (b0, t0, b1, t1) = (2 * i, 2 * i + 1, 2 * i + 2, 2 * i + 3);
        m.indices.extend_from_slice(&[b0, b1, t1, b0, t1, t0]);
    }

    // Caps: a center vertex fanned to its own ring of rim vertices.
    for (z, nz) in [(0.0, -1.0f32), (h, 1.0)] {
        let center = m.positions.len() as u32;
        m.positions.push([0.0, 0.0, z as f32]);
        m.normals.push([0.0, 0.0, nz]);
        for i in 0..n {
            let (s, c) = angle(i).sin_cos();
            m.positions.push([(r * c) as f32, (r * s) as f32, z as f32]);
            m.normals.push([0.0, 0.0, nz]);
        }
        for i in 0..n as u32 {
            let (a, b) = (center + 1 + i, center + 1 + (i + 1) % n as u32);
            if nz > 0.0 {
                m.indices.extend_from_slice(&[center, a, b]);
            } else {
                m.indices.extend_from_slice(&[center, b, a]);
            }
        }
    }
    m
}

/// Extrudes a counter-clockwise profile in the XZ plane (`[x, z]` pairs)
/// along +Y by `depth`, with `caps` triangulating the profile.
fn extrude(profile: &[[f64; 2]], caps: &[[usize; 3]], depth: f64) -> Mesh {
    let mut m = Mesh::default();
    let at = |p: [f64; 2], y: f64| [p[0], y, p[1]];
    // A profile that is counter-clockwise in (x, z) has normal x × z = −Y,
    // so it faces out of the y = 0 cap as is and is reversed for y = depth.
    for (y, flip) in [(0.0, false), (depth, true)] {
        for t in caps {
            let mut tri = t.map(|i| at(profile[i], y));
            if flip {
                tri.swap(1, 2);
            }
            push_face(&mut m, &tri);
        }
    }
    for i in 0..profile.len() {
        let (a, b) = (profile[i], profile[(i + 1) % profile.len()]);
        push_face(&mut m, &[at(a, 0.0), at(a, depth), at(b, depth)]);
        push_face(&mut m, &[at(a, 0.0), at(b, depth), at(b, 0.0)]);
    }
    m
}

/// Appends one triangle with its own three vertices and its face normal.
fn push_face(m: &mut Mesh, tri: &[[f64; 3]; 3]) {
    let (a, b, c) = (tri[0], tri[1], tri[2]);
    let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let n = [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ];
    let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    let base = m.positions.len() as u32;
    for p in tri {
        m.positions.push(p.map(|x| x as f32));
        m.normals.push(n.map(|x| (x / len) as f32));
    }
    m.indices.extend_from_slice(&[base, base + 1, base + 2]);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rel(actual: f64, expected: f64) {
        assert!(
            ((actual - expected) / expected).abs() < 1e-6,
            "volume {actual}, expected {expected}"
        );
    }

    #[test]
    fn fixtures_are_valid_and_have_expected_volumes() {
        for name in FIXTURE_NAMES {
            assert_eq!(fixture(name).unwrap().validate(), Ok(()), "{name}");
        }
        assert_rel(cube().signed_volume(), 1000.0);
        assert_rel(l_bracket().signed_volume(), 8320.0);
        let faceted = 16.0 * 25.0 * (std::f64::consts::PI / 16.0).sin() * 10.0;
        assert_rel(cylinder().signed_volume(), faceted);
        assert!(fixture("sphere").is_none());
    }

    #[test]
    fn cube_has_separate_vertices_per_face() {
        let m = cube();
        assert_eq!(m.triangle_count(), 12);
        assert_eq!(m.positions.len(), 36);
    }
}
