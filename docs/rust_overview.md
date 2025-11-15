
# Rust in Practice — Full Explanations + Examples (Book‑Aligned)

**Audience:** Senior engineers with C++/Python background.  **examples** plentiful and runnable.
**Alignment:** Mirrors *The Rust Programming Language* (TRPL) and adds requested real‑world topics:
**extension traits**, **orphan rule**, **newtype**, **FFI (C & C++)**, **GATs**, **const generics**, **object safety**, **concurrency & async**,
**Pin/Unpin**, **MaybeUninit/NonNull/UnsafeCell**, **interior mutability**, **macros**, **attributes**, **testing**, and **tooling**.

---

## Crosswalk to the Rust Book (TRPL)
- **Ch.1–2**: Getting started → §1–§3
- **Ch.3**: Common concepts → §4–§6
- **Ch.4**: Ownership/borrowing/slices → §7–§9
- **Ch.5–6**: Structs & enums/patterns → §10–§12
- **Ch.7**: Modules/paths/visibility → §13
- **Ch.8**: Collections & strings → §14
- **Ch.9**: Error handling → §15
- **Ch.10**: Generics/traits/lifetimes → §16–§19
- **Ch.11**: Testing & docs → §20
- **Ch.12**: I/O mini‑project → §21
- **Ch.13**: Closures & iterators → §22
- **Ch.14**: Cargo → §23
- **Ch.15**: Smart pointers → §24
- **Ch.16**: Concurrency → §25–§27
- **Ch.17**: Async (modern editions) → §28
- **Ch.18**: OO in Rust via traits → §29
- **Ch.19**: Patterns → §12
- **Ch.20**: Advanced features → §30–§37
- **Ch.21**: Web server → §38
- **Appendices**: Keywords/tools/operators → throughout, esp. §39

---

## 1) Tooling, Editions, and Mindset

**What & Why.** Rust pairs C/C++ performance with strong static guarantees. Tooling (Cargo + rustup) is first‑class.
**Mental model.** Expect to reason about ownership, borrowing, and trait bounds. The compiler is your reviewer.
**APIs & commands.**
```bash
rustup update
cargo new app && cd app
cargo run              # debug
cargo build --release  # optimized
cargo fmt && cargo clippy -- -D warnings
cargo test
cargo doc --open
```
**Pitfalls.** “Fix with clone()” is a smell; design correct ownership/borrowing instead.
**Checklist.** CI should run `fmt`, `clippy -D warnings`, `test`, and docs build.

### Examples
**Profiles & lints in `Cargo.toml`:**
```toml
[package]
name = "examples"
version = "0.1.0"
edition = "2021"

[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
```
**Lib + bin structure:**
```
src/
  lib.rs
  bin/
    app.rs
```

---

## 2) Project Layout & Workspaces

**Why workspaces.** Multi‑crate repos encourage separation and reuse.
**Lib vs bin.** Keep logic in a library crate; make thin binaries in `src/bin` that parse args and call into the lib.

### Examples
**Workspace root:**
```toml
[workspace]
members = ["corelib", "app"]
```
**`corelib/src/lib.rs`:**
```rust
pub mod math {
    pub fn mean(xs: &[f64]) -> Option<f64> {
        if xs.is_empty() {
            None
        } else {
            Some(xs.iter().sum::<f64>() / xs.len() as f64)
        }
    }
}

pub mod prelude {
    pub use crate::math::mean;
}
```
**`app/src/main.rs`:**
```rust
use corelib::prelude::*;

fn main() {
    println!("{:?}", mean(&[1.0, 2.0, 3.0]));
}
```

---

## 3) Warm‑Up CLI (Mini‑Grep‑Lite)

**What.** Parse args, read file or stdin, filter lines: exercises ownership, slices, and errors.
**Design.** Return `Result` from lib functions; do printing and `exit` only in `main`.

### Examples
**Read from stdin when no path is provided:**
```rust
use std::{env, fs, io::{self, Read}};

fn read_all(opt_path: Option<&str>) -> io::Result<String> {
    let mut s = String::new();
    match opt_path {
        Some(p) => s = fs::read_to_string(p)?,
        None => io::stdin().read_to_string(&mut s)?,
    }
    Ok(s)
}
```
**Case‑insensitive contains:**
```rust
fn contains_ci(hay: &str, needle: &str) -> bool {
    let hay_lower = hay.to_lowercase();
    let needle_lower = needle.to_lowercase();
    hay_lower.contains(&needle_lower)
}
```
**Context lines around matches:**
```rust
fn grep_context(text: &str, needle: &str, n: usize) {
    let lines: Vec<_> = text.lines().collect();
    for (i, &line) in lines.iter().enumerate() {
        if line.contains(needle) {
            let start = i.saturating_sub(n);
            let end = (i + n + 1).min(lines.len());
            for j in start..end {
                println!("{:>6}: {}", j + 1, lines[j]);
            }
            println!("------");
        }
    }
}
```

---

## 4) Variables, Mutability, and Syntax

**Immutable by default.** Rust variables are immutable unless marked `mut`.
**Shadowing.** Rebinding with `let` creates a new variable; can change type.
**Constants.** Use `const` for compile‑time values; `static` for global state.

### Examples
```rust
// Immutability
let x = 5;
// x = 6;  // error: cannot assign twice

// Mutability
let mut y = 5;
y = 6;

// Shadowing
let x = x + 1;      // new binding, same type
let x = "string";   // new binding, different type

// Constants
const MAX_POINTS: u32 = 100_000;

// Static (mutable statics require unsafe)
use std::sync::atomic::{AtomicUsize, Ordering};
static COUNTER: AtomicUsize = AtomicUsize::new(0);
```

**Expressions.** Blocks return their last expression; `if`, `match`, and `loop { break x }` are expressions.
**Use `let‑else`** for early validation. **Prefer `match`** to multi‑branch `if` for enums/ranges/guards.

### Examples
```rust
fn parse_pair(s: &str) -> (i32, i32) {
    let mut it = s.split(',');
    let Some(a) = it.next().and_then(|t| t.parse().ok()) else {
        panic!("bad a")
    };
    let Some(b) = it.next().and_then(|t| t.parse().ok()) else {
        panic!("bad b")
    };
    (a, b)
}
```
```rust
fn search_2d(grid: &[&[i32]], target: i32) -> Option<(usize, usize)> {
    for (r, row) in grid.iter().enumerate() {
        for (c, &x) in row.iter().enumerate() {
            if x == target {
                return Some((r, c));
            }
        }
    }
    None
}
```
```rust
fn score_to_letter(s: u32) -> &'static str {
    match s {
        90..=100 => "A",
        80..=89 if s % 2 == 0 => "B+",
        80..=89 => "B",
        _ => "C or below",
    }
}
```

