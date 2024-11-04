# Other content

---

## Benchmarks

**honeycomb-benches** is the crate used to group benchmarking routines of the Rust code. It contains:

- binaries used to profile code and kernels
- benchmarks implemented using the [criterion][CRITERION] crate
- scritps used to aggregate and (partially) process results of both

[CRITERION]: https://github.com/bheisler/criterion.rs


---

## Examples

**honeycomb-examples** is the crate used to group examples & snippets illustrating possible usages. It also 
contains sample VTK files to ensure examples and benchmarks can run out-of-the-box.


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
