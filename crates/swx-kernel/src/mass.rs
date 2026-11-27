//! Mass properties: mass, center of mass, inertia tensor and principal
//! moments (roadmap M6-04a). The backend measures a shape at unit density;
//! everything here is plain arithmetic, testable without a kernel.

use crate::{KernelError, Point3};

/// Mass properties of the solids of a shape, from
/// [`Kernel::mass_properties`](crate::Kernel::mass_properties).
///
/// Conventions: lengths are in model units (mm). `mass` is density ×
/// volume, in whatever mass unit the caller's density implies. `inertia` is
/// the inertia tensor about the center of mass, in the model's axes, with
/// diagonal entries Ixx = ∫ρ((y − cy)² + (z − cz)²) dV (and cyclic) and
/// off-diagonal entries Ixy = −∫ρ(x − cx)(y − cy) dV (the usual
/// inertia-tensor sign; H. Goldstein, *Classical Mechanics*, §5.3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MassProperties {
    /// Enclosed volume of the solids.
    pub volume: f64,
    /// Total area of all faces.
    pub area: f64,
    /// Density × volume.
    pub mass: f64,
    /// Center of mass.
    pub center: Point3,
    /// Inertia tensor about `center`, row-major, symmetric.
    pub inertia: [[f64; 3]; 3],
    /// Principal moments: the eigenvalues of `inertia`, ascending.
    pub principal: [f64; 3],
}

/// Refuses a density that is not finite or is ≤ 0.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn check_density(density: f64) -> Result<(), KernelError> {
    if density.is_finite() && density > 0.0 {
        Ok(())
    } else {
        Err(KernelError::InvalidInput(format!(
            "density must be finite and > 0, got {density}"
        )))
    }
}

/// [`MassProperties`] from unit-density measurements: `unit` holds the
/// volume, the center of mass (3) and the inertia tensor about it (9,
/// row-major), all at density 1. Mass and inertia scale linearly with a
/// uniform density; the center does not.
#[cfg_attr(not(feature = "occt"), allow(dead_code))]
pub(crate) fn scaled(unit: &[f64; 13], area: f64, density: f64) -> MassProperties {
    let mut inertia = [[0.0; 3]; 3];
    for (i, row) in inertia.iter_mut().enumerate() {
        for (j, v) in row.iter_mut().enumerate() {
            *v = density * unit[4 + 3 * i + j];
        }
    }
    MassProperties {
        volume: unit[0],
        area,
        mass: density * unit[0],
        center: [unit[1], unit[2], unit[3]],
        inertia,
        principal: principal_moments(&inertia),
    }
}

/// Largest number of sweeps [`principal_moments`] makes. Cyclic Jacobi
/// converges quadratically; a 3 × 3 matrix needs only a handful.
const MAX_SWEEPS: usize = 50;

