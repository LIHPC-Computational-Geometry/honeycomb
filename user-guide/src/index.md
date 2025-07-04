# The Honeycomb User Guide

[![Current Version](https://img.shields.io/crates/v/honeycomb-render?label=latest%20release)](https://crates.io/crates/honeycomb-core)
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/LIHPC-Computational-Geometry/honeycomb/latest)][GH]
[![Build Status](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build_nix.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build_nix.yml)
[![Rust Tests](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml)
[![codecov](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb/graph/badge.svg?token=QSN0TWFXO1)](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb)

[GH]: https://github.com/LIHPC-Computational-Geometry/honeycomb

---

## Honeycomb

Honeycomb aims to provide a safe, efficient and scalable implementation of combinatorial maps for meshing applications.
More specifically, the goal is to converge towards a (or multiple) structure(s) adapted to algorithms exploiting GPUs
and many-core architectures.

The current objective is to

- ~write a first implementation in Rust~
- ~improve the structure without having to deal with data races and similar issues, thanks to the Rust's guarantees~
- ~implement basic meshing algorithms to evaluate the viability of the implementation & improve our structure using
  Rust's framework to streamline the refactoring and parallelization process~
- ~Benchmark and/or profile and/or parallelize our first algorithm, **grisubal**~
- Ship a first stable version of the library (see
  this [issue](https://github.com/LIHPC-Computational-Geometry/honeycomb/issues/150))
- Work on efficient parallelism

### Core Requirements

- **Rust stable release** - The MSRV may not be the latest stable release, but we do not give any guarantees for older
  versions compatibility
- `hwloc` - The library is used by the benchmark binary to bind threads to physical cores; you can disable its usage
  by compiling the binary without default features

### Quickstart

#### Rust

You can add `honeycomb` as a dependency of your project by adding the following lines to its `Cargo.toml`:

```toml
# [dependencies]
honeycomb = {
  git = "https://github.com/LIHPC-Computational-Geometry/honeycomb",
  tag = "0.9.0" # remove tag for master branch build
}
```

Alternatively, you can add the sub-crates that are currently published on crates.io:

```toml
# [dependencies]
honeycomb-core = "0.9.0"
honeycomb-kernels = "0.9.0"
honeycomb-render = "0.9.0"
```

Note that if you want to access the latest changes and documentation, you may have to specify a commit instead of a
version, and use the [GitHub Pages documentation][DOC] instead of the one hosted on docs.rs.

[DOC]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb/

## Documentation

You can generate this book and the Rust documentation locally using respectively **mdbook** and **cargo doc**:

```shell
mdbook serve --open user-guide/
```

```shell
cargo +nightly doc --all --all-features --no-deps
```

Note that generating the doc using a stable toolchain is possible, the features just won't be documented as clearly.


## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](https://github.com/LIHPC-Computational-Geometry/honeycomb/blob/master/LICENSE-APACHE)
  or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](https://github.com/LIHPC-Computational-Geometry/honeycomb/blob/master/LICENSE-MIT)
  or http://opensource.org/licenses/MIT)

at your preference.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## References

### Combinatorial Maps

- Damiand and Lienhardt. *Combinatorial Maps: Efficient Data Structures for Computer Graphics and
  Image Processing*. Chapman&Hall/CRC, 2015.
    - Provides an in-depth presentation of the structure and its variants
    - [Link](https://hal.science/hal-01090890v1)
- The CGAL Project. *CGAL User and Reference Manual*. CGAL Editorial Board, 5.6.1 edition, 2024.
    - Provides concrete examples as well as code snippets of the CGAL implementation of the structure. The CGAL
      implementation uses a different approach than ours, & support N-dimensional map.
    - [Link](https://doc.cgal.org/latest/Combinatorial_map/)

### Algorithms

- Staten, Noble, and Wilson. *Constructing Tetrahedral Meshes No Matter How Ugly*. SIAM, 2024
    - Describes the logic behind an overlay grid algorithm.
    - [Link](https://internationalmeshingroundtable.com/assets/research-notes/imr32/2011.pdf)
- Rangarajan and Lew. *Provably Robust Directional Vertex Relaxation for geometric mesh optimization*. SIAM, 2017
    - usage TBD
    - [Link](https://epubs.siam.org/doi/abs/10.1137/16M1089101)

### Integration

- The repository structure and workspace system is heavily inspired by
  the [wgpu](https://github.com/gfx-rs/wgpu) and [bevy](https://github.com/bevyengine/bevy) repositories.
