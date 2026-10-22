//! Degree-of-freedom (DOF) analysis of a residual system.
//!
//! At a parameter vector `x`, the Jacobian `J` (m residuals × n parameters)
//! linearizes the system. Its numerical rank `k` gives:
//!
//! - DOF = n − k: directions in which the geometry can still move;
//! - m − k redundant rows: residuals whose gradients are linear combinations
//!   of earlier rows.
//!
//! A redundant set is *conflicting* when the system can't be satisfied, i.e.
//! the residual norm at `x` (normally a solver result) stays above a
//! tolerance.
//!
//! The rank comes from a Householder QR factorization of `Jᵀ` (Golub & Van
//! Loan, *Matrix Computations*, 4th ed., §5.2.2), so each column of `Jᵀ` is
//! one residual row. Instead of the usual largest-norm column pivoting
//! (§5.4.2), columns are taken in their original order. A column whose norm
//! orthogonal to the columns accepted so far is at most
//! `τ = rank_tolerance · max |Rᵢᵢ|` is dependent on earlier rows: it is
//! recorded as redundant and pivoted to the end. For `max |Rᵢᵢ|` this uses
//! the largest row norm of `J`, which is the first diagonal entry of the
//! column-pivoted factorization and an upper bound on every `|Rᵢᵢ|`.

use crate::solver::{all_finite, forward_difference, norm2, Residuals, SolveError};

/// Settings for [`analyze`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DofOptions {
    /// Relative rank tolerance: a row counts as independent only if its
    /// component orthogonal to the earlier rows exceeds this times
    /// `max |Rᵢᵢ|`.
    pub rank_tolerance: f64,
    /// A system with a redundant row is [`DofState::Conflicting`] when its
    /// residual norm `‖r‖₂` exceeds this.
    pub residual_tolerance: f64,
}

impl Default for DofOptions {
    fn default() -> Self {
        DofOptions {
            rank_tolerance: 1e-9,
            residual_tolerance: 1e-8,
        }
    }
}

/// How well a system is constrained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DofState {
    /// No redundant rows, and at least one degree of freedom is left.
    UnderConstrained,
    /// No redundant rows and no degrees of freedom left.
    FullyConstrained,
    /// At least one redundant row, but the system is satisfied.
    OverConstrained,
    /// At least one redundant row, and the residual norm exceeds the
    /// tolerance: the redundant equations contradict each other.
    Conflicting,
    /// A parameter, residual or Jacobian entry was NaN or infinite. No rank
    /// was computed.
    NonFinite,
}

/// Outcome of [`analyze`].
#[derive(Debug, Clone, PartialEq)]
pub struct DofReport {
    /// Remaining degrees of freedom, `n − rank`.
    pub dof: usize,
    /// Numerical rank of the Jacobian.
    pub rank: usize,
    /// Indices of residual rows that are linearly dependent on earlier rows,
    /// ascending.
    pub redundant: Vec<usize>,
    /// Residual norm `‖r‖₂` at the analyzed parameters.
    pub residual_norm: f64,
    pub state: DofState,
}

/// Analyzes the degrees of freedom of `problem` at `params` (see the module
/// docs). `params` is usually a [`solve`](crate::solver::solve) result.
pub fn analyze(
    problem: &dyn Residuals,
    params: &[f64],
    options: DofOptions,
) -> Result<DofReport, SolveError> {
    let n = problem.num_params();
    let m = problem.num_residuals();
    if params.len() != n {
        return Err(SolveError::DimensionMismatch {
            expected: n,
            got: params.len(),
        });
    }

    let non_finite = |residual_norm| DofReport {
        dof: n,
        rank: 0,
        redundant: Vec::new(),
        residual_norm,
        state: DofState::NonFinite,
    };
    if !all_finite(params) {
        return Ok(non_finite(f64::NAN));
    }
    let mut r = vec![0.0; m];
    problem.residuals(params, &mut r);
    let residual_norm = norm2(&r);
    if !residual_norm.is_finite() {
        return Ok(non_finite(residual_norm));
    }
    let mut jac = vec![0.0; m * n];
    if !problem.jacobian(params, &mut jac) {
        let mut work = vec![0.0; n];
        let mut work_r = vec![0.0; m];
        forward_difference(problem, params, &r, &mut jac, &mut work, &mut work_r);
    }
    if !all_finite(&jac) {
        return Ok(non_finite(residual_norm));
    }

    let (rank, redundant) = ordered_rank(&jac, m, n, options.rank_tolerance);
    let state = if !redundant.is_empty() {
        if residual_norm > options.residual_tolerance {
            DofState::Conflicting
        } else {
            DofState::OverConstrained
        }
    } else if rank < n {
        DofState::UnderConstrained
    } else {
        DofState::FullyConstrained
    };
    Ok(DofReport {
        dof: n - rank,
        rank,
        redundant,
        residual_norm,
        state,
    })
}

