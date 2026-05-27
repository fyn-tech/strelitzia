//! Mesh representation and constructors.
//!
//! [`Mesh<P>`] is the universal output type for all mesh generators.
//! The point type `P` provides compile-time dimension safety:
//! - `Mesh<Real>` -- 1D mesh
//! - `Mesh<Point2>` -- 2D mesh
//! - `Mesh<Point3>` -- 3D mesh

use crate::common::Real;
use crate::fields::Field;
use crate::multiarray::{Point2, Point3};
use crate::visualiser::CellType;

use spade::{DelaunayTriangulation, HasPosition, Triangulation};

/// A mesh consisting of vertices and cells with explicit connectivity.
///
/// This is the common output of all mesh generators (CVT, tensor product,
/// Delaunay, etc.) and the common input to downstream consumers (VTK export,
/// FVM solvers, field interpolation).
#[derive(Debug, Clone)]
pub struct Mesh<P> {
    pub vertices: Field<P>,
    pub cells: Vec<Vec<usize>>,
    pub cell_types: Vec<CellType>,
}

impl<P> Mesh<P> {
    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }
}

impl Mesh<Real> {
    /// Create a 1D mesh from sorted points.
    ///
    /// Produces `n - 1` edge cells connecting consecutive vertices.
    pub fn line(points: Field<Real>) -> Self {
        let n = points.len();
        let num_cells = if n > 1 { n - 1 } else { 0 };
        let mut cells = Vec::with_capacity(num_cells);
        for i in 0..num_cells {
            cells.push(vec![i, i + 1]);
        }
        let cell_types = vec![CellType::Edge; num_cells];
        Self {
            vertices: points,
            cells,
            cell_types,
        }
    }
}

