# Visualiser Module

Write mesh data to ParaView-compatible VTK files.

## Quick Start

```rust
use strelitzia::visualiser::{write_vtu, CellType, Encoding};

let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
let connectivity = vec![vec![0, 1, 2]];
let cell_types = vec![CellType::Triangle];

write_vtu::<_, 3>("mesh.vtu", &points, Some(&connectivity), Some(&cell_types), &[], &[], Encoding::Base64)?;
```

Open `mesh.vtu` in [ParaView](https://www.paraview.org/).

## Supported Cell Types

| Type | Description |
|------|-------------|
| `Vertex` | Point |
| `Edge` | Line segment |
| `EdgeChain` | Polyline |
| `Triangle` | Triangle |
| `Polygon` | N-sided polygon |
| `Quad` | Quadrilateral |
| `Tetra` | Tetrahedron |
| `Hexa` | Hexahedron |

## Usage

### Point Cloud

No connectivity needed - auto-generates VTK_VERTEX cells:

```rust
let points: Vec<[f64; 3]> = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
write_vtu::<_, 3>("cloud.vtu", &points, None, None, &[], &[], Encoding::Base64)?;
```

### 2D Mesh

2D points automatically padded to 3D:

```rust
let points: Vec<[f64; 2]> = vec![[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
let connectivity = vec![vec![0, 1, 2, 3]];
let cell_types = vec![CellType::Quad];
write_vtu::<_, 2>("quad.vtu", &points, Some(&connectivity), Some(&cell_types), &[], &[], Encoding::Base64)?;
```

### Mixed Cell Types

```rust
let connectivity = vec![vec![0, 1, 2], vec![1, 3, 4, 2]];
let cell_types = vec![CellType::Triangle, CellType::Quad];
write_vtu::<_, 3>("mixed.vtu", &points, Some(&connectivity), Some(&cell_types), &[], &[], Encoding::Base64)?;
```

### Field Data

```rust
use strelitzia::visualiser::FieldArray;

// Scalar field (1 component)
let temp: Vec<f64> = vec![0.0, 100.0, 200.0];
let temp_field = FieldArray::from_slice("temperature", &temp, 1);

// Vector field (3 components)
let vel: Vec<[f64; 3]> = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
let vel_field = FieldArray::from_slice("velocity", &vel, 3);

write_vtu::<_, 3>("mesh.vtu", &points, Some(&conn), Some(&types), &[temp_field, vel_field], &[], Encoding::Base64)?;
```

## Encoding

- `Encoding::Ascii`: Human-readable (~40% larger)
- `Encoding::Base64`: Binary (recommended)

## Notes

- 2D points automatically padded to 3D (VTK requirement)
- Point clouds auto-generate VTK_VERTEX cells
- Uses `bytemuck::Pod` for zero-cost type conversions
- See `examples/paraview_demo.rs` for more examples

