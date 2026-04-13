//! 2D CVT visualization using plotpy (Matplotlib backend).
//!
//! Provides functions to visualize 2D CVT results:
//! - Seed trajectory plots (paths colored by iteration)
//! - Voronoi cell diagrams (colored by area or mass)
//! - Animated GIF of Voronoi cell evolution

use crate::common::Real;
use crate::meshgen::cvt::{CvtDomain, CvtState, Domain2D};
use crate::multiarray::Point2;

use plotpy::{Plot, StrError};
use std::path::Path;

fn styled_plot() -> Plot {
    let mut plot = Plot::new();
    plot.extra("plt.style.use('dark_background')\nplt.rcParams['figure.dpi'] = 400\nplt.rcParams['savefig.dpi'] = 400\n");
    plot
}

// ============================================================================
// Seed trajectories
// ============================================================================

/// Plot 2D seed trajectories over Lloyd iterations.
///
/// Each seed's (x, y) path is drawn as a line with color gradient from
/// the initial position (blue) to the final position (red). Initial
/// positions are marked with circles, final positions with stars.
pub fn plot_seed_trajectories_2d<P: AsRef<Path>>(
    domain: &Domain2D,
    history: &[CvtState<Point2>],
    path: P,
) -> Result<(), StrError> {
    if history.is_empty() {
        return Ok(());
    }

    let n_seeds = history[0].seeds.len();
    let n_iters = history.len();
    let (bb_min, bb_max) = domain.bounding_box();

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 10.0);

    let mut py = String::new();
    py.push_str("import numpy as np\n");
    py.push_str("from matplotlib.collections import LineCollection\n");
    py.push_str("from matplotlib.colors import Normalize\n");
    py.push_str("import matplotlib.cm as cm\n\n");

    for si in 0..n_seeds {
        let xs: Vec<String> = history.iter().map(|s| format!("{:.10}", s.seeds[si][0])).collect();
        let ys: Vec<String> = history.iter().map(|s| format!("{:.10}", s.seeds[si][1])).collect();

        py.push_str(&format!("xs = [{}]\n", xs.join(",")));
        py.push_str(&format!("ys = [{}]\n", ys.join(",")));
        py.push_str("pts = np.array([xs, ys]).T.reshape(-1, 1, 2)\n");
        py.push_str("segs = np.concatenate([pts[:-1], pts[1:]], axis=1)\n");
        py.push_str(&format!(
            "lc = LineCollection(segs, cmap='plasma', norm=Normalize(0, {}), alpha=0.7, linewidths=0.8)\n",
            n_iters.saturating_sub(1)
        ));
        py.push_str(&format!(
            "lc.set_array(np.arange({}))\n",
            n_iters.saturating_sub(1)
        ));
        py.push_str("plt.gca().add_collection(lc)\n");
    }

    py.push_str("plt.colorbar(lc, label='Iteration', shrink=0.8)\n");

    // For small N, add thin crosshair markers at start/end so individual
    // seeds are identifiable.  For large N the colormap gradient suffices.
    if n_seeds <= 200 {
        let init_x: Vec<String> = (0..n_seeds).map(|i| format!("{:.10}", history[0].seeds[i][0])).collect();
        let init_y: Vec<String> = (0..n_seeds).map(|i| format!("{:.10}", history[0].seeds[i][1])).collect();
        py.push_str(&format!(
            "plt.scatter([{}], [{}], c='lightsteelblue', s=6, zorder=5, marker='+', linewidths=0.5, alpha=0.7, label='Initial')\n",
            init_x.join(","), init_y.join(",")
        ));

        let last = history.last().unwrap();
        let final_x: Vec<String> = (0..n_seeds).map(|i| format!("{:.10}", last.seeds[i][0])).collect();
        let final_y: Vec<String> = (0..n_seeds).map(|i| format!("{:.10}", last.seeds[i][1])).collect();
        py.push_str(&format!(
            "plt.scatter([{}], [{}], c='lightyellow', s=6, zorder=5, marker='x', linewidths=0.6, alpha=0.85, label='Final')\n",
            final_x.join(","), final_y.join(",")
        ));

        py.push_str("plt.legend(loc='upper right', fontsize=8, framealpha=0.3)\n");
    }

    append_domain_boundary(&mut py, domain);

    let pad = 0.05 * ((bb_max[0] - bb_min[0]).max(bb_max[1] - bb_min[1]));
    py.push_str(&format!(
        "plt.xlim({:.10}, {:.10})\nplt.ylim({:.10}, {:.10})\n",
        bb_min[0] - pad, bb_max[0] + pad,
        bb_min[1] - pad, bb_max[1] + pad,
    ));
    py.push_str("plt.gca().set_aspect('equal')\n");
    py.push_str("plt.grid(True, alpha=0.15)\n");

    plot.extra(&py);
    plot.set_title(&format!(
        "Seed Trajectories ({} seeds, {} iterations)",
        n_seeds, n_iters
    ));
    plot.set_labels("x", "y");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

