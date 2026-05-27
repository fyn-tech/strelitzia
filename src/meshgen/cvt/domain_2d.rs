//! 2D domain implementation for CVT.
//!
//! Contains [`Domain2D`] (PSLG-based 2D domain), its [`CvtDomain`] implementation,
//! Sutherland-Hodgman polygon clipping, Dunavant triangle quadrature, and 2D helpers.

use crate::common::Real;
use crate::fields::Field;
use crate::multiarray::Point2;

use super::{CvtCellData, CvtDomain, CvtPoint};

use spade::handles::VoronoiVertex;
use spade::{DelaunayTriangulation, HasPosition, Triangulation};

use std::cell::RefCell;

// ============================================================================
// SpadePoint -- adapter for spade's HasPosition trait
// ============================================================================

#[derive(Debug, Clone, Copy)]
struct SpadePoint {
    x: f64,
    y: f64,
}

impl SpadePoint {
    fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    fn from_point2(p: Point2) -> Self {
        Self::new(p[0], p[1])
    }
}

impl HasPosition for SpadePoint {
    type Scalar = f64;
    fn position(&self) -> spade::Point2<f64> {
        spade::Point2::new(self.x, self.y)
    }
}

// ============================================================================
// Domain2D
// ============================================================================

/// A 2D domain defined by a Planar Straight-Line Graph (PSLG).
///
/// Outer boundary edges are oriented CCW, hole boundary edges CW.
/// Internal boundaries (material interfaces) are also supported as
/// additional edge chains.
///
/// The domain maintains a persistent Delaunay triangulation via interior
/// mutability for incremental updates across CVT iterations.
pub struct Domain2D {
    vertices: Vec<Point2>,
    edges: Vec<[usize; 2]>,
    quad_order: usize,
    triangulation: RefCell<Option<DelaunayTriangulation<SpadePoint>>>,
}

impl Clone for Domain2D {
    fn clone(&self) -> Self {
        Self {
            vertices: self.vertices.clone(),
            edges: self.edges.clone(),
            quad_order: self.quad_order,
            triangulation: RefCell::new(None),
        }
    }
}

impl Domain2D {
    /// Create a Domain2D from PSLG vertices and edges.
    ///
    /// Edges are index pairs into the vertex array. Outer boundary should be
    /// CCW, holes CW. Default quadrature order is 5.
    pub fn new(vertices: Vec<Point2>, edges: Vec<[usize; 2]>) -> Self {
        Self {
            vertices,
            edges,
            quad_order: 5,
            triangulation: RefCell::new(None),
        }
    }

    /// Set the Dunavant quadrature order (1, 2, or 5).
    pub fn with_quadrature(mut self, order: usize) -> Self {
        self.quad_order = order;
        self
    }

    /// Create a rectangular domain.
    pub fn rectangle(x_min: Real, x_max: Real, y_min: Real, y_max: Real) -> Self {
        let vertices = vec![
            Point2::new(x_min, y_min),
            Point2::new(x_max, y_min),
            Point2::new(x_max, y_max),
            Point2::new(x_min, y_max),
        ];
        let edges = vec![[0, 1], [1, 2], [2, 3], [3, 0]];
        Self::new(vertices, edges)
    }

    /// Read access to boundary vertices.
    pub fn boundary_vertices(&self) -> &[Point2] {
        &self.vertices
    }

    /// Axis-aligned bounding box: (min_corner, max_corner).
    pub fn bounding_box(&self) -> (Point2, Point2) {
        let mut min_x = Real::INFINITY;
        let mut min_y = Real::INFINITY;
        let mut max_x = Real::NEG_INFINITY;
        let mut max_y = Real::NEG_INFINITY;
        for v in &self.vertices {
            min_x = min_x.min(v[0]);
            min_y = min_y.min(v[1]);
            max_x = max_x.max(v[0]);
            max_y = max_y.max(v[1]);
        }
        (Point2::new(min_x, min_y), Point2::new(max_x, max_y))
    }

