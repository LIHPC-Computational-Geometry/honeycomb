# Honeycomb

[![Current Version](https://img.shields.io/crates/v/honeycomb-render?label=latest%20release)][CIOHC]
[![GitHub commits since latest release](https://img.shields.io/github/commits-since/LIHPC-Computational-Geometry/honeycomb/latest)][GH]
[![Build Status](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build.yml)
[![Rust Tests](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml)
[![codecov](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb/graph/badge.svg?token=QSN0TWFXO1)](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb)

Honeycomb aims to provide a safe, efficient and scalable implementation of
combinatorial maps for meshing applications. More specifically, the goal is
to converge towards a (or multiple) structure(s) adapted to algorithms
exploiting GPU and many-core architectures.

The current objective is to implement basic meshing algorithms to evaluate the viability of the implementation & improve
our structure using Rust's framework to streamline the refactoring and parallelization process.

## Usage

The [user guide][UG] provides an overview of everything available in the
project as well as usage instructions. It can be generated offline using
**mdbook**:

```shell
# Serve the doc on a local server
mdbook serve --open -d ../target/doc/ user-guide/ &
cargo doc --all --no-deps
```

```shell
# Kill the local server
kill $(pidof mdbook) 

# Without pidof
kill $(ps -e | awk '/mdbook/ {print $1}')
```

[UG]: https://lihpc-computational-geometry.github.io/honeycomb/

### Rust

The content of each member is described in their respective Rust Doc as well as in the [user guide][UGW]. The following
crates are published:

- [![Core Version](https://img.shields.io/crates/v/honeycomb-core?label=honeycomb-core)][CIOHC] [![docs.rs](https://docs.rs/honeycomb-core/badge.svg)][DOCHC]
  basic structures
- [![Render Version](https://img.shields.io/crates/v/honeycomb-render?label=honeycomb-render)][CIOHR] [![docs.rs](https://docs.rs/honeycomb-render/badge.svg)][DOCHR]
  visualizing tool

The repository also hosts these members:

- Benchmarks are grouped in the **honeycomb-benches** crate ([Rust Doc][DOCHB])
- Examples are grouped in the **honeycomb-examples** crate ([Rust Doc][DOCHU])

[UGW]: https://lihpc-computational-geometry.github.io/honeycomb/project-structure/workspace.html

[DOCHC]: https://docs.rs/honeycomb-core/

[CIOHC]:https://crates.io/crates/honeycomb-core

[DOCHR]: https://docs.rs/honeycomb-render/

[CIOHR]:https://crates.io/crates/honeycomb-render

[DOCHB]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb_benches/

[DOCHU]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb_utils/

## Contributing

Contributions are welcome and accepted as pull requests on [GitHub][GH]. Feel free to use issues to report bugs,
missing documentation or suggest improvements of the project.

Note that a most of the code possess documentation, including private modules / items / sections. You can generate
the complete documentation by using the instructions [above](#usage) and passing the option `--document-private-items`
to `cargo doc`.

[GH]: https://github.com/LIHPC-Computational-Geometry/honeycomb

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your preference.

The [SPDX](https://spdx.dev) license identifier for this project is `MIT OR Apache-2.0`.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.