// ============================================================================
// Voronoi cell plots
// ============================================================================

/// Plot Voronoi cells colored by an arbitrary per-cell scalar field.
fn plot_voronoi_cells_impl<P: AsRef<Path>>(
    domain: &Domain2D,
    seeds: &[Point2],
    cells: &[Vec<Point2>],
    values: &[Real],
    field_label: &str,
    title: &str,
    path: P,
) -> Result<(), StrError> {
    let (bb_min, bb_max) = domain.bounding_box();

    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 10.0);

    let mut py = String::new();
    py.push_str("from matplotlib.patches import Polygon as MplPolygon\n");
    py.push_str("from matplotlib.collections import PatchCollection\n");
    py.push_str("import numpy as np\n\n");
    py.push_str("patches = []\n");
    py.push_str("vals = []\n");

    for (polygon, &val) in cells.iter().zip(values.iter()) {
        if polygon.len() < 3 {
            continue;
        }
        let verts: Vec<String> = polygon
            .iter()
            .map(|v| format!("({:.10},{:.10})", v[0], v[1]))
            .collect();
        py.push_str(&format!(
            "patches.append(MplPolygon([{}], closed=True))\n",
            verts.join(",")
        ));
        py.push_str(&format!("vals.append({:.10})\n", val));
    }

    py.push_str("pc = PatchCollection(patches, alpha=0.5, edgecolors='white', linewidths=0.5)\n");
    py.push_str("pc.set_array(np.array(vals))\n");
    py.push_str("pc.set_cmap('viridis')\n");
    py.push_str("plt.gca().add_collection(pc)\n");
    py.push_str(&format!(
        "plt.colorbar(pc, label='{}', shrink=0.8)\n",
        field_label
    ));

    let sx: Vec<String> = seeds.iter().map(|s| format!("{:.10}", s[0])).collect();
    let sy: Vec<String> = seeds.iter().map(|s| format!("{:.10}", s[1])).collect();
    py.push_str(&format!(
        "plt.scatter([{}], [{}], c='red', s=6, zorder=5, edgecolors='none')\n",
        sx.join(","), sy.join(",")
    ));

    let pad = 0.05 * ((bb_max[0] - bb_min[0]).max(bb_max[1] - bb_min[1]));
    py.push_str(&format!(
        "plt.xlim({:.10}, {:.10})\nplt.ylim({:.10}, {:.10})\n",
        bb_min[0] - pad, bb_max[0] + pad,
        bb_min[1] - pad, bb_max[1] + pad,
    ));
    py.push_str("plt.gca().set_aspect('equal')\n");

    plot.extra(&py);
    plot.set_title(title);
    plot.set_labels("x", "y");

    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

/// Compute polygon area via the shoelace formula.
fn polygon_area(polygon: &[Point2]) -> Real {
    let n = polygon.len();
    let mut area: Real = 0.0;
    for i in 0..n {
        let j = (i + 1) % n;
        area += polygon[i][0] * polygon[j][1];
        area -= polygon[j][0] * polygon[i][1];
    }
    area.abs() * 0.5
}