    /// Test whether a point lies inside the domain using the winding number algorithm
    /// with `robust::orient2d` for exact orientation tests.
    pub fn contains(&self, point: &Point2) -> bool {
        let px = point[0];
        let py = point[1];
        let mut winding = 0i32;

        for &[i, j] in &self.edges {
            let (ax, ay) = (self.vertices[i][0], self.vertices[i][1]);
            let (bx, by) = (self.vertices[j][0], self.vertices[j][1]);

            if ay <= py {
                if by > py {
                    let orient = robust::orient2d(
                        robust::Coord { x: ax, y: ay },
                        robust::Coord { x: bx, y: by },
                        robust::Coord { x: px, y: py },
                    );
                    if orient > 0.0 {
                        winding += 1;
                    }
                }
            } else if by <= py {
                let orient = robust::orient2d(
                    robust::Coord { x: ax, y: ay },
                    robust::Coord { x: bx, y: by },
                    robust::Coord { x: px, y: py },
                );
                if orient < 0.0 {
                    winding -= 1;
                }
            }
        }

        winding != 0
    }

    /// Closest point on the domain boundary to a given point.
    pub fn nearest_boundary_point(&self, point: &Point2) -> Point2 {
        let mut best_dist_sq = Real::INFINITY;
        let mut best_point = *point;

        for &[i, j] in &self.edges {
            let a = self.vertices[i];
            let b = self.vertices[j];
            let ab = b - a;
            let ap = *point - a;
            let ab_sq = ab[0] * ab[0] + ab[1] * ab[1];
            let t = if ab_sq > 0.0 {
                (ap[0] * ab[0] + ap[1] * ab[1]) / ab_sq
            } else {
                0.0
            };
            let t = t.clamp(0.0, 1.0);
            let proj = a + ab * t;
            let diff = *point - proj;
            let dist_sq = diff[0] * diff[0] + diff[1] * diff[1];
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_point = proj;
            }
        }

        best_point
    }

    /// Generate n uniformly distributed seed points inside the domain.
    ///
    /// Uses a Halton sequence within the bounding box, rejection-sampled
    /// by `contains`.
    pub fn uniform_seeds(&self, n: usize) -> Field<Point2> {
        if n == 0 {
            return Field::new();
        }

        let (bb_min, bb_max) = self.bounding_box();
        let dx = bb_max[0] - bb_min[0];
        let dy = bb_max[1] - bb_min[1];

        let mut seeds = Field::with_capacity(n);
        let mut idx = 0usize;
        while seeds.len() < n {
            let hx = halton(idx, 2);
            let hy = halton(idx, 3);
            let p = Point2::new(bb_min[0] + hx * dx, bb_min[1] + hy * dy);
            if self.contains(&p) {
                seeds.push(p);
            }
            idx += 1;
            if idx > n * 1000 {
                panic!("uniform_seeds: failed to generate enough points inside domain");
            }
        }
        seeds
    }

    /// Compute the Voronoi cell polygon for each seed, clipped to the domain boundary.
    ///
    /// Returns a vector of polygons (each polygon is a Vec<Point2>).
    /// This is for visualization purposes; the CvtDomain integration pipeline
    /// does not materialize all cells simultaneously.
    pub fn voronoi_cells(&self, seeds: &[Point2]) -> Vec<Vec<Point2>> {
        if seeds.is_empty() {
            return vec![];
        }
        let ghosts = self.ghost_points();
        let tri = build_triangulation_with_ghosts(seeds, &ghosts);

        let mut result = Vec::with_capacity(seeds.len());
        for face in tri.voronoi_faces() {
            let sp = face.as_delaunay_vertex().position();
            if is_ghost(&sp, &ghosts) {
                continue;
            }
            let poly = extract_voronoi_polygon_inner(&face);
            let clipped = clip_polygon_to_domain(&poly, &self.vertices, &self.edges);
            result.push(clipped);
        }
        result
    }

    fn ghost_points(&self) -> [Point2; 4] {
        let (bb_min, bb_max) = self.bounding_box();
        let dx = bb_max[0] - bb_min[0];
        let dy = bb_max[1] - bb_min[1];
        let pad = (dx + dy) * 5.0;
        [
            Point2::new(bb_min[0] - pad, bb_min[1] - pad),
            Point2::new(bb_max[0] + pad, bb_min[1] - pad),
            Point2::new(bb_max[0] + pad, bb_max[1] + pad),
            Point2::new(bb_min[0] - pad, bb_max[1] + pad),
        ]
    }
}

// ============================================================================
// CvtDomain for Domain2D
// ============================================================================

impl CvtDomain for Domain2D {
    type Point = Point2;

