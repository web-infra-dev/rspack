use crate::{Chunk, ChunkGraph, Context, ModuleGraph};

#[derive(Debug, Default)]
pub struct PluginContext<T = ()> {
  pub context: T,
}

impl PluginContext {
  pub fn new() -> Self {
    Self::with_context(())
  }
}

impl<T> PluginContext<T> {
  pub fn with_context(context: T) -> Self {
    Self { context }
  }

  pub fn into_context(self) -> T {
    self.context
  }
}

pub struct RenderModulePackageContext<'a> {
  pub chunk: &'a Chunk,
  pub context: &'a Context,
  pub module_graph: &'a ModuleGraph<'a>,
  pub chunk_graph: &'a ChunkGraph,
}
