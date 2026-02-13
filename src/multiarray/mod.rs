//! Multi-dimensional array type system.
//!
//! Provides `MultiArray<T, S, B>` -- a single generic struct that all mathematical
//! types (vectors, matrices) are aliases of. The stable API is defined through
//! trait hierarchies (`MultiArrayOps`, `DenseMultiArrayOps`, `NumericMultiArrayOps`),
//! with domain-specific operations available via extension traits in the `linalg`
//! submodule.
//!
//! # Example
//!
//! ```
//! use strelitzia::multiarray::{Vector3, Matrix3};
//! use strelitzia::multiarray::linalg::{VectorOps, CrossProduct};
//!
//! let v = Vector3::new(1.0, 2.0, 3.0);
//! let w = Vector3::new(4.0, 5.0, 6.0);
//!
//! let dot = v.dot(&w);
//! let cross = v.cross(&w);
//! let mat = Matrix3::identity();
//! let result = mat * v;
//! ```

mod types;
mod traits;
mod operators;
mod aliases;
pub mod linalg;

pub use types::*;
pub use traits::*;
pub use aliases::*;
// operators.rs contains only trait impls (no new public items to re-export),
// but we still need the module to be compiled.
