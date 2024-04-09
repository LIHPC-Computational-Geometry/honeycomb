# honeycomb-benches

[Documentation](../honeycomb_benches/)

--- 

**honeycomb-benches** is a Rust crate used to group benchmarking routines of
the Rust code. Benchmarks are defined in this crate using the [criterion][CRITERION]
and [iai-callgrind][IAI] crates.

Note that the **iai-callgrind** benchmarks require their runner to be
installed as well as Valgrind. Refer to the crate's [README][IAIRM] for
detailed instructions.

## Usage

You can run benchmarks using the following commands:

```shell
# Run all benchmarks
cargo bench --benches

# Run a specific benchmark
cargo bench --bench <BENCHMARK>
```

The following benchmarks are available:

| Name                         | Type          | Source file                       |
|------------------------------|---------------|-----------------------------------|
| `splitsquaremap-init`        | Criterion     | `benches/splitsquaremap/init.rs`  |
| `splitsquaremap-shift`       | Criterion     | `benches/splitsquaremap/shift.rs` |
| `squaremap-init`             | Criterion     | `benches/squaremap/init.rs`       |
| `squaremap-shift`            | Criterion     | `benches/squaremap/shift.rs`      |
| `squaremap-splitquads`       | Criterion     | `benches/squaremap/split.rs`      |
| `prof-cmap2-editing`         | Iai-callgrind | `benches/core/cmap2/editing.rs`   |
| `prof-cmap2-reading`         | Iai-callgrind | `benches/core/cmap2/reading.rs`   |
| `prof-cmap2-sewing-unsewing` | Iai-callgrind | `benches/core/cmap2/sewing.rs`    |

A detailed explanation about the purpose of each benchmark is provided at the beginning
of their respective source files. As a rule of thumb, the **iai-callgrind** benchmarks
cover individual methods of the structure while **criterion** benchmarks cover higher
level computations.

[CRITERION]: https://github.com/bheisler/criterion.rs

[IAI]: https://github.com/iai-callgrind/iai-callgrind

[IAIRM]: https://github.com/iai-callgrind/iai-callgrind?tab=readme-ov-file#installation
