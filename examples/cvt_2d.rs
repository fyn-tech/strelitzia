//! 2D Centroidal Voronoi Tessellation (CVT) demo.
//!
//! Demonstrates CVT algorithms for optimal seed distributions in 2D:
//! - Lloyd's algorithm on a rectangular domain
//! - Uniform and non-uniform density across multiple seed counts
//! - Delaunay triangulation mesh output
//! - VTK/ParaView export and animated GIFs
//!
//! Output layout:
//!   tmp/2d/uniform/     – plots for each N (area, mass, trajectories, GIF)
//!   tmp/2d/nonuniform/  – same for the Gaussian-density case
//!   tmp/2d/vtk/         – VTU/PVD files for ParaView
//!
//! Run with: cargo run --release --example cvt_2d

use std::fs;
use std::time::Instant;
use strelitzia::common::Real;
use strelitzia::meshgen::cvt::*;
use strelitzia::meshgen::cvt_plot::{
    plot_energy_convergence_nd, plot_mass_range_nd, plot_mass_spread_nd,
    plot_residual_convergence_nd,
};
use strelitzia::meshgen::cvt_plot_2d::*;
use strelitzia::meshgen::cvt_solvers::*;
use strelitzia::meshgen::cvt_vtk_2d::*;
use strelitzia::meshgen::mesh::Mesh;
use strelitzia::multiarray::Point2;

/// Seeds below this threshold get full history (trajectories + GIF).
/// Above it, `lloyd_cvt_no_history` is used to avoid O(N·iters) memory.
const LARGE_N: usize = 5_000;

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║          2D Centroidal Voronoi Tessellation          ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    fs::create_dir_all("tmp/2d/uniform").ok();
    fs::create_dir_all("tmp/2d/nonuniform").ok();
    fs::create_dir_all("tmp/2d/vtk").ok();

    let seed_counts = [100, 1_000, 2_000];

    uniform_density_sweep(&seed_counts);
    nonuniform_density_sweep(&seed_counts);
    mesh_output_demo();
    convergence_analysis();
    scalability_benchmark();

    println!("=== Output files ===\n");
    println!("Plots: tmp/2d/uniform/   and  tmp/2d/nonuniform/");
    println!("VTK:   tmp/2d/vtk/");
    println!("Open .vtu files in ParaView to visualize.\n");
}

// ============================================================================
// Convergence reporting
// ============================================================================

fn print_result(result: &CvtResult<Point2>, n_seeds: usize) {
    let final_state = result.history.last().unwrap();
    let total_iters = final_state.iteration + 1;

    println!(
        "  N={:<6}  energy={:.6e}  iters={:<6}  residual={:.2e}  time={:.3?}  converged={}",
        n_seeds,
        final_state.energy,
        total_iters,
        result.final_residual,
        result.elapsed,
        result.converged,
    );
    if let Some(ci) = result.converge_iter {
        println!(
            "           criterion first met at iter {}, sustained for {} more",
            ci,
            total_iters.saturating_sub(ci + 1)
        );
    } else {
        println!("           WARNING: hit max_iter without converging");
    }
}

// ============================================================================
// Plot / VTK emission for one CVT run
//
// When history has only one entry (large-N no-history run) we skip the
// trajectory plot and the animated GIF — both require per-iteration data.
// ============================================================================

