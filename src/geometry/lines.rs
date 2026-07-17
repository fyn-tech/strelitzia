use crate::common::{Real, approx_eq, approx_gte, approx_lte};
use crate::multiarray::Vector;
use crate::multiarray::linalg::{cross, cross2d, dot, l2_norm, normalised};

/// Common accessors shared by [`Line`], [`Ray`], and [`Segment`] so the
/// intersection kernel below can work generically over all three.
trait PointDirection<const D: usize> {
    fn point(&self) -> Vector<Real, D>;
    fn direction(&self) -> Vector<Real, D>;
}

/// Generates `new`/`from_points` constructors and the [`PointDirection`] impl
/// for a unit-direction point+direction primitive (`Line`/`Ray`). `direction`
/// is normalised by `new`; `from_points` delegates to `new` with
/// `direction = b - a`. Panics if the given direction is zero-length.
macro_rules! impl_unit_direction_type {
    ($ty:ident) => {
        impl<const D: usize> $ty<D> {
            pub fn new(origin: Vector<Real, D>, direction: Vector<Real, D>) -> Self {
                assert!(
                    !approx_eq(l2_norm(&direction), 0.0),
                    "{}::new: direction must be non-zero",
                    stringify!($ty)
                );
                Self {
                    point: origin,
                    direction: normalised(&direction),
                }
            }

            pub fn from_points(a: Vector<Real, D>, b: Vector<Real, D>) -> Self {
                Self::new(a, b - a)
            }
        }

        impl<const D: usize> PointDirection<D> for $ty<D> {
            fn point(&self) -> Vector<Real, D> {
                self.point
            }
            fn direction(&self) -> Vector<Real, D> {
                self.direction
            }
        }
    };
}

/// An infinite line: unconstrained in both directions along `direction`,
/// which is always unit length.
pub struct Line<const D: usize> {
    point: Vector<Real, D>,
    direction: Vector<Real, D>,
}

/// A ray: starts at `point` and extends along `direction` (always unit
/// length) for `t >= 0`.
pub struct Ray<const D: usize> {
    point: Vector<Real, D>,
    direction: Vector<Real, D>,
}

impl_unit_direction_type!(Line);
impl_unit_direction_type!(Ray);

/// A bounded segment between `point` and `other_point`; valid for `0 <= t <= 1`.
pub struct Segment<const D: usize> {
    point: Vector<Real, D>,
    other_point: Vector<Real, D>,
}

impl<const D: usize> Segment<D> {
    pub fn new(point: Vector<Real, D>, direction: Vector<Real, D>) -> Self {
        Self {
            point,
            other_point: point + direction,
        }
    }

    pub fn from_points(a: Vector<Real, D>, b: Vector<Real, D>) -> Self {
        Self::new(a, b - a)
    }
}

impl<const D: usize> PointDirection<D> for Segment<D> {
    fn point(&self) -> Vector<Real, D> {
        self.point
    }
    fn direction(&self) -> Vector<Real, D> {
        self.other_point - self.point
    }
}

// ============================================================================
// Kernel -- solves for the two intersection parameters. Generic over any mix
// of Line/Ray/Segment; every variant below is a thin wrapper around this
// that differs only in which range of `t`/`s` it considers valid.
// ============================================================================

/// Solves `l0.point + t*l0.direction == l1.point + s*l1.direction` for `(t, s)`.
/// Returns `None` if the directions are parallel (including a zero-length
/// direction on either side). Does not consider ray/segment bounds -- callers
/// apply their own range checks to `t` and `s`.
fn intersect_params_2d<A: PointDirection<2>, B: PointDirection<2>>(
    l0: &A,
    l1: &B,
) -> Option<(Real, Real)> {
    let denom = cross2d(&l0.direction(), &l1.direction());
    if approx_eq(denom, 0.0) {
        return None; // parallel (or a zero-length direction)
    }
    let vec_01 = l1.point() - l0.point();
    let t = cross2d(&vec_01, &l1.direction()) / denom;
    let s = cross2d(&vec_01, &l0.direction()) / denom;
    Some((t, s))
}