---

## 5) Types, Slices, and Strings (UTF‑8)

**Tuples & Arrays.** Tuples group mixed types; arrays are fixed‑size `[T; N]`.
**Strings.** `String` owns UTF‑8 bytes; `&str` is a borrowed slice. Never index by `s[i]`—use iterators or safe slicing.
**Slices.** `&[T]`/`&str` borrow views into data; lifetimes ensure they don’t outlive owners.

### Examples
```rust
// Tuples
let pair: (i32, &str) = (42, "answer");
let (num, text) = pair;  // destructure

// Arrays
let arr: [i32; 3] = [1, 2, 3];
let zeros = [0; 100];  // [0, 0, ..., 0]

// String operations
let mut s = String::from("hello");
s.push_str(" world");
s.push('!');

let greeting = format!("{} {}", "hello", "world");

// String vs &str
let owned: String = "text".to_string();
let borrowed: &str = &owned[..];

// Common methods
let trimmed = "  spaces  ".trim();
let parts: Vec<&str> = "a,b,c".split(',').collect();
```
```rust
fn first_n_chars(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}
```
```rust
fn join_words(words: &[&str]) -> String {
    let capacity: usize = words.iter().map(|w| w.len() + 1).sum();
    let mut out = String::with_capacity(capacity);
    for (i, w) in words.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        out.push_str(w);
    }
    out
}
```
```rust
type UserIdAlias = u64;

struct UserId(u64);

fn takes_newtype(_: UserId) {}

fn takes_alias(_: UserIdAlias) {}
```

---

## 6) Numbers, Conversions, and Overflow

**Rules.** Debug builds panic on overflow; release builds wrap. Prefer `From/TryFrom` to `as` for safety.
**APIs.** `wrapping_*`, `checked_*`, `saturating_*` on integer types.

### Examples
```rust
fn saturating_add_seq(xs: &[u32]) -> u32 {
    xs.iter().copied().fold(0u32, |acc, x| acc.saturating_add(x))
}
```
```rust
use std::convert::TryFrom;

fn narrow(v: u16) -> Option<u8> {
    u8::try_from(v).ok()
}
```
```rust
fn parse_or(s: &str, d: i32) -> i32 {
    s.parse().unwrap_or(d)
}
```

---

## 7) Ownership, Moves, RAII, and Drop

**Problem:** Manual memory management (C/C++) is error-prone; garbage collection (Java/Python) adds runtime overhead and unpredictability.
**Solution:** Ownership system enforces single owner at compile time; automatic cleanup via RAII; zero runtime cost.

**Ownership.** One owner at a time; move by default; small `Copy` types duplicate.
**RAII.** Deterministic cleanup with `Drop` guards; ideal for locks, timers, and transactional scopes.

### Examples
```rust
let a = String::from("hi");
let b = a;         // move
let c = b.clone(); // deep copy
let x = 5;
let y = x;         // Copy
```
```rust
fn take(v: Vec<i32>) -> usize {
    v.len()  // consumes
}

fn lend(v: &[i32]) -> usize {
    v.len()  // borrows
}
```
```rust
struct Timer(std::time::Instant);

impl Timer {
    fn new() -> Self {
        Self(std::time::Instant::now())
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        eprintln!("elapsed: {:?}", self.0.elapsed());
    }
}

fn work() {
    let _t = Timer::new();
    // ...
}
```

**Pitfalls.**
```rust
// ❌ Over-cloning
fn process(data: Vec<i32>) -> Vec<i32> {
    data.clone()  // unnecessary
}

// ✅ Borrow instead
fn process(data: &[i32]) -> Vec<i32> {
    data.to_vec()
}

// ❌ Unnecessary String allocations
fn greet(name: String) { }  // takes ownership

// ✅ Accept &str
fn greet(name: &str) { }
```

---

## 8) Borrowing & Lifetimes (Elision included)

**Problem:** Need to share data without transferring ownership, but must prevent data races and dangling pointers.
**Solution:** Borrow checker enforces aliasing XOR mutability at compile time; lifetimes track borrow validity.

**Borrowing.** Any number of `&T` OR exactly one `&mut T` at a time.
**Lifetimes.** Often elided; annotate when returning borrows linked to inputs.

### Examples
```rust
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```
```rust
let mut v = vec![1,2,3];
let first = &v[0];
println!("{first}");
v.push(4); // ok: previous borrow ended before mutation
```
```rust
fn pick_longer<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}
```

**Pitfalls.**
```rust
// ❌ Returning reference to local
fn bad() -> &str {
    let s = String::from("hi");
    &s  // error: s dropped at end of scope
}

// ✅ Return owned data
fn good() -> String {
    String::from("hi")
}
```

---

## 9) Slices (`&[T]`) and Views

**Why slices.** Zero‑copy views enable ergonomic, allocation‑free APIs. Accept `&[T]` in functions; return `Option<&T>` or sub‑slices.
**Strings.** Use `find` and UTF‑8 boundary‑aware slicing.

### Examples
```rust
fn split_once(s: &str, ch: char) -> Option<(&str, &str)> {
    s.find(ch).map(|i| {
        let before = &s[..i];
        let after = &s[i + ch.len_utf8()..];
        (before, after)
    })
}
```
```rust
fn mean(xs: &[f64]) -> Option<f64> {
    if xs.is_empty() {
        None
    } else {
        Some(xs.iter().sum::<f64>() / xs.len() as f64)
    }
}
```
```rust
fn median(xs: &mut [i32]) -> Option<f64> {
    if xs.is_empty() {
        return None;
    }
    xs.sort_unstable();
    let mid = xs.len() / 2;
    let result = if xs.len() % 2 == 1 {
        xs[mid] as f64
    } else {
        (xs[mid - 1] + xs[mid]) as f64 / 2.0
    };
    Some(result)
}
```

---

## 10) Structs, Methods, Builders, and `Default`

**Principles.** Keep data immutable where possible; use builders for configuration ergonomics; derive `Default`.
**Associated fns.** `new`, `with_*` conventions reduce boilerplate.

### Examples
```rust
#[derive(Default, Debug)]
struct Config {
    retries: u32,
    url: String,
}

impl Config {
    fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = url.into();
        self
    }

    fn with_retries(mut self, r: u32) -> Self {
        self.retries = r;
        self
    }
}
```
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Millis(u64);

impl Millis {
    fn as_secs_f64(self) -> f64 {
        self.0 as f64 / 1000.0
    }
}
```
```rust
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn origin() -> Self {
        Self { x: 0, y: 0 }
    }

    fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}
