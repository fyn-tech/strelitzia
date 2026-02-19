use crate::multiarray::linalg::*;

use Option;

#[derive(Debug, Clone, Copy)]
pub struct Node {
  pub i_value: Option<u32>,
  pub i_left: Option<u32>,  
  pub i_right: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub struct TreeData<T, N> {
  pub leaf_values: Vec<Vector<T, N> >,
  pub node_values: Vec<Vector<T, N> >,
  pub nodes: Vec<Node>,
}

impl Node {
  pub fn new() -> Self{
    Self::default()
  }
  
  pub fn i_point(index: u32) -> Self {
    self.i_point = index;
    self
  }

  pub fn i_left(i_child: Node) -> Self {
    self.left = Some(child);
    self
  }

  pub fn maybe_i_left(child: Option<Node>) -> Self {
    self.left = child;
    self
  }

  pub fn i_right(child: Node) -> Self {
    self.right = Some(child);
    self
  }

  pub fn maybe_i_right(child: Option<Node>) -> Self {
    self.right = child;
    self
  }

  pub fn is_leaf() -> bool {
    self.left.is_empty() && self.right.is_empty()
  }

  pub fn reorder_dfs(tree: &mut TreeData, i_root: u32) {
    let old_new = Vec<(u32, u32)>().reserve(TreeData.nodes.len()); // old new
    let mut stack: Vec;
    stack.push(i_root);
    
    while let Some(i_node) = stack.pop() {
      old_new.push((i_node, old_new.len()));
      if let Some(i_right) = tree.right { stack.push(i_right); }
      if let Some(i_left) = tree.left { stack.push(i_left); }
    }

    old_new.iter();
  }

}