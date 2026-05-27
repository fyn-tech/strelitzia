//! CVT optimization solvers using argmin.
//!
//! Provides Newton, BFGS, and L-BFGS optimization methods for CVT problems,
//! bridging the CVT energy/gradient/Hessian functions to argmin's trait system.
//!
//! # Usage
//!
//! ```ignore
//! use strelitzia::meshgen::cvt::*;
//! use strelitzia::meshgen::cvt_solvers::*;
//!
//! let domain = Domain1D::new(0.0, 1.0);
//! let density = |x: Real| 1.0;
//! let seeds = uniform_seeds(10, &domain);
//!
//! // Run L-BFGS optimization with default settings
//! let settings = CvtSolverSettings::default();
//! let result = lbfgs_cvt(&domain, seeds, density, &settings);
//! println!("Final energy: {}", result.last().unwrap().energy);
//! ```

use crate::common::Real;
use crate::fields::Field;
use crate::meshgen::cvt::{
    cvt_energy, cvt_gradient, cvt_hessian, CvtDomain, CvtPoint, CvtState, Domain1D,
};

use argmin::core::observers::{Observe, ObserverMode};
use argmin::core::{CostFunction, Error, Executor, Gradient, KV, State};
use argmin::solver::linesearch::MoreThuenteLineSearch;
use argmin::solver::quasinewton::LBFGS;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ============================================================================
// CvtSolverSettings -- centralised solver configuration
// ============================================================================

/// Configuration for CVT optimization solvers.
///
/// # Convergence criterion
///
/// All solvers (Lloyd, Newton) terminate when the **normalised displacement
/// residual** drops below `tol` for `converge_sustain` consecutive iterations:
///
/// ```text
/// r = max_i  ‖ centroid_i − seed_i ‖ / h_i      where h_i = √(m_i / ρ_i)
/// ```
///
/// - `m_i` — cell mass (density-weighted area) from `integrate_cells`.
/// - `ρ_i = ρ(seed_i)` — density evaluated at the seed.
/// - `h_i` — local characteristic cell size.
///
/// This metric is **dimensionless** and **problem-agnostic**: the same `tol`
/// gives consistent behaviour regardless of domain size, density scale, or
/// spatial dimension.
///
/// `max_iter` is a pure emergency backstop (default: `usize::MAX`).
/// Under normal operation the residual criterion terminates the solver long
/// before `max_iter` is reached.
#[derive(Debug, Clone)]
pub struct CvtSolverSettings {
    /// Emergency backstop: solver aborts if this many iterations are reached.
    /// Default is `usize::MAX` (effectively unlimited — rely on `tol` instead).
    pub max_iter: usize,
    /// Convergence tolerance on the normalised displacement residual
    /// (dimensionless). Default is `1e-7`.
    pub tol: Real,
    /// Number of consecutive iterations below `tol` required before the
    /// solver terminates.  Default is 100.
    pub converge_sustain: usize,
}

impl Default for CvtSolverSettings {
    fn default() -> Self {
        Self {
            max_iter: usize::MAX,
            tol: 1e-7,
            converge_sustain: 100,
        }
    }
}

// ============================================================================
// CvtResult -- solver output with timing
// ============================================================================

/// Result of a CVT solver run, including iteration history and timing.
#[derive(Debug, Clone)]
pub struct CvtResult<P: CvtPoint = Real> {
    /// Per-iteration state history.
    pub history: Vec<CvtState<P>>,
    /// Wall-clock time for the solver run.
    pub elapsed: Duration,
    /// True if the convergence criterion was satisfied within `max_iter`.
    pub converged: bool,
    /// Iteration at which the criterion was *first* satisfied (0-indexed).
    /// `None` if the solver hit `max_iter` without converging.
    pub converge_iter: Option<usize>,
    /// Final normalised displacement residual (dimensionless).
    pub final_residual: Real,
}

