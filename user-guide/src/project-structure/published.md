# Published crates

---

Several crates of this project are published on the registry _crates.io_: the main crate, **honeycomb** (not yet 
published), as well as specialized crates **honeycomb-core**, **honeycomb-kernels**, and **honeycomb-render**.

---

## honeycomb

**honeycomb** is the main crate provided to user and serve as the entrypoint for combinatorial map usage. It is
exclusively made up of re-exports from the core, kernels and render crate to provide a clean, all-in-one dependency.

### Usage

At the moment, the `honeycomb` name is not available on crates.io; this means that using this crate requires adding
the dependency using the git repository:

```toml
# Cargo.toml

[dependencies]
honeycomb = { git = "https://github.com/LIHPC-Computational-Geometry/honeycomb" }
```

The documentation is available [here](../honeycomb/).

---

## honeycomb-core

**honeycomb-core** is a Rust crate that provides basic structures and operations for combinatorial map manipulation.
This includes map structures, methods implementation, type aliases and geometric modeling for mesh representation.

### Usage 

A quickstart example encompassing most features is provided for the following structures:

- [CMap2](../honeycomb_core/struct.CMap2.html)
- [Vector2](../honeycomb_core/struct.Vector2.html)
- [Vertex2](../honeycomb_core/struct.Vertex2.html)
- [CMapBuilder](../honeycomb_core/cmap/struct.CMapBuilder.html)

Optional features can be enabled when using this crate:

- `utils` -- provides additionnal implementations for map generation, benchmarking & debugging
- `io` -- provides a function and a method for building maps from VTK meshes and vice versa

Both of these are enabled by default in the **honeycomb** crate.

---

## honeycomb-kernels

**honeycomb-kernels** is a Rust crate that provides implementations of meshing kernels using the core crate's
combinatorial maps. These implementations have multiple purposes:

1. Writing code using n-maps from a user's perspective
2. Covering a wide range of operations, with routines that are more topology-heavy / geometry-heavy / balanced
3. Stressing the data structure, to identify its advantages and its pitfalls in a meshing context
4. Testing for more unwanted behaviors / bugs

Explanations provided on this page focus on the overall workflow of algorithms; Implementation-specific details and
hypothesis are listed in the documentation of the crate.

### Usage

The documentation is available [here](../honeycomb_kernels/).

### Algorithms

**Grisubal**, short for **GRId SUBmersion ALgorithm**, is a mesh generation algorithm inspired by [Morph][IMR-RN].
The mesh is built by capturing the input geometry in an overlapping grid, by first computing intersection vertices and
then rebuild new edges from the captured vertices. This is explained in more details [here](../kernels/grisubal.md).

[IMR-RN]: https://internationalmeshingroundtable.com/assets/research-notes/imr32/2011.pdf

---

## honeycomb-render

**honeycomb-render** is a Rust crate that provides a simple visualization framework to allow the user to render their
combinatorial map. It is designed to be used directly in the code by reading data through a reference to the map (as
opposed to a binary that would read serialized data). This render tool can be used to debug algorithm results in a
significantly easier way than reading internal data would.

### Usage

Use the [App](../honeycomb_render/struct.App.html) structure to render a given combinatorial map. You may need to run
the program in `release` mode to render large maps. All items used to build that tool are kept public to allow users
to customize the render logic (e.g. to render a specific attribute).

The documentation is available [here](../honeycomb_render/).