/// Solves `l0.point + t*l0.direction == l1.point + s*l1.direction` for `(t, s)`.
/// Returns `None` if the lines are skew (not coplanar) or if the directions
/// are parallel (including a zero-length direction on either side).
fn intersect_params_3d<A: PointDirection<3>, B: PointDirection<3>>(
    l0: &A,
    l1: &B,
) -> Option<(Real, Real)> {
    let plane_norm = cross(&l0.direction(), &l1.direction());
    let vec_01 = l1.point() - l0.point();
    if !approx_eq(dot(&vec_01, &plane_norm), 0.0) {
        return None; // skew, not coplanar
    }
    let norm = dot(&plane_norm, &plane_norm);
    if approx_eq(norm, 0.0) {
        return None; // parallel (or a zero-length direction)
    }
    let t = dot(&cross(&vec_01, &l1.direction()), &plane_norm) / norm;
    let s = dot(&cross(&vec_01, &l0.direction()), &plane_norm) / norm;
    Some((t, s))
}

/// Point at parameter `t` along a point+direction primitive.
fn point_at<const D: usize, A: PointDirection<D>>(obj: &A, t: Real) -> Vector<Real, D> {
    obj.point() + t * obj.direction()
}

// ============================================================================
// Parameter range predicates -- the only thing that distinguishes a ray,
// a line, and a segment once the intersection parameters are known.
// A "line" side has no predicate: every parameter value is valid.
// ============================================================================

fn in_ray_param(t: Real) -> bool {
    approx_gte(t, 0.0)
}

fn in_segment_param(t: Real) -> bool {
    approx_gte(t, 0.0) && approx_lte(t, 1.0)
}

// ============================================================================
// Ray -- Ray
// ============================================================================

pub fn ray_ray_intersection_2d(r0: &Ray<2>, r1: &Ray<2>) -> Option<Vector<Real, 2>> {
    let (t, s) = intersect_params_2d(r0, r1)?;
    if !in_ray_param(t) || !in_ray_param(s) {
        return None;
    }
    Some(point_at(r0, t))
}

pub fn ray_ray_intersection_3d(r0: &Ray<3>, r1: &Ray<3>) -> Option<Vector<Real, 3>> {
    let (t, s) = intersect_params_3d(r0, r1)?;
    if !in_ray_param(t) || !in_ray_param(s) {
        return None;
    }
    Some(point_at(r0, t))
}

// ============================================================================
// Ray -- Line
// ============================================================================

pub fn ray_line_intersection_2d(ray: &Ray<2>, line: &Line<2>) -> Option<Vector<Real, 2>> {
    let (t, _s) = intersect_params_2d(ray, line)?;
    if !in_ray_param(t) {
        return None;
    }
    Some(point_at(ray, t))
}

pub fn ray_line_intersection_3d(ray: &Ray<3>, line: &Line<3>) -> Option<Vector<Real, 3>> {
    let (t, _s) = intersect_params_3d(ray, line)?;
    if !in_ray_param(t) {
        return None;
    }
    Some(point_at(ray, t))
}

// ============================================================================
// Ray -- Segment
// ============================================================================

pub fn ray_segment_intersection_2d(ray: &Ray<2>, segment: &Segment<2>) -> Option<Vector<Real, 2>> {
    let (t, s) = intersect_params_2d(ray, segment)?;
    if !in_ray_param(t) || !in_segment_param(s) {
        return None;
    }
    Some(point_at(ray, t))
}

pub fn ray_segment_intersection_3d(ray: &Ray<3>, segment: &Segment<3>) -> Option<Vector<Real, 3>> {
    let (t, s) = intersect_params_3d(ray, segment)?;
    if !in_ray_param(t) || !in_segment_param(s) {
        return None;
    }
    Some(point_at(ray, t))
}

// ============================================================================
// Line -- Segment
// ============================================================================

pub fn line_segment_intersection_2d(
    line: &Line<2>,
    segment: &Segment<2>,
) -> Option<Vector<Real, 2>> {
    let (t, s) = intersect_params_2d(line, segment)?;
    if !in_segment_param(s) {
        return None;
    }
    Some(point_at(line, t))
}

pub fn line_segment_intersection_3d(
    line: &Line<3>,
    segment: &Segment<3>,
) -> Option<Vector<Real, 3>> {
    let (t, s) = intersect_params_3d(line, segment)?;
    if !in_segment_param(s) {
        return None;
    }
    Some(point_at(line, t))
}

