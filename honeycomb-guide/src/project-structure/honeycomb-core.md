# honeycomb-core

[Documentation](../honeycomb_core/)

--- 

**honeycomb-core** is a Rust crate that provides basic structures and operations for combinatorial map manipulation.
This includes map structures, methods implementation, type aliases and geometric modeling for mesh representation.

## Usage

A quickstart example encompassing most features is provided for the following structures:

- [CMap2](../honeycomb_core/struct.CMap2.html#example)
- [Vector2](../honeycomb_core/struct.Vector2.html#example)
- [Vertex2](../honeycomb_core/struct.Vertex2.html#example)
- [GridBuilder](../honeycomb_core/utils/struct.GridBuilder.html#example)

## Content

The Rust documentation fully cover the API of the crate, the main items that you might be interested in are the
following:

- **CMap2**: 2D combinatorial map implementation
- **Orbit2**: Generic 2D implementation for orbit computation
- **Vector2**: 2D vector representation
- **Vertex2**: 2D vertex representation

Note that optional features can be enabled when compiling this crate:

- `utils` -- provides additionnal implementations for map generation, benchmarking & debugging