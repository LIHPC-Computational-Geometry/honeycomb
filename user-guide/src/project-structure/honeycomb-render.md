# honeycomb-render

[Documentation](../honeycomb_render/)

--- 

**honeycomb-render** is a Rust crate that provides a simple visualization framework to allow the user to render their
combinatorial map. It is designed to be used directly in the code by reading data through a reference to the map (as
opposed to a binary that would read serialized data). This render tool can be used to debug algorithm results in a 
significantly easier way than reading internal data would.

## Usage

Use the [App](../honeycomb_render/struct.App.html) structure to render a given combinatorial map. You may need to run
the program in `release` mode to render large maps.

Examples are provided in the [dedicated crate](./honeycomb-examples.md), under the `examples/render/` directory.