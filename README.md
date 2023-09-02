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

## Features

There is now experimental support for compiling with BLAS.

Available features:

- `n_k_quants` - Disables building with k_quant quantizations (i.e. Q4_K)
- `no_accelerate` - Only relevant on Mac, disables building with Accelerate.
- `use_cmake` - Builds and links against `libllama` using cmake.
- `cublas` - Nvidia's CUDA BLAS implementation.
- `clblast` - OpenCL BLAS.
- `hipblas` - AMD's ROCM/HIP BLAS implementation. Set the `ROCM_PATH` environment variable to point at your ROCM installation. It defaults to `/opt/rocm`
- `openblas` - OpenBLAS.
- `metal` - Metal support, only available on Mac.

Enabling any of the BLAS features or `metal` implies `use_cmake`. You will need a working C++ compiler and cmake set up to build with this feature. Due to limitations in the llama.cpp cmake build system currently, it's necessary to build and link against `libllama` (which pulls in stuff like `libstdc++`) even though we only need GGML. Also, although we can build the library using cmake there's no simple way to know the necessary library search paths and libraries: we try to make a reasonable choice here but if you have libraries in unusual locations or multiple versions then weird stuff may happen.


## Limitations

The project has a slow, irresponsible person like me maintaining it. This is not an ideal situation.

## Credits

### GGML

The files under `ggml-src/` are distributed under the terms of the MIT license. As they are simply copied from the source repo (see below) refer to that for definitive information on the license or credits.

Credit goes to the original authors: Copyright (c) 2023 Georgi Gerganov

Currently automatically generated from the [llama.cpp](https://github.com/ggerganov/llama.cpp/) project.

### Build Scripts

Initially derived from the build script and bindings generation in the Rustformers [llm](https://github.com/rustformers/llm/) project.
