use std::{ptr::NonNull, sync::Arc};

pub use rspack_loader_runner::{run_loaders, Content, Loader, LoaderContext, LoaderContextId};
use rspack_util::source_map::SourceMapKind;

use crate::{CompilationId, CompilerOptions, Module, ResolverFactory};

#[derive(Debug, Clone)]
pub struct RunnerContext {
  pub compilation_id: CompilationId,
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: NonNull<dyn Module>,
  pub module_source_map_kind: SourceMapKind,
}

unsafe impl Send for RunnerContext {}

pub type BoxLoader = Arc<dyn for<'a> Loader<RunnerContext>>;
