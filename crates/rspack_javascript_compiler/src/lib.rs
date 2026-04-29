pub mod ast;
mod compiler;
mod error;

pub use compiler::{
  IsolatedDtsTransformOutput, JavaScriptCompiler, TransformOutput, minify, parse, transform,
};
