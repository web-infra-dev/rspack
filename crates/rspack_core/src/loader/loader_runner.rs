use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
pub use rspack_loader_runner::{Content, Loader, LoaderContext, run_loaders};
use rspack_util::source_map::SourceMapKind;

use crate::{
  CompilationId, CompilerId, CompilerOptions, NormalModule, ResolverFactory,
  loader::LoaderCacheService,
};

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
  pub(crate) loader_cache: Arc<LoaderCacheService>,
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
