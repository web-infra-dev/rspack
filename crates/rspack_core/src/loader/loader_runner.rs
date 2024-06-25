use std::sync::Arc;

use rspack_loader_runner::ResourceData;
pub use rspack_loader_runner::{run_loaders, Content, Loader, LoaderContext};
use rspack_util::source_map::SourceMapKind;

use crate::{CompilerOptions, Context, FactoryMeta, Module, ModuleIdentifier, ResolverFactory};

#[derive(Debug, Clone)]
pub struct CompilerModuleContext {
  pub context: Option<Box<Context>>,
  pub resource: Option<ResourceData>,
  pub module_identifier: ModuleIdentifier,
  pub name_for_condition: Option<String>,
  pub request: Option<String>,
  pub user_request: Option<String>,
  pub raw_request: Option<String>,
  pub factory_meta: Option<FactoryMeta>,
}

impl CompilerModuleContext {
  pub fn from_module(module: &dyn Module) -> Self {
    let normal_module = module.as_normal_module();
    Self {
      context: module.get_context(),
      resource: normal_module.map(|normal_module| normal_module.resource_resolved_data().clone()),
      module_identifier: module.identifier(),
      name_for_condition: module.name_for_condition().map(|s| s.to_string()),
      request: normal_module.map(|normal_module| normal_module.request().to_owned()),
      user_request: normal_module.map(|normal_module| normal_module.user_request().to_owned()),
      raw_request: normal_module.map(|normal_module| normal_module.raw_request().to_owned()),
      factory_meta: normal_module
        .and_then(|normal_module| normal_module.factory_meta())
        .map(|factory_meta| factory_meta.to_owned()),
    }
  }
}

#[derive(Debug, Clone)]
pub struct RunnerContext {
  pub options: Arc<CompilerOptions>,
  pub resolver_factory: Arc<ResolverFactory>,
  pub module: CompilerModuleContext,
  pub module_source_map_kind: SourceMapKind,
}

pub type BoxLoader = Arc<dyn Loader<RunnerContext>>;
