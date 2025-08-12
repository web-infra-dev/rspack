pub mod ast;
mod compiler;
mod error;

pub use compiler::{JavaScriptCompiler, TransformOutput, minify, parse, transform};