/// Lightweight result for scalability benchmarks (no history storage).
#[derive(Debug, Clone)]
pub struct CvtBenchResult {
    /// Number of iterations performed.
    pub iterations: usize,
    /// Final CVT energy.
    pub final_energy: Real,
    /// Wall-clock time for the solver run.
    pub elapsed: Duration,
    /// Final normalised displacement residual (same metric as `CvtResult`).
    /// `Real::INFINITY` if the loop never executed.
    pub final_residual: Real,
    /// Whether the convergence criterion was satisfied (vs hitting `max_iter`).
    pub converged: bool,
}

/// Run Lloyd's algorithm returning only timing and final state (no history).
pub fn lloyd_cvt_bench<D, F>(
    domain: &D,
    seeds: Field<D::Point>,
    density: F,
    settings: &CvtSolverSettings,
) -> CvtBenchResult
where
    D: CvtDomain,
    F: Fn(D::Point) -> Real,
{
    let start = Instant::now();
    let mut current_seeds = seeds;
    let mut iters = 0;
    let mut final_energy = 0.0;
    let mut final_residual = Real::INFINITY;
    let mut sustain_count = 0usize;

    for iter in 0..settings.max_iter {
        let data = domain.integrate_cells(current_seeds.as_slice(), &density);
        final_energy = data.energy;
        final_residual = normalised_residual(&current_seeds, &data.centroids, &data.masses, &density);

        iters = iter + 1;
        if final_residual < settings.tol {
            sustain_count += 1;
            if sustain_count >= settings.converge_sustain {
                break;
            }
        } else {
            sustain_count = 0;
        }
        current_seeds = data.centroids;
    }

    CvtBenchResult {
        iterations: iters,
        final_energy,
        elapsed: start.elapsed(),
        final_residual,
        converged: sustain_count >= settings.converge_sustain,
    }
}

/// Run Newton's method returning only timing and final state (no history).
pub fn newton_cvt_bench<F>(
    domain: &Domain1D,
    seeds: Field<Real>,
    density: F,
    settings: &CvtSolverSettings,
) -> CvtBenchResult
where
    F: Fn(Real) -> Real,
{
    let start = Instant::now();
    let mut current_seeds = seeds;
    let mut iters = 0;
    let mut final_energy = 0.0;
    let mut final_residual = Real::INFINITY;
    let mut sustain_count = 0usize;

    for iter in 0..settings.max_iter {
        let data = domain.integrate_cells(current_seeds.as_slice(), &density);
        final_residual = normalised_residual(&current_seeds, &data.centroids, &data.masses, &density);
        final_energy = data.energy;
        iters = iter + 1;

        if final_residual < settings.tol {
            sustain_count += 1;
            if sustain_count >= settings.converge_sustain {
                break;
            }
        } else {
            sustain_count = 0;
        }

        let grad = cvt_gradient(domain, &current_seeds, &density);
        let hess = cvt_hessian(domain, &current_seeds, &density);
        let grad_vec: Vec<Real> = grad.iter().copied().collect();

        if let Some(step) = solve_tridiagonal(&hess, &grad_vec) {
            let new_seeds: Field<Real> = current_seeds
                .iter()
                .zip(step.iter())
                .map(|(s, d)| s - d)
                .collect();
            current_seeds = new_seeds;
        } else {
            break;
        }
    }
    CvtBenchResult {
        iterations: iters,
        final_energy,
        elapsed: start.elapsed(),
        final_residual,
        converged: sustain_count >= settings.converge_sustain,
    }
}

/// Run L-BFGS returning only timing and final state (no history).
pub fn lbfgs_cvt_bench<F>(
    domain: &Domain1D,
    seeds: Field<Real>,
    density: F,
    _settings: &CvtSolverSettings,
) -> CvtBenchResult
where
    F: Fn(Real) -> Real + Copy,
{
    let start = Instant::now();

    let problem = Cvt1dProblem::new(domain, density);
    let init_param: Vec<Real> = seeds.iter().copied().collect();

    let linesearch = MoreThuenteLineSearch::new();
    let solver = LBFGS::new(linesearch, 7);

    const ARGMIN_MAX_ITERS: u64 = 100_000;
    let result = Executor::new(problem, solver)
        .configure(|state| state.param(init_param).max_iters(ARGMIN_MAX_ITERS))
        .run();

    let (iters, final_energy, converged) = match result {
        Ok(res) => {
            let state = res.state();
            let param = state.get_best_param().cloned().unwrap_or_default();
            let field: Field<Real> = param.into_iter().collect();
            let hit_max = state.get_iter() >= ARGMIN_MAX_ITERS;
            (state.get_iter() as usize, cvt_energy(domain, &field, &density), !hit_max)
        }
        Err(e) => {
            eprintln!("L-BFGS bench error: {}", e);
            (0, f64::NAN, false)
        }
    };

    CvtBenchResult {
        iterations: iters,
        final_energy,
        elapsed: start.elapsed(),
        final_residual: Real::NAN,
        converged,
    }
}

