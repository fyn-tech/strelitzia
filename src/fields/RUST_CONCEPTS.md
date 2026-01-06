# Rust Concepts in the Fields Module

A guide for C++/Python developers to understand the Rust features used in this codebase.

---

## 1. Const Generics

**C++ equivalent:** Non-type template parameters

```rust
// Rust
pub struct Field<T, NumT, const N: usize> { ... }

// C++ equivalent
template<typename T, typename NumT, size_t N>
struct Field { ... };
```

`const N: usize` is a compile-time constant. The compiler generates separate code for `Field<_, _, 3>` and `Field<_, _, 9>`.

**Used in:** `layout.rs` — `Chunk<NumT, const N: usize>`, `Field<T, NumT, const N: usize>`

---

## 2. PhantomData

**C++ equivalent:** None (closest: empty tag types for SFINAE)

```rust
use std::marker::PhantomData;

pub struct Field<T, NumT, const N: usize> {
    chunks: Vec<Chunk<NumT, N>>,
    total_len: usize,
    _marker: PhantomData<T>,  // T is not stored, but affects the type signature
}
```

`PhantomData<T>` tells the compiler "this type logically owns a `T`" even though no `T` is stored. This makes `Field<Vector, f64, 3>` and `Field<Tensor, f64, 9>` **different types** at compile time.

**Why not just ignore T?** Without `PhantomData`, the compiler warns about unused type parameters and won't enforce type distinctions.

**Used in:** `layout.rs` — to carry semantic meaning without runtime cost

---

## 3. Zero-Sized Types (ZSTs)

**C++ equivalent:** Empty structs (but C++ gives them 1 byte; Rust gives them 0)

```rust
// types.rs
pub struct Scalar;   // 0 bytes
pub struct Vector;   // 0 bytes
pub struct Tensor;   // 0 bytes
```

These exist purely for compile-time type checking. They generate no runtime code or memory. Combined with `PhantomData`, they create a "type tag" system.

**Python analogy:** Like using class names as enum values, but enforced at compile time.

---

## 4. Trait Bounds

**C++ equivalent:** Concepts (C++20) or SFINAE constraints

```rust
impl<T, NumT: Copy + Default, const N: usize> Field<T, NumT, N> {
    pub fn new() -> Self { ... }
}
```

This reads: "Implement these methods for `Field` **only when** `NumT` implements both `Copy` and `Default`."

| Rust Trait | C++ Equivalent | Meaning |
|------------|----------------|---------|
| `Copy` | Trivially copyable | Can be duplicated with memcpy |
| `Default` | Default-constructible | Has a `T::default()` (like `T()` in C++) |
| `Clone` | Copy-constructible | Has explicit `.clone()` method |

**Used in:** `layout.rs`, `traits.rs` — to ensure elements can be copied and zero-initialized

---

## 5. Traits (Interfaces)

**C++ equivalent:** Abstract base classes / Concepts  
**Python equivalent:** Abstract base classes (ABC) / Protocols

```rust
// traits.rs
pub trait FieldView<NumT, const N: usize> {
    type Output;  // Associated type (like C++ typedef in class)
    fn get(&self, idx: usize) -> Self::Output;
}

// Implementation for a specific type
impl<NumT: Copy + Default, const N: usize> FieldView<NumT, N> for Field<Vector, NumT, N> {
    type Output = [NumT; N];
    fn get(&self, idx: usize) -> Self::Output { ... }
}
```

Unlike C++ virtual methods, Rust traits are **zero-cost** — the compiler monomorphizes (generates concrete code) at compile time. No vtable unless you explicitly use `dyn Trait`.

---

## 6. Visibility Modifiers

**C++ equivalent:** `public`, `private`, `protected` (but module-based, not class-based)

