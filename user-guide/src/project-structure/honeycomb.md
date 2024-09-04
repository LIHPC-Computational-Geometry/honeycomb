# honeycomb

[Documentation](../honeycomb/)

---

**honeycomb** is the main crate provided to user and serve as the entrypoint for combinatorial map usage. It is 
exclusively made up of re-exports from the core, kernels and render crate to provide a clean, all-in-one dependency.

At the moment, the `honeycomb` name is not available on crates.io; this means that using this crate requires adding 
the dependency using the git repository:

```toml
# Cargo.toml

[dependencies]
honeycomb = { git = "https://github.com/LIHPC-Computational-Geometry/honeycomb" }
```