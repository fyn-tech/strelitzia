
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
  pub i_left_child: Option<usize>,
  pub i_right_child: Option<usize>,
  pub leaves: Option<Vec<usize>>,
}

impl<T: Scalar> Node<T> {
  pub fn new() -> Self {
    Self { value: T::default(), i_left_child: None, i_right_child: None, leaves: None }
  }

  pub fn value(mut self, node_value: T) -> Self {
    self.value = node_value;
    self  
  }

  pub fn is_leaf(&self) -> bool {
    self.i_left_child.is_none() && self.i_right_child.is_none()  
  }

  pub fn leaves(mut self, leaf_indexes: &Vec<usize>) -> Self {
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
  
  if points.is_empty() {
    return Self::new();
  }
  assert!(max_depth > 0);

  let sorted_lists = create_sorted_lists(&points);
  self.recursive_build(points, sorted_lists, max_depth, 0);
  self
}

fn recursive_build(&mut self, points: &Vec<Vector<T, D> >, sorted_lists: Vec<Vec<usize>>, max_depth: u32, depth: u32) -> Option<usize> {
  
  // end recursion, due to max depth or points
  let i_dim = (depth as usize)%D;
  if (depth + 1) == max_depth || points.len() == 1 || sorted_lists[i_dim].len() <= 1 {
    return self.add_leaf_node(points, &sorted_lists[i_dim], depth);   
  }  
  let (left_list, right_list) = bisect_sorted_lists_along_dim(&points, &sorted_lists, i_dim);

  // create new node.
  let i_node = self.nodes.len();
  self.nodes.push(Node::new().value(points[*left_list[i_dim].last().unwrap()][i_dim]));
  self.nodes[i_node].i_left_child = self.recursive_build(&points, left_list, max_depth, depth + 1);
  self.nodes[i_node].i_right_child = self.recursive_build(&points, right_list, max_depth, depth + 1);
  Some(i_node)  
}

fn add_leaf_node(&mut self, points: &Vec<Vector<T, D> >, sorted_indices: &Vec<usize>, depth: u32) -> Option<usize> {
    if sorted_indices.len() == 0 {
      return None;
    }

    self.depth = std::cmp::max(self.depth, depth + 1);
    let offset = self.leaf_data.len();
    let leaf_indices: Vec<usize> = (offset..offset + sorted_indices.len()).collect();
    self.nodes.push(Node::new().leaves(&leaf_indices));
    self.leaf_data.extend(
      sorted_indices.iter().map(|i| points[*i])
    );
    Some(self.nodes.len() - 1)
}

fn search(&self, search_region: &Vector<Vector<T, D>, 2>, maybe_i_node: Option<usize>, maybe_bounded_region: Option<Vector<Vector<T, D>, 2> >) {
  let i_node = maybe_i_node.unwrap_or(0); // start at root node
  // let bounded_region = maybe_bounded_region.unwrap_or();

  if self.nodes[i_node].is_leaf(){
    let range = self.get_sub_tree_leaves(i_node);
  }
}

fn get_sub_tree_leaves(&self, i_node: usize) -> (usize, usize) {
  (self.get_bounding_leaf(i_node, true), self.get_bounding_leaf(i_node, false))
}

fn get_bounding_leaf(&self, i_node: usize, is_left: bool) -> usize {

  let node = &self.nodes[i_node];
  if node.is_leaf() {
    let leaves = node.leaves.as_ref().unwrap();
    if is_left { leaves.first() } else { leaves.last() }.copied().unwrap()
  }
  else {
    let child = if is_left {
        node.i_left_child.unwrap_or_else(|| node.i_right_child.unwrap())
    } else {
        node.i_right_child.unwrap_or_else(|| node.i_left_child.unwrap())
    };
    self.get_bounding_leaf(child, is_left)
  }

}

}


