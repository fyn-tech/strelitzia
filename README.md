# Strelitzia

A computational physics library in Rust, designed for high-performance simulations with future GPU support.

## Overview

Strelitzia provides core infrastructure for computational physics applications:

- **Field storage** for simulation data (positions, velocities, stress fields)
- **Visualization export** to VTK format for ParaView
- **Geometry utilities** for mesh generation and Voronoi tessellation

The library is built on `nalgebra` for linear algebra, with a focus on clean interfaces that can evolve to support GPU backends.

## Modules

### Common (`strelitzia::common`)

Crate-wide foundational types, starting with the precision-controlled scalar:

```rust
use strelitzia::common::Real;
// Real is f64 by default, or f32 with the "single-precision" feature
```

### MultiArray (`strelitzia::multiarray`)

The mathematical type system -- all vectors, matrices, and tensors are aliases of `MultiArray<T, S, B>`.

```rust
use strelitzia::multiarray::{Vector3, Matrix3};
use strelitzia::multiarray::linalg::{VectorOps, CrossProduct};

let position = Vector3::new(1.0, 2.0, 3.0);
let rotation = Matrix3::identity();

// Full operator support
let velocity = Vector3::new(0.1, 0.0, 0.0);
let new_pos = position + 0.01 * velocity;

// Domain operations via extension traits
let a = Vector3::new(1.0, 0.0, 0.0);
let b = Vector3::new(0.0, 1.0, 0.0);
let dot = a.dot(&b);       // 0.0
let cross = a.cross(&b);   // (0, 0, 1)
```

### Fields (`strelitzia::fields`)

Collection containers for simulation data, with zero-copy solver interop.

```rust
use strelitzia::common::Real;
use strelitzia::multiarray::Vector3;
use strelitzia::fields::{Vector3Field, ScalarField, SolverInterop};

let mut positions = Vector3Field::new();
positions.push(Vector3::new(1.0, 2.0, 3.0));
positions.push(Vector3::new(4.0, 5.0, 6.0));

// Zero-copy reinterpretation as flat slice for Ax = b solvers
let x: &[Real] = positions.as_flat_slice();
assert_eq!(x, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
```

**Available types:**

| Module | Type | Description |
|--------|------|-------------|
| `common` | `Real` | `f64` (default) or `f32` with `single-precision` feature |
| `multiarray` | `Vector2`, `Vector3`, `Vector4` | Static vector types |
| `multiarray` | `Matrix2`, `Matrix3`, `Matrix4` | Static matrix types |
| `multiarray` | `Point2`, `Point3`, `Point4` | Semantic aliases for positions |
| `fields` | `ScalarField` | Collection of `Real` values |
| `fields` | `Vector3Field` | Collection of `Vector3` values |
| `fields` | `Matrix3Field` | Collection of `Matrix3` values |

### Visualiser (`strelitzia::visualiser`)

Export simulation data to VTK XML format for visualization in ParaView.

```rust
use strelitzia::visualiser::{write_vtu, CellType, Encoding, FieldArray};

// Define mesh coordinates
let coords = vec![0.0, 0.0, 1.0, 0.0, 0.5, 1.0];

// Define cells (triangles)
let cells = vec![(CellType::Triangle, vec![0, 1, 2])];

// Add field data
let temperature = FieldArray::from_slice("temperature", &[100.0, 150.0, 120.0], 1);

write_vtu::<2>(
    "output.vtu",
    Encoding::Ascii,
    &coords,
    &cells,
    &[temperature],  // Point data
    &[],             // Cell data
).unwrap();
```

### Geometry (`strelitzia::geometry`)

Geometric utilities for mesh generation.

```rust
use strelitzia::geometry::{Point2D, generate_voronoi};

let points: Vec<Point2D> = vec![
    (0.0, 0.0),
    (1.0, 0.0),
    (0.5, 1.0),
];

// Generate Voronoi tessellation, optionally save visualization
let trigen = generate_voronoi(&points, Some("voronoi.svg")).unwrap();
```

