//! 1D Centroidal Voronoi Tessellation (CVT) demo.
//!
//! Demonstrates CVT algorithms for optimal seed distributions in 1D:
//! - Lloyd's algorithm (fixed-point iteration)
//! - Newton's method (quadratic convergence via tridiagonal Hessian, O(n))
//! - L-BFGS quasi-Newton method (O(m·n) per iteration)
//! - Scalability benchmark up to N=1M cells
//! - Matplotlib visualization via plotpy
//! - VTK/ParaView export for time series animation
//!
//! Run with: cargo run --release --example cvt_1d

use std::fs;

use strelitzia::common::Real;
use strelitzia::fields::SolverInterop;
use strelitzia::meshgen::cvt::*;
use strelitzia::meshgen::cvt_plot::*;
use strelitzia::meshgen::cvt_solvers::*;
use strelitzia::meshgen::cvt_vtk::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║          1D Centroidal Voronoi Tessellation          ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    fs::create_dir_all("tmp/1d/plots").ok();
    fs::create_dir_all("tmp/1d/vtk").ok();

    let settings = CvtSolverSettings::default();
    println!(
        "Settings: max_iter={}, tol={:.0e}\n",
        settings.max_iter, settings.tol
    );

    uniform_density_cvt(&settings);
    nonuniform_density_cvt(&settings);
    solver_comparison(&settings);
    scalability_benchmark(&settings);
    convergence_analysis(&settings);
    gradient_analysis(&settings);
    custom_cell_size(&settings);
    vtk_export_demo(&settings);
    solver_interop_demo(&settings);

    println!("\n=== Output Files ===\n");
    println!("Plots: tmp/1d/plots/");
    println!("VTK:   tmp/1d/vtk/");
}

// ============================================================================
// 1. Uniform density
// ============================================================================

fn uniform_density_cvt(settings: &CvtSolverSettings) {
    println!("=== 1. Uniform Density CVT ===\n");

    let domain = Domain1D::new(0.0, 1.0);
    let n_cells = 10;
    let density = |_x: Real| 1.0;

    let initial_seeds = (0..n_cells)
        .map(|i| {
            let t = (i as Real) / (n_cells as Real);
            0.05 + 0.9 * t * t
        })
        .collect();

    let result = lloyd_cvt(&domain, initial_seeds, density, settings);

    let first = &result.history[0];
    let last = result.history.last().unwrap();

    println!("Initial seeds: {:?}", first.seeds.as_slice());
    println!("Final seeds:   {:?}", last.seeds.as_slice());
    println!("Initial energy: {:.6e}", first.energy);
    println!("Final energy:   {:.6e}", last.energy);
    println!("Iterations:     {}", result.history.len());
    println!("Time:           {:.3?}", result.elapsed);
    println!(
        "Expected spacing: {:.4} (uniform optimal)\n",
        1.0 / n_cells as Real
    );

    let _ = plot_energy_convergence(&result.history, "tmp/1d/plots/uniform_energy.png");
    let _ = plot_residual_convergence(&result.history, settings.tol, "tmp/1d/plots/uniform_residual.png");
    let _ = plot_seed_evolution(&domain, &result.history, "tmp/1d/plots/uniform_seeds.png");
    let _ = plot_boundary_paths(&domain, &result.history, "tmp/1d/plots/uniform_bounds.png");
}

// ============================================================================
// 2. Non-uniform (sinusoidal) density
// ============================================================================

