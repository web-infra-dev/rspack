use std::sync::Mutex;

use rustc_hash::{FxHashMap, FxHashSet};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum ExecutionMode {
  #[default]
  Main,
  Parallel,
  Strict,
}

/// Per-compiler JavaScript loader metadata, independent of JS object lifetimes.
#[derive(Debug)]
pub struct JsLoaderRuntime {
  pub(super) main_object_handle: u32,
  parallel_loaders: Mutex<FxHashMap<String, ExecutionMode>>,
  pub(super) loaders_without_pitch: Mutex<FxHashSet<String>>,
}

impl JsLoaderRuntime {
  pub fn new(main_object_handle: u32) -> Self {
    Self {
      main_object_handle,
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
          ExecutionMode::Main
        } else if strict {
          ExecutionMode::Strict
        } else {
          ExecutionMode::Parallel
        },
      );
  }

  pub(super) fn execution_mode(&self, query: Option<&str>) -> ExecutionMode {
    let Some(ident) = query.and_then(|query| query.strip_prefix("??")) else {
      return ExecutionMode::Main;
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