// ============================================================================
// CvtHistoryObserver -- records per-iteration state from argmin solvers
// ============================================================================

struct CvtHistoryObserver {
    history: Arc<Mutex<Vec<(u64, Vec<Real>, Real)>>>,
}

impl<I> Observe<I> for CvtHistoryObserver
where
    I: State<Param = Vec<Real>, Float = Real>,
{
    fn observe_iter(&mut self, state: &I, _kv: &KV) -> Result<(), Error> {
        if let Some(param) = state.get_param() {
            self.history
                .lock()
                .unwrap()
                .push((state.get_iter(), param.clone(), state.get_cost()));
        }
        Ok(())
    }
}

fn collect_history(
    init_seeds: Field<Real>,
    init_energy: Real,
    raw: &[(u64, Vec<Real>, Real)],
) -> Vec<CvtState<Real>> {
    let mut history = Vec::with_capacity(1 + raw.len());
    history.push(CvtState {
        iteration: 0,
        seeds: init_seeds,
        energy: init_energy,
        residual: Real::NAN,
        max_mass: Real::NAN,
        min_mass: Real::NAN,
    });
    for (iter, param, cost) in raw {
        history.push(CvtState {
            iteration: *iter as usize,
            seeds: param.iter().copied().collect(),
            energy: *cost,
            residual: Real::NAN,
            max_mass: Real::NAN,
            min_mass: Real::NAN,
        });
    }
    history
}

// ============================================================================
// Cvt1dProblem -- generic bridge struct for 1D CVT with argmin
// ============================================================================

/// Bridge struct that implements argmin traits for a 1D CVT optimization problem.
///
/// Wraps a `Domain1D` and a density function closure.
pub struct Cvt1dProblem<'a, F>
where
    F: Fn(Real) -> Real,
{
    domain: &'a Domain1D,
    density: F,
}

impl<'a, F> Cvt1dProblem<'a, F>
where
    F: Fn(Real) -> Real,
{
    /// Create a new 1D CVT optimization problem.
    pub fn new(domain: &'a Domain1D, density: F) -> Self {
        Self { domain, density }
    }
}

// ============================================================================
// argmin trait implementations for Cvt1dProblem
// ============================================================================

impl<F> CostFunction for Cvt1dProblem<'_, F>
where
    F: Fn(Real) -> Real,
{
    type Param = Vec<Real>;
    type Output = Real;

    fn cost(&self, param: &Self::Param) -> Result<Self::Output, Error> {
        let seeds: Field<Real> = param.iter().copied().collect();
        Ok(cvt_energy(self.domain, &seeds, &self.density))
    }
}

impl<F> Gradient for Cvt1dProblem<'_, F>
where
    F: Fn(Real) -> Real,
{
    type Param = Vec<Real>;
    type Gradient = Vec<Real>;

    fn gradient(&self, param: &Self::Param) -> Result<Self::Gradient, Error> {
        let seeds: Field<Real> = param.iter().copied().collect();
        let grad = cvt_gradient(self.domain, &seeds, &self.density);
        Ok(grad.iter().copied().collect())
    }
}

// ============================================================================
// Convenience wrapper functions
// ============================================================================

