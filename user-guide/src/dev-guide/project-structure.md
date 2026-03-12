# Project structure

**This content has been copy-pasted from the previous guide. It is up-to-date but should be improved
at some point.**

---

The project root is organized using Cargo workspaces at the moment. This may change when other languages are
introduced to the project.

The [repository][GH] hosts both published crates (usable content) as well as complementary content such as benchmarks,
examples or this guide.

[GH]: https://github.com/LIHPC-Computational-Geometry/honeycomb

The following libraries are available:

- [honeycomb](../../honeycomb/index.html) *Main crate, which re-exports items from the three subcrates below*
- [honeycomb-core](../../honeycomb_core/index.html) *Core definitions and tools for combinatorial map implementation*
- [honeycomb-kernels](../../honeycomb_kernels/index.html) *Meshing kernel implementations using combinatorial maps*
- [honeycomb-render](../../honeycomb_render/index.html) *Visualization tool for combinatorial maps*

The repository also hosts:

- The `applications` crate, which contains a collection of algorithms which are used as benchmarks
  and/or examples
- This book's source files, available in the `user-guide` directory
