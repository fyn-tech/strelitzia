//! Stable API traits and their implementations for `MultiArray`.
//!
//! Defines `MultiArrayOps`, `DenseMultiArrayOps`, `NumericMultiArrayOps` and
//! implements them for `MultiArray` under the appropriate trait bounds.
//! Also provides `Index` and `IndexMut` for element access.

use super::types::*;
use std::ops::{
    Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
};

// ============================================================================
// Stable API Traits
// ============================================================================

/// Base trait for any multi-dimensional array, dense or sparse.
///
/// This is the permanent, stable contract. Minimal by design.
pub trait MultiArrayOps<T>: Sized + Clone {
    /// Total number of elements.
    fn len(&self) -> usize;
    /// Returns `true` if the array contains no elements.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Tensor rank: 1 for vectors, 2 for matrices, etc.
    fn rank(&self) -> usize;
}

/// Dense specialisation: contiguous memory access.
///
/// Implemented by `MultiArray` when the backend provides contiguous storage.
/// NOT implemented by sparse types.
pub trait DenseMultiArrayOps<T>: MultiArrayOps<T> {
    fn as_slice(&self) -> &[T];
    fn as_mut_slice(&mut self) -> &mut [T];
}

/// Stable API for numeric multi-dimensional arrays.
///
/// Pure operator bundle -- guarantees arithmetic support.
/// No methods; construction (`zeros`, `ones`, `identity`) is via
/// inherent methods on specific type aliases.
///
/// Instead of writing:
///   `fn process<A: MultiArrayOps<f64> + Add<Output=A> + Sub<Output=A> + ...>`
/// you write:
///   `fn process<A: NumericMultiArrayOps<f64>>`
pub trait NumericMultiArrayOps<T>:
    MultiArrayOps<T>
    + Add<Self, Output = Self>
    + Sub<Self, Output = Self>
    + Neg<Output = Self>
    + Mul<T, Output = Self>
    + Div<T, Output = Self>
    + AddAssign<Self>
    + SubAssign<Self>
    + MulAssign<T>
    + DivAssign<T>
where
    T: Copy,
{
}

// ============================================================================
// Trait implementations for MultiArray
// ============================================================================

// MultiArrayOps -- base, implemented for ALL MultiArray types
impl<T: Clone, S: Shape, B: RawStorage<T> + Clone> MultiArrayOps<T> for MultiArray<T, S, B> {
    fn len(&self) -> usize {
        self.data.len()
    }
    fn rank(&self) -> usize {
        S::RANK
    }
}

// DenseMultiArrayOps -- implemented ONLY when B provides contiguous storage
impl<T: Clone, S: Shape, B: DenseRawStorage<T> + Clone> DenseMultiArrayOps<T>
    for MultiArray<T, S, B>
{
    fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
    fn as_mut_slice(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }
}

// NumericMultiArrayOps -- implemented ONLY when T is numeric and all operator
// bounds are satisfied. This is a marker trait (no methods).
impl<T, S: Shape, B: RawStorage<T> + Clone> NumericMultiArrayOps<T> for MultiArray<T, S, B>
where
    T: Copy
        + nalgebra::Scalar
        + num_traits::Zero
        + num_traits::One
        + Add<Output = T>
        + Sub<Output = T>
        + Mul<Output = T>
        + Div<Output = T>
        + AddAssign
        + SubAssign
        + MulAssign
        + DivAssign
        + Neg<Output = T>,
    MultiArray<T, S, B>: Add<Output = Self>
        + Sub<Output = Self>
        + Neg<Output = Self>
        + Mul<T, Output = Self>
        + Div<T, Output = Self>
        + AddAssign
        + SubAssign
        + MulAssign<T>
        + DivAssign<T>
        + Clone,
{
}

// ============================================================================
// Index / IndexMut
// ============================================================================

impl<T, S, B: DenseRawStorage<T>> Index<usize> for MultiArray<T, S, B> {
    type Output = T;
    fn index(&self, i: usize) -> &T {
        &self.data.as_slice()[i]
    }
}

impl<T, S, B: DenseRawStorage<T>> IndexMut<usize> for MultiArray<T, S, B> {
    fn index_mut(&mut self, i: usize) -> &mut T {
        &mut self.data.as_mut_slice()[i]
    }
}
