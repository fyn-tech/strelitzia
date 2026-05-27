
use crate::multiarray::Vector3;

struct Edge {
  pub vertices: Vec<Vector3>,
}

struct Face {
  pub edges: Vec<Edge>,
}

struct Body{
  pub faces: Vec<Face>,
}

