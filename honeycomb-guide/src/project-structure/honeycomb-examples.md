# honeycomb-examples

[Documentation](../honeycomb_examples/)

--- 

**honeycomb-examples** is a Rust crate used to group examples & snippets illustrating the crates' usage.

## Usage

You can run examples using the following command:

```shell
# Run a specific example
cargo run --example <EXAMPLE>
```

The following examples are available:

| Name                          | Description                                                                                                                                 |
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|
| `memory_usage`                | Outputs the memory usage of a given map as three *csv* files. These files can be used to generate charts using the `memory_usage.py` script |
| `render_default_no_aa`        | Render a hardcoded arrow without anti-aliasing                                                                                              |
| `render_default_smaa1x`       | Render a hardcoded arrow with anti-aliasing                                                                                                 |
| `render_splitsquaremap`       | Render a map generated using functions provided by the `utils` module of the core crate                                                     |
| `render_squaremap`            | Render a map generated using functions provided by the `utils` module of the core crate                                                     |
| `render_squaremap_shift`      | Render a map computed by the `squaremap-shift` benchmark. Inner vertices are shifted by a random offset value.                              |
| `render_squaremap_split_diff` | Render a map computed by the `squaremap-splitquads` benchmark. All quads are split diagonally, which diagonal chosen at random.             |
| `render_squaremap_split_some` | Render a map computed by the `squaremap-splitquads` benchmark. Only some quads are split diagonally, chosen at random.                      |

### Scripts

- `memory_usage.py` - **requires matplotlib** - Plots pie charts using a *csv* file produced by
  a size method of CMap2.