    fn integrate_cells<F: Fn(Point2) -> Real>(
        &self,
        seeds: &[Point2],
        density: &F,
    ) -> CvtCellData<Point2> {
        let n = seeds.len();
        if n == 0 {
            return CvtCellData {
                centroids: Field::new(),
                masses: Field::new(),
                energy: 0.0,
            };
        }

        let ghosts = self.ghost_points();
        let quad = dunavant_rule(self.quad_order);

        // Build or update triangulation (always includes ghost points)
        let mut tri_ref = self.triangulation.borrow_mut();
        if tri_ref.is_none() {
            *tri_ref = Some(build_triangulation_with_ghosts(seeds, &ghosts));
        } else {
            let tri = tri_ref.as_mut().unwrap();
            while tri.num_vertices() > 0 {
                let handle = tri.fixed_vertices().next().unwrap();
                tri.remove(handle);
            }
            for ghost in &ghosts {
                let _ = tri.insert(SpadePoint::from_point2(*ghost));
            }
            for &seed in seeds {
                let _ = tri.insert(SpadePoint::from_point2(seed));
            }
        }

        let tri = tri_ref.as_ref().unwrap();

        let mut centroids = Field::with_capacity(n);
        let mut masses = Field::with_capacity(n);
        let mut total_energy = 0.0;

        for face in tri.voronoi_faces() {
            let seed_pos = face.as_delaunay_vertex().position();
            if is_ghost(&seed_pos, &ghosts) {
                continue;
            }
            let seed = Point2::new(seed_pos.x, seed_pos.y);

            let poly = extract_voronoi_polygon_inner(&face);
            let clipped = clip_polygon_to_domain(&poly, &self.vertices, &self.edges);

            if clipped.len() < 3 {
                centroids.push(seed);
                masses.push(0.0);
                continue;
            }

            let (mass, moment, energy) = integrate_polygon(&clipped, &seed, density, &quad);

            if mass > 0.0 {
                centroids.push(moment / mass);
            } else {
                centroids.push(seed);
            }
            masses.push(mass);
            total_energy += energy;
        }

        CvtCellData {
            centroids,
            masses,
            energy: total_energy,
        }
    }
}

// ============================================================================
// Triangulation helpers
// ============================================================================

fn build_triangulation_with_ghosts(
    seeds: &[Point2],
    ghosts: &[Point2; 4],
) -> DelaunayTriangulation<SpadePoint> {
    let mut points: Vec<SpadePoint> = ghosts.iter().map(|p| SpadePoint::from_point2(*p)).collect();
    points.extend(seeds.iter().map(|p| SpadePoint::from_point2(*p)));
    DelaunayTriangulation::bulk_load(points).expect("failed to build Delaunay triangulation")
}

fn is_ghost(pos: &spade::Point2<f64>, ghosts: &[Point2; 4]) -> bool {
    ghosts
        .iter()
        .any(|g| (g[0] - pos.x).abs() < 1e-10 && (g[1] - pos.y).abs() < 1e-10)
}

/// Extract Voronoi cell polygon from a VoronoiFace whose seed is interior
/// to the convex hull (guaranteed by ghost points). All vertices are inner
/// (circumcenters), so the polygon is always bounded and well-formed.
fn extract_voronoi_polygon_inner<'a, V, DE, UE, F>(
    face: &spade::handles::VoronoiFace<'a, V, DE, UE, F>,
) -> Vec<Point2>
where
    V: HasPosition<Scalar = f64>,
    DE: Default,
    UE: Default,
    F: Default,
{
    let mut poly = Vec::new();

    for edge in face.adjacent_edges() {
        match edge.from() {
            VoronoiVertex::Inner(f) => {
                let cc = f.circumcenter();
                poly.push(Point2::new(cc.x, cc.y));
            }
            VoronoiVertex::Outer(_) => {
                // Should not happen when ghost points ensure all seeds are interior.
                // If it does, skip -- clipping will handle the gap.
            }
        }
    }

    poly
}

// ============================================================================
// Sutherland-Hodgman polygon clipping
// ============================================================================

/// Clip a subject polygon against the domain boundary using Sutherland-Hodgman.
///
/// Uses `robust::orient2d` for exact side tests.
fn clip_polygon_to_domain(
    subject: &[Point2],
    boundary_verts: &[Point2],
    boundary_edges: &[[usize; 2]],
) -> Vec<Point2> {
    let mut output = subject.to_vec();

    for &[ei, ej] in boundary_edges {
        if output.is_empty() {
            break;
        }
        let edge_a = boundary_verts[ei];
        let edge_b = boundary_verts[ej];
        output = clip_against_edge(&output, &edge_a, &edge_b);
    }

    output
}

