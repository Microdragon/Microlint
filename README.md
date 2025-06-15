# Microlint
> Linting for the Microdragon kernel.

[![License](https://img.shields.io/github/license/Microdragon/Microlint?style=flat-square)](LICENSE)

Microlint provides rustc lints for use in the Microdragon kernel project. The
lints are implemented using [`dylint`](https://github.com/trailofbits/dylint)
and the rustc linting infrastructure.

Currently the following lints are provided:

### calling_init_from_non_init

A lint that checks if a functions marked as `#[init]` is called from a
non-`#[init]` function. This is required, since `#[init]` functions might be
unmapped, once the kernel init phase completes, dues calling them would cause a
page fault in the kernel. This lint prevents accidentally calling an `#[init]`
function.

### accessing_init_from_non_init

A lint that checks if a static marked as `#[init]` is accessed by a
non-`#[init]` function. This is required, since `#[init]` statics might be
unmapped, once the kernel init phase completes, dues accessing them would cause
a page fault in the kernel. This lint prevents accidentally accessing an
`#[init]` static.

## Contributing

Contributing is pretty straight forward. The only things required are
`cargo-dylint` and `dylint-link`, wich can be installed using cargo

```console
cargo install cargo-dylint dylint-link
```

Each lint is implemented as it's own crate under the `lint` folder, which are
then tied together in the main `microlint` crate. The main `microlint` crate
just depends on each lint crate and registers them in [`src/lib.rs`](src/lib.rs). Each lint
crates provides a readme, describing the lint in the clippy format as well as
snapshot tests in the `ui` folder of each lint.

If you got an idea for a new lint or found a bug in an existing one, don't
hesitate to create an issue for it. You are also free to implement the lint or
provide a bugfix yourself! We always welcome contributions!

## License

This Project, like most Microdragon projects, is licensed under the
[Mozilla Public License Version 2.0](https://www.mozilla.org/en-US/MPL/2.0/) ([LICENSE](LICENSE))

The [SPDX](https://spdx.dev) license identifier for this project is `MPL-2.0`.