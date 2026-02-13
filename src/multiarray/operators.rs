//! Blanket operator implementations for `MultiArray`.
//!
//! Provides `std::ops` trait implementations (Add, Sub, Neg, Mul, Div and
//! their compound-assignment variants), `std::iter::Sum`, and dedicated
//! matrix multiplication impls.

use super::types::*;
use crate::common::Real;
use nalgebra as na;
use std::marker::PhantomData;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

// ============================================================================
// Blanket operator impls
// ============================================================================

// --- Binary operators ---

impl<T, S: Shape, B: Add<Output = B>> Add for MultiArray<T, S, B> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            data: self.data + rhs.data,
            _phantoms: PhantomData,
        }
    }
}

impl<T, S: Shape, B: Sub<Output = B>> Sub for MultiArray<T, S, B> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            data: self.data - rhs.data,
            _phantoms: PhantomData,
        }
    }
}

impl<T, S: Shape, B: Neg<Output = B>> Neg for MultiArray<T, S, B> {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            data: -self.data,
            _phantoms: PhantomData,
        }
    }
}

// Scalar multiplication: Real * MultiArray  (scalar on the left only)
impl<S: Shape, B: Mul<Real, Output = B>> Mul<MultiArray<Real, S, B>> for Real {
    type Output = MultiArray<Real, S, B>;
    fn mul(self, rhs: MultiArray<Real, S, B>) -> MultiArray<Real, S, B> {
        MultiArray {
            data: rhs.data * self,
            _phantoms: PhantomData,
        }
    }
}

// Scalar division: MultiArray / T
impl<T: Copy, S: Shape, B: Div<T, Output = B>> Div<T> for MultiArray<T, S, B> {
    type Output = Self;
    fn div(self, scalar: T) -> Self {
        Self {
            data: self.data / scalar,
            _phantoms: PhantomData,
        }
    }
}

// --- Compound assignment operators ---

impl<T, S: Shape, B: AddAssign> AddAssign for MultiArray<T, S, B> {
    fn add_assign(&mut self, rhs: Self) {
        self.data += rhs.data;
    }
}

impl<T, S: Shape, B: SubAssign> SubAssign for MultiArray<T, S, B> {
    fn sub_assign(&mut self, rhs: Self) {
        self.data -= rhs.data;
    }
}

impl<T: Copy, S: Shape, B: MulAssign<T>> MulAssign<T> for MultiArray<T, S, B> {
    fn mul_assign(&mut self, scalar: T) {
        self.data *= scalar;
    }
}

impl<T: Copy, S: Shape, B: DivAssign<T>> DivAssign<T> for MultiArray<T, S, B> {
    fn div_assign(&mut self, scalar: T) {
        self.data /= scalar;
    }
}

// ============================================================================
// std::iter::Sum
// ============================================================================

impl<T, S: Shape, B> std::iter::Sum for MultiArray<T, S, B>
where
    B: Add<Output = B> + num_traits::Zero,
{
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(
            Self {
                data: B::zero(),
                _phantoms: PhantomData,
            },
            |acc, x| Self {
                data: acc.data + x.data,
                _phantoms: PhantomData,
            },
        )
    }
}

// ============================================================================
// Matrix multiplication (cross-type Mul impls)
// ============================================================================

// Matrix * Matrix -> Matrix
// Matrix<T, R, K> * Matrix<T, K, C> -> Matrix<T, R, C>
impl<T, const R: usize, const K: usize, const C: usize>
    Mul<MultiArray<T, Rank2<K, C>, na::SMatrix<T, K, C>>>
    for MultiArray<T, Rank2<R, K>, na::SMatrix<T, R, K>>
where
    T: na::Scalar
        + Copy
        + num_traits::Zero
        + num_traits::One
        + AddAssign
        + MulAssign
        + na::ClosedAddAssign
        + na::ClosedMulAssign,
{
    type Output = MultiArray<T, Rank2<R, C>, na::SMatrix<T, R, C>>;
    fn mul(
        self,
        rhs: MultiArray<T, Rank2<K, C>, na::SMatrix<T, K, C>>,
    ) -> Self::Output {
        let result: na::SMatrix<T, R, C> = self.into_inner() * rhs.into_inner();
        MultiArray::from_inner(result)
    }
}

// Matrix * Vector -> Vector
// Matrix<T, R, C> * Vector<T, C> -> Vector<T, R>
impl<T, const R: usize, const C: usize>
    Mul<MultiArray<T, Rank1<C>, na::SVector<T, C>>>
    for MultiArray<T, Rank2<R, C>, na::SMatrix<T, R, C>>
where
    T: na::Scalar
        + Copy
        + num_traits::Zero
        + num_traits::One
        + AddAssign
        + MulAssign
        + na::ClosedAddAssign
        + na::ClosedMulAssign,
{
    type Output = MultiArray<T, Rank1<R>, na::SVector<T, R>>;
    fn mul(
        self,
        rhs: MultiArray<T, Rank1<C>, na::SVector<T, C>>,
    ) -> Self::Output {
        let result: na::SVector<T, R> = self.into_inner() * rhs.into_inner();
        MultiArray::from_inner(result)
    }
}

// Vector * RowVector -> Matrix (outer product via transpose)
// Vector<T, N> * Matrix<T, 1, M> -> Matrix<T, N, M>
impl<T, const N: usize, const M: usize>
    Mul<MultiArray<T, Rank2<1, M>, na::SMatrix<T, 1, M>>>
    for MultiArray<T, Rank1<N>, na::SVector<T, N>>
where
    T: na::Scalar
        + Copy
        + num_traits::Zero
        + num_traits::One
        + AddAssign
        + MulAssign
        + na::ClosedAddAssign
        + na::ClosedMulAssign,
{
    type Output = MultiArray<T, Rank2<N, M>, na::SMatrix<T, N, M>>;
    fn mul(
        self,
        rhs: MultiArray<T, Rank2<1, M>, na::SMatrix<T, 1, M>>,
    ) -> Self::Output {
        let result: na::SMatrix<T, N, M> = self.into_inner() * rhs.into_inner();
        MultiArray::from_inner(result)
    }
}
