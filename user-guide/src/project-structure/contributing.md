# Contributing

Contributions are welcome and accepted as pull requests on [GitHub][GH]. Feel free to use issues to report bugs,
missing documentation or suggest improvements of the project.


## Environment

The repository contains a Nix flake to easily setup a development environment:

```
nix develop
```

Most notably, it handles `hwloc` install on both MacOs and Linux, as well as the libraries `bevy` depends on on Linux.


## Checks

### Nix

The flake also defines checks. They are identical to those of the CI, so use this rather than the pre-commit
if possible.

```
nix flake check
```


### Pre-commit hook

The repository contains a pre-commit hook config file. To use it:

```shell
pip install pre-commit # or whichever package manager
pre-commit install
pre-commit run # test it!
```

While it is not identical to the CI (most notably, it excludes `honeycomb-render` due to compile time), it is fine
for core and kernel crates development.

The hook can be bypassed by using the `--no-verify` option to `git commit`.


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
