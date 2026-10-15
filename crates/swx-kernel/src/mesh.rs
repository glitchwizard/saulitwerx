//! Backend-neutral triangle mesh produced by [`Kernel::tessellate`](crate::Kernel::tessellate).

/// An indexed triangle mesh in model space, in single precision for rendering
/// and STL export.
///
/// Invariants (see [`Mesh::validate`]): `positions.len() == normals.len()`,
/// `indices.len() % 3 == 0`, and every index is `< positions.len()`.
/// Triangles are wound counter-clockwise when seen from outside the solid.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Mesh {
    /// Vertex positions.
    pub positions: Vec<[f32; 3]>,
    /// Unit vertex normals, one per position, pointing out of the solid.
    pub normals: Vec<[f32; 3]>,
    /// Triangle vertex indices, three per triangle.
    pub indices: Vec<u32>,
}

impl Mesh {
    /// Checks the structural invariants listed on [`Mesh`].
    pub fn validate(&self) -> Result<(), String> {
        if self.positions.len() != self.normals.len() {
            return Err(format!(
                "{} positions but {} normals",
                self.positions.len(),
                self.normals.len()
            ));
        }
        if self.indices.len() % 3 != 0 {
            return Err(format!(
                "index count {} is not a multiple of 3",
                self.indices.len()
            ));
        }
        let n = self.positions.len();
        if let Some(&bad) = self.indices.iter().find(|&&i| i as usize >= n) {
            return Err(format!("index {bad} out of range for {n} vertices"));
        }
        Ok(())
    }

    /// Number of triangles.
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Enclosed volume of a closed, outward-wound mesh.
    ///
    /// Sums the signed volumes of the tetrahedra formed by the origin and each
    /// triangle, `(a · (b × c)) / 6`, which equals the enclosed volume by the
    /// divergence theorem (Zhang & Chen, "Efficient feature extraction for
    /// 2D/3D objects in mesh representation", ICIP 2001). Accumulates in `f64`.
    pub fn signed_volume(&self) -> f64 {
        let p = |i: u32| self.positions[i as usize].map(f64::from);
        self.indices
            .chunks_exact(3)
            .map(|t| {
                let (a, b, c) = (p(t[0]), p(t[1]), p(t[2]));
                let bxc = [
                    b[1] * c[2] - b[2] * c[1],
                    b[2] * c[0] - b[0] * c[2],
                    b[0] * c[1] - b[1] * c[0],
                ];
                (a[0] * bxc[0] + a[1] * bxc[1] + a[2] * bxc[2]) / 6.0
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit right tetrahedron at the origin, wound outward.
    fn tetra() -> Mesh {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ];
        let normals = vec![[0.0, 0.0, 1.0]; 4];
        let indices = vec![0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
        Mesh {
            positions,
            normals,
            indices,
        }
    }

    #[test]
    fn hand_built_mesh_satisfies_invariants() {
        let m = tetra();
        assert_eq!(m.validate(), Ok(()));
        assert_eq!(m.positions.len(), m.normals.len());
        assert_eq!(m.indices.len() % 3, 0);
        assert!(m.indices.iter().all(|&i| (i as usize) < m.positions.len()));
        assert_eq!(m.triangle_count(), 4);
        assert_eq!(Mesh::default().validate(), Ok(()));
    }

    #[test]
    fn validate_rejects_broken_meshes() {
        let mut m = tetra();
        m.normals.pop();
        assert!(m.validate().is_err());

        let mut m = tetra();
        m.indices.pop();
        assert!(m.validate().is_err());

        let mut m = tetra();
        m.indices[5] = 4;
        assert!(m.validate().is_err());
    }

    #[test]
    fn signed_volume_of_unit_tetra_is_one_sixth() {
        let m = tetra();
        assert!((m.signed_volume() - 1.0 / 6.0).abs() < 1e-12);
        // Flipping every triangle negates the volume.
        let mut inv = m.clone();
        for t in inv.indices.chunks_exact_mut(3) {
            t.swap(1, 2);
        }
        assert!((inv.signed_volume() + 1.0 / 6.0).abs() < 1e-12);
    }
}
