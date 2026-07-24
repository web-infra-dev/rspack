use std::sync::Arc;

pub use rspack_loader_runner::{Content, Loader, LoaderContext, run_loaders_with_cache};
use rspack_util::source_map::SourceMapKind;

use crate::{
  CompilationId, CompilerId, CompilerOptions, LoaderCacheContext, NormalModule, ResolverFactory,
};

#[derive(Debug)]
pub struct RunnerContext {
  pub loader_cache: LoaderCacheContext,
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
