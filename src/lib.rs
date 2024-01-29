#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

pub const GGMLSYS_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");

include!("bindings.rs");
