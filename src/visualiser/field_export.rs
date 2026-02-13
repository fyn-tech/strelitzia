//! Field data export utilities for VTK visualization.
//!
//! Provides helper functions to convert field storage to VTK-compatible
//! arrays for visualization in ParaView.
//!
//! # Example
//!
//! ```no_run
//! use strelitzia::multiarray::Vector3;
//! use strelitzia::fields::*;
//! use strelitzia::visualiser::*;
//!
//! let mut temperature = ScalarField::new();
//! temperature.push(25.0);
//! temperature.push(30.0);
//!
//! let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
//! let temp_vtk = scalar_field_to_vtk_array("temperature", &temperature);
//!
//! write_vtu::<_, 3>(
//!     "output.vtu",
//!     &points,
//!     None,
//!     None,
//!     &[temp_vtk],
//!     &[],
//!     Encoding::Base64,
//! )?;
//! # Ok::<(), std::io::Error>(())
//! ```

use crate::fields::{ScalarField, Vector3Field, SolverInterop};
use super::FieldArray;

/// Convert ScalarField to VTK FieldArray for visualization.
///
/// # Arguments
/// - `name`: Field name displayed in ParaView
/// - `field`: The scalar field data
///
/// # Example
/// ```
/// # use strelitzia::fields::*;
/// # use strelitzia::visualiser::*;
/// let mut temp = ScalarField::new();
/// temp.push(25.0);
/// temp.push(30.0);
/// let vtk_array = scalar_field_to_vtk_array("temperature", &temp);
/// ```
pub fn scalar_field_to_vtk_array<'a>(
    name: &'a str,
    field: &'a ScalarField,
) -> FieldArray<'a> {
    FieldArray::from_slice(name, field.as_slice(), 1)
}

/// Convert Vector3Field to VTK FieldArray for visualization.
///
/// Uses the flat slice representation (x,y,z interleaved) required by VTK.
///
/// # Arguments
/// - `name`: Field name displayed in ParaView
/// - `field`: The vector field data
///
/// # Example
/// ```
/// # use strelitzia::multiarray::Vector3;
/// # use strelitzia::fields::*;
/// # use strelitzia::visualiser::*;
/// let mut velocity = Vector3Field::new();
/// velocity.push(Vector3::new(1.0, 0.0, 0.0));
/// let vtk_array = vector3_field_to_vtk_array("velocity", &velocity);
/// ```
pub fn vector3_field_to_vtk_array<'a>(
    name: &'a str,
    field: &'a Vector3Field,
) -> FieldArray<'a> {
    FieldArray::from_slice(name, field.as_flat_slice(), 3)
}
