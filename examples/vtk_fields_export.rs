//! Example: Export fields to VTK format for ParaView visualization.

use strelitzia::multiarray::Vector3;
use strelitzia::fields::*;
use strelitzia::visualiser::*;

fn output_dir() -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(
        std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()),
    )
    .join("examples")
    .join("vtk_fields");
    std::fs::create_dir_all(&dir).expect("failed to create output directory");
    dir
}

fn main() -> std::io::Result<()> {
    let dir = output_dir();
    println!("Creating sample VTU files for ParaView verification...\n");
    
    // Example 1: Scalar field (temperature)
    println!("1. Creating scalar field (temperature)...");
    let mut temperature = ScalarField::new();
    temperature.push(25.0);  // Point 0: 25°C
    temperature.push(30.0);  // Point 1: 30°C
    temperature.push(28.0);  // Point 2: 28°C
    
    let points: Vec<[f64; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.5, 1.0, 0.0],
    ];
    
    let temp_array = scalar_field_to_vtk_array("temperature", &temperature);
    
    let path = dir.join("sample_scalar_field.vtu");
    write_vtu::<_, 3>(
        &path,
        &points,
        None,  // Point cloud (auto VTK_VERTEX cells)
        None,
        &[temp_array],
        &[],
        Encoding::Ascii,
    )?;
    println!("   ✓ Created {}", path.display());
    
    // Example 2: Vector field (velocity)
    println!("2. Creating vector field (velocity)...");
    let mut velocity = Vector3Field::new();
    velocity.push(Vector3::new(1.0, 0.0, 0.0));  // Point 0: velocity in +x
    velocity.push(Vector3::new(0.0, 1.0, 0.0));  // Point 1: velocity in +y
    velocity.push(Vector3::new(0.5, 0.5, 0.0));  // Point 2: velocity diagonal
    
    let vel_array = vector3_field_to_vtk_array("velocity", &velocity);
    
    let path = dir.join("sample_vector3_field.vtu");
    write_vtu::<_, 3>(
        &path,
        &points,
        None,
        None,
        &[vel_array],
        &[],
        Encoding::Base64,
    )?;
    println!("   ✓ Created {}", path.display());
    
    // Example 3: Multiple fields
    println!("3. Creating combined field (temperature + velocity)...");
    let combined_temp = scalar_field_to_vtk_array("temperature", &temperature);
    let combined_vel = vector3_field_to_vtk_array("velocity", &velocity);
    
    let path = dir.join("sample_combined_fields.vtu");
    write_vtu::<_, 3>(
        &path,
        &points,
        None,
        None,
        &[combined_temp, combined_vel],
        &[],
        Encoding::Base64,
    )?;
    println!("   ✓ Created {}", path.display());
    
    println!("\n✅ All sample files created successfully!");
    println!("\n📊 Manual verification steps:");
    println!("   1. Open each .vtu file in ParaView");
    println!("   2. For scalar fields: Apply 'Point Gaussian' representation");
    println!("   3. For vector fields: Apply 'Glyph' filter with 'Arrow' glyph type");
    println!("   4. Verify field names appear in Properties panel");
    println!("   5. Check that values match expected data");
    
    Ok(())
}
