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

/// Scalar field: density, pressure, temperature.
pub type ScalarField = Field<Scalar, Real, 1>;

// Vectors

/// 2D vector field: position, velocity.
pub type Vector2Field = Field<Vector, Real, 2>;
/// 3D vector field: position, velocity, force.
pub type Vector3Field = Field<Vector, Real, 3>;
/// 4D vector field: homogeneous coordinates, quaternions.
pub type Vector4Field = Field<Vector, Real, 4>;

// Tensors

/// 2×2 tensor field: 2D deformation gradient.
pub type Tensor2Field = Field<Tensor, Real, 4>;
/// 3×3 tensor field: deformation gradient, rotation.
pub type Tensor3Field = Field<Tensor, Real, 9>;
/// 4×4 tensor field: homogeneous transformations.
pub type Tensor4Field = Field<Tensor, Real, 16>;

// Symmetric Tensors (row-major upper triangle)

/// 2×2 symmetric: (xx, xy, yy). Stress, strain.
pub type SymmTensor2Field = Field<SymmTensor, Real, 3>;
/// 3×3 symmetric: (xx, xy, xz, yy, yz, zz). Stress, strain.
pub type SymmTensor3Field = Field<SymmTensor, Real, 6>;
/// 4×4 symmetric: (xx, xy, xz, xw, yy, yz, yw, zz, zw, ww).
pub type SymmTensor4Field = Field<SymmTensor, Real, 10>;
