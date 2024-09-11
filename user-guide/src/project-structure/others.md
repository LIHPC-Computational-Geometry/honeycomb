# Other content

---

## Benchmarks

**honeycomb-benches** is a Rust crate used to group benchmarking routines of the Rust code. Benchmarks are defined in
this crate using the [criterion][CRITERION] and [iai-callgrind][IAI] crates.

Note that the **iai-callgrind** benchmarks require their runner to be installed as well as Valgrind. Refer to the
crate's [README][IAIRM] for detailed instructions.

### Usage

You can run benchmarks using the following commands:

```shell
# Run all benchmarks
cargo bench --benches

# Run a specific benchmark
cargo bench --bench <BENCHMARK>
```

The following benchmarks are available:

| Name                         | Type          | Source file                          |
|------------------------------|---------------|--------------------------------------|
| `splitsquaremap-init`        | Criterion     | `benches/splitsquaremap/init.rs`     |
| `splitsquaremap-shift`       | Criterion     | `benches/splitsquaremap/shift.rs`    |
| `squaremap-init`             | Criterion     | `benches/squaremap/init.rs`          |
| `squaremap-shift`            | Criterion     | `benches/squaremap/shift.rs`         |
| `squaremap-splitquads`       | Criterion     | `benches/squaremap/split.rs`         |
| `prof-cmap2-basic`           | Iai-callgrind | `benches/core/cmap2/basic_ops.rs`    |
| `prof-cmap2-build`           | Iai-callgrind | `benches/core/cmap2/constructors.rs` |
| `prof-cmap2-sewing-unsewing` | Iai-callgrind | `benches/core/cmap2/link_and_sew.rs` |

A detailed explanation about the purpose of each benchmark is provided at the beginning of their respective source
files. As a rule of thumb, the **iai-callgrind** benchmarks cover individual methods of the structure while
**criterion** benchmarks cover higher level computations.

[CRITERION]: https://github.com/bheisler/criterion.rs

[IAI]: https://github.com/iai-callgrind/iai-callgrind

[IAIRM]: https://github.com/iai-callgrind/iai-callgrind?tab=readme-ov-file#installation

---

## Examples

**honeycomb-examples** is a Rust crate used to group examples & snippets illustrating the crates' usage.

### Usage

You can run examples using the following command:

```shell
# Run a specific example
cargo run --example <EXAMPLE>
```

The following examples are available:

| Name           | Description                                                                                                        |
|----------------|--------------------------------------------------------------------------------------------------------------------|
| `io_read`      | Initialize and render a map from the VTK file passed to the command line.                                          |
| `io_write`     | Serialize the map that is built by the `squaremap_split_some` benchmark.                                           |
| `memory_usage` | Outputs the memory usage of a given map as three *csv* files. Use `memory_usage.py` to generate charts from those. |
| `render`       | Render a map representing a simple orthogonal grid.                                                                |

### Scripts

- `memory_usage.py` - **requires matplotlib** - Plots pie charts using a *csv* file produced by
  a size method of CMap2.

---

## User guide

The **user guide** is the documentation you are currently reading right now. It is generated using mdbook. Its content
mainly focuses on definition and feature-listing rather than technical details on implementation. The latter can be
found in the code documentation.

### Building

You can generate this documentation locally using **mdbook** and **cargo doc**:

```shell
mdbook serve --open -d ../target/doc/ user-guide/ &
cargo doc --all --no-deps
```

### Additional Information

Note that a most of the code possess documentation, including private modules / items / sections. You can generate
the complete documentation by using the instructions above and passing the option `--document-private-items`
to `cargo doc`.

A few observations on writing documentation using **mdbook**:

- If you edit the user guide's content, you will have to generate the rust doc again as mdbook remove all files of its
  target folder at each update.
- When linking to a folder containing an `index.html` file, be sure to include the last `/` in the name of the folder
  if you don't name the index file directly. Not including that last character seems to break in-file linking of the
  local version.