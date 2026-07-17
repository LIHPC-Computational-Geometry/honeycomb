# Project structure

---

The project root is organized using Cargo workspaces. The [repository][GH] hosts both published
crates (libraries) as well as complementary content such as benchmarks, examples or this guide.

[GH]: https://github.com/LIHPC-Computational-Geometry/honeycomb

The following libraries are available:

- [honeycomb](../../honeycomb/index.html) *Main crate, which re-exports items from the three subcrates below*
- [honeycomb-core](../../honeycomb_core/index.html) *Core definitions and tools for combinatorial map implementation*
- [honeycomb-kernels](../../honeycomb_kernels/index.html) *Meshing kernel implementations using combinatorial maps*
- [honeycomb-render](../../honeycomb_render/index.html) *Visualization tool for combinatorial maps*

The repository also hosts:

- The `applications` crate, which contains a collection of algorithms which are used as benchmarks
  and/or examples.
- This book's source files, available in the `user-guide` directory.
