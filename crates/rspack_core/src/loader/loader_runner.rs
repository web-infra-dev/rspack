use std::{ptr::NonNull, sync::Arc};

use rspack_fs::ReadableFileSystem;
pub use rspack_loader_runner::{
  Content, Loader, LoaderContext, LoaderDependencies, LoaderRunnerOptions, run_loaders,
};
use rspack_util::source_map::SourceMapKind;

use crate::{
  CacheFacade, Compilation, CompilationId, CompilerId, CompilerOptions, FileSystemInfo,
  NormalModule, ResolverFactory,
};

/// Non-owning access to the Compilation for the duration of one loader run.
#[derive(Debug, Clone, Copy)]
pub struct LoaderCompilation(NonNull<Compilation>);

// SAFETY: NormalModule::build awaits the loader runner before the active Compilation can move or
// be dropped.
unsafe impl Send for LoaderCompilation {}
unsafe impl Sync for LoaderCompilation {}

impl LoaderCompilation {
  pub fn new(compilation: &Compilation) -> Self {
    Self(NonNull::from(compilation))
  }

  pub fn as_ref(&self) -> &Compilation {
    // SAFETY: upheld by the loader-run scope documented above.
    unsafe { self.0.as_ref() }
  }
}

#[derive(Debug)]
pub struct RunnerContext {
  pub compiler_id: CompilerId,
  pub compilation_id: CompilationId,
  pub compilation: LoaderCompilation,
  pub options: Arc<CompilerOptions>,
  pub fs: Arc<dyn ReadableFileSystem>,
  pub loader_cache: CacheFacade,
  pub file_system_info: FileSystemInfo,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Box<NormalModule>,
  pub source_map_kind: SourceMapKind,
}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
