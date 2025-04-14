# Contributing

Contributions are welcome and accepted as pull requests on [GitHub][GH]. Feel free to use issues to report bugs,
missing documentation or suggest improvements of the project.


## Local environment

### Nix

WIP


### Pre-commit hook

The repository contains a pre-commit hook config file. It will run basic checks that the CI performs on each PRs
and commits merged to the main branch. To use it:

```shell
pip install pre-commit # or whichever package manager
pre-commit install
pre-commit run # test it!
```

While it is not exhaustive (most notably, it excludes `honeycomb-render` due to compile time), it is a good
local proxy of the CI for core and kernel crates development.

The hook can be bypassed by using the `--no-verify` option to `git commit`. Also note that clippy warnings
are denied in the CI.


## Documentation

Note that a most of the code possess documentation, including private modules / items / sections. You can generate
the complete documentation by using the following instructions:

```shell
mdbook serve --open user-guide/
```

```shell
cargo +nightly doc --all --all-features --no-deps --document-private-items
```

[GH]: https://github.com/LIHPC-Computational-Geometry/honeycomb
