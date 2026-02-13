//! Tests for generic field operations.

use strelitzia::multiarray::Vector3;
use strelitzia::fields::{FieldOps, ReductionOps, ScalarField, SumOps, Vector3Field};

#[test]
fn test_field_ops_fill() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);
    field.push(3.0);

    field.fill(42.0);

    assert_eq!(field[0], 42.0);
    assert_eq!(field[1], 42.0);
    assert_eq!(field[2], 42.0);
}

#[test]
fn test_field_ops_resize_extend() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);

    field.resize(5, 99.0);

    assert_eq!(field.len(), 5);
    assert_eq!(field[0], 1.0);
    assert_eq!(field[1], 2.0);
    assert_eq!(field[2], 99.0);
    assert_eq!(field[3], 99.0);
    assert_eq!(field[4], 99.0);
}

#[test]
fn test_field_ops_resize_truncate() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);
    field.push(3.0);
    field.push(4.0);

    field.resize(2, 0.0);

    assert_eq!(field.len(), 2);
    assert_eq!(field[0], 1.0);
    assert_eq!(field[1], 2.0);
}

#[test]
fn test_field_ops_clear() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);

    field.clear();

    assert!(field.is_empty());
    assert_eq!(field.len(), 0);
}

#[test]
fn test_operator_mul_assign_scalar() {
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
fn test_operator_mul_assign_vector() {
    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 2.0, 3.0));
    field.push(Vector3::new(4.0, 5.0, 6.0));

    field *= 2.0;

    assert_eq!(field[0], Vector3::new(2.0, 4.0, 6.0));
    assert_eq!(field[1], Vector3::new(8.0, 10.0, 12.0));
}

#[test]
fn test_operator_add_assign_scalar() {
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
fn test_operator_add_assign_vector() {
    let mut field1 = Vector3Field::new();
    field1.push(Vector3::new(1.0, 2.0, 3.0));

    let mut field2 = Vector3Field::new();
    field2.push(Vector3::new(10.0, 20.0, 30.0));

    field1 += &field2;

    assert_eq!(field1[0], Vector3::new(11.0, 22.0, 33.0));
}

#[test]
#[should_panic(expected = "Fields must have same length")]
fn test_operator_add_assign_length_mismatch() {
    let mut field1 = ScalarField::new();
    field1.push(1.0);

    let mut field2 = ScalarField::new();
    field2.push(10.0);
    field2.push(20.0);

    field1 += &field2;
}

#[test]
fn test_operator_sub_assign() {
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

#[test]
fn test_operator_div_assign() {
    let mut field = ScalarField::new();
    field.push(10.0);
    field.push(20.0);
    field.push(30.0);

    field /= 2.0;

    assert_eq!(field[0], 5.0);
    assert_eq!(field[1], 10.0);
    assert_eq!(field[2], 15.0);
}

#[test]
fn test_operator_add_assign_real() {
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
fn test_operator_sub_assign_real() {
    let mut field = ScalarField::new();
    field.push(10.0);
    field.push(20.0);
    field.push(30.0);

    field -= 3.0;

    assert_eq!(field[0], 7.0);
    assert_eq!(field[1], 17.0);
    assert_eq!(field[2], 27.0);
}

#[test]
fn test_reduction_ops_max() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(5.0);
    field.push(3.0);

    assert_eq!(field.max(), Some(5.0));
}

#[test]
fn test_reduction_ops_min() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(5.0);
    field.push(3.0);

    assert_eq!(field.min(), Some(1.0));
}

#[test]
fn test_reduction_ops_empty() {
    let field = ScalarField::new();
    assert_eq!(field.max(), None);
    assert_eq!(field.min(), None);
}

#[test]
fn test_sum_ops_scalar() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);
    field.push(3.0);

    assert_eq!(field.sum(), 6.0);
}

#[test]
fn test_sum_ops_vector() {
    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 2.0, 3.0));
    field.push(Vector3::new(4.0, 5.0, 6.0));

    let sum = field.sum();
    assert_eq!(sum, Vector3::new(5.0, 7.0, 9.0));
}