/// Run Lloyd's algorithm for CVT optimization with convergence checking.
///
/// See [`CvtSolverSettings`] for the convergence criterion: the normalised
/// displacement residual must remain below `tol` for `converge_sustain`
/// consecutive iterations.
///
/// # Returns
/// `CvtResult` with iteration history, timing, and convergence metadata.
pub fn lloyd_cvt<D, F>(
    domain: &D,
    seeds: Field<D::Point>,
    density: F,
    settings: &CvtSolverSettings,
) -> CvtResult<D::Point>
where
    D: CvtDomain,
    F: Fn(D::Point) -> Real,
{
    let start = Instant::now();
    let mut history = Vec::new();
    let mut current_seeds = seeds;
    let mut sustain_count = 0usize;
    let mut converge_iter: Option<usize> = None;
    let mut final_residual = Real::INFINITY;

    for iter in 0..settings.max_iter {
        let data = domain.integrate_cells(current_seeds.as_slice(), &density);

        let residual = normalised_residual(
            &current_seeds,
            &data.centroids,
            &data.masses,
            &density,
        );
        final_residual = residual;

        let max_mass = data.masses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_mass = data.masses.iter().copied().fold(f64::INFINITY, f64::min);
        history.push(CvtState {
            iteration: iter,
            seeds: current_seeds,
            energy: data.energy,
            residual,
            max_mass,
            min_mass,
        });

        if residual < settings.tol {
            if converge_iter.is_none() {
                converge_iter = Some(iter);
            }
            sustain_count += 1;
            if sustain_count >= settings.converge_sustain {
                break;
            }
        } else {
            sustain_count = 0;
        }

        current_seeds = data.centroids;
    }

    let converged = sustain_count >= settings.converge_sustain;

    CvtResult {
        history,
        elapsed: start.elapsed(),
        converged,
        converge_iter,
        final_residual,
    }
}

/// Run Lloyd's algorithm without storing intermediate history.
///
/// Identical convergence logic to [`lloyd_cvt`], but uses O(N) memory
/// throughout regardless of iteration count — only the final seed positions
/// are retained.  Use this for large N where storing the full history would
/// be prohibitive.
///
/// The returned `CvtResult::history` contains exactly one entry: the final
/// state.  `CvtResult::history[0].iteration` holds the actual iteration
/// index at termination so that downstream reporting works unchanged.
pub fn lloyd_cvt_no_history<D, F>(
    domain: &D,
    seeds: Field<D::Point>,
    density: F,
    settings: &CvtSolverSettings,
) -> CvtResult<D::Point>
where
    D: CvtDomain,
    F: Fn(D::Point) -> Real,
{
    let start = Instant::now();
    let mut current_seeds = seeds;
    let mut sustain_count = 0usize;
    let mut converge_iter: Option<usize> = None;
    let mut final_residual = Real::INFINITY;
    let mut final_energy = 0.0;
    let mut final_iter = 0usize;

    for iter in 0..settings.max_iter {
        let data = domain.integrate_cells(current_seeds.as_slice(), &density);
        final_energy = data.energy;
        final_residual = normalised_residual(&current_seeds, &data.centroids, &data.masses, &density);
        final_iter = iter;

        if final_residual < settings.tol {
            if converge_iter.is_none() {
                converge_iter = Some(iter);
            }
            sustain_count += 1;
            if sustain_count >= settings.converge_sustain {
                break;
            }
        } else {
            sustain_count = 0;
        }

        current_seeds = data.centroids;
    }

    let converged = sustain_count >= settings.converge_sustain;

    CvtResult {
        history: vec![CvtState {
            iteration: final_iter,
            seeds: current_seeds,
            energy: final_energy,
            residual: final_residual,
            max_mass: Real::NAN,
            min_mass: Real::NAN,
        }],
        elapsed: start.elapsed(),
        converged,
        converge_iter,
        final_residual,
    }
}

