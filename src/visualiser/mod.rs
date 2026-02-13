//! Visualization utilities for mesh data.
//!
//! Provides writers for various visualization formats.

mod encoding;
mod field_export;
pub mod paraview_writer;
pub mod vtk_types;

pub use field_export::{scalar_field_to_vtk_array, vector3_field_to_vtk_array};
pub use paraview_writer::{write_pvd, write_vtu};
pub use vtk_types::{CellType, Encoding, FieldArray};
