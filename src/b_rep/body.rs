
use std::fmt::format;

use crate::multiarray::Vector3;

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
    pub vertices: [usize; 2],
}

struct Loop {
    pub edges: Vec<usize>,
    pub reversed: Vec<bool>,
}

impl Loop {
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty() && self.reversed.is_empty()
    }

    pub fn last(&self) -> Option<(usize, bool)> {
        Some((*self.edges.last()?, *self.reversed.last()?))
    }

    pub fn push(&mut self, edge: usize, reverse: bool) {
        self.edges.push(edge);
        self.reversed.push(reverse);
    }

    pub fn reserve(&mut self, additional: usize) {
        self.edges.reserve(additional);
        self.reversed.reserve(additional);
    }
}

struct Surface {
    
}

struct Face {
    pub outer_loop: Loop,
    pub inner_loops: Vec<Loop>,
    pub surface: Surface
}

struct Body {
    pub vertices: Vec<Vector3>,
    pub edges: Vec<Edge>,
    pub loops: Vec<Loop>,
    pub faces: Vec<Face>,
}

impl Body {
    pub fn add_vertex(&mut self, point: &Vector3) -> usize {
        self.vertices.push(point.clone());
        self.vertices.len() - 1
    }

    pub fn add_edge(&mut self, vertex_0: usize, vertex_1: usize) -> usize {
        self.edges.push(Edge{vertices: [vertex_0, vertex_1]});
        self.edges.len() - 1
    }

    pub fn create_loop(&mut self, edges: &Vec<usize>) -> Result<usize, String> {

        if edges.is_empty() {
            return Err("No edges provided.".to_string());
        }
        if edges.len() < 2 {
            return Err(format!("Loop requires two edges, got {}.", edges.len()));
        }

        let mut new_loop = Loop{ edges: Vec::new(), reversed: Vec::new() };
        new_loop.reserve(edges.len());

        for i_edge in edges.iter() {
            let edge = self.edges.get(*i_edge).ok_or( format!("Index {} not in range [0, {}).", *i_edge, self.edges.len()))?;
            
            if new_loop.is_empty() {
                new_loop.push(*i_edge, false);
                continue;
            }

            let (last_edge, reversed) = &new_loop.last().unwrap();
            let next_vertex = &self.edges[*last_edge].vertices[*reversed as usize];
            if *next_vertex == edge.vertices[0] {
                new_loop.push(*i_edge, false);
            }
            else if *next_vertex == edge.vertices[1] {
                new_loop.push(*i_edge, true);
            }
            else {
                return Err(format!(
                    "Edge {} (vertices: [{}, {}], reversed: {}) does not connect to edge {} (vertices [{}, {}]) or its reverse.",
                    *last_edge, &self.edges[*last_edge].vertices[0], &self.edges[*last_edge].vertices[1], reversed,
                    *i_edge, &self.edges[*i_edge].vertices[0], &self.edges[*i_edge].vertices[1]
                ));
            }
        }
        self.loops.push(new_loop);
        Ok(self.loops.len() - 1)
    }
}

