use std::sync::Arc;

pub use rspack_loader_runner::{run_loaders, Content, Loader, LoaderContext};
use rspack_util::source_map::SourceMapKind;
use rustc_hash::FxHashSet;

use crate::{
  cache::Cache, Compilation, CompilerOptions, Context, ModuleIdentifier, QueueHandler,
  ResolverFactory, SharedPluginDriver,
};

#[derive(Debug, Clone)]
pub struct CompilerContext {
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: ModuleIdentifier,             // current module
  pub module_context: Option<Box<Context>>, // current module context
  pub module_source_map_kind: SourceMapKind,

  pub queue_handler: Option<QueueHandler>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
}

#[derive(Debug, Default)]
pub struct ExecuteModuleResult {
  pub file_dependencies: FxHashSet<std::path::PathBuf>,
  pub context_dependencies: FxHashSet<std::path::PathBuf>,
  pub missing_dependencies: FxHashSet<std::path::PathBuf>,
  pub build_dependencies: FxHashSet<std::path::PathBuf>,
  pub assets: FxHashSet<String>,
  pub id: u32,
}

impl CompilerContext {
  pub async fn import_module(
    &self,
    request: String,
    public_path: Option<String>,
    base_uri: Option<String>,
  ) -> rspack_error::Result<ExecuteModuleResult> {
    if self.queue_handler.is_none() {
      return Err(rspack_error::error!(
        "use import_module without queue_handler"
      ));
    }

    Compilation::import_module_impl(
      self.queue_handler.clone().expect("unreachable"),
      self.resolver_factory.clone(),
      self.options.clone(),
      self.plugin_driver.clone(),
      self.cache.clone(),
      request,
      public_path,
      base_uri,
      Some(self.module),
      self.module_context.clone(),
    )
    .await
  }
}

pub type LoaderRunnerContext = CompilerContext;

pub type BoxLoader = Arc<dyn Loader<LoaderRunnerContext>>;
