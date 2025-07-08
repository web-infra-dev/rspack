pub mod ast;
mod compiler;
mod error;

pub use compiler::{minify, parse, transform, JavaScriptCompiler, TransformOutput};
pub use rspack_workspace::rspack_swc_core_version;
