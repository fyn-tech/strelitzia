


// ============================================================================
// The core struct
// ============================================================================

#[derive(Debug, Clone, Copy)]
pub struct KDTree<T, D> {
  pub depth: i32,
  pub leaf: bool,
  pub value: T, 
}


// ============================================================================
// Methods
// ============================================================================

impl<T, D> KDTree<T, D> {

pub fn new() -> Self {
  Self {
    depth: 0,
  }
} 

pub fn depth(self, depth: i32) -> Self{
  self.depth = depth;
  self
}

pub fn build(points: Vector<T, S>) -> self {
  self
}

}