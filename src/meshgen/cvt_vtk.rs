//! CVT VTK/ParaView export utilities.
//!
//! Provides functions to export CVT results to VTK format for visualization
//! in ParaView. Supports single snapshots and time series animation.
//!
//! # Example
//!
//! ```ignore
//! use strelitzia::meshgen::cvt::*;
//! use strelitzia::meshgen::cvt_vtk::*;
//!
//! let domain = Domain1D::new(0.0, 1.0);
//! let density = |x: Real| 1.0;
//! let seeds = uniform_seeds(10, &domain);
//! let history: Vec<_> = lloyd_iter(domain.clone(), seeds, density).take(50).collect();
//!
//! // Export final state
//! write_cvt_state_vtu("cvt_final.vtu", &domain, history.last().unwrap(), &density).unwrap();
//!
//! // Export time series for animation
//! write_cvt_history_pvd("cvt_output", &domain, &history, &density).unwrap();
//! ```

use crate::common::Real;
use crate::fields::ScalarField;
use crate::meshgen::cvt::{CvtState, Domain1D};
use crate::visualiser::{write_pvd, write_vtu, CellType, Encoding};

use std::fs;
use std::io;
use std::path::Path;

// ============================================================================
// Single state export
// ============================================================================

/// Write a single CVT state to VTK UnstructuredGrid format.
///
/// Creates a VTU file containing:
/// - Points: seed positions (1D coordinates, padded to 3D for VTK)
/// - Point fields: cell size, local density, cell energy
/// - Cells: Edge segments representing Voronoi cell boundaries
///
/// # Arguments
/// * `path` - Output .vtu file path
/// * `domain` - The 1D interval domain
/// * `state` - CVT state to export
/// * `density` - Density function
///
/// # Returns
/// `Ok(())` on success, or an IO error.
pub fn write_cvt_state_vtu<F, P>(
    path: P,
    domain: &Domain1D,
    state: &CvtState<Real>,
    density: F,
) -> io::Result<()>
where
    F: Fn(Real) -> Real,
    P: AsRef<Path>,
{
    let seeds = state.seeds.as_slice();
    let n = seeds.len();

    if n == 0 {
        return Ok(());
    }

    let cells = domain.voronoi_cells(seeds);

    let points: Vec<[f64; 3]> = seeds.iter().map(|&x| [x, 0.0, 0.0]).collect();

    let cell_sizes: ScalarField = cells.iter().map(|(l, r)| r - l).collect();

    let densities: ScalarField = seeds.iter().map(|&x| density(x)).collect();

    let energies: ScalarField = cells
        .iter()
        .zip(seeds.iter())
        .map(|(cell, seed)| domain.cell_energy(cell, seed, &density))
        .collect();

    use crate::visualiser::FieldArray;
    let cell_size_array = FieldArray::from_slice("cell_size", cell_sizes.as_slice(), 1);
    let density_array = FieldArray::from_slice("density", densities.as_slice(), 1);
    let energy_array = FieldArray::from_slice("cell_energy", energies.as_slice(), 1);

    let point_fields = vec![cell_size_array, density_array, energy_array];

    write_vtu::<_, 3>(
        path,
        &points,
        None,
        None,
        &point_fields,
        &[],
        Encoding::Base64,
    )
}

/// Write a single CVT state with Voronoi cell edges.
///
/// Similar to `write_cvt_state_vtu`, but includes explicit edge cells
/// showing the Voronoi cell boundaries.
///
/// # Arguments
/// * `path` - Output .vtu file path
/// * `domain` - The 1D interval domain
/// * `state` - CVT state to export
/// * `density` - Density function
///
/// # Returns
/// `Ok(())` on success, or an IO error.
pub fn write_cvt_state_with_edges_vtu<F, P>(
    path: P,
    domain: &Domain1D,
    state: &CvtState<Real>,
    density: F,
) -> io::Result<()>
where
    F: Fn(Real) -> Real,
    P: AsRef<Path>,
{
    let seeds = state.seeds.as_slice();
    let n = seeds.len();

    if n == 0 {
        return Ok(());
    }

    let cells = domain.voronoi_cells(seeds);

    let mut points: Vec<[f64; 3]> = Vec::with_capacity(2 * n);
    for (left, right) in &cells {
        points.push([*left, 0.0, 0.0]);
        points.push([*right, 0.0, 0.0]);
    }

    let connectivity: Vec<Vec<usize>> = (0..n).map(|i| vec![2 * i, 2 * i + 1]).collect();
    let cell_types: Vec<CellType> = vec![CellType::Edge; n];

    let cell_sizes: ScalarField = cells.iter().map(|(l, r)| r - l).collect();
    let densities: ScalarField = seeds.iter().map(|&x| density(x)).collect();

    use crate::visualiser::FieldArray;
    let cell_size_array = FieldArray::from_slice("cell_size", cell_sizes.as_slice(), 1);
    let density_array = FieldArray::from_slice("density", densities.as_slice(), 1);

    write_vtu::<_, 3>(
        path,
        &points,
        Some(&connectivity),
        Some(&cell_types),
        &[],
        &[cell_size_array, density_array],
        Encoding::Base64,
    )
}

