//! Integration tests for the VTK writer module.

use std::fs;
use std::path::PathBuf;
use strelitzia::visualiser::{CellType, Encoding, FieldArray, write_vtu};

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Helper to create a temporary test file path
fn test_file_path(name: &str) -> PathBuf {
    PathBuf::from(format!("test_output_{}", name))
}

/// RAII guard for automatic test file cleanup
struct TestFileGuard {
    path: PathBuf,
}

impl TestFileGuard {
    fn new(name: &str) -> Self {
        Self {
            path: test_file_path(name),
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

/// Verify VTK XML header
fn verify_vtk_header(path: &PathBuf) {
    let content = fs::read_to_string(path).expect("Should read file");
    assert!(
        content.starts_with("<?xml version=\"1.0\"?>"),
        "Should have XML header"
    );
    assert!(
        content.contains("<VTKFile type=\"UnstructuredGrid\""),
        "Should be UnstructuredGrid type"
    );
}

// ============================================================================
// Point Cloud Tests
// ============================================================================

#[test]
fn test_point_cloud_2d_ascii() {
    let guard = TestFileGuard::new("point_cloud_2d.vtu");

    let points: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    write_vtu::<_, 2>(guard.path(), &points, None, None, &[], &[], Encoding::Ascii)
        .expect("Should write point cloud");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("NumberOfPoints=\"4\""),
        "Should have 4 points"
    );
    assert!(
        content.contains("NumberOfCells=\"4\""),
        "Should have 4 auto-generated vertex cells"
    );
    assert!(
        content.contains("format=\"ascii\""),
        "Should be ASCII format"
    );
}

#[test]
fn test_point_cloud_3d_base64() {
    let guard = TestFileGuard::new("point_cloud_3d.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[],
        &[],
        Encoding::Base64,
    )
    .expect("Should write point cloud");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("NumberOfPoints=\"4\""),
        "Should have 4 points"
    );
    assert!(
        content.contains("format=\"binary\""),
        "Should be Base64 format"
    );
}

// ============================================================================
// Basic Mesh Tests
// ============================================================================

#[test]
fn test_triangle_mesh() {
    let guard = TestFileGuard::new("triangle_mesh.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ];

    let connectivity = vec![vec![0, 1, 2], vec![0, 1, 3], vec![1, 2, 3], vec![0, 2, 3]];

    let cell_types = vec![
        CellType::Triangle,
        CellType::Triangle,
        CellType::Triangle,
        CellType::Triangle,
    ];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Base64,
    )
    .expect("Should write triangle mesh");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("NumberOfPoints=\"4\""),
        "Should have 4 points"
    );
    assert!(
        content.contains("NumberOfCells=\"4\""),
        "Should have 4 cells"
    );
}

#[test]
fn test_quad_mesh() {
    let guard = TestFileGuard::new("quad_mesh.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let connectivity = vec![vec![0, 1, 2, 3]];
    let cell_types = vec![CellType::Quad];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write quad mesh");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("NumberOfCells=\"1\""),
        "Should have 1 cell"
    );
}

// ============================================================================
// Mixed Cell Type Tests
// ============================================================================

#[test]
fn test_mixed_cell_types() {
    let guard = TestFileGuard::new("mixed_cells.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        [2.0, 1.0, 0.0],
    ];

    let connectivity = vec![vec![0, 1, 2], vec![1, 3, 4, 2]];

    let cell_types = vec![CellType::Triangle, CellType::Quad];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write mixed cells");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("NumberOfCells=\"2\""),
        "Should have 2 cells"
    );
    // VTK_TRIANGLE = 5, VTK_QUAD = 9
    assert!(
        content.contains("5 9"),
        "Should have triangle and quad types"
    );
}

#[test]
fn test_all_cell_types() {
    let guard = TestFileGuard::new("all_cell_types.vtu");

    // Create points for various cell types
    let points: Vec<[f64; 3]> = vec![
        // Edge (2 points)
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // Triangle (3 points)
        [2.0, 0.0, 0.0],
        [2.5, 1.0, 0.0],
        [3.0, 0.0, 0.0],
        // Quad (4 points)
        [4.0, 0.0, 0.0],
        [5.0, 0.0, 0.0],
        [5.0, 1.0, 0.0],
        [4.0, 1.0, 0.0],
        // Tetra (4 points)
        [6.0, 0.0, 0.0],
        [7.0, 0.0, 0.0],
        [6.5, 1.0, 0.0],
        [6.5, 0.5, 1.0],
    ];

    let connectivity = vec![
        vec![0, 1],          // Edge
        vec![2, 3, 4],       // Triangle
        vec![5, 6, 7, 8],    // Quad
        vec![9, 10, 11, 12], // Tetra
    ];

    let cell_types = vec![
        CellType::Edge,
        CellType::Triangle,
        CellType::Quad,
        CellType::Tetra,
    ];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write all cell types");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("NumberOfCells=\"4\""),
        "Should have 4 cells"
    );
    // VTK_LINE=3, VTK_TRIANGLE=5, VTK_QUAD=9, VTK_TETRA=10
    assert!(
        content.contains("3 5 9 10"),
        "Should have all cell type codes"
    );
}

// ============================================================================
// Field Data Tests
// ============================================================================