/// Plot Voronoi cells colored by geometric area.
pub fn plot_voronoi_cells_2d<P: AsRef<Path>>(
    domain: &Domain2D,
    seeds: &[Point2],
    path: P,
) -> Result<(), StrError> {
    if seeds.is_empty() {
        return Ok(());
    }
    let cells = domain.voronoi_cells(seeds);
    let areas: Vec<Real> = cells.iter().map(|p| polygon_area(p)).collect();
    plot_voronoi_cells_impl(
        domain,
        seeds,
        &cells,
        &areas,
        "Cell Area",
        &format!("Voronoi Cells — Area ({} seeds)", seeds.len()),
        path,
    )
}

/// Plot Voronoi cells colored by density-weighted mass.
pub fn plot_voronoi_cells_2d_mass<F, P>(
    domain: &Domain2D,
    seeds: &[Point2],
    density: F,
    path: P,
) -> Result<(), StrError>
where
    F: Fn(Point2) -> Real,
    P: AsRef<Path>,
{
    if seeds.is_empty() {
        return Ok(());
    }
    let cells = domain.voronoi_cells(seeds);
    let data = domain.integrate_cells(seeds, &density);
    plot_voronoi_cells_impl(
        domain,
        seeds,
        &cells,
        data.masses.as_slice(),
        "Cell Mass",
        &format!("Voronoi Cells — Mass ({} seeds)", seeds.len()),
        path,
    )
}

// ============================================================================
// Animated GIF of Voronoi cell evolution
// ============================================================================