/// Clip a polygon against a single directed edge (a -> b).
/// Points on the left side (CCW) are kept.
#[allow(clippy::collapsible_if)]
fn clip_against_edge(polygon: &[Point2], edge_a: &Point2, edge_b: &Point2) -> Vec<Point2> {
    let n = polygon.len();
    if n == 0 {
        return vec![];
    }

    let ca = robust::Coord {
        x: edge_a[0],
        y: edge_a[1],
    };
    let cb = robust::Coord {
        x: edge_b[0],
        y: edge_b[1],
    };

    let inside = |p: &Point2| -> bool {
        let cp = robust::Coord { x: p[0], y: p[1] };
        robust::orient2d(ca, cb, cp) >= 0.0
    };

    let mut result = Vec::new();

    for i in 0..n {
        let current = &polygon[i];
        let previous = &polygon[(i + n - 1) % n];
        let curr_in = inside(current);
        let prev_in = inside(previous);

        if curr_in {
            if !prev_in {
                if let Some(pt) = line_segment_intersect(previous, current, edge_a, edge_b) {
                    result.push(pt);
                }
            }
            result.push(*current);
        } else if prev_in {
            if let Some(pt) = line_segment_intersect(previous, current, edge_a, edge_b) {
                result.push(pt);
            }
        }
    }

    result
}

/// Compute the intersection of line segments (p1,p2) and (p3,p4).
fn line_segment_intersect(p1: &Point2, p2: &Point2, p3: &Point2, p4: &Point2) -> Option<Point2> {
    let d1x = p2[0] - p1[0];
    let d1y = p2[1] - p1[1];
    let d2x = p4[0] - p3[0];
    let d2y = p4[1] - p3[1];

    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-15 {
        return None;
    }

    let t = ((p3[0] - p1[0]) * d2y - (p3[1] - p1[1]) * d2x) / denom;
    Some(Point2::new(p1[0] + t * d1x, p1[1] + t * d1y))
}

// ============================================================================
// Dunavant triangle quadrature
// ============================================================================

/// A single quadrature point: barycentric coords (L1, L2, L3) and weight.
struct QuadPoint {
    l1: Real,
    l2: Real,
    l3: Real,
    w: Real,
}

/// Pre-computed Dunavant quadrature rules for triangles.
fn dunavant_rule(order: usize) -> Vec<QuadPoint> {
    match order {
        // Degree 1: centroid rule, 1 point, exact for linear
        1 => vec![QuadPoint {
            l1: 1.0 / 3.0,
            l2: 1.0 / 3.0,
            l3: 1.0 / 3.0,
            w: 1.0,
        }],
        // Degree 2: 3 points, exact for quadratic
        2 => vec![
            QuadPoint { l1: 2.0 / 3.0, l2: 1.0 / 6.0, l3: 1.0 / 6.0, w: 1.0 / 3.0 },
            QuadPoint { l1: 1.0 / 6.0, l2: 2.0 / 3.0, l3: 1.0 / 6.0, w: 1.0 / 3.0 },
            QuadPoint { l1: 1.0 / 6.0, l2: 1.0 / 6.0, l3: 2.0 / 3.0, w: 1.0 / 3.0 },
        ],
        // Degree 5: 7 points, exact for 5th-degree polynomials
        _ => {
            let a1 = 0.059715871789770;
            let b1 = 0.470142064105115;
            let a2 = 0.797426985353087;
            let b2 = 0.101286507323456;
            let w0 = 0.225000000000000;
            let w1 = 0.132394152788506;
            let w2 = 0.125939180544827;

            vec![
                QuadPoint { l1: 1.0 / 3.0, l2: 1.0 / 3.0, l3: 1.0 / 3.0, w: w0 },
                QuadPoint { l1: a1, l2: b1, l3: b1, w: w1 },
                QuadPoint { l1: b1, l2: a1, l3: b1, w: w1 },
                QuadPoint { l1: b1, l2: b1, l3: a1, w: w1 },
                QuadPoint { l1: a2, l2: b2, l3: b2, w: w2 },
                QuadPoint { l1: b2, l2: a2, l3: b2, w: w2 },
                QuadPoint { l1: b2, l2: b2, l3: a2, w: w2 },
            ]
        }
    }
}

