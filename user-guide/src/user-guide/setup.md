# Setup

---

Honeycomb's sub-crates are published on [crates.io](https://crates.io/crates/honeycomb-core). The main crate isn't due
to name availability. The framework is used like any Rust crate would be:

```sh
cargo new my_mesh_app
cd my_mesh_app
```

Add sub-crates from the registry:

```toml
# Cargo.toml
[dependencies]
honeycomb-core    = "0.12.0"
honeycomb-kernels = "0.12.0"
```

Or the main crate from the repository:

```toml
# Cargo.toml
[dependencies]
honeycomb = {
    git = "https://github.com/LIHPC-Computational-Geometry/honeycomb.git",
    tag = "0.12.0",
}
```

The `honeycomb` crate is a facade re-exporting the implementation crates:

| Crate               | Content                                       | Re-exported as      |
| ------------------- | --------------------------------------------- | ------------------- |
| `honeycomb-core`    | combinatorial maps, attributes, geometry, STM | `honeycomb::core`   |
| `honeycomb-kernels` | ready-to-use meshing algorithms               | `honeycomb::kernels`|
| `honeycomb-render`  | visual debugging tool (using Bevy)            | `honeycomb::render` |

Most tutorials use the prelude, which gathers the commonly used items:

```rust
// src/*.rs
use honeycomb::prelude::*;
```

## Features

Multiple features can be enabled and disabled depending on application needs:

| Feature         | Default | Effect                                                                                          |
|-----------------|---------|-------------------------------------------------------------------------------------------------|
| `kernels`       | yes     | exposes `honeycomb::kernels` (meshing algorithms)                                               |
| `par-internals` | yes     | parallelizes some internal routines of the core crate with rayon                                |
| `render`        | no      | exposes `render_2d_map` / `render_3d_map`; significantly lengthens the dependency tree and compile time |
| `profiling`     | no      | STM event counters (`TransactionTallies`) and `perf` control fifo macros, see [Tutorial #4](./tutorial4-optimization.md)      |

```toml
# Cargo.toml
[dependencies]
honeycomb = {
    git = "https://github.com/LIHPC-Computational-Geometry/honeycomb.git",
    tag = "0.12.0",
    features = ["render"], # visual debugging (Tutorial #1)
}
```

If you only need part of the library, you can depend on the sub-crates directly
(`honeycomb-core`, `honeycomb-kernels`, `honeycomb-render`).

## Requirements

- **Rust stable release** -- the MSRV may not be the latest stable release, but no compatibility
  guarantee is given for older versions.

> **Disclaimer**: this guide assumes some level of familiarity with Rust -- ownership and
> borrowing, generics, traits and closures are used throughout the tutorials. If needed, the
> [Rust Book](https://doc.rust-lang.org/book/) and
> [Rust by Example](https://doc.rust-lang.org/rust-by-example/) are good starting points; see also
> the [Resources](../definitions/resources.md) page.
