// src/fields/mod.rs
//!
//! High-performance field containers for scientific computing.
//!
//! This module provides [`Field`], an AoSoA (Array of Structures of Arrays)
//! container optimized for cache-efficient iteration and SIMD vectorization.
//!
//! # Quick Start
//!
//! ```
//! use strelitzia::fields::{Vector3Field, Field};
//!
//! let mut velocities: Vector3Field = Field::new();
//! velocities.push_raw([1.0, 2.0, 3.0]);
//! ```
//!
//! # Type Aliases
//!
//! | Alias | Type | Components |
//! |-------|------|------------|
//! | [`ScalarField`] | Scalar | 1 |
//! | [`Vector3Field`] | Vector | 3 |
//! | [`Tensor3Field`] | Tensor | 9 |
//! | [`SymmTensor3Field`] | SymmTensor | 6 |

pub mod layout;
pub mod types;
pub mod traits;
pub mod aliases;

// Re-export core types
pub use layout::Field;
pub use types::{Scalar, Vector, Tensor, SymmTensor};
pub use aliases::*;
