//! Demonstration of ParaView VTK writer functionality.

use strelitzia::visualiser::{write_vtu, CellType, Encoding, FieldArray};

fn main() -> std::io::Result<()> {
    println!("=== ParaView VTK Writer Demo ===\n");

    // Example 1: Point cloud (auto-infers VTK_VERTEX cells)
    println!("Example 1: Writing point cloud...");
    let points_2d: Vec<[f64; 2]> = vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ];

    write_vtu::<_, 2>(
        "point_cloud_2d.vtu",
        &points_2d,
        None,
        None,
        &[],
        &[],
        Encoding::Ascii,
    )?;
    println!("  ✓ Wrote point_cloud_2d.vtu (2D point cloud, ASCII)\n");

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

    write_vtu::<_, 3>(
        "tetrahedron.vtu",
        &points_3d,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[],
        Encoding::Base64,
    )?;
    println!("  ✓ Wrote tetrahedron.vtu (3D triangle mesh, Base64)\n");

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

    write_vtu::<_, 3>(
        "cube_with_fields.vtu",
        &cube_points,
        Some(&cube_connectivity),
        Some(&cube_cell_types),
        &[temp_field, vel_field],
        &[pressure_field],
        Encoding::Base64,
    )?;
    println!("  ✓ Wrote cube_with_fields.vtu (cube with temperature, velocity, pressure)\n");

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

    write_vtu::<_, 3>(
        "mixed_cells.vtu",
        &mixed_points,
        Some(&mixed_connectivity),
        Some(&mixed_cell_types),
        &[],
        &[],
        Encoding::Ascii,
    )?;
    println!("  ✓ Wrote mixed_cells.vtu (triangle + quad, ASCII)\n");

    println!("=== All files written successfully! ===");
    println!("\nOpen these files in ParaView to visualize:");
    println!("  - point_cloud_2d.vtu");
    println!("  - tetrahedron.vtu");
    println!("  - cube_with_fields.vtu");
    println!("  - mixed_cells.vtu");

    Ok(())
}

