//! Axis-aligned bounding box primitive.

use crate::multiarray::*;
use num_traits::{Bounded, Zero};
use std::fmt::Debug;

pub trait Scalar: Copy + Clone + PartialOrd + Debug + Default + 'static {}
impl<T: Copy + Clone + PartialOrd + Debug + Default + 'static> Scalar for T {}

/// An axis-aligned bounding box in `D` dimensions.
///
/// Initialised via the builder: `AABBox::new().min(&min_pt).max(&max_pt)`.
/// For growing a box from a point cloud, start with `AABBox::new()` (which
/// sets `min` to `T::MAX` and `max` to `T::MIN`) and call `expand` per point.
#[derive(Debug, Clone)]
pub struct AABBox<T: Scalar + Bounded + Zero, const D: usize> {
    pub min: Vector<T, D>,
    pub max: Vector<T, D>,
}

impl<T: Scalar + Bounded + Zero, const D: usize> AABBox<T, D> {
    // --- Construction --------------------------------------------------------

    /// Returns an inverted box (`min = T::MAX`, `max = T::MIN`) ready for
    /// expansion via [`expand`](Self::expand).
    fn new() -> Self {
        Self {
            min: Vector::from_slice(&[T::max_value(); D]),
            max: Vector::from_slice(&[T::min_value(); D]),
        }
    }

    /// Sets `max` corner. Intended for builder-style construction.
    fn max(mut self, point: &Vector<T, D>) -> Self {
        self.max = *point;
        self
    }

    /// Sets `min` corner. Intended for builder-style construction.
    fn min(mut self, point: &Vector<T, D>) -> Self {
        self.min = *point;
        self
    }

    // --- Queries -------------------------------------------------------------

    /// Returns `true` if `point` lies within the box.
    ///
    /// When `inclusive` is `true`, points on the boundary are considered
    /// inside. When `false`, the boundary is excluded.
    fn contains(&self, point: &Vector<T, D>, inclusive: bool) -> bool {
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
    fn intersects(&self, other: &AABBox<T, D>, inclusive: bool) -> bool {
        let mut intersection: AABBox<T, D> = AABBox::new();
        for d in 0..D {
            intersection.min[d] = if self.min[d] > other.min[d] {
                self.min[d]
            } else {
                other.min[d]
            };
            intersection.max[d] = if self.max[d] < other.max[d] {
                self.max[d]
            } else {
                other.max[d]
            };
        }
        !intersection.is_degenerate(inclusive)
    }

    /// Returns `true` if the box has zero or negative volume in any dimension.
    ///
    /// When `inclusive` is `false`, a flat box (`min == max` in any dimension)
    /// is also considered degenerate.
    fn is_degenerate(&self, inclusive: bool) -> bool {
        for d in 0..D {
            if self.min[d] > self.max[d] || (!inclusive && self.min[d] == self.max[d]) {
                return true;
            }
        }
        false
    }

    // --- Mutations -----------------------------------------------------------

    /// Grows the box to include `point`. Has no effect if `point` is already inside.
    fn expand(&mut self, point: &Vector<T, D>) {
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

    /// Clips the box to one side of a splitting hyperplane per dimension.
    ///
    /// For each dimension `d`, `split_point[d]` defines the split value.
    /// If `keep_left[d]` is `true`, `max[d]` is clipped down to `split_point[d]`;
    /// otherwise `min[d]` is clipped up. Split values outside the current
    /// extents have no effect.
    fn split(&mut self, split_point: &Vector<T, D>, keep_left: &Vector<bool, D>) {
        for d in 0..D {
            if keep_left[d] {
                if split_point[d] < self.max[d] {
                    self.max[d] = split_point[d];
                }
            } else {
                if split_point[d] > self.min[d] {
                    self.min[d] = split_point[d];
                }
            }
        }
    }
}

// --- Free functions ----------------------------------------------------------

/// Returns the smallest box enclosing both `a` and `b`.
fn merge<T: Scalar + Bounded + Zero, const D: usize>(
    a: &AABBox<T, D>,
    b: &AABBox<T, D>,
) -> AABBox<T, D> {
    let mut result: AABBox<T, D> = AABBox::new();
    for d in 0..D {
        result.min[d] = if a.min[d] < b.min[d] {
            a.min[d]
        } else {
            b.min[d]
        };
        result.max[d] = if a.max[d] > b.max[d] {
            a.max[d]
        } else {
            b.max[d]
        };
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

    fn make_keep(x: bool, y: bool) -> Vector<bool, 2> {
        Vector::<bool, 2>::from_inner(nalgebra::SVector::<bool, 2>::from([x, y]))
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
    fn intersects_contained() {
        let a = make_box([0.0, 0.0], [3.0, 3.0]);
        let b = make_box([1.0, 1.0], [2.0, 2.0]);
        assert!(a.intersects(&b, true));
        assert!(a.intersects(&b, false));
    }

    #[test]
    fn intersects_disjoint() {
        let a = make_box([0.0, 0.0], [1.0, 1.0]);
        let b = make_box([2.0, 0.0], [3.0, 1.0]);
        assert!(!a.intersects(&b, true));
        assert!(!a.intersects(&b, false));
    }

    #[test]
    fn intersects_overlapping() {
        let a = make_box([0.0, 0.0], [2.0, 2.0]);
        let b = make_box([1.0, 1.0], [3.0, 3.0]);
        assert!(a.intersects(&b, true));
        assert!(a.intersects(&b, false));
    }

    #[test]
    fn intersects_touching_boundary() {
        let a = make_box([0.0, 0.0], [1.0, 1.0]);
        let b = make_box([1.0, 0.0], [2.0, 1.0]);
        assert!(a.intersects(&b, true));
        assert!(!a.intersects(&b, false));
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

    // --- Mutations: split ----------------------------------------------------

    #[test]
    fn split_keep_left_upper() {
        let mut b = make_box([0.0, 0.0], [4.0, 4.0]);
        let keep_side = make_keep(true, false);
        b.split(&Vector::from_slice(&[2.0, 2.0]), &keep_side);
        assert_eq!(b.min[0], 0.0); // unchanged
        assert_eq!(b.min[1], 2.0); // clipped
        assert_eq!(b.max[0], 2.0); // clipped
        assert_eq!(b.max[1], 4.0); // unchanged
    }

    #[test]
    fn split_point_outside_no_effect() {
        let mut b = make_box([0.0, 0.0], [4.0, 4.0]);
        b.split(&Vector::from_slice(&[5.0, 5.0]), &make_keep(true, true));
        assert_eq!(b.max[0], 4.0);
        assert_eq!(b.max[1], 4.0);
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
}
