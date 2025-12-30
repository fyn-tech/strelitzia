# Strelitzia

A zero-cost abstraction library for geometric computing in Rust.

## Overview

Strelitzia provides a unified interface for working with geometric primitives (vectors and matrices) across different backend libraries like `nalgebra` and `robust`. The library uses the **adapter pattern** to ensure your core algorithms never depend on specific external crates.

## Key Features

- **Zero-cost abstractions**: All wrappers use `#[repr(transparent)]` and `#[inline(always)]` for zero runtime overhead
- **Backend independence**: Core code depends only on traits, not concrete implementations
- **Plug-and-play**: Easy to switch between different math libraries
- **Type safety**: Compile-time dimension checking with const generics
- **Interoperability**: Seamless conversion between wrapped and unwrapped types
- **Functional design**: Operations are free functions, keeping data structures minimal
- **Operator support**: Full arithmetic operator support (`+`, `-`, `*`, `/`, `+=`, `-=`, `*=`, `/=`)
- **Optimization library integration**: Native support for argmin optimization framework

## Architecture

### Core Traits

The library provides a hierarchy of traits for multi-dimensional arrays:

- **`MultiArray`**: Base trait for all geometric objects (rank 0-2)
  - `type Scalar`: The scalar element type (e.g., `f64`)
  - `const RANK`: Number of dimensions (0 for scalar, 1 for vector, 2 for matrix)
  - `fn shape()`: Returns dimensional information as `&[usize]`
  - `fn len()`: Total number of elements (product of shape)

- **`Vector`**: Trait for rank-1 arrays (vectors)
  - Extends `MultiArray + Index<usize>` (supports `v[i]` syntax)
  - `const DIM`: Vector dimension
  - `fn get_elem(i)`: Get element at index
  - `fn set_elem(i, value)`: Set element at index

- **`Matrix`**: Trait for rank-2 arrays (matrices)
  - `const ROWS`, `const COLS`: Matrix dimensions (supports non-square matrices)
  - `fn get_elem(row, col)`: Get element at position
  - `fn set_elem(row, col, value)`: Set element at position

### Free Functions

Mathematical operations are provided as free functions that operate on types implementing the traits:

- **Vector operations**:
  - `dot(&v1, &v2)` - Dot product
  - `norm_squared(&v)` - Squared Euclidean norm
  - `zeros::<V>()` - Create zero vector

- **Matrix operations**:
  - `matrix_zeros::<M>()` - Create zero matrix
  - `identity::<M>()` - Create identity matrix (square matrices only)
  - `transpose(&m)` - Transpose matrix
  - `outer(&v1, &v2)` - Outer product (v1 ⊗ v2)
  - `matrix_vec_mul(&m, &v)` - Matrix-vector multiplication (M × v)
  - `matrix_mul(&m1, &m2)` - General matrix multiplication (supports non-square matrices)

This design keeps the traits minimal and makes it easy to add new operations without modifying the traits.

### Adapters

Adapters provide zero-cost wrappers around external crate types:

- **`RobustCoord2D<T>`**: Wraps `robust::Coord<T>` (2D vector)
- **`RobustCoord3D<T>`**: Wraps `robust::Coord3D<T>` (3D vector)
- **`NalgebraVec<T, N>`**: Wraps `nalgebra::SVector<T, N>` (N-dimensional vector)
- **`NalgebraMat<T, R, C>`**: Wraps `nalgebra::SMatrix<T, R, C>` (R×C matrix)

Type aliases for common sizes:
- `NalgebraVec2`, `NalgebraVec3`, `NalgebraVec4`
- `NalgebraMat2`, `NalgebraMat3`, `NalgebraMat4`

### Operator Support

All vector and matrix types support standard arithmetic operators:

- **Binary operators**: `v1 + v2`, `v1 - v2`, `scalar * v`, `v / scalar`
- **Compound assignment**: `v1 += v2`, `v1 -= v2`, `v *= scalar`, `v /= scalar`
- **Negation**: `-v`
- **Indexing**: `v[i]` for vectors, `m[(row, col)]` for matrices

Note: Only `scalar * v` is supported (not `v * scalar`) to maintain consistency with mathematical notation.

### Optimization Integration

