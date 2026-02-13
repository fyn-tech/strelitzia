//! Example: Export time series data to ParaView format.
//!
//! Demonstrates writing multiple VTU files for different time steps
//! and creating a PVD collection file for time series visualization.

use strelitzia::multiarray::Vector3;
use strelitzia::fields::{ScalarField, Vector3Field};
use strelitzia::visualiser::{
    scalar_field_to_vtk_array, vector3_field_to_vtk_array, write_pvd, write_vtu, Encoding,
};

fn main() -> std::io::Result<()> {
    println!("=== Time Series Export Example ===\n");
    
    // Simulation parameters
    let num_steps = 5;
    let dt = 0.1;  // Time step
    let output_dir = std::path::PathBuf::from(
        std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()),
    )
    .join("examples")
    .join("time_series_output");
    std::fs::create_dir_all(&output_dir)?;
    
    let mut pvd_entries = Vec::new();
    
    // Simulate time evolution
    for step in 0..num_steps {
        let time = step as f64 * dt;
        let vtu_filename = output_dir.join(format!("step_{:04}.vtu", step));
        
        println!("Step {}: t = {:.2}", step, time);
        
        // Create mesh points (simple 2D grid that moves over time)
        let mut points = Vec::new();
        for i in 0..3 {
            for j in 0..3 {
                let x = i as f64 * 0.5 + time * 0.1;  // Move in x direction
                let y = j as f64 * 0.5;
                let z = 0.0;
                points.push([x, y, z]);
            }
        }
        
        // Create scalar field (temperature that increases with time)
        let mut temperature = ScalarField::new();
        for i in 0..points.len() {
            let base_temp = 20.0;
            let temp = base_temp + time * 10.0 + i as f64 * 2.0;
            temperature.push(temp);
        }
        
        // Create vector field (velocity that rotates)
        let mut velocity = Vector3Field::new();
        for (idx, _point) in points.iter().enumerate() {
            let angle = time + idx as f64 * 0.1;
            let vx = angle.cos() * 0.5;
            let vy = angle.sin() * 0.5;
            let vz = 0.0;
            velocity.push(Vector3::new(vx, vy, vz));
        }
        
        // Convert fields to VTK arrays
        let temp_array = scalar_field_to_vtk_array("temperature", &temperature);
        let vel_array = vector3_field_to_vtk_array("velocity", &velocity);
        
        // Write VTU file
        write_vtu::<_, 3>(
            &vtu_filename,
            &points,
            None,  // Point cloud (auto VTK_VERTEX cells)
            None,
            &[temp_array, vel_array],
            &[],
            Encoding::Ascii,
        )?;
        
        println!("  ✓ Wrote {}", vtu_filename.display());
        
        // Add to PVD collection (use relative path from PVD location)
        let relative_path = format!("step_{:04}.vtu", step);
        pvd_entries.push((time, relative_path));
    }
    
    // Write PVD collection file
    let pvd_path = output_dir.join("simulation.pvd");
    write_pvd(&pvd_path, &pvd_entries)?;
    println!("\n✓ Wrote {}", pvd_path.display());
    
    println!("\n=== Export Complete ===");
    println!("\nTo visualize in ParaView:");
    println!("  1. Open ParaView");
    println!("  2. File -> Open -> Select '{}'", pvd_path.display());
    println!("  3. Click 'Apply'");
    println!("  4. Use the time slider to animate through time steps");
    println!("  5. Color by 'temperature' or show 'velocity' vectors");
    
    Ok(())
}
