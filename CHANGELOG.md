# Change Log

---

## To be released

**This update contains breaking changes**

### New features

#### honeycomb-benches (new)

<sup>core structures & methods benchmarks</sup>

- move and update all benchmarks previously defined in `honeycomb-utils` to this crate (#31, #36)

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

- add two new structures for 2D spatial representation: `Vertex2` & `Vector2`,
  which act as wrappers around a `Coords2` value (#25)
- remove the `Vertex2` type alias in favor of the new structure (#25)
- add a new public module, `utils`, compiled when the `utils` feature is enabled (#31)
    - the module contains functions previously defined in the `honeycomb-utils` crate
- add two new traits, `AttributeLogic` and `AttributeSupport`, for basic attribute genericity (#33)
- implement new attribute traits for the `Vertex2` struct (#33)
- add two storage structures for generic attributes `AttributeSparseVec` and `AttributeSparseVec` (#34)
- add collection structures that can be used to retrieve all cells of a given dimension of a map (#36)

#### honeycomb-examples (new)

<sup>project examples</sup>

- move all examples previously defined in `honeycomb-utils` & `honeycomb-render` to this crate (#29)
- update examples
    - to reflect mark removal (#24)
    - to fix import path of utility functions (#31)
    - to fit new `CMap2` methods signature (#36)

#### honeycomb-guide

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project</sup>

- add pages for the new project members (#32)

### Refactor

#### honeycomb-core

- change visibility of all `honeycomb-core` modules to private & re-export types
  accordingly (#23)
- rename `TwoMap`/`Orbit` to `CMap2`/`Orbit2` for consistency (#23)
- remove marks table from `DartData` (#24), resulting in a signature change for many
  functions and structures (anything with `const N_MARKS: usize`) '
- update code to make use of the new 2D representation structures (#25)
- reorganize internal module structure (#27, #42)
    - create modules `cells`, `spatial_repr`, `attributes`
    - move `orbits`, `coords`, `vector`, `vertex`, inside new modules
    - clean-up source files
- rename the `benchmarking_utils` feature to `utils` (#31)
- rework the structure and interface of `CMap2` (#36)
    - implement the new cell id computation logic
    - replace the vertex storage with an `AttributeSparseVec`
- add support for incomplete vertex orbits in `Orbit2` implementation (#36)
- remove `darts`, `embed` modules and their content (#42)
    - move ID aliases to `cells` and `cells::collections` modules

#### honeycomb-guide

- update all references to renamed types (#23)
- remove `honeycomb-utils` page (#32)
- update index, summary, core & render pages (#32)

#### honeycomb-render

<sup>visualization tool for combinatorial maps</sup>

- update code:
    - to reflect mark removal (#24)
    - to make use of the new 2D representation structures (#25)
    - to fit new `CMap2` methods signature (#36)

#### honeycomb-utils (removed)

<sup>utility routines used in benchmarking and testing</sup>

- remove this crate in favor of:
    - a new member dedicated to benchmarks (#31)
    - a new member dedicated to examples (#29)
    - a new `utils` module in the core crate (#31)

---

## 0.1.3

### New Features

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

- `Orbit<'a, const N_MARKS: usize, T: CoordsFloat>` - Generic implementation for
  2D orbit computations. The structure itself only contains meta-data, the orbit
  computation is done through the `Iterator` trait implementation (#18)
- `OrbitPolicy<'a>` - Enum used to specify the beta functions used by an orbit.
  It currently does not support compositions (#18)
- New (temporary?) method for `TwoMap`: `beta_runtime`. It works by redirecting
  to the original `beta` method, using match block and a beta identifier provided
  at runtime (#18)

#### honeycomb-guide

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project</sup>

- update content of the workspace section to include new member (#19)
- update **honeycomb-core**'s page content (#19)

#### honeycomb-render (new member)

<sup>visualization tool for combinatorial maps</sup>

- implement `TwoMap` rendering code
- add examples illustrating basic usage

---

## 0.1.2

### Repository Changes

- remove `Cargo.lock` file from the repository
- add changelog file (this one!)

### New Features

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

- `Coords2<T: CoordsFloat>` - Custom 2D coordinates representation using a
  generic type for inner value.
- refactor two attributes of `TwoMap`:
    - `free_darts: Vec<DartIdentifier>`: rename to `unused_darts` & change type to `BTreeSet`
    - `free_vertices: Vec<VertexIdentifier>`: rename to `unused_vertices` & change type to `BTreeSet`

#### honeycomb-guide

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project</sup>

- update usage instructions

#### honeycomb-utils

<sup>utility routines used in benchmarking and testing</sup>

- update content according to features introduced in #15, #16

---

## 0.1.1

### Repository Changes

- new CI workflow:
    - `bench`: run hardware-counter based benchmarks on new version release

### New Features

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

- introduce `benchmarking_utils` feature, used to compile additional methods &
  trait implementation useful for benchmarking

#### honeycomb-guide

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project.</sup>

- complete (partially) definition sections (#5)
- add documentation for the core implementatiosn (#5)
- add usage instructions (#12)
- add `honeycomb-utils` section (#12)

#### honeycomb-utils (new member)

<sup>utility routines used in benchmarking and testing</sup>

- benchmarks for the `TwoMap` structure and methods (#10, #11)
- utility functions for benchmarking and testing.
- example of memory usage information for a given `TwoMap`

### Fixes

- typos by @cedricchevalier19 (#9)

---

## 0.1.0

### Repository Changes

- new CI workflows:
    - `doc`: build & deploy user guide and code documentation
    - `rust-test`: run Rust tests, format checker & linter (`clippy`)

### New Features

#### honeycomb-core (new member)

<sup>core definitions and tools for combinatorial map implementation</sup>

- `TwoMap` - basic 2D combinatorial map implementation
- full documentation support & deployment

#### honeycomb-guide (new member)

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project</sup>
