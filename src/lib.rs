//! Strelitzia -- a computational physics library in Rust.
//!
//! Provides core infrastructure for high-performance simulations:
//!
//! - [`common`] -- crate-wide types (`Real` precision alias)
//! - [`multiarray`] -- mathematical type system (`Vector3`, `Matrix3`, etc.)
//! - [`fields`] -- simulation data collections with zero-copy solver interop
//! - [`geometry`] -- mesh generation and Voronoi tessellation
//! - [`visualiser`] -- VTK export for ParaView visualisation

pub mod b_rep;
pub mod common;
pub mod fields;
pub mod multiarray;
pub mod prelude;
pub mod visualiser;
