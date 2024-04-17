# Honeycomb

[![Rust Tests](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/rust-test.yml)
[![Documentation](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/doc.yml/badge.svg)][UG]
[![Build Status](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build.yml/badge.svg)](https://github.com/LIHPC-Computational-Geometry/honeycomb/actions/workflows/build.yml)
[![codecov](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb/graph/badge.svg?token=QSN0TWFXO1)](https://codecov.io/github/LIHPC-Computational-Geometry/honeycomb)

Honeycomb aims to provide a safe, efficient and scalable implementation of
combinatorial maps for meshing applications. More specifically, the goal is
to converge towards a (or multiple) structure(s) adapted to algorithms
exploiting GPU and many-core architectures.

The current objective is to ~write a first implementation in Rust, to then~
improve the structure without having to deal with data races and similar
issues, thanks to the Rust's guarantees.

## Usage

The [user guide][UG] provides an overview of everything available in the
project as well as usage instructions. It can be generated offline using
**mdbook**:

```shell
# Serve the doc on a local server
mdbook serve --open -d ../target/doc/ honeycomb-guide/ &
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

The content of each member is described in their respective Rust Doc as well as in the [user guide][UGW].

- Basic structures are provided in the **honeycomb-core** crate ([Rust Doc][DOCHC]).
- Benchmarks are grouped in the **honeycomb-benches** crate ([Rust Doc][DOCHB])
- Examples are grouped in the **honeycomb-examples** crate ([Rust Doc][DOCHU])
- A visualing tool is provided in the **honeycomb-render** crate ([Rust Doc][DOCHR]).

[UGW]: https://lihpc-computational-geometry.github.io/honeycomb/project-structure/workspace.html

[DOCHC]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb_core/

[DOCHB]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb_benches/

[DOCHU]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb_utils/

[DOCHR]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb_render/

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