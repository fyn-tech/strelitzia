//! VTK type definitions for mesh visualization.

use bytemuck::Pod;

/// Cell topology types.
///
/// | Type | VTK ID | Description |
/// |------|--------|-------------|
/// | `Vertex` | 1 | Point |
/// | `Edge` | 3 | Line segment |
/// | `EdgeChain` | 4 | Polyline |
/// | `Triangle` | 5 | Triangle |
/// | `Polygon` | 7 | N-sided polygon |
/// | `Quad` | 9 | Quadrilateral |
/// | `Tetra` | 10 | Tetrahedron |
/// | `Hexa` | 12 | Hexahedron |
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Vertex,
    Edge,
    EdgeChain,
    Triangle,
    Polygon,
    Quad,
    Tetra,
    Hexa,
}

/// Internal VTK cell type IDs.
///
/// These correspond to the VTK specification's cell type codes.
/// Marked as `pub(crate)` for use within the visualiser module only.
#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub(crate) enum VTKCellType {
    Vertex = 1,
    Line = 3,
    PolyLine = 4,
    Triangle = 5,
    Polygon = 7,
    Quad = 9,
    Tetra = 10,
    Hexahedron = 12,
}

impl From<CellType> for VTKCellType {
    #[inline]
    fn from(cell_type: CellType) -> Self {
        match cell_type {
            CellType::Vertex => VTKCellType::Vertex,
            CellType::Edge => VTKCellType::Line,
            CellType::EdgeChain => VTKCellType::PolyLine,
            CellType::Triangle => VTKCellType::Triangle,
            CellType::Polygon => VTKCellType::Polygon,
            CellType::Quad => VTKCellType::Quad,
            CellType::Tetra => VTKCellType::Tetra,
            CellType::Hexa => VTKCellType::Hexahedron,
        }
    }
}

impl VTKCellType {
    #[inline]
    pub(crate) const fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Data encoding format.
///
/// - `Ascii`: Human-readable text (~40% larger)
/// - `Base64`: Binary encoding (recommended)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Ascii,
    Base64,
}

/// Field data array for point or cell data.
///
/// # Components
/// - `1`: Scalar (temperature, pressure)
/// - `3`: Vector (velocity, force)
/// - `9`: Tensor (3×3 stress matrix)
pub struct FieldArray<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
    pub num_components: usize,
}

impl<'a> FieldArray<'a> {
    /// Create field array from Pod-compatible data.
    ///
    /// # Example
    /// ```
    /// use strelitzia::visualiser::FieldArray;
    ///
    /// let temp: Vec<f64> = vec![0.0, 1.0, 2.0];
    /// let field = FieldArray::from_slice("temperature", &temp, 1);
    /// ```
    pub fn from_slice<T: Pod>(name: &'a str, data: &'a [T], num_components: usize) -> Self {
        Self {
            name,
            data: bytemuck::cast_slice(data),
            num_components,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unit test for cell type conversions
    #[test]
    fn test_cell_type_conversions() {
        assert_eq!(VTKCellType::from(CellType::Vertex) as u8, 1);
        assert_eq!(VTKCellType::from(CellType::Edge) as u8, 3);
        assert_eq!(VTKCellType::from(CellType::EdgeChain) as u8, 4);
        assert_eq!(VTKCellType::from(CellType::Triangle) as u8, 5);
        assert_eq!(VTKCellType::from(CellType::Polygon) as u8, 7);
        assert_eq!(VTKCellType::from(CellType::Quad) as u8, 9);
        assert_eq!(VTKCellType::from(CellType::Tetra) as u8, 10);
        assert_eq!(VTKCellType::from(CellType::Hexa) as u8, 12);
    }
}
