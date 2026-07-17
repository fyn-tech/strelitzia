//! Crate-wide foundational types.
//!
//! Contains definitions shared across all modules: the precision-controlled
//! floating-point scalar `Real`, and fixed-width integer types `Int` and `UInt`.

use std::fmt::Display;

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

/// Default absolute tolerance for Real comparisons: a power-of-two multiple
/// of the type's machine epsilon, so the scaling itself introduces no
/// additional rounding error.
#[cfg(feature = "single-precision")]
pub const EPSILON_ABS: Real = Real::EPSILON * 64.0;
/// Default absolute tolerance for Real comparisons: a power-of-two multiple
/// of the type's machine epsilon, so the scaling itself introduces no
/// additional rounding error.
#[cfg(not(feature = "single-precision"))]
pub const EPSILON_ABS: Real = Real::EPSILON * 1024.0;

/// Default relative tolerance for Real comparisons.
#[cfg(feature = "single-precision")]
pub const EPSILON_REL: Real = Real::EPSILON * 64.0;
/// Default relative tolerance for Real comparisons.
#[cfg(not(feature = "single-precision"))]
pub const EPSILON_REL: Real = Real::EPSILON * 1024.0;

pub fn abs_diff_eq(a: Real, b: Real, epsilon: Real) -> bool {
    (a - b).abs() <= epsilon
}

pub fn relative_eq(a: Real, b: Real, epsilon: Real) -> bool {
    let diff = (a - b).abs();
    diff <= epsilon * a.abs().max(b.abs())
}

pub fn approx_eq(a: Real, b: Real) -> bool {
    abs_diff_eq(a, b, EPSILON_ABS) || relative_eq(a, b, EPSILON_REL)
}

pub fn approx_gte(a: Real, b: Real) -> bool {
    a > b || abs_diff_eq(a, b, EPSILON_ABS) || relative_eq(a, b, EPSILON_REL)
}

pub fn approx_lte(a: Real, b: Real) -> bool {
    a < b || abs_diff_eq(a, b, EPSILON_ABS) || relative_eq(a, b, EPSILON_REL)
}

pub fn approx_gt(a: Real, b: Real) -> bool {
    !approx_lte(a, b)
}

pub fn approx_lt(a: Real, b: Real) -> bool {
    !approx_gte(a, b)
}

// ============================================================================
// Integer types (fixed width, no feature flag)
// ============================================================================

/// The default signed integer type. Always `i64`.
pub type Int = i64;

/// The default unsigned integer type. Always `u64`.
pub type UInt = u64;

// ============================================================================
// Macro-like functions
// ============================================================================

pub fn bounds_failure_str<T>(index: T, upper_bounds: T) -> String
where
    T: Display,
{
    format!("Index {} not in range [0, {}).", index, upper_bounds)
}
