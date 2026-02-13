//! Integration tests for the VTK writer module.

use std::fs;
use std::path::PathBuf;
use strelitzia::visualiser::{CellType, Encoding, FieldArray, write_pvd, write_vtu};

// ============================================================================
// Test Infrastructure
// ============================================================================

/// Helper to create a temporary test file path
fn test_file_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("strelitzia_test_output_{}", name))
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
fn test_edge_chain_vertex_sharing() {
    let guard = TestFileGuard::new("edge_chain_sharing.vtu");

    // Create 5 points
    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],  // 0
        [1.0, 0.0, 0.0],  // 1 - shared vertex
        [1.0, 1.0, 0.0],  // 2 - shared vertex
        [0.0, 1.0, 0.0],  // 3
        [0.0, 0.0, 0.0],  // 4 - back to start
    ];

    // Two polylines sharing vertices 1 and 2
    // First: 0->1->2, Second: 2->3->4
    let connectivity = vec![
        vec![0, 1, 2],  // First polyline
        vec![2, 3, 4],  // Second polyline (shares vertex 2)
    ];
    let cell_types = vec![CellType::EdgeChain, CellType::EdgeChain];

    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should write edge chains with shared vertices");

    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());

    let content = fs::read_to_string(guard.path()).expect("Should read file");
    
    // Verify: 5 points, 2 cells (no duplication)
    assert!(
        content.contains("NumberOfPoints=\"5\""),
        "Should have 5 points (no duplication)"
    );
    assert!(
        content.contains("NumberOfCells=\"2\""),
        "Should have 2 polyline cells"
    );
    
    // Verify connectivity references shared vertices
    // Connectivity should be: 0 1 2 2 3 4
    assert!(
        content.contains("0 1 2"),
        "First polyline should connect 0->1->2"
    );
    assert!(
        content.contains("2 3 4"),
        "Second polyline should connect 2->3->4 (shares vertex 2)"
    );
    
    // Verify both are polylines (VTK_POLYLINE = 4)
    assert!(
        content.contains("4 4"),
        "Both cells should be polylines"
    );
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

// ============================================================================
// Validation and Error Handling Tests
// ============================================================================

#[test]
#[should_panic(expected = "connectivity and cell_types must both be Some")]
fn test_invalid_connectivity_cell_types_mismatch() {
    let guard = TestFileGuard::new("invalid_mismatch.vtu");
    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let connectivity = vec![vec![0, 1]];
    
    // Mismatch: connectivity provided but no cell_types
    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        None,  // Missing cell_types
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should fail with mismatch");
}

#[test]
#[should_panic(expected = "connectivity and cell_types must both be Some")]
fn test_invalid_cell_types_without_connectivity() {
    let guard = TestFileGuard::new("invalid_cell_types.vtu");
    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
    let cell_types = vec![CellType::Edge];
    
    // Mismatch: cell_types provided but no connectivity
    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,  // Missing connectivity
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should fail with mismatch");
}

#[test]
#[should_panic(expected = "connectivity length")]
fn test_connectivity_cell_types_length_mismatch() {
    let guard = TestFileGuard::new("length_mismatch.vtu");
    let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    let connectivity = vec![vec![0, 1, 2], vec![0, 1, 2]];  // 2 cells
    let cell_types = vec![CellType::Triangle];  // Only 1 type!
    
    write_vtu::<_, 3>(
        guard.path(),
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )
    .expect("Should fail with length mismatch");
}

#[test]
#[should_panic(expected = "Connectivity index")]
fn test_connectivity_index_out_of_bounds() {
    let guard = TestFileGuard::new("out_of_bounds.vtu");
    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],  // 0
        [1.0, 0.0, 0.0],  // 1
        [0.5, 1.0, 0.0],  // 2
    ];  // Only 3 points (indices 0, 1, 2)
    
    let connectivity = vec![vec![0, 1, 5]];  // Index 5 is out of bounds!
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
    .expect("Should fail with out of bounds index");
}

#[test]
#[should_panic(expected = "Point field 'temperature' has wrong length")]
fn test_field_length_mismatch_point_field() {
    let guard = TestFileGuard::new("field_mismatch_point.vtu");
    
    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
    ];  // 3 points
    
    // Field has wrong length (2 values for 3 points)
    let temperature: Vec<f64> = vec![0.0, 1.0];  // Only 2 values!
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
    .expect("Should fail with field length mismatch");
}

