//! Visualization utilities for mesh data.
//!
//! Provides writers for various visualization formats.

mod encoding;
pub mod paraview_writer;
pub mod vtk_types;

pub use paraview_writer::write_vtu;
pub use vtk_types::{CellType, Encoding, FieldArray};