// ============================================================================
// Unified intersection interface -- one method name, dispatched by the
// concrete type of `Rhs`. Each impl is a one-line delegation to the
// corresponding `_2d`/`_3d` function above (matching or swapping argument
// order as needed); all the real logic lives in the kernel/predicates.
// ============================================================================

pub trait Intersect<Rhs, const D: usize> {
    fn intersect(&self, other: &Rhs) -> Option<Vector<Real, D>>;
}

impl Intersect<Ray<2>, 2> for Ray<2> {
    fn intersect(&self, other: &Ray<2>) -> Option<Vector<Real, 2>> {
        ray_ray_intersection_2d(self, other)
    }
}
impl Intersect<Ray<3>, 3> for Ray<3> {
    fn intersect(&self, other: &Ray<3>) -> Option<Vector<Real, 3>> {
        ray_ray_intersection_3d(self, other)
    }
}

impl Intersect<Line<2>, 2> for Ray<2> {
    fn intersect(&self, other: &Line<2>) -> Option<Vector<Real, 2>> {
        ray_line_intersection_2d(self, other)
    }
}
impl Intersect<Ray<2>, 2> for Line<2> {
    fn intersect(&self, other: &Ray<2>) -> Option<Vector<Real, 2>> {
        ray_line_intersection_2d(other, self)
    }
}
impl Intersect<Line<3>, 3> for Ray<3> {
    fn intersect(&self, other: &Line<3>) -> Option<Vector<Real, 3>> {
        ray_line_intersection_3d(self, other)
    }
}
impl Intersect<Ray<3>, 3> for Line<3> {
    fn intersect(&self, other: &Ray<3>) -> Option<Vector<Real, 3>> {
        ray_line_intersection_3d(other, self)
    }
}

impl Intersect<Segment<2>, 2> for Ray<2> {
    fn intersect(&self, other: &Segment<2>) -> Option<Vector<Real, 2>> {
        ray_segment_intersection_2d(self, other)
    }
}
impl Intersect<Ray<2>, 2> for Segment<2> {
    fn intersect(&self, other: &Ray<2>) -> Option<Vector<Real, 2>> {
        ray_segment_intersection_2d(other, self)
    }
}
impl Intersect<Segment<3>, 3> for Ray<3> {
    fn intersect(&self, other: &Segment<3>) -> Option<Vector<Real, 3>> {
        ray_segment_intersection_3d(self, other)
    }
}
impl Intersect<Ray<3>, 3> for Segment<3> {
    fn intersect(&self, other: &Ray<3>) -> Option<Vector<Real, 3>> {
        ray_segment_intersection_3d(other, self)
    }
}

