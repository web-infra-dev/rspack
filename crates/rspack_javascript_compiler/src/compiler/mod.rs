use std::sync::Arc;

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
  pub code: String,
  pub map: Option<SourceMap>,
}
