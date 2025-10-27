# Other content

---

## Applications

The `applications` crate contains multiple binaries used as material to showcase the library. These
serve as examples and/or benchmarks of our implementation.

The following binaries are available; run `cargo run --bin <BIN> -- --help` for usage information:

- `cut-edges` -- cut edges of a triangular mesh recursively until a target length is reached
- `generate-grid` -- generate a grid-like mesh of a 2D or 3D box
- `grisubal` -- capture an input geometry using an overlay grid algorithm
- `remesh` -- capture and iteratively remesh an input geometry
- `shift-vertices` -- relax all inner vertices of a mesh
- `triangulate` -- triangulate a 2D polygonal mesh

All binaries have a `hwloc` dependency by default to bind threads to phyisical cores. It can be
removed by using the option `--no-default-features`.

---

## User guide

The **user guide** is the documentation you are currently reading right now. It is generated using mdbook. Its content
mainly focuses on definition and feature-listing rather than technical details on implementation. The latter can be
found in the code documentation.

### Building

You can generate this documentation locally using **mdbook**:

```shell
mdbook serve --open user-guide/
```

### Additional Information

A few observations on writing documentation using **mdbook**:

- If you edit the user guide's content, you will have to generate the rust doc again as mdbook remove all files of its
  target folder at each update.
- Linking to `html` files (and not markdown) has a varying level of success when working locally. Your browser may or
  may not like links toward folders instead of explicit `index.html`.
