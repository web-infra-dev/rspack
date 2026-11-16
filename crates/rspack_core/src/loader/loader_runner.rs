use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
pub use rspack_loader_runner::{
  CodeFilter, Content, Loader, LoaderContext, LoaderDependencies, LoaderExecutionKind,
  LoaderFilter, LoaderRunnerOptions, run_loaders,
};
use rspack_util::source_map::SourceMapKind;

use crate::{
  CompilationId, CompilerId, CompilerOptions, FileSystemInfo, LoaderCache, NormalModule,
  ResolverFactory,
};

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub(crate) loader_cache: Option<Arc<LoaderCache>>,
  pub(crate) loader_cache_counters: Arc<crate::CacheCounters>,
  pub file_system_info: FileSystemInfo,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