```

---

## 11) Enums & Exhaustive Matching

**Sum types.** Model valid states explicitly. Exhaustive `match` prevents missing cases; use `#[non_exhaustive]` for public enums you plan to grow.
**Option<T>.** Rust's "no null" solution: `Some(T)` or `None`.
**Result<T, E>.** For recoverable errors: `Ok(T)` or `Err(E)`.

### Examples
```rust
// Option<T> - presence/absence
fn find_user(id: u64) -> Option<User> {
    // ...
}

match find_user(42) {
    Some(user) => println!("{}", user.name),
    None => println!("not found"),
}

// Combinators
let name = find_user(42)
    .map(|u| u.name)
    .unwrap_or_else(|| "Unknown".to_string());

// Result<T, E> - success/failure
fn parse_config(path: &str) -> Result<Config, std::io::Error> {
    let text = std::fs::read_to_string(path)?;
    Ok(Config::parse(&text))
}
```
```rust
enum Op {
    Add(i64, i64),
    Mul(i64, i64),
    Neg(i64),
}

fn eval(op: Op) -> i64 {
    match op {
        Op::Add(a, b) => a + b,
        Op::Mul(a, b) => a * b,
        Op::Neg(x) => -x,
    }
}
```
```rust
#[non_exhaustive]
enum ApiEvent {
    Start,
    Stop,
    // more later
}
```
```rust
let data = Some("hi");
if let Some(s) = data {
    println!("{s}");
}
```

---

## 12) Patterns Everywhere

**Use cases.** Destructure tuples/structs/enums; guards; or‑patterns; `@` bindings.
**Guideline.** Match on types (enums/ranges), not on stringly values.
**Refutable vs irrefutable.** `let` needs irrefutable; `if let`/`while let` accept refutable.

### Examples
```rust
fn category(c: char) -> &'static str {
    match c {
        'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
        x if x.is_ascii_digit() => "digit",
        _ => "other",
    }
}
```
```rust
struct User {
    id: u64,
    name: String,
    email: String,
}

let u = User {
    id: 1,
    name: "A".into(),
    email: "a@x".into(),
};

let User { name, .. } = u;
```
```rust
match 7 {
    n @ 1..=10 => println!("small {n}"),
    _ => {}
}
```
```rust
// while let
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("{top}");
}

// Function parameters
fn print_coords(&(x, y): &(i32, i32)) {
    println!("({x}, {y})");
}

// Multiple patterns
match value {
    1 | 2 | 3 => println!("small"),
    _ => println!("other"),
}
```

---

## 13) Modules, Visibility, and Re‑exports

**Why.** Encapsulation and curated public APIs. `pub use` shapes user‑facing surface.
**Paths.** Use `crate::`, `self::`, `super::` anchors to avoid fragile relative imports.
**File organization.** `mod.rs` or `module_name.rs`; split large modules into files.

### Examples
```rust
// File structure
// src/
//   lib.rs
//   models/
//     mod.rs      // or models.rs at src/ level
//     user.rs
//     post.rs

// In lib.rs
mod models;  // looks for models.rs or models/mod.rs

// In models/mod.rs
pub mod user;
pub mod post;

// Path resolution
use crate::models::user::User;  // absolute from crate root
use super::helper;               // parent module
use self::submodule::Item;       // current module
```
```rust
// lib.rs
mod internal {
    pub fn secret() {}
}

pub mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }
}

pub use math::add;
```
```rust
pub(crate) fn helper() {
    println!("internal helper");
}
```
```rust
mod a {
    pub mod b {
        pub fn f() {}
    }
}

use crate::a::b::f;
```

---

## 14) Collections & the Entry API

**Vec<T>.** Growable array; use `with_capacity` when size is known.
**HashMap.** Amortized O(1); `BTreeMap` for order/ranges.
**Entry API.** Mutate in place without extra lookups.

### Examples
```rust
// Vec operations
let mut v = vec![1, 2, 3];
v.push(4);
v.pop();  // Option<i32>

// Safe access
let third = v.get(2);  // Option<&i32>
// let third = &v[2];  // panics if out of bounds

// Iteration
for item in &v { }       // borrow
for item in &mut v { }   // mutable borrow
for item in v { }        // consume

// Common patterns
v.retain(|x| x % 2 == 0);
let doubled: Vec<_> = v.iter().map(|x| x * 2).collect();
```
```rust
use std::collections::HashMap;

fn counts(words: &str) -> HashMap<String, usize> {
    let mut m = HashMap::new();
    for w in words.split_whitespace() {
        *m.entry(w.to_string()).or_insert(0) += 1;
    }
    m
}
```
```rust
fn group_by_len<'a>(
    ws: impl IntoIterator<Item = &'a str>
) -> std::collections::HashMap<usize, Vec<&'a str>> {
    let mut m = std::collections::HashMap::new();
    for w in ws {
        m.entry(w.len()).or_default().push(w);
    }
    m
}
```
```rust
let mut v = vec![1, 2, 3, 4, 5];
v.retain(|x| x % 2 == 0);  // [2, 4]
```

---

## 15) Error Handling (Result/Option/thiserror/anyhow)

**Problem:** Exceptions hide control flow and make error handling optional; null pointers cause crashes.
**Solution:** `Result<T, E>` and `Option<T>` make errors explicit and composable; `?` operator for ergonomic propagation.

**Philosophy.** No exceptions; explicit `Result<T, E>` enables composition and recovery.
**Library vs app.** Libraries expose typed errors (e.g., `thiserror`); apps often use `anyhow` for ergonomics.

**When to use:**
- `thiserror`: Library code that needs typed, matchable errors
- `anyhow`: Application code prioritizing ergonomics and context
- `Option<T>`: Absence is not an error (e.g., cache miss, optional config)

### Examples
```rust
use thiserror::Error;
#[derive(Debug, Error)]
pub enum ConfigErr {
    #[error("io: {0}")] Io(#[from] std::io::Error),
    #[error("parse int: {0}")] Parse(#[from] std::num::ParseIntError),
    #[error("empty input")] Empty,
}
```
```rust
use anyhow::{Context, Result};
fn run(path: &str) -> Result<i32> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {path}"))?;
    Ok(text.trim().parse::<i32>().context("parsing integer")?)
}
```
```rust
let xs = ["1", "2", "x"];
let parsed: Result<Vec<i32>, _> = xs.iter()
    .map(|s| s.parse())
    .collect();
```

**Pitfalls.**
```rust
// ❌ Using unwrap() in library code
pub fn parse(s: &str) -> i32 {
    s.parse().unwrap()  // panics on invalid input
}

// ✅ Return Result
pub fn parse(s: &str) -> Result<i32, std::num::ParseIntError> {
    s.parse()
}

// ❌ Swallowing errors
let _ = risky_operation();  // ignores error

// ✅ Handle or propagate
risky_operation()?;
```

---

## 16) Traits: Design & Bounds

