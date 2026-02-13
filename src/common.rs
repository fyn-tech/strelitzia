//! Crate-wide foundational types.
//!
//! Contains definitions shared across all modules, starting with the
//! precision-controlled scalar type `Real`.

// ============================================================================
// Precision control via feature flag
// ============================================================================

/// The default floating-point scalar type.
///
/// Defaults to `f64`. Enable the `single-precision` feature to switch to `f32`.
#[cfg(feature = "single-precision")]
pub type Real = f32;
/// The default floating-point scalar type.
///
/// Defaults to `f64`. Enable the `single-precision` feature to switch to `f32`.
#[cfg(not(feature = "single-precision"))]
pub type Real = f64;
