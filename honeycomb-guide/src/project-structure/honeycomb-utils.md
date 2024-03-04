# honeycomb-utils

[Documentation](../honeycomb_utils/)

--- 

**honeycomb-utils** is a Rust crate that provides utility functions to
the user (and developer) in order to write benchmarks and tests more
easily. A number of benchmarks are already defined in this crate using the 
[criterion][CRITERION] and [iai-callgrind][IAI] crates. Additionally, 
one example is provided to illustrate the result of the different *size*
methods on a given map, as well as a plotting script for the methods' output.

Note that the **iai-callgrind** benchmarks require their runner to be 
installed as well as Valgrind. Refer to the crate's [README][IAIRM] for 
detailed instructions.


## Usage

### Benchmarks

You can run benchmarks using the following commands:

```shell
# Run all benchmarks
cargo bench --benches

# Run a specific benchmark
cargo bench --bench <BENCHMARK>
```

The following benchmarks are available:

| Name                          | Type          | Source file                        |
|-------------------------------|---------------|-----------------------------------|
| `splitsquaremap-init`         | Criterion     | `benches/splitsquaremap/init.rs`  |
| `splitsquaremap-shift`        | Criterion     | `benches/splitsquaremap/shift.rs` |
| `squaremap-init`              | Criterion     | `benches/squaremap/init.rs`       |
| `squaremap-shift`             | Criterion     | `benches/squaremap/shift.rs`      |
| `squaremap-splitquads`        | Criterion     | `benches/squaremap/split.rs`      |
| `prof-twomap-editing`         | Iai-callgrind | `benches/core/twomap/editing.rs`  |
| `prof-twomap-reading`         | Iai-callgrind | `benches/core/twomap/reading.rs`  |
| `prof-twomap-sewing-unsewing` | Iai-callgrind | `benches/core/twomap/sewing.rs`   |

A detailed explanation about the purpose of each benchmark is provided at the beginning
of their respective source files. As a rule of thumb, the **iai-callgrind** benchmarks
cover individual methods of the structure while **criterion** benchmarks cover higher 
level computations.


### Examples

You can run examples using the following command:

```shell
# Run a specific example
cargo run --example <EXAMPLE>
```

The following examples are available:

| Name           | Description |
|----------------|-------------|
| `memory_usage` | Outputs the memory usage of a given map as three *csv* files. These files can be used to generate charts using the `memory_usage.py` script |


### Scripts

- `memory_usage.py` - **requires matplotlib** - Plots pie charts using a *csv* file produced by a size method of TwoMap.


[CRITERION]: https://github.com/bheisler/criterion.rs
[IAI]: https://github.com/iai-callgrind/iai-callgrind
[IAIRM]: https://github.com/iai-callgrind/iai-callgrind?tab=readme-ov-file#installation