/// Integrate density, moment, and energy over a polygon using fan triangulation
/// and Dunavant quadrature.
///
/// The polygon is fan-triangulated from vertex 0. For each triangle, quadrature
/// points are mapped from barycentric coordinates to Cartesian, then density
/// is evaluated.
#[allow(clippy::assign_op_pattern)]
fn integrate_polygon(
    polygon: &[Point2],
    seed: &Point2,
    density: &impl Fn(Point2) -> Real,
    quad: &[QuadPoint],
) -> (Real, Point2, Real) {
    let n = polygon.len();
    if n < 3 {
        return (0.0, Point2::zero(), 0.0);
    }

    let mut total_mass = 0.0;
    let mut total_moment = Point2::zero();
    let mut total_energy = 0.0;

    let a = polygon[0];
    for i in 1..(n - 1) {
        let b = polygon[i];
        let c = polygon[i + 1];

        // Triangle area (signed, but we use absolute value)
        let area = 0.5 * ((b[0] - a[0]) * (c[1] - a[1]) - (c[0] - a[0]) * (b[1] - a[1]));
        let abs_area = area.abs();

        if abs_area < 1e-15 {
            continue;
        }

        for qp in quad {
            let x = a * qp.l1 + b * qp.l2 + c * qp.l3;
            let rho = density(x);
            let w = qp.w * abs_area;

            total_mass += rho * w;
            total_moment = total_moment + x * (rho * w);
            let dx = x - *seed;
            total_energy += rho * dx.norm_squared() * w;
        }
    }

    (total_mass, total_moment, total_energy)
}

// ============================================================================
// Halton sequence
// ============================================================================

