# `ggml-sys-bleedingedge`

Bleeding edge Rust bindings for GGML.

## Release Info

See [`VERSION.txt`](./VERSION.txt), [`ggml-tag-current.txt`](./ggml-tag-current.txt)

This repo is set up with a workflow to automatically check for the latest GGML release
several times per day. The workflow currently just builds for Linux x86: if that build
succeeds, then a new release and package will be published.

Note that the GGML project is undergoing very rapid development. Other than being able
to generate the binding and build the package (on x86 Linux at least) you really can't
make any assumptions about a release of this crate.

Releases will be in the format `YYMMDDHHMM.0.0+sourcerepo-release.releasename` (UTC).
At present, `sourcerepo` is going to be `llamacpp` (from the `llama.cpp` repo) but at
some point it may change to point to the `ggml` repo instead (currently `llama.cpp` seems
to get the features first). Build metadata after the `+` is informational only.

## Crate

You can find the crate published here: https://crates.io/crates/ggml-sys-bleedingedge

Automatically generated documentation: https://docs.rs/ggml-sys-bleedingedge/

## Limitations

GGML recently added support for GPU offloading via CUDA and OpenCL. At the moment, it's not possible to
take advantage of that with this crate. I have plans to support this in the future via feature flags
but it may be a while before that actually happens.

## Credits

### GGML

The `ggml.c` and `ggml.h` files are distributed under the terms of the MIT license.

Credit goes to the original authors: Copyright (c) 2023 Georgi Gerganov

Currently automatically generated from the [llama.cpp](https://github.com/ggerganov/llama.cpp/) project.

### Build Scripts

Initially derived from the build script and bindings generation in the [llama-rs](https://github.com/rustformers/llama-rs/) project.
