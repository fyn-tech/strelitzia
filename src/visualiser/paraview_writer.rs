//! VTK XML UnstructuredGrid writer for ParaView visualization.
//!
//! Provides a minimalistic API for writing mesh data to VTK XML format (.vtu files).
//! Supports point clouds (auto-inferred as VTK_VERTEX cells) and meshes with explicit topology.
//!
//! # Example
//!
//! ```no_run
//! use strelitzia::visualiser::*;
//!
//! // Write a point cloud (auto-infers VTK_VERTEX cells)
//! let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
//! write_vtu::<_, 3>("cloud.vtu", &points, None, None, &[], &[], Encoding::Base64)?;
//!
//! // Write a triangle mesh
//! let connectivity = vec![vec![0, 1, 2], vec![1, 2, 3]];
//! let cell_types = vec![CellType::Triangle, CellType::Triangle];
//! write_vtu::<_, 3>("mesh.vtu", &points, Some(&connectivity), Some(&cell_types), &[], &[], Encoding::Base64)?;
//! # Ok::<(), std::io::Error>(())
//! ```

use crate::visualiser::encoding;
use crate::visualiser::vtk_types::{CellType, Encoding, FieldArray, VTKCellType};
use bytemuck::Pod;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

// ASCII formatting constants for line wrapping
const POINTS_PER_LINE: usize = 2;
const CONNECTIVITY_PER_LINE: usize = 10;
const OFFSETS_PER_LINE: usize = 10;
const CELL_TYPES_PER_LINE: usize = 20;
const FIELD_VALUES_PER_LINE: usize = 6;

/// Write mesh data to VTK XML UnstructuredGrid format (.vtu).
///
/// # Arguments
/// - `path`: Output file path
/// - `points`: Point coordinates (must be `Pod`, e.g., `[f64; 2]` or `[f64; 3]`)
/// - `connectivity`: Cell connectivity (`None` = auto-generate VTK_VERTEX for point clouds)
/// - `cell_types`: Cell types (must match connectivity length)
/// - `point_fields`: Field data on points
/// - `cell_fields`: Field data on cells
/// - `encoding`: `Encoding::Ascii` or `Encoding::Base64`
///
/// # Examples
///
/// Point cloud:
/// ```no_run
/// # use strelitzia::visualiser::*;
/// # let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
/// write_vtu::<_, 3>("cloud.vtu", &points, None, None, &[], &[], Encoding::Base64)?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// Triangle mesh:
/// ```no_run
/// # use strelitzia::visualiser::*;
/// # let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
/// let connectivity = vec![vec![0, 1, 2]];
/// let cell_types = vec![CellType::Triangle];
/// write_vtu::<_, 3>("mesh.vtu", &points, Some(&connectivity), Some(&cell_types), &[], &[], Encoding::Base64)?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// With fields:
/// ```no_run
/// # use strelitzia::visualiser::*;
/// # let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0]];
/// # let conn = vec![vec![0]];
/// # let types = vec![CellType::Vertex];
/// # let temp_data: Vec<f64> = vec![25.0];
/// let temp = FieldArray::from_slice("temperature", &temp_data, 1);
/// write_vtu::<_, 3>("mesh.vtu", &points, Some(&conn), Some(&types), &[temp], &[], Encoding::Base64)?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn write_vtu<V, const DIM: usize>(
    path: impl AsRef<Path>,
    points: &[V],
    connectivity: Option<&[Vec<usize>]>,
    cell_types: Option<&[CellType]>,
    point_fields: &[FieldArray],
    cell_fields: &[FieldArray],
    encoding: Encoding,
) -> io::Result<()>
where
    V: Pod,
{
    // Runtime size check: V must be DIM * f64
    if std::mem::size_of::<V>() != DIM * std::mem::size_of::<f64>() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Point type size ({}) must match DIM * sizeof(f64) ({})",
                std::mem::size_of::<V>(),
                DIM * std::mem::size_of::<f64>()
            ),
        ));
    }

    let mut file = File::create(path)?;
    let writer = VtkWriter {
        file: &mut file,
        encoding,
    };

    writer.write_unstructured_grid::<V, DIM>(
        points,
        connectivity,
        cell_types,
        point_fields,
        cell_fields,
    )
}