impl Intersect<Segment<2>, 2> for Line<2> {
    fn intersect(&self, other: &Segment<2>) -> Option<Vector<Real, 2>> {
        line_segment_intersection_2d(self, other)
    }
}
impl Intersect<Line<2>, 2> for Segment<2> {
    fn intersect(&self, other: &Line<2>) -> Option<Vector<Real, 2>> {
        line_segment_intersection_2d(other, self)
    }
}
impl Intersect<Segment<3>, 3> for Line<3> {
    fn intersect(&self, other: &Segment<3>) -> Option<Vector<Real, 3>> {
        line_segment_intersection_3d(self, other)
    }
}
impl Intersect<Line<3>, 3> for Segment<3> {
    fn intersect(&self, other: &Line<3>) -> Option<Vector<Real, 3>> {
        line_segment_intersection_3d(other, self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line2(px: Real, py: Real, dx: Real, dy: Real) -> Line<2> {
        Line::new(Vector::from_slice(&[px, py]), Vector::from_slice(&[dx, dy]))
    }
    fn ray2(px: Real, py: Real, dx: Real, dy: Real) -> Ray<2> {
        Ray::new(Vector::from_slice(&[px, py]), Vector::from_slice(&[dx, dy]))
    }
    fn segment2(px: Real, py: Real, dx: Real, dy: Real) -> Segment<2> {
        Segment::new(Vector::from_slice(&[px, py]), Vector::from_slice(&[dx, dy]))
    }

    fn line3(px: Real, py: Real, pz: Real, dx: Real, dy: Real, dz: Real) -> Line<3> {
        Line::new(
            Vector::from_slice(&[px, py, pz]),
            Vector::from_slice(&[dx, dy, dz]),
        )
    }
    fn ray3(px: Real, py: Real, pz: Real, dx: Real, dy: Real, dz: Real) -> Ray<3> {
        Ray::new(
            Vector::from_slice(&[px, py, pz]),
            Vector::from_slice(&[dx, dy, dz]),
        )
    }
    fn segment3(px: Real, py: Real, pz: Real, dx: Real, dy: Real, dz: Real) -> Segment<3> {
        Segment::new(
            Vector::from_slice(&[px, py, pz]),
            Vector::from_slice(&[dx, dy, dz]),
        )
    }

    fn point2(x: Real, y: Real) -> Vector<Real, 2> {
        Vector::from_slice(&[x, y])
    }

    fn point3(x: Real, y: Real, z: Real) -> Vector<Real, 3> {
        Vector::from_slice(&[x, y, z])
    }

    // --- Kernel: 2D --------------------------------------------------------------
    // Heavy coverage lives here -- every combination function above is a thin
    // wrapper around this, so the math only needs proving once per dimension.
    // Kind doesn't matter to the kernel, so `segment2` stands in for all
    // fixtures -- unlike Line/Ray it doesn't normalise, which lets these
    // reuse the original hand-verified (t, s) values including non-unit
    // and zero-length directions.

    #[test]
    fn kernel_2d_general_position() {
        // l0: (0,0) + t(1,2); l1: (-7,1) + s(3,1). Meet at (2,4), t=2, s=3.
        let l0 = segment2(0.0, 0.0, 1.0, 2.0);
        let l1 = segment2(-7.0, 1.0, 3.0, 1.0);
        assert_eq!(intersect_params_2d(&l0, &l1), Some((2.0, 3.0)));
    }

    #[test]
    fn kernel_2d_perpendicular() {
        let l0 = segment2(0.0, 0.0, 1.0, 0.0);
        let l1 = segment2(5.0, -5.0, 0.0, 1.0);
        assert_eq!(intersect_params_2d(&l0, &l1), Some((5.0, 5.0)));
    }

    #[test]
    fn kernel_2d_shared_origin() {
        let l0 = segment2(2.0, 3.0, 1.0, 0.0);
        let l1 = segment2(2.0, 3.0, 0.0, 1.0);
        assert_eq!(intersect_params_2d(&l0, &l1), Some((0.0, 0.0)));
    }

    #[test]
    fn kernel_2d_negative_t() {
        let l0 = segment2(0.0, 0.0, 1.0, 0.0);
        let l1 = segment2(-5.0, -5.0, 0.0, 1.0);
        assert_eq!(intersect_params_2d(&l0, &l1), Some((-5.0, 5.0)));
    }

    #[test]
    fn kernel_2d_negative_s() {
        let l0 = segment2(-5.0, 0.0, 1.0, 0.0);
        let l1 = segment2(0.0, 5.0, 0.0, 1.0);
        assert_eq!(intersect_params_2d(&l0, &l1), Some((5.0, -5.0)));
    }

    #[test]
    fn kernel_2d_parallel_distinct_is_none() {
        let l0 = segment2(0.0, 0.0, 1.0, 0.0);
        let l1 = segment2(0.0, 1.0, 1.0, 0.0);
        assert_eq!(intersect_params_2d(&l0, &l1), None);
    }

    #[test]
    fn kernel_2d_parallel_colinear_is_none() {
        let l0 = segment2(0.0, 0.0, 1.0, 0.0);
        let l1 = segment2(2.0, 0.0, 1.0, 0.0);
        assert_eq!(intersect_params_2d(&l0, &l1), None);
    }

    #[test]
    fn kernel_2d_zero_length_direction_l0_is_none() {
        let l0 = segment2(0.0, 0.0, 0.0, 0.0);
        let l1 = segment2(0.0, 0.0, 1.0, 0.0);
        assert_eq!(intersect_params_2d(&l0, &l1), None);
    }

    #[test]
    fn kernel_2d_zero_length_direction_l1_is_none() {
        let l0 = segment2(0.0, 0.0, 1.0, 0.0);
        let l1 = segment2(0.0, 0.0, 0.0, 0.0);
        assert_eq!(intersect_params_2d(&l0, &l1), None);
    }

    // --- Kernel: 3D --------------------------------------------------------------

    #[test]
    fn kernel_3d_general_position() {
        // l0: (0,0,0) + t(1,0,1); l1: (2,-3,-1) + s(0,1,1). Meet at (2,0,2), t=2, s=3.
        let l0 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 1.0);
        let l1 = segment3(2.0, -3.0, -1.0, 0.0, 1.0, 1.0);
        assert_eq!(intersect_params_3d(&l0, &l1), Some((2.0, 3.0)));
    }

    #[test]
    fn kernel_3d_perpendicular_coplanar() {
        // Same as kernel_2d_perpendicular, embedded in the z=0 plane.
        let l0 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let l1 = segment3(5.0, -5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), Some((5.0, 5.0)));
    }

