# Change Log

---

## To be released

---

## 0.5.0

**This update contains breaking changes**

### Workspace

*fix:*

- fix errors introduced by `rand` update by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/130
- indent lists in doc correctly to comply with clippy by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/126

*deps:*

- bump codecov/codecov-action from 4.4.1 to 4.5.0 by @dependabot
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/112
- update iai-callgrind requirement from 0.11.0 to 0.12.0 by @dependabot
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/133
- bump `vtkio` version to `0.7.0-rc1` & disable its default features by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/142
- update iai-callgrind requirement from 0.12.0 to 0.13.0 by @dependabot
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/145


### Published crates

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

*new:*

- add `splitn_edge` method to `CMap2` by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/127
- add `remove_storage` method to `AttrStorageManager` by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/128
- add origin offset directly to grid descriptor/builder by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/148

*refactor:*

- **rewrite `Vertex2` & `Vector2` as tuple structs** by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/124

*fix:*

- extend generic attribute storages when adding darts by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/122

#### honeycomb-kernels (new member)

<sup>implementations of meshing kernels using combinatorial maps</sup>

*new:*

- add an overlay grid type algorithm: `grisubal`; it takes a 2D boundary as input and return a combinatorial map of 
  the boundary, captured in an orthogonal grid; an optional clipping step is also implemented
  - this was implemented in PRs #109, #111, #113, #115, #114, #116, #123, #119, #129, #131, #134, #135, #136, #137, #138, #140, #141, #143, #146, #147, #149, #151, and #152

*test:*

- add coverage using basic geometries by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/139


#### honeycomb-render

<sup>visualization tool for combinatorial maps</sup>

*fix:*

- add condition over shrink dir to prevent some crashes by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/121

### Others

#### honeycomb-examples

<sup>project examples</sup>

*new:*

- add example for the `grisubal` kernel by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/120

#### honeycomb-guide

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project</sup>

*new:*

- add `honeycomb-kernel` and `grisubal` pages by @imrn99 
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/117 and https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/153


---

## 0.4.0

**This update contains breaking changes**

### Workspace

- update visuals in code documentation & user guide by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/95
- bump various actions' versions

### Published crates

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

- implement a generic attribute system to bind custom data structures to topological entities of a `CMap2`
    - implement `AttrStorageManager` struct and `AttributeStorage` trait by @imrn99
      in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/89
    - **replace standalone impl blocks of attribute collections** by @imrn99
      in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/92
    - add `UnknownAttributeStorage` trait for attribute-agnostic methods by @imrn99
      in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/96
    - replace `Any` by `UnknownAttributeStorage` to handle collections in attribute manager by @imrn99
      in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/97
    - add storage manager to builder by @imrn99
      in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/102
    - add generic storage manager to `CMap2` by @imrn99
      in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/103
- change internal Vtk building routine to return a `Result` by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/91
- **expand `BuilderError` with more specialized variants** by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/91

---

## 0.3.1

### Workspace

- remove the feature matrix from the test CI, which now uses the `--all-features` option

### Published crates

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

*new:*

- add `CMap2::split_edge` for future algorithm implementations by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/83
- add `CMapBuilder` structure by @imrn99 in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/87

*refactor:*

- deprecate old constructors by @imrn99 in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/88

#### honeycomb-render

<sup>visualization tool for combinatorial maps</sup>

*fix:*

- update aspect ratio to prevent deformation of the rendered map by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/85
- shrink face relatively to their original sizes by @imrn99
  in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/86

### Others

#### honeycomb-benches

*refactor:*

- update and fix benchmarks by @imrn99 in https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/84

---

## 0.3.0

New changelog format!

### Workspace

- rename non-published members' folder name (#77)
- bump various dependencies' version & update code accordingly (#70, #81):
    - `iai-callgrind` from `0.10.2` to `0.11.0`
    - `smaa` from `0.13.0` to `0.14.0`
    - `wgpu` from `0.19.4` to `0.20.0`
    - `winit` from `0.29.15` to `0.30.0`