fn emit_plots<F: Fn(Point2) -> Real>(
    label: &str,      // "uniform" | "nonuniform"  — used in VTK filenames
    subdir: &str,     // "uniform" | "nonuniform"  — output subdirectory
    domain: &Domain2D,
    result: &CvtResult<Point2>,
    density: F,
    n_seeds: usize,
) {
    let final_state = result.history.last().unwrap();
    let has_history = result.history.len() > 1;
    let base = format!("tmp/2d/{}/n{}", subdir, n_seeds);

    if has_history {
        let tol = CvtSolverSettings::default().tol;
        macro_rules! emit {
            ($fn:expr, $suffix:expr, $label:expr) => {
                match $fn {
                    Ok(()) => println!("  Plot: {}_{}.png", base, $suffix),
                    Err(e)  => println!("  Plot error ({}): {}", $label, e),
                }
            };
        }
        emit!(plot_seed_trajectories_2d(domain, &result.history,
            format!("{}_trajectories.png", base)), "trajectories", "trajectories");
        emit!(plot_energy_convergence_nd(&result.history,
            format!("{}_energy.png", base)), "energy", "energy");
        emit!(plot_residual_convergence_nd(&result.history, tol,
            format!("{}_residual.png", base)), "residual", "residual");
        emit!(plot_mass_range_nd(&result.history,
            format!("{}_mass_range.png", base)), "mass_range", "mass_range");
        emit!(plot_mass_spread_nd(&result.history,
            format!("{}_mass_spread.png", base)), "mass_spread", "mass_spread");
    }

    match plot_voronoi_cells_2d(domain, final_state.seeds.as_slice(), format!("{}_area.png", base)) {
        Ok(()) => println!("  Plot: {}_area.png", base),
        Err(e)  => println!("  Plot error (area): {}", e),
    }
    match plot_voronoi_cells_2d_mass(domain, final_state.seeds.as_slice(), &density, format!("{}_mass.png", base)) {
        Ok(()) => println!("  Plot: {}_mass.png", base),
        Err(e)  => println!("  Plot error (mass): {}", e),
    }

    // GIFs are expensive at high resolution; skip for large N.
    const GIF_MAX_N: usize = 1_000;
    if has_history && n_seeds <= GIF_MAX_N {
        let frame_step = (result.history.len() / 60).max(1);
        match animate_voronoi_evolution(domain, &result.history, &density,
            format!("{}_evolution.gif", base), frame_step) {
            Ok(()) => println!("  GIF:  {}_evolution.gif", base),
            Err(e)  => println!("  GIF error: {}", e),
        }
    } else if n_seeds > GIF_MAX_N {
        println!("  GIF:  skipped for N={} (use N≤{} for animations)", n_seeds, GIF_MAX_N);
    }

    match write_cvt_2d_delaunay_vtu(
        format!("tmp/2d/vtk/{}_{}_delaunay.vtu", label, n_seeds),
        domain, final_state, &density,
    ) {
        Ok(()) => println!("  VTU:  tmp/2d/vtk/{}_{}_delaunay.vtu", label, n_seeds),
        Err(e)  => println!("  VTU error: {}", e),
    }
    match write_cvt_2d_voronoi_vtu(
        format!("tmp/2d/vtk/{}_{}_voronoi.vtu", label, n_seeds),
        domain, final_state, &density,
    ) {
        Ok(()) => println!("  VTU:  tmp/2d/vtk/{}_{}_voronoi.vtu", label, n_seeds),
        Err(e)  => println!("  VTU error: {}", e),
    }
}

// ============================================================================
// 1. Uniform density sweep
// ============================================================================

fn uniform_density_sweep(seed_counts: &[usize]) {
    println!("=== 1. Uniform Density CVT (Unit Square) ===\n");

    let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
    let density = |_: Point2| 1.0_f64;
    let settings = CvtSolverSettings::default();

    for &n in seed_counts {
        println!("--- N = {} ---", n);
        let seeds = domain.uniform_seeds(n);

        let result = if n >= LARGE_N {
            println!("  (large-N mode: history not stored; trajectories and GIF skipped)");
            lloyd_cvt_no_history(&domain, seeds, density, &settings)
        } else {
            lloyd_cvt(&domain, seeds, density, &settings)
        };

        print_result(&result, n);
        emit_plots("uniform", "uniform", &domain, &result, density, n);
        println!();
    }
}

// ============================================================================
// 2. Non-uniform density sweep (Gaussian centred at corner (0, 0))
// ============================================================================

fn nonuniform_density_sweep(seed_counts: &[usize]) {
    println!("=== 2. Non-uniform Density CVT (Gaussian at corner) ===\n");
    println!("Higher density near (0, 0) → smaller cells at bottom-left\n");

    let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
    // Density ratio 100:1  →  cell linear-size ratio √100 = 10:1
    let density = |p: Point2| -> Real {
        let r2 = p[0] * p[0] + p[1] * p[1];
        1.0 + 99.0 * (-r2 / 0.15).exp()
    };
    let settings = CvtSolverSettings::default();

    for &n in seed_counts {
        println!("--- N = {} ---", n);
        let seeds = domain.uniform_seeds(n);

        let result = if n >= LARGE_N {
            println!("  (large-N mode: history not stored; trajectories and GIF skipped)");
            lloyd_cvt_no_history(&domain, seeds, density, &settings)
        } else {
            lloyd_cvt(&domain, seeds, density, &settings)
        };

        print_result(&result, n);
        emit_plots("nonuniform", "nonuniform", &domain, &result, density, n);
        println!();
    }
}

// ============================================================================
// 3. Mesh output via Delaunay triangulation
// ============================================================================

fn mesh_output_demo() {
    println!("=== 3. Delaunay Mesh from CVT Seeds ===\n");

    let domain = Domain2D::rectangle(0.0, 2.0, 0.0, 1.0);
    let density = |_: Point2| 1.0_f64;
    let seeds = domain.uniform_seeds(100);

    let result = lloyd_cvt(&domain, seeds, density, &CvtSolverSettings::default());
    let final_state = result.history.last().unwrap();

    let mesh = Mesh::delaunay(final_state.seeds.clone());

    println!("Mesh vertices:  {}", mesh.num_vertices());
    println!("Mesh triangles: {}", mesh.num_cells());
    println!("Iterations:     {}", final_state.iteration + 1);
    println!("Residual:       {:.2e}", result.final_residual);
    println!("Converged:      {}", result.converged);
    println!("Time:           {:.3?}", result.elapsed);

    let _ = write_cvt_2d_delaunay_vtu("tmp/2d/vtk/rect_mesh.vtu", &domain, final_state, density);
    println!("Exported: tmp/2d/vtk/rect_mesh.vtu\n");
}