## Project Structure

```
strelitzia/
├── src/
│   ├── common.rs           # Crate-wide types (Real)
│   ├── multiarray/
│   │   ├── mod.rs          # Module exports
│   │   ├── types.rs        # MultiArray struct, Shape, RawStorage
│   │   ├── traits.rs       # MultiArrayOps, DenseMultiArrayOps, NumericMultiArrayOps
│   │   ├── operators.rs    # Blanket operator impls, matrix multiplication
│   │   ├── aliases.rs      # Type aliases + constructors/accessors
│   │   └── linalg.rs       # Extension traits (VectorOps, CrossProduct, etc.)
│   ├── fields/
│   │   ├── mod.rs          # Module exports
│   │   ├── storage.rs      # Field<T>, FieldElement, SolverInterop
│   │   ├── ops.rs          # Field compound assignment operators
│   │   └── cast.rs         # Legacy zero-copy slice utilities
│   ├── geometry/
│   │   ├── mod.rs
│   │   └── cvt.rs          # Voronoi tessellation
│   ├── visualiser/
│   │   ├── mod.rs
│   │   ├── field_export.rs     # Field-to-VTK conversion
│   │   ├── paraview_writer.rs  # VTK XML export
│   │   ├── vtk_types.rs        # VTK type definitions
│   │   └── encoding.rs         # ASCII/Base64 encoding
│   ├── prelude.rs          # Convenient imports
│   ├── lib.rs
│   └── main.rs
├── tests/
│   ├── fields_tests.rs            # Field module tests
│   ├── fields_ops_tests.rs        # Field operations tests
│   ├── fields_ops_operators_tests.rs # Operator overload tests
│   ├── fields_vtk_tests.rs        # Field VTK export tests
│   └── visualiser_tests.rs        # VTK writer tests
└── Cargo.toml
```

## Features

### Precision Control

Use the `single-precision` feature for `f32` instead of `f64`:

```toml
[dependencies]
strelitzia = { version = "0.1", features = ["single-precision"] }
```

This changes the `Real` type alias and all field storage accordingly.

## Running Tests

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test fields
cargo test visualiser

# Run with output
cargo test -- --nocapture
```

## Dependencies

- **nalgebra** (0.34): Linear algebra types and operations
- **num-traits** (0.2): Generic numeric traits (`Zero`, `One`)
- **bytemuck** (1.24): Safe byte reinterpretation for VTK encoding
- **tritet** (3.0): Delaunay triangulation and Voronoi tessellation
- **plotpy** (1.19): SVG visualisation for geometry output

## Roadmap

### Current Status

- ✅ Field storage with nalgebra backend
- ✅ Zero-copy solver interface
- ✅ Field operations (arithmetic operators, fill, resize, reductions)
- ✅ VTK export for ParaView
- ✅ PVD time series support
- ✅ Voronoi tessellation

### Planned Features

- **Parallel iteration**: Rayon-based `par_iter()` for multi-threaded processing
- **GPU support**: wgpu/CUDA backends for GPU computation
- **Extended math types**: Symmetric matrices, more dimension variants

## Design Philosophy

### Simple First, Optimize Later

The library uses straightforward `Vec<nalgebra::VectorN>` storage rather than complex data layouts. This provides:

- **Immediate productivity**: Standard patterns, full nalgebra API
- **Easy debugging**: Data is inspectable and predictable
- **Future flexibility**: Interface-based design allows storage optimization later

### Interface Stability

Fields expose a trait-based interface so storage can evolve:

- Current: `Vec<Vector3>` (simple, compatible with nalgebra)
- Future: SoA layout for SIMD optimization
- Future: GPU buffers for compute shaders

Your algorithms work with the interface, not the implementation.

### Zero-Copy Where It Matters

The `as_flat_slice()` methods provide zero-copy access for solver interfaces, avoiding unnecessary data copies for billion-element meshes.

## License

This project is part of the Strelitzia computational physics framework.