fn halton(index: usize, base: usize) -> Real {
    let mut result = 0.0;
    let mut f = 1.0 / base as Real;
    let mut i = index + 1;
    while i > 0 {
        result += f * (i % base) as Real;
        i /= base;
        f /= base as Real;
    }
    result
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const TOL: Real = 1e-6;

    fn unit_square() -> Domain2D {
        Domain2D::rectangle(0.0, 1.0, 0.0, 1.0)
    }

    #[test]
    fn contains_inside_point() {
        let domain = unit_square();
        assert!(domain.contains(&Point2::new(0.5, 0.5)));
        assert!(domain.contains(&Point2::new(0.1, 0.9)));
    }

    #[test]
    fn contains_outside_point() {
        let domain = unit_square();
        assert!(!domain.contains(&Point2::new(-0.1, 0.5)));
        assert!(!domain.contains(&Point2::new(0.5, 1.1)));
        assert!(!domain.contains(&Point2::new(2.0, 2.0)));
    }

    #[test]
    fn bounding_box_rectangle() {
        let domain = Domain2D::rectangle(1.0, 3.0, 2.0, 5.0);
        let (lo, hi) = domain.bounding_box();
        assert!((lo[0] - 1.0).abs() < TOL);
        assert!((lo[1] - 2.0).abs() < TOL);
        assert!((hi[0] - 3.0).abs() < TOL);
        assert!((hi[1] - 5.0).abs() < TOL);
    }

    #[test]
    fn nearest_boundary_point_inside() {
        let domain = unit_square();
        let p = Point2::new(0.1, 0.5);
        let nearest = domain.nearest_boundary_point(&p);
        // Should be on the left edge at (0.0, 0.5)
        assert!((nearest[0] - 0.0).abs() < TOL);
        assert!((nearest[1] - 0.5).abs() < TOL);
    }

    #[test]
    fn uniform_seeds_count() {
        let domain = unit_square();
        let seeds = domain.uniform_seeds(20);
        assert_eq!(seeds.len(), 20);
        for i in 0..seeds.len() {
            assert!(domain.contains(&seeds[i]), "seed {} is outside domain", i);
        }
    }

    #[test]
    fn integrate_cells_four_quadrants() {
        let domain = unit_square();
        let seeds: Field<Point2> = vec![
            Point2::new(0.25, 0.25),
            Point2::new(0.75, 0.25),
            Point2::new(0.25, 0.75),
            Point2::new(0.75, 0.75),
        ]
        .into_iter()
        .collect();

        let data = domain.integrate_cells(seeds.as_slice(), &|_| 1.0);

        assert_eq!(data.centroids.len(), 4);
        assert_eq!(data.masses.len(), 4);

        for i in 0..4 {
            assert!(
                (data.masses[i] - 0.25).abs() < 0.02,
                "cell {} mass = {} (expected ~0.25)",
                i,
                data.masses[i]
            );
        }

        let total_mass: Real = data.masses.iter().sum();
        assert!(
            (total_mass - 1.0).abs() < 0.05,
            "total mass = {} (expected ~1.0)",
            total_mass
        );
    }

    #[test]
    fn integrate_cells_uniform_density() {
        let domain = unit_square();
        let seeds = domain.uniform_seeds(10);

        let data = domain.integrate_cells(seeds.as_slice(), &|_| 1.0);

        assert_eq!(data.centroids.len(), 10);
        assert_eq!(data.masses.len(), 10);

        // Total mass should be close to 1.0 (area of unit square)
        let total_mass: Real = data.masses.iter().sum();
        assert!(
            (total_mass - 1.0).abs() < 0.05,
            "total mass = {} (expected ~1.0)",
            total_mass
        );
    }

    #[test]
    fn energy_decreases_lloyd() {
        let domain = unit_square();
        let seeds = domain.uniform_seeds(10);

        use crate::meshgen::cvt::lloyd_iter;
        let history: Vec<_> = lloyd_iter(domain, seeds, |_| 1.0).take(20).collect();

        for i in 1..history.len() {
            assert!(
                history[i].energy <= history[i - 1].energy + 1e-10,
                "energy increased at iteration {}: {} > {}",
                i,
                history[i].energy,
                history[i - 1].energy
            );
        }
    }

    #[test]
    fn clip_polygon_inside_domain() {
        // A small square fully inside the unit square -- clipping should return it unchanged
        let polygon = vec![
            Point2::new(0.2, 0.2),
            Point2::new(0.8, 0.2),
            Point2::new(0.8, 0.8),
            Point2::new(0.2, 0.8),
        ];
        let domain = unit_square();
        let clipped = clip_polygon_to_domain(&polygon, &domain.vertices, &domain.edges);
        assert_eq!(clipped.len(), 4);
    }

    #[test]
    fn clip_polygon_partially_outside() {
        // A polygon that extends beyond [0,1]x[0,1]
        let polygon = vec![
            Point2::new(-0.5, 0.5),
            Point2::new(0.5, 0.5),
            Point2::new(0.5, 1.5),
            Point2::new(-0.5, 1.5),
        ];
        let domain = unit_square();
        let clipped = clip_polygon_to_domain(&polygon, &domain.vertices, &domain.edges);
        // Should be clipped to the intersection with the unit square
        assert!(clipped.len() >= 3, "clipped polygon should have >= 3 vertices");

        // All clipped vertices should be inside the domain (or on boundary)
        for v in &clipped {
            assert!(
                v[0] >= -TOL && v[0] <= 1.0 + TOL && v[1] >= -TOL && v[1] <= 1.0 + TOL,
                "clipped vertex ({}, {}) is outside domain",
                v[0],
                v[1]
            );
        }
    }

    #[test]
    fn dunavant_centroid_rule_integrates_constant() {
        let quad = dunavant_rule(1);
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(1.0, 0.0);
        let c = Point2::new(0.0, 1.0);
        let seed = Point2::new(0.0, 0.0);

        let (mass, _, _) = integrate_polygon(&[a, b, c], &seed, &|_| 1.0, &quad);
        // Area of right triangle = 0.5
        assert!(
            (mass - 0.5).abs() < TOL,
            "mass = {} (expected 0.5)",
            mass
        );
    }

    #[test]
    fn dunavant_degree5_integrates_quadratic() {
        let quad = dunavant_rule(5);
        let a = Point2::new(0.0, 0.0);
        let b = Point2::new(1.0, 0.0);
        let c = Point2::new(0.0, 1.0);
        let seed = Point2::new(0.0, 0.0);

        // Integrate f(x,y) = x over triangle (0,0)-(1,0)-(0,1)
        // Expected: 1/6
        let (mass, _, _) = integrate_polygon(&[a, b, c], &seed, &|p: Point2| p[0], &quad);
        assert!(
            (mass - 1.0 / 6.0).abs() < TOL,
            "integral of x = {} (expected 1/6)",
            mass
        );
    }

    #[test]
    fn halton_sequence_basics() {
        // Halton base 2: 1/2, 1/4, 3/4, 1/8, ...
        assert!((halton(0, 2) - 0.5).abs() < TOL);
        assert!((halton(1, 2) - 0.25).abs() < TOL);
        assert!((halton(2, 2) - 0.75).abs() < TOL);
    }
}
