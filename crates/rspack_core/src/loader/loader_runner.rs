use std::sync::Arc;

pub use rspack_loader_runner::{
  Content, Loader, LoaderChain, LoaderChainCacheAction, LoaderChainCacheState, LoaderContext,
  LoaderExecutionKind, LoaderRunnerOptions,
};
use rspack_util::source_map::SourceMapKind;

use crate::{
  CompilationId, CompilerId, CompilerOptions, LoaderCache, NormalModule, ResolverFactory,
};

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
  pub(crate) loader_cache: Arc<LoaderCache>,
}

pub type Loaders = rspack_loader_runner::Loaders<RunnerContext>;
pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
