//! Centroidal Voronoi Tessellation (CVT) algorithms.
//!
//! This module provides dimension-agnostic CVT infrastructure via traits,
//! with a concrete 1D implementation using Gauss-Legendre quadrature.
//!
//! # Architecture
//!
//! - [`CvtPoint`] -- trait for point types usable as CVT seeds (extends `FieldElement`)
//! - [`CvtDomain`] -- trait for domain geometries (single `integrate_cells` method)
//! - [`CvtCellData`] -- integrated quantities returned by `integrate_cells`
//! - [`Domain1D`] -- 1D domain implementation
//! - Generic free functions -- `lloyd_step`, `cvt_energy`, `cvt_gradient`, `cvt_residual`, `lloyd_iter`
//!
//! # Example
//!
//! ```
//! use strelitzia::meshgen::cvt::*;
//! use strelitzia::common::Real;
//!
//! let domain = Domain1D::new(0.0, 1.0);
//! let density = |_x: Real| 1.0;  // uniform density
//! let seeds = uniform_seeds(10, &domain);
//!
//! // Run 100 Lloyd iterations
//! let history: Vec<_> = lloyd_iter(domain, seeds, density).take(100).collect();
//! let final_energy = history.last().unwrap().energy;
//! ```

pub mod domain_1d;
pub mod domain_2d;

pub use domain_1d::{
    cvt_hessian, density_from_cell_size, uniform_seeds, Domain1D,
};
pub use domain_2d::Domain2D;

use crate::common::Real;
use crate::fields::{Field, FieldElement};
use crate::multiarray::{Point2, Point3};
use std::ops::{Add, Div, Mul, Sub};

// ============================================================================
// CvtPoint trait
// ============================================================================

/// A point type usable as a CVT seed.
///
/// Extends `FieldElement<Scalar = Real>` to inherit `COMPONENTS` and scalar
/// conversion, enabling automatic `SolverInterop` for `Field<P: CvtPoint>`.
///
/// Implemented for `Real` (1D). Future: `Point2`, `Point3`.
pub trait CvtPoint:
    FieldElement<Scalar = Real>
    + Copy
    + std::fmt::Debug
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Real, Output = Self>
    + Div<Real, Output = Self>
{
    /// The additive identity (zero vector / zero scalar).
    fn zero() -> Self;
    /// Squared Euclidean norm. Enables generic residual and gradient computation.
    fn norm_squared(&self) -> Real;
}

impl CvtPoint for Real {
    fn zero() -> Self {
        0.0
    }
    fn norm_squared(&self) -> Real {
        *self * *self
    }
}

impl CvtPoint for Point2 {
    fn zero() -> Self {
        Point2::new(0.0, 0.0)
    }
    fn norm_squared(&self) -> Real {
        self[0] * self[0] + self[1] * self[1]
    }
}

impl CvtPoint for Point3 {
    fn zero() -> Self {
        Point3::new(0.0, 0.0, 0.0)
    }
    fn norm_squared(&self) -> Real {
        self[0] * self[0] + self[1] * self[1] + self[2] * self[2]
    }
}

// ============================================================================
// CvtCellData
// ============================================================================

/// Integrated quantities derived from Voronoi cell decomposition.
///
/// Returned by [`CvtDomain::integrate_cells`]. Contains everything the
/// generic CVT algorithms need, without exposing cell geometry.
#[derive(Debug, Clone)]
pub struct CvtCellData<P: CvtPoint> {
    /// Centroid of each Voronoi cell (moment / mass).
    pub centroids: Field<P>,
    /// Mass of each Voronoi cell (integral of density).
    pub masses: Field<Real>,
    /// Total CVT energy: sum of integral(density * ||x - seed||^2) over all cells.
    pub energy: Real,
}

// ============================================================================
// CvtDomain trait
// ============================================================================

/// A domain over which CVT is computed.
///
/// Encapsulates dimension-specific operations: Voronoi cell decomposition
/// and numerical integration. Implementations may stream cells internally
/// (compute, integrate, drop) without materializing all cells simultaneously.
pub trait CvtDomain {
    /// The point type for this domain (e.g., `Real` for 1D, `Point2` for 2D).
    type Point: CvtPoint;

    /// Decompose the domain into Voronoi cells for the given seeds,
    /// integrate density over each cell, and return the results.
    fn integrate_cells<F: Fn(Self::Point) -> Real>(
        &self,
        seeds: &[Self::Point],
        density: &F,
    ) -> CvtCellData<Self::Point>;
}

// ============================================================================
// Generic free functions
// ============================================================================

