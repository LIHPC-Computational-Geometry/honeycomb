# Change Log

## To be released

### Repository Changes

...

### New Features

#### honeycomb-core

- `Orbit<'a, const N_MARKS: usize, T: CoordsFloat>` - Generic implementation for
  2D orbit computations. The structure itself only contains meta-data, the orbit
  computation is done through the `Iterator` trait implementation.
- `OrbitPolicy<'a>` - Enum used to specify the beta functions used by an orbit.
  It currently does not not support compositions.
- New (temporary?) method for `TwoMap`: `beta_bis`. It works exactly like the
  regular `beta` method except the beta index is passed as an argument instead
  of a `const` generic.

## 0.1.2

### Repository Changes

- remove `Cargo.lock` file from the repository
- add changelog file (this one!)

### New Features

#### honeycomb-core

- `Coords2<T: CoordsFloat>` - Custom 2D coordinates representation using a
  generic type for inner value.
- refactor two attributes of `TwoMap`:
    - `free_darts: Vec<DartIdentifier>`: rename to `unused_darts` & change type to `BTreeSet`
    - `free_vertices: Vec<VertexIdentifier>`: rename to `unused_vertices` & change type to `BTreeSet`

#### honeycomb-guide

- update usage instructions

#### honeycomb-utils

- update content according to features introduced in #15, #16

## 0.1.1

### Repository Changes

- new project members: `honeycomb-utils` (#10)
- new CI workflow:
    - `bench`: run hardware-counter based benchmarks on new version release

### New Features

#### honeycomb-core

- introduce `benchmarking_utils` feature, used to compile additional methods &
  trait implementation useful for benchmarking

#### honeycomb-guide

- update content (#5, #12)

#### honeycomb-utils

- benchmarks for the `TwoMap` structure and methods (#10, #11)
- utility functions for benchmarking and testing.
- example of memory usage information for a given `TwoMap`

### Fixes

- typos by @cedricchevalier19 (#9)

## 0.1.0

### Repository Changes

- new project members: `honeycomb-core`, `honeycomb-guide`
- new CI workflows:
    - `doc`: build & deploy user guide and code documentation
    - `rust-test`: run Rust tests, format checker & linter (`clippy`)

### New Features

#### honeycomb-core

- `TwoMap` - basic 2D combinatorial map implementation
- full documentation support & deployment

#### honeycomb-guide

- **mdbook**-based user guide with information regarding usage &
  non-code-related aspects of the project.
