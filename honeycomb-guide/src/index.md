# The Honeycomb User Guide

## Honeycomb

Honeycomb aims to provide a safe, efficient and scalable implementation of
combinatorial maps for meshing applications. More specifically, the goal is
to converge towards a (or multiple) structure(s) adapted to algorithms
exploiting GPUs and many-core architectures.

The current objective is to write a first implementation in Rust, to then
improve the structure without having to deal with data races and similar
issues, thanks to the language's guarantees.

### Core Requirements

- **Rust stable release** - *Development started on 1.75, but we might use
  newer features as the project progresses*

### Quickstart

#### Rust

The crate is not currently being published on crates.io, meaning you will have
to add the dependency manually to your project. This can be done by adding the
following line to the manifest of the project:

```toml
# Cargo.toml
# ...

[dependencies]
# Other dependencies...
honeycomb-core = { git = "https://github.com/LIHPC-Computational-Geometry/honeycomb.git" }
```

Optionally, you can add other member(s) of the workspace and specify which version to use.

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
- [honeycomb-render](honeycomb_render/) *Visualization tool*
- [honeycomb-utils](honeycomb_utils/) *Utility routines*

### References

## Contributing

## License