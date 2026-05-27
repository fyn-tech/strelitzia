//! CVT visualization using plotpy (Matplotlib backend).
//!
//! Provides functions to generate publication-quality plots of CVT results,
//! matching the style of the Python Jupyter notebook prototypes (dark background,
//! consistent colors, log-scaled energy axes, boundary-path evolution plots).
//!
//! # Requirements
//!
//! Requires Python 3 and Matplotlib installed on the system.
//!
//! # Example
//!
//! ```ignore
//! use strelitzia::meshgen::cvt::*;
//! use strelitzia::meshgen::cvt_plot::*;
//!
//! let domain = Domain1D::new(0.0, 1.0);
//! let density = |x: Real| 1.0;
//! let seeds = uniform_seeds(10, &domain);
//! let history: Vec<_> = lloyd_iter(domain.clone(), seeds, density).take(50).collect();
//!
//! plot_energy_convergence(&history, "energy.png").unwrap();
//! ```

use crate::common::Real;
use crate::meshgen::cvt::{cvt_gradient, CvtPoint, CvtState, Domain1D};

use plotpy::{Curve, Plot, StrError};
use std::path::Path;

fn styled_plot() -> Plot {
    let mut plot = Plot::new();
    plot.extra("plt.style.use('dark_background')\nplt.rcParams['figure.dpi'] = 400\nplt.rcParams['savefig.dpi'] = 400\n");
    plot
}

// ============================================================================
// Energy convergence plot (generic, works for any dimension)
// ============================================================================

/// Plot CVT energy vs iteration number (log-scaled y-axis).
///
/// Works for any dimension (1D, 2D, 3D).
///
/// # Arguments
/// * `history` - Slice of CVT states from an optimization run
/// * `path` - Output file path (supports .png, .pdf, .svg)
pub fn plot_energy_convergence_nd<P2: CvtPoint, P: AsRef<Path>>(
    history: &[CvtState<P2>],
    path: P,
) -> Result<(), StrError> {
    plot_energy_convergence_impl(
        &history.iter().map(|s| (s.iteration, s.energy)).collect::<Vec<_>>(),
        path,
    )
}

