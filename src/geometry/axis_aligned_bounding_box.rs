// use nalgebra::indexing;

use num_traits::{Bounded, Zero};
use crate::multiarray::*;
use std::{fmt::Debug, usize};

pub trait Scalar: Copy + Clone + PartialOrd + Debug + Default + 'static {}
impl<T: Copy + Clone + PartialOrd + Debug + Default + 'static> Scalar for T {}

// ============================================================================
// The Axis Aligned Bounding Box
// ============================================================================

#[derive(Debug, Clone)]
pub struct AABBox<T: Scalar + Bounded + Zero, const D: usize> {
  pub min: Vector<T, D>,
  pub max: Vector<T, D>,
}

impl <T: Scalar + Bounded + Zero, const D: usize> AABBox<T, D> {

    fn new() -> Self {
        Self {
            min: Vector::from_slice(&[T::min_value(); D]),            
            max: Vector::from_slice(&[T::max_value(); D]),
        }
    }

    fn min(mut self, point: &Vector<T, D>) -> Self {
        self.min = *point;
        self
    }

    fn max(mut self, point: &Vector<T, D>) -> Self {
        self.max = *point;
        self
    }

    fn is_degenerate(&self, inclusive: bool) -> bool {
        for d in 0..D {
            if self.min[d] > self.max[d] || (!inclusive && self.min[d] == self.max[d]) { return true; }
        }
        false
    }

    fn contains(&self, point: &Vector<T, D>, inclusive: bool) -> bool {
        for d in 0..D {
            if point[d] < self.min[d] || (!inclusive && point[d] == self.min[d]) { return false; } 
            if point[d] > self.max[d] || (!inclusive && point[d] == self.max[d]) { return false; } 
        }
        true
    }

    fn intersects(&self, other: &AABBox<T, D>, inclusive: bool) -> bool {
        let mut box_2 : AABBox<T, D> = AABBox::new();
        for d in 0..D {
            box_2.min[d] = if self.min[d] > other.min[d] { self.min[d] } else { other.min[d] };
            box_2.max[d] = if self.max[d] < other.max[d] { self.max[d] } else { other.max[d] };
        }
        !box_2.is_degenerate(inclusive)
    }

    fn expand(&mut self, point: &Vector<T, D>) {
        for d in 0..D {
            self.min[d] = if point[d] < self.min[d] { point[d] } else { self.min[d] };
            self.max[d] = if point[d] > self.max[d] { point[d] } else { self.max[d] };
        }
    }


    fn split(&mut self, split_point: &Vector<T, D>, keep_left: &Vector<bool, D>) {
        for d in 0..D {
            if keep_left[d] {
                if split_point[d] < self.max[d] { self.max[d] = split_point[d]; }
            } else {
                if split_point[d] > self.min[d] { self.min[d] = split_point[d]; }
            }
        }
    }


}