fn nonuniform_density_cvt(settings: &CvtSolverSettings) {
    println!("=== 2. Non-uniform Density CVT ===\n");

    let domain = Domain1D::new(0.0, 1.0);
    let n_cells = 15;
    let density = |x: Real| 1.0 + 0.8 * (std::f64::consts::PI * x).sin();

    let seeds = uniform_seeds(n_cells, &domain);
    let result = lloyd_cvt(&domain, seeds, density, settings);

    let final_state = result.history.last().unwrap();
    let cells = domain.voronoi_cells(final_state.seeds.as_slice());

    println!("Final seeds (non-uniform density):");
    println!("{:?}\n", final_state.seeds.as_slice());

    println!("{:<6} {:>10} {:>10}", "Cell", "Seed", "Size");
    println!("{}", "-".repeat(28));
    for (i, ((left, right), &seed)) in cells.iter().zip(final_state.seeds.iter()).enumerate() {
        println!("{:<6} {:>10.6} {:>10.6}", i, seed, right - left);
    }
    println!();

    let _ = plot_cell_sizes(
        &domain,
        final_state.seeds.as_slice(),
        density,
        "tmp/1d/plots/nonuniform_cells.png",
    );
    let _ = plot_energy_convergence(&result.history, "tmp/1d/plots/nonuniform_energy.png");
    let _ = plot_residual_convergence(&result.history, settings.tol, "tmp/1d/plots/nonuniform_residual.png");
}

// ============================================================================
// 3. Solver comparison (small N, with plots)
// ============================================================================

fn make_initial_seeds(n_cells: usize) -> strelitzia::fields::Field<Real> {
    (0..n_cells)
        .map(|i| {
            let t = (i as Real) / (n_cells as Real);
            0.02 + 0.96 * t * t
        })
        .collect()
}

fn solver_comparison(settings: &CvtSolverSettings) {
    println!("=== 3. Solver Comparison ===\n");

    // Lloyd's 1D convergence is O(N²) iterations; at tight tolerances this
    // makes full-history runs prohibitively slow.  The performance TABLE uses
    // the caller's settings (actual tol), while the convergence PLOTS use a
    // fixed, loose tolerance so all three solvers finish quickly and the plot
    // remains a fair pedagogical comparison regardless of settings.tol.
    const PLOT_TOL: Real = 1e-5;
    let plot_settings = CvtSolverSettings {
        tol: PLOT_TOL,
        ..CvtSolverSettings::default()
    };

    let domain = Domain1D::new(0.0, 1.0);
    let density = |_x: Real| 1.0;
    let mesh_sizes = [100, 1_000, 10_000];

    // ── Performance table: bench variants at the caller's tol ─────────────
    println!(
        "{:>8} {:<10} {:>8} {:>12} {:>14} {:>9}",
        "N", "Solver", "Iters", "Time", "Energy", "Residual"
    );
    println!("{}", "─".repeat(70));

    for &n in &mesh_sizes {
        let seeds = make_initial_seeds(n);
        let lloyd  = lloyd_cvt_bench(&domain, seeds.clone(), density, settings);
        let newton = newton_cvt_bench(&domain, seeds.clone(), density, settings);
        let lbfgs  = lbfgs_cvt_bench(&domain, seeds.clone(), density, settings);

        for (name, r) in [("Lloyd", &lloyd), ("Newton", &newton), ("L-BFGS", &lbfgs)] {
            println!(
                "{:>8} {:<10} {:>8} {:>12.3?} {:>14.6e} {:>9.2e}",
                n, name, r.iterations, r.elapsed, r.final_energy, r.final_residual
            );
        }
        println!("{}", "─".repeat(70));
    }
    println!();

    // ── Convergence plots: full-history at fixed PLOT_TOL (N=100 only) ────
    // Newton and L-BFGS converge in ~10–20 iterations; Lloyd in ~2 000.
    // Using a small fixed N keeps the plots readable and the run fast.
    let n_plot = 100_usize;
    let seeds = make_initial_seeds(n_plot);
    let lloyd  = lloyd_cvt(&domain, seeds.clone(), density, &plot_settings);
    let newton = newton_cvt(&domain, seeds.clone(), density, &plot_settings);
    let lbfgs  = lbfgs_cvt(&domain, seeds.clone(), density, &plot_settings);

    let plot_results: Vec<(&str, &[CvtState<Real>])> = vec![
        ("Lloyd",  &lloyd.history),
        ("Newton", &newton.history),
        ("L-BFGS", &lbfgs.history),
    ];
    println!(
        "Comparison plots (N={}, tol={:.0e}):", n_plot, PLOT_TOL
    );
    let _ = plot_solver_comparison(
        &plot_results,
        format!("tmp/1d/plots/solver_comparison_n{}.png", n_plot),
    );
    println!("  tmp/1d/plots/solver_comparison_n{}.png", n_plot);
    let _ = plot_residual_solver_comparison(
        &plot_results,
        PLOT_TOL,
        format!("tmp/1d/plots/solver_residual_n{}.png", n_plot),
    );
    println!("  tmp/1d/plots/solver_residual_n{}.png", n_plot);
    println!();

    println!();
}

