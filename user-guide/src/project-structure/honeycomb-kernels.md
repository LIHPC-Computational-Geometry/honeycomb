# honeycomb-kernels

[Documentation](../honeycomb_kernels/)

---

**honeycomb-kernels** is a Rust crate that provides implementations of meshing kernels using the core crate's
combinatorial maps. These implementations have multiple purposes:

1. Writing code using n-maps from a user's perspective
2. Covering a wide range of operations, with routines that are more topology-heavy / geometry-heavy / balanced
3. Stressing the data structure, to identify its advantages and its pitfalls in a meshing context
4. Testing for more unwanted behaviors / bugs

Explanations provided on this page focus on the overall workflow of algorithms; Implementation-specific details and
hypothesis are listed in the documentation of the crate.

## Algorithms

### Grisubal

**Grisubal**, short for **GRId SUBmersion ALgorithm**, is a mesh generation algorithm inspired by [Morph][IMR-RN].
The mesh is built by capturing the input geometry in an overlapping grid, by first computing intersection vertices and
then rebuild new edges from the captured vertices. This is explained in more details [here](../kernels/grisubal.md).

[IMR-RN]: https://internationalmeshingroundtable.com/assets/research-notes/imr32/2011.pdf

### Examples

Using these algorihtms can be done with a one-shot call to their respective function, so there are no examples beside
the ones provided in the crate documentation.