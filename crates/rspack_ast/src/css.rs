use std::{fmt, hash::Hash, sync::Arc};

use swc_core::common::SourceMap;
use swc_core::css::ast::Stylesheet as SwcStylesheet;

pub struct Context {
  pub source_map: Arc<SourceMap>,
}

impl Context {
  pub fn new(source_map: Arc<SourceMap>) -> Self {
    Self { source_map }
  }
}

impl fmt::Debug for Context {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Context").finish()
  }
}

#[derive(Debug, Clone)]
pub struct Ast {
  root: SwcStylesheet,
  context: Arc<Context>,
}

impl Hash for Ast {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.root.hash(state);
  }
}

impl Ast {
  pub fn new(root: SwcStylesheet, source_map: Arc<SourceMap>) -> Self {
    Self {
      root,
      context: Arc::new(Context::new(source_map)),
    }
  }

  pub fn get_context(&self) -> &Context {
    &self.context
  }

  pub fn get_root(&self) -> &SwcStylesheet {
    &self.root
  }

  pub fn get_root_mut(&mut self) -> &mut SwcStylesheet {
    &mut self.root
  }
}