impl Mesh<Point2> {
    /// Create a 2D structured quad mesh from two 1D axis arrays.
    ///
    /// Given `nx` x-coordinates and `ny` y-coordinates, produces
    /// `nx * ny` vertices and `(nx-1) * (ny-1)` quadrilateral cells.
    pub fn tensor_product(x: &Field<Real>, y: &Field<Real>) -> Self {
        let nx = x.len();
        let ny = y.len();

        let mut vertices = Field::with_capacity(nx * ny);
        for i in 0..nx {
            for j in 0..ny {
                vertices.push(Point2::new(x[i], y[j]));
            }
        }

        let num_cells = if nx > 1 && ny > 1 {
            (nx - 1) * (ny - 1)
        } else {
            0
        };
        let mut cells = Vec::with_capacity(num_cells);
        for i in 0..(nx.saturating_sub(1)) {
            for j in 0..(ny.saturating_sub(1)) {
                let v00 = i * ny + j;
                let v10 = (i + 1) * ny + j;
                let v11 = (i + 1) * ny + (j + 1);
                let v01 = i * ny + (j + 1);
                cells.push(vec![v00, v10, v11, v01]);
            }
        }
        let cell_types = vec![CellType::Quad; num_cells];

        Self {
            vertices,
            cells,
            cell_types,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MeshSpadePoint {
    x: f64,
    y: f64,
}

impl HasPosition for MeshSpadePoint {
    type Scalar = f64;
    fn position(&self) -> spade::Point2<f64> {
        spade::Point2::new(self.x, self.y)
    }
}

impl Mesh<Point2> {
    /// Create a 2D triangle mesh via Delaunay triangulation.
    ///
    /// Given a set of 2D points, computes a Delaunay triangulation and
    /// returns a mesh with triangle cells. Requires at least 3 non-collinear points.
    pub fn delaunay(vertices: Field<Point2>) -> Self {
        let spade_pts: Vec<MeshSpadePoint> = vertices
            .iter()
            .map(|p| MeshSpadePoint { x: p[0], y: p[1] })
            .collect();

        let tri: DelaunayTriangulation<MeshSpadePoint> =
            DelaunayTriangulation::bulk_load(spade_pts)
                .expect("failed to build Delaunay triangulation");

        let mut cells = Vec::with_capacity(tri.num_inner_faces());
        for face in tri.inner_faces() {
            let vs = face.vertices();
            let i0 = vs[0].index();
            let i1 = vs[1].index();
            let i2 = vs[2].index();
            cells.push(vec![i0, i1, i2]);
        }

        let cell_types = vec![CellType::Triangle; cells.len()];

        Self {
            vertices,
            cells,
            cell_types,
        }
    }
}

impl Mesh<Point3> {
    /// Create a 3D structured hex mesh from three 1D axis arrays.
    ///
    /// Given `nx`, `ny`, `nz` coordinates, produces `nx * ny * nz` vertices
    /// and `(nx-1) * (ny-1) * (nz-1)` hexahedral cells.
    pub fn tensor_product(x: &Field<Real>, y: &Field<Real>, z: &Field<Real>) -> Self {
        let nx = x.len();
        let ny = y.len();
        let nz = z.len();

        let mut vertices = Field::with_capacity(nx * ny * nz);
        for i in 0..nx {
            for j in 0..ny {
                for k in 0..nz {
                    vertices.push(Point3::new(x[i], y[j], z[k]));
                }
            }
        }

        let num_cells = if nx > 1 && ny > 1 && nz > 1 {
            (nx - 1) * (ny - 1) * (nz - 1)
        } else {
            0
        };
        let mut cells = Vec::with_capacity(num_cells);
        let idx = |i: usize, j: usize, k: usize| i * ny * nz + j * nz + k;
        for i in 0..(nx.saturating_sub(1)) {
            for j in 0..(ny.saturating_sub(1)) {
                for k in 0..(nz.saturating_sub(1)) {
                    cells.push(vec![
                        idx(i, j, k),
                        idx(i + 1, j, k),
                        idx(i + 1, j + 1, k),
                        idx(i, j + 1, k),
                        idx(i, j, k + 1),
                        idx(i + 1, j, k + 1),
                        idx(i + 1, j + 1, k + 1),
                        idx(i, j + 1, k + 1),
                    ]);
                }
            }
        }
        let cell_types = vec![CellType::Hexa; num_cells];

        Self {
            vertices,
            cells,
            cell_types,
        }
    }

    /// Create a 3D mesh by extruding a 2D mesh along a 1D axis.
    ///
    /// Cell topology depends on the base cell type:
    /// - Triangle x segment -> Wedge (6 vertices)
    /// - Quad x segment -> Hexahedron (8 vertices)
    pub fn extrusion(base: &Mesh<Point2>, z: &Field<Real>) -> Self {
        let nb = base.num_vertices();
        let nz = z.len();

        let mut vertices = Field::with_capacity(nb * nz);
        for k in 0..nz {
            for vi in 0..nb {
                let p2 = base.vertices[vi];
                vertices.push(Point3::new(p2.x(), p2.y(), z[k]));
            }
        }

        let nz_cells = if nz > 1 { nz - 1 } else { 0 };
        let num_cells = base.num_cells() * nz_cells;
        let mut cells = Vec::with_capacity(num_cells);
        let mut cell_types = Vec::with_capacity(num_cells);

        for k in 0..nz_cells {
            let bot = k * nb;
            let top = (k + 1) * nb;
            for (ci, base_cell) in base.cells.iter().enumerate() {
                let base_type = base.cell_types[ci];
                match base_type {
                    CellType::Triangle => {
                        let (a, b, c) = (base_cell[0], base_cell[1], base_cell[2]);
                        cells.push(vec![
                            bot + a, bot + b, bot + c,
                            top + a, top + b, top + c,
                        ]);
                        cell_types.push(CellType::Wedge);
                    }
                    CellType::Quad => {
                        let (a, b, c, d) = (base_cell[0], base_cell[1], base_cell[2], base_cell[3]);
                        cells.push(vec![
                            bot + a, bot + b, bot + c, bot + d,
                            top + a, top + b, top + c, top + d,
                        ]);
                        cell_types.push(CellType::Hexa);
                    }
                    other => panic!(
                        "Extrusion not supported for base cell type {:?}",
                        other
                    ),
                }
            }
        }

        Self {
            vertices,
            cells,
            cell_types,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_mesh() {
        let points: Field<Real> = vec![0.0, 0.25, 0.5, 0.75, 1.0].into_iter().collect();
        let mesh = Mesh::line(points);
        assert_eq!(mesh.num_vertices(), 5);
        assert_eq!(mesh.num_cells(), 4);
        assert_eq!(mesh.cells[0], vec![0, 1]);
        assert_eq!(mesh.cells[3], vec![3, 4]);
        assert!(mesh.cell_types.iter().all(|&t| t == CellType::Edge));
    }

    #[test]
    fn test_line_mesh_single_point() {
        let points: Field<Real> = vec![0.0].into_iter().collect();
        let mesh = Mesh::line(points);
        assert_eq!(mesh.num_vertices(), 1);
        assert_eq!(mesh.num_cells(), 0);
    }

    #[test]
    fn test_tensor_product_2d() {
        let x: Field<Real> = vec![0.0, 1.0, 2.0].into_iter().collect();
        let y: Field<Real> = vec![0.0, 1.0].into_iter().collect();
        let mesh = Mesh::<Point2>::tensor_product(&x, &y);

        assert_eq!(mesh.num_vertices(), 6); // 3 * 2
        assert_eq!(mesh.num_cells(), 2); // 2 * 1
        assert!(mesh.cell_types.iter().all(|&t| t == CellType::Quad));

        assert_eq!(mesh.vertices[0], Point2::new(0.0, 0.0));
        assert_eq!(mesh.vertices[1], Point2::new(0.0, 1.0));
        assert_eq!(mesh.vertices[2], Point2::new(1.0, 0.0));
        assert_eq!(mesh.vertices[3], Point2::new(1.0, 1.0));

        // First quad: (0,0)-(1,0)-(1,1)-(0,1) -> indices 0,2,3,1
        assert_eq!(mesh.cells[0], vec![0, 2, 3, 1]);
    }

    #[test]
    fn test_tensor_product_3d() {
        let x: Field<Real> = vec![0.0, 1.0].into_iter().collect();
        let y: Field<Real> = vec![0.0, 1.0].into_iter().collect();
        let z: Field<Real> = vec![0.0, 1.0].into_iter().collect();
        let mesh = Mesh::<Point3>::tensor_product(&x, &y, &z);

        assert_eq!(mesh.num_vertices(), 8); // 2 * 2 * 2
        assert_eq!(mesh.num_cells(), 1); // 1 * 1 * 1
        assert_eq!(mesh.cell_types[0], CellType::Hexa);
        assert_eq!(mesh.cells[0].len(), 8);
    }

    #[test]
    fn test_extrusion_from_quads() {
        let x: Field<Real> = vec![0.0, 1.0].into_iter().collect();
        let y: Field<Real> = vec![0.0, 1.0].into_iter().collect();
        let base = Mesh::<Point2>::tensor_product(&x, &y);

        let z: Field<Real> = vec![0.0, 0.5, 1.0].into_iter().collect();
        let mesh = Mesh::extrusion(&base, &z);

        assert_eq!(mesh.num_vertices(), 4 * 3); // 4 base vertices * 3 z-levels
        assert_eq!(mesh.num_cells(), 1 * 2); // 1 base cell * 2 z-segments
        assert!(mesh.cell_types.iter().all(|&t| t == CellType::Hexa));
        assert_eq!(mesh.cells[0].len(), 8);
    }

    #[test]
    fn test_extrusion_from_triangles() {
        let base = Mesh {
            vertices: vec![
                Point2::new(0.0, 0.0),
                Point2::new(1.0, 0.0),
                Point2::new(0.5, 1.0),
            ]
            .into_iter()
            .collect(),
            cells: vec![vec![0, 1, 2]],
            cell_types: vec![CellType::Triangle],
        };

        let z: Field<Real> = vec![0.0, 1.0].into_iter().collect();
        let mesh = Mesh::extrusion(&base, &z);

        assert_eq!(mesh.num_vertices(), 6); // 3 * 2
        assert_eq!(mesh.num_cells(), 1);
        assert_eq!(mesh.cell_types[0], CellType::Wedge);
        assert_eq!(mesh.cells[0].len(), 6);
        // Bottom face: 0, 1, 2; Top face: 3, 4, 5
        assert_eq!(mesh.cells[0], vec![0, 1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_delaunay_mesh() {
        let vertices: Field<Point2> = vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(1.0, 1.0),
            Point2::new(0.0, 1.0),
            Point2::new(0.5, 0.5),
        ]
        .into_iter()
        .collect();

        let mesh = Mesh::delaunay(vertices);
        assert_eq!(mesh.num_vertices(), 5);
        assert!(mesh.num_cells() >= 4, "expected >= 4 triangles, got {}", mesh.num_cells());
        assert!(mesh
            .cell_types
            .iter()
            .all(|&t| t == CellType::Triangle));
        for cell in &mesh.cells {
            assert_eq!(cell.len(), 3);
            for &vi in cell {
                assert!(vi < 5, "vertex index {} out of range", vi);
            }
        }
    }

    #[test]
    fn test_delaunay_mesh_triangle() {
        let vertices: Field<Point2> = vec![
            Point2::new(0.0, 0.0),
            Point2::new(1.0, 0.0),
            Point2::new(0.5, 1.0),
        ]
        .into_iter()
        .collect();

        let mesh = Mesh::delaunay(vertices);
        assert_eq!(mesh.num_vertices(), 3);
        assert_eq!(mesh.num_cells(), 1);
        assert_eq!(mesh.cell_types[0], CellType::Triangle);
    }

    #[test]
    fn test_tensor_product_2d_vertex_positions() {
        let x: Field<Real> = vec![0.0, 0.5, 1.0].into_iter().collect();
        let y: Field<Real> = vec![0.0, 1.0, 2.0].into_iter().collect();
        let mesh = Mesh::<Point2>::tensor_product(&x, &y);

        assert_eq!(mesh.num_vertices(), 9);
        assert_eq!(mesh.num_cells(), 4);

        // Check corners
        assert_eq!(mesh.vertices[0], Point2::new(0.0, 0.0)); // (0,0)
        assert_eq!(mesh.vertices[2], Point2::new(0.0, 2.0)); // (0,2)
        assert_eq!(mesh.vertices[6], Point2::new(1.0, 0.0)); // (2,0)
        assert_eq!(mesh.vertices[8], Point2::new(1.0, 2.0)); // (2,2)
    }
}