// ============================================================================
// 4. Scalability benchmark (no history, timing only)
// ============================================================================

fn scalability_benchmark(settings: &CvtSolverSettings) {
    println!("=== 4. Scalability Benchmark ===\n");
    println!("Sparse solvers only (Lloyd O(n), Newton O(n), L-BFGS O(m·n)).");
    println!("BFGS excluded: O(n²) per iteration, does not scale.\n");

    let domain = Domain1D::new(0.0, 1.0);
    let density = |_x: Real| 1.0;
    let mesh_sizes = [100, 1_000, 10_000, 100_000, 1_000_000];

    println!(
        "{:>10} {:<10} {:>8} {:>14} {:>14}",
        "N", "Solver", "Iters", "Final Energy", "Time"
    );
    println!("{}", "─".repeat(62));

    for &n in &mesh_sizes {
        let seeds = make_initial_seeds(n);

        let lloyd = lloyd_cvt_bench(&domain, seeds.clone(), density, settings);
        let newton = newton_cvt_bench(&domain, seeds.clone(), density, settings);
        let lbfgs = lbfgs_cvt_bench(&domain, seeds.clone(), density, settings);

        let solvers: [(&str, &CvtBenchResult); 3] = [
            ("Lloyd", &lloyd),
            ("Newton", &newton),
            ("L-BFGS", &lbfgs),
        ];

        for (name, r) in &solvers {
            println!(
                "{:>10} {:<10} {:>8} {:>14.6e} {:>14.3?}",
                n, name, r.iterations, r.final_energy, r.elapsed
            );
        }
        println!("{}", "─".repeat(62));
    }

    println!();
}

// ============================================================================
// 5. Convergence rate
// ============================================================================

fn convergence_analysis(settings: &CvtSolverSettings) {
    println!("=== 5. Convergence Rate Analysis ===\n");

    let domain = Domain1D::new(0.0, 1.0);
    let density = |_x: Real| 1.0;
    let seeds: strelitzia::fields::Field<Real> =
        [0.1, 0.3, 0.4, 0.7, 0.95].into_iter().collect();

    let result = lloyd_cvt(&domain, seeds, density, settings);
    let history = &result.history;

    let final_energy = history.last().unwrap().energy;

    println!("{:<6} {:>14} {:>14}", "Iter", "Energy", "ln(E - E*)");
    println!("{}", "-".repeat(36));
    for s in history.iter().take(20) {
        let err = s.energy - final_energy;
        if err > 1e-15 {
            println!("{:<6} {:>14.6e} {:>14.4}", s.iteration, s.energy, err.ln());
        } else {
            println!("{:<6} {:>14.6e} {:>14}", s.iteration, s.energy, "converged");
        }
    }
    if history.len() > 20 {
        println!("... ({} more iterations)", history.len() - 20);
    }
    println!();
}

// ============================================================================
// 6. Gradient analysis
// ============================================================================