/// Create an animated GIF showing the Voronoi cell evolution over iterations.
///
/// Each frame renders the clipped Voronoi cells colored by mass at that
/// iteration, with seed positions overlaid. Uses matplotlib's FuncAnimation
/// and Pillow writer.
///
/// `frame_step` controls how many Lloyd iterations to skip between frames
/// (e.g. 5 means every 5th iteration becomes a frame).
pub fn animate_voronoi_evolution<F, P>(
    domain: &Domain2D,
    history: &[CvtState<Point2>],
    density: F,
    path: P,
    frame_step: usize,
) -> Result<(), StrError>
where
    F: Fn(Point2) -> Real,
    P: AsRef<Path>,
{
    if history.is_empty() {
        return Ok(());
    }

    let frames: Vec<&CvtState<Point2>> = history
        .iter()
        .step_by(frame_step.max(1))
        .collect();
    let n_frames = frames.len();
    let (bb_min, bb_max) = domain.bounding_box();
    let pad = 0.05 * ((bb_max[0] - bb_min[0]).max(bb_max[1] - bb_min[1]));

    let out = path.as_ref().to_str().unwrap_or("animation.gif");
    let script_path = format!("{}.py", out.trim_end_matches(".gif"));

    let mut py = String::new();
    py.push_str("import numpy as np\n");
    py.push_str("import matplotlib\n");
    py.push_str("matplotlib.use('Agg')\n");
    py.push_str("import matplotlib.pyplot as plt\n");
    py.push_str("from matplotlib.patches import Polygon as MplPolygon\n");
    py.push_str("from matplotlib.collections import PatchCollection\n");
    py.push_str("from matplotlib.animation import FuncAnimation\n");
    py.push_str("plt.style.use('dark_background')\n\n");
    py.push_str("fig, ax = plt.subplots(1, 1, figsize=(10, 10))\n\n");

    py.push_str("all_frames = []\n");

    for state in &frames {
        let seeds = state.seeds.as_slice();
        let cells = domain.voronoi_cells(seeds);
        let data = domain.integrate_cells(seeds, &density);

        py.push_str("fp = []\nfm = []\n");

        for (polygon, &mass) in cells.iter().zip(data.masses.as_slice().iter()) {
            if polygon.len() < 3 {
                continue;
            }
            let verts: Vec<String> = polygon
                .iter()
                .map(|v| format!("({:.8},{:.8})", v[0], v[1]))
                .collect();
            py.push_str(&format!("fp.append([{}])\n", verts.join(",")));
            py.push_str(&format!("fm.append({:.8})\n", mass));
        }

        let sx: Vec<String> = seeds.iter().map(|s| format!("{:.8}", s[0])).collect();
        let sy: Vec<String> = seeds.iter().map(|s| format!("{:.8}", s[1])).collect();

        py.push_str(&format!(
            "all_frames.append((fp, fm, [{}], [{}], {}))\n",
            sx.join(","), sy.join(","), state.iteration
        ));
    }

    py.push_str("all_masses = [m for f in all_frames for m in f[1]]\n");
    py.push_str("vmin, vmax = min(all_masses), max(all_masses)\n\n");

    py.push_str("def update(i):\n");
    py.push_str("    ax.clear()\n");
    py.push_str("    verts_list, masses, sx, sy, it = all_frames[i]\n");
    py.push_str("    patches = [MplPolygon(v, closed=True) for v in verts_list]\n");
    py.push_str("    pc = PatchCollection(patches, alpha=0.6, edgecolors='white', linewidths=0.5)\n");
    py.push_str("    pc.set_array(np.array(masses))\n");
    py.push_str("    pc.set_cmap('viridis')\n");
    py.push_str("    pc.set_clim(vmin, vmax)\n");
    py.push_str("    ax.add_collection(pc)\n");
    py.push_str("    ax.scatter(sx, sy, c='red', s=6, zorder=5, edgecolors='none')\n");
    py.push_str(&format!(
        "    ax.set_xlim({:.8}, {:.8})\n    ax.set_ylim({:.8}, {:.8})\n",
        bb_min[0] - pad, bb_max[0] + pad,
        bb_min[1] - pad, bb_max[1] + pad,
    ));
    py.push_str("    ax.set_aspect('equal')\n");
    py.push_str("    ax.set_xlabel('x')\n    ax.set_ylabel('y')\n");
    py.push_str("    ax.set_title('Voronoi Cells — Iter %d' % it)\n");
    py.push_str("    return []\n\n");

    py.push_str(&format!(
        "anim = FuncAnimation(fig, update, frames={}, interval=100, blit=False)\n",
        n_frames
    ));
    py.push_str(&format!(
        "anim.save('{}', writer='pillow', fps=10, dpi=200)\n",
        out
    ));
    py.push_str("plt.close(fig)\n");
    py.push_str(&format!("print('Saved {}')\n", out));

    std::fs::write(&script_path, &py).map_err(|_| "failed to write animation script")?;

    let output = std::process::Command::new("python3")
        .arg(&script_path)
        .output()
        .map_err(|_| "failed to run python3")?;

    if !output.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        return Err("python3 animation script failed");
    }

    Ok(())
}

// ============================================================================
// Helpers
// ============================================================================

// ============================================================================
// Scalability throughput plot
// ============================================================================