/// Householder QR of `Jᵀ` with its columns (the rows of the row-major
/// `m × n` matrix `jac`) taken in order. Returns the rank and the rows that
/// depend on earlier ones (see the module docs).
fn ordered_rank(jac: &[f64], m: usize, n: usize, rank_tolerance: f64) -> (usize, Vec<usize>) {
    let max_norm = (0..m)
        .map(|i| norm2(&jac[i * n..(i + 1) * n]))
        .fold(0.0, f64::max);
    let tau = rank_tolerance * max_norm;

    // Accepted reflectors H = I − β v vᵀ; reflector k is zero above entry k.
    let mut reflectors: Vec<(Vec<f64>, f64)> = Vec::new();
    let mut redundant = Vec::new();
    let mut col = vec![0.0; n];
    for i in 0..m {
        col.copy_from_slice(&jac[i * n..(i + 1) * n]);
        for (v, beta) in &reflectors {
            let s = beta * v.iter().zip(&col).map(|(a, b)| a * b).sum::<f64>();
            for (c, a) in col.iter_mut().zip(v) {
                *c -= s * a;
            }
        }
        let k = reflectors.len();
        let rest = norm2(&col[k..]);
        if rest <= tau || rest == 0.0 {
            redundant.push(i);
            continue;
        }
        // v = x − αe₁ with α = −sign(x₁)‖x‖ avoids cancellation (§5.1.3).
        let alpha = if col[k] >= 0.0 { -rest } else { rest };
        let mut v = vec![0.0; n];
        v[k..].copy_from_slice(&col[k..]);
        v[k] -= alpha;
        let beta = 2.0 / v.iter().map(|a| a * a).sum::<f64>();
        reflectors.push((v, beta));
    }
    (reflectors.len(), redundant)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::{solve, SolveOptions};

    type JacFn = fn(&[f64], &mut [f64]);

    /// Test-only system given by a residual closure (forward-difference
    /// Jacobian) and, optionally, an analytic Jacobian.
    struct Fns<R> {
        m: usize,
        n: usize,
        r: R,
        j: Option<JacFn>,
    }

    impl<R: Fn(&[f64], &mut [f64])> Residuals for Fns<R> {
        fn num_residuals(&self) -> usize {
            self.m
        }
        fn num_params(&self) -> usize {
            self.n
        }
        fn residuals(&self, p: &[f64], out: &mut [f64]) {
            (self.r)(p, out)
        }
        fn jacobian(&self, p: &[f64], jac: &mut [f64]) -> bool {
            match self.j {
                Some(j) => {
                    j(p, jac);
                    true
                }
                None => false,
            }
        }
    }

    fn fns<R: Fn(&[f64], &mut [f64])>(m: usize, n: usize, r: R) -> Fns<R> {
        Fns { m, n, r, j: None }
    }

    fn run<R: Fn(&[f64], &mut [f64])>(sys: &Fns<R>, params: &[f64]) -> DofReport {
        analyze(sys, params, DofOptions::default()).unwrap()
    }

    fn solved<R: Fn(&[f64], &mut [f64])>(sys: &Fns<R>, start: &[f64]) -> Vec<f64> {
        let mut p = start.to_vec();
        solve(sys, &mut p, SolveOptions::default()).unwrap();
        p
    }

    fn point_at_1_2(p: &[f64], o: &mut [f64]) {
        o[0] = p[0] - 1.0;
        o[1] = p[1] - 2.0;
    }

    #[test]
    fn defaults_match_spec() {
        let o = DofOptions::default();
        assert_eq!(o.rank_tolerance, 1e-9);
        assert_eq!(o.residual_tolerance, 1e-8);
    }

    #[test]
    fn a_free_point_has_two_dof() {
        let sys = fns(0, 2, |_: &[f64], _: &mut [f64]| {});
        let rep = run(&sys, &[3.0, -1.0]);
        assert_eq!((rep.dof, rep.rank), (2, 0));
        assert!(rep.redundant.is_empty());
        assert_eq!(rep.state, DofState::UnderConstrained);
    }

    #[test]
    fn b_fixed_point_is_fully_constrained() {
        let sys = Fns {
            m: 2,
            n: 2,
            r: point_at_1_2,
            j: Some(|_: &[f64], j: &mut [f64]| j.copy_from_slice(&[1.0, 0.0, 0.0, 1.0])),
        };
        let rep = run(&sys, &[1.0, 2.0]);
        assert_eq!((rep.dof, rep.rank), (0, 2));
        assert!(rep.redundant.is_empty());
        assert_eq!(rep.state, DofState::FullyConstrained);
    }

    #[test]
    fn c_line_with_fixed_start_horizontal_and_length_is_fully_constrained() {
        // Params x0, y0, x1, y1.
        let sys = fns(4, 4, |p: &[f64], o: &mut [f64]| {
            o[0] = p[0];
            o[1] = p[1];
            o[2] = p[3] - p[1];
            o[3] = (p[2] - p[0]).powi(2) + (p[3] - p[1]).powi(2) - 25.0;
        });
        let p = solved(&sys, &[0.0, 0.0, 4.0, 1.0]);
        assert!((p[2] - 5.0).abs() < 1e-8 && p[3].abs() < 1e-8, "{p:?}");
        let rep = run(&sys, &p);
        assert_eq!((rep.dof, rep.rank), (0, 4));
        assert_eq!(rep.state, DofState::FullyConstrained);
    }

    #[test]
    fn d_duplicate_constraint_is_redundant() {
        let sys = fns(3, 2, |p: &[f64], o: &mut [f64]| {
            point_at_1_2(p, &mut o[..2]);
            o[2] = p[0] - 1.0;
        });
        let rep = run(&sys, &[1.0, 2.0]);
        assert_eq!(rep.rank, 2);
        assert_eq!(rep.dof, 0);
        assert_eq!(rep.redundant, vec![2]);
        assert_eq!(rep.state, DofState::OverConstrained);
    }

    #[test]
    fn e_contradicting_constraint_is_conflicting() {
        let sys = fns(3, 2, |p: &[f64], o: &mut [f64]| {
            point_at_1_2(p, &mut o[..2]);
            o[2] = p[0] - 3.0;
        });
        let p = solved(&sys, &[0.0, 0.0]);
        let rep = run(&sys, &p);
        assert_eq!(rep.state, DofState::Conflicting);
        assert!(rep.redundant.contains(&2), "{rep:?}");
        assert!(rep.residual_norm > 1e-8);
    }

    #[test]
    fn f_circle_with_fixed_center_has_one_dof() {
        // Params cx, cy, r.
        let sys = fns(2, 3, |p: &[f64], o: &mut [f64]| {
            o[0] = p[0] - 2.0;
            o[1] = p[1] + 1.0;
        });
        let rep = run(&sys, &[2.0, -1.0, 4.0]);
        assert_eq!((rep.dof, rep.rank), (1, 2));
        assert_eq!(rep.state, DofState::UnderConstrained);
    }

    #[test]
    fn redundant_rows_are_the_later_ones() {
        // Row 0 equals row 2 scaled, row 1 is independent: row 2 is the
        // redundant one, never row 0.
        let sys = fns(3, 2, |p: &[f64], o: &mut [f64]| {
            o[0] = p[0] + p[1];
            o[1] = p[0] - p[1];
            o[2] = 2.0 * p[0] + 2.0 * p[1];
        });
        let rep = run(&sys, &[0.0, 0.0]);
        assert_eq!(rep.redundant, vec![2]);
        assert_eq!(rep.state, DofState::OverConstrained);
    }

    #[test]
    fn rank_tolerance_is_relative_and_configurable() {
        // Rows (1, 0) and (1, 1e-6): independent at the default tolerance,
        // dependent at a relative tolerance of 1e-3.
        let sys = fns(2, 2, |p: &[f64], o: &mut [f64]| {
            o[0] = p[0];
            o[1] = p[0] + 1e-6 * p[1];
        });
        let j: JacFn = |_, j| j.copy_from_slice(&[1.0, 0.0, 1.0, 1e-6]);
        let sys = Fns { j: Some(j), ..sys };
        assert_eq!(run(&sys, &[0.0, 0.0]).rank, 2);
        let loose = DofOptions {
            rank_tolerance: 1e-3,
            ..DofOptions::default()
        };
        let rep = analyze(&sys, &[0.0, 0.0], loose).unwrap();
        assert_eq!(rep.rank, 1);
        assert_eq!(rep.redundant, vec![1]);

        // Scaling the whole system does not change the rank.
        let scaled: JacFn = |_, j| j.copy_from_slice(&[1e6, 0.0, 1e6, 1.0]);
        let sys = Fns {
            j: Some(scaled),
            ..fns(2, 2, |_: &[f64], o: &mut [f64]| o.fill(0.0))
        };
        assert_eq!(run(&sys, &[0.0, 0.0]).rank, 2);
    }

    #[test]
    fn non_finite_input_is_reported_not_panicking() {
        let sys = fns(2, 2, point_at_1_2);
        assert_eq!(run(&sys, &[f64::NAN, 0.0]).state, DofState::NonFinite);

        let sys = fns(1, 1, |_: &[f64], o: &mut [f64]| o[0] = f64::INFINITY);
        assert_eq!(run(&sys, &[0.0]).state, DofState::NonFinite);

        // Finite at x = 0, NaN anywhere else: the forward difference is NaN.
        let sys = fns(1, 1, |p: &[f64], o: &mut [f64]| {
            o[0] = if p[0] == 0.0 { 1.0 } else { f64::NAN };
        });
        assert_eq!(run(&sys, &[0.0]).state, DofState::NonFinite);

        let sys = Fns {
            j: Some(|_: &[f64], j: &mut [f64]| j[0] = f64::NAN),
            ..fns(1, 1, |_: &[f64], o: &mut [f64]| o[0] = 0.0)
        };
        assert_eq!(run(&sys, &[0.0]).state, DofState::NonFinite);
    }

    #[test]
    fn dimension_mismatch_is_an_error() {
        let sys = fns(2, 2, point_at_1_2);
        let err = analyze(&sys, &[0.0; 3], DofOptions::default()).unwrap_err();
        assert_eq!(
            err,
            SolveError::DimensionMismatch {
                expected: 2,
                got: 3
            }
        );
    }
}
