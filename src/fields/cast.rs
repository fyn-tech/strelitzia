//! Zero-copy slice reinterpretation for solver interfaces.
//!
//! These functions are retained for backward compatibility and direct use.
//! The primary mechanism is now the generic `SolverInterop` implementation
//! via the `FieldElement` trait in `storage.rs`.

use crate::common::Real;
use crate::multiarray::{Vector3, Matrix3};

/// Reinterpret `&[Vector3]` as `&[Real]` (3 Reals per Vector3).
///
/// # Safety
/// MultiArray<T, Rank1<3>, na::SVector<T, 3>> is `#[repr(transparent)]`
/// over nalgebra::SVector which is laid out as 3 contiguous values.
pub fn as_flat_slice(vectors: &[Vector3]) -> &[Real] {
    if vectors.is_empty() {
        return &[];
    }
    unsafe { std::slice::from_raw_parts(vectors.as_ptr() as *const Real, vectors.len() * 3) }
}

/// Mutable version of `as_flat_slice`.
pub fn as_flat_slice_mut(vectors: &mut [Vector3]) -> &mut [Real] {
    if vectors.is_empty() {
        return &mut [];
    }
    unsafe {
        std::slice::from_raw_parts_mut(vectors.as_mut_ptr() as *mut Real, vectors.len() * 3)
    }
}

/// Reinterpret `&[Matrix3]` as `&[Real]` (9 Reals per Matrix3, column-major).
pub fn as_flat_slice_matrix3(matrices: &[Matrix3]) -> &[Real] {
    if matrices.is_empty() {
        return &[];
    }
    unsafe { std::slice::from_raw_parts(matrices.as_ptr() as *const Real, matrices.len() * 9) }
}

/// Mutable version of `as_flat_slice_matrix3`.
pub fn as_flat_slice_mut_matrix3(matrices: &mut [Matrix3]) -> &mut [Real] {
    if matrices.is_empty() {
        return &mut [];
    }
    unsafe {
        std::slice::from_raw_parts_mut(matrices.as_mut_ptr() as *mut Real, matrices.len() * 9)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slice_returns_empty() {
        let empty: &[Vector3] = &[];
        assert!(as_flat_slice(empty).is_empty());

        let mut empty_mut: Vec<Vector3> = vec![];
        assert!(as_flat_slice_mut(&mut empty_mut).is_empty());
    }

    #[test]
    fn vector3_roundtrip() {
        let vectors = vec![Vector3::new(1.0, 2.0, 3.0), Vector3::new(4.0, 5.0, 6.0)];

        let flat = as_flat_slice(&vectors);
        assert_eq!(flat, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn matrix3_column_major_layout() {
        // Matrix (row-major input to nalgebra's new()):
        // [1, 2, 3]
        // [4, 5, 6]
        // [7, 8, 9]
        let mat = Matrix3::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let matrices = vec![mat];

        let flat = as_flat_slice_matrix3(&matrices);

        // nalgebra stores column-major: col0, col1, col2
        // col0 = [1, 4, 7], col1 = [2, 5, 8], col2 = [3, 6, 9]
        assert_eq!(flat, &[1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0]);
    }
}
