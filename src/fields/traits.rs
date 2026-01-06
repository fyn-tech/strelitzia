// src/fields/traits.rs
//! Traits for field element access.

use super::layout::Field;
use super::types::*;

/// Provides typed element access. Output is `NumT` for scalars, `[NumT; N]` otherwise.
pub trait FieldView<NumT, const N: usize> {
    type Output;

    /// Returns element at `idx`. Panics if out of bounds.
    fn get(&self, idx: usize) -> Self::Output;
}

impl<NumT: Copy + Default> FieldView<NumT, 1> for Field<Scalar, NumT, 1> {
    type Output = NumT;

    fn get(&self, idx: usize) -> Self::Output {
        let chunk_idx = idx / super::layout::CHUNK_SIZE;
        let local_idx = idx % super::layout::CHUNK_SIZE;
        self.chunks[chunk_idx].data[0][local_idx]
    }
}

impl<NumT: Copy + Default, const N: usize> FieldView<NumT, N> for Field<Vector, NumT, N> {
    type Output = [NumT; N];

    #[allow(clippy::needless_range_loop)] // Intentional for SIMD auto-vectorization
    fn get(&self, idx: usize) -> Self::Output {
        let chunk_idx = idx / super::layout::CHUNK_SIZE;
        let local_idx = idx % super::layout::CHUNK_SIZE;

        let mut out = [NumT::default(); N];
        // AoSoA layout: data[component][element]
        for i in 0..N {
            out[i] = self.chunks[chunk_idx].data[i][local_idx];
        }
        out
    }
}

impl<NumT: Copy + Default, const N: usize> FieldView<NumT, N> for Field<Tensor, NumT, N> {
    type Output = [NumT; N];

    #[allow(clippy::needless_range_loop)] // Intentional for SIMD auto-vectorization
    fn get(&self, idx: usize) -> Self::Output {
        let chunk_idx = idx / super::layout::CHUNK_SIZE;
        let local_idx = idx % super::layout::CHUNK_SIZE;

        let mut out = [NumT::default(); N];
        for i in 0..N {
            out[i] = self.chunks[chunk_idx].data[i][local_idx];
        }
        out
    }
}

impl<NumT: Copy + Default, const N: usize> FieldView<NumT, N> for Field<SymmTensor, NumT, N> {
    type Output = [NumT; N];

    #[allow(clippy::needless_range_loop)] // Intentional for SIMD auto-vectorization
    fn get(&self, idx: usize) -> Self::Output {
        let chunk_idx = idx / super::layout::CHUNK_SIZE;
        let local_idx = idx % super::layout::CHUNK_SIZE;

        let mut out = [NumT::default(); N];
        for i in 0..N {
            out[i] = self.chunks[chunk_idx].data[i][local_idx];
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fields::layout::CHUNK_SIZE;

    #[test]
    fn scalar_field_get_returns_value() {
        let mut field: Field<Scalar, f64, 1> = Field::new();
        field.push_raw([42.0]);
        field.push_raw([99.0]);

        assert_eq!(field.get(0), 42.0);
        assert_eq!(field.get(1), 99.0);
    }

    #[test]
    fn vector_field_get_returns_array() {
        let mut field: Field<Vector, f64, 3> = Field::new();
        field.push_raw([1.0, 2.0, 3.0]);
        field.push_raw([4.0, 5.0, 6.0]);

        assert_eq!(field.get(0), [1.0, 2.0, 3.0]);
        assert_eq!(field.get(1), [4.0, 5.0, 6.0]);
    }

    #[test]
    fn tensor_field_get_returns_array() {
        let mut field: Field<Tensor, f64, 9> = Field::new();
        let t = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        field.push_raw(t);

        assert_eq!(field.get(0), t);
    }

    #[test]
    fn symm_tensor_field_get_returns_array() {
        let mut field: Field<SymmTensor, f64, 6> = Field::new();
        let s = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]; // xx, xy, xz, yy, yz, zz
        field.push_raw(s);

        assert_eq!(field.get(0), s);
    }

    #[test]
    fn get_across_chunk_boundary() {
        let mut field: Field<Scalar, f64, 1> = Field::new();

        // Fill first chunk and add one more
        for i in 0..=CHUNK_SIZE {
            field.push_raw([i as f64]);
        }

        // Element at CHUNK_SIZE is in second chunk
        assert_eq!(field.get(0), 0.0);
        assert_eq!(field.get(CHUNK_SIZE - 1), (CHUNK_SIZE - 1) as f64);
        assert_eq!(field.get(CHUNK_SIZE), CHUNK_SIZE as f64);
    }

    #[test]
    fn vector2_field() {
        let mut field: Field<Vector, f64, 2> = Field::new();
        field.push_raw([1.0, 2.0]);
        assert_eq!(field.get(0), [1.0, 2.0]);
    }

    #[test]
    fn f32_field_view() {
        let mut field: Field<Vector, f32, 3> = Field::new();
        field.push_raw([1.0_f32, 2.0_f32, 3.0_f32]);
        assert_eq!(field.get(0), [1.0_f32, 2.0_f32, 3.0_f32]);
    }
}