/// Run L-BFGS (limited-memory BFGS) for 1D CVT optimization.
///
/// L-BFGS stores only the last `m` gradient differences, making it
/// memory-efficient. This is the recommended solver for most CVT problems.
///
/// # Returns
/// `CvtResult` with iteration history and wall-clock timing.
pub fn lbfgs_cvt<F>(
    domain: &Domain1D,
    seeds: Field<Real>,
    density: F,
    _settings: &CvtSolverSettings,
) -> CvtResult<Real>
where
    F: Fn(Real) -> Real + Copy,
{
    let start = Instant::now();

    let problem = Cvt1dProblem::new(domain, density);
    let init_param: Vec<Real> = seeds.iter().copied().collect();
    let init_energy = cvt_energy(domain, &seeds, &density);

    let linesearch = MoreThuenteLineSearch::new();
    let solver = LBFGS::new(linesearch, 7);

    let history_data = Arc::new(Mutex::new(Vec::new()));
    let observer = CvtHistoryObserver {
        history: Arc::clone(&history_data),
    };

    const ARGMIN_MAX_ITERS: u64 = 100_000;
    let result = Executor::new(problem, solver)
        .configure(|state| state.param(init_param).max_iters(ARGMIN_MAX_ITERS))
        .add_observer(observer, ObserverMode::Always)
        .run();

    let hit_max = result
        .as_ref()
        .map(|r| r.state().get_iter() >= ARGMIN_MAX_ITERS)
        .unwrap_or(true);
    if let Err(e) = &result {
        eprintln!("L-BFGS optimization error: {}", e);
    }

    let raw = history_data.lock().unwrap();
    let n_iters = raw.len();
    CvtResult {
        history: collect_history(seeds, init_energy, &raw),
        elapsed: start.elapsed(),
        converged: !hit_max,
        converge_iter: if hit_max { None } else { Some(n_iters.saturating_sub(1)) },
        final_residual: Real::NAN,
    }
}

/// Run BFGS quasi-Newton method for 1D CVT optimization.
///
/// BFGS approximates the inverse Hessian, avoiding explicit Hessian computation.
/// For small 1D problems, this is often faster than L-BFGS.
///
/// # Returns
/// `CvtResult` with iteration history and wall-clock timing.
pub fn bfgs_cvt<F>(
    domain: &Domain1D,
    seeds: Field<Real>,
    density: F,
    _settings: &CvtSolverSettings,
) -> CvtResult<Real>
where
    F: Fn(Real) -> Real + Copy,
{
    use argmin::solver::quasinewton::BFGS;

    let start = Instant::now();

    let problem = Cvt1dProblem::new(domain, density);
    let init_param: Vec<Real> = seeds.iter().copied().collect();
    let n = init_param.len();
    let init_energy = cvt_energy(domain, &seeds, &density);

    let linesearch = MoreThuenteLineSearch::new();
    let solver = BFGS::new(linesearch);
    let init_inv_hessian = identity_matrix(n);

    let history_data = Arc::new(Mutex::new(Vec::new()));
    let observer = CvtHistoryObserver {
        history: Arc::clone(&history_data),
    };

    const ARGMIN_MAX_ITERS: u64 = 100_000;
    let result = Executor::new(problem, solver)
        .configure(|state| {
            state
                .param(init_param)
                .inv_hessian(init_inv_hessian)
                .max_iters(ARGMIN_MAX_ITERS)
        })
        .add_observer(observer, ObserverMode::Always)
        .run();

    let hit_max = result
        .as_ref()
        .map(|r| r.state().get_iter() >= ARGMIN_MAX_ITERS)
        .unwrap_or(true);
    if let Err(e) = &result {
        eprintln!("BFGS optimization error: {}", e);
    }

    let raw = history_data.lock().unwrap();
    let n_iters = raw.len();
    CvtResult {
        history: collect_history(seeds, init_energy, &raw),
        elapsed: start.elapsed(),
        converged: !hit_max,
        converge_iter: if hit_max { None } else { Some(n_iters.saturating_sub(1)) },
        final_residual: Real::NAN,
    }
}

