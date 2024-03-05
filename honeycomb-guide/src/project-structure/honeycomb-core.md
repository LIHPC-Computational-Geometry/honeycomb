# honeycomb-core

[Documentation](../honeycomb_core/)

--- 

**honeycomb-core** is a Rust crate that provides basic structures and 
operations for combinatorial map manipulation. This includes map structures, 
methods implementation, type aliases and geometric modeling for mesh
representation.

## Usage

A quickstart example encompassing most features is provided for the following 
structures:

- [TwoMap](../honeycomb_core/twomap/struct.TwoMap.html#example)

## Content

### Structures

- **TwoMap**: 2D combinatorial map implementation

### Aliases

- **Vertex2**: 2-elements array
- **DartIdentifier**: Integer identifier for darts
- **VertexIdentifier**: Integer identifier for 0D cells
- **FaceIdentifier**: Integer identifier for 2D cells
- **VolumeIdentifier**: Integer identifier for 3D cells

### Enums

- **SewPolicy**: Logic to follow for the geometrical part of the sewing operation.
- **UnsewPolicy**: Logic to follow for the geometrical part of the unsewing operation.

## Future additions

- [x] Write structure benchmarks (2D) - done as of **0.1.1**
- [ ] Add a custom vector type for spatial representation (2D & 3D)
- [ ] Replace returned `Vec` by an alternative structure or type
  to prevent too many runtime allocations.
- [ ] Add I/O support for mesh formats (2D)
- [ ] Add orientation checks (2D)
- [ ] Implement 3D maps