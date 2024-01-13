use std::sync::Arc;

use rspack_common::SourceMapKind;
pub use rspack_loader_runner::{run_loaders, Content, Loader, LoaderContext};

use crate::{CompilerOptions, Context, ModuleIdentifier, ResolverFactory};

#[derive(Debug, Clone)]
pub struct CompilerContext {
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: ModuleIdentifier,             // current module
  pub module_context: Option<Box<Context>>, // current module context
  pub module_source_map_kind: SourceMapKind,
}

pub type LoaderRunnerContext = CompilerContext;

pub type BoxLoader = Arc<dyn Loader<LoaderRunnerContext>>;
