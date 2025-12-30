// src/fields/types.rs
//! Zero-sized type markers for field types.

/// Scalar (rank 0): density, pressure, temperature.
pub struct Scalar;

/// Vector (rank 1): position, velocity, force.
pub struct Vector;

/// Tensor (rank n > 1): deformation gradient, rotation matrices.
pub struct Tensor;

/// Symmetric tensor (rank n > 1): stress, strain.
///
/// Stores n(n+1)/2 unique components in row-major upper triangle order:
/// - 2×2: (xx, xy, yy)
/// - 3×3: (xx, xy, xz, yy, yz, zz)
/// - 4×4: (xx, xy, xz, xw, yy, yz, yw, zz, zw, ww)
pub struct SymmTensor;