fn gradient_analysis(settings: &CvtSolverSettings) {
    println!("=== 6. Gradient Analysis ===\n");

    let domain = Domain1D::new(0.0, 1.0);
    let density = |_x: Real| 1.0;
    let seeds: strelitzia::fields::Field<Real> =
        [0.1, 0.3, 0.4, 0.7, 0.95].into_iter().collect();

    let result = lloyd_cvt(&domain, seeds, &density, settings);
    let history = &result.history;

    println!("{:<6} {:>14}", "Iter", "ln(||∇E||)");
    println!("{}", "-".repeat(22));
    for s in history.iter().take(15) {
        let grad = cvt_gradient(&domain, &s.seeds, &density);
        let norm: Real = grad.iter().map(|g| g * g).sum::<Real>().sqrt();
        let log_norm = if norm > 1e-30 { norm.ln() } else { -30.0 };
        println!("{:<6} {:>14.4}", s.iteration, log_norm);
    }
    if history.len() > 15 {
        println!("... ({} more iterations)", history.len() - 15);
    }
    println!();

    let _ = plot_gradient_convergence(&domain, history, &density, "tmp/1d/plots/gradient.png");
}

// ============================================================================
// 7. Custom cell-size function
// ============================================================================

fn custom_cell_size(settings: &CvtSolverSettings) {
    println!("=== 7. Custom Cell-Size Function ===\n");
    println!("Target: small cells at boundaries, large in center\n");

    let domain = Domain1D::new(0.0, 1.0);
    let cell_size_fn = |x: Real| 0.5 + 1.5 * x * (1.0 - x);
    let density_fn = density_from_cell_size(cell_size_fn, domain.length());

    let seeds = uniform_seeds(12, &domain);
    let result = lloyd_cvt(&domain, seeds, &density_fn, settings);

    let final_state = result.history.last().unwrap();
    let cells = domain.voronoi_cells(final_state.seeds.as_slice());

    println!(
        "{:<6} {:>10} {:>10} {:>10}",
        "Cell", "Seed", "Size", "Target"
    );
    println!("{}", "-".repeat(40));
    for (i, ((left, right), &seed)) in cells.iter().zip(final_state.seeds.iter()).enumerate() {
        let actual_size = right - left;
        let target = cell_size_fn(seed) * 0.12;
        println!(
            "{:<6} {:>10.6} {:>10.6} {:>10.6}",
            i, seed, actual_size, target
        );
    }
    println!();
}

// ============================================================================
// 8. VTK export
// ============================================================================

fn vtk_export_demo(settings: &CvtSolverSettings) {
    println!("=== 8. VTK/ParaView Export ===\n");

    let domain = Domain1D::new(0.0, 1.0);
    let density = |_x: Real| 1.0;
    let seeds = uniform_seeds(10, &domain);

    let result = lloyd_cvt(&domain, seeds, density, settings);

    let final_state = result.history.last().unwrap();
    let vtu = write_cvt_state_vtu("tmp/1d/vtk/cvt_final.vtu", &domain, final_state, |_| 1.0);
    match vtu {
        Ok(()) => println!("Exported: tmp/1d/vtk/cvt_final.vtu"),
        Err(e) => println!("VTU export error: {}", e),
    }

    let pvd = write_cvt_history_sampled_pvd(
        "tmp/1d/vtk/time_series",
        &domain,
        &result.history,
        |_| 1.0,
        5,
    );
    match pvd {
        Ok(()) => {
            println!("Exported: tmp/1d/vtk/time_series/cvt.pvd");
            println!("  ({} VTU frames)", (result.history.len() + 4) / 5 + 1);
        }
        Err(e) => println!("PVD export error: {}", e),
    }
    println!();
}

// ============================================================================
// 9. Solver interop
// ============================================================================

fn solver_interop_demo(settings: &CvtSolverSettings) {
    println!("=== 9. Solver Interop ===\n");

    let domain = Domain1D::new(0.0, 1.0);
    let density = |_x: Real| 1.0;
    let seeds = uniform_seeds(10, &domain);

    let result = lloyd_cvt(&domain, seeds, density, settings);
    let final_state = result.history.last().unwrap();

    let flat: &[Real] = final_state.seeds.as_flat_slice();
    println!("Field length:      {}", final_state.seeds.len());
    println!("Flat slice length: {}", flat.len());
    println!("Flat data: {:?}", flat);
}
