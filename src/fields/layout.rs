// src/fields/layout.rs
//!
//! Core data structures for the fields module.
//!
//! This module provides [`Field`], a high-performance container using
//! Array of Structures of Arrays (AoSoA) memory layout for cache-efficient
//! iteration and SIMD-friendly operations.

use std::marker::PhantomData;

/// Number of elements per chunk.
///
/// This is a tuning parameter that affects cache performance.
/// 1024 elements × 8 bytes × N components should fit in L1/L2 cache.
pub const CHUNK_SIZE: usize = 1024;

/// Low-level storage chunk using AoSoA (Array of Structures of Arrays) layout.
///
/// Each chunk stores up to [`CHUNK_SIZE`] elements, with components stored
/// in separate contiguous arrays for cache locality and SIMD vectorization.
///
/// # Type Parameters
///
/// * `NumT` - Numeric type for each component (`f64`, `f32`)
/// * `N` - Number of components per element
#[derive(Clone, Debug)]
pub(crate) struct Chunk<NumT, const N: usize> {
    /// Component arrays: `data[component][element_index]`
    pub data: [[NumT; CHUNK_SIZE]; N],
    /// Number of active elements in this chunk (may be < CHUNK_SIZE for last chunk)
    pub len: usize,
}

impl<NumT: Copy + Default, const N: usize> Chunk<NumT, N> {
    /// Creates a new empty chunk with zero-initialized data.
    pub fn new() -> Self {
        Self {
            data: [[NumT::default(); CHUNK_SIZE]; N],
            len: 0,
        }
    }
}

/// High-performance field container using AoSoA memory layout.
///
/// Optimized for cache-efficient iteration, SIMD vectorization, and solver interop.
///
/// # Type Parameters
///
/// * `T` - Field type marker (`Scalar`, `Vector`, `Tensor`, `SymmTensor`)
/// * `NumT` - Numeric storage type (`f64`, `f32`). Must be `Copy + Default`.
/// * `N` - Components per element
///
/// # Example
///
/// ```
/// use strelitzia::fields::{Vector3Field, Field};
///
/// let mut velocities: Vector3Field = Field::new();
/// velocities.push_raw([1.0, 2.0, 3.0]);
/// ```
pub struct Field<T, NumT, const N: usize> {
    /// Storage chunks
    pub(crate) chunks: Vec<Chunk<NumT, N>>,
    /// Total number of elements across all chunks
    pub(crate) total_len: usize,
    /// Zero-sized marker for the mathematical type
    _marker: PhantomData<T>,
}

impl<T, NumT: Copy + Default, const N: usize> Field<T, NumT, N> {
    /// Creates a new empty field.
    pub fn new() -> Self {
        Self {
            chunks: Vec::new(),
            total_len: 0,
            _marker: PhantomData,
        }
    }

    /// Appends an element with the given component values.
    pub fn push_raw(&mut self, components: [NumT; N]) {
        if self.chunks.last().map_or(true, |b| b.len == CHUNK_SIZE) {
            self.chunks.push(Chunk::new());
        }
        let chunk = self.chunks.last_mut().unwrap();
        let idx = chunk.len;

        // Auto-vectorized copy
        for i in 0..N {
            chunk.data[i][idx] = components[i];
        }
        chunk.len += 1;
        self.total_len += 1;
    }

    /// Returns the number of elements in the field.
    pub fn len(&self) -> usize {
        self.total_len
    }

    /// Returns `true` if the field contains no elements.
    pub fn is_empty(&self) -> bool {
        self.total_len == 0
    }
}

impl<T, NumT: Copy + Default, const N: usize> Default for Field<T, NumT, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use a dummy marker type for testing layout without importing types module
    struct TestMarker;

    #[test]
    fn new_field_is_empty() {
        let field: Field<TestMarker, f64, 3> = Field::new();
        assert_eq!(field.len(), 0);
        assert!(field.is_empty());
        assert!(field.chunks.is_empty());
    }

    #[test]
    fn default_equals_new() {
        let field1: Field<TestMarker, f64, 3> = Field::new();
        let field2: Field<TestMarker, f64, 3> = Field::default();
        assert_eq!(field1.len(), field2.len());
        assert_eq!(field1.chunks.len(), field2.chunks.len());
    }

    #[test]
    fn push_increments_len() {
        let mut field: Field<TestMarker, f64, 3> = Field::new();
        assert_eq!(field.len(), 0);

        field.push_raw([1.0, 2.0, 3.0]);
        assert_eq!(field.len(), 1);
        assert!(!field.is_empty());
    }

    #[test]
    fn push_multiple_elements() {
        let mut field: Field<TestMarker, f64, 3> = Field::new();

        for i in 0..10 {
            field.push_raw([i as f64, 0.0, 0.0]);
        }
        assert_eq!(field.len(), 10);
    }

    #[test]
    fn first_push_creates_chunk() {
        let mut field: Field<TestMarker, f64, 3> = Field::new();
        assert_eq!(field.chunks.len(), 0);

        field.push_raw([1.0, 2.0, 3.0]);
        assert_eq!(field.chunks.len(), 1);
        assert_eq!(field.chunks[0].len, 1);
    }

    #[test]
    fn chunk_boundary_creates_new_chunk() {
        let mut field: Field<TestMarker, f64, 1> = Field::new();

        // Fill first chunk completely
        for i in 0..CHUNK_SIZE {
            field.push_raw([i as f64]);
        }
        assert_eq!(field.chunks.len(), 1);
        assert_eq!(field.chunks[0].len, CHUNK_SIZE);

        // One more push should create a new chunk
        field.push_raw([9999.0]);
        assert_eq!(field.chunks.len(), 2);
        assert_eq!(field.chunks[1].len, 1);
        assert_eq!(field.len(), CHUNK_SIZE + 1);
    }

    #[test]
    fn partial_last_chunk() {
        let mut field: Field<TestMarker, f64, 1> = Field::new();
        let n = 1500;

        for i in 0..n {
            field.push_raw([i as f64]);
        }

        assert_eq!(field.len(), n);
        assert_eq!(field.chunks.len(), 2);
        assert_eq!(field.chunks[0].len, CHUNK_SIZE); // 1024
        assert_eq!(field.chunks[1].len, n - CHUNK_SIZE); // 476
    }

    #[test]
    fn scalar_field_n1() {
        let mut field: Field<TestMarker, f64, 1> = Field::new();
        field.push_raw([42.0]);
        assert_eq!(field.len(), 1);
        assert_eq!(field.chunks[0].data[0][0], 42.0);
    }

    #[test]
    fn vector_field_n3() {
        let mut field: Field<TestMarker, f64, 3> = Field::new();
        field.push_raw([1.0, 2.0, 3.0]);
        assert_eq!(field.chunks[0].data[0][0], 1.0);
        assert_eq!(field.chunks[0].data[1][0], 2.0);
        assert_eq!(field.chunks[0].data[2][0], 3.0);
    }

    #[test]
    fn tensor_field_n9() {
        let mut field: Field<TestMarker, f64, 9> = Field::new();
        let components = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        field.push_raw(components);

        for i in 0..9 {
            assert_eq!(field.chunks[0].data[i][0], components[i]);
        }
    }

    #[test]
    fn f32_precision() {
        let mut field: Field<TestMarker, f32, 2> = Field::new();
        field.push_raw([1.5_f32, 2.5_f32]);
        assert_eq!(field.chunks[0].data[0][0], 1.5_f32);
        assert_eq!(field.chunks[0].data[1][0], 2.5_f32);
    }
}
