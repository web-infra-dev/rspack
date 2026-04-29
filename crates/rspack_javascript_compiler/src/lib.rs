pub mod ast;
mod compiler;
mod error;

pub use compiler::{
  IsolatedDtsDiagnostic, IsolatedDtsTransformOutput, JavaScriptCompiler, TransformOutput, minify,
  parse, transform,
};
pub use error::render_pretty_span_diagnostic;
