use std::sync::Arc;

use rspack_fs::ReadableFileSystem;
pub use rspack_loader_runner::{
  Content, Loader, LoaderContext, LoaderDependencies, LoaderRunnerOptions, run_loaders,
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
  pub fs: Arc<dyn ReadableFileSystem>,
  pub loader_cache: CacheFacade,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
