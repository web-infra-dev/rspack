use std::sync::Mutex;

use rspack_core::JavaScriptExecutionMode;
use rustc_hash::{FxHashMap, FxHashSet};

/// Per-compiler JavaScript loader metadata, independent of JS object lifetimes.
#[derive(Debug)]
pub struct JsLoaderRuntime {
  parallel_loaders: Mutex<FxHashMap<String, JavaScriptExecutionMode>>,
  pub(super) loaders_without_pitch: Mutex<FxHashSet<String>>,
}

impl JsLoaderRuntime {
  pub fn new() -> Self {
    Self {
      parallel_loaders: Default::default(),
      loaders_without_pitch: Default::default(),
    }
  }

  pub fn register_loader(&self, ident: String, parallel: bool, strict: bool) {
    self
      .parallel_loaders
      .lock()
      .expect("loader references lock poisoned")
      .insert(
        ident,
        if !parallel {
          JavaScriptExecutionMode::Main
        } else if strict {
          JavaScriptExecutionMode::Strict
        } else {
          JavaScriptExecutionMode::Parallel
        },
      );
  }

  pub(super) fn execution_mode(&self, query: Option<&str>) -> JavaScriptExecutionMode {
    let Some(ident) = query.and_then(|query| query.strip_prefix("??")) else {
      return JavaScriptExecutionMode::Main;
    };
    self
      .parallel_loaders
      .lock()
      .expect("loader references lock poisoned")
      .get(ident)
      .copied()
      .unwrap_or_default()
  }
}