/// The eigenvalues of the symmetric 3 × 3 matrix `m`, ascending.
///
/// Cyclic Jacobi eigenvalue method (G. H. Golub and C. F. Van Loan, *Matrix
/// Computations*, §8.5): each sweep visits the off-diagonal pairs (p, q) in
/// row order and applies the rotation J(p, q, θ) that zeroes `a[p][q]`, with
/// t = tan θ the smaller root of t² + 2τt − 1 = 0, τ = (a_qq − a_pp) / 2a_pq
/// (their Algorithm 8.5.1, `sym.schur2`). Sweeps stop when the off-diagonal
/// part is negligible next to the whole matrix. Only the upper triangle of
/// `m` is read.
pub fn principal_moments(m: &[[f64; 3]; 3]) -> [f64; 3] {
    let mut a = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in i..3 {
            a[i][j] = m[i][j];
            a[j][i] = m[i][j];
        }
    }
    let norm = |a: &[[f64; 3]; 3], off_only: bool| -> f64 {
        let mut s = 0.0;
        for (i, row) in a.iter().enumerate() {
            for (j, v) in row.iter().enumerate() {
                if !off_only || i != j {
                    s += v * v;
                }
            }
        }
        s.sqrt()
    };
    let total = norm(&a, false);
    for _ in 0..MAX_SWEEPS {
        if norm(&a, true) <= f64::EPSILON * total {
            break;
        }
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            if a[p][q] == 0.0 {
                continue;
            }
            let tau = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
            let t = if tau >= 0.0 {
                1.0 / (tau + (1.0 + tau * tau).sqrt())
            } else {
                -1.0 / (-tau + (1.0 + tau * tau).sqrt())
            };
            let c = 1.0 / (1.0 + t * t).sqrt();
            let s = t * c;
            // a ← Jᵀ a J, with J the identity except J[p][p] = J[q][q] = c,
            // J[p][q] = s, J[q][p] = −s.
            let mut j = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
            j[p][p] = c;
            j[q][q] = c;
            j[p][q] = s;
            j[q][p] = -s;
            let mut aj = [[0.0; 3]; 3];
            for r in 0..3 {
                for k in 0..3 {
                    aj[r][k] = (0..3).map(|l| a[r][l] * j[l][k]).sum();
                }
            }
            for r in 0..3 {
                for k in 0..3 {
                    a[r][k] = (0..3).map(|l| j[l][r] * aj[l][k]).sum();
                }
            }
            a[p][q] = 0.0;
            a[q][p] = 0.0;
        }
    }
    let mut d = [a[0][0], a[1][1], a[2][2]];
    d.sort_by(f64::total_cmp);
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rel(actual: [f64; 3], expected: [f64; 3]) {
        for k in 0..3 {
            let tol = 1e-9 * expected[k].abs().max(1.0);
            assert!(
                (actual[k] - expected[k]).abs() <= tol,
                "expected {expected:?}, got {actual:?}"
            );
        }
    }

    #[test]
    fn principal_moments_of_a_diagonal_matrix_are_sorted() {
        let m = [[3.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 2.0]];
        assert_eq!(principal_moments(&m), [1.0, 2.0, 3.0]);
    }

    #[test]
    fn principal_moments_of_two_boxes() {
        let m = [
            [1232.0 / 3.0, -400.0, 0.0],
            [-400.0, 1232.0 / 3.0, 0.0],
            [0.0, 0.0, 2432.0 / 3.0],
        ];
        assert_rel(
            principal_moments(&m),
            [32.0 / 3.0, 2432.0 / 3.0, 2432.0 / 3.0],
        );
    }

    #[test]
    fn principal_moments_of_a_full_symmetric_matrix() {
        // [[2, 1], [1, 2]] has eigenvalues 1 and 3; J (all ones) has 0, 0, 3,
        // so 3·I + J has 3, 3, 6.
        let m = [[2.0, 1.0, 0.0], [1.0, 2.0, 0.0], [0.0, 0.0, 4.0]];
        assert_rel(principal_moments(&m), [1.0, 3.0, 4.0]);
        let m = [[4.0, 1.0, 1.0], [1.0, 4.0, 1.0], [1.0, 1.0, 4.0]];
        assert_rel(principal_moments(&m), [3.0, 3.0, 6.0]);
        assert_eq!(principal_moments(&[[0.0; 3]; 3]), [0.0; 3]);
    }

    #[test]
    fn density_must_be_finite_and_positive() {
        assert!(check_density(1.0).is_ok());
        assert!(check_density(7.85e-6).is_ok());
        for d in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(
                matches!(check_density(d), Err(KernelError::InvalidInput(_))),
                "density {d}"
            );
        }
    }

    #[test]
    fn scaling_multiplies_mass_and_inertia_but_not_the_center() {
        let mut unit = [0.0; 13];
        unit[0] = 100.0;
        unit[1..4].copy_from_slice(&[5.0, 2.5, 1.0]);
        unit[4] = 2.0;
        unit[8] = 3.0;
        unit[12] = 1.0;
        let p = scaled(&unit, 160.0, 2.0);
        assert_eq!(p.volume, 100.0);
        assert_eq!(p.area, 160.0);
        assert_eq!(p.mass, 200.0);
        assert_eq!(p.center, [5.0, 2.5, 1.0]);
        assert_eq!(p.inertia[0][0], 4.0);
        assert_eq!(p.inertia[1][1], 6.0);
        assert_eq!(p.principal, [2.0, 4.0, 6.0]);
    }
}