**Problem:** Need polymorphism without inheritance; want zero-cost abstractions with compile-time guarantees.
**Solution:** Traits define shared behavior; generics enable monomorphization (static dispatch); trait objects enable dynamic dispatch.

**Design.** Keep traits focused; prefer `&self`; use supertraits sparingly; default methods are fine.
**Dispatch.** Use generics for static dispatch; trait objects for runtime polymorphism.

**When to use:**
- **Generics (`<T: Trait>`)**: Known types at compile time, want inlining and optimization
- **Trait objects (`&dyn Trait`)**: Heterogeneous collections, plugin systems, runtime polymorphism

### Examples
```rust
trait Area {
    fn area(&self) -> f64;
}

struct Circle {
    r: f64,
}

impl Area for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.r * self.r
    }
}

fn total_area<T: Area>(xs: &[T]) -> f64 {
    xs.iter().map(|s| s.area()).sum()
}
```
```rust
fn sum_display<T>(xs: &[T])
where
    T: std::fmt::Display + std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    for x in xs {
        println!("{}", x);
    }
}
```
```rust
trait Print {
    fn print(&self);
}

impl<T: std::fmt::Debug> Print for T {
    fn print(&self) {
        println!("{:?}", self);
    }
}
```

---

## 17) Associated Types & HRTBs

**Problem:** Generic traits with type parameters can be implemented multiple times for the same type; closures need to work with borrows of any lifetime.
**Solution:** Associated types enforce single implementation per type; HRTBs (Higher-Rank Trait Bounds) express "for all lifetimes" constraints.

**Associated types.** Types that belong to the trait (e.g., `Iterator::Item`). Use when there's one logical output type per implementation.
**HRTBs.** "for any lifetime" bounds (`for<'a>`) used when a closure or function must work for all borrows. Common with `Fn` traits.

### Examples
```rust
fn apply_all<'a, F>(xs: &'a [i32], f: F) -> i32
where
    for<'b> F: Fn(&'b i32) -> i32,
{
    xs.iter().map(|x| f(x)).sum()
}
```
```rust
trait MyIter {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}
```

---

## 18) GATs (Generic Associated Types)

**Problem:** Associated types can't be parameterized by lifetimes, preventing "lending iterators" that return references into `self`.
**Solution:** GATs allow associated types to be generic over lifetimes or types, enabling self-referential iteration patterns.

**Why.** Express “lending” iterators or associated types parameterized by lifetimes or consts.
**When.** Avoids awkward lifetime plumbing on methods when the associated type truly belongs to the trait.

### Examples
```rust
trait Lender {
    type Iter<'a>: Iterator<Item = &'a str>
    where
        Self: 'a;
    fn lend<'a>(&'a self) -> Self::Iter<'a>;
}

impl Lender for String {
    type Iter<'a> = std::str::SplitWhitespace<'a>;

    fn lend<'a>(&'a self) -> Self::Iter<'a> {
        self.split_whitespace()
    }
}
```

---

## 19) Object Safety, Trait Objects, and `dyn`

**Problem:** Need runtime polymorphism with heterogeneous types, but vtables can't handle `Self` types or generics.
**Solution:** Object-safe traits enable `dyn Trait` with vtable dispatch; compiler enforces restrictions at trait definition.

**Object safety.** Traits used as `dyn Trait` cannot use `Self` in return types or have generic methods (unless `Self: Sized`). Why: vtables need fixed function signatures.
**Use cases.** Heterogeneous collections, plugin systems, late binding.

### Examples
```rust
// not object‑safe
trait Bad {
    fn make(&self) -> Self;
}

// object‑safe
trait Good {
    fn draw(&self);
}

fn use_dyn(g: &dyn Good) {
    g.draw();
}
```
```rust
trait Drawable {
    fn draw(&self) -> String;
}

impl Drawable for String {
    fn draw(&self) -> String {
        format!("text:{self}")
    }
}

impl Drawable for i32 {
    fn draw(&self) -> String {
        format!("num:{self}")
    }
}

fn render(xs: &[&dyn Drawable]) {
    for x in xs {
        println!("{}", x.draw());
    }
}
```

**Pitfalls.**
```rust
// ❌ Not object-safe: returns Self
trait Cloneable {
    fn clone_self(&self) -> Self;  // error: can't use in dyn Cloneable
}

// ✅ Object-safe: use Box<dyn Trait>
trait Cloneable {
    fn clone_box(&self) -> Box<dyn Cloneable>;
}

// ❌ Not object-safe: generic method
trait Process {
    fn process<T>(&self, item: T);  // error: can't monomorphize in vtable
}

// ✅ Use associated types or separate trait
trait Process {
    type Item;
    fn process(&self, item: Self::Item);
}
```

---

## 20) Testing & Documentation

**Problem:** Need to verify correctness, prevent regressions, and ensure examples stay up-to-date.
**Solution:** Rust's built-in test framework with unit tests, integration tests, and executable documentation examples.

**Unit tests.** In `#[cfg(test)]` modules; use `assert!`, `assert_eq!`, `assert_ne!`. Test private implementation details.
**Integration tests.** In `tests/` directory to test public API as external users would. Each file is a separate crate.
**Doc tests.** Code blocks in `///` comments are compiled and run. Ensures examples stay correct.

### Examples
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 2), 4);
    }

    #[test]
    #[should_panic(expected = "divide by zero")]
    fn test_divide_by_zero() {
        divide(10, 0);
    }

    #[test]
    #[ignore]
    fn expensive_test() {
        // Run with: cargo test -- --ignored
    }
}

// Integration test: tests/integration_test.rs
use my_crate;

#[test]
fn test_public_api() {
    assert_eq!(my_crate::add(2, 2), 4);
}
```
```rust
/// Returns the first word, or empty string if none.
///
/// ```
/// assert_eq!(mycrate::first_word("hi there"), "hi");
/// assert_eq!(mycrate::first_word(""), "");
/// ```
pub fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}
```

---

## 21) I/O Mini-Project (Library-First)

**Problem:** CLI apps are hard to test; I/O makes unit tests slow; mixing concerns makes code brittle.
**Solution:** Separate library (pure logic) from binary (CLI parsing); use trait objects (`Read`/`Write`) for testability.

**Design.** Trait-based I/O improves testability; use `Cursor<Vec<u8>>` for in-memory tests; return `Result` for error handling.
**Separation.** CLI (`main.rs`) parses args and handles I/O; lib (`lib.rs`) exposes pure, testable functions.

### Examples
```rust
use std::io::{self, BufRead, Write};

