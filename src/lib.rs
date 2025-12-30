//! Strelitzia - A zero-cost abstraction library for geometric computing.
//!
//! This library provides a unified interface for working with geometric primitives
//! (scalars, vectors, matrices) across different backend libraries like `nalgebra`
//! and `robust`.

pub mod fields;
pub mod geometry;
pub mod prelude;
pub mod visualiser;

pub fn run() {
    println!("Strelitzia is running...");
}
