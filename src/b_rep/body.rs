use std::num::FpCategory::Infinite;
use std::usize;

use crate::common::{Real, bounds_failure_str};
use crate::multiarray::{Vector2, Vector3};
use crate::multiarray::linalg::{dot, normalised, l2_norm, cross};
// notes:
//  pages.mtu.edu/~shene/COURSES/cs3621/NOTES/
//  developer.rhino3d.com/guides/general/essential-mathematics/parametric-curves-surfaces/
//  https://opencascade.blogspot.com/

enum CurveType {
    Line,
    Circle,
    Nurbs,
}

enum SurfaceType {
    Plane,
    Cylinder,
    Nurbs,
}


struct Edge {
    pub i_vertices: [usize; 2],
    pub direction: Vector3,
}

struct EdgeLoop {
    pub i_edges: Vec<usize>,
    pub reversed: Vec<bool>,
}

impl EdgeLoop {
    pub fn is_empty(&self) -> bool {
        self.i_edges.is_empty() && self.reversed.is_empty()
    }

    pub fn last(&self) -> Option<(usize, bool)> {
        Some((*self.i_edges.last()?, *self.reversed.last()?))
    }

    pub fn push(&mut self, edge: usize, reverse: bool) {
        self.i_edges.push(edge);
        self.reversed.push(reverse);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.i_edges.reserve(additional);
        self.reversed.reserve(additional);
    }
}


struct UVBasis {
    pub origin: Vector3, // origin in R3 canonical
    pub u_axis: Vector3, // u_axis in R3 canonical
    pub v_axis: Vector3, // v_axis in R3 canonical
}

impl UVBasis {

    pub fn new(origin: &Vector3, normal: &Vector3) -> Self {

        // degenerate cases -> normal no normalised
        let seed = if normal[0].abs() < 0.9 {
            Vector3::new(1.0, 0.0, 0.0)
        } else {
            Vector3::new(0.0, 1.0, 0.0)
        };
        let u_axis = normalised(&cross(&seed, normal));
        let v_axis = normalised(&cross(normal, &u_axis));
        Self { origin: *origin, u_axis, v_axis }
    }

    pub fn origin_from_xyz(&mut self, origin: &Vector3) {
        self.origin = *origin
    }

    pub fn origin_from_uv(&mut self, origin: &Vector2) {
        self.origin = self.to_xyz(&origin)
    }


    pub fn to_uv(&self, v: &Vector3) -> Vector2 {
        let translate = v - &self.origin;
        Vector2::new(dot(&translate, &self.u_axis), dot(&translate, &self.v_axis))
    }

    pub fn to_xyz(&self, uv: &Vector2) -> Vector3 {
        uv[0] * &self.u_axis + uv[1] * &self.v_axis + self.origin
    }
    
}


struct Surface {
    // u, v data
    pub uvvertices: Vec<Vector2>,
    pub uvedge: Vec<Edge>,
}

struct Face {
    pub outer_edge_loop: usize,
    pub inner_edge_loops: Vec<usize>,
    pub surface: Surface,
}

struct Body {
    // x, y, z
    pub vertices: Vec<Vector3>,
    pub edges: Vec<Edge>,
    pub edge_loops: Vec<EdgeLoop>,
    pub faces: Vec<Face>,
}

impl Body {
    pub fn add_vertex(&mut self, point: &Vector3) -> usize {
        self.vertices.push(point.clone());
        self.vertices.len() - 1
    }

    pub fn add_edge(&mut self, i_vertex_0: usize, i_vertex_1: usize) -> Result<usize, String> {
        let vertex_0 = self
            .vertices
            .get(i_vertex_0)
            .ok_or(bounds_failure_str(i_vertex_0, self.vertices.len()))?;

        let vertex_1 = self
            .vertices
            .get(i_vertex_1)
            .ok_or(bounds_failure_str(i_vertex_1, self.vertices.len()))?;

        self.edges.push(Edge {
            i_vertices: [i_vertex_0, i_vertex_1],
            direction: vertex_1 - vertex_0,
        });
        Ok(self.edges.len() - 1)
    }

    pub fn create_loop(&mut self, i_edges: &Vec<usize>) -> Result<usize, String> {
        if i_edges.is_empty() {
            return Err("No edges provided.".to_string());
        }
        if i_edges.len() < 2 {
            return Err(format!("Loop requires two edges, got {}.", i_edges.len()));
        }

        let mut new_loop = EdgeLoop {
            i_edges: Vec::new(),
            reversed: Vec::new(),
        };
        new_loop.reserve(i_edges.len());

        for i_edge in i_edges.iter() {
            let edge = self.edges.get(*i_edge).ok_or(format!(
                "Index {} not in range [0, {}).",
                *i_edge,
                self.edges.len()
            ))?;

            if new_loop.is_empty() {
                new_loop.push(*i_edge, false);
                continue;
            }

            let (last_edge, reversed) = &new_loop.last().unwrap();
            let next_vertex = &self.edges[*last_edge].i_vertices[!*reversed as usize];
            if *next_vertex == edge.i_vertices[0] {
                new_loop.push(*i_edge, false);
            } else if *next_vertex == edge.i_vertices[1] {
                new_loop.push(*i_edge, true);
            } else {
                return Err(format!(
                    "Edge {} (vertices: [{}, {}], reversed: {}) does not connect to edge {} (vertices [{}, {}]) or its reverse.",
                    *last_edge,
                    &self.edges[*last_edge].i_vertices[0],
                    &self.edges[*last_edge].i_vertices[1],
                    reversed,
                    *i_edge,
                    &self.edges[*i_edge].i_vertices[0],
                    &self.edges[*i_edge].i_vertices[1]
                ));
            }
        }
        self.edge_loops.push(new_loop);
        Ok(self.edge_loops.len() - 1)
    }

    pub fn create_face(&mut self, i_loop: usize) -> Result<usize, String> {
        let edge_loop = &self.edge_loops.get(i_loop).ok_or(format!(
            "Index {} not in range [0, {}).",
            i_loop,
            self.edge_loops.len()
        ))?;

        // planar face - create face normal
        if edge_loop.i_edges.len() < 3 {
            return Err(
                "Currently loops need at least 3 edges to prevent degenerate faces".to_string(),
            );
        }

        // degenerate cases?
        let co_plane_0 =
            (!edge_loop.reversed[0] as u8 as Real) * &self.edges[edge_loop.i_edges[0]].direction;
        let co_plane_1 =
            (!edge_loop.reversed[1] as u8 as Real) * &self.edges[edge_loop.i_edges[1]].direction;
        let plain_normal = normalised(&cross(&co_plane_0, &co_plane_1));

        // todo plane constant and add to surface.
        let mut uv_basis = UVBasis::new(&self.vertices[self.edges[edge_loop.i_edges[0]].i_vertices[0]], &plain_normal);
        
        let mut new_uv_origin = Vector2::new(Real::INFINITY,Real::INFINITY);
        for (i_local, i_edge) in edge_loop.i_edges.iter().enumerate() {
            let edge = &self.edges[*i_edge];
            let uv_point = uv_basis.to_uv(&self.vertices[edge.i_vertices[0 + edge_loop.reversed[i_local] as usize]]);
            new_uv_origin[0] = Real::min(new_uv_origin[0], uv_point[0]);
            new_uv_origin[1] = Real::min(new_uv_origin[1], uv_point[1]);
        }
        uv_basis.origin_from_uv(&new_uv_origin);
        

        
        Ok(0)
    }

    pub fn get_point_face(&self, i_face: usize) -> Vector3 {
        // return Vector3.new();
    }
pub fn get_point_surface -> Option<Vector3>
}
