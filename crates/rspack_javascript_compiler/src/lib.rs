// See crates/rspack_javascript_compiler/src/compiler/compiler_builtins_probestack.rs
// for the reason why this feature is needed.
#![feature(abi_custom)]
#![allow(internal_features)]
#![feature(rustc_attrs)]

pub mod ast;
mod compiler;
mod error;

pub use compiler::{JavaScriptCompiler, TransformOutput, minify, parse, transform};
