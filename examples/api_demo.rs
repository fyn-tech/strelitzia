//! Full tour of the MultiArray and Field APIs.
//!
//! Run with: cargo run --example api_demo

use strelitzia::common::Real;
use strelitzia::multiarray::*;
use strelitzia::multiarray::linalg::*;
use strelitzia::fields::*;

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║               Strelitzia API Demo                    ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // ========================================================================
    // 1. VECTOR CREATION
    // ========================================================================
    println!("=== 1. Vector Creation ===\n");

    let v = Vector3::new(1.0, 2.0, 3.0);
    let w = Vector3::new(4.0, 5.0, 6.0);
    println!("  v          = ({}, {}, {})", v.x(), v.y(), v.z());
    println!("  w          = ({}, {}, {})", w.x(), w.y(), w.z());

    let origin = Vector3::zeros();
    println!("  zeros()    = ({}, {}, {})", origin.x(), origin.y(), origin.z());

    let from_data = Vector3::from_slice(&[7.0, 8.0, 9.0]);
    println!("  from_slice = ({}, {}, {})", from_data.x(), from_data.y(), from_data.z());

    let v2 = Vector2::new(1.0, 2.0);
    let v4 = Vector4::new(1.0, 2.0, 3.0, 4.0);
    println!("  Vector2    = ({}, {})", v2.x(), v2.y());
    println!("  Vector2    = ({}, {})", v2.x(), v2.y());
    println!("  Vector4    = ({}, {}, {}, {})", v4.x(), v4.y(), v4.z(), v4.w());

    // Point aliases -- same types, semantic naming
    let pos: Point3 = Vector3::new(10.0, 20.0, 30.0);
    println!("  Point3     = ({}, {}, {})", pos.x(), pos.y(), pos.z());

    // ========================================================================
    // 2. MATRIX CREATION
    // ========================================================================
    println!("\n=== 2. Matrix Creation ===\n");

    let identity = Matrix3::identity();
    println!("  identity =");
    for r in 0..3 {
        println!("    [{:.1}  {:.1}  {:.1}]", identity[r * 3], identity[r * 3 + 1], identity[r * 3 + 2]);
    }

    // Note: Matrix3::new takes row-major input (like writing on paper),
    // but nalgebra stores column-major internally.
    let m = Matrix3::new(
        1.0, 2.0, 3.0,
        4.0, 5.0, 6.0,
        7.0, 8.0, 9.0,
    );
    println!("\n  custom matrix (row-major input) =");
    // as_slice gives column-major layout
    let s = m.as_slice();
    println!("    as_slice (column-major): {:?}", s);
    println!("    nrows={}, ncols={}", m.nrows(), m.ncols());

    let z = Matrix3::zeros();
    println!("  zeros: all elements = {}", z.as_slice().iter().all(|&x| x == 0.0));

    // ========================================================================
    // 3. VECTOR ARITHMETIC (binary + compound assignment)
    // ========================================================================
    println!("\n=== 3. Vector Arithmetic ===\n");

    let sum = v + w;
    println!("  v + w      = ({}, {}, {})", sum.x(), sum.y(), sum.z());

    let diff = v - w;
    println!("  v - w      = ({}, {}, {})", diff.x(), diff.y(), diff.z());

    let neg = -v;
    println!("  -v         = ({}, {}, {})", neg.x(), neg.y(), neg.z());

    let scaled = 2.0 * v;                     // scalar * array (left-multiply)
    println!("  2.0 * v    = ({}, {}, {})", scaled.x(), scaled.y(), scaled.z());

    let halved = v / 2.0;                     // array / scalar
    println!("  v / 2.0    = ({}, {}, {})", halved.x(), halved.y(), halved.z());

    // Compound assignment (in-place)
    let mut acc = Vector3::zeros();
    acc += v;
    acc += w;
    println!("  0 += v += w = ({}, {}, {})", acc.x(), acc.y(), acc.z());

    acc *= 0.5;
    println!("  then *= 0.5 = ({}, {}, {})", acc.x(), acc.y(), acc.z());

    acc -= v;
    println!("  then -= v   = ({}, {}, {})", acc.x(), acc.y(), acc.z());

    // ========================================================================
    // 4. LINALG EXTENSION TRAITS
    // ========================================================================
    println!("\n=== 4. Linear Algebra (extension traits) ===\n");

    // VectorOps: dot, norm, normalised
    let dot = v.dot(&w);
    println!("  v . w         = {}", dot);

    let norm = v.norm();
    println!("  ||v||         = {:.6}", norm);

    let norm_sq = v.norm_squared();
    println!("  ||v||^2       = {}", norm_sq);

    let l1 = v.l1_norm();
    println!("  ||v||_1       = {}", l1);

    let linf = v.linf_norm();
    println!("  ||v||_inf     = {}", linf);

    let unit = v.normalised();
    println!("  normalised(v) = ({:.6}, {:.6}, {:.6}), norm = {:.6}",
        unit.x(), unit.y(), unit.z(), unit.norm());

    // CrossProduct
    let cross = v.cross(&w);
    println!("  v x w         = ({}, {}, {})", cross.x(), cross.y(), cross.z());

    // 2D cross product (returns a 3D vector along z)
    let a2 = Vector2::new(1.0, 0.0);
    let b2 = Vector2::new(0.0, 1.0);
    let cross2d = a2.cross(&b2);
    println!("  2D cross      = ({}, {}, {})", cross2d.x(), cross2d.y(), cross2d.z());

    // OuterProduct
    let u = Vector3::new(1.0, 0.0, 0.0);
    let outer = u.outer(&w);
    println!("  u outer w     = {:?}", outer.as_slice());

    // Hadamard (element-wise multiply)
    let h = v.hadamard(&w);
    println!("  v hadamard w  = ({}, {}, {})", h.x(), h.y(), h.z());

    // Transpose
    let row = v.transpose();
    println!("  v^T           = 1x3 matrix, {} elements", row.as_slice().len());

    let mt = m.transpose();
    println!("  M^T           = {:?}", mt.as_slice());

    // ========================================================================
    // 5. MATRIX ARITHMETIC + MATRIX MULTIPLICATION
    // ========================================================================
    println!("\n=== 5. Matrix Arithmetic & Multiplication ===\n");

    let m2 = 2.0 * identity;
    println!("  2 * I diagonal = [{}, {}, {}]", m2[0], m2[4], m2[8]);

    // Matrix * Vector
    let rotated = identity * v;
    println!("  I * v = ({}, {}, {})", rotated.x(), rotated.y(), rotated.z());

    // Matrix * Matrix
    let product = identity * m;
    println!("  I * M = M? {}", product == m);

    // ========================================================================
    // 6. ELEMENT ACCESS + TRAIT API
    // ========================================================================
    println!("\n=== 6. Element Access & Trait API ===\n");

    // Index access
    println!("  v[0]={}, v[1]={}, v[2]={}", v[0], v[1], v[2]);

    let mut editable = v;
    editable[1] = 99.0;
    println!("  after v[1]=99: ({}, {}, {})", editable.x(), editable.y(), editable.z());

    // MultiArrayOps trait
    println!("  v.len()    = {}", v.len());
    println!("  v.rank()   = {}", v.rank());
    println!("  m.len()    = {}", m.len());
    println!("  m.rank()   = {}", m.rank());

    // DenseMultiArrayOps trait
    println!("  v.as_slice() = {:?}", v.as_slice());
    println!("  m.as_slice() = {:?} (column-major)", m.as_slice());

    // ========================================================================
    // 7. GENERIC TYPES (non-Real scalars)
    // ========================================================================
    println!("\n=== 7. Generic Scalar Types ===\n");

    let vi = Vector::<f32, 3>::new(1.0_f32, 2.0, 3.0);
    println!("  Vector<f32, 3>  = ({}, {}, {})", vi.x(), vi.y(), vi.z());

    let big = Vector::<Real, 6>::zeros();
    println!("  Vector<Real, 6> = {:?}, dim={}", big.as_slice(), big.dim());

    let rect = Matrix::<Real, 2, 3>::zeros();
    println!("  Matrix<Real,2,3> = {:?}, {}x{}", rect.as_slice(), rect.nrows(), rect.ncols());

    // ========================================================================
    // 8. FIELD STORAGE
    // ========================================================================
    println!("\n=== 8. Field Storage ===\n");

    let mut positions = Vector3Field::new();
    positions.push(Vector3::new(0.0, 0.0, 0.0));
    positions.push(Vector3::new(1.0, 0.0, 0.0));
    positions.push(Vector3::new(0.5, 1.0, 0.0));
    println!("  positions.len() = {}", positions.len());

    // With capacity
    let mut temps = ScalarField::with_capacity(1000);
    for i in 0..5 {
        temps.push(20.0 + i as Real);
    }
    println!("  temps = {:?}", temps.as_slice());

    // Iteration
    let centroid: Vector3 = positions.iter().copied().sum::<Vector3>();
    let centroid = (1.0 / positions.len() as Real) * centroid;
    println!("  centroid = ({:.4}, {:.4}, {:.4})", centroid.x(), centroid.y(), centroid.z());

    // Mutable iteration
    for pos in positions.iter_mut() {
        *pos = *pos - centroid;    // centre around origin
    }
    println!("  after centring: p0 = ({:.4}, {:.4}, {:.4})",
        positions[0].x(), positions[0].y(), positions[0].z());

    // Index access
    positions[0] = Vector3::new(99.0, 0.0, 0.0);
    println!("  after positions[0] = (99,0,0): {}", positions[0].x());

    // Extend
    let extras = vec![Vector3::new(2.0, 2.0, 0.0), Vector3::new(3.0, 3.0, 0.0)];
    positions.extend(extras);
    println!("  after extend: len = {}", positions.len());

    // ========================================================================
    // 9. FIELD OPERATORS (compound-assignment only)
    // ========================================================================
    println!("\n=== 9. Field Operators ===\n");

    let mut a_field = ScalarField::new();
    let mut b_field = ScalarField::new();
    for i in 0..4 {
        a_field.push(i as Real);
        b_field.push(10.0 * (i + 1) as Real);
    }
    println!("  a = {:?}", a_field.as_slice());
    println!("  b = {:?}", b_field.as_slice());

    a_field += &b_field;      // element-wise addition
    println!("  a += &b  -> {:?}", a_field.as_slice());

    a_field -= &b_field;      // element-wise subtraction
    println!("  a -= &b  -> {:?}", a_field.as_slice());

    a_field *= 3.0;           // scalar broadcast multiply
    println!("  a *= 3.0 -> {:?}", a_field.as_slice());

    a_field /= 3.0;           // scalar broadcast divide
    println!("  a /= 3.0 -> {:?}", a_field.as_slice());

    a_field += 100.0;         // scalar broadcast add
    println!("  a += 100 -> {:?}", a_field.as_slice());

    a_field -= 100.0;         // scalar broadcast subtract
    println!("  a -= 100 -> {:?}", a_field.as_slice());

    // Also works on vector fields
    let mut vf = Vector3Field::new();
    vf.push(Vector3::new(1.0, 0.0, 0.0));
    vf.push(Vector3::new(0.0, 1.0, 0.0));
    vf *= 5.0;
    println!("  vector field *= 5: ({},{},{}), ({},{},{})",
        vf[0].x(), vf[0].y(), vf[0].z(),
        vf[1].x(), vf[1].y(), vf[1].z());

    // ========================================================================
    // 10. FIELD OPS TRAITS (fill, resize, reduce, sum)
    // ========================================================================
    println!("\n=== 10. FieldOps, ReductionOps, SumOps ===\n");

    let mut f = ScalarField::new();
    for x in [3.0, 1.0, 4.0, 1.0, 5.0, 9.0] {
        f.push(x);
    }
    println!("  field = {:?}", f.as_slice());
    println!("  max   = {:?}", f.max());
    println!("  min   = {:?}", f.min());
    println!("  sum   = {}", f.sum());

    f.fill(42.0);
    println!("  fill(42) -> {:?}", f.as_slice());

    f.resize(3, 0.0);
    println!("  resize(3) -> {:?}", f.as_slice());

    f.resize(6, -1.0);
    println!("  resize(6, -1) -> {:?}", f.as_slice());

    f.clear();
    println!("  clear() -> len={}", f.len());

    // ========================================================================
    // 11. SOLVER INTEROP (zero-copy flat slices)
    // ========================================================================
    println!("\n=== 11. SolverInterop (zero-copy) ===\n");

    let mut field = Vector3Field::new();
    field.push(Vector3::new(1.0, 2.0, 3.0));
    field.push(Vector3::new(4.0, 5.0, 6.0));

    // Read: flat view for passing to solver as right-hand side
    let flat: &[Real] = field.as_flat_slice();
    println!("  flat slice = {:?}", flat);
    println!("  (6 Reals for 2 Vector3s, zero-copy)");

    // Write: solver writes results back directly
    let flat_mut: &mut [Real] = field.as_flat_slice_mut();
    flat_mut[0] = 10.0;   // modify first component of first vector
    flat_mut[5] = 60.0;   // modify last component of second vector
    println!("  after solver write: v0=({},{},{}), v1=({},{},{})",
        field[0].x(), field[0].y(), field[0].z(),
        field[1].x(), field[1].y(), field[1].z());

    // Works for scalars too
    let mut sf = ScalarField::new();
    sf.push(1.0);
    sf.push(2.0);
    sf.push(3.0);
    let flat_scalar = sf.as_flat_slice();
    println!("  scalar flat = {:?}", flat_scalar);

    // And matrices (column-major layout)
    let mut mf = Matrix3Field::new();
    mf.push(Matrix3::identity());
    let flat_mat = mf.as_flat_slice();
    println!("  matrix flat = {:?} (9 Reals, column-major)", flat_mat);

    // ========================================================================
    // 12. PUTTING IT ALL TOGETHER: mini simulation step
    // ========================================================================
    println!("\n=== 12. Mini Simulation Step ===\n");

    // Set up particles
    let n = 4;
    let mut pos_field = Vector3Field::with_capacity(n);
    let mut vel_field = Vector3Field::with_capacity(n);
    let mut force_field = Vector3Field::with_capacity(n);

    for i in 0..n {
        pos_field.push(Vector3::new(i as Real, 0.0, 0.0));
        vel_field.push(Vector3::zeros());
        force_field.push(Vector3::new(0.0, -9.81, 0.0));   // gravity
    }
    println!("  Initial positions: {:?}",
        pos_field.iter().map(|p| (p.x(), p.y())).collect::<Vec<_>>());

    // Euler integration: v += dt * F,  x += dt * v
    let dt: Real = 0.1;
    for i in 0..n {
        vel_field[i] += dt * force_field[i];     // MultiArray compound assign
        pos_field[i] += dt * vel_field[i];        // scalar * MultiArray
    }

    println!("  After one step (dt={}):", dt);
    for i in 0..n {
        println!("    particle {}: pos=({:.4}, {:.4}), vel=({:.4}, {:.4})",
            i, pos_field[i].x(), pos_field[i].y(),
            vel_field[i].x(), vel_field[i].y());
    }

    // Compute kinetic energy using field iteration + linalg
    let ke: Real = vel_field.iter()
        .map(|v| 0.5 * v.norm_squared())
        .sum();
    println!("  Total kinetic energy = {:.6}", ke);

    println!("\n=== Demo Complete ===");
}
