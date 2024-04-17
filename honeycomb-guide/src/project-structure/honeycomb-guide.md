# honeycomb-guide

---

**honeycomb-guide** is the mdbook project used to generate the documentation you are currently reading. Its content
mainly focuses on definition and feature-listing rather than technical details on implementation. The latter can be
found in the code documentation.

## Building

You can generate this documentation locally using **mdbook** and **cargo doc**:

```shell
mdbook serve --open -d ../target/doc/ honeycomb-guide/ &
cargo doc --all --no-deps
```

## Additional Information

Note that a most of the code possess documentation, including private modules / items / sections. You can generate
the complete documentation by using the instructions above and passing the option `--document-private-items`
to `cargo doc`.

A few observations on writing documentation using **mdbook**:

- If you edit the user guide's content, you will have to generate the rust doc again as mdbook remove all files of its
  target folder at each update.
- When linking to a folder containing an `index.html` file, be sure to include the last `/` in the name of the folder
  if you don't name the index file directly. Not including that last character seems to break in-file linking of the
  local version.