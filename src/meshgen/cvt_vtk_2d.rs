//! 2D CVT VTK/ParaView export utilities.
//!
//! Provides functions to export 2D CVT results (Delaunay mesh, Voronoi cells)
//! to VTK format for visualization in ParaView.

use crate::common::Real;
use crate::fields::ScalarField;
use crate::meshgen::cvt::{CvtDomain, CvtState, Domain2D};
use crate::meshgen::mesh::Mesh;
use crate::multiarray::Point2;
use crate::visualiser::{write_pvd, write_vtu, CellType, Encoding, FieldArray};

use std::fs;
use std::io;
use std::path::Path;

/// Write a 2D CVT state as a Delaunay triangulation to VTU format.
///
/// Creates a VTU file containing:
/// - Points: seed positions (2D, padded to 3D for VTK)
/// - Point fields: local density, cell mass
/// - Cells: Delaunay triangles
pub fn write_cvt_2d_delaunay_vtu<F, P>(
    path: P,
    domain: &Domain2D,
    state: &CvtState<Point2>,
    density: F,
) -> io::Result<()>
where
    F: Fn(Point2) -> Real,
    P: AsRef<Path>,
{
    let seeds = state.seeds.as_slice();
    if seeds.is_empty() {
        return Ok(());
    }

    let mesh = Mesh::delaunay(state.seeds.clone());

    let points: Vec<[f64; 3]> = mesh
        .vertices
        .iter()
        .map(|p| [p[0], p[1], 0.0])
        .collect();

    let densities: ScalarField = seeds.iter().map(|&p| density(p)).collect();

    let data = domain.integrate_cells(seeds, &density);
    let masses = data.masses;

    let density_array = FieldArray::from_slice("density", densities.as_slice(), 1);
    let mass_array = FieldArray::from_slice("cell_mass", masses.as_slice(), 1);

    write_vtu::<_, 3>(
        path,
        &points,
        Some(&mesh.cells),
        Some(&mesh.cell_types),
        &[density_array, mass_array],
        &[],
        Encoding::Base64,
    )
}

/// Write 2D Voronoi cell polygons to VTU format.
///
/// Creates a VTU file containing:
/// - Points: Voronoi cell polygon vertices
/// - Cells: polygons (one per seed)
/// - Cell fields: mass, energy per cell
pub fn write_cvt_2d_voronoi_vtu<F, P>(
    path: P,
    domain: &Domain2D,
    state: &CvtState<Point2>,
    density: F,
) -> io::Result<()>
where
    F: Fn(Point2) -> Real,
    P: AsRef<Path>,
{
    let seeds = state.seeds.as_slice();
    if seeds.is_empty() {
        return Ok(());
    }

    let voronoi_cells = domain.voronoi_cells(seeds);
    let data = domain.integrate_cells(seeds, &density);

    let mut points: Vec<[f64; 3]> = Vec::new();
    let mut connectivity: Vec<Vec<usize>> = Vec::new();
    let mut cell_types: Vec<CellType> = Vec::new();
    let mut offset = 0;

    for polygon in &voronoi_cells {
        if polygon.len() < 3 {
            continue;
        }
        let cell_indices: Vec<usize> = (offset..offset + polygon.len()).collect();
        for v in polygon {
            points.push([v[0], v[1], 0.0]);
        }
        connectivity.push(cell_indices);
        cell_types.push(CellType::Polygon);
        offset += polygon.len();
    }

    let mass_array = FieldArray::from_slice("cell_mass", data.masses.as_slice(), 1);
    let densities: ScalarField = seeds.iter().map(|&p| density(p)).collect();
    let density_array = FieldArray::from_slice("seed_density", densities.as_slice(), 1);

    write_vtu::<_, 3>(
        path,
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[mass_array, density_array],
        Encoding::Base64,
    )
}