- implement nighly conditional compilation for doc generation (#66)

### Published crates

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

*new:*

- implement basic I/O logic for `CMap2` using the `vtkio` crate (#73, 75)
    - the scope of the support is detailed in the Rust Doc
    - new method & function are available when enabling the new `io` feature
- add a new feature, `io`, used to gate the implementation of VTK I/O code (#73)

*refactor:*

- remove the `FloatType`alias and the `single_precision` feature (#74, #76)
- replace `CoordsFloat` implementation blocks to automatize implementation
  for all types that fit the traits requirement (#68)
- remove `FloatType` from the public API (#68)
- change some main methods return type to have better overall consistency (#67):
    - `Vector2::normal_dir` now returns a result instead of potentially panicking
    - `CMap2::vertex` now returns a result instead of panicking if no associated vertex is found
- remove deprecated items from the last release (#64):
    - `AttrSparseVec::get_mut`, `AttrCompactVec::get_mut`
    - `utils::square_cmap2`, `utils::splitsquare_cmap2`

*fix*:

- correct the usage of epsilon values for floating point numbers comparison in tests (#79)

#### honeycomb-render

<sup>visualization tool for combinatorial maps</sup>

*refactor:*

- replace the `Runner` struct by a simpler `launch` function (#70)
    - the function takes the same parameters that were provided to `Runner::run`
- rewrite & reorganize most of the internal code due to `winit` update (#70)

### Others

#### honeycomb-examples

<sup>project examples</sup>

*new:*

- add three new examples to illustrate the new I/O features of the core crate (#73, #75)

#### honeycomb-guide

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project</sup>

- update usage information & content (#82)

---

## 0.2.1

### Workspace

- bump `rand` version from `0.8.5` to `0.9.0-alpha.1` & update code accordingly (#63)

### New features

#### honeycomb-core

<sup>core definitions and tools for combinatorial map implementation</sup>

- expand on tests of the core crate (#49)
- implement the `GridBuilder` struct as a better, more versatile way to generate grid maps (#60)

#### honeycomb-examples

<sup>project examples</sup>

- add the following examples:
    - `render_squaremap_parameterized` (#60)
    - `render_squaremap_shift`, based on benchmarking code (#52)
    - `render_squaremap_split_diff`, based on benchmarking code (#52)
    - `render_squaremap_split_some`, based on benchmarking code (#52)

#### honeycomb-guide

<sup>**mdbook**-based user guide with information regarding usage & non-code-related
aspects of the project</sup>

- update usage instructions (#50)
- add a **References** section to the index (#61)

#### honeycomb-render

<sup>visualization tool for combinatorial maps</sup>

- add code to properly render faces instead of using implicit coloring (#54)
    - this implied creating new internal structures for efficiency purposes
- add a cap on the number of frames rendered per second to fix speed disparity induced by machine performance (#56)

### Refactor

#### honeycomb-core

- mark as deprecated:
    - `AttrSparseVec::get_mut`, `AttrCompactVec::get_mut` (#49)
    - `utils::square_cmap2`, `utils::splitsquare_cmap2` (#60)
- fix various `clippy::pedantic` lints that were temporarily left as allowed (#51)
- fix some unwanted behaviors:
    - attribute re-insertion in `CMap2::two_unsew` (#55)
    - not-panicking execution paths of `CMap2::one_sew` & `CMap2::two_sew` (#59)

#### honeycomb-examples

- fix `memory_usage` Rust code and associated script (#55)

#### honeycomb-render

- update render code to:
    - skip darts and faces that have only one or less vertex defined (#59)
    - draw proper arrows instead of triangles (#62)
    - draw beta2 function as diamonds (#62)
- edit the shader to color triangles according to the entity they form (#62)
- add a key binding (F1) to close the render window (#62)

---

## 0.2.0

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
- update the `CMap2` quickstart example (#43)
- change the `CMap2::n_darts` method's return type to be more intuitive (#45)
    - add the `CMap2::n_unused_darts` method to provide an alternative to the old method
- remove the `CMap2::set_beta` method because of the lack of valid use case (#45)
- gate the `CMap2::set_betas` behind the `utils` feature (#45)

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