// ============================================================================
// Time series export
// ============================================================================

/// Write CVT optimization history as a ParaView time series.
///
/// Creates a directory containing:
/// - One .vtu file per iteration (or sampled subset)
/// - A .pvd collection file for animation in ParaView
///
/// # Arguments
/// * `dir` - Output directory path (will be created if it doesn't exist)
/// * `domain` - The 1D interval domain
/// * `history` - Vector of CVT states from optimization
/// * `density` - Density function
///
/// # Returns
/// `Ok(())` on success, or an IO error.
pub fn write_cvt_history_pvd<F, P>(
    dir: P,
    domain: &Domain1D,
    history: &[CvtState<Real>],
    density: F,
) -> io::Result<()>
where
    F: Fn(Real) -> Real,
    P: AsRef<Path>,
{
    let dir = dir.as_ref();

    fs::create_dir_all(dir)?;

    let mut pvd_entries: Vec<(f64, String)> = Vec::new();

    for state in history {
        let filename = format!("cvt_{:04}.vtu", state.iteration);
        let filepath = dir.join(&filename);

        write_cvt_state_vtu(&filepath, domain, state, &density)?;

        pvd_entries.push((state.iteration as f64, filename));
    }

    let pvd_path = dir.join("cvt.pvd");
    let entries_ref: Vec<(f64, &str)> = pvd_entries
        .iter()
        .map(|(t, f)| (*t, f.as_str()))
        .collect();

    write_pvd(&pvd_path, &entries_ref)?;

    Ok(())
}

/// Write CVT history with sampling to reduce file count.
///
/// Similar to `write_cvt_history_pvd`, but only exports every Nth frame.
///
/// # Arguments
/// * `dir` - Output directory path
/// * `domain` - The 1D interval domain
/// * `history` - Vector of CVT states
/// * `density` - Density function
/// * `sample_rate` - Export every Nth state (1 = export all)
///
/// # Returns
/// `Ok(())` on success, or an IO error.
pub fn write_cvt_history_sampled_pvd<F, P>(
    dir: P,
    domain: &Domain1D,
    history: &[CvtState<Real>],
    density: F,
    sample_rate: usize,
) -> io::Result<()>
where
    F: Fn(Real) -> Real,
    P: AsRef<Path>,
{
    let dir = dir.as_ref();
    let sample_rate = sample_rate.max(1);

    fs::create_dir_all(dir)?;

    let mut pvd_entries: Vec<(f64, String)> = Vec::new();

    for (idx, state) in history.iter().enumerate() {
        if idx % sample_rate != 0 && idx != history.len() - 1 {
            continue;
        }

        let filename = format!("cvt_{:04}.vtu", state.iteration);
        let filepath = dir.join(&filename);

        write_cvt_state_vtu(&filepath, domain, state, &density)?;

        pvd_entries.push((state.iteration as f64, filename));
    }

    let pvd_path = dir.join("cvt.pvd");
    let entries_ref: Vec<(f64, &str)> = pvd_entries
        .iter()
        .map(|(t, f)| (*t, f.as_str()))
        .collect();

    write_pvd(&pvd_path, &entries_ref)?;

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meshgen::cvt::{lloyd_iter, uniform_seeds, Domain1D};
    use std::fs;

    #[test]
    fn write_single_state_creates_file() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds = uniform_seeds(5, &domain);
        let state = lloyd_iter(domain.clone(), seeds, density).nth(10).unwrap();

        let path = "/tmp/strelitzia_test_cvt_state.vtu";
        let result = write_cvt_state_vtu(path, &domain, &state, |_| 1.0);

        assert!(result.is_ok());
        assert!(fs::metadata(path).is_ok());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn write_history_creates_directory() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds = uniform_seeds(5, &domain);
        let history: Vec<_> = lloyd_iter(domain.clone(), seeds, density).take(5).collect();

        let dir = "/tmp/strelitzia_test_cvt_history";
        let result = write_cvt_history_pvd(dir, &domain, &history, |_| 1.0);

        assert!(result.is_ok());
        assert!(fs::metadata(format!("{}/cvt.pvd", dir)).is_ok());
        assert!(fs::metadata(format!("{}/cvt_0000.vtu", dir)).is_ok());

        let _ = fs::remove_dir_all(dir);
    }
}
