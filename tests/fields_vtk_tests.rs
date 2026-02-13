//! Integration tests for field VTK export.

use std::fs;
use std::path::PathBuf;
use strelitzia::multiarray::Vector3;
use strelitzia::fields::{ScalarField, Vector3Field};
use strelitzia::visualiser::*;

/// RAII guard for automatic test file cleanup
struct TestFileGuard {
    path: PathBuf,
}

impl TestFileGuard {
    fn new(name: &str) -> Self {
        Self {
            path: std::env::temp_dir().join(format!("strelitzia_test_fields_{}", name)),
        }
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for TestFileGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Verify file exists and has content
fn verify_file_exists(path: &PathBuf) {
    assert!(path.exists(), "File should exist: {:?}", path);
    let metadata = fs::metadata(path).expect("Should read metadata");
    assert!(metadata.len() > 0, "File should not be empty");
}

/// Verify VTK XML structure and content
fn verify_vtk_content(
    path: &PathBuf,
    expected_points: usize,
    field_name: &str,
    num_components: usize,
) {
    let content = fs::read_to_string(path).expect("Should read file");

    assert!(
        content.starts_with("<?xml version=\"1.0\"?>"),
        "Should have XML header"
    );
    assert!(
        content.contains("<VTKFile type=\"UnstructuredGrid\""),
        "Should be UnstructuredGrid"
    );
    assert!(
        content.contains(&format!("NumberOfPoints=\"{}\"", expected_points)),
        "Should have {} points",
        expected_points
    );
    assert!(
        content.contains(&format!("Name=\"{}\"", field_name)),
        "Should contain field named '{}'",
        field_name
    );
    assert!(
        content.contains(&format!("NumberOfComponents=\"{}\"", num_components)),
        "Field should have {} components",
        num_components
    );
}

#[test]
fn test_scalar_field_vtk_export_ascii() {
    let guard = TestFileGuard::new("scalar_ascii.vtu");

    let mut field = ScalarField::new();
    field.push(25.0);
    field.push(30.0);
    field.push(28.0);

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];

    let temp_array = scalar_field_to_vtk_array("temperature", &field);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[temp_array],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write VTU");

    verify_file_exists(guard.path());
    verify_vtk_content(guard.path(), 3, "temperature", 1);

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(content.contains("format=\"ascii\""), "Should be ASCII format");
    assert!(content.contains("25"), "Should contain temperature value 25");
}

#[test]
fn test_scalar_field_vtk_export_base64() {
    let guard = TestFileGuard::new("scalar_base64.vtu");

    let mut field = ScalarField::new();
    field.push(100.0);
    field.push(200.0);

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]];

    let pressure_array = scalar_field_to_vtk_array("pressure", &field);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[pressure_array],
        &[],
        Encoding::Base64,
    )
    .expect("Should write VTU");

    verify_file_exists(guard.path());
    verify_vtk_content(guard.path(), 2, "pressure", 1);

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("format=\"binary\""),
        "Should be Base64 format"
    );
}

#[test]
fn test_vector3_field_vtk_export_ascii() {
    let guard = TestFileGuard::new("vector3_ascii.vtu");

    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 0.0, 0.0));
    field.push(Vector3::new(0.0, 1.0, 0.0));
    field.push(Vector3::new(0.5, 0.5, 0.0));

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];

    let vel_array = vector3_field_to_vtk_array("velocity", &field);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[vel_array],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write VTU");

    verify_file_exists(guard.path());
    verify_vtk_content(guard.path(), 3, "velocity", 3);

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(content.contains("format=\"ascii\""), "Should be ASCII format");
}

#[test]
fn test_vector3_field_vtk_export_base64() {
    let guard = TestFileGuard::new("vector3_base64.vtu");

    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 2.0, 3.0));
    field.push(Vector3::new(4.0, 5.0, 6.0));

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];

    let force_array = vector3_field_to_vtk_array("force", &field);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[force_array],
        &[],
        Encoding::Base64,
    )
    .expect("Should write VTU");

    verify_file_exists(guard.path());
    verify_vtk_content(guard.path(), 2, "force", 3);
}

#[test]
fn test_multiple_fields_export() {
    let guard = TestFileGuard::new("multiple_fields.vtu");

    let mut temperature = ScalarField::new();
    temperature.push(300.0);
    temperature.push(350.0);

    let mut velocity = Vector3Field::new();
    velocity.push(Vector3::new(1.0, 0.0, 0.0));
    velocity.push(Vector3::new(0.0, 1.0, 0.0));

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 1.0, 0.0]];

    let temp_array = scalar_field_to_vtk_array("temperature", &temperature);
    let vel_array = vector3_field_to_vtk_array("velocity", &velocity);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[temp_array, vel_array],
        &[],
        Encoding::Base64,
    )
    .expect("Should write VTU");

    verify_file_exists(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("Name=\"temperature\""),
        "Should have temperature field"
    );
    assert!(
        content.contains("NumberOfComponents=\"1\""),
        "Temperature should be scalar"
    );
    assert!(
        content.contains("Name=\"velocity\""),
        "Should have velocity field"
    );
    assert!(
        content.contains("NumberOfComponents=\"3\""),
        "Velocity should be vector"
    );
}

// ============================================================================
// Golden File Regression Tests
// ============================================================================

fn assert_matches_golden_file(generated_path: &PathBuf, golden_path: &str) {
    let generated = fs::read_to_string(generated_path).expect("Should read generated file");
    let golden = fs::read_to_string(golden_path).expect("Should read golden reference file");

    assert_eq!(
        generated, golden,
        "\n\nGenerated file does not match golden reference!\n\
         Golden file: {}\n\
         Generated file: {:?}\n",
        golden_path, generated_path,
    );
}

#[test]
fn test_golden_file_simple_scalar() {
    let guard = TestFileGuard::new("golden_simple_scalar.vtu");

    let mut field = ScalarField::new();
    field.push(100.0);
    field.push(200.0);

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];

    let temp_array = scalar_field_to_vtk_array("temperature", &field);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[temp_array],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write VTU");

    assert_matches_golden_file(guard.path(), "tests/fixtures/golden_vtu/simple_scalar_ascii.vtu");
}

#[test]
fn test_golden_file_simple_vector() {
    let guard = TestFileGuard::new("golden_simple_vector.vtu");

    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 0.0, 0.0));
    field.push(Vector3::new(0.0, 1.0, 0.0));

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];

    let vel_array = vector3_field_to_vtk_array("velocity", &field);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[vel_array],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write VTU");

    assert_matches_golden_file(
        guard.path(),
        "tests/fixtures/golden_vtu/simple_vector_ascii.vtu",
    );
}
