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
to clone this repository and add the dependency using the path to the project.

```shell
# Example: Setting up a new project using honeycomb-core

# Create your project using cargo
cargo new --bin my_project

# Clone the honeycomb repository
git clone https://github.com/LIHPC-Computational-Geometry/honeycomb.git

# Add the dependency to your project
# Version is set to whatever the current version is in the repository
cd my_project
cargo add honeycomb-core --path=../honeycomb/honeycomb-core/
```

The following lines should have appeared in the manifest of your project:

```toml
# Cargo.toml
# ...

[dependencies]
honeycomb-core = { path = "../honeycomb/honeycomb-core/" }
```

You can also copy-paste the above directly to your manifest. From there, you can
manually add whichever features your application requires. 


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
- [honeycomb-utils](honeycomb_utils/) *Utility routines*

### References

## Contributing

## License