fn filter<R: BufRead, W: Write>(
    mut r: R,
    mut w: W,
    needle: &str,
) -> io::Result<()> {
    let mut line = String::new();
    while r.read_line(&mut line)? != 0 {
        if line.contains(needle) {
            w.write_all(line.as_bytes())?;
        }
        line.clear();
    }
    Ok(())
}
```
```rust
#[test]
fn filters() {
    let input = "a\nx\na\n";
    let mut out = Vec::new();
    filter(std::io::Cursor::new(input), &mut out, "a").unwrap();
    assert_eq!(String::from_utf8(out).unwrap(), "a\na\n");
}
```

---

## 22) Closures & Iterators Deep Dive

**Captures.** by ref → `Fn`, by mut → `FnMut`, by value → `FnOnce`; `move` forces by value.
**Iterators.** Prefer chaining to indexing; zero‑cost abstractions.
**Adaptors.** `map`, `filter`, `take`, `skip`, `zip`, `enumerate`, `flat_map`.

### Examples
```rust
// Closure captures
let x = 5;
let f = || x + 1;        // Fn (borrows x)

let mut y = 5;
let mut g = || y += 1;   // FnMut (mutably borrows y)

let z = String::from("hi");
let h = move || z;       // FnOnce (takes ownership)

// Iterator adaptors
let data = vec![1, 2, 3, 4, 5];

let doubled: Vec<_> = data.iter().map(|x| x * 2).collect();
let evens: Vec<_> = data.iter().filter(|&&x| x % 2 == 0).collect();
let sum: i32 = data.iter().sum();

// Chaining
let result: Vec<_> = data
    .iter()
    .filter(|&&x| x > 2)
    .map(|x| x * 2)
    .take(2)
    .collect();

// enumerate, zip
for (i, &val) in data.iter().enumerate() {
    println!("{i}: {val}");
}

let a = vec![1, 2, 3];
let b = vec![4, 5, 6];
let pairs: Vec<_> = a.iter().zip(&b).collect();

// flat_map
let nested = vec![vec![1, 2], vec![3, 4]];
let flat: Vec<_> = nested.iter().flat_map(|v| v).collect();
```

**Pitfalls.**
```rust
// ❌ Collecting unnecessarily
let sum = data.iter().collect::<Vec<_>>().iter().sum();

// ✅ Chain directly
let sum: i32 = data.iter().sum();

// ❌ Wrong iterator method
for x in data.iter() {  // x is &i32
    consume(x);  // if consume needs i32, must deref
}

// ✅ Use into_iter for owned values
for x in data.into_iter() {  // x is i32
    consume(x);
}
```
```rust
let prefix_sums: Vec<i32> = [1, 2, 3, 4]
    .into_iter()
    .scan(0, |acc, x| {
        *acc += x;
        Some(*acc)
    })
    .collect();
```
```rust
let product: Result<i32, &'static str> = [1, 2, 0, 3]
    .into_iter()
    .try_fold(1, |acc, x| {
        if x == 0 {
            Err("zero")
        } else {
            Ok(acc * x)
        }
    });
```
```rust
struct Counter {
    n: usize,
}

impl Iterator for Counter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.n += 1;
        (self.n <= 3).then_some(self.n)
    }
}
```

---

## 23) Cargo Deep Dive

**Features.** Gate optional deps/APIs; default minimal public surface.
**Profiles.** Tune LTO, codegen units, and opt levels; `cargo tree` and `cargo expand` are invaluable.

### Examples
```toml
[features]
fast = []
```
```rust
#[cfg(feature = "fast")]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}

#[cfg(not(feature = "fast"))]
fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| x * y).sum()
}
```

---

## 24) Smart Pointers & Interior Mutability

**Problem:** Ownership rules prevent shared mutable state; need heap allocation for large/recursive types; reference cycles cause leaks.
**Solution:** Smart pointers provide controlled sharing (`Rc/Arc`), interior mutability (`RefCell/Mutex`), and weak references.

**When to use:**
- `Box<T>`: Heap allocation for large structs, recursive types (e.g., linked lists, trees)
- `Rc<T>`: Shared ownership, single-threaded (e.g., graph nodes, caches)
- `Arc<T>`: Shared ownership, multi-threaded (thread-safe `Rc`)
- `Cell<T>`: Interior mutability for `Copy` types (no runtime checks)
- `RefCell<T>`: Interior mutability with runtime borrow checking (panics on violation)
- `Mutex<T>/RwLock<T>`: Thread-safe interior mutability (blocks on contention)
- `Weak<T>`: Break reference cycles (doesn't prevent deallocation)

**Decision tree:**
```
Need shared ownership?
  → Single-threaded? → Rc<T>
  → Multi-threaded? → Arc<T>
  → Need mutation? → Wrap in RefCell<T> (single) or Mutex<T> (multi)
  → Potential cycles? → Use Weak<T> for back-references
```

**Deref coercion.** `Deref` trait enables `&String` → `&str`, `&Box<T>` → `&T`.

### Examples
```rust
// Deref coercion
fn takes_str(s: &str) { }

let owned = String::from("hello");
takes_str(&owned);  // &String coerces to &str via Deref

let boxed = Box::new(5);
let val: &i32 = &boxed;  // &Box<i32> coerces to &i32
```
```rust
use std::{rc::Rc, cell::RefCell};

let shared: Rc<RefCell<Vec<i32>>> = Rc::new(RefCell::new(vec![]));
shared.borrow_mut().push(1);
```
```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0));
{
    let c = Arc::clone(&counter);
    *c.lock().unwrap() += 1;
}
```
```rust
// Reference cycles & Weak
use std::{rc::{Rc, Weak}, cell::RefCell};

#[derive(Default)]
struct Node {
    next: RefCell<Option<Rc<Node>>>,
    prev: RefCell<Option<Weak<Node>>>,  // Weak breaks cycle
}

// Without Weak, circular Rc references leak memory
// Weak doesn't increment strong count; use upgrade() to get Option<Rc<T>>
```

---

## 25) Threads, Channels, and Scoped Threads

**Problem:** Need parallelism for CPU-bound work; sharing mutable state across threads is error-prone; normal threads can't borrow from stack.
**Solution:** `Send`/`Sync` traits prevent data races at compile time; channels enable safe message passing; scoped threads allow stack borrowing.

**Threads.** Use `Arc<Mutex<_>>` to share mutable state; prefer channels (`mpsc`, `crossbeam`) for message passing (easier to reason about).
**Scoped threads.** Borrow from parent stack safely with `thread::scope` (threads guaranteed to finish before scope ends).

### Examples
```rust
use std::{sync::{mpsc, Arc, Mutex}, thread};