fn plot_energy_convergence_impl<P: AsRef<Path>>(
    data: &[(usize, Real)],
    path: P,
) -> Result<(), StrError> {
    if data.is_empty() {
        return Ok(());
    }

    let x: Vec<Real> = data.iter().map(|&(i, _)| i as Real).collect();
    let y: Vec<Real> = data.iter().map(|&(_, e)| e).collect();

    let mut curve = Curve::new();
    curve.set_line_width(1.0);
    curve.set_line_color("cyan");
    curve.draw(&x, &y);

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.set_log_y(true);
    plot.add(&curve)
        .set_title("Total Energy Over Iterations")
        .grid_labels_legend("Iteration", "Total Energy");
    plot.extra("plt.grid(True, alpha=0.3)\n");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

/// Plot CVT energy vs iteration number (log-scaled y-axis).
///
/// Delegates to the generic [`plot_energy_convergence_nd`].
pub fn plot_energy_convergence<P: AsRef<Path>>(
    history: &[CvtState<Real>],
    path: P,
) -> Result<(), StrError> {
    plot_energy_convergence_nd(history, path)
}

// ============================================================================
// Residual convergence plot
// ============================================================================

/// Plot the normalised displacement residual vs iteration (log-scaled y-axis).
///
/// States with `NAN` residuals (argmin-backed solvers) are silently skipped.
/// A horizontal dashed line is drawn at the solver tolerance if all non-NAN
/// values are non-negative.
pub fn plot_residual_convergence_nd<P2: CvtPoint, P: AsRef<Path>>(
    history: &[CvtState<P2>],
    tol: Real,
    path: P,
) -> Result<(), StrError> {
    let data: Vec<(usize, Real)> = history
        .iter()
        .filter(|s| s.residual.is_finite())
        .map(|s| (s.iteration, s.residual))
        .collect();

    if data.is_empty() {
        return Ok(());
    }

    let x: Vec<Real> = data.iter().map(|&(i, _)| i as Real).collect();
    let y: Vec<Real> = data.iter().map(|&(_, r)| r).collect();

    let mut curve = Curve::new();
    curve.set_line_width(1.0);
    curve.set_line_color("cyan");
    curve.draw(&x, &y);

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.set_log_y(true);
    plot.add(&curve)
        .set_title("Normalised Displacement Residual")
        .grid_labels_legend("Iteration", "Residual");

    // Tolerance reference line
    let n_last = *x.last().unwrap_or(&0.0);
    plot.extra(&format!(
        "plt.axhline({tol:.2e}, color='orange', linewidth=0.8, linestyle='--', alpha=0.7, label='tol')\n\
         plt.legend(fontsize=9, framealpha=0.3)\n\
         plt.xlim(0, {n_last})\n\
         plt.grid(True, which='both', alpha=0.2)\n"
    ));

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

/// Plot the normalised displacement residual vs iteration (1D convenience wrapper).
pub fn plot_residual_convergence<P: AsRef<Path>>(
    history: &[CvtState<Real>],
    tol: Real,
    path: P,
) -> Result<(), StrError> {
    plot_residual_convergence_nd(history, tol, path)
}

// ============================================================================
// Boundary paths plot (seed / cell boundary evolution)
// ============================================================================

/// Plot cell boundary paths over iterations.
///
/// Mimics the Python `plot_bound_paths` function: x-axis shows boundary
/// positions within the domain, y-axis shows iteration number (with
/// iteration 0 at the top). All boundaries are drawn in red.
///
/// # Arguments
/// * `domain` - The 1D interval domain
/// * `history` - Slice of CVT states from an optimization run
/// * `path` - Output file path (supports .png, .pdf, .svg)
pub fn plot_boundary_paths<P: AsRef<Path>>(
    domain: &Domain1D,
    history: &[CvtState<Real>],
    path: P,
) -> Result<(), StrError> {
    if history.is_empty() {
        return Ok(());
    }

    let n_seeds = history[0].seeds.len();
    let n_bounds = n_seeds + 1;

    let start_iter = history.first().map(|s| s.iteration).unwrap_or(0);
    let end_iter = history.last().map(|s| s.iteration).unwrap_or(0);

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 8.0);

    for bound_idx in 0..n_bounds {
        let positions: Vec<Real> = history
            .iter()
            .map(|s| {
                let cells = domain.voronoi_cells(s.seeds.as_slice());
                if bound_idx == 0 {
                    cells[0].0
                } else if bound_idx == n_bounds - 1 {
                    cells[n_seeds - 1].1
                } else {
                    cells[bound_idx].0
                }
            })
            .collect();

        let neg_iters: Vec<Real> = history.iter().map(|s| -(s.iteration as Real)).collect();

        let mut curve = Curve::new();
        curve.set_line_width(0.6);
        curve.set_line_color("red");
        curve.set_line_alpha(0.8);
        curve.draw(&positions, &neg_iters);

        plot.add(&curve);
    }

    plot.set_title(&format!(
        "Paths Over Iters: [{}, {}]",
        start_iter, end_iter
    ))
    .set_labels("Cell Boundary Positions", "Iteration")
    .set_xrange(domain.min, domain.max)
    .set_yrange(-(end_iter as Real), -(start_iter as Real) + 1.0);

    let tick_step = std::cmp::max(1, (end_iter - start_iter) / 10);
    let mut tick_cmd = String::from("import numpy as np\n");
    tick_cmd.push_str(&format!(
        "yticks = np.arange({}, {}, {})\n",
        -(end_iter as i64),
        -(start_iter as i64) + 1,
        tick_step
    ));
    tick_cmd.push_str("plt.yticks(yticks, [-int(t) for t in yticks])\n");
    plot.extra(&tick_cmd);

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

/// Plot seed position trajectories over iterations.
///
/// Matches the Python `plot_bound_paths` orientation: x-axis shows
/// seed positions within the domain, y-axis shows iteration number
/// with iteration 0 at the top (inverted). Each seed is drawn as a
/// separate line in cyan.
///
/// # Arguments
/// * `domain` - The 1D interval domain (for x-axis range)
/// * `history` - Slice of CVT states from an optimization run
/// * `path` - Output file path (supports .png, .pdf, .svg)
pub fn plot_seed_evolution<P: AsRef<Path>>(
    domain: &Domain1D,
    history: &[CvtState<Real>],
    path: P,
) -> Result<(), StrError> {
    if history.is_empty() {
        return Ok(());
    }

    let n_seeds = history[0].seeds.len();
    let start_iter = history.first().map(|s| s.iteration).unwrap_or(0);
    let end_iter = history.last().map(|s| s.iteration).unwrap_or(0);

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 8.0);

    for seed_idx in 0..n_seeds {
        let positions: Vec<Real> = history.iter().map(|s| s.seeds[seed_idx]).collect();
        let neg_iters: Vec<Real> = history.iter().map(|s| -(s.iteration as Real)).collect();

        let mut curve = Curve::new();
        curve.set_line_width(0.6);
        curve.set_line_color("cyan");
        curve.set_line_alpha(0.8);
        curve.draw(&positions, &neg_iters);

        plot.add(&curve);
    }

    plot.set_title(&format!(
        "Seed Paths Over Iters: [{}, {}]",
        start_iter, end_iter
    ))
    .set_labels("Seed Positions", "Iteration")
    .set_xrange(domain.min, domain.max)
    .set_yrange(-(end_iter as Real), -(start_iter as Real) + 1.0);

    let tick_step = std::cmp::max(1, (end_iter - start_iter) / 10);
    let mut tick_cmd = String::from("import numpy as np\n");
    tick_cmd.push_str(&format!(
        "yticks = np.arange({}, {}, {})\n",
        -(end_iter as i64),
        -(start_iter as i64) + 1,
        tick_step
    ));
    tick_cmd.push_str("plt.yticks(yticks, [-int(t) for t in yticks])\n");
    plot.extra(&tick_cmd);

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

// ============================================================================
// Cell sizes plot
// ============================================================================

/// Plot cell sizes vs position, with expected curve overlay.
///
/// Shows the actual cell sizes from the CVT result compared to the
/// theoretically expected sizes (inversely proportional to density).
pub fn plot_cell_sizes<F, P>(
    domain: &Domain1D,
    seeds: &[Real],
    density: F,
    path: P,
) -> Result<(), StrError>
where
    F: Fn(Real) -> Real,
    P: AsRef<Path>,
{
    if seeds.is_empty() {
        return Ok(());
    }

    let cells = domain.voronoi_cells(seeds);

    let cell_positions: Vec<Real> = seeds.to_vec();
    let cell_sizes: Vec<Real> = cells.iter().map(|(l, r)| r - l).collect();

    let mut actual = Curve::new();
    actual.set_marker_style("o");
    actual.set_marker_size(6.0);
    actual.set_line_style("None");
    actual.set_label("Actual");
    actual.draw(&cell_positions, &cell_sizes);

    let n_points = 101;
    let expected_x: Vec<Real> = (0..n_points)
        .map(|i| domain.min + (i as Real / (n_points - 1) as Real) * domain.length())
        .collect();

    let total_density: Real =
        expected_x.iter().map(|&x| density(x)).sum::<Real>() / n_points as Real;
    let scale = domain.length() / (seeds.len() as Real * total_density);

    let expected_y: Vec<Real> = expected_x
        .iter()
        .map(|&x| scale / density(x) * total_density)
        .collect();

    let mut expected = Curve::new();
    expected.set_line_width(1.5);
    expected.set_line_color("red");
    expected.set_line_alpha(0.7);
    expected.set_label("Expected (∝ 1/ρ)");
    expected.draw(&expected_x, &expected_y);

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.add(&expected)
        .add(&actual)
        .set_title("Cell Size Distribution")
        .grid_labels_legend("Position", "Cell Size");
    plot.extra("plt.grid(True, alpha=0.3)\n");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

// ============================================================================
// Gradient convergence plot
// ============================================================================

/// Plot gradient norm (log scale) vs iteration.
///
/// Shows convergence of the gradient toward zero, indicating approach
/// to the CVT solution.
pub fn plot_gradient_convergence<F, P>(
    domain: &Domain1D,
    history: &[CvtState<Real>],
    density: F,
    path: P,
) -> Result<(), StrError>
where
    F: Fn(Real) -> Real,
    P: AsRef<Path>,
{
    if history.is_empty() {
        return Ok(());
    }

    let x: Vec<Real> = history.iter().map(|s| s.iteration as Real).collect();
    let y: Vec<Real> = history
        .iter()
        .map(|s| {
            let grad = cvt_gradient(domain, &s.seeds, &density);
            grad.iter().map(|g| g * g).sum::<Real>().sqrt()
        })
        .collect();

    let mut curve = Curve::new();
    curve.set_line_width(1.0);
    curve.set_line_color("cyan");
    curve.draw(&x, &y);

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.set_log_y(true);
    plot.add(&curve)
        .set_title("Energy Gradient Magnitude")
        .grid_labels_legend("Iteration", "Gradient Magnitude");
    plot.extra("plt.grid(True, alpha=0.3)\n");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

// ============================================================================
// Solver comparison plot
// ============================================================================

/// Plot energy convergence comparison for multiple solvers (log-scaled y-axis).
pub fn plot_solver_comparison<P: AsRef<Path>>(
    results: &[(&str, &[CvtState<Real>])],
    path: P,
) -> Result<(), StrError> {
    if results.is_empty() {
        return Ok(());
    }

    let colors = ["blue", "red", "green", "orange", "purple"];
    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.set_log_y(true);

    for (i, (name, history)) in results.iter().enumerate() {
        if history.is_empty() {
            continue;
        }

        let x: Vec<Real> = history.iter().map(|s| s.iteration as Real).collect();
        let y: Vec<Real> = history.iter().map(|s| s.energy).collect();

        let mut curve = Curve::new();
        curve.set_line_width(1.5);
        curve.set_line_color(colors[i % colors.len()]);
        curve.set_marker_style("o");
        curve.set_marker_size(4.0);
        curve.set_label(name);
        curve.draw(&x, &y);

        plot.add(&curve);
    }

    plot.set_title("Solver Comparison")
        .grid_labels_legend("Iteration", "Energy");
    plot.extra("plt.grid(True, alpha=0.3)\n");

    // Axis limits - adjust these to change plot scales
    plot.extra("plt.grid(True, alpha=0.3)\n");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

// ============================================================================
// Mass range plots
// ============================================================================

/// Plot max and min Voronoi cell mass on the same axes (log-y) over iterations.
///
/// States with `NAN` mass values are skipped. For uniform density the two
/// curves converge toward each other; for non-uniform density they converge
/// to fixed values reflecting the target density ratio.
pub fn plot_mass_range_nd<P: CvtPoint, Pa: AsRef<Path>>(
    history: &[CvtState<P>],
    path: Pa,
) -> Result<(), StrError> {
    let data: Vec<(usize, Real, Real)> = history
        .iter()
        .filter(|s| s.max_mass.is_finite() && s.min_mass.is_finite())
        .map(|s| (s.iteration, s.max_mass, s.min_mass))
        .collect();

    if data.is_empty() {
        return Ok(());
    }

    let x: Vec<Real>   = data.iter().map(|&(i, _, _)| i as Real).collect();
    let ymax: Vec<Real> = data.iter().map(|&(_, hi, _)| hi).collect();
    let ymin: Vec<Real> = data.iter().map(|&(_, _, lo)| lo).collect();

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.set_log_y(true);

    let mut c_max = Curve::new();
    c_max.set_line_width(1.2).set_line_color("tomato").set_label("max mass");
    c_max.draw(&x, &ymax);

    let mut c_min = Curve::new();
    c_min.set_line_width(1.2).set_line_color("dodgerblue").set_label("min mass");
    c_min.draw(&x, &ymin);

    plot.add(&c_max).add(&c_min)
        .set_title("Cell Mass Range Over Iterations")
        .grid_labels_legend("Iteration", "Cell Mass");
    plot.extra("plt.legend(fontsize=9, framealpha=0.3)\nplt.grid(True, which='both', alpha=0.2)\n");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

/// Plot the spread (max − min cell mass) over iterations (log-y).
///
/// For uniform density this should converge toward zero; for non-uniform
/// density it converges to a non-zero value equal to `max_ρ/min_ρ` times
/// the minimum cell area.
pub fn plot_mass_spread_nd<P: CvtPoint, Pa: AsRef<Path>>(
    history: &[CvtState<P>],
    path: Pa,
) -> Result<(), StrError> {
    let data: Vec<(usize, Real)> = history
        .iter()
        .filter(|s| s.max_mass.is_finite() && s.min_mass.is_finite())
        .map(|s| (s.iteration, (s.max_mass - s.min_mass).abs()))
        .collect();

    if data.is_empty() {
        return Ok(());
    }

    let x: Vec<Real> = data.iter().map(|&(i, _)| i as Real).collect();
    let y: Vec<Real> = data.iter().map(|&(_, d)| d).collect();

    let mut curve = Curve::new();
    curve.set_line_width(1.2).set_line_color("gold");
    curve.draw(&x, &y);

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.set_log_y(true);
    plot.add(&curve)
        .set_title("Cell Mass Spread (max − min) Over Iterations")
        .grid_labels_legend("Iteration", "max mass − min mass");
    plot.extra("plt.grid(True, which='both', alpha=0.2)\n");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

/// Overlay normalised displacement residuals for multiple solvers (semi-log y-axis).
///
/// Each solver's residual history is drawn in a distinct colour. States with
/// `NAN` residuals (argmin-backed solvers) are silently skipped, so this
/// function degrades gracefully for solvers that do not record the residual.
/// A horizontal dashed line is drawn at `tol`.
pub fn plot_residual_solver_comparison<P: AsRef<Path>>(
    results: &[(&str, &[CvtState<Real>])],
    tol: Real,
    path: P,
) -> Result<(), StrError> {
    if results.is_empty() {
        return Ok(());
    }

    let colors = ["dodgerblue", "tomato", "limegreen", "gold", "violet"];
    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 6.0);
    plot.set_log_y(true);

    for (i, (name, history)) in results.iter().enumerate() {
        let data: Vec<(usize, Real)> = history
            .iter()
            .filter(|s| s.residual.is_finite())
            .map(|s| (s.iteration, s.residual))
            .collect();

        if data.is_empty() {
            continue;
        }

        let x: Vec<Real> = data.iter().map(|&(i, _)| i as Real).collect();
        let y: Vec<Real> = data.iter().map(|&(_, r)| r).collect();

        let mut curve = Curve::new();
        curve.set_line_width(1.5);
        curve.set_line_color(colors[i % colors.len()]);
        curve.set_label(name);
        curve.draw(&x, &y);
        plot.add(&curve);
    }

    let n_last = results
        .iter()
        .flat_map(|(_, h)| h.iter())
        .map(|s| s.iteration)
        .max()
        .unwrap_or(0) as f64;

    plot.extra(&format!(
        "plt.axhline({tol:.2e}, color='orange', linewidth=0.8, linestyle='--', alpha=0.7, label='tol')\n\
         plt.legend(fontsize=9, framealpha=0.3)\n\
         plt.xlim(0, {n_last})\n\
         plt.grid(True, which='both', alpha=0.2)\n"
    ));

    plot.set_title("Solver Residual Comparison")
        .grid_labels_legend("Iteration", "Normalised Residual");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
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
    fn energy_plot_generates_file() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds = uniform_seeds(5, &domain);
        let history: Vec<_> = lloyd_iter(domain, seeds, density).take(10).collect();

        let path = "/tmp/strelitzia_test_energy.png";
        let result = plot_energy_convergence(&history, path);

        if result.is_ok() {
            assert!(fs::metadata(path).is_ok());
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn seed_evolution_generates_file() {
        let domain = Domain1D::new(0.0, 1.0);
        let density = |_x: Real| 1.0;
        let seeds = uniform_seeds(5, &domain);
        let history: Vec<_> = lloyd_iter(domain.clone(), seeds, density).take(10).collect();

        let path = "/tmp/strelitzia_test_seeds.png";
        let result = plot_seed_evolution(&domain, &history, path);

        if result.is_ok() {
            assert!(fs::metadata(path).is_ok());
            let _ = fs::remove_file(path);
        }
    }
}
