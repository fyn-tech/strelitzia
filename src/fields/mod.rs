//! Field storage for computational physics.
//!
//! Provides `Field<T>` -- a generic collection container for simulation data.
//! Elements are typically `MultiArray` types (from `crate::multiarray`), bridged
//! via the `FieldElement` trait for zero-copy solver interop.
//!
//! # Example
//!
//! ```
//! use strelitzia::multiarray::Vector3;
//! use strelitzia::fields::{Vector3Field, SolverInterop};
//!
//! let mut positions = Vector3Field::new();
//! positions.push(Vector3::new(1.0, 2.0, 3.0));
//! positions.push(Vector3::new(4.0, 5.0, 6.0));
//!
//! // Zero-copy access for solvers
//! let flat: &[f64] = positions.as_flat_slice();
//! assert_eq!(flat, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
//! ```

mod storage;
mod cast;
mod ops;

pub use storage::*;
pub use cast::*;
pub use ops::*;
