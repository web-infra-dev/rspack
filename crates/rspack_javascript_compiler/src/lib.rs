// See crates/rspack_javascript_compiler/src/compiler/compiler_builtins_probestack.rs
// for the reason why this feature is needed.
#![feature(abi_custom)]
#![allow(internal_features)]
#![feature(rustc_attrs)]

pub mod ast;
mod compiler;
mod error;

pub use compiler::{JavaScriptCompiler, TransformOutput, minify, parse, transform};

// Force linking of probestack symbols when conditions are met
// This ensures the __rust_probestack symbol is always included in the binary
// See crates/rspack_javascript_compiler/src/compiler/compiler_builtins_probestack.rs
#[cfg(all(
  not(any(windows, target_os = "cygwin")),
  any(target_arch = "x86_64", target_arch = "x86")
))]
#[used]
static _FORCE_LINK_PROBESTACK: unsafe extern "custom" fn() =
  compiler::compiler_builtins_probestack::__rust_probestack;
