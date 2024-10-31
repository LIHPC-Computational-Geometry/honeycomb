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
# [dependencies]
honeycomb = { git = "https://github.com/LIHPC-Computational-Geometry/honeycomb" }
```

---

## honeycomb-core

**honeycomb-core** is a Rust crate that provides basic structures and operations for combinatorial map manipulation.
This includes map structures, methods implementation, type aliases and geometric modeling for mesh representation.

### Implemented data structures

TODO

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

### Implemented algorithms

- [Directional Vertex Relaxation](../kernels/dvr.md)
- [Grisubal](../kernels/grisubal.md)
- [Polygon triangulations](../kernels/triangulations.md)
- [Cell splits](../kernels/splits.md)


---

## honeycomb-render

**honeycomb-render** is a Rust crate that provides a simple visualization framework to allow the user to render their
combinatorial map. It is designed to be used directly in the code by reading data through a reference to the map (as
opposed to a binary that would read serialized data). This render tool can be used to debug algorithm results in a
significantly easier way than reading internal data would.

### Usage

Use the [App](../honeycomb_render/struct.App.html) structure to render a given combinatorial map. **You may need to run
the program in `release` mode to render large maps**. All items used to build that tool are kept public to allow users
to customize the render logic (e.g. to render a specific attribute).

