# Strelitzia

A computational physics library in Rust, designed for high-performance simulations with future GPU support.

## Overview

Strelitzia provides core infrastructure for computational physics applications:

- **Mathematical types** (`Vector3`, `Matrix3`, integer/boolean variants)
- **Field storage** for simulation data (positions, velocities, stress fields)
- **Visualization export** to VTK format for ParaView

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
| `common` | `Int`, `UInt` | `i64`, `u64` (fixed width) |
| `multiarray` | `Vector<T,N>`, `Matrix<T,R,C>` | Generic static types |
| `multiarray` | `Vector2`, `Vector3`, `Vector4` | Static vector types (fixed to `Real`) |
| `multiarray` | `Matrix2`, `Matrix3`, `Matrix4` | Static matrix types (fixed to `Real`) |
| `multiarray` | `Vector2i`..`Vector4i`, `Matrix2i`..`Matrix4i` | Signed integer variants (suffix `i` = `Int`) |
| `multiarray` | `Vector2u`..`Vector4u`, `Matrix2u`..`Matrix4u` | Unsigned integer variants (suffix `u` = `UInt`) |
| `multiarray` | `Vector2b`..`Vector4b`, `Matrix2b`..`Matrix4b` | Boolean variants (suffix `b` = `bool`) |
| `multiarray` | `DynVector<T>`, `DynMatrix<T>` | Dynamic (heap-allocated) types |
| `multiarray` | `Point<T,N>`, `Point2`, `Point3`, `Point4` | Semantic aliases for positions |
| `multiarray` | `MultiIndex<N>`, `MultiIndex2`, `MultiIndex3`, `MultiIndex4` | Index aliases (`Point<usize, N>`) |
| `multiarray` | `X_AXIS2`, `Y_AXIS2` | Compile-time 2D basis vector constants (`Vector2`) |
| `multiarray` | `X_AXIS3`, `Y_AXIS3`, `Z_AXIS3` | Compile-time 3D basis vector constants (`Vector3`) |
| `multiarray` | `X_AXIS`, `Y_AXIS`, `Z_AXIS` | Convenience aliases (default to 3D) |
| `fields` | `RealField` (`ScalarField`) | Collection of `Real` values |
| `fields` | `IntField`, `UIntField`, `BoolField` | Scalar integer/boolean collections |
| `fields` | `Vector3Field`, `Vector3iField`, `Vector3uField`, `Vector3bField` | Vector3 collections |
| `fields` | `Matrix3Field`, `Matrix3iField`, `Matrix3uField`, `Matrix3bField` | Matrix3 collections |

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

## Project Structure

```
strelitzia/
├── src/
│   ├── common.rs           # Crate-wide types (Real, Int, UInt)
│   ├── multiarray/
│   │   ├── mod.rs          # Module exports
│   │   ├── types.rs        # MultiArray struct, Shape, RawStorage
│   │   ├── traits.rs       # MultiArrayOps, DenseMultiArrayOps, NumericMultiArrayOps
│   │   ├── operators.rs    # Backend-delegating + element-wise ops, matrix multiplication
│   │   ├── aliases.rs      # Type aliases (Real, Int, UInt, bool variants) + constructors
│   │   └── linalg.rs       # Extension traits (VectorOps, CrossProduct, etc.)
│   ├── fields/
│   │   ├── mod.rs          # Module exports
│   │   ├── storage.rs      # Field<T>, FieldElement, SolverInterop
│   │   ├── ops.rs          # Field compound assignment operators
│   │   └── cast.rs         # Legacy zero-copy slice utilities
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

## Roadmap

### Current Status

- ✅ Field storage with nalgebra backend
- ✅ Zero-copy solver interface
- ✅ Field operations (arithmetic operators, fill, resize, reductions)
- ✅ VTK export for ParaView
- ✅ PVD time series support
### Planned Features

- **Geometry module**: Mesh generation, Voronoi tessellation (being rewritten)
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