/// Perform one Lloyd iteration step.
///
/// Returns the new seed positions (centroids) and the current CVT energy.
pub fn lloyd_step<D: CvtDomain, F: Fn(D::Point) -> Real>(
    domain: &D,
    seeds: &Field<D::Point>,
    density: &F,
) -> (Field<D::Point>, Real) {
    let data = domain.integrate_cells(seeds.as_slice(), density);
    (data.centroids, data.energy)
}

/// Total CVT energy: sum of integral(density * ||x - seed||^2) over all cells.
pub fn cvt_energy<D: CvtDomain, F: Fn(D::Point) -> Real>(
    domain: &D,
    seeds: &Field<D::Point>,
    density: &F,
) -> Real {
    domain.integrate_cells(seeds.as_slice(), density).energy
}

/// Energy gradient with respect to seeds.
///
/// For CVT: grad_i = 2 * m_i * (s_i - c_i)
pub fn cvt_gradient<D: CvtDomain, F: Fn(D::Point) -> Real>(
    domain: &D,
    seeds: &Field<D::Point>,
    density: &F,
) -> Field<D::Point> {
    let data = domain.integrate_cells(seeds.as_slice(), density);
    seeds
        .iter()
        .zip(data.centroids.iter())
        .zip(data.masses.iter())
        .map(|((s, c), &m)| (*s - *c) * (2.0 * m))
        .collect()
}

/// Centroid displacement residual: max ||seed_i - centroid_i||.
///
/// This is the natural CVT convergence metric. A value of zero means
/// every generator coincides with the centroid of its Voronoi cell.
pub fn cvt_residual<D: CvtDomain, F: Fn(D::Point) -> Real>(
    domain: &D,
    seeds: &Field<D::Point>,
    density: &F,
) -> Real {
    let data = domain.integrate_cells(seeds.as_slice(), density);
    seeds
        .iter()
        .zip(data.centroids.iter())
        .map(|(s, c)| (*s - *c).norm_squared().sqrt())
        .fold(0.0, Real::max)
}

// ============================================================================
// CvtState and Lloyd iterator
// ============================================================================

/// State of a CVT optimization at one iteration.
///
/// Captures the current seed positions, energy, and iteration number.
/// Returned by Lloyd iteration and other CVT solvers.
#[derive(Debug, Clone)]
pub struct CvtState<P: CvtPoint> {
    /// Iteration number (0-indexed).
    pub iteration: usize,
    /// Current seed positions.
    pub seeds: Field<P>,
    /// Current CVT energy.
    pub energy: Real,
    /// Normalised displacement residual at this iteration.
    /// `f64::NAN` when not computed (argmin-backed solvers, LloydIterator).
    pub residual: Real,
    /// Maximum Voronoi cell mass across all seeds.
    /// `f64::NAN` when not computed.
    pub max_mass: Real,
    /// Minimum Voronoi cell mass across all seeds.
    /// `f64::NAN` when not computed.
    pub min_mass: Real,
}

/// Internal iterator state for Lloyd iteration.
struct LloydIterator<D, F>
where
    D: CvtDomain,
    F: Fn(D::Point) -> Real,
{
    domain: D,
    density: F,
    seeds: Field<D::Point>,
    iteration: usize,
}

impl<D, F> Iterator for LloydIterator<D, F>
where
    D: CvtDomain,
    F: Fn(D::Point) -> Real,
{
    type Item = CvtState<D::Point>;

    fn next(&mut self) -> Option<Self::Item> {
        let (new_seeds, energy) = lloyd_step(&self.domain, &self.seeds, &self.density);
        let state = CvtState {
            iteration: self.iteration,
            seeds: self.seeds.clone(),
            energy,
            residual: Real::NAN,
            max_mass: Real::NAN,
            min_mass: Real::NAN,
        };
        self.seeds = new_seeds;
        self.iteration += 1;
        Some(state)
    }
}

/// Create a lazy iterator over Lloyd iterations.
///
/// Each call to `next()` yields a `CvtState` containing the current
/// iteration number, seed positions, and energy, then advances by one
/// Lloyd step.
///
/// # Example
///
/// ```
/// use strelitzia::meshgen::cvt::*;
/// use strelitzia::common::Real;
///
/// let domain = Domain1D::new(0.0, 1.0);
/// let density = |_x: Real| 1.0;
/// let seeds = uniform_seeds(5, &domain);
///
/// // Collect 100 iterations
/// let history: Vec<_> = lloyd_iter(domain, seeds, density).take(100).collect();
/// ```
pub fn lloyd_iter<D, F>(
    domain: D,
    seeds: Field<D::Point>,
    density: F,
) -> impl Iterator<Item = CvtState<D::Point>>
where
    D: CvtDomain,
    F: Fn(D::Point) -> Real,
{
    LloydIterator {
        domain,
        density,
        seeds,
        iteration: 0,
    }
}
