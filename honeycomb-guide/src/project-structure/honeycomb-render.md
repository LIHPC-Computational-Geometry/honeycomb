# honeycomb-render

[Documentation](../honeycomb_render/)

--- 

**honeycomb-render** is a Rust crate that provides a simple visualization framework
to allow the user to render their combinatorial map. It is designed to be used
directly in the code by reading data through a reference to the map (as opposed to
a binary that would read serialized data).

## Usage

### Quickstart

TODO

### Examples

You can run examples using the following command:

```shell
# Run a specific example
cargo run --example <EXAMPLE>
```

The following examples are available:

| Name                    | Description                                                                      |
|-------------------------|----------------------------------------------------------------------------------|
| `render_default_no_aa`  | Render a hardcoded arrow without anti-aliasing                                   |
| `render_default_smaa1x` | Render a hardcoded arrow with anti-aliasing                                      |
| `render_splitsquaremap` | Render a map generated using functions provided by the **honeycomb-utils** crate |
| `render_squaremap`      | Render a map generated using functions provided by the **honeycomb-utils** crate |

