//! Geometric primitives used across the library.
//!
//! Currently provides [`axis_aligned_bounding_box::AABBox`], a D-dimensional
//! axis-aligned bounding box suitable for spatial queries and tree construction;
//! [`lines`], `Line`/`Ray`/`Segment` primitives and their pairwise intersections;
//! and [`polygon::Polygon`], a 2D polygon (with holes) supporting point containment.

mod axis_aligned_bounding_box;
mod lines;
mod polygon;
