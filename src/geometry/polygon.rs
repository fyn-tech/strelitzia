use crate::common::{Real, approx_gt};
use crate::multiarray::Vector;

/// A 2D polygon: an outer vertex loop plus zero or more hole loops, all
/// implicitly closed (the last vertex connects back to the first).
pub struct Polygon {
    outer: Vec<Vector<Real, 2>>,
    holes: Vec<Vec<Vector<Real, 2>>>,
}

impl Polygon {
    /// A polygon with no holes.
    pub fn new(outer: Vec<Vector<Real, 2>>) -> Self {
        Self::with_holes(outer, Vec::new())
    }

    /// A polygon with hole loops subtracted from the outer loop.
    pub fn with_holes(outer: Vec<Vector<Real, 2>>, holes: Vec<Vec<Vector<Real, 2>>>) -> Self {
        assert!(
            outer.len() >= 3,
            "Polygon::with_holes: outer loop needs at least 3 vertices"
        );
        for hole in &holes {
            assert!(
                hole.len() >= 3,
                "Polygon::with_holes: each hole loop needs at least 3 vertices"
            );
        }
        Self { outer, holes }
    }

    /// True if `point` is inside the polygon (in any hole counts as outside).
    /// Boundary points (exactly on an edge or vertex) are not guaranteed to
    /// classify consistently -- this is a property of the crossing-number
    /// algorithm, not a bug.
    pub fn contains(&self, point: &Vector<Real, 2>) -> bool {
        let crossings: usize = std::iter::once(&self.outer)
            .chain(self.holes.iter())
            .map(|loop_| loop_crossings(loop_, point))
            .sum();
        crossings % 2 == 1
    }
}

/// Counts how many edges of a (implicitly closed) vertex loop cross a
/// rightward horizontal ray cast from `point`.
fn loop_crossings(vertices: &[Vector<Real, 2>], point: &Vector<Real, 2>) -> usize {
    let n = vertices.len();
    (0..n)
        .filter(|&i| edge_crosses_ray(vertices[i], vertices[(i + 1) % n], *point))
        .count()
}

/// Standard crossing-number test (Franklin's PNPOLY): counts a crossing only
/// if exactly one endpoint is strictly above `point`'s height, which avoids
/// double-counting when the ray passes exactly through a vertex shared by
/// two edges.
fn edge_crosses_ray(a: Vector<Real, 2>, b: Vector<Real, 2>, point: Vector<Real, 2>) -> bool {
    if approx_gt(a.y(), point.y()) == approx_gt(b.y(), point.y()) {
        return false;
    }
    let x_intersect = a.x() + (point.y() - a.y()) / (b.y() - a.y()) * (b.x() - a.x());
    approx_gt(x_intersect, point.x())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(x: Real, y: Real) -> Vector<Real, 2> {
        Vector::from_slice(&[x, y])
    }

    fn square(min: Real, max: Real) -> Vec<Vector<Real, 2>> {
        vec![
            v(min, min),
            v(max, min),
            v(max, max),
            v(min, max),
        ]
    }

    #[test]
    fn point_inside_convex_square() {
        let poly = Polygon::new(square(0.0, 4.0));
        assert!(poly.contains(&v(2.0, 2.0)));
    }

    #[test]
    fn point_outside_convex_square() {
        let poly = Polygon::new(square(0.0, 4.0));
        assert!(!poly.contains(&v(5.0, 5.0)));
    }

    #[test]
    fn point_in_concave_notch_is_outside() {
        // L-shape: horizontal bar x:[0,4] y:[0,2] plus vertical bar x:[0,2] y:[0,4].
        let l_shape = vec![
            v(0.0, 0.0),
            v(4.0, 0.0),
            v(4.0, 2.0),
            v(2.0, 2.0),
            v(2.0, 4.0),
            v(0.0, 4.0),
        ];
        let poly = Polygon::new(l_shape);
        assert!(!poly.contains(&v(3.0, 3.0))); // in the cut-out notch
        assert!(poly.contains(&v(1.0, 1.0))); // in the horizontal bar
    }

    #[test]
    fn ray_through_shared_vertex_is_not_double_counted() {
        // Triangle with a vertex exactly at the query point's height --
        // the classic PNPOLY robustness case.
        let triangle = vec![v(0.0, 0.0), v(4.0, 2.0), v(0.0, 4.0)];
        let poly = Polygon::new(triangle);
        assert!(poly.contains(&v(1.0, 2.0)));
    }

    #[test]
    fn point_inside_hole_is_outside() {
        let outer = square(0.0, 10.0);
        let hole = square(3.0, 7.0);
        let poly = Polygon::with_holes(outer, vec![hole]);
        assert!(!poly.contains(&v(5.0, 5.0)));
    }

    #[test]
    fn point_between_outer_and_hole_is_inside() {
        let outer = square(0.0, 10.0);
        let hole = square(3.0, 7.0);
        let poly = Polygon::with_holes(outer, vec![hole]);
        assert!(poly.contains(&v(1.0, 1.0)));
    }

    #[test]
    #[should_panic(expected = "at least 3 vertices")]
    fn new_panics_on_too_few_vertices() {
        Polygon::new(vec![v(0.0, 0.0), v(1.0, 0.0)]);
    }

    #[test]
    #[should_panic(expected = "each hole loop needs at least 3 vertices")]
    fn with_holes_panics_on_too_few_hole_vertices() {
        Polygon::with_holes(square(0.0, 10.0), vec![vec![v(1.0, 1.0), v(2.0, 2.0)]]);
    }
}
