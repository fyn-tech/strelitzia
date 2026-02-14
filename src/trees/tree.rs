use crate::multiarray::linalg::*;

use Option;

#[derive(Debug, Clone, Copy)]
pub struct Tree<T, S> {
  pub points: MultiArray<T, S>,

}


#[derive(Debug, Clone, Copy)]
pub struct Node {
  pub i_point: i64,
  pub left: Option<Node>,  
  pub right: Option<Node>,
}

impl Node {
  pub fn new() -> Self{
    Self::default()
  }
  
  pub fn i_point(index: i64) -> Self {
    self.i_point = index;
    self
  }

  pub fn left(child: Node) -> Self {
    self.left = Some(child);
    self
  }

  pub fn maybe_left(child: Option<Node>) -> Self {
    self.left = child;
    self
  }

  pub fn right(child: Node) -> Self {
    self.right = Some(child);
    self
  }

  pub fn maybe_right(child: Option<Node>) -> Self {
    self.right = child;
    self
  }

  pub fn is_leaf() -> bool {
    self.left.is_empty() && self.right.is_empty()
  }


}