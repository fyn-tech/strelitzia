//! Axis-aligned bounding box primitive.

use crate::multiarray::*;
use num_traits::{Bounded, Zero};
use std::fmt::Debug;

pub trait Scalar: Copy + Clone + PartialOrd + Debug + Default + 'static + Bounded + Zero {}
impl<T: Copy + Clone + PartialOrd + Debug + Default + 'static + Bounded + Zero> Scalar for T {}

/// Classification of the spatial relationship between two [`AABBox`] instances.
#[derive(Debug, PartialEq)]
pub enum Overlap {
    /// The boxes do not touch or overlap.
    Disjoint,
    /// The boxes partially overlap but neither fully contains the other.
    Intersecting,
    /// One box fully contains the other, including the case where they are equal.
    Containing,
}

/// An axis-aligned bounding box in `D` dimensions.
///
/// Initialised via the builder: `AABBox::new().min(&min_pt).max(&max_pt)`.
/// For growing a box from a point cloud, start with `AABBox::new()` (which
/// sets `min` to `T::MAX` and `max` to `T::MIN`) and call `expand` per point.
#[derive(Debug, Clone)]
pub struct AABBox<T: Scalar, const D: usize> {
    pub min: Vector<T, D>,
    pub max: Vector<T, D>,
}
    
impl<T: Scalar, const D: usize> AABBox<T, D> {
    // --- Construction --------------------------------------------------------

    /// Returns an inverted box (`min = T::MAX`, `max = T::MIN`) ready for
    /// expansion via [`expand`](Self::expand).
    pub fn new() -> Self {
        Self {
            min: Vector::from_slice(&[T::max_value(); D]),
            max: Vector::from_slice(&[T::min_value(); D]),
        }
    }

    /// Sets `max` corner. Intended for builder-style construction.
    pub fn max(mut self, point: &Vector<T, D>) -> Self {
        self.max = *point;
        self
    }

    /// Sets `min` corner. Intended for builder-style construction.
    pub fn min(mut self, point: &Vector<T, D>) -> Self {
        self.min = *point;
        self
    }

    // --- Queries -------------------------------------------------------------

    /// Returns `true` if `point` lies within the box.
    ///
    /// When `inclusive` is `true`, points on the boundary are considered
    /// inside. When `false`, the boundary is excluded.
    pub fn contains(&self, point: &Vector<T, D>, inclusive: bool) -> bool {
        for d in 0..D {
            if point[d] < self.min[d] || (!inclusive && point[d] == self.min[d]) {
                return false;
            }
            if point[d] > self.max[d] || (!inclusive && point[d] == self.max[d]) {
                return false;
            }
        }
        true
    }

    /// Returns `true` if this box overlaps `other`.
    ///
    /// Computes the intersection box and checks whether it is non-degenerate.
    /// `inclusive` controls whether touching boundaries count as intersection.
    pub fn intersects(&self, other: &AABBox<T, D>, inclusive: bool) -> Overlap {
        let mut ordinate_use_count: i32 = 0;
        let mut intersection: AABBox<T, D> = AABBox::new();
        for d in 0..D {
            intersection.min[d] = if self.min[d] > other.min[d] {
                ordinate_use_count += 1;
                self.min[d]
            } else {
                ordinate_use_count -= 1;
                other.min[d]
            };
            intersection.max[d] = if self.max[d] < other.max[d] {
                ordinate_use_count += 1;
                self.max[d]
            } else {
                ordinate_use_count -= 1;
                other.max[d]
            };
        }
        
        if intersection.is_degenerate(inclusive) { return Overlap::Disjoint; }
        else {
            if ordinate_use_count.abs() == (2 * D) as i32 { return Overlap::Containing; }
            else {return Overlap::Intersecting; }
        }
    }

    /// Returns `true` if the box has zero or negative volume in any dimension.
    ///
    /// When `inclusive` is `false`, a flat box (`min == max` in any dimension)
    /// is also considered degenerate.
    pub fn is_degenerate(&self, inclusive: bool) -> bool {
        for d in 0..D {
            if self.min[d] > self.max[d] || (!inclusive && self.min[d] == self.max[d]) {
                return true;
            }
        }
        false
    }

    // --- Mutations -----------------------------------------------------------

    /// Grows the box to include `point`. Has no effect if `point` is already inside.
    pub fn expand(&mut self, point: &Vector<T, D>) {
        for d in 0..D {
            self.min[d] = if point[d] < self.min[d] {
                point[d]
            } else {
                self.min[d]
            };
            self.max[d] = if point[d] > self.max[d] {
                point[d]
            } else {
                self.max[d]
            };
        }
    }
}

// --- Free functions ----------------------------------------------------------

/// Returns the smallest box enclosing both `a` and `b`.
pub fn merge<T: Scalar, const D: usize>(a: &AABBox<T, D>, b: &AABBox<T, D>) -> AABBox<T, D> {
    let mut result: AABBox<T, D> = AABBox::new();
    for d in 0..D {
        result.min[d] = if a.min[d] < b.min[d] { a.min[d] } else { b.min[d] };
        result.max[d] = if a.max[d] > b.max[d] { a.max[d] } else { b.max[d] };
    }
    result
}

