use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
use rspack_loader_runner::LoaderRunnerContext;
pub use rspack_loader_runner::{
  Content, Loader, LoaderContext, LoaderDependencies, LoaderExecutionKind, LoaderRunnerOptions,
};
use rspack_util::source_map::SourceMapKind;

use crate::{
  CacheFacade, CompilationId, CompilerId, CompilerOptions, FileSystemInfo, NormalModule,
  ResolverFactory,
};

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub loader_cache: CacheFacade,
  pub file_system_info: FileSystemInfo,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

impl LoaderRunnerContext for RunnerContext {
  fn loaders(&self) -> &Loaders {
    &self.module.loaders
  }
}

pub type Loaders = rspack_loader_runner::Loaders<RunnerContext>;
pub type ResolvedLoader = rspack_loader_runner::ResolvedLoader<RunnerContext>;
pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
