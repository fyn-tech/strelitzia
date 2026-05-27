//! 1D domain implementation for CVT.
//!
//! Contains [`Domain1D`] (the 1D interval domain), its [`CvtDomain`] implementation,
//! and 1D-specific helpers (Hessian, Newton, uniform seeds, density conversion).

use crate::common::Real;
use crate::fields::Field;
use gauss_quad::GaussLegendre;

use super::{CvtCellData, CvtDomain};

/// A 1D interval domain for CVT.
///
/// Uses Gauss-Legendre quadrature for numerical integration.
#[derive(Clone)]
pub struct Domain1D {
    /// Left boundary of the interval.
    pub min: Real,
    /// Right boundary of the interval.
    pub max: Real,
    /// Cached quadrature rule.
    quad: GaussLegendre,
}

impl Domain1D {
    /// Create a new interval with default quadrature order (5).
    pub fn new(min: Real, max: Real) -> Self {
        Self::with_quadrature(min, max, 5)
    }

    /// Create a new interval with specified quadrature order.
    pub fn with_quadrature(min: Real, max: Real, quad_order: usize) -> Self {
        Self {
            min,
            max,
            quad: GaussLegendre::new(quad_order).expect("invalid quadrature order"),
        }
    }

    /// Length of the interval.
    pub fn length(&self) -> Real {
        self.max - self.min
    }

    /// Compute Voronoi cells (sub-intervals) from seed positions.
    ///
    /// Each cell is defined by the midpoints between adjacent seeds,
    /// with domain boundaries at the extremes.
    pub fn voronoi_cells(&self, seeds: &[Real]) -> Vec<(Real, Real)> {
        if seeds.is_empty() {
            return vec![];
        }

        let n = seeds.len();
        let mut cells = Vec::with_capacity(n);

        for i in 0..n {
            let left = if i == 0 {
                self.min
            } else {
                0.5 * (seeds[i - 1] + seeds[i])
            };
            let right = if i == n - 1 {
                self.max
            } else {
                0.5 * (seeds[i] + seeds[i + 1])
            };
            cells.push((left, right));
        }

        cells
    }

    /// Mass of a cell: integral of density over the cell.
    pub fn cell_mass<F: Fn(Real) -> Real>(&self, cell: &(Real, Real), density: &F) -> Real {
        let (left, right) = *cell;
        self.quad.integrate(left, right, density)
    }

    /// First moment of a cell: integral of x * density over the cell.
    pub fn cell_moment<F: Fn(Real) -> Real>(&self, cell: &(Real, Real), density: &F) -> Real {
        let (left, right) = *cell;
        self.quad.integrate(left, right, |x| x * density(x))
    }

    /// Energy contribution of a cell: integral of density * (x - seed)^2 over the cell.
    pub fn cell_energy<F: Fn(Real) -> Real>(
        &self,
        cell: &(Real, Real),
        seed: &Real,
        density: &F,
    ) -> Real {
        let (left, right) = *cell;
        let s = *seed;
        self.quad
            .integrate(left, right, |x| density(x) * (x - s).powi(2))
    }

    /// Compute density values at Voronoi cell boundaries.
    ///
    /// For n seeds, returns n-1 boundary density values at the midpoints
    /// between adjacent seeds. Needed for Hessian computation.
    pub fn boundary_densities<F: Fn(Real) -> Real>(
        &self,
        seeds: &[Real],
        density: &F,
    ) -> Vec<Real> {
        if seeds.len() <= 1 {
            return vec![];
        }
        (0..seeds.len() - 1)
            .map(|i| {
                let boundary = 0.5 * (seeds[i] + seeds[i + 1]);
                density(boundary)
            })
            .collect()
    }
}

impl CvtDomain for Domain1D {
    type Point = Real;

    fn integrate_cells<F: Fn(Real) -> Real>(
        &self,
        seeds: &[Real],
        density: &F,
    ) -> CvtCellData<Real> {
        let cells = self.voronoi_cells(seeds);
        let n = seeds.len();
        let mut centroids = Field::with_capacity(n);
        let mut masses = Field::with_capacity(n);
        let mut energy = 0.0;

        for (i, cell) in cells.iter().enumerate() {
            let m = self.cell_mass(cell, density);
            let moment = self.cell_moment(cell, density);
            let e = self.cell_energy(cell, &seeds[i], density);
            centroids.push(moment / m);
            masses.push(m);
            energy += e;
        }

        CvtCellData {
            centroids,
            masses,
            energy,
        }
    }
}

// ============================================================================
// 1D-specific functions
// ============================================================================

