pub mod ast;
mod compiler;
mod error;

#[cfg(feature = "codspeed")]
#[doc(hidden)]
pub use compiler::stringify::benchmark_source_map_position_conversion;
pub use compiler::{
  IsolatedDtsTransformOutput, JavaScriptCompiler, TransformOutput, minify, parse, transform,
};
