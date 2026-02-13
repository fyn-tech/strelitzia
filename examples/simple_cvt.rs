//! Simple CVT example using the minimal API

use strelitzia::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Simple CVT Example ===\n");

    // The original 10 points
    let points: Vec<Point2D> = vec![
        (0.478554, 0.00869692),
        (0.13928, 0.180603),
        (0.578587, 0.760349),
        (0.903726, 0.975904),
        (0.0980015, 0.981755),
        (0.133721, 0.348832),
        (0.648071, 0.369534),
        (0.230951, 0.558482),
        (0.0307942, 0.459123),
        (0.540745, 0.331184),
    ];

    // Generate without visualization
    println!("Generating Voronoi tessellation (no visualization)...");
    generate_voronoi(&points, None)?;
    println!("✓ Success!\n");

    // Generate with visualization
    let dir = std::path::PathBuf::from(
        std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into()),
    )
    .join("examples");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("simple_cvt.svg");

    println!("Generating Voronoi tessellation with visualization...");
    generate_voronoi(&points, Some(&path.to_string_lossy()))?;
    println!("✓ Success! Saved to {}", path.display());

    Ok(())
}
