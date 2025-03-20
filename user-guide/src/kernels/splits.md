# Cell insertions

---

Cell insertion routinesare implemented in [this](../../honeycomb_kernels/cell_insertion/index.html) module. For
consistency reasons, we limit our implementations to inserting cell of dimension `N-1` in cellof dimension `N`.

## Functions

### Edges

Two splitting functions are implemented over edges, each having a no-allocation variant:

- `insert_vertex_on_edge`: insert a vertex on an edge, effectively splitting it into 2 segments.
- `insert_vertices_on_edge`: insert `n` vertices on an edge, effectively splitting it into `n+1` segments.

