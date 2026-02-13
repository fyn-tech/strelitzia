//! Extension traits for domain-specific linear algebra operations.
//!
//! These traits provide method syntax for operations like `dot`, `cross`, `norm`,
//! etc. without polluting the core `MultiArray` type. Users must `use` the
//! trait to access its methods (like `use itertools::Itertools`).
//!
//! # Example
//!
//! ```
//! use strelitzia::multiarray::{Vector3, linalg::{VectorOps, CrossProduct}};
//!
//! let v = Vector3::new(1.0, 0.0, 0.0);
//! let w = Vector3::new(0.0, 1.0, 0.0);
//!
//! assert_eq!(v.dot(&w), 0.0);           // orthogonal
//! assert_eq!(v.cross(&w), Vector3::new(0.0, 0.0, 1.0));
//! assert!((v.norm() - 1.0).abs() < 1e-10);
//! ```

use super::aliases::*;
use super::types::*;
use nalgebra as na;
use std::ops::Mul;

// ============================================================================
// VectorOps -- implemented for Vector<T, N> (any dimension)
// ============================================================================

/// Vector-specific operations. Implemented for rank-1 types only.
pub trait VectorOps<T> {
    /// Inner product.
    fn dot(&self, other: &Self) -> T;
    /// L1 norm (sum of absolute values).
    fn l1_norm(&self) -> T;
    /// L2 (Euclidean) norm.
    fn l2_norm(&self) -> T;
    /// L-infinity norm (max absolute value).
    fn linf_norm(&self) -> T;
    /// General Lp norm.
    fn lp_norm(&self, p: i32) -> T;
    /// Convenience: forwards to `l2_norm()`.
    fn norm(&self) -> T {
        self.l2_norm()
    }
    /// Squared L2 norm (avoids sqrt, useful for comparisons).
    fn norm_squared(&self) -> T;
    /// Returns a new unit vector (does not modify self). Panics if zero.
    fn normalised(&self) -> Self;
}

impl<T: na::RealField + Copy, const N: usize> VectorOps<T> for Vector<T, N> {
    fn dot(&self, other: &Self) -> T {
        self.as_inner().dot(other.as_inner())
    }
    fn l1_norm(&self) -> T {
        self.as_inner().lp_norm(1)
    }
    fn l2_norm(&self) -> T {
        self.as_inner().norm()
    }
    fn linf_norm(&self) -> T {
        self.as_inner().amax()
    }
    fn lp_norm(&self, p: i32) -> T {
        self.as_inner().lp_norm(p)
    }
    fn norm_squared(&self) -> T {
        self.as_inner().norm_squared()
    }
    fn normalised(&self) -> Self {
        Self::from_inner(self.as_inner().normalize())
    }
}

// ============================================================================
// CrossProduct -- implemented for 2-vectors and 3-vectors
// ============================================================================

/// Cross product. Return type varies by dimension.
pub trait CrossProduct<T> {
    type Output;
    fn cross(&self, other: &Self) -> Self::Output;
}

// 3D cross: Vector<T,3> x Vector<T,3> -> Vector<T,3>
impl<T: na::RealField + Copy> CrossProduct<T> for Vector<T, 3> {
    type Output = Vector<T, 3>;
    fn cross(&self, other: &Self) -> Vector<T, 3> {
        Self::from_inner(self.as_inner().cross(other.as_inner()))
    }
}

// 2D cross: Vector<T,2> x Vector<T,2> -> Vector<T,3> (scaled z-basis vector)
impl<T: na::RealField + Copy> CrossProduct<T> for Vector<T, 2> {
    type Output = Vector<T, 3>;
    fn cross(&self, other: &Self) -> Vector<T, 3> {
        let z = self.as_inner()[0] * other.as_inner()[1]
            - self.as_inner()[1] * other.as_inner()[0];
        Vector::<T, 3>::new(T::zero(), T::zero(), z)
    }
}

// ============================================================================
// OuterProduct -- vector x vector -> matrix
// ============================================================================

/// Outer product: produces a matrix from two vectors.
pub trait OuterProduct<T, Rhs> {
    type Output;
    fn outer(&self, other: &Rhs) -> Self::Output;
}

// Vector<T,N> x Vector<T,M> -> Matrix<T,N,M>
impl<
        T: na::Scalar
            + Copy
            + Mul<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::AddAssign
            + std::ops::MulAssign
            + num_traits::Zero
            + num_traits::One,
        const N: usize,
        const M: usize,
    > OuterProduct<T, Vector<T, M>> for Vector<T, N>
{
    type Output = Matrix<T, N, M>;
    fn outer(&self, other: &Vector<T, M>) -> Matrix<T, N, M> {
        Matrix::from_inner(self.as_inner() * other.as_inner().transpose())
    }
}

// ============================================================================
// Hadamard -- element-wise multiplication (single generic impl)
// ============================================================================

/// Element-wise (Hadamard/Schur) product.
pub trait Hadamard {
    fn hadamard(&self, other: &Self) -> Self;
}

// Single blanket implementation -- works for vectors, matrices, dynamic types.
impl<T, S, B> Hadamard for MultiArray<T, S, B>
where
    T: Copy + Mul<Output = T>,
    S: Shape,
    B: DenseRawStorage<T> + Clone,
{
    fn hadamard(&self, other: &Self) -> Self {
        let mut result = self.clone();
        for (a, b) in result
            .data
            .as_mut_slice()
            .iter_mut()
            .zip(other.data.as_slice())
        {
            *a = *a * *b;
        }
        result
    }
}

// ============================================================================
// Transpose -- standalone trait (vectors AND matrices)
// ============================================================================

/// Transpose operation. Applies to both vectors and matrices.
pub trait Transpose {
    type Output;
    fn transpose(&self) -> Self::Output;
}

// Vector<T, N> -> Matrix<T, 1, N> (column -> row vector)
impl<T: na::Scalar + Copy, const N: usize> Transpose for Vector<T, N> {
    type Output = Matrix<T, 1, N>;
    fn transpose(&self) -> Matrix<T, 1, N> {
        Matrix::from_inner(self.as_inner().transpose())
    }
}

// Matrix<T, R, C> -> Matrix<T, C, R>
impl<T: na::Scalar + Copy, const R: usize, const C: usize> Transpose for Matrix<T, R, C> {
    type Output = Matrix<T, C, R>;
    fn transpose(&self) -> Matrix<T, C, R> {
        Matrix::from_inner(self.as_inner().transpose())
    }
}

// ============================================================================
// SquareMatrixOps -- placeholder (implement when needed)
// ============================================================================

/// Square matrix operations.
pub trait SquareMatrixOps<T>: Sized {
    /// Matrix inverse. Panics if singular.
    fn inverse(&self) -> Self;
    /// Matrix determinant.
    fn determinant(&self) -> T;
    /// Sum of diagonal elements.
    fn trace(&self) -> T;
}
// Implementations deferred -- placeholder trait definition only.
// inverse() panics on singular matrices. If a fallible variant is needed later,
// try_inverse() -> Option<Self> can be added as a non-breaking extension.
