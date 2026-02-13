//! Generic field operations for all field types.
//!
//! Provides common operations that work across all `Field<T>` types where
//! the element type `T` supports the required operations.
//!
//! # Operator Overloads
//!
//! Only **compound assignment** operators are provided for `Field<T>` to avoid
//! hidden large allocations. Binary operators that would copy the entire data
//! array are intentionally omitted.
//!
//! ```rust
//! use strelitzia::fields::{ScalarField, Vector3Field};
//!
//! let mut field1 = ScalarField::new();
//! field1.push(1.0);
//! field1.push(2.0);
//!
//! let mut field2 = ScalarField::new();
//! field2.push(10.0);
//! field2.push(20.0);
//!
//! // Field-to-Field in-place operations
//! field1 += &field2;              // In-place addition
//! field1 -= &field2;              // In-place subtraction
//!
//! // Scalar in-place operations
//! field1 *= 2.0;                  // Scalar multiplication
//! field1 /= 2.0;                  // Scalar division
//! field1 += 5.0;                  // Scalar addition
//! field1 -= 3.0;                  // Scalar subtraction
//! ```

use super::Field;
use crate::common::Real;

/// Generic operations available for all field types with appropriate trait bounds.
pub trait FieldOps<T> {
    /// Fill all elements with the given value.
    fn fill(&mut self, value: T);

    /// Resize the field to the given length, filling with `value` if extending.
    fn resize(&mut self, new_len: usize, value: T);

    /// Clear all elements from the field.
    fn clear(&mut self);
}

impl<T: Clone> FieldOps<T> for Field<T> {
    fn fill(&mut self, value: T) {
        for elem in self.iter_mut() {
            *elem = value.clone();
        }
    }

    fn resize(&mut self, new_len: usize, value: T) {
        let current_len = self.len();
        if new_len > current_len {
            for _ in current_len..new_len {
                self.push(value.clone());
            }
        } else if new_len < current_len {
            let mut new_field = Field::with_capacity(new_len);
            for i in 0..new_len {
                new_field.push(self.as_slice()[i].clone());
            }
            *self = new_field;
        }
    }

    fn clear(&mut self) {
        self.clear();
    }
}

// ============================================================================
// Compound Assignment Operators (NO binary operators that allocate new Fields)
// ============================================================================

use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign};

// Field += &Field
impl<T> AddAssign<&Field<T>> for Field<T>
where
    T: Add<T, Output = T> + Copy,
{
    fn add_assign(&mut self, other: &Field<T>) {
        assert_eq!(self.len(), other.len(), "Fields must have same length");
        for (a, b) in self.iter_mut().zip(other.iter()) {
            *a = *a + *b;
        }
    }
}

// Field -= &Field
impl<T> SubAssign<&Field<T>> for Field<T>
where
    T: Sub<T, Output = T> + Copy,
{
    fn sub_assign(&mut self, other: &Field<T>) {
        assert_eq!(self.len(), other.len(), "Fields must have same length");
        for (a, b) in self.iter_mut().zip(other.iter()) {
            *a = *a - *b;
        }
    }
}

// Field *= Real
impl<T> MulAssign<Real> for Field<T>
where
    Real: Mul<T, Output = T>,
    T: Copy,
{
    fn mul_assign(&mut self, factor: Real) {
        for elem in self.iter_mut() {
            *elem = factor * *elem;
        }
    }
}

// Field /= Real
impl<T> DivAssign<Real> for Field<T>
where
    T: Div<Real, Output = T> + Copy,
{
    fn div_assign(&mut self, divisor: Real) {
        for elem in self.iter_mut() {
            *elem = *elem / divisor;
        }
    }
}

// Field += Real
impl<T> AddAssign<Real> for Field<T>
where
    T: Add<Real, Output = T> + Copy,
{
    fn add_assign(&mut self, scalar: Real) {
        for elem in self.iter_mut() {
            *elem = *elem + scalar;
        }
    }
}

// Field -= Real
impl<T> SubAssign<Real> for Field<T>
where
    T: Sub<Real, Output = T> + Copy,
{
    fn sub_assign(&mut self, scalar: Real) {
        for elem in self.iter_mut() {
            *elem = *elem - scalar;
        }
    }
}

// ============================================================================
// Reduction and sum operations
// ============================================================================

/// Reduction operations for fields with orderable element types.
pub trait ReductionOps<T> {
    /// Find the maximum element. Returns `None` if the field is empty.
    fn max(&self) -> Option<T>;
    /// Find the minimum element. Returns `None` if the field is empty.
    fn min(&self) -> Option<T>;
}

impl<T: PartialOrd + Copy> ReductionOps<T> for Field<T> {
    fn max(&self) -> Option<T> {
        self.iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
    }

    fn min(&self) -> Option<T> {
        self.iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
    }
}

/// Sum operations for fields with summable element types.
pub trait SumOps<T> {
    /// Sum all elements.
    fn sum(&self) -> T;
}

impl<T: std::iter::Sum<T>> SumOps<T> for Field<T>
where
    T: Copy,
{
    fn sum(&self) -> T {
        self.iter().copied().sum()
    }
}
