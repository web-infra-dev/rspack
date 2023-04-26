use std::sync::Arc;

pub use rspack_loader_runner::{run_loaders, Content, Loader, LoaderContext, ResourceData};

use crate::{CompilerOptions, ResolverFactory};

#[derive(Debug)]
pub struct CompilerContext {
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
}

pub type LoaderRunnerContext = CompilerContext;

pub type BoxLoader = Arc<dyn Loader<LoaderRunnerContext>>;
