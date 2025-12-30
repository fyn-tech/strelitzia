// tests/fields_tests.rs
//! Integration tests for the fields module public API.

use strelitzia::fields::*;

#[test]
fn vector3_field_workflow() {
    let mut field: Vector3Field = Field::new();
    assert!(field.is_empty());

    field.push_raw([1.0, 2.0, 3.0]);
    field.push_raw([4.0, 5.0, 6.0]);

    assert_eq!(field.len(), 2);
    assert!(!field.is_empty());
}

#[test]
fn scalar_field_workflow() {
    let mut field: ScalarField = Field::new();

    field.push_raw([100.0]);
    field.push_raw([200.0]);
    field.push_raw([300.0]);

    assert_eq!(field.len(), 3);
}

#[test]
fn tensor_field_workflow() {
    let mut field: Tensor3Field = Field::new();

    // 3x3 tensor has 9 components
    let identity = [1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
    field.push_raw(identity);

    assert_eq!(field.len(), 1);
}

#[test]
fn symm_tensor_field_workflow() {
    let mut field: SymmTensor3Field = Field::new();

    // 3x3 symmetric tensor has 6 components (row-major upper triangle)
    let stress = [100.0, 10.0, 20.0, 200.0, 30.0, 300.0]; // xx, xy, xz, yy, yz, zz
    field.push_raw(stress);

    assert_eq!(field.len(), 1);
}

#[test]
fn field_default_trait() {
    let field: Vector3Field = Field::default();
    assert!(field.is_empty());
}

#[test]
fn large_field() {
    let mut field: Vector3Field = Field::new();

    // Push many elements to test chunk management
    for i in 0..5000 {
        field.push_raw([i as f64, 0.0, 0.0]);
    }

    assert_eq!(field.len(), 5000);
}

#[test]
fn all_type_aliases_compile() {
    // Verify all type aliases are accessible and work
    let _: ScalarField = Field::new();
    let _: Vector2Field = Field::new();
    let _: Vector3Field = Field::new();
    let _: Vector4Field = Field::new();
    let _: Tensor2Field = Field::new();
    let _: Tensor3Field = Field::new();
    let _: Tensor4Field = Field::new();
    let _: SymmTensor2Field = Field::new();
    let _: SymmTensor3Field = Field::new();
    let _: SymmTensor4Field = Field::new();
}

#[test]
fn type_markers_are_reexported() {
    // Verify type markers are accessible from the public API
    let _scalar: Field<Scalar, Real, 1> = Field::new();
    let _vector: Field<Vector, Real, 3> = Field::new();
    let _tensor: Field<Tensor, Real, 9> = Field::new();
    let _symm: Field<SymmTensor, Real, 6> = Field::new();
}