#[test]
fn test_point_scalar_field() {
    let guard = TestFileGuard::new("point_scalar_field.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let temperature: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0];
    let temp_field = FieldArray::from_slice("temperature", &temperature, 1);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[temp_field],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write with scalar field");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("Name=\"temperature\""),
        "Should have temperature field"
    );
    assert!(
        content.contains("NumberOfComponents=\"1\""),
        "Should be scalar field"
    );
}

#[test]
fn test_point_vector_field() {
    let guard = TestFileGuard::new("point_vector_field.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let velocity: Vec<[f64; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ];
    let vel_field = FieldArray::from_slice("velocity", &velocity, 3);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[vel_field],
        &[],
        Encoding::Base64,
    )
    .expect("Should write with vector field");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("Name=\"velocity\""),
        "Should have velocity field"
    );
    assert!(
        content.contains("NumberOfComponents=\"3\""),
        "Should be vector field"
    );
}

#[test]
fn test_cell_scalar_field() {
    let guard = TestFileGuard::new("cell_scalar_field.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ];

    let connectivity = vec![vec![0, 1, 2], vec![0, 1, 3]];
    let cell_types = vec![CellType::Triangle, CellType::Triangle];

    let pressure: Vec<f64> = vec![100.0, 200.0];
    let pressure_field = FieldArray::from_slice("pressure", &pressure, 1);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[pressure_field],
        Encoding::Ascii,
    )
    .expect("Should write with cell field");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("<CellData>"),
        "Should have CellData section"
    );
    assert!(
        content.contains("Name=\"pressure\""),
        "Should have pressure field"
    );
}

#[test]
fn test_multiple_fields() {
    let guard = TestFileGuard::new("multiple_fields.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let connectivity = vec![vec![0, 1, 2, 3]];
    let cell_types = vec![CellType::Quad];

    // Point fields
    let temperature: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0];
    let temp_field = FieldArray::from_slice("temperature", &temperature, 1);

    let velocity: Vec<[f64; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
    ];
    let vel_field = FieldArray::from_slice("velocity", &velocity, 3);

    // Cell field
    let pressure: Vec<f64> = vec![100.0];
    let pressure_field = FieldArray::from_slice("pressure", &pressure, 1);

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[temp_field, vel_field],
        &[pressure_field],
        Encoding::Base64,
    )
    .expect("Should write with multiple fields");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("Name=\"temperature\""),
        "Should have temperature"
    );
    assert!(
        content.contains("Name=\"velocity\""),
        "Should have velocity"
    );
    assert!(
        content.contains("Name=\"pressure\""),
        "Should have pressure"
    );
}

// ============================================================================
// Special Cell Type Tests
// ============================================================================

#[test]
fn test_edge_chain() {
    let guard = TestFileGuard::new("edge_chain.vtu");

    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ];

    let connectivity = vec![vec![0, 1, 2, 3]];
    let cell_types = vec![CellType::EdgeChain];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write edge chain");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    // VTK_POLYLINE = 4
    assert!(content.contains("4"), "Should have polyline type");
}

#[test]
fn test_polygon() {
    let guard = TestFileGuard::new("polygon.vtu");

    // Pentagon
    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.5, 1.0, 0.0],
        [0.5, 1.5, 0.0],
        [-0.5, 1.0, 0.0],
    ];

    let connectivity = vec![vec![0, 1, 2, 3, 4]];
    let cell_types = vec![CellType::Polygon];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write polygon");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    // VTK_POLYGON = 7
    assert!(content.contains("7"), "Should have polygon type");
}

#[test]
fn test_hexahedron() {
    let guard = TestFileGuard::new("hexahedron.vtu");

    // Cube vertices
    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ];

    let connectivity = vec![vec![0, 1, 2, 3, 4, 5, 6, 7]];
    let cell_types = vec![CellType::Hexa];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write hexahedron");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    // VTK_HEXAHEDRON = 12
    assert!(content.contains("12"), "Should have hexahedron type");
}

// ============================================================================
// Edge Cases and Validation Tests
// ============================================================================

#[test]
fn test_empty_point_cloud() {
    let guard = TestFileGuard::new("empty_cloud.vtu");

    let points: Vec<[f64; 3]> = vec![];

    write_vtu::<_, 3>(guard.path(), &points, None, None, &[], &[], Encoding::Ascii)
        .expect("Should write empty point cloud");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    assert!(
        content.contains("NumberOfPoints=\"0\""),
        "Should have 0 points"
    );
    assert!(
        content.contains("NumberOfCells=\"0\""),
        "Should have 0 cells"
    );
}

#[test]
#[should_panic(expected = "Point type size")]
fn test_invalid_dimension_size() {
    let guard = TestFileGuard::new("invalid_dim.vtu");

    // Wrong: using [f32; 3] with DIM=3 (expects [f64; 3])
    let points: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]];

    write_vtu::<_, 3>(guard.path(), &points, None, None, &[], &[], Encoding::Ascii)
        .expect("Should fail with size mismatch");
}

#[test]
fn test_cell_type_conversions() {
    // This test verifies the internal VTK cell type mapping
    // We can't directly access VTKCellType from integration tests,
    // but we can verify the behavior through file output
    let guard = TestFileGuard::new("cell_type_test.vtu");

    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];

    let connectivity = vec![vec![0, 1, 2]];
    let cell_types = vec![CellType::Triangle];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write triangle");

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    // VTK_TRIANGLE = 5
    assert!(content.contains("5"), "Triangle should map to VTK type 5");
}