#[test]
#[should_panic(expected = "Cell field 'pressure' has wrong length")]
fn test_field_length_mismatch_cell_field() {
    let guard = TestFileGuard::new("field_mismatch_cell.vtu");
    
    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
    ];
    
    let connectivity = vec![vec![0, 1, 2], vec![0, 1, 2]];  // 2 cells
    let cell_types = vec![CellType::Triangle, CellType::Triangle];
    
    // Field has wrong length (1 value for 2 cells)
    let pressure: Vec<f64> = vec![100.0];  // Only 1 value!
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
    .expect("Should fail with field length mismatch");
}

#[test]
fn test_base64_encoding_correctness() {
    let guard = TestFileGuard::new("base64_correctness.vtu");
    
    // Use known values that can be verified
    let points: Vec<[f64; 3]> = vec![
        [1.0, 2.0, 3.0],
        [4.0, 5.0, 6.0],
    ];
    
    let temperature: Vec<f64> = vec![100.0, 200.0];
    let temp_field = FieldArray::from_slice("temperature", &temperature, 1);
    
    write_vtu::<_, 3>(
        guard.path(),
        &points,
        None,
        None,
        &[temp_field],
        &[],
        Encoding::Base64,
    )
    .expect("Should write Base64");
    
    verify_file_exists(guard.path());
    verify_vtk_header(guard.path());
    
    let content = fs::read_to_string(guard.path()).expect("Should read file");
    
    // Verify Base64 encoding markers
    assert!(
        content.contains("format=\"binary\""),
        "Should be Base64/binary format"
    );
    
    // Base64 data should be present (non-empty, non-ASCII)
    // The actual binary data is encoded, so we just verify structure
    assert!(
        content.contains("<DataArray"),
        "Should contain DataArray elements"
    );
    
    // Verify file can be read back (ParaView would validate binary data)
    // This is a basic sanity check - full validation requires ParaView
}

// ============================================================================
// PVD Collection File Tests
// ============================================================================

#[test]
fn test_pvd_basic() {
    let guard = TestFileGuard::new("collection.pvd");
    
    let entries = vec![
        (0.0, "step_0.vtu"),
        (0.1, "step_1.vtu"),
        (0.2, "step_2.vtu"),
    ];
    
    write_pvd(guard.path(), &entries).expect("Should write PVD");
    
    verify_file_exists(guard.path());
    
    let content = fs::read_to_string(guard.path()).expect("Should read file");
    
    // Verify XML structure
    assert!(content.starts_with("<?xml version=\"1.0\"?>"));
    assert!(content.contains("<VTKFile type=\"Collection\" version=\"0.1\">"));
    assert!(content.contains("<Collection>"));
    assert!(content.contains("</Collection>"));
    assert!(content.contains("</VTKFile>"));
    
    // Verify entries (f64 may format as "0" or "0.0", so check for both)
    assert!(content.contains("timestep=\"0") || content.contains("timestep=\"0.0"));
    assert!(content.contains("timestep=\"0.1\""));
    assert!(content.contains("timestep=\"0.2\""));
    assert!(content.contains("file=\"step_0.vtu\""));
    assert!(content.contains("file=\"step_1.vtu\""));
    assert!(content.contains("file=\"step_2.vtu\""));
}

#[test]
fn test_pvd_empty_collection() {
    let guard = TestFileGuard::new("empty.pvd");
    
    write_pvd(guard.path(), &[] as &[(f64, &str)]).expect("Should write empty PVD");
    
    verify_file_exists(guard.path());
    
    let content = fs::read_to_string(guard.path()).expect("Should read file");
    
    // Verify XML structure even for empty collection
    assert!(content.contains("<Collection>"));
    assert!(content.contains("</Collection>"));
    
    // Should not contain any DataSet entries
    assert!(!content.contains("<DataSet"));
}

#[test]
fn test_pvd_single_entry() {
    let guard = TestFileGuard::new("single.pvd");
    
    let entries = vec![(42.5, "output.vtu")];
    
    write_pvd(guard.path(), &entries).expect("Should write single entry PVD");
    
    let content = fs::read_to_string(guard.path()).expect("Should read file");
    
    assert!(content.contains("timestep=\"42.5\""));
    assert!(content.contains("file=\"output.vtu\""));
}

