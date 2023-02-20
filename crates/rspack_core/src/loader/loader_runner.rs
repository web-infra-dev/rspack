use std::sync::Arc;

use rspack_error::{Result, TWithDiagnosticArray};
pub use rspack_loader_runner::{
  Content, Loader, LoaderContext, LoaderResult, LoaderRunner, LoaderRunnerAdditionalContext,
  ResourceData,
};

use crate::{
  CompilerOptions, LoaderRunnerPluginProcessResource, ResolverFactory, SharedPluginDriver,
};

#[derive(Debug)]
pub struct CompilerContext {
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
}

pub type CompilationContext = ();

pub type BoxLoader = rspack_loader_runner::BoxLoader<CompilerContext, CompilationContext>;

#[derive(Debug)]
pub struct LoaderRunnerRunner {
  pub options: Arc<CompilerOptions>,
  pub plugin_driver: SharedPluginDriver,
  pub compiler_context: CompilerContext,
}

impl LoaderRunnerRunner {
  pub fn new(
    options: Arc<CompilerOptions>,
    resolver_factory: Arc<ResolverFactory>,
    plugin_driver: SharedPluginDriver,
  ) -> Self {
    let compiler_context = CompilerContext {
      options: options.clone(),
      resolver_factory,
    };

    Self {
      options,
      plugin_driver,
      compiler_context,
    }
  }
  pub async fn run(
    &self,
    resource_data: ResourceData,
    loaders: impl IntoIterator<Item = &dyn Loader<CompilerContext, CompilationContext>>,
  ) -> Result<TWithDiagnosticArray<LoaderResult>> {
    LoaderRunner::new(
      resource_data,
      vec![Box::new(LoaderRunnerPluginProcessResource::new(
        self.plugin_driver.clone(),
      ))],
    )
    .run(
      loaders.into_iter().collect::<Vec<_>>(),
      &LoaderRunnerAdditionalContext {
        compiler: &self.compiler_context,
        compilation: &(),
      },
    )
    .await
  }
}