/// Log-log plot of Lloyd iteration throughput (ms/iter vs N) for one or more
/// density configurations.
///
/// `series` is a slice of `(label, sizes, ms_per_iter)` tuples — one per
/// density configuration.  Reference O(N) and O(N log N) guide-lines are
/// drawn automatically, anchored to the first data point of the first series.
pub fn plot_scalability_throughput<P: AsRef<Path>>(
    series: &[(&str, &[usize], &[f64])],
    path: P,
) -> Result<(), StrError> {
    let mut plot = styled_plot();
    plot.set_figure_size_inches(10.0, 7.0);

    let mut py = String::new();
    py.push_str("import numpy as np\n\n");

    let colors = ["dodgerblue", "tomato", "limegreen", "gold"];

    for (idx, (label, sizes, ms)) in series.iter().enumerate() {
        let xs: Vec<String> = sizes.iter().map(|n| n.to_string()).collect();
        let ys: Vec<String> = ms.iter().map(|m| format!("{:.6}", m)).collect();
        let col = colors[idx % colors.len()];
        py.push_str(&format!("xs_{idx} = [{}]\n", xs.join(",")));
        py.push_str(&format!("ys_{idx} = [{}]\n", ys.join(",")));
        py.push_str(&format!(
            "plt.loglog(xs_{idx}, ys_{idx}, 'o-', color='{col}', linewidth=1.8, \
             markersize=5, markerfacecolor='none', markeredgewidth=1.2, label='{label}')\n"
        ));
    }

    // Guide lines anchored to first point of first series.
    if let Some((_, sizes, ms)) = series.first() {
        if !sizes.is_empty() {
            let n0 = sizes[0] as f64;
            let m0 = ms[0];
            let n_max = *sizes.last().unwrap() as f64;
            py.push_str(&format!(
                "n_ref = np.logspace(np.log10({n0}), np.log10({n_max}), 200)\n"
            ));
            py.push_str(&format!(
                "plt.loglog(n_ref, {m0} * (n_ref/{n0}), \
                 'w--', linewidth=0.8, alpha=0.4, label='O(N)')\n"
            ));
            py.push_str(&format!(
                "plt.loglog(n_ref, {m0} * (n_ref/{n0}) * np.log2(n_ref/n_ref[0]+1), \
                 'w:', linewidth=0.8, alpha=0.4, label='O(N log N)')\n"
            ));
        }
    }

    py.push_str("plt.xlabel('N  (seeds)', fontsize=12)\n");
    py.push_str("plt.ylabel('ms / iteration', fontsize=12)\n");
    py.push_str("plt.legend(fontsize=10, framealpha=0.3)\n");
    py.push_str("plt.grid(True, which='both', alpha=0.15)\n");

    plot.extra(&py);
    plot.set_title("Lloyd iteration throughput");
    plot.save(path.as_ref().to_str().unwrap_or("plot.png"))?;
    Ok(())
}

fn append_domain_boundary(py: &mut String, domain: &Domain2D) {
    let bverts = domain.boundary_vertices();
    if !bverts.is_empty() {
        let bx: Vec<String> = bverts
            .iter()
            .chain(std::iter::once(&bverts[0]))
            .map(|v| format!("{:.10}", v[0]))
            .collect();
        let by: Vec<String> = bverts
            .iter()
            .chain(std::iter::once(&bverts[0]))
            .map(|v| format!("{:.10}", v[1]))
            .collect();
        py.push_str(&format!(
            "plt.plot([{}], [{}], 'w-', linewidth=1.5, alpha=0.6)\n",
            bx.join(","),
            by.join(",")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::meshgen::cvt::lloyd_iter;
    use std::fs;

    #[test]
    fn trajectory_plot_generates_file() {
        let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
        let seeds = domain.uniform_seeds(10);
        let history: Vec<_> = lloyd_iter(domain.clone(), seeds, |_| 1.0)
            .take(20)
            .collect();

        let path = "/tmp/strelitzia_test_trajectories_2d.png";
        let result = plot_seed_trajectories_2d(&domain, &history, path);

        if result.is_ok() {
            assert!(fs::metadata(path).is_ok());
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn voronoi_area_plot_generates_file() {
        let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
        let seeds = domain.uniform_seeds(15);
        let final_state: CvtState<Point2> = lloyd_iter(domain.clone(), seeds, |_| 1.0)
            .nth(30)
            .unwrap();

        let path = "/tmp/strelitzia_test_voronoi_area_2d.png";
        let result = plot_voronoi_cells_2d(&domain, final_state.seeds.as_slice(), path);

        if result.is_ok() {
            assert!(fs::metadata(path).is_ok());
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn voronoi_mass_plot_generates_file() {
        let domain = Domain2D::rectangle(0.0, 1.0, 0.0, 1.0);
        let seeds = domain.uniform_seeds(15);
        let final_state: CvtState<Point2> = lloyd_iter(domain.clone(), seeds, |_| 1.0)
            .nth(30)
            .unwrap();

        let path = "/tmp/strelitzia_test_voronoi_mass_2d.png";
        let result = plot_voronoi_cells_2d_mass(
            &domain,
            final_state.seeds.as_slice(),
            |_| 1.0,
            path,
        );

        if result.is_ok() {
            assert!(fs::metadata(path).is_ok());
            let _ = fs::remove_file(path);
        }
    }
}
