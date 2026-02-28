
// use nalgebra::indexing;

use crate::multiarray::*;
use std::{fmt::Debug, usize};
// use crate::multiarray::linalg::*;


pub trait Scalar: Copy + Clone + PartialOrd + Debug + Default + 'static {}
impl<T: Copy + Clone + PartialOrd + Debug + Default + 'static> Scalar for T {}

// ============================================================================
// The node struct
// ============================================================================

#[derive(Debug, Clone)]
pub struct Node<T: Scalar> {
  pub value: T,
  pub i_left_child: Option<u32>,
  pub i_right_child: Option<u32>,
  pub leaves: Option<Vec<u32>>,
}

impl<T: Scalar> Node<T> {
  pub fn new() -> Self {
    Self { value: T::default(), i_left_child: None, i_right_child: None, leaves: None }
  }

  pub fn value(mut self, node_value: T) -> Self {
    self.value = node_value;
    self  
  }

  pub fn is_leaf(self) -> bool {
    self.i_left_child.is_none() && self.i_right_child.is_none()  
  }

  pub fn leaves(mut self, leaf_indexes: &Vec<u32>) -> Self {
    self.leaves = Some(leaf_indexes.clone());
    self
  }
}

// ============================================================================
// The core struct
// ============================================================================


#[derive(Debug, Clone)]
pub struct KDTree<T: Scalar, const D: usize> {
  pub depth: u32,
  pub nodes: Vec<Node<T> >,
  pub leaf_data: Vec<Vector<T, D> >,
}


// ============================================================================
// Methods
// ============================================================================

impl<T: Scalar, const D: usize> KDTree<T, D> {

pub fn new() -> Self {
  Self {
    depth: 0,
    nodes: vec![],
    leaf_data: vec![],
  }
} 

pub fn depth(mut self, depth: u32) -> Self{
  self.depth = depth;
  self
}

pub fn build(mut self, points: &Vec<Vector<T, D> >, max_depth: u32) -> Self{
  
  if points.len() == 0 {
    return Self::new();
  }
  assert!(max_depth > 0);

  let sorted_lists = create_sorted_lists(&points);
  self.recursive_build(points, sorted_lists, max_depth, 0);
  self
}

fn recursive_build(&mut self, points: &Vec<Vector<T, D> >, sorted_lists: Vec<Vec<(T, usize)>>, max_depth: u32, depth: u32) -> Option<u32> {
  
  // end recursion, due to max depth or points
  println!("here {}", depth);
  let i_dim = (depth as usize)%D;
  if (depth + 1) == max_depth || points.len() == 1 || sorted_lists[i_dim].len() <= 1 {
  println!("leaf {}", depth);
    return self.add_leaf_node(points, &sorted_lists[i_dim], depth);   
  }  
  let (left_list, right_list) = bisect_sorted_lists_along_dim(&sorted_lists, i_dim);

  // create new node.
  println!("no leaf {}", depth);
  println!("left length {}", left_list.len());
  println!("right length {}", right_list.len());
  println!("left length - 0: {}", left_list[0].len());
  println!("right length - 0: {}", right_list[0].len());
  println!("left length - 1: {}", left_list[1].len());
  println!("right length - 1: {}", right_list[1].len());
  let i_node = self.nodes.len();
  self.nodes.push(Node::new().value(left_list[i_dim].last().unwrap().0));
  self.nodes[i_node].i_left_child = self.recursive_build(&points, left_list, max_depth, depth + 1);
  self.nodes[i_node].i_right_child = self.recursive_build(&points, right_list, max_depth, depth + 1);
  Some(i_node as u32)  
}


fn add_leaf_node(&mut self, points: &Vec<Vector<T, D> >, sorted_indices: &Vec<(T, usize)>, depth: u32) -> Option<u32> {
    if sorted_indices.len() == 0 {
      return None;
    }

    self.depth = std::cmp::max(self.depth, depth + 1);
    let offset = self.leaf_data.len();
    let leaf_indices: Vec<u32> = sorted_indices
        .iter()
        .map(|(_, i)| *i as u32 + offset as u32)
        .collect();
    self.nodes.push(Node::new().leaves(&leaf_indices));
    self.leaf_data.extend(
      sorted_indices.iter().map(|(_, i)| points[*i])
    );
    Some(self.nodes.len() as u32 - 1)
}

}