    #[test]
    fn kernel_3d_shared_origin() {
        let l0 = segment3(1.0, 1.0, 1.0, 1.0, 0.0, 0.0);
        let l1 = segment3(1.0, 1.0, 1.0, 0.0, 1.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), Some((0.0, 0.0)));
    }

    #[test]
    fn kernel_3d_negative_t() {
        let l0 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let l1 = segment3(-5.0, -5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), Some((-5.0, 5.0)));
    }

    #[test]
    fn kernel_3d_negative_s() {
        let l0 = segment3(-5.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let l1 = segment3(0.0, 5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), Some((5.0, -5.0)));
    }

    #[test]
    fn kernel_3d_parallel_distinct_is_none() {
        let l0 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let l1 = segment3(0.0, 1.0, 0.0, 1.0, 0.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), None);
    }

    #[test]
    fn kernel_3d_parallel_colinear_is_none() {
        let l0 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let l1 = segment3(2.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), None);
    }

    #[test]
    fn kernel_3d_zero_length_direction_l0_is_none() {
        let l0 = segment3(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let l1 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), None);
    }

    #[test]
    fn kernel_3d_zero_length_direction_l1_is_none() {
        let l0 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let l1 = segment3(0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), None);
    }

    #[test]
    fn kernel_3d_skew_lines_are_none() {
        // Classic skew pair: x-axis at z=0 vs y-axis at z=1.
        let l0 = segment3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let l1 = segment3(0.0, 0.0, 1.0, 0.0, 1.0, 0.0);
        assert_eq!(intersect_params_3d(&l0, &l1), None);
    }

    // --- Parameter predicates ------------------------------------------------

    #[test]
    fn in_ray_param_accepts_zero_and_positive() {
        assert!(in_ray_param(0.0));
        assert!(in_ray_param(5.0));
    }

    #[test]
    fn in_ray_param_rejects_negative() {
        assert!(!in_ray_param(-0.5));
        assert!(!in_ray_param(-5.0));
    }

    #[test]
    fn in_segment_param_accepts_endpoints_and_interior() {
        assert!(in_segment_param(0.0));
        assert!(in_segment_param(1.0));
        assert!(in_segment_param(0.5));
    }

    #[test]
    fn in_segment_param_rejects_outside_unit_interval() {
        assert!(!in_segment_param(-0.5));
        assert!(!in_segment_param(1.5));
    }

    // --- Constructors: from_points must behave exactly like new(a, b - a) ------

    #[test]
    fn ray_from_points_behaves_like_new() {
        let via_points = Ray::from_points(point2(0.0, 0.0), point2(1.0, 0.0));
        let via_new = Ray::new(point2(0.0, 0.0), point2(1.0, 0.0));
        let line = line2(5.0, -5.0, 0.0, 1.0);
        assert_eq!(via_points.intersect(&line), via_new.intersect(&line));
        assert_eq!(via_points.intersect(&line), Some(point2(5.0, 0.0)));
    }

    #[test]
    fn line_from_points_behaves_like_new() {
        let via_points = Line::from_points(point2(5.0, -5.0), point2(5.0, 5.0));
        let via_new = Line::new(point2(5.0, -5.0), point2(0.0, 10.0));
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        assert_eq!(ray.intersect(&via_points), ray.intersect(&via_new));
        assert_eq!(ray.intersect(&via_points), Some(point2(5.0, 0.0)));
    }

    #[test]
    fn segment_from_points_behaves_like_new() {
        let via_points = Segment::from_points(point2(5.0, -5.0), point2(5.0, 5.0));
        let via_new = Segment::new(point2(5.0, -5.0), point2(0.0, 10.0));
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        assert_eq!(ray.intersect(&via_points), ray.intersect(&via_new));
        assert_eq!(ray.intersect(&via_points), Some(point2(5.0, 0.0)));
    }

    // --- Constructors: Ray/Line normalise direction; Segment stores raw endpoints ---

    #[test]
    fn ray_new_normalises_direction() {
        let ray = Ray::new(point2(0.0, 0.0), point2(3.0, 4.0)); // magnitude 5
        let d = ray.direction();
        assert!(approx_eq(d.x(), 0.6));
        assert!(approx_eq(d.y(), 0.8));
    }

    #[test]
    fn line_new_normalises_direction() {
        let line = Line::new(point2(0.0, 0.0), point2(3.0, 4.0)); // magnitude 5
        let d = line.direction();
        assert!(approx_eq(d.x(), 0.6));
        assert!(approx_eq(d.y(), 0.8));
    }

    #[test]
    #[should_panic(expected = "direction must be non-zero")]
    fn ray_new_panics_on_zero_direction() {
        Ray::new(point2(0.0, 0.0), point2(0.0, 0.0));
    }

    #[test]
    #[should_panic(expected = "direction must be non-zero")]
    fn line_new_panics_on_zero_direction() {
        Line::new(point2(0.0, 0.0), point2(0.0, 0.0));
    }

    #[test]
    fn segment_stores_endpoints_without_normalising() {
        let segment = Segment::new(point2(0.0, 0.0), point2(3.0, 4.0)); // magnitude 5
        assert_eq!(segment.direction(), point2(3.0, 4.0));
    }

    // --- Ray -- Ray: only the additional edge case (both sides gated by
    // `in_ray_param`) needs proving; the underlying math is the kernel's job.

    #[test]
    fn ray_ray_2d_valid_when_both_params_nonnegative() {
        let r0 = ray2(0.0, 0.0, 1.0, 0.0);
        let r1 = ray2(5.0, -5.0, 0.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), Some(point2(5.0, 0.0)));
    }

    #[test]
    fn ray_ray_2d_none_when_first_param_negative() {
        let r0 = ray2(0.0, 0.0, 1.0, 0.0);
        let r1 = ray2(-5.0, -5.0, 0.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    #[test]
    fn ray_ray_2d_none_when_second_param_negative() {
        let r0 = ray2(-5.0, 0.0, 1.0, 0.0);
        let r1 = ray2(0.0, 5.0, 0.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    #[test]
    fn ray_ray_3d_sanity() {
        let r0 = ray3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let r1 = ray3(5.0, -5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(
            ray_ray_intersection_3d(&r0, &r1),
            Some(point3(5.0, 0.0, 0.0))
        );
    }

    // --- Ray -- Line: the new behaviour is that the line side is unconstrained.

    #[test]
    fn ray_line_2d_valid() {
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        let line = line2(5.0, -5.0, 0.0, 1.0);
        assert_eq!(ray_line_intersection_2d(&ray, &line), Some(point2(5.0, 0.0)));
    }

    #[test]
    fn ray_line_2d_none_when_ray_param_negative() {
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        let line = line2(-5.0, -5.0, 0.0, 1.0);
        assert_eq!(ray_line_intersection_2d(&ray, &line), None);
    }

    #[test]
    fn ray_line_2d_valid_even_when_line_param_negative() {
        // Same geometry as ray_ray_2d_none_when_second_param_negative, but the
        // second object is now an unconstrained line, not a ray.
        let ray = ray2(-5.0, 0.0, 1.0, 0.0);
        let line = line2(0.0, 5.0, 0.0, 1.0);
        assert_eq!(ray_line_intersection_2d(&ray, &line), Some(point2(0.0, 0.0)));
    }

    #[test]
    fn ray_line_3d_sanity() {
        let ray = ray3(-5.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let line = line3(0.0, 5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(
            ray_line_intersection_3d(&ray, &line),
            Some(point3(0.0, 0.0, 0.0))
        );
    }

    // --- Ray -- Segment: the new behaviour is the segment's [0,1] upper bound.

    #[test]
    fn ray_segment_2d_valid_in_range() {
        // segment spans (5,-5) to (5,5); ray hits it at s=0.5.
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 10.0);
        assert_eq!(
            ray_segment_intersection_2d(&ray, &segment),
            Some(point2(5.0, 0.0))
        );
    }

    #[test]
    fn ray_segment_2d_none_when_ray_param_negative() {
        let ray = ray2(0.0, 0.0, -1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 10.0);
        assert_eq!(ray_segment_intersection_2d(&ray, &segment), None);
    }

    #[test]
    fn ray_segment_2d_none_when_beyond_segment_end() {
        // segment spans (5,-5) to (5,-1); ray would hit it at s=1.25.
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 4.0);
        assert_eq!(ray_segment_intersection_2d(&ray, &segment), None);
    }

    #[test]
    fn ray_segment_3d_sanity() {
        let ray = ray3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let segment = segment3(5.0, -5.0, 0.0, 0.0, 10.0, 0.0);
        assert_eq!(
            ray_segment_intersection_3d(&ray, &segment),
            Some(point3(5.0, 0.0, 0.0))
        );
    }

    // --- Line -- Segment: the new behaviour is line unconstrained + segment [0,1].

    #[test]
    fn line_segment_2d_valid() {
        let line = line2(0.0, 0.0, 1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 10.0);
        assert_eq!(
            line_segment_intersection_2d(&line, &segment),
            Some(point2(5.0, 0.0))
        );
    }

    #[test]
    fn line_segment_2d_valid_even_when_line_param_negative() {
        let line = line2(0.0, 0.0, -1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 10.0);
        assert_eq!(
            line_segment_intersection_2d(&line, &segment),
            Some(point2(5.0, 0.0))
        );
    }

    #[test]
    fn line_segment_2d_none_when_beyond_segment_end() {
        let line = line2(0.0, 0.0, 1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 4.0);
        assert_eq!(line_segment_intersection_2d(&line, &segment), None);
    }

    #[test]
    fn line_segment_3d_sanity() {
        let line = line3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let segment = segment3(5.0, -5.0, 0.0, 0.0, 10.0, 0.0);
        assert_eq!(
            line_segment_intersection_3d(&line, &segment),
            Some(point3(5.0, 0.0, 0.0))
        );
    }

    // --- Unified `Intersect` trait: confirms the dispatch-by-type wiring, both
    // directions where a reversed impl exists.

    #[test]
    fn intersect_trait_ray_ray() {
        let r0 = ray2(0.0, 0.0, 1.0, 0.0);
        let r1 = ray2(5.0, -5.0, 0.0, 1.0);
        assert_eq!(r0.intersect(&r1), Some(point2(5.0, 0.0)));

        let r0 = ray3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let r1 = ray3(5.0, -5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(r0.intersect(&r1), Some(point3(5.0, 0.0, 0.0)));
    }

    #[test]
    fn intersect_trait_ray_line_both_orders() {
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        let line = line2(5.0, -5.0, 0.0, 1.0);
        assert_eq!(ray.intersect(&line), Some(point2(5.0, 0.0)));
        assert_eq!(line.intersect(&ray), Some(point2(5.0, 0.0)));

        let ray = ray3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let line = line3(5.0, -5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(ray.intersect(&line), Some(point3(5.0, 0.0, 0.0)));
        assert_eq!(line.intersect(&ray), Some(point3(5.0, 0.0, 0.0)));
    }

    #[test]
    fn intersect_trait_ray_segment_both_orders() {
        let ray = ray2(0.0, 0.0, 1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 10.0);
        assert_eq!(ray.intersect(&segment), Some(point2(5.0, 0.0)));
        assert_eq!(segment.intersect(&ray), Some(point2(5.0, 0.0)));

        let ray = ray3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let segment = segment3(5.0, -5.0, 0.0, 0.0, 10.0, 0.0);
        assert_eq!(ray.intersect(&segment), Some(point3(5.0, 0.0, 0.0)));
        assert_eq!(segment.intersect(&ray), Some(point3(5.0, 0.0, 0.0)));
    }

    #[test]
    fn intersect_trait_line_segment_both_orders() {
        let line = line2(0.0, 0.0, 1.0, 0.0);
        let segment = segment2(5.0, -5.0, 0.0, 10.0);
        assert_eq!(line.intersect(&segment), Some(point2(5.0, 0.0)));
        assert_eq!(segment.intersect(&line), Some(point2(5.0, 0.0)));

        let line = line3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let segment = segment3(5.0, -5.0, 0.0, 0.0, 10.0, 0.0);
        assert_eq!(line.intersect(&segment), Some(point3(5.0, 0.0, 0.0)));
        assert_eq!(segment.intersect(&line), Some(point3(5.0, 0.0, 0.0)));
    }
}
