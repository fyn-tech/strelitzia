# Fields Module

Collection containers for simulation data, with zero-copy solver interop.

Mathematical types (`Vector3`, `Matrix3`, etc.) live in `strelitzia::multiarray`. This module provides `Field<T>` -- the container that stores collections of those types for simulation use.

For the full architectural design, see [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md).

## Quick Start

```rust
use strelitzia::multiarray::Vector3;
use strelitzia::fields::{Vector3Field, ScalarField, SolverInterop};

let mut positions = Vector3Field::new();
positions.push(Vector3::new(1.0, 2.0, 3.0));
positions.push(Vector3::new(4.0, 5.0, 6.0));

// Element access and iteration
let p = positions[0];
for pos in positions.iter_mut() {
    *pos *= 2.0;
}

// Zero-copy solver interop
let flat: &[f64] = positions.as_flat_slice();
assert_eq!(flat, &[2.0, 4.0, 6.0, 8.0, 10.0, 12.0]);
```

## Type Aliases

| Type | Definition | Description |
|------|-----------|-------------|
| `ScalarField` | `Field<Real>` | Field of scalars |
| `Vector3Field` | `Field<Vector3>` | Field of 3-vectors |
| `Matrix3Field` | `Field<Matrix3>` | Field of 3x3 matrices |

## Field Operators

**Only compound assignment** -- no binary operators that allocate new Fields:

```rust
field1 += &field2;        // Element-wise addition
field1 -= &field2;        // Element-wise subtraction
field1 *= 2.0;            // Scalar multiplication
field1 /= 2.0;            // Scalar division
field1 += 5.0;            // Scalar addition (broadcast)
field1 -= 3.0;            // Scalar subtraction (broadcast)
```

## Solver Interface

Zero-copy reinterpretation for sparse matrix solvers via `FieldElement` and `SolverInterop`:

```rust
use strelitzia::multiarray::Vector3;
use strelitzia::fields::{Vector3Field, SolverInterop};

let mut field = Vector3Field::new();
field.push(Vector3::new(1.0, 2.0, 3.0));

let flat: &[f64] = field.as_flat_slice();       // [1, 2, 3]
let flat_mut: &mut [f64] = field.as_flat_slice_mut();
```

## Other Operations

```rust
use strelitzia::fields::{FieldOps, ReductionOps, SumOps};

field.fill(0.0);          // Fill all elements
field.resize(10, 0.0);    // Resize with default
field.clear();             // Remove all elements

let max = field.max();     // Option<T>
let min = field.min();     // Option<T>
let sum = field.sum();     // T
```

## Files

| File | Contents |
|------|----------|
| `storage.rs` | `Field<T>`, `FieldElement` trait, `SolverInterop` (generic impl), field type aliases |
| `ops.rs` | Field compound assignment operators, FieldOps, ReductionOps, SumOps |
| `cast.rs` | Legacy zero-copy slice utilities |
