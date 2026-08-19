use std::sync::Arc;

pub use rspack_loader_runner::{
  Content, Loader, LoaderContext, LoaderRunnerOptions, run_loaders, run_loaders_with_options,
};
use rspack_util::source_map::SourceMapKind;

use crate::{
  CacheFacade, CompilationId, CompilerId, CompilerOptions, NormalModule, ResolverFactory,
};

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub loader_cache: CacheFacade,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