| Rust | C++ Closest | Meaning |
|------|-------------|---------|
| `pub` | `public` | Visible everywhere |
| `pub(crate)` | `internal` (C#) | Visible within this crate only |
| `pub(super)` | — | Visible to parent module |
| (none) | `private` | Visible within this module only |

```rust
pub struct Field<...> {           // Public struct
    pub(crate) chunks: Vec<...>,  // Crate-internal field
    pub(crate) total_len: usize,  // Crate-internal field
    _marker: PhantomData<T>,      // Private field
}
```

**Used in:** `layout.rs` — `Chunk` is `pub(crate)` (internal implementation detail)

---

## 7. Conditional Compilation

**C++ equivalent:** `#ifdef` / `#if defined(...)`

```rust
// aliases.rs
#[cfg(not(feature = "single-precision"))]
pub type Real = f64;

#[cfg(feature = "single-precision")]
pub type Real = f32;
```

Controlled by `Cargo.toml`:
```toml
[features]
single-precision = []
```

Build with: `cargo build --features single-precision`

**Key difference from C++:** Features are additive and type-safe. No header guard issues.

---

## 8. Type Aliases

**C++ equivalent:** `using` / `typedef`

```rust
// aliases.rs
pub type Vector3Field = Field<Vector, Real, 3>;
pub type ScalarField = Field<Scalar, Real, 1>;
```

```cpp
// C++ equivalent
using Vector3Field = Field<Vector, Real, 3>;
```

These create convenient shorthand without runtime cost.

---

## 9. Module System

**C++ equivalent:** Namespaces + headers (but unified)

```rust
// mod.rs
pub mod layout;    // Declares submodule from layout.rs
pub mod types;     // Type markers (Scalar, Vector, etc.)
pub mod traits;
pub mod aliases;

pub use layout::Field;           // Re-export for convenience
pub use aliases::*;              // Re-export everything from aliases
```

**Key differences from C++:**
- No header/source split — `mod.rs` is like a header that auto-includes the implementation
- `pub use` creates re-exports (like `using namespace` but explicit)
- Modules define visibility boundaries, not just namespaces

---

## 10. Arrays vs Vectors

**C++ equivalent:** `std::array` vs `std::vector`

```rust
// Fixed-size array (stack-allocated, size known at compile time)
data: [[NumT; CHUNK_SIZE]; N]  // Like std::array<std::array<NumT, 1024>, N>

// Dynamic vector (heap-allocated, growable)
chunks: Vec<Chunk<NumT, N>>    // Like std::vector<Chunk<NumT, N>>
```

Rust arrays `[T; N]` are always stack-allocated with compile-time known size. `Vec<T>` is heap-allocated and growable.

---

## 11. Method Syntax

**C++ equivalent:** Member functions, but with explicit `self`  
**Python equivalent:** Methods with explicit `self`

```rust
impl<...> Field<...> {
    // Like Python's __init__, but not a constructor
    pub fn new() -> Self { ... }
    
    // &self = const reference (C++: const T&)
    pub fn len(&self) -> usize { self.total_len }
    
    // &mut self = mutable reference (C++: T&)
    pub fn push_raw(&mut self, components: [NumT; N]) { ... }
}
```

| Rust | C++ | Python |
|------|-----|--------|
| `&self` | `const T* this` | `self` (read-only by convention) |
| `&mut self` | `T* this` | `self` (mutating) |
| `self` | Move/consume | N/A |

---

## 12. The `assert!` Macro

**C++ equivalent:** `assert()` or `static_assert`

```rust
assert!(idx < self.total_len, "index {} out of bounds", idx);
```

Unlike C++, Rust assertions are **always enabled** in debug builds and can be configured for release. The message is formatted like `println!`.

---

## Generic Parameter Naming Convention

| Parameter | Meaning | Example Values |
|-----------|---------|----------------|
| `T` | Field type marker | `Scalar`, `Vector`, `Tensor`, `SymmTensor` |
| `NumT` | Numeric storage type | `f64`, `f32` |
| `N` | Number of components | `1`, `3`, `9`, `6` |

---

## Quick Reference Table

| Rust Feature | C++ Equivalent | Python Equivalent |
|--------------|----------------|-------------------|
| `const N: usize` | `template<size_t N>` | N/A |
| `PhantomData<T>` | Tag types | N/A |
| `trait` | Abstract class / Concept | ABC / Protocol |
| `impl Trait for Type` | Template specialization | `class X(Protocol)` |
| `#[cfg(feature)]` | `#ifdef` | N/A |
| `pub(crate)` | Internal linkage | `_prefix` convention |
| `Vec<T>` | `std::vector<T>` | `list` |
| `[T; N]` | `std::array<T, N>` | `tuple` (fixed size) |
| `&self` / `&mut self` | `const` / non-`const` methods | `self` |

---

## Further Reading

- [The Rust Book](https://doc.rust-lang.org/book/) — Official guide
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) — Learn by doing
- [Rust for C++ Programmers](https://github.com/pnkfelix/rust-for-cpp-programmers) — Targeted guide