let (tx, rx) = mpsc::channel::<i32>();
let rx = Arc::new(Mutex::new(rx));
for id in 0..2 {
    let rx = Arc::clone(&rx);
    thread::spawn(move || {
        while let Ok(x) = rx.lock().unwrap().recv() {
            println!("worker{id}: {x}");
        }
    });
}
for n in 0..5 {
    tx.send(n).unwrap();
}
drop(tx);
```
```rust
std::thread::scope(|s| {
    let mut data = vec![1,2,3];
    s.spawn(|| { /* can read data */ });
    s.spawn(|| { /* borrow different parts carefully */ });
    drop(data); // enforced by scope lifetime
});
```

---

## 26) Atomics & Memory Ordering

**Problem:** Mutexes have overhead; need lock-free counters or flags; must synchronize across threads without locks.
**Solution:** Atomic types provide lock-free operations with explicit memory ordering guarantees.

**When to use:**
- **Atomics**: Simple counters, flags, lock-free algorithms (expert-level)
- **Mutex/RwLock**: Default choice for shared mutable state (easier to reason about)

**Orderings:**
- `Relaxed`: Atomicity only, no ordering guarantees (use for counters where order doesn't matter)
- `Acquire/Release`: Synchronizes with matching release/acquire (use for producer-consumer patterns)
- `SeqCst`: Total global ordering (easiest to reason about, slight performance cost)
- **Rule of thumb**: Start with `SeqCst`, optimize to `Acquire/Release` if profiling shows need.

### Examples
```rust
use std::sync::atomic::{AtomicUsize, Ordering};

static HITS: AtomicUsize = AtomicUsize::new(0);

fn hit() {
    HITS.fetch_add(1, Ordering::Relaxed);
}
```
```rust
use std::sync::atomic::{AtomicBool, Ordering};

static READY: AtomicBool = AtomicBool::new(false);

fn producer() {
    // write data
    READY.store(true, Ordering::Release);
}

fn consumer() {
    while !READY.load(Ordering::Acquire) {}
    // read data
}
```

---

## 27) Concurrency Patterns (Worker Pool, Pipelines)

**Problem:** Need to process many tasks concurrently; unbounded queues cause memory issues; need coordinated shutdown.
**Solution:** Worker pool pattern with bounded channels; pipeline stages for data flow; graceful shutdown via channel drop.

**Patterns:**
- **Bounded queues**: Provide backpressure (sender blocks when full)
- **Fan-in/fan-out**: Multiple producers/consumers on same channel
- **Pipelining**: Chain stages with channels between them
- **Graceful shutdown**: Drop sender to signal workers; workers exit when channel empty

### Examples

**Worker pool with bounded channel:**
```rust
use std::sync::mpsc::sync_channel;
use std::thread;

fn worker_pool() {
    let (tx, rx) = sync_channel::<i32>(10);
    let rx = std::sync::Arc::new(std::sync::Mutex::new(rx));

    // Spawn 4 workers
    for id in 0..4 {
        let rx = std::sync::Arc::clone(&rx);
        thread::spawn(move || {
            while let Ok(job) = rx.lock().unwrap().recv() {
                println!("Worker {} processing {}", id, job);
                thread::sleep(std::time::Duration::from_millis(100));
            }
        });
    }

    // Send work (blocks when queue is full - backpressure)
    for i in 0..20 {
        tx.send(i).unwrap();
    }
    drop(tx); // Close channel for graceful shutdown
}
```

**Pipeline pattern (multi-stage processing):**
```rust
use std::sync::mpsc::channel;
use std::thread;

fn pipeline() {
    let (tx1, rx1) = channel::<i32>();
    let (tx2, rx2) = channel::<i32>();

    // Stage 1: multiply by 2
    thread::spawn(move || {
        for x in rx1 {
            tx2.send(x * 2).unwrap();
        }
    });

    // Stage 2: add 10
    thread::spawn(move || {
        for x in rx2 {
            println!("Result: {}", x + 10);
        }
    });

    // Send input
    for i in 1..=5 {
        tx1.send(i).unwrap();
    }
    drop(tx1); // Triggers cascade shutdown
}
```

---

## 28) Async/Await (Tokio Essentials)

**Problem:** Threads are expensive for I/O-bound workloads (10k+ concurrent connections); blocking wastes resources.
**Solution:** Async/await enables cooperative multitasking; runtime polls futures; single thread handles many tasks.

**Model.** `async fn` → `Future` (state machine); `.await` yields control; runtimes (tokio/async-std) poll tasks cooperatively.
**Practice.** Use `join!`/`select!`; be cancellation-safe.

**When to use:**
- **Async**: I/O-bound workloads (web servers, databases, network services), high concurrency
- **Threads**: CPU-bound workloads, need true parallelism, simpler mental model

### Examples
```rust
#[tokio::main]
async fn main() {
    println!("{}", add(2, 3).await);
}

async fn add(a: i32, b: i32) -> i32 {
    a + b
}
```
```rust
use tokio::join;

async fn slow(n: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(n)).await;
}

async fn both() {
    join!(slow(10), slow(20));
}
```
```rust
use tokio::{time, time::timeout};

async fn fetch() -> Result<String, Box<dyn std::error::Error>> {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    Ok("data".to_string())
}

async fn fetch_with_timeout() -> anyhow::Result<()> {
    timeout(time::Duration::from_secs(2), fetch()).await??;
    Ok(())
}
```

**Pitfalls: Cancellation Safety**
```rust
// ❌ NOT cancellation-safe: loses data if select! drops this branch
async fn process_queue(queue: &mut VecDeque<Item>) {
    loop {
        tokio::select! {
            item = queue.pop_front() => {  // if other branch wins, item is lost!
                process(item).await;
            }
            _ = shutdown.recv() => break,
        }
    }
}

// ✅ Cancellation-safe: peek before removing
async fn process_queue(queue: &mut VecDeque<Item>) {
    loop {
        let item = queue.front();  // peek, don't remove yet
        tokio::select! {
            _ = process(item).await => {
                queue.pop_front();  // only remove after processing
            }
            _ = shutdown.recv() => break,
        }
    }
}
```

---

## 29) OO‑Style APIs with Traits

**Encapsulation via modules; polymorphism via traits.** Strategy, Visitor, and Plugin patterns map cleanly.
**Guidance.** Prefer generics when concrete, trait objects when heterogeneous.

### Examples
```rust
trait Strategy {
    fn run(&self, n: i32) -> i32;
}

struct AddOne;

impl Strategy for AddOne {
    fn run(&self, n: i32) -> i32 {
        n + 1
    }
}

struct MulTwo;

impl Strategy for MulTwo {
    fn run(&self, n: i32) -> i32 {
        n * 2
    }
}

