// Build script and bindings generation modified from https://github.com/rustformers/llama-rs

use std::{collections::HashSet, env, path::PathBuf};

const GGML_SOURCE_DIR: &str = "ggml-src";
const GGML_HEADER: &str = "ggml.h";
const GGML_SOURCE: &str = "ggml.c";

fn generate_bindings() {
    let ggml_header_path = PathBuf::from(GGML_SOURCE_DIR).join(GGML_HEADER);
    let librs_path = PathBuf::from("src").join("lib.rs");

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

    // If running on docs.rs, the filesystem is readonly so we can't actually generate
    // anything. This package should have been fetched with the bindings already generated
    // so we just exit  here.
    if env::var("DOCS_RS").is_ok() {
        return;
    }

    generate_bindings();

    let ggml_source_path = PathBuf::from(GGML_SOURCE_DIR).join(GGML_SOURCE);
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

                features.iter().for_each(|feat| {
                    build.flag(&format!("-m{feat}"));
                });
            } else if compiler.is_like_msvc() {
                if features.contains("avx2") {
                    build.flag("/arch:AVX2");
                } else if features.contains("avx") {
                    build.flag("/arch:AVX");
                }
            }
        }
        "aarch64" => {
            if compiler.is_like_clang() || compiler.is_like_gnu() {
                if std::env::var("HOST") == std::env::var("TARGET") {
                    build.flag("-mcpu=native");
                } else if &target_os == "macos" {
                    build.flag("-mcpu=apple-m1");
                    build.flag("-mfpu=neon");
                }
                build.flag("-pthread");
            }
        }
        _ => (),
    }

    if &target_os == "macos" {
        build.define("GGML_USE_ACCELERATE", None);
        println!("cargo:rustc-link-lib=framework=Accelerate");
    }

    if is_release {
        build.define("NDEBUG", None);
    }
    build.warnings(false);
    build.compile(GGML_SOURCE_DIR);
}

fn get_supported_target_features() -> HashSet<String> {
    env::var("CARGO_CFG_TARGET_FEATURE")
        .unwrap()
        .split(',')
        .filter(|s| x86::RELEVANT_FLAGS.contains(s))
        .map(ToString::to_string)
        .collect::<HashSet<_>>()
}

mod x86 {
    use super::HashSet;

    pub const RELEVANT_FLAGS: &[&str] = &["fma", "avx", "avx2", "f16c", "sse3"];
    pub struct Features(HashSet<String>);

    impl std::ops::Deref for Features {
        type Target = HashSet<String>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Features {
        pub fn get() -> Self {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            if std::env::var("HOST") == std::env::var("TARGET") {
                return Self::get_host();
            }
            Self(super::get_supported_target_features())
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        pub fn get_host() -> Self {
            Self(
                [
                    std::is_x86_feature_detected!("fma"),
                    std::is_x86_feature_detected!("avx"),
                    std::is_x86_feature_detected!("avx2"),
                    std::is_x86_feature_detected!("f16c"),
                    std::is_x86_feature_detected!("sse3"),
                ]
                .into_iter()
                .enumerate()
                .filter(|(_, exists)| *exists)
                .map(|(idx, _)| RELEVANT_FLAGS[idx].to_string())
                .collect::<HashSet<_>>(),
            )
        }
    }
}
