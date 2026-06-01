
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
}

struct Face {
    pub edges: Vec<Edge>,
}

struct Body {
    pub vertices: Vec<Vector3>,
    pub edges: Vec<Edge>,
    pub loops: Vec<usize>,
    pub faces: Vec<Face>,
}

impl Body {
    pub fn add_vertex(mut self, point: &Vector3) -> usize {
        self.vertices.push(point.clone());
        self.vertices.len() - 1
    }

    pub fn add_edge(mut self, vertex_0: usize, vertex_1: usize) -> usize {
        self.edges.push(Edge{vertices: [vertex_0, vertex_1]});
        self.edges.len() - 1
    }

    pub fn create_loop(mut self, edges: &Vec<usize>) -> Result<usize, String> {

        if edges.is_empty() {
            return Err("No edges provided.".to_string());
        }
        if edges.len() < 2 {
            return Err(format!("Loop requires two edges, got {}.", edges.len()));
        }

        let mut new_loop = Loop{ edges: Vec::new() };
        new_loop.edges.reserve(edges.len());
        for (i_edge, i_next_edge) in edges.iter().zip(edges.iter().cycle().skip(1)) {
            let edge = self.edges.get(*i_edge).ok_or( format!("Index {} not in range [0, {}).", *i_edge, self.edges.len()))?;
            let next_edge = self.edges.get(*i_next_edge).ok_or( format!("Index {} not in range [0, {}).", *i_next_edge, self.edges.len()))?;
            
            if edge.vertices[1] == next_edge.vertices[0] {
                new_loop.edges.push(*i_edge);
            }
            else {
                return Err(format!(
                    "Edge {} (vertices [{}, {}]) does not connect to edge {} (vertices [{}, {}]).",
                    *i_edge, edge.vertices[0], edge.vertices[1],
                    *i_next_edge, next_edge.vertices[0], next_edge.vertices[1]
                ));
            }
        }
        self.loops.push(new_loop);
        Ok(self.loops.len() - 1)
    }
}

