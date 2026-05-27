//! Mesh generation algorithms.
//!
//! This module provides:
//! - [`mesh`] -- Universal mesh type (`Mesh<P>`) with tensor-product, extrusion, and Delaunay constructors
//! - [`cvt`] -- Centroidal Voronoi Tessellation (CVT) algorithms
//! - [`cvt_solvers`] -- Newton/BFGS/L-BFGS optimization for CVT
//! - [`cvt_plot`] -- Matplotlib-based visualization for 1D CVT results
//! - [`cvt_plot_2d`] -- Matplotlib-based visualization for 2D CVT results
//! - [`cvt_vtk`] -- VTK/ParaView export for 1D CVT results
//! - [`cvt_vtk_2d`] -- VTK/ParaView export for 2D CVT results

pub mod mesh;

pub mod cvt;
pub mod cvt_plot;
pub mod cvt_plot_2d;
pub mod cvt_solvers;
pub mod cvt_vtk;
pub mod cvt_vtk_2d;