/// Hessian of the 1D CVT energy with respect to seeds.
///
/// The Hessian is symmetric tridiagonal with entries derived from
/// differentiating `g_i = 2(s_i m_i - M_i)`:
///
/// - Off-diagonal: `H_{i,i+1} = -rho(b_i) * (s_{i+1} - s_i) / 2`
/// - Diagonal: `H_{ii} = 2 * mass_i + sum of off-diagonal entries in row i`
///
/// where `b_i = (s_i + s_{i+1}) / 2` is the Voronoi boundary between cells.
pub fn cvt_hessian<F: Fn(Real) -> Real>(
    domain: &Domain1D,
    seeds: &Field<Real>,
    density: &F,
) -> Vec<Vec<Real>> {
    let n = seeds.len();
    if n == 0 {
        return vec![];
    }

    let cells = domain.voronoi_cells(seeds.as_slice());
    let boundary_rho = domain.boundary_densities(seeds.as_slice(), density);

    let mut hess = vec![vec![0.0; n]; n];

    for i in 0..n - 1 {
        let spacing = seeds[i + 1] - seeds[i];
        let off_diag = -boundary_rho[i] * spacing * 0.5;
        hess[i][i + 1] = off_diag;
        hess[i + 1][i] = off_diag;
    }

    for i in 0..n {
        let mass = domain.cell_mass(&cells[i], density);
        hess[i][i] = 2.0 * mass;
        if i > 0 {
            hess[i][i] += hess[i][i - 1];
        }
        if i < n - 1 {
            hess[i][i] += hess[i][i + 1];
        }
    }

    hess
}

/// Generate uniformly spaced seeds in the interior of an interval.
///
/// Seeds are placed at positions that would be centroids for uniform density,
/// avoiding the boundaries.
pub fn uniform_seeds(n: usize, domain: &Domain1D) -> Field<Real> {
    if n == 0 {
        return Field::new();
    }

    let step = domain.length() / (n as Real);
    (0..n)
        .map(|i| domain.min + step * (i as Real + 0.5))
        .collect()
}

