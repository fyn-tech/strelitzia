
use crate::multiarray::*;
use std::fmt::Debug;
// use crate::multiarray::linalg::*;


// ============================================================================
// The node struct
// ============================================================================


#[derive(Debug, Clone)]
pub struct Node<T: Copy + Clone + PartialOrd + Debug + Default + 'static> {
  pub value: T,
  pub i_left_child: Option<u32>,
  pub i_right_child: Option<u32>,
  pub leaves: Option<Vec<u32>>,
}

impl<T: Copy + Clone + PartialOrd + Debug + Default + 'static> Node<T> {
  pub fn new() -> Self {
    Self { value: T::default(), i_left_child: None, i_right_child: None, leaves: None }
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
pub struct KDTree<T: Copy + Clone + PartialOrd + Debug + Default + 'static, const D: usize> {
  pub depth: u32,
  pub nodes: Vec<Node<T> >,
  pub leaf_data: Vec<Vector<T, D> >,
}


// ============================================================================
// Methods
// ============================================================================

impl<T: Copy + Clone + PartialOrd + Debug + Default + 'static, const D: usize> KDTree<T, D> {

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

pub fn build(mut self, points: &Vec<Vector<T, D> >, max_depth: u32, 
             mut maybe_sorted_lists: Option<Vec<Vec<(T, usize)> > >, depth: Option<u32>) -> Self {
  
  if points.len() == 0 {
    self = Self::new();
    return self;
  }

  assert!(max_depth > 0);
  let depth = depth.unwrap_or(0) + 1;
  let index = (depth as usize)%D; // depth will always be 1.

  // if the list are not sorted sort them
  if maybe_sorted_lists.is_none() {
    // sort
    let mut lists = Vec::new();
    for d in 0..D {
        let mut scalars: Vec<(T, usize)> = points.iter()
            .enumerate()
            .map(|(i, p)| (p[d].clone(), i))
            .collect();
        scalars.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Greater));
        lists.push(scalars);
    }
    maybe_sorted_lists = Some(lists);
  }

  // end recursion, due to max depth or points
  if depth == max_depth || points.len() == 1 {
      
    self.depth = max_depth;

    let offset = self.leaf_data.len();

    let leaf_indices: Vec<u32> = maybe_sorted_lists.as_ref().unwrap()[index]
        .iter()
        .map(|(_, i)| *i as u32 + offset as u32)
        .collect();
    self.nodes.push(Node::new().leaves(&leaf_indices));
    
    self.leaf_data.extend(
      maybe_sorted_lists.as_ref().unwrap()[index].iter().map(|(_, i)| points[*i])
    );

    return self;
  }
  
  // build left
  let sorted_lists = maybe_sorted_lists.unwrap();
  let mid_index = sorted_lists.len()/2;
  let mid_value = points[sorted_lists[index][mid_index].1][index];
  let mut left_sorted: Vec<Vec<(T, usize)>> = vec![vec![]; D];
  let mut right_sorted: Vec<Vec<(T, usize)>> = vec![vec![]; D];
  for index_d in 0..D {

    if index_d == index {
      left_sorted[index] = sorted_lists[index][..mid_index].to_vec();
      right_sorted[index] = sorted_lists[index][..mid_index].to_vec();
    }
    else {
      for index in 0..sorted_lists.len() {
        let i_point = sorted_lists[index_d][index].1;
        let point_dim_value = points[i_point][index];
        if point_dim_value <= mid_value {
          left_sorted[index_d].push((point_dim_value, i_point));
        }
        else {
          right_sorted[index_d].push((point_dim_value, i_point));
        }
      }
    }
  }


  // self.build(points, max_depth, Some(left_sorted), Some(depth));
  // self.build(points, max_depth, Some(right_sorted), Some(depth));

  self

}

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_tree() {
        let points: Vec<Vector<f64, 2>> = vec![];
        let tree = KDTree::new().build(&points, 3, None, None);
        assert_eq!(tree.depth, 0);
        assert_eq!(tree.nodes.len(), 0);
        assert_eq!(tree.leaf_data.len(), 0);
    }

    #[test]
    fn single_depth_test() {
        let points: Vec<Vector<i32, 2>> = vec![
          Vector::<i32, 2>::new(1, 2),
          Vector::<i32, 2>::new(3, 4),
          Vector::<i32, 2>::new(5, 6),
        ];
        let tree = KDTree::new().build(&points, 1, None, None);
        assert_eq!(tree.depth, 1);
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.leaf_data.len(), 3);
        println!("{}, {}", tree.leaf_data[0][0], tree.leaf_data[0][1]);
        println!("{}, {}", tree.leaf_data[1][0], tree.leaf_data[1][1]);
        println!("{}, {}", tree.leaf_data[2][0], tree.leaf_data[2][1]);
    }
}
