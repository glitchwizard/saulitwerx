//! Nonlinear least-squares solver for sketch constraints.
//!
//! The solver minimizes `½‖r(x)‖²` over a flat parameter vector `x` (the
//! [`Sketch`](crate::Sketch) layout) with the Levenberg–Marquardt method:
//! each iteration solves the damped normal equations
//!
//! ```text
//! (JᵀJ + λI) δ = −Jᵀr
//! ```
//!
//! and accepts `x + δ` only if it lowers the residual norm. On success λ
//! shrinks, so the step approaches a Gauss–Newton step; on failure λ grows,
//! so the step approaches a short gradient-descent step. The damping term
//! keeps the system positive definite even when `J` is rank deficient, e.g.
//! for under-determined sketches.
//!
//! References: J. Nocedal and S. J. Wright, *Numerical Optimization*, 2nd ed.,
//! Springer 2006, ch. 10 (nonlinear least squares, Levenberg–Marquardt);
//! K. Madsen, H. B. Nielsen and O. Tingleff, *Methods for Non-Linear Least
//! Squares Problems*, 2nd ed., DTU 2004, §3.2 (initial damping
//! `λ₀ = τ·max diag(JᵀJ)`).

use std::fmt;

/// A system of residual equations `r(x) = 0` over `num_params` unknowns.
pub trait Residuals {
    /// Number of residuals (rows of the Jacobian).
    fn num_residuals(&self) -> usize;

    /// Number of parameters (columns of the Jacobian).
    fn num_params(&self) -> usize;

    /// Writes `r(params)` into `out` (length [`num_residuals`](Self::num_residuals)).
    fn residuals(&self, params: &[f64], out: &mut [f64]);

    /// Writes the analytic Jacobian into `jac`, row-major
    /// (`jac[i * num_params + j] = ∂rᵢ/∂xⱼ`), and returns `true`.
    ///
    /// The default returns `false`, and the solver then uses a forward
    /// difference with a relative step of about 1e-7.
    fn jacobian(&self, params: &[f64], jac: &mut [f64]) -> bool {
        let _ = (params, jac);
        false
    }
}

/// Settings for [`solve`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolveOptions {
    /// Converged when the residual norm `‖r‖₂` is at most this.
    pub tolerance: f64,
    /// Maximum number of iterations (attempted steps, accepted or not).
    pub max_iterations: usize,
    /// τ in the initial damping `λ₀ = τ·max diag(JᵀJ)`.
    pub initial_damping: f64,
}

impl Default for SolveOptions {
    fn default() -> Self {
        SolveOptions {
            tolerance: 1e-10,
            max_iterations: 100,
            initial_damping: 1e-6,
        }
    }
}

/// Why [`solve`] stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    /// The residual norm reached the tolerance.
    Converged,
    /// The iteration limit was reached first.
    MaxIterations,
    /// No step can lower the residual norm any further (a least-squares
    /// minimum of an inconsistent system, or a numerically flat region).
    Stagnated,
    /// A parameter, residual or Jacobian entry was NaN or infinite.
    NonFinite,
}

/// Outcome of [`solve`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SolveReport {
    pub converged: bool,
    pub iterations: usize,
    pub final_residual_norm: f64,
    pub reason: StopReason,
}

/// Errors that keep [`solve`] from starting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolveError {
    /// `params.len()` differs from [`Residuals::num_params`].
    DimensionMismatch { expected: usize, got: usize },
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolveError::DimensionMismatch { expected, got } => {
                write!(f, "expected {expected} parameters, got {got}")
            }
        }
    }
}

impl std::error::Error for SolveError {}

/// Damping above this means no useful step is left.
const MAX_DAMPING: f64 = 1e16;
/// Damping never drops below this, so the normal matrix stays definite.
const MIN_DAMPING: f64 = 1e-30;
/// Relative step of the forward-difference Jacobian.
const FD_STEP: f64 = 1e-7;