/// Convert a cell-size function to a density function.
///
/// The relationship is: density(x) = domain_length / cell_size(x)
///
/// This allows specifying desired local cell sizes rather than densities.
pub fn density_from_cell_size<F>(cell_size: F, domain_length: Real) -> impl Fn(Real) -> Real
where
    F: Fn(Real) -> Real,
{
    move |x| domain_length / cell_size(x)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::SolverInterop;
    use crate::meshgen::cvt::{cvt_gradient, lloyd_iter};

    const TOL: Real = 1e-10;

    #[test]
    fn interval_voronoi_cells() {
        let domain = Domain1D::new(0.0, 1.0);
        let seeds: Field<Real> = [0.25, 0.5, 0.75].into_iter().collect();

        let cells = domain.voronoi_cells(seeds.as_slice());

        assert_eq!(cells.len(), 3);
        assert!((cells[0].0 - 0.0).abs() < TOL);
        assert!((cells[0].1 - 0.375).abs() < TOL);
        assert!((cells[1].0 - 0.375).abs() < TOL);
        assert!((cells[1].1 - 0.625).abs() < TOL);
        assert!((cells[2].0 - 0.625).abs() < TOL);
        assert!((cells[2].1 - 1.0).abs() < TOL);
    }

    #[test]
    fn uniform_density_analytical_solution() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let n = 5;

        let seeds: Field<Real> = [0.1, 0.3, 0.4, 0.7, 0.95].into_iter().collect();

        let final_state = lloyd_iter(domain, seeds, density).nth(999).unwrap();

        let expected_step = 1.0 / (n as Real);
        for (i, &seed) in final_state.seeds.iter().enumerate() {
            let expected = expected_step * (i as Real + 0.5);
            assert!(
                (seed - expected).abs() < 1e-6,
                "seed {} = {}, expected {}",
                i,
                seed,
                expected
            );
        }
    }

    #[test]
    fn energy_decreases_monotonically() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |x: Real| 1.0 + 0.5 * (2.0 * std::f64::consts::PI * x).sin();

        let seeds: Field<Real> = [0.1, 0.25, 0.6, 0.85].into_iter().collect();

        let history: Vec<_> = lloyd_iter(domain, seeds, density).take(50).collect();

        for i in 1..history.len() {
            assert!(
                history[i].energy <= history[i - 1].energy + TOL,
                "energy increased at iteration {}: {} > {}",
                i,
                history[i].energy,
                history[i - 1].energy
            );
        }
    }

    #[test]
    fn gradient_zero_at_convergence() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;

        let seeds = uniform_seeds(5, &domain);

        let grad = cvt_gradient(&domain, &seeds, &density);

        let grad_norm: Real = grad.iter().map(|g| g * g).sum::<Real>().sqrt();
        assert!(
            grad_norm < 1e-10,
            "gradient norm = {} (expected ~0)",
            grad_norm
        );
    }

    #[test]
    fn uniform_seeds_helper() {
        let domain = Domain1D::new(0.0, 1.0);
        let seeds = uniform_seeds(4, &domain);

        assert_eq!(seeds.len(), 4);
        assert!((seeds[0] - 0.125).abs() < TOL);
        assert!((seeds[1] - 0.375).abs() < TOL);
        assert!((seeds[2] - 0.625).abs() < TOL);
        assert!((seeds[3] - 0.875).abs() < TOL);
    }

    #[test]
    fn density_from_cell_size_conversion() {
        let cell_size = |x: Real| 0.5 + 0.5 * x;
        let domain_length = 1.0;
        let density = density_from_cell_size(cell_size, domain_length);

        assert!((density(0.0) - 2.0).abs() < TOL);
        assert!((density(1.0) - 1.0).abs() < TOL);
    }

    #[test]
    fn solver_interop_on_seeds() {
        let domain = Domain1D::new(0.0, 1.0);
        let seeds = uniform_seeds(3, &domain);

        let flat: &[Real] = seeds.as_flat_slice();
        assert_eq!(flat.len(), 3);
        assert!((flat[0] - seeds[0]).abs() < TOL);
        assert!((flat[1] - seeds[1]).abs() < TOL);
        assert!((flat[2] - seeds[2]).abs() < TOL);
    }

    #[test]
    fn hessian_is_symmetric() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |x: Real| 1.0 + 0.5 * x;
        let seeds = uniform_seeds(5, &domain);

        let hess = cvt_hessian(&domain, &seeds, &density);

        assert_eq!(hess.len(), 5);
        for row in &hess {
            assert_eq!(row.len(), 5);
        }

        for i in 0..5 {
            for j in 0..5 {
                assert!(
                    (hess[i][j] - hess[j][i]).abs() < TOL,
                    "Hessian not symmetric at ({}, {}): {} vs {}",
                    i,
                    j,
                    hess[i][j],
                    hess[j][i]
                );
            }
        }
    }

    #[test]
    fn hessian_is_tridiagonal() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds = uniform_seeds(5, &domain);

        let hess = cvt_hessian(&domain, &seeds, &density);

        for i in 0..5 {
            for j in 0..5 {
                if (i as i32 - j as i32).abs() > 1 {
                    assert!(
                        hess[i][j].abs() < TOL,
                        "Hessian should be tridiagonal, but H[{}][{}] = {}",
                        i,
                        j,
                        hess[i][j]
                    );
                }
            }
        }
    }

    #[test]
    fn boundary_densities_count() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |x: Real| 1.0 + x;
        let seeds = uniform_seeds(5, &domain);

        let bd = domain.boundary_densities(seeds.as_slice(), &density);
        assert_eq!(bd.len(), 4);
    }

    #[test]
    fn hessian_matches_finite_differences() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds: Field<Real> = [0.2, 0.5, 0.8].into_iter().collect();

        let hess = cvt_hessian(&domain, &seeds, &density);
        let n = seeds.len();

        let eps = 1e-6;
        for j in 0..n {
            let mut sp = seeds.clone();
            let mut sm = seeds.clone();
            sp[j] += eps;
            sm[j] -= eps;
            let gp = cvt_gradient(&domain, &sp, &density);
            let gm = cvt_gradient(&domain, &sm, &density);
            for i in 0..n {
                let numerical = (gp[i] - gm[i]) / (2.0 * eps);
                let analytic = hess[i][j];
                let err = (analytic - numerical).abs();
                assert!(
                    err < 1e-4,
                    "H[{}][{}]: analytic={:.8}, numerical={:.8}, err={:.2e}",
                    i,
                    j,
                    analytic,
                    numerical,
                    err
                );
            }
        }
    }

    #[test]
    fn newton_converges_in_one_step_for_two_seeds() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;

        let seeds: Field<Real> = [0.2, 0.8].into_iter().collect();
        let grad = cvt_gradient(&domain, &seeds, &density);
        let hess = cvt_hessian(&domain, &seeds, &density);

        let det = hess[0][0] * hess[1][1] - hess[0][1] * hess[1][0];
        let step0 = (hess[1][1] * grad[0] - hess[0][1] * grad[1]) / det;
        let step1 = (-hess[1][0] * grad[0] + hess[0][0] * grad[1]) / det;

        let new_s0 = seeds[0] - step0;
        let new_s1 = seeds[1] - step1;

        assert!(
            (new_s0 - 0.25).abs() < 1e-6,
            "Newton should reach 0.25 in one step, got {}",
            new_s0
        );
        assert!(
            (new_s1 - 0.75).abs() < 1e-6,
            "Newton should reach 0.75 in one step, got {}",
            new_s1
        );
    }

    #[test]
    fn integrate_cells_matches_individual_methods() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |x: Real| 1.0 + 0.5 * x;
        let seeds: Field<Real> = [0.2, 0.5, 0.8].into_iter().collect();

        let cells = domain.voronoi_cells(seeds.as_slice());
        let data = domain.integrate_cells(seeds.as_slice(), &density);

        let mut expected_energy = 0.0;
        for (i, cell) in cells.iter().enumerate() {
            let mass = domain.cell_mass(cell, &density);
            let moment = domain.cell_moment(cell, &density);
            let e = domain.cell_energy(cell, &seeds[i], &density);
            let centroid = moment / mass;

            assert!(
                (data.masses[i] - mass).abs() < TOL,
                "mass mismatch at cell {}",
                i
            );
            assert!(
                (data.centroids[i] - centroid).abs() < TOL,
                "centroid mismatch at cell {}",
                i
            );
            expected_energy += e;
        }
        assert!(
            (data.energy - expected_energy).abs() < TOL,
            "energy mismatch"
        );
    }
}
