pub mod ast;
mod compiler;
mod error;

pub use compiler::{minify, parse, transform, JavaScriptCompiler, TransformOutput};
