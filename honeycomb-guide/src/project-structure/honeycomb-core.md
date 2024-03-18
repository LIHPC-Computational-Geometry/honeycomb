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

- [Coords2](../honeycomb_core/struct.Coords2.html#example)
- [TwoMap](../honeycomb_core/twomap/struct.TwoMap.html#example)

## Content

### Structures

- **TwoMap**: 2D combinatorial map implementation
- **Coords2**: 2D coordinates implementation
- **Orbit**: Generic 2D implementation for orbit computation

### Aliases

- **Vertex2**: Coords2 alias
- **DartIdentifier**: Integer identifier for darts
- **VertexIdentifier**: Integer identifier for 0D cells
- **FaceIdentifier**: Integer identifier for 2D cells
- **VolumeIdentifier**: Integer identifier for 3D cells

### Enums

- **OrbitPolicy**: Orbit parameterization.
- **SewPolicy**: Logic to follow for the geometrical part of the sewing operation.
- **UnsewPolicy**: Logic to follow for the geometrical part of the unsewing operation.

### Traits

- **CoordsFloat**: Common trait implemented by types used for coordinate representation.