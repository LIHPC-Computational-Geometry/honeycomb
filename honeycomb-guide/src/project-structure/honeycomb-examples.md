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

| Name                    | Description                                                                                                                                 |
|-------------------------|---------------------------------------------------------------------------------------------------------------------------------------------|
| `memory_usage`          | Outputs the memory usage of a given map as three *csv* files. These files can be used to generate charts using the `memory_usage.py` script |
| `render_default_no_aa`  | Render a hardcoded arrow without anti-aliasing                                                                                              |
| `render_default_smaa1x` | Render a hardcoded arrow with anti-aliasing                                                                                                 |
| `render_splitsquaremap` | Render a map generated using functions provided by the **honeycomb-utils** crate                                                            |
| `render_squaremap`      | Render a map generated using functions provided by the **honeycomb-utils** crate                                                            |

### Scripts

- **OUTDATED, WILL FIX** `memory_usage.py` - **requires matplotlib** - Plots pie charts using a *csv* file produced by
  a size method of CMap2.
