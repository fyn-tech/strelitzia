
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
    Nurbs
}

struct Edge {
  pub vertices: Vec<Vector3>,
}

struct Face {
  pub edges: Vec<Edge>,
}

struct Body{
  pub faces: Vec<Face>,
}


