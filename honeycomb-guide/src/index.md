# The Honeycomb User Guide

## Honeycomb

Honeycomb aims to provide a safe, efficient and scalable implementation of 
combinatorial maps for meshing applications. More specifically, the goal is
to converge towards a (or multiple) structure(s) adapted to algorithms 
exploiting GPUs and many-core architectures.

The current objective is to write a first implementation in Rust, to then 
improve the structure without having to deal with data races and similar 
issues, thanks to the language's guarantees.

### Requirements

- **Rust 1.76** - *The code may compile and work for earlier versions, but we do not test for those*

### Quickstart

#### Rust

...

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

### References

## Contributing

## License