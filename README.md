# Honeycomb

Honeycomb aims to provide a safe, efficient and scalable implementation of
combinatorial maps for meshing applications. More specifically, the goal is
to converge towards a (or multiple) structure(s) adapted to algorithms
exploiting GPU and many-core architectures.

The current objective is to ~write a first implementation in Rust, to then~
improve the structure without having to deal with data races and similar
issues, thanks to the Rust's guarantees.

# Usage

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

## License
