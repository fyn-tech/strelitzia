//! Type aliases and dimension-specific constructors / accessors.
//!
//! Users interact with these aliases rather than `MultiArray<T, S, B>` directly.

use super::types::*;
use crate::common::Real;
use nalgebra as na;

// ============================================================================
// Type Aliases
// ============================================================================

/// Static vector type (stack-allocated, `Copy`).
pub type Vector<T, const N: usize> = MultiArray<T, Rank1<N>, na::SVector<T, N>>;
/// Static matrix type (stack-allocated, `Copy`).
pub type Matrix<T, const R: usize, const C: usize> =
    MultiArray<T, Rank2<R, C>, na::SMatrix<T, R, C>>;

/// Dynamic vector type (heap-allocated, NOT `Copy`).
pub type DynVector<T> = MultiArray<T, DynRank1, na::DVector<T>>;
/// Dynamic matrix type (heap-allocated, NOT `Copy`).
pub type DynMatrix<T> = MultiArray<T, DynRank2, na::DMatrix<T>>;

// Convenience aliases fixed to Real
pub type Vector2 = Vector<Real, 2>;
pub type Vector3 = Vector<Real, 3>;
pub type Vector4 = Vector<Real, 4>;
pub type Matrix2 = Matrix<Real, 2, 2>;
pub type Matrix3 = Matrix<Real, 3, 3>;
pub type Matrix4 = Matrix<Real, 4, 4>;

// Geometric point aliases (same type as VectorN, intent-explicit naming)
pub type Point2 = Vector2;
pub type Point3 = Vector3;
pub type Point4 = Vector4;

// ============================================================================
// Dimension-specific constructors and accessors
// ============================================================================

// --- Vector2 ---
impl<T: na::Scalar + Copy> MultiArray<T, Rank1<2>, na::SVector<T, 2>> {
    pub fn new(x: T, y: T) -> Self {
        let inner: na::SVector<T, 2> = na::SVector::from_column_slice(&[x, y]);
        Self::from_inner(inner)
    }
    pub fn x(&self) -> T {
        self.data[(0, 0)]
    }
    pub fn y(&self) -> T {
        self.data[(1, 0)]
    }
}

// --- Vector3 ---
impl<T: na::Scalar + Copy> MultiArray<T, Rank1<3>, na::SVector<T, 3>> {
    pub fn new(x: T, y: T, z: T) -> Self {
        let inner: na::SVector<T, 3> = na::SVector::from_column_slice(&[x, y, z]);
        Self::from_inner(inner)
    }
    pub fn x(&self) -> T {
        self.data[(0, 0)]
    }
    pub fn y(&self) -> T {
        self.data[(1, 0)]
    }
    pub fn z(&self) -> T {
        self.data[(2, 0)]
    }
}

// --- Vector4 ---
impl<T: na::Scalar + Copy> MultiArray<T, Rank1<4>, na::SVector<T, 4>> {
    pub fn new(x: T, y: T, z: T, w: T) -> Self {
        let inner: na::SVector<T, 4> = na::SVector::from_column_slice(&[x, y, z, w]);
        Self::from_inner(inner)
    }
    pub fn x(&self) -> T {
        self.data[(0, 0)]
    }
    pub fn y(&self) -> T {
        self.data[(1, 0)]
    }
    pub fn z(&self) -> T {
        self.data[(2, 0)]
    }
    pub fn w(&self) -> T {
        self.data[(3, 0)]
    }
}

// --- Generic static vector: zeros, from_slice, dim ---
impl<T: na::Scalar + Copy + num_traits::Zero, const N: usize>
    MultiArray<T, Rank1<N>, na::SVector<T, N>>
{
    pub fn zeros() -> Self {
        let inner: na::SVector<T, N> = na::SVector::zeros();
        Self::from_inner(inner)
    }

    pub fn dim(&self) -> usize {
        N
    }

    pub fn from_slice(data: &[T]) -> Self {
        let inner: na::SVector<T, N> = na::SVector::from_column_slice(data);
        Self::from_inner(inner)
    }
}

// --- Generic static matrix: zeros, identity, nrows, ncols, from_slice ---
impl<T: na::Scalar + Copy + num_traits::Zero, const R: usize, const C: usize>
    MultiArray<T, Rank2<R, C>, na::SMatrix<T, R, C>>
{
    pub fn zeros() -> Self {
        let inner: na::SMatrix<T, R, C> = na::SMatrix::zeros();
        Self::from_inner(inner)
    }

    pub fn nrows(&self) -> usize {
        R
    }

    pub fn ncols(&self) -> usize {
        C
    }

    pub fn from_slice(data: &[T]) -> Self {
        let inner: na::SMatrix<T, R, C> = na::SMatrix::from_column_slice(data);
        Self::from_inner(inner)
    }
}

// --- Square matrix: identity ---
impl<T: na::Scalar + Copy + num_traits::Zero + num_traits::One, const N: usize>
    MultiArray<T, Rank2<N, N>, na::SMatrix<T, N, N>>
{
    pub fn identity() -> Self {
        let inner: na::SMatrix<T, N, N> = na::SMatrix::identity();
        Self::from_inner(inner)
    }
}

// --- Matrix3 constructor (nalgebra-style row-major input) ---
impl<T: na::Scalar + Copy> MultiArray<T, Rank2<3, 3>, na::SMatrix<T, 3, 3>> {
    /// Construct a 3x3 matrix from elements in row-major order
    /// (matching nalgebra's `Matrix3::new` convention).
    ///
    /// nalgebra stores in column-major internally, but accepts row-major input.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        m11: T, m12: T, m13: T,
        m21: T, m22: T, m23: T,
        m31: T, m32: T, m33: T,
    ) -> Self {
        let inner: na::SMatrix<T, 3, 3> = na::SMatrix::from_column_slice(&[
            m11, m21, m31, // col 0
            m12, m22, m32, // col 1
            m13, m23, m33, // col 2
        ]);
        Self::from_inner(inner)
    }
}

// --- Vector<T, 1>: to_scalar convenience ---
impl<T: Copy> MultiArray<T, Rank1<1>, na::SVector<T, 1>> {
    /// Extract the single element as a scalar.
    pub fn to_scalar(self) -> T
    where
        T: na::Scalar,
    {
        self.data[(0, 0)]
    }
}

// --- DynVector ---
impl<T: na::Scalar + Copy + num_traits::Zero> DynVector<T> {
    pub fn zeros(n: usize) -> Self {
        Self::from_inner(na::DVector::zeros(n))
    }

    pub fn dim(&self) -> usize {
        self.data.nrows()
    }

    pub fn from_slice(data: &[T]) -> Self {
        Self::from_inner(na::DVector::from_column_slice(data))
    }
}

// --- DynMatrix ---
impl<T: na::Scalar + Copy + num_traits::Zero> DynMatrix<T> {
    pub fn zeros(nrows: usize, ncols: usize) -> Self {
        Self::from_inner(na::DMatrix::zeros(nrows, ncols))
    }

    pub fn nrows(&self) -> usize {
        self.data.nrows()
    }

    pub fn ncols(&self) -> usize {
        self.data.ncols()
    }

    pub fn from_slice(data: &[T], nrows: usize, ncols: usize) -> Self {
        Self::from_inner(na::DMatrix::from_column_slice(nrows, ncols, data))
    }
}
