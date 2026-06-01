
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;

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

struct Face {
    pub edges: Vec<Edge>,
}

struct Body {
    pub vertices: Vec<Vector3>,
    pub edges: Vec<Edge>,
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
}

