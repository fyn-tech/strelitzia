//! Field storage containers.
//!
//! Generic Vec-based storage providing a consistent API and future extensibility
//! (GPU buffers, parallel iteration, metadata).

use crate::common::{Int, Real, UInt};
use crate::multiarray::*;
use nalgebra as na;

// ============================================================================
// FieldElement trait
// ============================================================================

/// Bridges `MultiArray` types to `Field<T>` and `SolverInterop`.
///
/// Expresses: "I am a fixed-size element that can be stored in a Field,
/// with known scalar type and component count."
pub trait FieldElement: Copy {
    type Scalar: Copy;
    const COMPONENTS: usize;
    fn component(&self, i: usize) -> Self::Scalar;
    fn from_scalars(data: &[Self::Scalar]) -> Self;
}

// FieldElement for Real (scalar)
impl FieldElement for Real {
    type Scalar = Real;
    const COMPONENTS: usize = 1;
    fn component(&self, _i: usize) -> Real {
        *self
    }
    fn from_scalars(data: &[Real]) -> Self {
        data[0]
    }
}

// FieldElement for static vectors
impl<T: na::Scalar + Copy, const N: usize> FieldElement
    for MultiArray<T, Rank1<N>, na::SVector<T, N>>
{
    type Scalar = T;
    const COMPONENTS: usize = N;
    fn component(&self, i: usize) -> T {
        self.as_inner().as_slice()[i]
    }
    fn from_scalars(data: &[T]) -> Self {
        Self::from_inner(na::SVector::from_column_slice(data))
    }
}

// FieldElement for static matrices
impl<T: na::Scalar + Copy, const R: usize, const C: usize> FieldElement
    for MultiArray<T, Rank2<R, C>, na::SMatrix<T, R, C>>
{
    type Scalar = T;
    const COMPONENTS: usize = R * C;
    fn component(&self, i: usize) -> T {
        self.as_inner().as_slice()[i]
    }
    fn from_scalars(data: &[T]) -> Self {
        Self::from_inner(na::SMatrix::from_column_slice(data))
    }
}

// ============================================================================
// Field struct
// ============================================================================

/// Generic field container for any element type.
///
/// Wraps `Vec<T>` to provide:
/// - Type safety and semantic clarity
/// - Consistent API across all field types
/// - Extensibility without breaking user code
#[derive(Debug, Clone)]
pub struct Field<T> {
    data: Vec<T>,
}

impl<T> Default for Field<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Field<T> {
    /// Creates a new empty field.
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// Creates a new field with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Returns the number of elements in the field.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the field contains no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Appends an element to the end of the field.
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    /// Returns the capacity of the field.
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Reserves capacity for at least `additional` more elements.
    pub fn reserve(&mut self, additional: usize) {
        self.data.reserve(additional);
    }

    /// Returns a slice containing all elements.
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Returns a mutable slice containing all elements.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Returns an iterator over references to elements.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.iter()
    }

    /// Returns an iterator over mutable references to elements.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.iter_mut()
    }

    /// Clears the field, removing all elements.
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Extends the field with elements from an iterator.
    pub fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = T>,
    {
        self.data.extend(iter);
    }
}

// Index trait for ergonomic element access: field[i]
use std::ops::{Index, IndexMut};

impl<T> Index<usize> for Field<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<T> IndexMut<usize> for Field<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

// ============================================================================
// SolverInterop
// ============================================================================

/// Trait for fields that can be reinterpreted as flat Real slices for solver interfaces.
///
/// This enables zero-copy access to field data in the format required by sparse
/// linear solvers (Ax = b where x is a flat array).
pub trait SolverInterop {
    /// Returns a flat slice view of the field data.
    ///
    /// For Vector3: [x₀, y₀, z₀, x₁, y₁, z₁, ...]
    /// For Matrix3: column-major per matrix (nalgebra default)
    fn as_flat_slice(&self) -> &[Real];

    /// Returns a mutable flat slice view for writing solver results.
    fn as_flat_slice_mut(&mut self) -> &mut [Real];
}