fn bisect_sorted_lists_along_dim<T: Scalar, const D: usize>(points: &Vec<Vector<T, D> >, sorted_lists: &Vec<Vec<usize> >, i_dim: usize) -> (Vec<Vec<usize> >, Vec<Vec<usize> >) {
  
  // Determine splitting line
  assert!(sorted_lists[i_dim].len() > 0, "List must have size greater than zero");
  let mid_index = (sorted_lists[i_dim].len() - 1)/2;
  let mid_value = points[sorted_lists[i_dim][mid_index]][i_dim];

  // construct left and right sorted lists.
  let mut left_sorted: Vec<Vec<usize>> = vec![vec![]; sorted_lists.len()];
  let mut right_sorted: Vec<Vec<usize>> = vec![vec![]; sorted_lists.len()];
  for i_sort_dim in 0..sorted_lists.len() {
      for index in 0..sorted_lists[i_sort_dim].len() {
        if points[sorted_lists[i_sort_dim][index]][i_dim] <= mid_value {
          left_sorted[i_sort_dim].push(sorted_lists[i_sort_dim][index]);
        }
        else {
          right_sorted[i_sort_dim].push(sorted_lists[i_sort_dim][index]);
        }
    }
  }

  (left_sorted, right_sorted)
}

fn create_sorted_lists<T: Scalar, const D: usize>(points: &Vec<Vector<T, D> >) -> Vec<Vec<usize> > {
  let mut lists = Vec::new();
  for d in 0..D {
      let mut indices: Vec<usize> = (0..points.len()).collect();
      indices.sort_by(|&a, &b| points[a][d].partial_cmp(&points[b][d]).unwrap_or(std::cmp::Ordering::Greater));
      lists.push(indices);
  }
  lists
}

// ============================================================================
// Visualisation
// ============================================================================

/// Assigns normalised x positions (0, 1, 2, … per leaf) and depths to every
/// node via a left-to-right DFS.  Returns the x centre of the subtree rooted
/// at `i`.
fn layout_nodes<T: Scalar>(nodes: &[Node<T>], i: usize, depth: u32,
                            x: &mut Vec<f64>, depths: &mut Vec<u32>,
                            counter: &mut f64) -> f64 {
  let node = &nodes[i as usize];
  depths[i as usize] = depth;
  if node.is_leaf() {
    let pos = *counter;
    *counter += 1.0;
    x[i as usize] = pos;
    return pos;
  }
  let lx = node.i_left_child .map_or(0.0, |c| layout_nodes(nodes, c, depth + 1, x, depths, counter));
  let rx = node.i_right_child.map_or(lx,  |c| layout_nodes(nodes, c, depth + 1, x, depths, counter));
  x[i as usize] = (lx + rx) / 2.0;
  x[i as usize]
}

