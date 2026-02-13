//! Comprehensive tests for field compound assignment operator overloads.
//!
//! Binary operators that create new Fields are intentionally deleted.
//! Only in-place (compound assignment) operators are tested here.

use strelitzia::multiarray::Vector3;
use strelitzia::fields::{ScalarField, Vector3Field};

// ============================================================================
// Field-to-Field Compound Assignment Operations
// ============================================================================

#[test]
fn test_add_assign_field_to_field() {
    let mut field1 = ScalarField::new();
    field1.push(1.0);
    field1.push(2.0);

    let mut field2 = ScalarField::new();
    field2.push(10.0);
    field2.push(20.0);

    field1 += &field2;

    assert_eq!(field1[0], 11.0);
    assert_eq!(field1[1], 22.0);
}

#[test]
fn test_sub_assign_field_to_field() {
    let mut field1 = ScalarField::new();
    field1.push(10.0);
    field1.push(20.0);

    let mut field2 = ScalarField::new();
    field2.push(1.0);
    field2.push(2.0);

    field1 -= &field2;

    assert_eq!(field1[0], 9.0);
    assert_eq!(field1[1], 18.0);
}

// ============================================================================
// Scalar Multiplication
// ============================================================================

#[test]
fn test_mul_assign_scalar() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);
    field.push(3.0);

    field *= 2.0;

    assert_eq!(field[0], 2.0);
    assert_eq!(field[1], 4.0);
    assert_eq!(field[2], 6.0);
}

#[test]
fn test_mul_assign_vector() {
    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 2.0, 3.0));
    field.push(Vector3::new(4.0, 5.0, 6.0));

    field *= 2.0;

    assert_eq!(field[0], Vector3::new(2.0, 4.0, 6.0));
    assert_eq!(field[1], Vector3::new(8.0, 10.0, 12.0));
}

// ============================================================================
// Scalar Division
// ============================================================================

#[test]
fn test_div_assign_scalar() {
    let mut field = ScalarField::new();
    field.push(10.0);
    field.push(20.0);
    field.push(30.0);

    field /= 2.0;

    assert_eq!(field[0], 5.0);
    assert_eq!(field[1], 10.0);
    assert_eq!(field[2], 15.0);
}

// ============================================================================
// Scalar Addition / Subtraction
// ============================================================================

#[test]
fn test_add_assign_scalar() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);
    field.push(3.0);

    field += 5.0;

    assert_eq!(field[0], 6.0);
    assert_eq!(field[1], 7.0);
    assert_eq!(field[2], 8.0);
}

#[test]
fn test_sub_assign_scalar() {
    let mut field = ScalarField::new();
    field.push(10.0);
    field.push(20.0);
    field.push(30.0);

    field -= 3.0;

    assert_eq!(field[0], 7.0);
    assert_eq!(field[1], 17.0);
    assert_eq!(field[2], 27.0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_empty_field_compound_assign() {
    let mut empty1: ScalarField = ScalarField::new();
    let empty2: ScalarField = ScalarField::new();

    // Compound assignment on empty fields should succeed silently
    empty1 += &empty2;
    assert!(empty1.is_empty());

    empty1 *= 2.0;
    assert!(empty1.is_empty());

    empty1 += 5.0;
    assert!(empty1.is_empty());
}

#[test]
#[should_panic(expected = "Fields must have same length")]
fn test_add_assign_length_mismatch() {
    let mut field1 = ScalarField::new();
    field1.push(1.0);

    let mut field2 = ScalarField::new();
    field2.push(10.0);
    field2.push(20.0);

    field1 += &field2;
}

#[test]
#[should_panic(expected = "Fields must have same length")]
fn test_sub_assign_length_mismatch() {
    let mut field1 = ScalarField::new();
    field1.push(1.0);

    let mut field2 = ScalarField::new();
    field2.push(10.0);
    field2.push(20.0);

    field1 -= &field2;
}
