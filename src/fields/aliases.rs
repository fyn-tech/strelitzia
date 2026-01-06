// src/fields/aliases.rs
//! Type aliases for common field configurations.

use super::layout::Field;
use super::types::*;

/// Default floating-point type. Use `--features single-precision` for `f32`.
#[cfg(not(feature = "single-precision"))]
pub type Real = f64;
#[cfg(feature = "single-precision")]
pub type Real = f32;

// Scalars
pub type ScalarField = Field<Scalar, Real, 1>;

// Vectors
pub type Vector2Field = Field<Vector, Real, 2>;
pub type Vector3Field = Field<Vector, Real, 3>;
pub type Vector4Field = Field<Vector, Real, 4>;

// Tensors (N×N = N² components)
pub type Tensor2Field = Field<Tensor, Real, 4>;
pub type Tensor3Field = Field<Tensor, Real, 9>;
pub type Tensor4Field = Field<Tensor, Real, 16>;

// Symmetric tensors (N(N+1)/2 components, see [`SymmTensor`] for layout)
pub type SymmTensor2Field = Field<SymmTensor, Real, 3>;
pub type SymmTensor3Field = Field<SymmTensor, Real, 6>;
pub type SymmTensor4Field = Field<SymmTensor, Real, 10>;
