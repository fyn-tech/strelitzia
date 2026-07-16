use crate::common::{Real, approx_eq, approx_lt, bounds_failure_str};
use crate::multiarray::Vector;
use crate::multiarray::linalg::{cross, cross2d, dot, l2_norm, normalised};

pub struct Line<const D: usize> {
    point: Vector<Real, D>,
    direction: Vector<Real, D>,
}

pub fn ray_ray_intersection_2d(r0: &Line<2>, r1: &Line<2>) -> Option<Vector<Real, 2>> {
    let denom = cross2d(&r0.direction, &r1.direction);
    if approx_eq(denom, 0.0) {
        return None; // parallel
    }
    let vec_01 = r1.point - r0.point;
    let t = cross2d(&vec_01, &r1.direction) / denom;
    let s = cross2d(&vec_01, &r0.direction) / denom;
    if approx_lt(t, 0.0) || approx_lt(s, 0.0) {
        return None; // line intersection, behind r0 or r1
    }
    Some(r0.point + t * r0.direction)
}

pub fn ray_ray_intersection_3d(r0: &Line<3>, r1: &Line<3>) -> Option<Vector<Real, 3>> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line2(px: Real, py: Real, dx: Real, dy: Real) -> Line<2> {
        Line {
            point: Vector::from_slice(&[px, py]),
            direction: Vector::from_slice(&[dx, dy]),
        }
    }

    fn line3(px: Real, py: Real, pz: Real, dx: Real, dy: Real, dz: Real) -> Line<3> {
        Line {
            point: Vector::from_slice(&[px, py, pz]),
            direction: Vector::from_slice(&[dx, dy, dz]),
        }
    }

    fn point2(x: Real, y: Real) -> Vector<Real, 2> {
        Vector::from_slice(&[x, y])
    }

    // --- 2D: rays that should intersect ---------------------------------------

    #[test]
    fn perpendicular_rays_intersect_in_front_of_both() {
        // r0: (0,0) + t(1,0); r1: (5,-5) + s(0,1). Meet at (5,0), t=5, s=5.
        let r0 = line2(0.0, 0.0, 1.0, 0.0);
        let r1 = line2(5.0, -5.0, 0.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), Some(point2(5.0, 0.0)));
    }

    #[test]
    fn oblique_rays_intersect_in_front_of_both() {
        // r0: (0,0) + t(1,2); r1: (-7,1) + s(3,1). Meet at (2,4), t=2, s=3.
        let r0 = line2(0.0, 0.0, 1.0, 2.0);
        let r1 = line2(-7.0, 1.0, 3.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), Some(point2(2.0, 4.0)));
    }

    #[test]
    fn rays_sharing_an_origin_intersect_at_that_point() {
        // Both rays start at (2,3): t=0, s=0.
        let r0 = line2(2.0, 3.0, 1.0, 0.0);
        let r1 = line2(2.0, 3.0, 0.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), Some(point2(2.0, 3.0)));
    }

    // --- 2D: line intersection exists, but not as rays --------------------------

    #[test]
    fn intersection_behind_ray0_origin_returns_none() {
        // Lines meet at (-5,0), which is t=-5 on r0 (behind its start).
        let r0 = line2(0.0, 0.0, 1.0, 0.0);
        let r1 = line2(-5.0, -5.0, 0.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    #[test]
    fn intersection_behind_ray1_origin_returns_none() {
        // Lines meet at (0,0), which is s=-5 on r1 (behind its start).
        let r0 = line2(-5.0, 0.0, 1.0, 0.0);
        let r1 = line2(0.0, 5.0, 0.0, 1.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    // --- 2D: parallel / degenerate cases -----------------------------------------

    #[test]
    fn parallel_distinct_rays_never_intersect() {
        let r0 = line2(0.0, 0.0, 1.0, 0.0);
        let r1 = line2(0.0, 1.0, 1.0, 0.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    #[test]
    fn parallel_colinear_overlapping_rays_have_no_unique_intersection() {
        let r0 = line2(0.0, 0.0, 1.0, 0.0);
        let r1 = line2(2.0, 0.0, 1.0, 0.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    #[test]
    fn zero_length_direction_on_ray0_returns_none() {
        let r0 = line2(0.0, 0.0, 0.0, 0.0);
        let r1 = line2(0.0, 0.0, 1.0, 0.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    #[test]
    fn zero_length_direction_on_ray1_returns_none() {
        let r0 = line2(0.0, 0.0, 1.0, 0.0);
        let r1 = line2(0.0, 0.0, 0.0, 0.0);
        assert_eq!(ray_ray_intersection_2d(&r0, &r1), None);
    }

    // --- 3D dispatch --------------------------------------------------------------

    #[test]
    fn dispatch_3d_currently_unimplemented() {
        // ray_ray_intersection_3d is a stub that always returns None; this
        // documents today's behaviour, not a claim that 3D is handled
        // correctly. Update once ray_ray_intersection_3d is implemented --
        // these rays (analogous to `perpendicular_rays_intersect_in_front_of_both`
        // embedded in the z=0 plane) should meet at (5,0,0).
        let r0 = line3(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let r1 = line3(5.0, -5.0, 0.0, 0.0, 1.0, 0.0);
        assert_eq!(ray_ray_intersection_3d(&r0, &r1), None);
    }
}
