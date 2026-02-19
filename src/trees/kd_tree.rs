
use crate::multiarray::*;
use std::fmt::Debug;
// use crate::multiarray::linalg::*;


// ============================================================================
// The core struct
// ============================================================================

#[derive(Debug, Clone)]
pub struct KDTree<T: Copy + Clone + PartialOrd + Debug  + 'static, const D: usize> {
  pub depth: u32,
  pub leaf_data: Vec<Vector<T, D> >,
}


// ============================================================================
// Methods
// ============================================================================

impl<T: Copy + Clone + PartialOrd + Debug + 'static, const D: usize> KDTree<T, D> {

pub fn new() -> Self {
  Self {
    depth: 0,
    leaf_data: vec![],
  }
} 

pub fn depth(mut self, depth: u32) -> Self{
  self.depth = depth;
  self
}

pub fn build(mut self, points: &Vec<Vector<T, D> >, max_depth: u32, 
             mut maybe_sorted_lists: Option<Vec<Vec<(T, usize)> > >, depth: Option<u32>) -> Self {
  self.depth = max_depth;
  let depth = depth.unwrap_or(0);
  let index = D%(depth as usize);

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

  if depth == max_depth || points.len() == 1 {
    // end recursion  
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


  self.build(points, max_depth, Some(left_sorted), Some(depth + 1));
  self.build(points, max_depth, Some(right_sorted), Some(depth + 1));

  self

}

}