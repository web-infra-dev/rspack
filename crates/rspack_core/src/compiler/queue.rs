use std::sync::Arc;

use rspack_error::{internal_error, Diagnostic, Error, Result};

use crate::{
  cache::Cache, CompilerOptions, Dependency, Module, ModuleGraphModule, ModuleIdentifier,
  ModuleType, NormalModuleFactory, NormalModuleFactoryContext, Resolve, SharedPluginDriver,
  WorkerQueue,
};

pub enum TaskResult {
  Factorize(FactorizeResult),
  Add(AddResult),
  Build(BuildResult),
}

#[async_trait::async_trait]
pub trait WorkerTask {
  async fn run(self) -> Result<TaskResult>;
}

pub struct FactorizeContext {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub dependencies: Vec<Dependency>,

  pub is_entry: bool,
  pub module_name: Option<String>,
  pub module_type: Option<ModuleType>,
  pub side_effects: Option<bool>,
  pub resolve_options: Option<Resolve>,
  pub options: Arc<CompilerOptions>,
  pub lazy_visit_modules: std::collections::HashSet<String>,
  pub plugin_driver: SharedPluginDriver,
  pub cache: Arc<Cache>,
}

pub struct FactorizeResult {
  module: Box<dyn Module>,
  module_graph_module: ModuleGraphModule,
}

#[async_trait::async_trait]
impl WorkerTask for FactorizeContext {
  async fn run(self) -> Result<TaskResult> {
    let factory = NormalModuleFactory::new(
      NormalModuleFactoryContext {
        module_name: self.module_name,
        module_type: self.module_type,
        side_effects: self.side_effects,
        options: self.options,
        lazy_visit_modules: self.lazy_visit_modules,
      },
      self.dependencies[0],
      self.plugin_driver,
      self.cache,
    );

    let (module, context) = factory.create(self.is_entry, self.resolve_options).await?;
    let mgm = ModuleGraphModule::new(
      context.module_name.clone(),
      module.identifier(),
      vec![],
      context.module_type.ok_or_else(|| {
        Error::InternalError(internal_error!(format!(
          "Unable to get the module type for module {}, did you forget to configure `Rule.type`? ",
          module.identifier()
        )))
      })?,
      !self.context.options.builtins.side_effects,
    );

    Ok(TaskResult::Factorize(FactorizeResult {
      module,
      module_graph_module: mgm,
    }))
  }
}

pub type FactorizeQueue = WorkerQueue<FactorizeContext>;

pub struct AddContext {
  module: Box<dyn Module>,
}

pub struct AddResult {}

#[async_trait::async_trait]
impl WorkerTask for AddContext {
  async fn run(self) -> Result<TaskResult> {
    Ok(TaskResult::Add(AddResult {}))
  }
}

pub type AddQueue = WorkerQueue<AddContext>;

pub struct BuildContext {}

pub struct BuildResult {}

#[async_trait::async_trait]
impl WorkerTask for BuildContext {
  async fn run(self) -> Result<TaskResult> {
    Ok(TaskResult::Build(BuildResult {}))
  }
}

pub type BuildQueue = WorkerQueue<BuildContext>;