/// Write ParaView Data (.pvd) collection file for time series.
///
/// Creates an XML file that references multiple .vtu files with timesteps,
/// enabling time series visualization and animation in ParaView.
///
/// # Arguments
/// - `path`: Output .pvd file path
/// - `entries`: Vector of (timestep, relative_vtu_path) tuples
///
/// Paths should be relative to the .pvd file location for portability.
/// Absolute paths are also supported for cross-directory references.
///
/// # Example
///
/// ```no_run
/// use strelitzia::visualiser::write_pvd;
///
/// let entries = vec![
///     (0.0, "output_0000.vtu"),
///     (0.1, "output_0001.vtu"),
///     (0.2, "output_0002.vtu"),
/// ];
/// write_pvd("simulation.pvd", &entries)?;
/// # Ok::<(), std::io::Error>(())
/// ```
///
/// # PVD Format
///
/// The generated file follows the ParaView Data Collection format:
///
/// ```xml
/// <?xml version="1.0"?>
/// <VTKFile type="Collection" version="0.1">
///   <Collection>
///     <DataSet timestep="0.0" file="output_0000.vtu"/>
///     <DataSet timestep="0.1" file="output_0001.vtu"/>
///   </Collection>
/// </VTKFile>
/// ```
pub fn write_pvd(
    path: impl AsRef<Path>,
    entries: &[(f64, impl AsRef<str>)],
) -> io::Result<()> {
    let mut file = File::create(path)?;
    
    writeln!(file, r#"<?xml version="1.0"?>"#)?;
    writeln!(file, r#"<VTKFile type="Collection" version="0.1">"#)?;
    writeln!(file, "  <Collection>")?;
    
    for (timestep, vtu_path) in entries {
        // XML escape the file path
        let escaped_path = vtu_path.as_ref()
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;");
        
        // Format timestep to always show decimal point
        writeln!(
            file,
            r#"    <DataSet timestep="{}" file="{}"/>"#,
            timestep,
            escaped_path
        )?;
    }
    
    writeln!(file, "  </Collection>")?;
    writeln!(file, "</VTKFile>")?;
    
    Ok(())
}

/// Internal writer implementation.
struct VtkWriter<'a> {
    file: &'a mut File,
    encoding: Encoding,
}

impl<'a> VtkWriter<'a> {
    /// Write complete UnstructuredGrid file.
    fn write_unstructured_grid<V, const DIM: usize>(
        mut self,
        points: &[V],
        connectivity: Option<&[Vec<usize>]>,
        cell_types: Option<&[CellType]>,
        point_fields: &[FieldArray],
        cell_fields: &[FieldArray],
    ) -> io::Result<()>
    where
        V: Pod,
    {
        // Validate connectivity and cell_types match
        match (connectivity, cell_types) {
            (Some(_), None) | (None, Some(_)) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "connectivity and cell_types must both be Some or both be None",
                ));
            }
            _ => {}
        }

        // Determine cell data (auto-infer vertices if not provided)
        let (conn, types): (Vec<Vec<usize>>, Vec<VTKCellType>) = match (connectivity, cell_types) {
            (Some(c), Some(t)) => {
                // Validate connectivity and cell_types have same length
                if c.len() != t.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!(
                            "connectivity length ({}) must match cell_types length ({})",
                            c.len(),
                            t.len()
                        ),
                    ));
                }
                
                // Validate all connectivity indices are within bounds
                let num_points = points.len();
                for (cell_idx, cell) in c.iter().enumerate() {
                    for &point_idx in cell {
                        if point_idx >= num_points {
                            return Err(io::Error::new(
                                io::ErrorKind::InvalidInput,
                                format!(
                                    "Connectivity index {} in cell {} is out of bounds (max: {})",
                                    point_idx, cell_idx, num_points - 1
                                ),
                            ));
                        }
                    }
                }
                
                (c.to_vec(), t.iter().map(|&ct| ct.into()).collect())
            }
            (None, None) => {
                // Auto-generate vertex cells
                let vertex_conn: Vec<Vec<usize>> = (0..points.len()).map(|i| vec![i]).collect();
                let vertex_types = vec![VTKCellType::Vertex; points.len()];
                (vertex_conn, vertex_types)
            }
            _ => unreachable!(),
        };

        let num_cells = conn.len();
        
        // Validate point field lengths
        for field in point_fields {
            let expected_bytes = points.len() * field.num_components * std::mem::size_of::<f64>();
            let actual_bytes = field.data.len();
            if actual_bytes != expected_bytes {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Point field '{}' has wrong length: expected {} bytes ({} points × {} components × {} bytes/f64), got {} bytes",
                        field.name, expected_bytes, points.len(), field.num_components, std::mem::size_of::<f64>(), actual_bytes
                    ),
                ));
            }
        }
        
        // Validate cell field lengths
        for field in cell_fields {
            let expected_bytes = num_cells * field.num_components * std::mem::size_of::<f64>();
            let actual_bytes = field.data.len();
            if actual_bytes != expected_bytes {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Cell field '{}' has wrong length: expected {} bytes ({} cells × {} components × {} bytes/f64), got {} bytes",
                        field.name, expected_bytes, num_cells, field.num_components, std::mem::size_of::<f64>(), actual_bytes
                    ),
                ));
            }
        }

        // Write XML header
        writeln!(self.file, r#"<?xml version="1.0"?>"#)?;
        writeln!(
            self.file,
            r#"<VTKFile type="UnstructuredGrid" version="1.0" byte_order="LittleEndian">"#
        )?;
        writeln!(self.file, "  <UnstructuredGrid>")?;
        writeln!(
            self.file,
            r#"    <Piece NumberOfPoints="{}" NumberOfCells="{}">"#,
            points.len(),
            num_cells
        )?;

        // Write point data fields
        self.write_point_data(point_fields)?;

        // Write cell data fields
        self.write_cell_data(cell_fields)?;

        // Write points
        self.write_points::<V, DIM>(points)?;

        // Write cells
        self.write_cells(&conn, &types)?;

        // Close tags
        writeln!(self.file, "    </Piece>")?;
        writeln!(self.file, "  </UnstructuredGrid>")?;
        writeln!(self.file, "</VTKFile>")?;

        Ok(())
    }

    /// Write PointData section.
    fn write_point_data(&mut self, fields: &[FieldArray]) -> io::Result<()> {
        if fields.is_empty() {
            writeln!(self.file, "      <PointData/>")?;
        } else {
            writeln!(self.file, "      <PointData>")?;
            for field in fields {
                self.write_data_array(field)?;
            }
            writeln!(self.file, "      </PointData>")?;
        }
        Ok(())
    }

    /// Write CellData section.
    fn write_cell_data(&mut self, fields: &[FieldArray]) -> io::Result<()> {
        if fields.is_empty() {
            writeln!(self.file, "      <CellData/>")?;
        } else {
            writeln!(self.file, "      <CellData>")?;
            for field in fields {
                self.write_data_array(field)?;
            }
            writeln!(self.file, "      </CellData>")?;
        }
        Ok(())
    }

    /// Write Points section.
    ///
    /// VTK always requires 3D coordinates (NumberOfComponents=3).
    /// For 2D data, z-coordinates are padded with 0.0.
    fn write_points<V, const DIM: usize>(&mut self, points: &[V]) -> io::Result<()>
    where
        V: Pod,
    {
        writeln!(self.file, "      <Points>")?;

        // Convert points to f64 slice
        let coords: &[f64] = bytemuck::cast_slice(points);

        // VTK always requires 3 components for points
        write!(
            self.file,
            r#"        <DataArray type="Float64" NumberOfComponents="3" format="{}""#,
            match self.encoding {
                Encoding::Ascii => "ascii",
                Encoding::Base64 => "binary",
            }
        )?;

        match self.encoding {
            Encoding::Ascii => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;

                // Write coordinates, padding to 3D if needed
                let num_points = points.len();
                for i in 0..num_points {
                    if i > 0 && i % POINTS_PER_LINE == 0 {
                        write!(self.file, "\n          ")?;
                    }

                    // Write x, y, (z if 3D, else 0)
                    for d in 0..DIM {
                        write!(self.file, "{} ", coords[i * DIM + d])?;
                    }
                    // Pad with zeros for missing dimensions
                    for _ in DIM..3 {
                        write!(self.file, "0 ")?;
                    }
                }
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
            Encoding::Base64 => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;

                // For Base64, we need to expand to 3D coordinates
                if DIM == 3 {
                    // Already 3D, write directly
                    self.write_base64_f64(coords)?;
                } else {
                    // Expand to 3D by padding with zeros
                    let num_points = points.len();
                    let mut coords_3d = Vec::with_capacity(num_points * 3);
                    for i in 0..num_points {
                        coords_3d.extend_from_slice(&coords[i * DIM..(i + 1) * DIM]);
                        coords_3d.extend(std::iter::repeat_n(0.0, 3 - DIM));
                    }
                    self.write_base64_f64(&coords_3d)?;
                }

                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
        }

        writeln!(self.file, "      </Points>")?;
        Ok(())
    }

    /// Write Cells section (connectivity, offsets, types).
    fn write_cells(
        &mut self,
        connectivity: &[Vec<usize>],
        types: &[VTKCellType],
    ) -> io::Result<()> {
        writeln!(self.file, "      <Cells>")?;

        // Write connectivity array (on-the-fly flattening)
        write!(
            self.file,
            r#"        <DataArray type="Int32" Name="connectivity" format="{}""#,
            match self.encoding {
                Encoding::Ascii => "ascii",
                Encoding::Base64 => "binary",
            }
        )?;

        match self.encoding {
            Encoding::Ascii => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                let mut count = 0;
                for cell in connectivity {
                    for &idx in cell {
                        if count > 0 && count % CONNECTIVITY_PER_LINE == 0 {
                            write!(self.file, "\n          ")?;
                        }
                        write!(self.file, "{} ", idx)?;
                        count += 1;
                    }
                }
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
            Encoding::Base64 => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                let flat: Vec<i32> = connectivity
                    .iter()
                    .flat_map(|cell| cell.iter().map(|&i| i as i32))
                    .collect();
                self.write_base64_i32(&flat)?;
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
        }

        // Write offsets array (on-the-fly computation)
        write!(
            self.file,
            r#"        <DataArray type="Int32" Name="offsets" format="{}""#,
            match self.encoding {
                Encoding::Ascii => "ascii",
                Encoding::Base64 => "binary",
            }
        )?;

        match self.encoding {
            Encoding::Ascii => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                let mut offset = 0;
                for (i, cell) in connectivity.iter().enumerate() {
                    offset += cell.len();
                    if i > 0 && i % OFFSETS_PER_LINE == 0 {
                        write!(self.file, "\n          ")?;
                    }
                    write!(self.file, "{} ", offset)?;
                }
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
            Encoding::Base64 => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                let mut offset = 0;
                let offsets: Vec<i32> = connectivity
                    .iter()
                    .map(|cell| {
                        offset += cell.len() as i32;
                        offset
                    })
                    .collect();
                self.write_base64_i32(&offsets)?;
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
        }

        // Write types array
        write!(
            self.file,
            r#"        <DataArray type="UInt8" Name="types" format="{}""#,
            match self.encoding {
                Encoding::Ascii => "ascii",
                Encoding::Base64 => "binary",
            }
        )?;

        match self.encoding {
            Encoding::Ascii => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                for (i, &cell_type) in types.iter().enumerate() {
                    if i > 0 && i % CELL_TYPES_PER_LINE == 0 {
                        write!(self.file, "\n          ")?;
                    }
                    write!(self.file, "{} ", cell_type.as_u8())?;
                }
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
            Encoding::Base64 => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                let type_ids: Vec<u8> = types.iter().map(|&t| t.as_u8()).collect();
                self.write_base64_u8(&type_ids)?;
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
        }

        writeln!(self.file, "      </Cells>")?;
        Ok(())
    }

    /// Write a generic DataArray for field data.
    fn write_data_array(&mut self, field: &FieldArray) -> io::Result<()> {
        write!(
            self.file,
            r#"        <DataArray type="Float64" Name="{}" NumberOfComponents="{}" format="{}""#,
            field.name,
            field.num_components,
            match self.encoding {
                Encoding::Ascii => "ascii",
                Encoding::Base64 => "binary",
            }
        )?;

        let values: &[f64] = bytemuck::cast_slice(field.data);

        match self.encoding {
            Encoding::Ascii => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                for (i, &val) in values.iter().enumerate() {
                    if i > 0 && i % FIELD_VALUES_PER_LINE == 0 {
                        write!(self.file, "\n          ")?;
                    }
                    write!(self.file, "{} ", val)?;
                }
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
            Encoding::Base64 => {
                writeln!(self.file, ">")?;
                write!(self.file, "          ")?;
                self.write_base64_f64(values)?;
                writeln!(self.file)?;
                writeln!(self.file, "        </DataArray>")?;
            }
        }

        Ok(())
    }

    /// Write f64 array as base64 with VTK header.
    ///
    /// VTK binary format requires a 4-byte header containing the data size in bytes.
    fn write_base64_f64(&mut self, data: &[f64]) -> io::Result<()> {
        let bytes: &[u8] = bytemuck::cast_slice(data);
        self.write_base64_with_header(bytes)?;
        Ok(())
    }

    /// Write i32 array as base64 with VTK header.
    fn write_base64_i32(&mut self, data: &[i32]) -> io::Result<()> {
        let bytes: &[u8] = bytemuck::cast_slice(data);
        self.write_base64_with_header(bytes)?;
        Ok(())
    }

    /// Write u8 array as base64 with VTK header.
    fn write_base64_u8(&mut self, data: &[u8]) -> io::Result<()> {
        self.write_base64_with_header(data)?;
        Ok(())
    }

    /// Write data as base64 with VTK 4-byte size header.
    fn write_base64_with_header(&mut self, data: &[u8]) -> io::Result<()> {
        // VTK binary format: [4-byte size header][actual data]
        let size = data.len() as u32;
        let size_bytes = size.to_le_bytes();

        // Combine header and data
        let mut full_data = Vec::with_capacity(4 + data.len());
        full_data.extend_from_slice(&size_bytes);
        full_data.extend_from_slice(data);

        // Encode to base64
        let encoded = encoding::encode(&full_data);
        write!(self.file, "{}", encoded)?;
        Ok(())
    }
}