/// Clips `bound_box` to one side of a splitting hyperplane per dimension, 
/// returning the clipped result.
///
/// For each dimension `d`, `split_point[d]` defines the split value.
/// If `keep_left[d]` is `true`, `max[d]` is clipped down to `split_point[d]`;
/// otherwise `min[d]` is clipped up. Split values outside the current extents
/// have no effect.
pub fn split<T: Scalar, const D: usize>(
    bound_box: &AABBox<T, D>,
    split_point: &Vector<T, D>,
    keep_left: &[bool; D],
) -> AABBox<T, D> {
    let mut result = bound_box.clone();
    for d in 0..D {
        if keep_left[d] {
            if split_point[d] < result.max[d] {
                result.max[d] = split_point[d];
            }
        } else if split_point[d] > result.min[d] {
            result.min[d] = split_point[d];
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box(min: [f64; 2], max: [f64; 2]) -> AABBox<f64, 2> {
        AABBox::new()
            .min(&Vector::from_slice(&min))
            .max(&Vector::from_slice(&max))
    }

    // --- Queries: contains ---------------------------------------------------

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
        let p = Vector::from_slice(&[0.0, 1.0]);
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

    // --- Queries: intersects -------------------------------------------------

    #[test]
    fn intersects_containing_self_contains_other() {
        let a = make_box([0.0, 0.0], [3.0, 3.0]);
        let b = make_box([1.0, 1.0], [2.0, 2.0]);
        assert_eq!(a.intersects(&b, true),  Overlap::Containing);
        assert_eq!(a.intersects(&b, false), Overlap::Containing);
    }

    #[test]
    fn intersects_containing_other_contains_self() {
        let a = make_box([1.0, 1.0], [2.0, 2.0]);
        let b = make_box([0.0, 0.0], [3.0, 3.0]);
        assert_eq!(a.intersects(&b, true),  Overlap::Containing);
        assert_eq!(a.intersects(&b, false), Overlap::Containing);
    }

    #[test]
    fn intersects_containing_equal_boxes() {
        let a = make_box([0.0, 0.0], [2.0, 2.0]);
        let b = make_box([0.0, 0.0], [2.0, 2.0]);
        assert_eq!(a.intersects(&b, true),  Overlap::Containing);
        assert_eq!(a.intersects(&b, false), Overlap::Containing);
    }

    #[test]
    fn intersects_disjoint() {
        let a = make_box([0.0, 0.0], [1.0, 1.0]);
        let b = make_box([2.0, 0.0], [3.0, 1.0]);
        assert_eq!(a.intersects(&b, true),  Overlap::Disjoint);
        assert_eq!(a.intersects(&b, false), Overlap::Disjoint);
    }

    #[test]
    fn intersects_overlapping() {
        let a = make_box([0.0, 0.0], [2.0, 2.0]);
        let b = make_box([1.0, 1.0], [3.0, 3.0]);
        assert_eq!(a.intersects(&b, true),  Overlap::Intersecting);
        assert_eq!(a.intersects(&b, false), Overlap::Intersecting);
    }

    #[test]
    fn intersects_touching_boundary() {
        let a = make_box([0.0, 0.0], [1.0, 1.0]);
        let b = make_box([1.0, 0.0], [2.0, 1.0]);
        assert_eq!(a.intersects(&b, true),  Overlap::Intersecting);
        assert_eq!(a.intersects(&b, false), Overlap::Disjoint);
    }

    // --- Queries: is_degenerate ----------------------------------------------

    #[test]
    fn is_degenerate_flat_exclusive() {
        let b = make_box([1.0, 1.0], [1.0, 2.0]);
        assert!(b.is_degenerate(false));
        assert!(!b.is_degenerate(true));
    }

    #[test]
    fn is_degenerate_inverted() {
        let b = make_box([1.0, 1.0], [0.0, 2.0]);
        assert!(b.is_degenerate(true));
    }

    #[test]
    fn is_degenerate_not() {
        let b = make_box([0.0, 0.0], [1.0, 1.0]);
        assert!(!b.is_degenerate(true));
        assert!(!b.is_degenerate(false));
    }

    // --- Mutations: expand ---------------------------------------------------

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
        assert_eq!(b.min[0], 0.0);
        assert_eq!(b.min[1], 0.0);
        assert_eq!(b.max[0], 2.0);
        assert_eq!(b.max[1], 2.0);
    }

    // --- Free functions: merge -----------------------------------------------

    #[test]
    fn merge_two_boxes() {
        let a = make_box([0.0, 1.0], [2.0, 3.0]);
        let b = make_box([-1.0, 0.0], [1.0, 4.0]);
        let m = merge(&a, &b);
        assert_eq!(m.min[0], -1.0);
        assert_eq!(m.min[1], 0.0);
        assert_eq!(m.max[0], 2.0);
        assert_eq!(m.max[1], 4.0);
    }

    // --- Free functions: split -----------------------------------------------

    #[test]
    fn split_keep_left_upper() {
        let b = make_box([0.0, 0.0], [4.0, 4.0]);
        let result = split(&b, &Vector::from_slice(&[2.0, 2.0]), &[true, false]);
        assert_eq!(result.min[0], 0.0); // unchanged
        assert_eq!(result.min[1], 2.0); // clipped
        assert_eq!(result.max[0], 2.0); // clipped
        assert_eq!(result.max[1], 4.0); // unchanged
    }

    #[test]
    fn split_point_outside_no_effect() {
        let b = make_box([0.0, 0.0], [4.0, 4.0]);
        let result = split(&b, &Vector::from_slice(&[5.0, 5.0]), &[true, true]);
        assert_eq!(result.max[0], 4.0);
        assert_eq!(result.max[1], 4.0);
    }
}
