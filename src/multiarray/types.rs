//! Core `MultiArray` struct, shape types, and backend storage traits.
//!
//! This file defines the foundational building blocks:
//! - `Shape` trait and concrete shape types (`Rank1`, `Rank2`, `DynRank1`, `DynRank2`)
//! - `RawStorage` / `DenseRawStorage` backend traits + nalgebra implementations
//! - `MultiArray<T, S, B>` struct with inherent escape-hatch methods

use nalgebra as na;
use std::marker::PhantomData;

// ============================================================================
// Shape trait and types
// ============================================================================

/// Describes the dimensionality of a multi-dimensional array.
///
/// Each shape type is a zero-sized struct encoding tensor rank and
/// (for static types) dimensions at compile time.
pub trait Shape: Copy + 'static {
    /// Tensor rank: 1 for vectors, 2 for matrices, etc.
    const RANK: usize;
    /// Total element count. `None` if dynamic (runtime-determined).
    const SIZE: Option<usize>;
}

/// Shape for a static rank-1 array (vector) of dimension `N`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rank1<const N: usize>;
impl<const N: usize> Shape for Rank1<N> {
    const RANK: usize = 1;
    const SIZE: Option<usize> = Some(N);
}

/// Shape for a static rank-2 array (matrix) of dimensions `R` x `C`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rank2<const R: usize, const C: usize>;
impl<const R: usize, const C: usize> Shape for Rank2<R, C> {
    const RANK: usize = 2;
    const SIZE: Option<usize> = Some(R * C);
}

/// Shape for a dynamic rank-1 array (vector with runtime length).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynRank1;
impl Shape for DynRank1 {
    const RANK: usize = 1;
    const SIZE: Option<usize> = None;
}

/// Shape for a dynamic rank-2 array (matrix with runtime dimensions).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynRank2;
impl Shape for DynRank2 {
    const RANK: usize = 2;
    const SIZE: Option<usize> = None;
}

// ============================================================================
// Backend storage traits
// ============================================================================

/// Base backend trait. ALL backends implement this.
pub trait RawStorage<T> {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Dense backend: contiguous memory, supports slice access.
pub trait DenseRawStorage<T>: RawStorage<T> {
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
}

// --- RawStorage impls for nalgebra types ---

impl<T, R, C, S> RawStorage<T> for na::Matrix<T, R, C, S>
where
    T: na::Scalar,
    R: na::Dim,
    C: na::Dim,
    S: na::RawStorage<T, R, C>,
{
    fn len(&self) -> usize {
        self.nrows() * self.ncols()
    }
}

impl<T, R, C, St> DenseRawStorage<T> for na::Matrix<T, R, C, St>
where
    T: na::Scalar,
    R: na::Dim,
    C: na::Dim,
    St: na::Storage<T, R, C> + na::StorageMut<T, R, C> + na::IsContiguous,
{
    fn as_slice(&self) -> &[T] {
        na::Matrix::as_slice(self)
    }
    fn as_mut_slice(&mut self) -> &mut [T] {
        na::Matrix::as_mut_slice(self)
    }
}

// ============================================================================
// The core struct
// ============================================================================

/// A generic N-dimensional array.
///
/// # Type Parameters
/// - `T`: element type (f64, f32, etc.)
/// - `S`: shape -- encodes tensor rank and dimensions
/// - `B`: backend -- the storage implementation, hidden behind type aliases
///
/// Users should interact with type aliases (`Vector3`, `Matrix3`, etc.)
/// rather than this struct directly.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MultiArray<T, S, B> {
    pub(crate) data: B,
    pub(crate) _phantoms: PhantomData<(T, S)>,
}

// ============================================================================
// Inherent methods (escape hatches + convenience)
// ============================================================================

impl<T, S, B> MultiArray<T, S, B> {
    /// Wrap a raw backend value.
    pub fn from_inner(inner: B) -> Self {
        Self {
            data: inner,
            _phantoms: PhantomData,
        }
    }

    /// Unwrap into the raw backend value.
    pub fn into_inner(self) -> B {
        self.data
    }

    /// Borrow the raw backend value.
    pub fn as_inner(&self) -> &B {
        &self.data
    }
}