/// Write 2D CVT Delaunay history as a ParaView time series.
///
/// Creates a directory containing one VTU file per iteration and a PVD
/// collection file for animation.
pub fn write_cvt_2d_history_pvd<F, P>(
    dir: P,
    domain: &Domain2D,
    history: &[CvtState<Point2>],
    density: F,
) -> io::Result<()>
where
    F: Fn(Point2) -> Real,
    P: AsRef<Path>,
{
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    let mut pvd_entries: Vec<(f64, String)> = Vec::new();

    for state in history {
        let filename = format!("cvt2d_{:04}.vtu", state.iteration);
        let filepath = dir.join(&filename);

        write_cvt_2d_delaunay_vtu(&filepath, domain, state, &density)?;

        pvd_entries.push((state.iteration as f64, filename));
    }

    let pvd_path = dir.join("cvt2d.pvd");
    let entries_ref: Vec<(f64, &str)> = pvd_entries
        .iter()
        .map(|(t, f)| (*t, f.as_str()))
        .collect();

    write_pvd(&pvd_path, &entries_ref)?;

    Ok(())
}

/// Write 2D CVT Voronoi cell history as a ParaView time series.
///
/// Each frame exports the clipped Voronoi cells as polygon VTU files,
/// with cell mass and seed density as cell data fields.
pub fn write_cvt_2d_voronoi_history_pvd<F, P>(
    dir: P,
    domain: &Domain2D,
    history: &[CvtState<Point2>],
    density: F,
) -> io::Result<()>
where
    F: Fn(Point2) -> Real,
    P: AsRef<Path>,
{
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    let mut pvd_entries: Vec<(f64, String)> = Vec::new();

    for state in history {
        let filename = format!("voronoi_{:04}.vtu", state.iteration);
        let filepath = dir.join(&filename);

        write_cvt_2d_voronoi_vtu(&filepath, domain, state, &density)?;

        pvd_entries.push((state.iteration as f64, filename));
    }

    let pvd_path = dir.join("voronoi.pvd");
    let entries_ref: Vec<(f64, &str)> = pvd_entries
        .iter()
        .map(|(t, f)| (*t, f.as_str()))
        .collect();

    write_pvd(&pvd_path, &entries_ref)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meshgen::cvt::{lloyd_iter, Domain2D};
    use std::fs;

    #[test]
    fn write_2d_delaunay_vtu_creates_file() {
        let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
        let seeds = domain.uniform_seeds(10);
        let state: CvtState<Point2> = lloyd_iter(domain.clone(), seeds, |_| 1.0)
            .nth(5)
            .unwrap();

        let path = "/tmp/strelitzia_test_cvt2d_delaunay.vtu";
        let result = write_cvt_2d_delaunay_vtu(path, &domain, &state, |_| 1.0);
        assert!(result.is_ok(), "VTU write failed: {:?}", result.err());
        assert!(fs::metadata(path).is_ok());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn write_2d_voronoi_vtu_creates_file() {
        let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
        let seeds = domain.uniform_seeds(10);
        let state: CvtState<Point2> = lloyd_iter(domain.clone(), seeds, |_| 1.0)
            .nth(5)
            .unwrap();

        let path = "/tmp/strelitzia_test_cvt2d_voronoi.vtu";
        let result = write_cvt_2d_voronoi_vtu(path, &domain, &state, |_| 1.0);
        assert!(result.is_ok(), "VTU write failed: {:?}", result.err());
        assert!(fs::metadata(path).is_ok());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn write_2d_history_pvd_creates_files() {
        let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
        let seeds = domain.uniform_seeds(8);
        let history: Vec<_> = lloyd_iter(domain.clone(), seeds, |_| 1.0)
            .take(3)
            .collect();

        let dir = "/tmp/strelitzia_test_cvt2d_history";
        let result = write_cvt_2d_history_pvd(dir, &domain, &history, |_| 1.0);
        assert!(result.is_ok());
        assert!(fs::metadata(format!("{}/cvt2d.pvd", dir)).is_ok());
        assert!(fs::metadata(format!("{}/cvt2d_0000.vtu", dir)).is_ok());

        let _ = fs::remove_dir_all(dir);
    }
}