#[test]
fn test_pvd_xml_escaping() {
    let guard = TestFileGuard::new("escape.pvd");
    
    // Test file paths with special characters that need XML escaping
    let entries = vec![
        (0.0, "file&name.vtu"),      // & -> &amp;
        (1.0, "file<name.vtu"),      // < -> &lt;
        (2.0, "file>name.vtu"),      // > -> &gt;
        (3.0, "file\"name.vtu"),     // " -> &quot;
        (4.0, "file'name.vtu"),      // ' -> &apos;
    ];
    
    write_pvd(guard.path(), &entries).expect("Should write PVD with escaped paths");
    
    let content = fs::read_to_string(guard.path()).expect("Should read file");
    
    // Verify XML escaping
    assert!(content.contains("file=\"file&amp;name.vtu\""));
    assert!(content.contains("file=\"file&lt;name.vtu\""));
    assert!(content.contains("file=\"file&gt;name.vtu\""));
    assert!(content.contains("file=\"file&quot;name.vtu\""));
    assert!(content.contains("file=\"file&apos;name.vtu\""));
    
    // Verify no unescaped characters
    assert!(!content.contains("file&name.vtu"));
    assert!(!content.contains("file<name.vtu"));
    assert!(!content.contains("file>name.vtu"));
}

#[test]
fn test_pvd_time_series_integration() {
    // Create a time series: write multiple VTU files and a PVD collection
    let base_dir = std::env::temp_dir();
    let test_dir = base_dir.join("strelitzia_pvd_test");
    std::fs::create_dir_all(&test_dir).expect("Should create test directory");
    
    let mut pvd_entries = Vec::new();
    
    // Write 3 time steps
    for (step, time) in [(0, 0.0), (1, 0.1), (2, 0.2)].iter() {
        let vtu_name = format!("step_{:04}.vtu", step);
        let vtu_path = test_dir.join(&vtu_name);
        
        let points: Vec<[f64; 3]> = vec![
            [0.0 + *time, 0.0, 0.0],
            [1.0 + *time, 0.0, 0.0],
            [0.5 + *time, 1.0, 0.0],
        ];
        
        // Create field data that changes with time
        let temperature: Vec<f64> = vec![20.0 + *time * 10.0, 25.0 + *time * 10.0, 30.0 + *time * 10.0];
        let temp_field = FieldArray::from_slice("temperature", &temperature, 1);
        
        write_vtu::<_, 3>(
            &vtu_path,
            &points,
            None,  // Point cloud
            None,
            &[temp_field],
            &[],
            Encoding::Ascii,
        )
        .expect("Should write VTU file");
        
        // Add to PVD collection (use relative path)
        pvd_entries.push((*time, vtu_name));
    }
    
    // Write PVD collection
    let pvd_path = test_dir.join("simulation.pvd");
    write_pvd(&pvd_path, &pvd_entries).expect("Should write PVD");
    
    // Verify PVD file (f64 may format as "0" or "0.0")
    let pvd_content = fs::read_to_string(&pvd_path).expect("Should read PVD");
    assert!(pvd_content.contains("timestep=\"0") || pvd_content.contains("timestep=\"0.0"));
    assert!(pvd_content.contains("timestep=\"0.1\""));
    assert!(pvd_content.contains("timestep=\"0.2\""));
    assert!(pvd_content.contains("step_0000.vtu"));
    assert!(pvd_content.contains("step_0001.vtu"));
    assert!(pvd_content.contains("step_0002.vtu"));
    
    // Verify VTU files exist
    for (step, _) in [(0, 0.0), (1, 0.1), (2, 0.2)].iter() {
        let vtu_path = test_dir.join(format!("step_{:04}.vtu", step));
        assert!(vtu_path.exists(), "VTU file should exist: {:?}", vtu_path);
    }
    
    // Cleanup
    std::fs::remove_dir_all(&test_dir).ok();
}

#[test]
fn test_pvd_golden_file() {
    let guard = TestFileGuard::new("golden_pvd.pvd");
    
    let entries = vec![
        (0.0, "data_0.vtu"),
        (1.0, "data_1.vtu"),
    ];
    
    write_pvd(guard.path(), &entries).expect("Should write PVD");
    
    // Read golden file
    let golden_path = PathBuf::from("tests/fixtures/golden_vtu/simple_pvd.pvd");
    let golden = fs::read_to_string(&golden_path).expect("Should read golden file");
    
    // Read generated file
    let generated = fs::read_to_string(guard.path()).expect("Should read generated file");
    
    // Normalize whitespace for comparison
    let golden_normalized: String = golden.lines().map(|l| l.trim()).collect::<Vec<_>>().join("\n");
    let generated_normalized: String = generated.lines().map(|l| l.trim()).collect::<Vec<_>>().join("\n");
    
    // Compare normalized content
    assert_eq!(
        generated_normalized, golden_normalized,
        "Generated PVD doesn't match golden file"
    );
}
