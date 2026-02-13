//! Integration tests for the fields module.

use strelitzia::common::Real;
use strelitzia::multiarray::{Matrix3, Vector3};
use strelitzia::multiarray::linalg::{VectorOps, CrossProduct};
use strelitzia::fields::{Matrix3Field, ScalarField, SolverInterop, Vector3Field};

#[test]
fn scalar_field_public_api() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);

    assert_eq!(field.len(), 2);
    assert_eq!(field[0], 1.0);
    assert_eq!(field.as_slice(), &[1.0, 2.0]);
}

#[test]
fn vector3_field_public_api() {
    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 2.0, 3.0));

    assert_eq!(field.len(), 1);
    assert_eq!(field[0], Vector3::new(1.0, 2.0, 3.0));
}

#[test]
fn vector3_field_solver_interface() {
    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 2.0, 3.0));
    field.push(Vector3::new(4.0, 5.0, 6.0));

    // Zero-copy access for Ax = b solvers
    let x: &[Real] = field.as_flat_slice();
    assert_eq!(x.len(), 6);
    assert_eq!(x, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
}

#[test]
fn vector3_field_solver_writeback() {
    let mut field = Vector3Field::with_capacity(2);
    field.push(Vector3::zeros());
    field.push(Vector3::zeros());

    // Simulate solver writing results directly
    let x: &mut [Real] = field.as_flat_slice_mut();
    x[0] = 1.0;
    x[1] = 2.0;
    x[2] = 3.0;
    x[3] = 4.0;
    x[4] = 5.0;
    x[5] = 6.0;

    // Verify data is in Vector3Field
    assert_eq!(field[0], Vector3::new(1.0, 2.0, 3.0));
    assert_eq!(field[1], Vector3::new(4.0, 5.0, 6.0));
}

#[test]
fn matrix3_field_public_api() {
    let mut field = Matrix3Field::new();
    field.push(Matrix3::identity());

    assert_eq!(field.len(), 1);
    assert_eq!(field[0], Matrix3::identity());
}

#[test]
fn field_iteration() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);
    field.push(3.0);

    let sum: Real = field.iter().sum();
    assert_eq!(sum, 6.0);
}

#[test]
fn field_mutation_via_iterator() {
    let mut field = ScalarField::new();
    field.push(1.0);
    field.push(2.0);

    for val in field.iter_mut() {
        *val *= 2.0;
    }

    assert_eq!(field[0], 2.0);
    assert_eq!(field[1], 4.0);
}

#[test]
fn vector3_math_operations() {
    let a = Vector3::new(1.0, 2.0, 3.0);
    let b = Vector3::new(4.0, 5.0, 6.0);

    // Operations via extension traits and operators
    let sum = a + b;
    let dot = a.dot(&b);
    let cross = a.cross(&b);
    let scaled = 2.0 * a;
    let norm = a.norm();

    assert_eq!(sum, Vector3::new(5.0, 7.0, 9.0));
    assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6
    assert_eq!(cross, Vector3::new(-3.0, 6.0, -3.0));
    assert_eq!(scaled, Vector3::new(2.0, 4.0, 6.0));
    assert!((norm - 14.0_f64.sqrt()).abs() < 1e-10);
}
