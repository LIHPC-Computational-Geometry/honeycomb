# The Honeycomb Book

[![Current Version](https://img.shields.io/crates/v/honeycomb-render?label=latest%20release)](https://crates.io/crates/honeycomb-core)
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/LIHPC-Computational-Geometry/honeycomb/latest)][GH]
[![Build Status](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build_nix.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build_nix.yml)
[![Rust Tests](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/test.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/test.yml)
[![codecov](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb/graph/badge.svg?token=QSN0TWFXO1)](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb)

[GH]: https://github.com/LIHPC-Computational-Geometry/honeycomb

---

## Statement of need

Honeycomb aims to provide a safe, efficient and scalable implementation of combinatorial maps
for meshing applications.

The goal is to converge towards a (or multiple) structure(s) which could be used to easily
experiment with parallel meshing algorithm, specifically targeting many-core architectures.

More extensive explanations regarding our needs and design choices of this solution is included in
[one of our paper](https://drive.google.com/file/d/1D_SLFSMMlBc2ycZURztTXnwX6jN0A8r8/view).

## Requirements

### Core

- **Rust stable release**
  - The MSRV may not be the latest stable release, but we do not give any guarantees for older
    versions compatibility

### Optional

- **CUDA**
  - Used in one of the application, as a PoC for GPU usage
  - `nvcc` and `libcudart` may be sufficient, but we recommend installing the full toolkit
- **hwloc**
  - The library is used by the application binaries to bind threads to physical cores;
  - Consider this a core requirement if you are running benchmarks
  - Disable usage by compiling application binaries with the `--no-default-features` option

## Quickstart

You can add `honeycomb` as a dependency of your project by adding the following lines
to its `Cargo.toml`:

```toml
# [dependencies]
honeycomb = {
  git = "https://github.com/LIHPC-Computational-Geometry/honeycomb",
  tag = "0.10.2" # it is highly encouraged to pin version using a tag or a revision
}
```

Alternatively, you can add the sub-crates that are currently published on crates.io:

```toml
# [dependencies]
honeycomb-core    = "0.10.2"
honeycomb-kernels = "0.10.2"
honeycomb-render  = "0.10.2"
```

Note that the documentation hosted on GitHub corresponds to the master branch.
Versioned documentation is available on docs.rs.

