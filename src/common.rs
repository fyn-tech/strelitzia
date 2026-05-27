//! Crate-wide foundational types.
//!
//! Contains definitions shared across all modules: the precision-controlled
//! floating-point scalar `Real`, and fixed-width integer types `Int` and `UInt`.

// ============================================================================
// Floating-point precision control via feature flag
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

// ============================================================================
// Integer types (fixed width, no feature flag)
// ============================================================================

/// The default signed integer type. Always `i64`.
pub type Int = i64;

/// The default unsigned integer type. Always `u64`.
pub type UInt = u64;