fn apply_all(xs: &[&dyn Strategy], n: i32) -> Vec<i32> {
    xs.iter().map(|s| s.run(n)).collect()
}
```
```rust
fn make(name: &str) -> Box<dyn Strategy> {
    match name {
        "add" => Box::new(AddOne),
        "mul" => Box::new(MulTwo),
        _ => Box::new(AddOne),
    }
}
```

---

## 30) Extension Traits

**Problem:** Want to add methods to foreign types (e.g., `str`, `Vec<T>`) but orphan rule prevents implementing foreign traits on foreign types.
**Solution:** Define your own trait, implement it for the foreign type, users import the trait to get the methods.

**Why.** Add methods to foreign types coherently without conflicting impls.
**Name.** Use clear, domain-specific names (e.g., `StrExt`, `SliceExt`); consider an `ext` module to avoid pollution.

### Examples

**Define extension trait:**
```rust
pub trait StrExt {
    fn shout(&self) -> String;
    fn snake(&self) -> String;
}

impl StrExt for str {
    fn shout(&self) -> String {
        self.to_uppercase() + "!"
    }

    fn snake(&self) -> String {
        self.split_whitespace()
            .collect::<Vec<_>>()
            .join("_")
            .to_lowercase()
    }
}
```

**Usage (must import the trait):**
```rust
use crate::StrExt;

fn main() {
    let greeting = "hello world";
    println!("{}", greeting.shout());  // "HELLO WORLD!"
    println!("{}", greeting.snake());  // "hello_world"
}
```

**Another extension trait:**
```rust
pub trait BytesHex {
    fn hex_upper(&self) -> String;
}

impl BytesHex for [u8] {
    fn hex_upper(&self) -> String {
        self.iter().map(|b| format!("{:02X}", b)).collect()
    }
}
```

**Usage:**
```rust
use crate::BytesHex;

fn main() {
    let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    println!("{}", data.hex_upper());  // "DEADBEEF"
}
```

---

## 31) Orphan Rule & Coherence

**Problem:** Multiple crates could implement the same foreign trait for the same foreign type, causing conflicts.
**Solution:** Orphan rule requires you own either the trait or the type; ensures global coherence (one impl per type).

**Rule.** You can `impl Trait for Type` only if you own the trait or the type. Prevents diamond dependency conflicts.  
**Patterns.** Use extension traits (own the trait) or **newtype** wrappers (own the type) to satisfy the rule.



### Examples
```rust
// impl std::fmt::Display for Vec<u8> { /* error: both foreign */ }
```
```rust
use std::fmt::{self, Display};

struct Bytes(pub Vec<u8>);

impl Display for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &b in &self.0 {
            write!(f, "{:02X}", b)?;
        }
        Ok(())
    }
}
```

---

## 32) Newtype Pattern & Sealed Traits

**Problem (Newtype):** Need to implement foreign trait on foreign type; want type-level guarantees (e.g., validated email).
**Solution:** Wrap in a tuple struct (`struct Email(String)`); zero-cost abstraction, distinct type for trait impls.

**Problem (Sealed):** Public trait allows downstream impls, breaking exhaustiveness or invariants in your code.
**Solution:** Sealed trait pattern uses private supertrait; only your crate can implement it.

**Newtype.** Zero-cost wrapper giving a distinct type for traits/invariants/semantics.
**Sealed traits.** Prevent downstream crates from adding impls to your public traits (use for closed enums-as-traits).

### Examples
```rust
struct NonEmpty(String);