/// Solves `problem` for `params` in place with Levenberg–Marquardt (see the
/// module docs). A rejected step never changes `params`.
pub fn solve(
    problem: &dyn Residuals,
    params: &mut [f64],
    options: SolveOptions,
) -> Result<SolveReport, SolveError> {
    let n = problem.num_params();
    let m = problem.num_residuals();
    if params.len() != n {
        return Err(SolveError::DimensionMismatch {
            expected: n,
            got: params.len(),
        });
    }

    let report = |iterations, norm, reason| SolveReport {
        converged: reason == StopReason::Converged,
        iterations,
        final_residual_norm: norm,
        reason,
    };

    let mut r = vec![0.0; m];
    if !all_finite(params) {
        return Ok(report(0, f64::NAN, StopReason::NonFinite));
    }
    problem.residuals(params, &mut r);
    let mut norm = norm2(&r);
    if !norm.is_finite() {
        return Ok(report(0, norm, StopReason::NonFinite));
    }

    let mut jac = vec![0.0; m * n];
    let mut normal = vec![0.0; n * n];
    let mut grad = vec![0.0; n];
    let mut step = vec![0.0; n];
    let mut trial = vec![0.0; n];
    let mut trial_r = vec![0.0; m];
    let mut lambda = f64::NAN;
    let mut need_jacobian = true;
    let mut iterations = 0;

    loop {
        if norm <= options.tolerance {
            return Ok(report(iterations, norm, StopReason::Converged));
        }
        if iterations >= options.max_iterations {
            return Ok(report(iterations, norm, StopReason::MaxIterations));
        }

        if need_jacobian {
            if !problem.jacobian(params, &mut jac) {
                forward_difference(problem, params, &r, &mut jac, &mut trial, &mut trial_r);
            }
            if !all_finite(&jac) {
                return Ok(report(iterations, norm, StopReason::NonFinite));
            }
            normal_equations(&jac, &r, m, n, &mut normal, &mut grad);
            if grad.iter().all(|g| g.abs() <= f64::EPSILON * f64::EPSILON) {
                // Stationary point of ½‖r‖² with a non-zero residual.
                return Ok(report(iterations, norm, StopReason::Stagnated));
            }
            if lambda.is_nan() {
                let max_diag = (0..n).map(|i| normal[i * n + i]).fold(0.0, f64::max);
                lambda = (options.initial_damping * max_diag).max(MIN_DAMPING);
            }
            need_jacobian = false;
        }

        iterations += 1;
        let solved = damped_solve(&normal, &grad, lambda, n, &mut step);
        if solved {
            for ((t, p), s) in trial.iter_mut().zip(params.iter()).zip(&step) {
                *t = p + s;
            }
            let moved = trial.iter().zip(params.iter()).any(|(t, p)| t != p);
            if !moved {
                return Ok(report(iterations, norm, StopReason::Stagnated));
            }
            problem.residuals(&trial, &mut trial_r);
            let trial_norm = norm2(&trial_r);
            if trial_norm.is_finite() && trial_norm < norm {
                params.copy_from_slice(&trial);
                r.copy_from_slice(&trial_r);
                norm = trial_norm;
                lambda = (lambda / 10.0).max(MIN_DAMPING);
                need_jacobian = true;
                continue;
            }
        }
        lambda *= 10.0;
        if lambda > MAX_DAMPING {
            return Ok(report(iterations, norm, StopReason::Stagnated));
        }
    }
}

pub(crate) fn all_finite(values: &[f64]) -> bool {
    values.iter().all(|v| v.is_finite())
}

pub(crate) fn norm2(values: &[f64]) -> f64 {
    values.iter().map(|v| v * v).sum::<f64>().sqrt()
}

/// Forward-difference Jacobian, `h = 1e-7·max(|xⱼ|, 1)`.
pub(crate) fn forward_difference(
    problem: &dyn Residuals,
    params: &[f64],
    r: &[f64],
    jac: &mut [f64],
    work: &mut [f64],
    work_r: &mut [f64],
) {
    let n = params.len();
    work.copy_from_slice(params);
    for j in 0..n {
        let h = FD_STEP * params[j].abs().max(1.0);
        work[j] = params[j] + h;
        problem.residuals(work, work_r);
        work[j] = params[j];
        for (i, (rp, r0)) in work_r.iter().zip(r).enumerate() {
            jac[i * n + j] = (rp - r0) / h;
        }
    }
}

/// Computes `JᵀJ` into `normal` and `Jᵀr` into `grad`.
fn normal_equations(
    jac: &[f64],
    r: &[f64],
    m: usize,
    n: usize,
    normal: &mut [f64],
    grad: &mut [f64],
) {
    normal.fill(0.0);
    grad.fill(0.0);
    for i in 0..m {
        let row = &jac[i * n..(i + 1) * n];
        for a in 0..n {
            grad[a] += row[a] * r[i];
            for b in 0..n {
                normal[a * n + b] += row[a] * row[b];
            }
        }
    }
}