// Generic SolverInterop implementation for any Field<M> where M is a FieldElement
// with Real scalars.
impl<M: FieldElement<Scalar = Real>> SolverInterop for Field<M> {
    fn as_flat_slice(&self) -> &[Real] {
        if self.is_empty() {
            return &[];
        }
        // SAFETY: MultiArray is #[repr(transparent)] over nalgebra,
        // which stores contiguous Real values. FieldElement guarantees
        // COMPONENTS scalars per element.
        unsafe {
            std::slice::from_raw_parts(
                self.as_slice().as_ptr() as *const Real,
                self.len() * M::COMPONENTS,
            )
        }
    }

    fn as_flat_slice_mut(&mut self) -> &mut [Real] {
        if self.is_empty() {
            return &mut [];
        }
        unsafe {
            std::slice::from_raw_parts_mut(
                self.as_mut_slice().as_mut_ptr() as *mut Real,
                self.len() * M::COMPONENTS,
            )
        }
    }
}

// Type aliases -- {ElementType}Field pattern
pub type RealField = Field<Real>;
pub type ScalarField = RealField; // backward-compat synonym
pub type Vector3Field = Field<Vector3>;
pub type Matrix3Field = Field<Matrix3>;

pub type IntField = Field<Int>;
pub type UIntField = Field<UInt>;
pub type BoolField = Field<bool>;
pub type Vector3iField = Field<Vector3i>;
pub type Vector3uField = Field<Vector3u>;
pub type Vector3bField = Field<Vector3b>;
pub type Matrix3iField = Field<Matrix3i>;
pub type Matrix3uField = Field<Matrix3u>;
pub type Matrix3bField = Field<Matrix3b>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_field_basic_ops() {
        let mut field = ScalarField::new();
        assert!(field.is_empty());

        field.push(1.0);
        field.push(2.0);
        field.push(3.0);

        assert_eq!(field.len(), 3);
        assert_eq!(field[0], 1.0);
        assert_eq!(field[1], 2.0);
        assert_eq!(field[2], 3.0);

        field[1] = 42.0;
        assert_eq!(field[1], 42.0);
    }

    #[test]
    fn vector3_field_basic_ops() {
        let mut field = Vector3Field::new();
        field.push(Vector3::new(1.0, 2.0, 3.0));
        field.push(Vector3::new(4.0, 5.0, 6.0));

        assert_eq!(field.len(), 2);
        assert_eq!(field[0], Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(field[1], Vector3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn vector3_field_flat_slice() {
        let mut field = Vector3Field::new();
        field.push(Vector3::new(1.0, 2.0, 3.0));
        field.push(Vector3::new(4.0, 5.0, 6.0));

        let flat = field.as_flat_slice();
        assert_eq!(flat, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn vector3_field_flat_slice_mut() {
        let mut field = Vector3Field::new();
        field.push(Vector3::new(0.0, 0.0, 0.0));
        field.push(Vector3::new(0.0, 0.0, 0.0));

        // Simulate solver writing results
        let flat = field.as_flat_slice_mut();
        for (i, val) in flat.iter_mut().enumerate() {
            *val = (i + 1) as Real;
        }

        assert_eq!(field[0], Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(field[1], Vector3::new(4.0, 5.0, 6.0));
    }

    #[test]
    fn matrix3_field_basic_ops() {
        let mut field = Matrix3Field::new();
        let mat = Matrix3::identity();
        field.push(mat);

        assert_eq!(field.len(), 1);
        assert_eq!(field[0], Matrix3::identity());
    }

    #[test]
    fn matrix3_field_flat_slice() {
        let mut field = Matrix3Field::new();
        field.push(Matrix3::identity());

        let flat = field.as_flat_slice();
        assert_eq!(flat.len(), 9);
        // Identity matrix in column-major: [1,0,0, 0,1,0, 0,0,1]
        assert_eq!(flat[0], 1.0); // (0,0)
        assert_eq!(flat[4], 1.0); // (1,1)
        assert_eq!(flat[8], 1.0); // (2,2)
    }

    #[test]
    fn field_iteration() {
        let mut field = ScalarField::new();
        field.push(1.0);
        field.push(2.0);
        field.push(3.0);

        let sum: Real = field.iter().sum();
        assert_eq!(sum, 6.0);
    }

    #[test]
    fn field_mutation_via_iterator() {
        let mut field = ScalarField::new();
        field.push(1.0);
        field.push(2.0);

        for val in field.iter_mut() {
            *val *= 2.0;
        }

        assert_eq!(field[0], 2.0);
        assert_eq!(field[1], 4.0);
    }
}
