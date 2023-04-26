// Build script and bindings generation modified from https://github.com/rustformers/llama-rs

use std::{env, fs::metadata, path::PathBuf};

fn generate_bindings() {
    const SRC_FILES: &[&str] = &["ggml.h", "ggml.c"];

    let ggml_paths = SRC_FILES
        .iter()
        .map(|fname| PathBuf::from("ggml-src").join(fname))
        .collect::<Vec<_>>();
    let ggml_header_path = &ggml_paths[0];
    let ggml_mdata = ggml_paths
        .iter()
        .map(|p| metadata(p).expect(&format!("Cannot get metadata for {p:?}")))
        .collect::<Vec<_>>();
    let librs_path = PathBuf::from("src").join("lib.rs");
    if let Ok(lib_md) = metadata(&librs_path) {
        assert!(
            ggml_mdata.iter().all(|md| md.is_file()),
            "Unexpected non-file in source list"
        );
        assert!(lib_md.is_file(), "lib.rs unexpectedly isn't a file?");
        let ggml_stamps = ggml_mdata
            .iter()
            .map(|md| {
                md.modified()
                    .or_else(|_| md.created())
                    .expect("Could not get timestamp")
            })
            .collect::<Vec<_>>();
        let lib_stamp = lib_md
            .modified()
            .or_else(|_| lib_md.created())
            .expect("Couldn't get timestamp for lib.rs");
        if lib_md.len() > 0 && ggml_stamps.iter().all(|stamp| &lib_stamp >= stamp) {
            return;
        }
    }

    let bindings = bindgen::Builder::default()
        .header(ggml_header_path.to_string_lossy())
        // Suppress some warnings
        .raw_line("#![allow(non_upper_case_globals)]")
        .raw_line("#![allow(non_camel_case_types)]")
        .raw_line("#![allow(non_snake_case)]")
        .raw_line("#![allow(unused)]")
        .raw_line("pub const GGMLSYS_VERSION: Option<&str> = option_env!(\"CARGO_PKG_VERSION\");")
        // Do not generate code for ggml's includes (stdlib)
        .allowlist_file(ggml_header_path.to_string_lossy())
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(librs_path)
        .expect("Couldn't write bindings");
}

fn main() {
    // By default, this crate will attempt to compile ggml with the features of your host system if
    // the host and target are the same. If they are not, it will turn off auto-feature-detection,
    // and you will need to manually specify target features through target-features.

    println!("cargo:rerun-if-changed=ggml-src");
    println!("cargo:rerun-if-changed=src/lib.rs");

    generate_bindings();

    let ggml_source_path = PathBuf::from("ggml-src").join("ggml.c");
    let mut builder = cc::Build::new();
    let build = builder.files([ggml_source_path].iter()).include("include");

    // This is a very basic heuristic for applying compile flags.
    // Feel free to update this to fit your operating system.
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let is_release = env::var("PROFILE").unwrap() == "release";
    let compiler = build.get_compiler();

    match target_arch.as_str() {
        "x86" | "x86_64" => {
            let features = x86::Features::get();

            if compiler.is_like_clang() || compiler.is_like_gnu() {
                build.flag("-pthread");

                if features.avx {
                    build.flag("-mavx");
                }
                if features.avx2 {
                    build.flag("-mavx2");
                }
                if features.fma {
                    build.flag("-mfma");
                }
                if features.f16c {
                    build.flag("-mf16c");
                }
                if features.sse3 {
                    build.flag("-msse3");
                }
            } else if compiler.is_like_msvc() {
                match (features.avx2, features.avx) {
                    (true, _) => {
                        build.flag("/arch:AVX2");
                    }
                    (_, true) => {
                        build.flag("/arch:AVX");
                    }
                    _ => {}
                }
            }
        }
        "aarch64" => {
            if compiler.is_like_clang() || compiler.is_like_gnu() {
                if std::env::var("HOST") == std::env::var("TARGET") {
                    build.flag("-mcpu=native");
                } else {
                    #[allow(clippy::single_match)]
                    match target_os.as_str() {
                        "macos" => {
                            build.flag("-mcpu=apple-m1");
                            build.flag("-mfpu=neon");
                        }
                        _ => {}
                    }
                }
                build.flag("-pthread");
            }
        }
        _ => {}
    }

    #[allow(clippy::single_match)]
    match target_os.as_str() {
        "macos" => {
            build.define("GGML_USE_ACCELERATE", None);
            println!("cargo:rustc-link-lib=framework=Accelerate");
        }
        _ => {}
    }

    if is_release {
        build.define("NDEBUG", None);
    }
    build.warnings(false);
    build.compile("ggml-src");
}

fn get_supported_target_features() -> std::collections::HashSet<String> {
    env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap()
        .split(',')
        .map(ToString::to_string)
        .collect()
}

mod x86 {
    #[allow(clippy::struct_excessive_bools)]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct Features {
        pub fma: bool,
        pub avx: bool,
        pub avx2: bool,
        pub f16c: bool,
        pub sse3: bool,
    }
    impl Features {
        pub fn get() -> Self {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if std::env::var("HOST") == std::env::var("TARGET") {
                return Self::get_host();
            }

            Self::get_target()
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        pub fn get_host() -> Self {
            Self {
                fma: std::is_x86_feature_detected!("fma"),
                avx: std::is_x86_feature_detected!("avx"),
                avx2: std::is_x86_feature_detected!("avx2"),
                f16c: std::is_x86_feature_detected!("f16c"),
                sse3: std::is_x86_feature_detected!("sse3"),
            }
        }

        pub fn get_target() -> Self {
            let features = crate::get_supported_target_features();
            Self {
                fma: features.contains("fma"),
                avx: features.contains("avx"),
                avx2: features.contains("avx2"),
                f16c: features.contains("f16c"),
                sse3: features.contains("sse3"),
            }
        }
    }
}
