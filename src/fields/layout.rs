// src/fields/layout.rs
//! Core data structures: [`Field`] and internal [`Chunk`].

use std::marker::PhantomData;

/// Elements per chunk. Tuned for L1/L2 cache (1024 × 8 bytes × N components).
pub const CHUNK_SIZE: usize = 1024;

/// Internal storage chunk. Stores up to [`CHUNK_SIZE`] elements with
/// components in separate arrays (`data[component][element]`).
#[derive(Clone, Debug)]
pub(crate) struct Chunk<NumT, const N: usize> {
    pub data: [[NumT; CHUNK_SIZE]; N],
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
/// See [module docs](super) for usage examples.
///
/// # Type Parameters
///
/// * `T` - Type marker ([`Scalar`](super::Scalar), [`Vector`](super::Vector), etc.)
/// * `NumT` - Numeric type (`f64`, `f32`)
/// * `N` - Components per element
pub struct Field<T, NumT, const N: usize> {
    pub(crate) chunks: Vec<Chunk<NumT, N>>,
    pub(crate) total_len: usize,
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
    #[allow(clippy::needless_range_loop)] // Intentional for SIMD auto-vectorization
    pub fn push_raw(&mut self, components: [NumT; N]) {
        // Allocate new chunk if none exist or current is full
        if self.chunks.last().is_none_or(|b| b.len == CHUNK_SIZE) {
            self.chunks.push(Chunk::new());
        }
        let chunk = self.chunks.last_mut().unwrap();
        let idx = chunk.len;

        // AoSoA layout: data[component][element] for cache-friendly component streams
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
    struct TestType;

    #[test]
    fn new_field_is_empty() {
        let field: Field<TestType, f64, 3> = Field::new();
        assert_eq!(field.len(), 0);
        assert!(field.is_empty());
        assert!(field.chunks.is_empty());
    }

    #[test]
    fn default_equals_new() {
        let field1: Field<TestType, f64, 3> = Field::new();
        let field2: Field<TestType, f64, 3> = Field::default();
        assert_eq!(field1.len(), field2.len());
        assert_eq!(field1.chunks.len(), field2.chunks.len());
    }

    #[test]
    fn push_increments_len() {
        let mut field: Field<TestType, f64, 3> = Field::new();
        assert_eq!(field.len(), 0);

        field.push_raw([1.0, 2.0, 3.0]);
        assert_eq!(field.len(), 1);
        assert!(!field.is_empty());
    }

    #[test]
    fn push_multiple_elements() {
        let mut field: Field<TestType, f64, 3> = Field::new();

        for i in 0..10 {
            field.push_raw([i as f64, 0.0, 0.0]);
        }
        assert_eq!(field.len(), 10);
    }

    #[test]
    fn first_push_creates_chunk() {
        let mut field: Field<TestType, f64, 3> = Field::new();
        assert_eq!(field.chunks.len(), 0);

        field.push_raw([1.0, 2.0, 3.0]);
        assert_eq!(field.chunks.len(), 1);
        assert_eq!(field.chunks[0].len, 1);
    }

    #[test]
    fn chunk_boundary_creates_new_chunk() {
        let mut field: Field<TestType, f64, 1> = Field::new();

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
        let mut field: Field<TestType, f64, 1> = Field::new();
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
        let mut field: Field<TestType, f64, 1> = Field::new();
        field.push_raw([42.0]);
        assert_eq!(field.len(), 1);
        assert_eq!(field.chunks[0].data[0][0], 42.0);
    }

    #[test]
    fn vector_field_n3() {
        let mut field: Field<TestType, f64, 3> = Field::new();
        field.push_raw([1.0, 2.0, 3.0]);
        assert_eq!(field.chunks[0].data[0][0], 1.0);
        assert_eq!(field.chunks[0].data[1][0], 2.0);
        assert_eq!(field.chunks[0].data[2][0], 3.0);
    }

    #[test]
    fn tensor_field_n9() {
        let mut field: Field<TestType, f64, 9> = Field::new();
        let components = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        field.push_raw(components);

        for (i, &expected) in components.iter().enumerate() {
            assert_eq!(field.chunks[0].data[i][0], expected);
        }
    }

    #[test]
    fn f32_precision() {
        let mut field: Field<TestType, f32, 2> = Field::new();
        field.push_raw([1.5_f32, 2.5_f32]);
        assert_eq!(field.chunks[0].data[0][0], 1.5_f32);
        assert_eq!(field.chunks[0].data[1][0], 2.5_f32);
    }
}
