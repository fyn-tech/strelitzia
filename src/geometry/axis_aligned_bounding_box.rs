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

    fn min(&mut self, point: &Vector<T, D>) -> Self {
        self.min = *point;
        self
    }
    
    fn max(&mut self, point: &Vector<T, D>) -> Self {
        self.max = *point;
        self
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
            box_2.min[D] = if self.min[d] > other.min[d] { self.min[d] } else { other.min[d] };
            box_2.max[D] = if self.max[d] < other.max[d] { self.max[d] } else { other.max[d] };
        }
        box_2.is_degenerate(inclusive)
    }

    fn expand(&mut self, point: &Vector<T, D>) {
        for d in 0..D {
            self.min[D] = if point[d] < self.min[d] { point[d] } else { self.min[d] };
            self.max[D] = if point[d] > self.max[d] { point[d] } else { self.max[d] };
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

    fn is_degenerate(&self, inclusive: bool) -> bool {
        for d in 0..D {
            if self.min[d] > self.max[d] || (!inclusive && self.min[d] == self.max[d]) { return true; }
        }
        false
    }

}


fn merge<T: Scalar + Bounded + Zero, const D: usize>(box_0: &AABBox<T, D>, box_1: &AABBox<T, D>) -> AABBox<T, D>{
    let mut box_2 : AABBox<T, D> = AABBox::new();
    for d in 0..D {
        box_2.min[D] = if  box_0.min[d] < box_1.min[d] { box_0.min[d] } else { box_1.min[d] };
        box_2.max[D] = if box_0.max[d] > box_1.max[d] { box_0.max[d] } else { box_1.max[d] };
    }
    box_2
}