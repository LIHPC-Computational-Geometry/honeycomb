# Benchmarks

## Binary

The package provides a single binary, `hc-bench`, which exposes several benchmarks as subcommands.
For details on options and arguments, run:

```shell
cargo run --bin hc-bench -- --help
```

The following subcommands are available:

- `generate-2d-grid`: 2D grid generation using `CMapBuilder` and `GridDescriptor`.
- `cut-edges`: Edge size reduction in triangular meshes using vertex/edge insertions.
- `grisubal`: `grisubal` kernel execution.
- `remesh`: Geometry capture, triangulation and remeshing kernel.
- `shift`: Simple vertex relaxation routine.

## Benchmarks

```shell
cargo bench --bench <BENCHMARK>
```

The following benchmarks are available:

| Name                         | Type          | Source file                          |
|------------------------------|---------------|--------------------------------------|
| `builder`                    | Criterion     | `benches/builder/time.rs`            |
| `builder-grid-size`          | Criterion     | `benches/builder∕grid_size.rs`       |
| `fetch-icells`               | Criterion     | `benches/core/cmap2/fetch_icells.rs` |
| `grisubal`                   | Criterion     | `benches/grisubal/time.rs`           |
| `grisubal-grid-size`         | Criterion     | `benches/grisubal/grid_size.rs`      |
| `prof-cmap2-basic`           | Iai-callgrind | `benches/core/cmap2/basic_ops.rs`    |
| `prof-cmap2-build`           | Iai-callgrind | `benches/core/cmap2/constructors.rs` |
| `prof-cmap2-sewing-unsewing` | Iai-callgrind | `benches/core/cmap2/link_and_sew.rs` |
| `triangulate-quads`          | Criterion     | `benches/triangulate/quads.rs`       |

A detailed explanation about the purpose of each benchmark is provided at the beginning of their
respective source files.

## Scripts

### Benchmarking

Both scripts provide an interactive menu with four options:

```
(0) all
(1) fixed-size profiling
(2) grid size scaling
(3) thread number scaling (not yet implemented)
```

#### `builder.py`

The script aggregates metrics about the grid building routines used by `CMapBuilder`. Data is
collected from Criterion (runtime benchmarks), perf (CPU profiling), flamegraph (visualization),
heaptrack (memory analysis), and across different grid sizes (from 128x128 to 8192x8192).

#### `grisubal.py`

The script aggregates metrics about the `grisubal` kernel. Data is collected from Criterion
(runtime benchmarks), perf (CPU profiling), flamegraph (visualization), and heaptrack (memory
analysis). Additionally, internal timers are implemented to measure time-per-sections of the
algorithm. Measurements are done across different grid granularities (from `0.1` to `1.0`
in `0.1` increments).

### Plotting

#### `grisubal_plot.py`

This script generates a pie chart from per-section timings sampled by the `grisubal.py` script.

