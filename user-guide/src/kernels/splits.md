# Splits

---

We refer to "splits" or "splitting functions" when talking about higher-level routines involving the split of
a geometrical cell (e.g. an edge/1-cell). They are implemented in [this](../../honeycomb_kernels/splits/index.html)
module.

## No-allocation variants

To avoid small, repetitive call to system allocator, we implement no-allocation variants of these routines.
Instead of adding darts to the map in the function, the variants take an additional argument: a slice of
free darts. These darts are then used to apply modifications to the map.

This, coupled with grouped pre-allocation, allows to significantly reduce the number of allocator calls in programs.

## Functions

### Edges

Two splitting functions are implemented over edges, each having a no-allocation variant:

- `split_edge` / `split_edge_noalloc`: split an edge in two, inserting one new vertex in the map
- `splitn_edge` / `splitn_edge_noalloc`: split an edge into `n+1` segments, `n` being the number of new
  intermediate vertices