impl NonEmpty {
    fn parse(s: &str) -> Option<Self> {
        if s.is_empty() {
            None
        } else {
            Some(Self(s.to_string()))
        }
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}
```
```rust
mod sealed {
    pub trait Sealed {}
}

pub trait Stable: sealed::Sealed {
    fn id(&self) -> u32;
}

impl sealed::Sealed for i32 {}

impl Stable for i32 {
    fn id(&self) -> u32 {
        *self as u32
    }
}
```

---

## 33) FFI with C

**Problem:** Need to call C libraries or expose Rust to C; Rust's layout/ABI differs from C.
**Solution:** `#[repr(C)]` matches C layout; `extern "C"` uses C calling convention; `unsafe` blocks acknowledge unchecked invariants.

**ABI.** Use `#[repr(C)]` / `#[repr(transparent)]` for struct layout; export functions with `#[no_mangle] extern "C"`.
**Strings.** `CString`/`CStr` for null-terminated strings; **critical**: decide allocation ownership (who calls `free`?).

### Examples
```rust
#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
```
```rust
use std::ffi::CString;
use std::os::raw::c_char;

extern "C" {
    fn puts(s: *const c_char) -> i32;
}

fn main() {
    let s = CString::new("Hello").unwrap();
    unsafe {
        puts(s.as_ptr());
    }
}
```
```rust
#[no_mangle]
pub extern "C" fn make_buf(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len);
    let p = v.as_mut_ptr();
    std::mem::forget(v); // transfer ownership; provide a matching free_buf in your API
    p
}
```

**Pitfalls.**
```rust
// ❌ Undefined behavior: C code modifies Rust reference
#[no_mangle]
pub extern "C" fn process(data: &mut [u8]) {  // C doesn't know Rust aliasing rules!
    // If C code aliases this pointer, UB!
}

// ✅ Use raw pointers at FFI boundary
#[no_mangle]
pub unsafe extern "C" fn process(data: *mut u8, len: usize) {
    let slice = std::slice::from_raw_parts_mut(data, len);  // document safety requirements
    // ...
}

// ❌ Memory leak: Rust allocates, C never frees
#[no_mangle]
pub extern "C" fn get_string() -> *mut c_char {
    CString::new("hello").unwrap().into_raw()  // C must call free_string!
}

// ✅ Provide matching free function
#[no_mangle]
pub unsafe extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}
```

---

## 34) FFI with C++ (`cxx`/`bindgen`)

**Problem:** C++ has name mangling, templates, exceptions, move semantics—incompatible with C ABI.
**Solution:** `cxx` provides safe bridge with shared types; `bindgen` generates raw bindings for manual wrapping.

**Approaches:**
- `cxx`: Safe, opinionated interop with modern C++ (recommended for new projects)
- `bindgen`: Generate raw bindings from headers; wrap with safe Rust APIs (for existing C++ codebases)

**Guidance.** Keep the unsafe boundary thin; document invariants, nullability, and lifetimes. Prefer `cxx` for safety.

### Examples

**Using `cxx` crate (safe bridge):**
```rust
// build.rs
fn main() {
    cxx_build::bridge("src/bridge.rs")
        .file("src/cpp/wrapper.cpp")
        .flag_if_supported("-std=c++17")
        .compile("mycxxbridge");
}
```
```rust
// src/bridge.rs
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("wrapper.h");

        type MyClass;

        fn create_instance() -> UniquePtr<MyClass>;
        fn process(&self, value: i32) -> i32;
    }
}

pub fn use_cpp() {
    let obj = ffi::create_instance();
    let result = obj.process(42);
    println!("Result: {}", result);
}
```

**Using `bindgen` (raw bindings):**
```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-lib=mylib");
    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
```
```rust
// src/lib.rs - wrap generated bindings safely
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn safe_wrapper(x: i32) -> i32 {
    unsafe { cpp_function(x) }
}
```

---

## 35) Const, Static, `const fn`, and Const Generics

**Const generics.** Parameterize over values; ideal for fixed sizes and array‑backed algorithms.
**`const fn`.** Compute in const contexts; initialize `static` data safely.

### Examples
```rust
struct Matrix<T, const R: usize, const C: usize> {
    data: [[T; C]; R],
}

impl<T: Default + Copy, const R: usize, const C: usize> Default for Matrix<T, R, C> {
    fn default() -> Self {
        Self {
            data: [[T::default(); C]; R],
        }
    }
}
```
```rust
const POW2: [u32; 8] = [1, 2, 4, 8, 16, 32, 64, 128];
```

---

## 36) Pin/Unpin, Self-Refs, and Futures

**Problem:** Self-referential structs (e.g., async state machines) have pointers into themselves; moving invalidates pointers.
**Solution:** `Pin<P>` is a compiler-enforced promise: "this value won't move in memory." Enables safe self-referential types.


**Pinning.** `Pin<P>` prevents moves after pinning; most types are `Unpin` (can be moved even when pinned). Self-referential types are `!Unpin`.  
**Async.** Executors pin futures; async state machines are self-referential. Don't manually move pinned futures.

**Mental model:** Think of `Pin` as "address stability guarantee"—once pinned, the value stays at that memory address until dropped.

### Examples
```rust
use std::pin::Pin;

// Note: This works because slices are Unpin (most types are)
fn use_pinned(mut buf: Pin<&mut [u8]>) {
    let s: &mut [u8] = &mut *buf;
    s.fill(0);
}
```
```rust
fn is_unpin<T: Unpin>() {}

is_unpin::<i32>();  // compiles: i32 is Unpin
is_unpin::<String>();  // compiles: String is Unpin
```

---

## 37) `MaybeUninit`, `NonNull`, and `UnsafeCell`

**Problem:** Need uninitialized memory for performance; raw pointers can be null; need interior mutability without runtime checks.
**Solution:** These unsafe primitives enable building safe abstractions (e.g., `Vec`, `Cell`, `RefCell`).

**Why.** Lower-level building blocks for safe abstractions:
- `MaybeUninit<T>`: Uninitialized memory without UB (use for arrays, FFI buffers)
- `NonNull<T>`: Non-null raw pointer with covariance (use in data structures like `Box`, `Vec`)
- `UnsafeCell<T>`: The **only** legal way to get `&mut T` from `&T` (foundation of `Cell`, `RefCell`, `Mutex`)

**When to use:** Only when building low-level abstractions. Most code should use safe wrappers (`Vec`, `RefCell`, etc.).

### Examples
```rust
use std::mem::MaybeUninit;

fn make_array() -> [u32; 4] {
    let mut a: [MaybeUninit<u32>; 4] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    for i in 0..4 {
        a[i].write(i as u32);
    }
    unsafe { std::mem::transmute::<_, [u32; 4]>(a) }
}
```
```rust
use std::ptr::NonNull;

let mut x = 5;
let p = NonNull::new(&mut x as *mut i32).unwrap();
```

---

## 38) Mini‑Project: Tiny Web Server (Thread Pool)

**Overview.** Listener → channel → worker threads → parse → respond; graceful shutdown by closing senders.
**Scaling.** Tune backlog, timeouts, and connection pool; offload CPU‑heavy routes to worker pool.

### Example (sketch)
```rust
use std::{
    sync::{mpsc, Arc, Mutex},
    thread,
    net::{TcpListener, TcpStream},
    io::{Read, Write},
};

fn handle(mut s: TcpStream) {
    let mut buf = [0u8; 512];
    let _ = s.read(&mut buf);
    let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length:2\r\n\r\nOK");
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    let (tx, rx) = mpsc::channel::<TcpStream>();
    for _ in 0..4 {
        let rx = rx.clone();
        thread::spawn(move || {
            for s in rx {
                handle(s);
            }
        });
    }
    for conn in listener.incoming() {
        tx.send(conn?).unwrap();
    }
    Ok(())
}
```

---

## 39) Macros, Attributes, and `cfg`

**Macros.** Use `macro_rules!` for small utilities; isolate complexity in proc‑macros.
**Attributes.** `#[derive]`, `#[inline]`, `#[must_use]`, `#[repr(C)]`, `#[non_exhaustive]`.
**`cfg`.** Gate code by platform/feature; `cfg_attr` to apply attributes conditionally.

### Examples
```rust
macro_rules! vec_of_strings {
    ( $( $x:expr ),* $(,)? ) => {
        vec![ $( $x.to_string() ),* ]
    };
}

let v = vec_of_strings!["a", "b", "c"];
```
```rust
#[deprecated(note = "use `new_api` instead")]
fn old_api() {
    println!("deprecated");
}
```
```rust
#[cfg_attr(debug_assertions, derive(Debug))]
struct Foo;
```

---

## 40) Tooling for Quality (fmt, clippy, miri, coverage)

**Policy.** Keep lints strict; run UB checks in unsafe code; track coverage and benchmarks.
**Docs.** Treat `rustdoc` as user docs; include runnable examples.

### Examples
```rust
#![forbid(unsafe_code)]
#![deny(unused_must_use)]
#![warn(missing_docs)]
```
```text
cargo +nightly miri test   # run UB checker on tests that exercise unsafe code
```

---

## Appendix: Quick Reference & Glossary

**Keywords.** `fn`, `let`, `struct`, `enum`, `impl`, `trait`, `async`, `await`, `unsafe`, `mod`, `use`, `pub`, `match`, `move`, `const`, `static`, `type`.
**Symbols.** `?`, `::`, `..`, `..=`, `@`, `_`, `ref`, `mut`, `&`, `&mut`.
**Derives.** `Debug`, `Clone`, `Copy`, `Eq`, `PartialEq`, `Ord`, `PartialOrd`, `Hash`, `Default`.
**Auto traits.** `Send`, `Sync`, `Unpin`.
**Crates to know.** `serde`, `anyhow/thiserror`, `tokio`, `tracing`, `reqwest`, `regex`, `rayon`, `parking_lot`, `cxx`, `bindgen`.
**Glossary.** Ownership, borrow, lifetime, trait, blanket impl, orphan rule, extension trait, newtype, object safety, pinning, interior mutability, FFI, DST, variance, ZST.

---

*Edition: Rust 2021+ idioms. You may adapt this for personal or internal use.*
