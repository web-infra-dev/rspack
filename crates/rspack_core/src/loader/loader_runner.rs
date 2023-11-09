use std::sync::Arc;

pub use rspack_loader_runner::{run_loaders, Content, Loader, LoaderContext};

use crate::{CompilerOptions, Context, ModuleIdentifier, ResolverFactory};

#[derive(Debug, Clone)]
pub struct CompilerContext {
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: Option<ModuleIdentifier>,
  pub context: Option<Box<Context>>,
}

pub type LoaderRunnerContext = CompilerContext;

pub type BoxLoader = Arc<dyn Loader<LoaderRunnerContext>>;
