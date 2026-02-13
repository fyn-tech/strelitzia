//! Demonstration of ParaView VTK writer functionality.

use strelitzia::visualiser::{CellType, Encoding, FieldArray, write_vtu};

fn output_dir() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(
        std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()),
    )
    .join("examples")
    .join("paraview_demo");
    std::fs::create_dir_all(&dir).expect("failed to create output directory");
    dir
}

fn main() -> std::io::Result<()> {
    let dir = output_dir();
    println!("=== ParaView VTK Writer Demo ===\n");

    // Example 1: Point cloud (auto-infers VTK_VERTEX cells)
    println!("Example 1: Writing point cloud...");
    let points_2d: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

    let path = dir.join("point_cloud_2d.vtu");
    write_vtu::<_, 2>(
        &path,
        &points_2d,
        None,
        None,
        &[],
        &[],
        Encoding::Ascii,
    )?;
    println!("  ✓ Wrote {} (2D point cloud, ASCII)\n", path.display());

    // Example 2: Triangle mesh in 3D
    println!("Example 2: Writing triangle mesh...");
    let points_3d: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [0.5, 0.5, 1.0],
    ];

    let connectivity = vec![
        vec![0, 1, 2], // Triangle 1
        vec![0, 1, 3], // Triangle 2
        vec![1, 2, 3], // Triangle 3
        vec![0, 2, 3], // Triangle 4
    ];

    let cell_types = vec![
        CellType::Triangle,
        CellType::Triangle,
        CellType::Triangle,
        CellType::Triangle,
    ];

    let path = dir.join("tetrahedron.vtu");
    write_vtu::<_, 3>(
        &path,
        &points_3d,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Base64,
    )?;
    println!("  ✓ Wrote {} (3D triangle mesh, Base64)\n", path.display());

    // Example 3: Mesh with field data
    println!("Example 3: Writing mesh with field data...");
    let cube_points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ];

    let cube_connectivity = vec![
        vec![0, 1, 2, 3], // Bottom face
        vec![4, 5, 6, 7], // Top face
        vec![0, 1, 5, 4], // Front face
        vec![2, 3, 7, 6], // Back face
        vec![0, 3, 7, 4], // Left face
        vec![1, 2, 6, 5], // Right face
    ];

    let cube_cell_types = vec![CellType::Quad; 6];

    // Point field: temperature (scalar)
    let temperature: Vec<f64> = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let temp_field = FieldArray::from_slice("temperature", &temperature, 1);

    // Point field: velocity (vector)
    let velocity: Vec<[f64; 3]> = vec![
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
        [1.0, 1.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 0.0, 0.0],
    ];
    let vel_field = FieldArray::from_slice("velocity", &velocity, 3);

    // Cell field: pressure (scalar)
    let pressure: Vec<f64> = vec![100.0, 200.0, 150.0, 175.0, 125.0, 225.0];
    let pressure_field = FieldArray::from_slice("pressure", &pressure, 1);

    let path = dir.join("cube_with_fields.vtu");
    write_vtu::<_, 3>(
        &path,
        &cube_points,
        Some(&cube_connectivity),
        Some(&cube_cell_types),
        &[temp_field, vel_field],
        &[pressure_field],
        Encoding::Base64,
    )?;
    println!("  ✓ Wrote {} (cube with temperature, velocity, pressure)\n", path.display());

    // Example 4: Mixed cell types
    println!("Example 4: Writing mesh with mixed cell types...");
    let mixed_points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
        [2.0, 0.0, 0.0],
        [2.0, 1.0, 0.0],
    ];

    let mixed_connectivity = vec![
        vec![0, 1, 2],    // Triangle
        vec![1, 3, 4, 2], // Quad
    ];

    let mixed_cell_types = vec![CellType::Triangle, CellType::Quad];

    let path = dir.join("mixed_cells.vtu");
    write_vtu::<_, 3>(
        &path,
        &mixed_points,
        Some(&mixed_connectivity),
        Some(&mixed_cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )?;
    println!("  ✓ Wrote {} (triangle + quad, ASCII)\n", path.display());

    println!("=== All files written successfully! ===");
    println!("\nOpen these files in ParaView to visualize.");

    Ok(())
}
