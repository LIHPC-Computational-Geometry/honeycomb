# Honeycomb

Honeycomb aims to provide a safe, efficient and scalable implementation of 
combinatorial maps for meshing applications. More specifically, the goal is
to converge towards a (or multiple) structure(s) adapted to algorithms 
exploiting GPUs and many-core architectures.

The current objective is to write a first implementation in Rust, to then 
improve the structure without having to deal with data races and similar 
issues, thanks to the language's guarantees.

## Usage

The [user guide][UG] provides an overview of everything available in the 
project. It can be generated offline using **mdbook**: 

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

Basic structure are provided in the **honeycomb-core** crate. Its content is listed
in the [user guide][UGHC] and details about its usage can be found in its [Rust documentation][DOCHC]

[UGHC]: https://lihpc-computational-geometry.github.io/honeycomb/project-structure/honeycomb-core.html
[DOCHC]: https://lihpc-computational-geometry.github.io/honeycomb/honeycomb_core/

## Contributing

## License