fn bisect_sorted_lists_along_dim<T: Scalar>(sorted_lists: &Vec<Vec<(T, usize)> >, i_dim: usize) -> (Vec<Vec<(T, usize)> >, Vec<Vec<(T, usize)> >) {
  
  println!("\tbsl - {}", i_dim);


  let mid_index = sorted_lists[i_dim].len()/2;
  let mid_value = sorted_lists[i_dim][mid_index].0;



  println!("\tmid_index - {}", mid_index);
  println!("\tmid_value - {:?}", mid_value);
  let mut left_sorted: Vec<Vec<(T, usize)>> = vec![vec![]; sorted_lists.len()];
  let mut right_sorted: Vec<Vec<(T, usize)>> = vec![vec![]; sorted_lists.len()];
  for i_sort_dim in 0..sorted_lists.len() {
    if i_sort_dim == i_dim {
      left_sorted[i_sort_dim] = sorted_lists[i_sort_dim][..mid_index].to_vec();
      right_sorted[i_sort_dim] = sorted_lists[i_sort_dim][mid_index..].to_vec();
    }
    else {
      for index in 0..sorted_lists[i_sort_dim].len() {
        println!("\tmid_value check - {:?}", mid_value);
        if sorted_lists[i_dim][index].0 < mid_value {
          left_sorted[i_sort_dim].push(sorted_lists[i_sort_dim][index]);
        }
        else {
          right_sorted[i_sort_dim].push(sorted_lists[i_sort_dim][index]);
        }
      }
    }
  }

  (left_sorted, right_sorted)
}

fn create_sorted_lists<T: Scalar, const D: usize>(points: &Vec<Vector<T, D> >) -> Vec<Vec<(T, usize)> > {
  let mut lists = Vec::new();
  for d in 0..D {
      let mut scalars: Vec<(T, usize)> = points.iter()
          .enumerate()
          .map(|(i, p)| (p[d].clone(), i))
          .collect();
      scalars.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Greater));
      lists.push(scalars);
  }
  lists
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_tree() {
        let points: Vec<Vector<f64, 2>> = vec![];
        let tree = KDTree::new().build(&points, 3);
        assert_eq!(tree.depth, 0);
        assert_eq!(tree.nodes.len(), 0);
        assert_eq!(tree.leaf_data.len(), 0);
    }

    #[test]
    fn single_depth_test() {
        let points: Vec<Vector<i32, 2>> = vec![
          Vector::<i32, 2>::new(5, -2),
          Vector::<i32, 2>::new(1, -4),
          Vector::<i32, 2>::new(3, 0),
        ];
        let tree = KDTree::new().build(&points, 1);
        assert_eq!(tree.depth, 1);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.leaf_data.len(), 3);
        println!("{}, {}", tree.leaf_data[0][0], tree.leaf_data[0][1]);
        println!("{}, {}", tree.leaf_data[1][0], tree.leaf_data[1][1]);
        println!("{}, {}", tree.leaf_data[2][0], tree.leaf_data[2][1]);
    }

    #[test]
    fn second_depth_test() {
        let points: Vec<Vector<i32, 2>> = vec![
          Vector::<i32, 2>::new(5, -2),
          Vector::<i32, 2>::new(1, -4),
          Vector::<i32, 2>::new(3, 0),
        ];
        let tree = KDTree::new().build(&points, 2);
        assert_eq!(tree.depth, 2);
        assert_eq!(tree.nodes.len(), 3);
        assert_eq!(tree.leaf_data.len(), 3);
        println!("{}, {}", tree.leaf_data[0][0], tree.leaf_data[0][1]);
        println!("{}, {}", tree.leaf_data[1][0], tree.leaf_data[1][1]);
        println!("{}, {}", tree.leaf_data[2][0], tree.leaf_data[2][1]);
    }

    #[test]
    fn third_depth_test() {
        let points: Vec<Vector<i32, 2>> = vec![
          Vector::<i32, 2>::new(5, -2),
          Vector::<i32, 2>::new(1, -4),
          Vector::<i32, 2>::new(3, 0),
        ];
        let tree = KDTree::new().build(&points, 3);
        assert_eq!(tree.depth, 3);
        assert_eq!(tree.nodes.len(), 4);
        assert_eq!(tree.leaf_data.len(), 3);
        println!("{}, {}", tree.leaf_data[0][0], tree.leaf_data[0][1]);
        println!("{}, {}", tree.leaf_data[1][0], tree.leaf_data[1][1]);
        println!("{}, {}", tree.leaf_data[2][0], tree.leaf_data[2][1]);
    }
}