Strelitzia provides native integration with the [argmin](https://argmin-rs.org/) optimization framework through `argmin-math` trait implementations:

```rust
use strelitzia::prelude::*;
use argmin::core::{CostFunction, Executor};
use argmin::solver::gradientdescent::SteepestDescent;

// Your wrapper types automatically work with argmin solvers!
let init_param = NalgebraVec3::from(nalgebra::Vector3::new(1.0, 2.0, 3.0));
// Use with any argmin solver...
```

Supported `argmin-math` traits:
- `ArgminDot`, `ArgminAdd`, `ArgminSub`, `ArgminMul`, `ArgminDiv`
- `ArgminL1Norm`, `ArgminL2Norm`
- `ArgminZero`, `ArgminSignum`, `ArgminMinMax`

## Usage

### Basic Example

```rust
use strelitzia::prelude::*;

// Create vectors using different backends
let p1 = RobustCoord2D::new(3.0, 4.0);
let p2 = RobustCoord2D::new(1.0, 2.0);

// Use free functions for operations
println!("Dot product: {}", dot(&p1, &p2));        // 11
println!("Norm squared: {}", norm_squared(&p1));   // 25

// Use operators
let p3 = p1 + p2;                    // Vector addition
let p4 = 2.0 * p1;                   // Scalar multiplication
let p5 = p1[0];                      // Index access

// Still can use robust predicates directly
use robust::{orient2d, Coord};
let a = Coord { x: 0.0, y: 0.0 };
let b = Coord { x: 1.0, y: 0.0 };
let c = Coord { x: 0.0, y: 1.0 };
let orientation = orient2d(a, b, c); // 1 (counterclockwise)
```

### Generic Algorithms

Write algorithms that work with **any** backend:

```rust
use strelitzia::prelude::*;

/// Compute Euclidean distance between two vectors.
/// Works with ANY type that implements the Vector trait!
fn compute_distance<V>(a: &V, b: &V) -> f64
where
    V: Vector<Scalar = f64>,
{
    let mut sum = 0.0;
    for i in 0..V::DIM {
        let diff = a[i] - b[i];  // Can use indexing!
        sum += diff * diff;
    }
    sum.sqrt()
}

// Works with RobustCoord2D
let p1 = RobustCoord2D::new(0.0, 0.0);
let p2 = RobustCoord2D::new(3.0, 4.0);
let dist1 = compute_distance(&p1, &p2); // 5.0

// Also works with NalgebraVec3
let n1 = NalgebraVec3::from(nalgebra::Vector3::new(0.0, 0.0, 0.0));
let n2 = NalgebraVec3::from(nalgebra::Vector3::new(1.0, 2.0, 2.0));
let dist2 = compute_distance(&n1, &n2); // 3.0
```

### Working with Matrices

```rust
use strelitzia::prelude::*;

// Create identity matrix using free function
let mat = identity::<NalgebraMat3<f64>>();

// Access elements
let elem = mat.get_elem(0, 0).unwrap(); // 1.0
let elem2 = mat[(0, 0)];                // Also via indexing!

// Transpose using free function
let transposed = transpose(&mat);

// Matrix-vector multiplication
let v = NalgebraVec3::from(nalgebra::Vector3::new(1.0, 2.0, 3.0));
let result: NalgebraVec3<f64> = matrix_vec_mul(&mat, &v);

// Matrix multiplication (supports non-square matrices)
let m1: NalgebraMat<f64, 3, 2> = matrix_zeros();
let m2: NalgebraMat<f64, 2, 4> = matrix_zeros();
let m3: NalgebraMat<f64, 3, 4> = matrix_mul(&m1, &m2);

// Outer product
let a = NalgebraVec3::from(nalgebra::Vector3::new(1.0, 2.0, 3.0));
let b = NalgebraVec2::from(nalgebra::Vector2::new(4.0, 5.0));
let mat: NalgebraMat<f64, 3, 2> = outer(&a, &b);
```

### Conversion Between Types

```rust
use strelitzia::prelude::*;
use robust::Coord;

// Create from underlying type
let coord = Coord { x: 1.0, y: 2.0 };
let wrapped = RobustCoord2D::from(coord);

// Convert back to underlying type
let unwrapped: Coord<f64> = wrapped.into();

// Access underlying type
let coord_ref = wrapped.as_coord();
```

## Project Structure

```
strelitzia/
├── src/
│   ├── geometry/
│   │   ├── mod.rs
│   │   └── multi_array.rs       # Core traits: MultiArray, Vector, Matrix
│   ├── adapters/
│   │   ├── mod.rs
│   │   ├── robust_adapter.rs    # RobustCoord2D, RobustCoord3D
│   │   ├── nalgebra_adapter.rs  # NalgebraVec, NalgebraMat
│   │   └── argmin_integration.rs # argmin-math trait implementations
│   ├── prelude.rs               # Convenient imports
│   ├── lib.rs
│   └── main.rs                  # Demo examples
└── Cargo.toml
```

## Design Principles

### 1. Zero-Cost Abstractions

All adapters use `#[repr(transparent)]` to ensure the wrapper has the same memory layout as the wrapped type:

```rust
#[repr(transparent)]
pub struct RobustCoord2D<T: Into<f64> + Copy>(pub Coord<T>);
```

Combined with `#[inline(always)]`, the wrapper completely disappears at runtime - there is **zero** performance overhead.

### 2. Separation of Data and Operations

The library follows a functional design philosophy:

- **Traits** define only fundamental data access operations (get/set elements)
- **Free functions** provide mathematical operations (dot product, transpose, etc.)

This separation keeps the traits minimal and makes it easier to add new operations without modifying the traits. It's similar to how Rust's standard library works (e.g., `Iterator` trait with free functions for operations).

### 3. Dependency Inversion

The architecture follows the dependency inversion principle:

- **Core algorithms** depend only on traits (`MultiArray`, `Vector`, `Matrix`)
- **Adapters** depend on both traits and external crates
- **External crates** are never imported in core algorithms

This means you can:
- Switch backends by changing which adapter you use
- Add new backends without modifying existing code
- Test algorithms with mock implementations

### 4. Compile-Time Guarantees

Using const generics for compile-time dimension checking:

```rust
pub struct NalgebraVec<T, const N: usize>(pub na::SVector<T, N>);

impl<T, const N: usize> Vector for NalgebraVec<T, N> {
    const DIM: usize = N;
    // Dimension is known at compile time!
}
```

This prevents dimension mismatches at compile time rather than runtime.

## Running the Examples

```bash
# Build the project
cargo build

# Run the demo
cargo run
```

The demo showcases:
- Using `RobustCoord2D` and `RobustCoord3D` with Vector trait
- Using `NalgebraVec3` and `NalgebraMat3` with Vector/Matrix traits
- Generic `compute_distance()` function that works with any Vector implementation
- Direct use of `robust` predicates alongside wrapped types

## Implementation Details

### Supported Operations

**Vector trait methods (data access):**
- `v[i]` - Index access (via `Index` trait)
- `get_elem(i)` - Get element at index (returns `Option`)
- `set_elem(i, value)` - Set element at index

**Vector operators:**
- `v1 + v2`, `v1 - v2` - Vector addition/subtraction
- `scalar * v` - Scalar multiplication
- `v / scalar` - Scalar division
- `v1 += v2`, `v1 -= v2`, `v *= scalar`, `v /= scalar` - Compound assignment
- `-v` - Negation

**Vector free functions:**
- `zeros::<V>()` - Create zero vector
- `dot(&v1, &v2)` - Dot product
- `norm_squared(&v)` - Squared Euclidean norm (avoids sqrt for performance)

**Matrix trait methods (data access):**
- `m[(row, col)]` - Index access (via `Index` trait)
- `get_elem(row, col)` - Get element at position (returns `Option`)
- `set_elem(row, col, value)` - Set element at position

**Matrix operators:**
- `m1 + m2`, `m1 - m2` - Matrix addition/subtraction
- `scalar * m` - Scalar multiplication
- `m / scalar` - Scalar division
- `m1 += m2`, `m1 -= m2`, `m *= scalar`, `m /= scalar` - Compound assignment
- `-m` - Negation

**Matrix free functions:**
- `matrix_zeros::<M>()` - Create zero matrix
- `identity::<M>()` - Create identity matrix (square matrices only)
- `transpose(&m)` - Transpose the matrix
- `outer(&v1, &v2)` - Outer product (v1 ⊗ v2)
- `matrix_vec_mul(&m, &v)` - Matrix-vector multiplication (M × v)
- `matrix_mul(&m1, &m2)` - General matrix multiplication (supports non-square matrices)

### Current Limitations

- Rank-3+ tensors are not yet supported (by design for simplicity)
- Some advanced operations not yet implemented:
  - Cross product for 3D vectors
  - Matrix determinant and inverse
  - Eigenvalue/eigenvector computation
  - QR, SVD, and other decompositions

These limitations can be easily addressed by extending the free functions as needed.

## Dependencies

- `nalgebra` (0.34.1): Comprehensive linear algebra library
- `robust` (1.2.0): Adaptive precision floating-point arithmetic for computational geometry
- `num-traits` (0.2): Numeric trait abstractions
- `argmin` (0.11.0): Mathematical optimization framework
- `argmin-math` (0.5.1): Math backend abstraction for argmin

## Future Extensions

The architecture is designed to be easily extensible:

1. **Add more operations**: Cross product, determinant, matrix inverse, decompositions, etc.
2. **Add more adapters**: `glam`, `cgmath`, `ultraviolet`, `ndarray`, `sprs`, etc.
3. **Add rank-3 tensors**: Extend `MultiArray` to support higher-rank tensors
4. **Add SIMD support**: Leverage SIMD through backend libraries
5. **Expand argmin integration**: Add implementations for matrix types and other wrapper types

## License

This project is part of the Strelitzia computational geometry framework.