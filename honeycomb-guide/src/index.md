# The Honeycomb User Guide

[![Current Version](https://img.shields.io/crates/v/honeycomb-render?label=latest%20release)](https://crates.io/crates/honeycomb-core)
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/LIHPC-Computational-Geometry/honeycomb/latest)][GH]
[![Build Status](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build.yml)
[![Rust Tests](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml)
[![codecov](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb/graph/badge.svg?token=QSN0TWFXO1)](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb)

---

## Honeycomb

Honeycomb aims to provide a safe, efficient and scalable implementation of combinatorial maps for meshing applications.
More specifically, the goal is to converge towards a (or multiple) structure(s) adapted to algorithms exploiting GPUs
and many-core architectures.

The current objective is to ~write a first implementation in Rust, to then~ improve the structure without having to
deal with data races and similar issues, thanks to the language's guarantees.

### Core Requirements

- **Rust stable release** - *Development started on 1.75, but we might use newer features as the project progresses*

### Quickstart

#### Rust

The core and render crates are being published on crates.io. You can add those to your project by adding the following
lines to the manifest of the project:

```toml
# Cargo.toml
# ...

[dependencies]
# Other dependencies...
honeycomb-core = "0.2.0"
honeycomb-render = "0.2.0"
```

Note that if you want to access the latest changes and documentation, you may have to specify a commit instead of a
version, and use the GitHub Pages documentation instead of the one hosted on docs.rs.

#### Documentation

You can generate this documentation locally using **mdbook** and **cargo doc**:

```shell
# Serve the doc on a local server
mdbook serve --open -d ../target/doc/ honeycomb-guide/ &
cargo doc --all --no-deps
```

## Links

### Documentation

- [honeycomb-core](honeycomb_core/) *Core definitions and tools*
- [honeycomb-benches](honeycomb_benches/) *Rust code benchmarks*
- [honeycomb-examples](honeycomb_examples/) *Rust code examples*
- [honeycomb-render](honeycomb_render/) *Visualization tool*

### References

## Contributing

Contributions are welcome and accepted as pull requests on [GitHub][GH]. Feel free to use issues to report bugs,
missing documentation or suggest improvements of the project.

Note that a most of the code possess documentation, including private modules / items / sections. You can generate the
complete documentation by using the instructions [above](#Documentation) and passing the option
`--document-private-items` to `cargo doc`.

[GH]: https://github.com/LIHPC-Computational-Geometry/honeycomb

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

- Damiand, Guillaume, and Pascal Lienhardt. *Combinatorial Maps: Efficient Data Structures for Computer Graphics and
  Image Processing*. Chapman&Hall/CRC, 2015.
    - Provides an in-depth presentation of the structure and its variants
    - [Link](https://hal.science/hal-01090890v1)
- The CGAL Project. *CGAL User and Reference Manual*. CGAL Editorial Board, 5.6.1 edition, 2024.
    - Provides concrete examples as well as code snippets of the CGAL implementation of the structure. The CGAL
      implementation uses a different approach than ours, & support N-dimensionnal map.
    - [Link](https://doc.cgal.org/latest/Combinatorial_map/)

### Integration

- The repository structure and workspace system is heavily inspired by
  the [wgpu repository](https://github.com/gfx-rs/wgpu)
