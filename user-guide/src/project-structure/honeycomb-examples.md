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

| Name           | Description                                                                                                        |
|----------------|--------------------------------------------------------------------------------------------------------------------|
| `io_read`      | Initialize and render a map from the VTK file passed to the command line.                                          |
| `io_write`     | Serialize the map that is built by the `squaremap_split_some` benchmark.                                           |
| `memory_usage` | Outputs the memory usage of a given map as three *csv* files. Use `memory_usage.py` to generate charts from those. |
| `render`       | Render a map representing a simple orthogonal grid.                                                                |

### Scripts

- `memory_usage.py` - **requires matplotlib** - Plots pie charts using a *csv* file produced by
  a size method of CMap2.