impl<T: Scalar, const D: usize> KDTree<T, D> {
  /// Render the tree as a top-down graph and write it to an SVG file at `path`.
  pub fn draw(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    use plotters::prelude::*;

    if self.nodes.is_empty() { return Ok(()); }

    // --- layout -------------------------------------------------------
    let n = self.nodes.len();
    let mut x_pos: Vec<f64> = vec![0.0; n];
    let mut depths: Vec<u32> = vec![0; n];
    let mut counter = 0.0f64;
    layout_nodes(&self.nodes, 0, 0, &mut x_pos, &mut depths, &mut counter);
    let n_leaves   = counter as usize;
    let max_depth  = *depths.iter().max().unwrap();

    // --- pre-compute labels so we can size boxes to fit content --------
    // plotters converts the size via a pt→px factor; 20 here renders as ~16px in browsers.
    const FONT_SZ: u32 = 20;
    const FONT_PX: i32 = 16;  // actual rendered glyph height (plotters applies a pt→px factor)
    const CHAR_W:  i32 = 9;   // ~px per character used only for node width estimation
    const LINE_H:  i32 = 22;  // line spacing (glyph height + gap)
    const PAD:     i32 = 10;
    const V_GAP:   i32 = 55;

    let labels: Vec<Vec<String>> = self.nodes.iter().enumerate().map(|(i, node)| {
      if node.is_leaf() {
        node.leaves.as_ref().unwrap().iter().map(|&idx| {
          let pt = &self.leaf_data[idx as usize];
          let coords: Vec<String> = (0..D).map(|d| format!("{:?}", pt[d])).collect();
          format!("#{}: ({})", idx, coords.join(", "))
        }).collect()
      } else {
        let i_dim = depths[i] as usize % D;
        vec![format!("x{} \u{2264} {:?}", i_dim, node.value)]
      }
    }).collect();

    // node width = widest label + padding, same for every node so the tree looks uniform
    let node_w = labels.iter().flat_map(|ls| ls.iter())
      .map(|s| s.len() as i32 * CHAR_W + 2 * PAD)
      .max().unwrap_or(80).max(80);

    let h_slot  = node_w + 40;
    let max_pts = self.nodes.iter().filter_map(|n| n.leaves.as_ref()).map(|l| l.len()).max().unwrap_or(1);
    let leaf_h  = max_pts as i32 * LINE_H + 2 * PAD;
    let inner_h = LINE_H + 2 * PAD;
    let row_h   = leaf_h + V_GAP;

    let cw = (h_slot * n_leaves as i32 + 60).max(300) as u32;
    let ch = ((max_depth as i32 + 1) * row_h + 60).max(150) as u32;

    // pixel helpers
    let px  = |xn: f64| -> i32 { 30 + h_slot / 2 + (xn * h_slot as f64) as i32 };
    let top = |d: u32, is_leaf: bool| -> i32 {
      let base = 30 + d as i32 * row_h + V_GAP / 2;
      if is_leaf { base } else { base + (leaf_h - inner_h) / 2 }
    };
    let nh = |is_leaf: bool| -> i32 { if is_leaf { leaf_h } else { inner_h } };

    // --- render -------------------------------------------------------
    let root = SVGBackend::new(path, (cw, ch)).into_drawing_area();
    root.fill(&WHITE)?;

    // edges (drawn first so nodes paint on top)
    for (i, node) in self.nodes.iter().enumerate() {
      let il  = node.is_leaf();
      let cx  = px(x_pos[i]);
      let cby = top(depths[i], il) + nh(il);
      for &child in [node.i_left_child, node.i_right_child].iter().flatten() {
        let c   = child as usize;
        let cil = self.nodes[c].is_leaf();
        root.draw(&PathElement::new([(cx, cby), (px(x_pos[c]), top(depths[c], cil))], BLACK))?;
      }
    }

    // nodes — use HPos::Center so the SVG backend emits text-anchor="middle",
    // giving exact horizontal centering without any character-width guesswork.
    use plotters::style::text_anchor::{HPos, Pos, VPos};
    let font = plotters::style::TextStyle::from(("sans-serif", FONT_SZ).into_font())
      .pos(Pos::new(HPos::Center, VPos::Top));
    for (i, node) in self.nodes.iter().enumerate() {
      let il  = node.is_leaf();
      let cx  = px(x_pos[i]);
      let ty  = top(depths[i], il);
      let nh_val = nh(il);
      let rect = [(cx - node_w / 2, ty), (cx + node_w / 2, ty + nh_val)];
      let fill = if il { RGBColor(173, 216, 230) } else { RGBColor(144, 238, 144) };
      root.draw(&Rectangle::new(rect, fill.filled()))?;
      root.draw(&Rectangle::new(rect, BLACK.stroke_width(1)))?;

      // text block vertically centred; each line horizontally centred at cx.
      // block height = spacing between first and last baseline + one glyph height.
      let lines    = &labels[i];
      let text_h   = (lines.len() as i32 - 1) * LINE_H + FONT_PX;
      let text_top = ty + (nh_val - text_h) / 2;
      for (j, line) in lines.iter().enumerate() {
        root.draw(&Text::new(line.clone(), (cx, text_top + j as i32 * LINE_H), font.clone()))?;
      }
    }

    root.present()?;
    Ok(())
  }
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
        println!("{}, {}", tree.leaf_data[0][0], tree.leaf_data[0][1]);
        println!("{}, {}", tree.leaf_data[1][0], tree.leaf_data[1][1]);
        println!("{}, {}", tree.leaf_data[2][0], tree.leaf_data[2][1]);
        assert_eq!(tree.depth, 3);
        assert_eq!(tree.nodes.len(), 5);
        assert_eq!(tree.leaf_data.len(), 3);
    }
}
