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

- [CMap2](../honeycomb_core/struct.CMap2.html#example)
- [Vector2](../honeycomb_core/struct.Vector2.html#example)
- [Vertex2](../honeycomb_core/struct.Vertex2.html#example)

## Content

### Features

Refer to the Rust documentation.

### Structures

- **CMap2**: 2D combinatorial map implementation
- **Coords2**: 2D coordinates implementation; *Not meant to be used directly*
- **Orbit2**: Generic 2D implementation for orbit computation
- **Vector2**: 2D vector representation
- **Vertex2**: 2D vertex representation

### Aliases

- **DartIdentifier**: Integer identifier for darts
- **VertexIdentifier**: Integer identifier for 0D cells
- **EdgeIdentifier**: Integer identifier for 1D cells
- **FaceIdentifier**: Integer identifier for 2D cells
- **VolumeIdentifier**: Integer identifier for 3D cells

### Enums

- **OrbitPolicy**: Orbit parameterization.
- **SewPolicy**: Logic to follow for the geometrical part of the sewing operation.
- **UnsewPolicy**: Logic to follow for the geometrical part of the unsewing operation.

### Traits

- **CoordsFloat**: Common trait implemented by types used for coordinate representation.