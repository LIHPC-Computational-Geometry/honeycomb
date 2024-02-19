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

- **Vertex2**: 2-elements vector; This should be replaced by a custom struct

- **DartIdentifier**: Integer identifier for darts
- **VertexIdentifier**: Integer identifier for 0D cells
- **FaceIdentifier**: Integer identifier for 2D cells
- **VolumeIdentifier**: Integer identifier for 3D cells

### Enums

- **SewPolicy**: Logic to follow for the geometrical part of the sewing operation.
- **UnsewPolicy**: Logic to follow for the geometrical part of the unsewing operation.