fn merge<T: Scalar + Bounded + Zero, const D: usize>(box_0: &AABBox<T, D>, box_1: &AABBox<T, D>) -> AABBox<T, D>{
    let mut box_2 : AABBox<T, D> = AABBox::new();
    for d in 0..D {
        box_2.min[d] = if box_0.min[d] < box_1.min[d] { box_0.min[d] } else { box_1.min[d] };
        box_2.max[d] = if box_0.max[d] > box_1.max[d] { box_0.max[d] } else { box_1.max[d] };
    }
    box_2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box(min: [f64; 2], max: [f64; 2]) -> AABBox<f64, 2> {
        AABBox::new()
            .min(&Vector::from_slice(&min))
            .max(&Vector::from_slice(&max))
    }

    // --- is_degenerate -------------------------------------------------------

    #[test]
    fn degenerate_inverted() {
        let b = make_box([1.0, 1.0], [0.0, 2.0]);  // min.x > max.x
        assert!(b.is_degenerate(true));
    }

    #[test]
    fn degenerate_flat_exclusive() {
        let b = make_box([1.0, 1.0], [1.0, 2.0]);  // zero width in x
        assert!(b.is_degenerate(false));
        assert!(!b.is_degenerate(true));             // flat but valid when inclusive
    }

    #[test]
    fn not_degenerate() {
        let b = make_box([0.0, 0.0], [1.0, 1.0]);
        assert!(!b.is_degenerate(true));
        assert!(!b.is_degenerate(false));
    }

    // --- contains ------------------------------------------------------------

    #[test]
    fn contains_interior() {
        let b = make_box([0.0, 0.0], [2.0, 2.0]);
        let p = Vector::from_slice(&[1.0, 1.0]);
        assert!(b.contains(&p, false));
        assert!(b.contains(&p, true));
    }

    #[test]
    fn contains_on_boundary_inclusive() {
        let b = make_box([0.0, 0.0], [2.0, 2.0]);
        let p = Vector::from_slice(&[0.0, 1.0]);  // on min boundary
        assert!(b.contains(&p, true));
        assert!(!b.contains(&p, false));
    }

    #[test]
    fn contains_outside() {
        let b = make_box([0.0, 0.0], [2.0, 2.0]);
        let p = Vector::from_slice(&[3.0, 1.0]);
        assert!(!b.contains(&p, true));
        assert!(!b.contains(&p, false));
    }

    // --- expand --------------------------------------------------------------

    #[test]
    fn expand_grows_box() {
        let mut b = make_box([0.0, 0.0], [1.0, 1.0]);
        b.expand(&Vector::from_slice(&[2.0, -1.0]));
        assert_eq!(b.min[0], 0.0);
        assert_eq!(b.min[1], -1.0);
        assert_eq!(b.max[0], 2.0);
        assert_eq!(b.max[1], 1.0);
    }

    #[test]
    fn expand_interior_point_no_change() {
        let mut b = make_box([0.0, 0.0], [2.0, 2.0]);
        b.expand(&Vector::from_slice(&[1.0, 1.0]));
        assert_eq!(b.min[0], 0.0); assert_eq!(b.min[1], 0.0);
        assert_eq!(b.max[0], 2.0); assert_eq!(b.max[1], 2.0);
    }

    // --- intersects ----------------------------------------------------------

    #[test]
    fn intersects_overlapping() {
        let a = make_box([0.0, 0.0], [2.0, 2.0]);
        let b = make_box([1.0, 1.0], [3.0, 3.0]);
        assert!(a.intersects(&b, true));
        assert!(a.intersects(&b, false));
    }

        #[test]
    fn intersects_contained() {
        let a = make_box([0.0, 0.0], [3.0, 3.0]);
        let b = make_box([1.0, 1.0], [2.0, 2.0]);
        assert!(a.intersects(&b, true));
        assert!(a.intersects(&b, false));
    }

    #[test]
    fn intersects_touching_boundary() {
        let a = make_box([0.0, 0.0], [1.0, 1.0]);
        let b = make_box([1.0, 0.0], [2.0, 1.0]);  // touch at x=1
        assert!(a.intersects(&b, true));
        assert!(!a.intersects(&b, false));
    }

    #[test]
    fn intersects_disjoint() {
        let a = make_box([0.0, 0.0], [1.0, 1.0]);
        let b = make_box([2.0, 0.0], [3.0, 1.0]);
        assert!(!a.intersects(&b, true));
        assert!(!a.intersects(&b, false));
    }

    // --- merge ---------------------------------------------------------------

    #[test]
    fn merge_two_boxes() {
        let a = make_box([0.0, 1.0], [2.0, 3.0]);
        let b = make_box([-1.0, 0.0], [1.0, 4.0]);
        let m = merge(&a, &b);
        assert_eq!(m.min[0], -1.0); 
        assert_eq!(m.min[1], 0.0);
        assert_eq!(m.max[0],  2.0); 
        assert_eq!(m.max[1], 4.0);
    }

    // --- split ---------------------------------------------------------------

    #[test]
    fn split_keep_left_upper() {
        let mut b = make_box([0.0, 0.0], [4.0, 4.0]);
        let p = Vector::from_slice(&[2.0, 2.0]);
        let keep = Vector::<bool, 2>::from_inner(nalgebra::SVector::<bool, 2>::from([true, false]));
        b.split(&p, &keep);
        assert_eq!(b.min[0], 0.0);  // unchanged
        assert_eq!(b.min[1], 2.0);  // clipped
        assert_eq!(b.max[0], 2.0);  // clipped
        assert_eq!(b.max[1], 4.0);  // unchanged
    }

    #[test]
    fn split_point_outside_no_effect() {
        let mut b = make_box([0.0, 0.0], [4.0, 4.0]);
        let p = Vector::from_slice(&[5.0, 5.0]);  // outside max
        let keep = Vector::<bool, 2>::from_inner(nalgebra::SVector::<bool, 2>::from([true, true]));
        b.split(&p, &keep);
        assert_eq!(b.max[0], 4.0);  // split point > max, no clip
        assert_eq!(b.max[1], 4.0);
    }
}