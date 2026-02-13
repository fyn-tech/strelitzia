# MultiArray Module

The mathematical type system for strelitzia. All vectors, matrices, and tensors are aliases of `MultiArray<T, S, B>`.

For the full architectural design, see [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md).

## Quick Start

```rust
use strelitzia::multiarray::{Vector3, Matrix3};
use strelitzia::multiarray::linalg::{VectorOps, CrossProduct};

let v = Vector3::new(1.0, 2.0, 3.0);
let w = Vector3::new(4.0, 5.0, 6.0);

// Arithmetic operators
let sum = v + w;
let scaled = 2.0 * v;
let neg = -v;

// Matrix multiplication
let mat = Matrix3::identity();
let result = mat * v;

// Domain operations via extension traits
let dot = v.dot(&w);
let cross = v.cross(&w);
let norm = v.norm();
let unit = v.normalised();
```

## Type Aliases

| Type | Definition | Description |
|------|-----------|-------------|
| `Vector<T, N>` | `MultiArray<T, Rank1<N>, SVector<T,N>>` | Static vector |
| `Matrix<T, R, C>` | `MultiArray<T, Rank2<R,C>, SMatrix<T,R,C>>` | Static matrix |
| `DynVector<T>` | `MultiArray<T, DynRank1, DVector<T>>` | Dynamic vector |
| `DynMatrix<T>` | `MultiArray<T, DynRank2, DMatrix<T>>` | Dynamic matrix |
| `Vector2/3/4` | `Vector<Real, 2/3/4>` | Convenience aliases |
| `Matrix2/3/4` | `Matrix<Real, N, N>` | Convenience aliases |
| `Point2/3/4` | Same as `Vector2/3/4` | Semantic aliases for positions |

## Operators

All arithmetic via standard Rust `std::ops` traits (blanket impls -- work for all type aliases):

| Operator | Trait | Description |
|----------|-------|-------------|
| `a + b` | `Add` | Element-wise addition |
| `a - b` | `Sub` | Element-wise subtraction |
| `-a` | `Neg` | Negation |
| `scalar * a` | `Mul<MultiArray>` | Scalar multiplication |
| `a / scalar` | `Div<T>` | Scalar division |
| `a += b` | `AddAssign` | In-place addition |
| `a -= b` | `SubAssign` | In-place subtraction |
| `a *= scalar` | `MulAssign<T>` | In-place scalar multiplication |
| `a /= scalar` | `DivAssign<T>` | In-place scalar division |

Matrix multiplication via `*`:

| Expression | Result |
|-----------|--------|
| `Matrix * Matrix` | Matrix (dimension-compatible) |
| `Matrix * Vector` | Vector |
| `Vector * RowVector` | Matrix (outer product) |

## Extension Traits (linalg.rs)

Import the trait to use its methods:

```rust
use strelitzia::multiarray::linalg::{VectorOps, CrossProduct, OuterProduct, Hadamard, Transpose};
```

| Trait | Methods | Applies to |
|-------|---------|-----------|
| `VectorOps<T>` | `dot`, `norm`, `l1_norm`, `l2_norm`, `linf_norm`, `lp_norm`, `norm_squared`, `normalised` | Vectors |
| `CrossProduct<T>` | `cross` | Vector2 (returns Vector3), Vector3 |
| `OuterProduct<T, Rhs>` | `outer` | Vectors (returns Matrix) |
| `Hadamard` | `hadamard` | All MultiArray types |
| `Transpose` | `transpose` | Vectors (returns row matrix), Matrices |

## Files

| File | Contents |
|------|----------|
| `types.rs` | `MultiArray` struct, Shape trait + types, RawStorage/DenseRawStorage + nalgebra impls |
| `traits.rs` | `MultiArrayOps`, `DenseMultiArrayOps`, `NumericMultiArrayOps` + impls, Index/IndexMut |
| `operators.rs` | Blanket operator impls, matrix multiplication, `std::iter::Sum` |
| `aliases.rs` | Type aliases + dimension-specific constructors/accessors |
| `linalg.rs` | Extension traits: VectorOps, CrossProduct, OuterProduct, Hadamard, Transpose, SquareMatrixOps |
