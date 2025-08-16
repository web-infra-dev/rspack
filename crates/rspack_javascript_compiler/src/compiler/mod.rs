use std::sync::Arc;

// Wasmer depends on the __rust_probestack symbol
// This symbol is now mangled and no longer exposed (on nightly). Therefore, building wasmer fails during linking.
// Added this symbol back as a workaround.
// See: https://github.com/wasmerio/wasmer/issues/5610
#[doc(hidden)]
pub mod compiler_builtins_probestack;

pub mod minify;
pub mod parse;
pub mod stringify;
pub mod transform;

use rspack_sources::SourceMap;
use swc_core::common::{GLOBALS, Globals, SourceMap as SwcSourceMap};

#[derive(Default)]
/// JavaScriptCompiler is a struct that represents a JavaScript compiler instance.
///
/// It holds the global configuration and a reference to the source map.
/// You can use the JavaScript compiler to parse, transform, minify, and stringify JavaScript code.
///
/// Thanks to swc as lower tools, it is fast and efficient.
pub struct JavaScriptCompiler {
  globals: Globals,
  cm: Arc<SwcSourceMap>,
}

impl JavaScriptCompiler {
  pub fn new() -> Self {
    Self::default()
  }

  fn run<R>(&self, op: impl FnOnce() -> R) -> R {
    GLOBALS.set(&self.globals, op)
  }
}

#[derive(Debug)]
pub struct TransformOutput {
  /// The transformed code
  pub code: String,

  /// The source map for the transformed code
  pub map: Option<SourceMap>,

  /// The warning diagnostics for the transformed code
  pub diagnostics: Vec<String>,
}

impl TransformOutput {
  pub fn with_diagnostics(mut self, diagnostics: Vec<String>) -> Self {
    self.diagnostics = diagnostics;
    self
  }
}