/// Run Newton's method for 1D CVT optimization.
///
/// Newton's method uses the full Hessian for quadratic convergence near the solution.
/// For 1D CVT, the Hessian is tridiagonal so this is efficient.
///
/// # Returns
/// `CvtResult` with iteration history and wall-clock timing.
pub fn newton_cvt<F>(
    domain: &Domain1D,
    seeds: Field<Real>,
    density: F,
    settings: &CvtSolverSettings,
) -> CvtResult<Real>
where
    F: Fn(Real) -> Real,
{
    let start = Instant::now();
    let mut history = Vec::new();
    let mut current_seeds = seeds;
    let mut sustain_count = 0usize;
    let mut converge_iter: Option<usize> = None;
    let mut final_residual = Real::INFINITY;

    for iter in 0..settings.max_iter {
        // Use integrate_cells for both energy/history AND the convergence residual
        // so that newton_cvt uses the same dimensionless criterion as lloyd_cvt.
        let data = domain.integrate_cells(current_seeds.as_slice(), &density);
        final_residual = normalised_residual(&current_seeds, &data.centroids, &data.masses, &density);

        let max_mass = data.masses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let min_mass = data.masses.iter().copied().fold(f64::INFINITY, f64::min);
        history.push(CvtState {
            iteration: iter,
            seeds: current_seeds.clone(),
            energy: data.energy,
            residual: final_residual,
            max_mass,
            min_mass,
        });

        if final_residual < settings.tol {
            if converge_iter.is_none() {
                converge_iter = Some(iter);
            }
            sustain_count += 1;
            if sustain_count >= settings.converge_sustain {
                break;
            }
        } else {
            sustain_count = 0;
        }

        let grad = cvt_gradient(domain, &current_seeds, &density);
        let hess = cvt_hessian(domain, &current_seeds, &density);
        let grad_vec: Vec<Real> = grad.iter().copied().collect();

        if let Some(step) = solve_tridiagonal(&hess, &grad_vec) {
            let new_seeds: Field<Real> = current_seeds
                .iter()
                .zip(step.iter())
                .map(|(s, d)| s - d)
                .collect();
            current_seeds = new_seeds;
        } else {
            eprintln!("Newton: failed to solve linear system at iteration {}", iter);
            break;
        }
    }

    let converged = sustain_count >= settings.converge_sustain;
    CvtResult {
        history,
        elapsed: start.elapsed(),
        converged,
        converge_iter,
        final_residual,
    }
}

// ============================================================================
// Convergence metric
// ============================================================================

/// Dimensionless normalised displacement residual.
///
/// ```text
/// r = max_i  ‖ centroid_i − seed_i ‖ / h_i
/// ```
///
/// where `h_i = √(mass_i / ρ(seed_i))` is the local characteristic cell
/// size.  This is scale-invariant and density-adaptive: the same tolerance
/// value produces consistent convergence behaviour across arbitrary domain
/// sizes and density fields.
///
/// Computed as `√(max_i disp_i² / h_i²)` to require only one `sqrt` call
/// across all N seeds rather than two per seed.
fn normalised_residual<P, F>(seeds: &Field<P>, centroids: &Field<P>, masses: &Field<Real>, density: &F) -> Real
where
    P: CvtPoint,
    F: Fn(P) -> Real,
{
    let max_sq = seeds
        .iter()
        .zip(centroids.iter())
        .zip(masses.iter())
        .map(|((s, c), &m)| {
            let disp_sq = (*s - *c).norm_squared();
            let rho = density(*s).max(1e-30);
            let h_sq = (m / rho).max(1e-30);
            disp_sq / h_sq
        })
        .fold(0.0_f64, f64::max);
    max_sq.sqrt()
}

// ============================================================================
// Helper functions
// ============================================================================

fn identity_matrix(n: usize) -> Vec<Vec<Real>> {
    let mut m = vec![vec![0.0; n]; n];
    for i in 0..n {
        m[i][i] = 1.0;
    }
    m
}

