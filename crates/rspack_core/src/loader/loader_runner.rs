use std::sync::Arc;

pub use rspack_loader_runner::{Content, Loader, LoaderContext, run_loaders};
use rspack_util::source_map::SourceMapKind;

use crate::{BuildInfo, CompilationId, CompilerId, CompilerOptions, NormalModule, ResolverFactory};

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

impl RunnerContext {
  pub fn build_info(&self) -> &BuildInfo {
    self.module.build_info()
  }

  pub fn build_info_mut(&mut self) -> &mut BuildInfo {
    self.module.build_info_mut()
  }
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
