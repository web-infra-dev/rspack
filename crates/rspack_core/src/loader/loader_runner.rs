use std::sync::Arc;

pub use rspack_loader_runner::{
  Content, Loader, LoaderChain, LoaderChainCacheAction, LoaderChainCacheState, LoaderChainLocation,
  LoaderChainStrategy, LoaderContext, LoaderExecutionKind, LoaderItem, LoaderItemState,
  LoaderRunnerOptions, create_loader_items, plan_loader_chains, run_loaders,
  run_loaders_with_options, run_loaders_with_options_and_strategy, run_loaders_with_preplanned,
};
use rspack_util::source_map::SourceMapKind;

use crate::{CompilationId, CompilerId, CompilerOptions, NormalModule, ResolverFactory};

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub fs: Arc<dyn rspack_fs::ReadableFileSystem>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