// ============================================================================
// 4. Convergence analysis (energy vs iteration)
// ============================================================================

fn convergence_analysis() {
    println!("=== 4. Convergence Rate Analysis ===\n");

    let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
    let density = |_: Point2| 1.0_f64;
    let seeds = domain.uniform_seeds(30);

    let result = lloyd_cvt(&domain, seeds, density, &CvtSolverSettings::default());
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

    let _ = plot_energy_convergence_nd(history, "tmp/2d/convergence_energy.png");
    println!("Plot: tmp/2d/convergence_energy.png\n");
}

// ============================================================================
// 5. Scalability benchmark
//    5a. Throughput  — fixed iterations, N up to 1M, log-log plot
//    5b. Convergence — full convergence, N up to 10K
// ============================================================================

fn scalability_benchmark() {
    let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);

    let densities: [(&str, fn(Point2) -> Real); 2] = [
        ("uniform",    |_: Point2| 1.0_f64),
        ("nonuniform", |p: Point2| 1.0 + 99.0 * (-(p[0]*p[0] + p[1]*p[1]) / 0.15).exp()),
    ];

    // ── 5a. Throughput ──────────────────────────────────────────────────────
    // Pure Lloyd iteration cost: integrate_cells only, no residual overhead.
    // Measures raw ms/iter to expose algorithmic scaling vs N.

    println!("=== 5a. Throughput Benchmark (fixed 10 iterations) ===\n");

    let throughput_sizes: &[usize] = &[
        500, 1_000, 2_000, 5_000, 10_000,
        20_000, 50_000, 100_000,
    ];
    const N_FIXED: usize = 10;

    println!(
        "  {:>10} {:>12} {:>12}",
        "N", "uniform ms/it", "nonunif ms/it"
    );
    println!("  {}", "─".repeat(38));

    // Measure each density separately, printing rows as they come in.
    let mut series_data: Vec<(&str, Vec<usize>, Vec<f64>)> =
        densities.iter().map(|(l, _)| (*l, vec![], vec![])).collect();

    for (col, (label, density)) in densities.iter().enumerate() {
        println!("\n  {label}");
        println!(
            "  {:>10} {:>12}",
            "N", "ms/iter"
        );
        println!("  {}", "─".repeat(24));
        for &n in throughput_sizes {
            let seeds = domain.uniform_seeds(n);
            let mut current = seeds;
            let t0 = Instant::now();
            for _ in 0..N_FIXED {
                let data = domain.integrate_cells(current.as_slice(), &density);
                current = data.centroids;
            }
            let ms = t0.elapsed().as_secs_f64() * 1000.0 / N_FIXED as f64;
            println!("  {:>10} {:>12.2}", n, ms);
            series_data[col].1.push(n);
            series_data[col].2.push(ms);
        }
    }
    println!();

    let series_refs: Vec<(&str, &[usize], &[f64])> = series_data
        .iter()
        .map(|(l, s, m)| (*l, s.as_slice(), m.as_slice()))
        .collect();
    match plot_scalability_throughput(&series_refs, "tmp/2d/scalability_throughput.png") {
        Ok(()) => println!("  Plot: tmp/2d/scalability_throughput.png\n"),
        Err(e)  => println!("  Plot error: {}\n", e),
    }

    // ── 5b. Convergence ─────────────────────────────────────────────────────
    // Full convergence with the default tolerance, bounded to sizes where
    // it completes in a reasonable time.

    println!("=== 5b. Convergence Benchmark (full convergence, N ≤ 10K) ===\n");

    let conv_sizes: &[usize] = &[100, 500, 1_000, 2_000];
    let settings = CvtSolverSettings::default();

    for (label, density) in densities {
        println!("  Density: {label}\n");
        println!(
            "  {:>8} {:>8} {:>10} {:>12} {:>11} {:>9}",
            "N", "Iters", "Residual", "Energy", "Time", "ms/iter"
        );
        println!("  {}", "─".repeat(66));

        for &n in conv_sizes {
            let seeds = domain.uniform_seeds(n);
            let r = lloyd_cvt_bench(&domain, seeds, density, &settings);
            let ms = r.elapsed.as_secs_f64() * 1000.0 / r.iterations as f64;
            println!(
                "  {:>8} {:>8} {:>10.2e} {:>12.6e} {:>11.3?} {:>7.2}ms{}",
                n, r.iterations, r.final_residual, r.final_energy,
                r.elapsed, ms,
                if r.converged { "" } else { "  !" },
            );
        }
        println!();
    }
}