/// Solves `(normal + λI) step = −grad` by Cholesky factorization
/// (Golub & Van Loan, *Matrix Computations*, 4th ed., §4.2). Returns `false`
/// if the matrix is not numerically positive definite.
fn damped_solve(normal: &[f64], grad: &[f64], lambda: f64, n: usize, step: &mut [f64]) -> bool {
    let mut l = normal.to_vec();
    for i in 0..n {
        l[i * n + i] += lambda;
    }
    for j in 0..n {
        let mut d = l[j * n + j];
        for k in 0..j {
            d -= l[j * n + k] * l[j * n + k];
        }
        if !d.is_finite() || d <= 0.0 {
            return false;
        }
        let d = d.sqrt();
        l[j * n + j] = d;
        for i in (j + 1)..n {
            let mut s = l[i * n + j];
            for k in 0..j {
                s -= l[i * n + k] * l[j * n + k];
            }
            l[i * n + j] = s / d;
        }
    }
    // Forward substitution L y = −grad, then back substitution Lᵀ step = y.
    for i in 0..n {
        let mut s = -grad[i];
        for k in 0..i {
            s -= l[i * n + k] * step[k];
        }
        step[i] = s / l[i * n + i];
    }
    for i in (0..n).rev() {
        let mut s = step[i];
        for k in (i + 1)..n {
            s -= l[k * n + i] * step[k];
        }
        step[i] = s / l[i * n + i];
    }
    all_finite(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only system given by closures.
    struct Fns<R, J> {
        m: usize,
        n: usize,
        r: R,
        j: Option<J>,
    }

    impl<R, J> Residuals for Fns<R, J>
    where
        R: Fn(&[f64], &mut [f64]),
        J: Fn(&[f64], &mut [f64]),
    {
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
            match &self.j {
                Some(j) => {
                    j(p, jac);
                    true
                }
                None => false,
            }
        }
    }

    type VecFn = fn(&[f64], &mut [f64]);
    type NoJac = VecFn;

    fn residual_only<R: Fn(&[f64], &mut [f64])>(m: usize, n: usize, r: R) -> Fns<R, NoJac> {
        Fns { m, n, r, j: None }
    }

    fn linear() -> Fns<VecFn, VecFn> {
        Fns {
            m: 2,
            n: 2,
            r: |p: &[f64], o: &mut [f64]| {
                o[0] = p[0] + p[1] - 3.0;
                o[1] = p[0] - p[1] - 1.0;
            },
            j: Some(|_: &[f64], j: &mut [f64]| j.copy_from_slice(&[1.0, 1.0, 1.0, -1.0])),
        }
    }

    fn circle_line_residuals(p: &[f64], o: &mut [f64]) {
        o[0] = p[0] * p[0] + p[1] * p[1] - 25.0;
        o[1] = p[1] - p[0] - 1.0;
    }

    fn circle_line() -> Fns<VecFn, VecFn> {
        Fns {
            m: 2,
            n: 2,
            r: circle_line_residuals,
            j: Some(|p: &[f64], j: &mut [f64]| {
                j.copy_from_slice(&[2.0 * p[0], 2.0 * p[1], -1.0, 1.0])
            }),
        }
    }

    #[test]
    fn defaults_match_spec() {
        let o = SolveOptions::default();
        assert_eq!(o.tolerance, 1e-10);
        assert_eq!(o.max_iterations, 100);
    }

    #[test]
    fn a_linear_system_converges_in_at_most_three_iterations() {
        let mut p = [0.0, 0.0];
        let rep = solve(&linear(), &mut p, SolveOptions::default()).unwrap();
        assert!(rep.converged, "{rep:?}");
        assert_eq!(rep.reason, StopReason::Converged);
        assert!(rep.iterations <= 3, "{rep:?}");
        assert!(rep.final_residual_norm < 1e-10);
        assert!(
            (p[0] - 2.0).abs() < 1e-8 && (p[1] - 1.0).abs() < 1e-8,
            "{p:?}"
        );
    }

    #[test]
    fn b_circle_line_intersection_from_1_1() {
        let mut p = [1.0, 1.0];
        let rep = solve(&circle_line(), &mut p, SolveOptions::default()).unwrap();
        assert!(rep.converged, "{rep:?}");
        assert!(
            (p[0] - 3.0).abs() < 1e-8 && (p[1] - 4.0).abs() < 1e-8,
            "{p:?}"
        );
    }

    #[test]
    fn c_forward_difference_matches_analytic() {
        let mut analytic = [1.0, 1.0];
        solve(&circle_line(), &mut analytic, SolveOptions::default()).unwrap();
        let mut fd = [1.0, 1.0];
        let rep = solve(
            &residual_only(2, 2, circle_line_residuals),
            &mut fd,
            SolveOptions::default(),
        )
        .unwrap();
        assert!(rep.converged, "{rep:?}");
        assert!(
            (fd[0] - analytic[0]).abs() < 1e-6 && (fd[1] - analytic[1]).abs() < 1e-6,
            "{fd:?}"
        );
        assert!(
            (fd[0] - 3.0).abs() < 1e-8 && (fd[1] - 4.0).abs() < 1e-8,
            "{fd:?}"
        );
    }

    #[test]
    fn d_already_satisfied_takes_zero_iterations() {
        let mut p = [2.0, 1.0];
        let rep = solve(&linear(), &mut p, SolveOptions::default()).unwrap();
        assert_eq!(rep.iterations, 0);
        assert_eq!(rep.reason, StopReason::Converged);
        assert_eq!(p, [2.0, 1.0]);
    }

    #[test]
    fn e_under_determined_system_converges() {
        // x + 2y = 4: one equation, two unknowns.
        let sys = residual_only(1, 2, |p: &[f64], o: &mut [f64]| {
            o[0] = p[0] + 2.0 * p[1] - 4.0
        });
        let mut p = [0.0, 0.0];
        let rep = solve(&sys, &mut p, SolveOptions::default()).unwrap();
        assert!(rep.converged, "{rep:?}");
        assert!((p[0] + 2.0 * p[1] - 4.0).abs() < 1e-10, "{p:?}");
    }

    #[test]
    fn f_inconsistent_system_does_not_converge() {
        let sys = residual_only(2, 1, |p: &[f64], o: &mut [f64]| {
            o[0] = p[0];
            o[1] = p[0] - 1.0;
        });
        let mut p = [3.0];
        let rep = solve(&sys, &mut p, SolveOptions::default()).unwrap();
        assert!(!rep.converged, "{rep:?}");
        assert!(p[0].is_finite());
        // The least-squares minimum is x = 0.5, residual norm √0.5.
        assert!((p[0] - 0.5).abs() < 1e-6, "{p:?}");
        assert!((rep.final_residual_norm - 0.5f64.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn rejected_step_leaves_params_unchanged() {
        // A residual that is worse everywhere except the start: every step is
        // rejected, and params must stay put.
        let sys = residual_only(1, 1, |p: &[f64], o: &mut [f64]| {
            o[0] = if p[0] == 1.0 { 1.0 } else { 2.0 + p[0] * p[0] };
        });
        let mut p = [1.0];
        let rep = solve(&sys, &mut p, SolveOptions::default()).unwrap();
        assert!(!rep.converged, "{rep:?}");
        assert_eq!(p, [1.0]);
    }

    #[test]
    fn damping_grows_on_failure_and_shrinks_on_success() {
        // Rosenbrock as residuals: needs both rejected and accepted steps.
        let sys = residual_only(2, 2, |p: &[f64], o: &mut [f64]| {
            o[0] = 10.0 * (p[1] - p[0] * p[0]);
            o[1] = 1.0 - p[0];
        });
        let mut p = [-1.2, 1.0];
        let rep = solve(&sys, &mut p, SolveOptions::default()).unwrap();
        assert!(rep.converged, "{rep:?}");
        assert!(
            (p[0] - 1.0).abs() < 1e-8 && (p[1] - 1.0).abs() < 1e-8,
            "{p:?}"
        );
    }

    #[test]
    fn max_iterations_is_reported() {
        let mut p = [1.0, 1.0];
        let opts = SolveOptions {
            max_iterations: 1,
            ..SolveOptions::default()
        };
        let rep = solve(&circle_line(), &mut p, opts).unwrap();
        assert_eq!(rep.reason, StopReason::MaxIterations);
        assert_eq!(rep.iterations, 1);
        assert!(!rep.converged);
    }

    #[test]
    fn non_finite_params_and_residuals_are_reported() {
        let mut p = [f64::NAN, 0.0];
        let rep = solve(&linear(), &mut p, SolveOptions::default()).unwrap();
        assert_eq!(rep.reason, StopReason::NonFinite);
        assert!(!rep.converged);

        let sys = residual_only(1, 1, |_: &[f64], o: &mut [f64]| o[0] = f64::INFINITY);
        let mut p = [0.0];
        let rep = solve(&sys, &mut p, SolveOptions::default()).unwrap();
        assert_eq!(rep.reason, StopReason::NonFinite);

        // NaN everywhere except the start: the residual is finite at x = 0 but
        // its forward-difference Jacobian is not.
        let sys = residual_only(1, 1, |p: &[f64], o: &mut [f64]| {
            o[0] = if p[0] == 0.0 { 1.0 } else { f64::NAN };
        });
        let mut p = [0.0];
        let rep = solve(&sys, &mut p, SolveOptions::default()).unwrap();
        assert_eq!(rep.reason, StopReason::NonFinite);
        assert_eq!(p, [0.0]);
    }

    #[test]
    fn dimension_mismatch_is_an_error() {
        let mut p = [0.0; 3];
        let err = solve(&linear(), &mut p, SolveOptions::default()).unwrap_err();
        assert_eq!(
            err,
            SolveError::DimensionMismatch {
                expected: 2,
                got: 3
            }
        );
    }
}