/// Solve a tridiagonal system Hx = b using the Thomas algorithm.
fn solve_tridiagonal(h: &[Vec<Real>], b: &[Real]) -> Option<Vec<Real>> {
    let n = b.len();
    if n == 0 || h.len() != n {
        return None;
    }

    let mut c_prime = vec![0.0; n];
    let mut d_prime = vec![0.0; n];

    let diag = |i: usize| h[i][i];
    let upper = |i: usize| if i + 1 < n { h[i][i + 1] } else { 0.0 };
    let lower = |i: usize| if i > 0 { h[i][i - 1] } else { 0.0 };

    if diag(0).abs() < 1e-15 {
        return None;
    }
    c_prime[0] = upper(0) / diag(0);
    d_prime[0] = b[0] / diag(0);

    for i in 1..n {
        let denom = diag(i) - lower(i) * c_prime[i - 1];
        if denom.abs() < 1e-15 {
            return None;
        }
        c_prime[i] = upper(i) / denom;
        d_prime[i] = (b[i] - lower(i) * d_prime[i - 1]) / denom;
    }

    let mut x = vec![0.0; n];
    x[n - 1] = d_prime[n - 1];
    for i in (0..n - 1).rev() {
        x[i] = d_prime[i] - c_prime[i] * x[i + 1];
    }

    Some(x)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meshgen::cvt::{uniform_seeds, Domain1D};

    const TOL: Real = 1e-6;

    #[test]
    fn lloyd_converges_uniform_density() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds: Field<Real> = [0.1, 0.3, 0.4, 0.7, 0.95].into_iter().collect();
        let settings = CvtSolverSettings::default();

        let result = lloyd_cvt(&domain, seeds, density, &settings);

        assert!(!result.history.is_empty());
        let final_state = result.history.last().unwrap();
        assert!(final_state.energy < 0.01);
    }

    #[test]
    fn lbfgs_converges_uniform_density() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds: Field<Real> = [0.1, 0.3, 0.4, 0.7, 0.95].into_iter().collect();
        let settings = CvtSolverSettings::default();

        let result = lbfgs_cvt(&domain, seeds, density, &settings);

        assert!(!result.history.is_empty());
        let final_state = result.history.last().unwrap();
        assert!(final_state.energy < 0.01);
    }

    #[test]
    fn bfgs_converges_uniform_density() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds: Field<Real> = [0.1, 0.3, 0.4, 0.7, 0.95].into_iter().collect();
        let settings = CvtSolverSettings::default();

        let result = bfgs_cvt(&domain, seeds, density, &settings);

        assert!(!result.history.is_empty());
        let final_state = result.history.last().unwrap();
        assert!(final_state.energy < 0.01);
    }

    #[test]
    fn newton_converges_uniform_density() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds: Field<Real> = [0.1, 0.3, 0.4, 0.7, 0.95].into_iter().collect();
        let settings = CvtSolverSettings::default();

        let result = newton_cvt(&domain, seeds, density, &settings);

        assert!(!result.history.is_empty());
        let final_state = result.history.last().unwrap();
        assert!(final_state.energy < 0.01);
    }

    #[test]
    fn cost_function_matches_cvt_energy() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |x: Real| 1.0 + x;
        let seeds = uniform_seeds(5, &domain);

        let problem = Cvt1dProblem::new(&domain, density);
        let param: Vec<Real> = seeds.iter().copied().collect();

        let cost = problem.cost(&param).unwrap();
        let expected = cvt_energy(&domain, &seeds, &density);

        assert!((cost - expected).abs() < TOL);
    }

    #[test]
    fn gradient_matches_cvt_gradient() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |x: Real| 1.0 + x;
        let seeds = uniform_seeds(5, &domain);

        let problem = Cvt1dProblem::new(&domain, density);
        let param: Vec<Real> = seeds.iter().copied().collect();

        let grad = problem.gradient(&param).unwrap();
        let expected = cvt_gradient(&domain, &seeds, &density);

        for (g, e) in grad.iter().zip(expected.iter()) {
            assert!((*g - *e).abs() < TOL);
        }
    }

    #[test]
    fn tridiagonal_solver_simple() {
        let h = vec![
            vec![4.0, 1.0, 0.0],
            vec![1.0, 4.0, 1.0],
            vec![0.0, 1.0, 4.0],
        ];
        let b = vec![1.0, 2.0, 3.0];

        let x = solve_tridiagonal(&h, &b).unwrap();

        let mut ax = vec![0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                ax[i] += h[i][j] * x[j];
            }
        }

        for i in 0..3 {
            assert!((ax[i] - b[i]).abs() < TOL, "ax[{}] = {}, b[{}] = {}", i, ax[i], i, b[i]);
        }
    }
}
