use std::sync::Arc;

use rspack_error::{internal_error, Diagnostic, Error, Result};

use crate::{
  cache::Cache, BoxModule, CompilerOptions, Dependency, LoaderRunnerRunner, Module, ModuleGraph,
  ModuleGraphModule, ModuleIdentifier, ModuleType, NormalModuleFactory, NormalModuleFactoryContext,
  Resolve, SharedPluginDriver, WorkerQueue,
};

pub enum TaskResult {
  Factorize(FactorizeTaskResult),
  Add(AddTaskResult),
  Build(BuildTaskResult),
  ProcessDependencies(ProcessDependenciesResult),
}

#[async_trait::async_trait]
pub trait WorkerTask {
  async fn run(self) -> Result<TaskResult>;
}

pub trait WorkerTaskSync {
  fn run(self) -> Result<TaskResult>;
}

pub struct FactorizeTask {
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

pub struct FactorizeTaskResult {
  pub original_module_identifier: Option<ModuleIdentifier>,
  pub module: Box<dyn Module>,
  pub module_graph_module: Box<ModuleGraphModule>,
  pub dependencies: Vec<Dependency>,
}

#[async_trait::async_trait]
impl WorkerTask for FactorizeTask {
  async fn run(self) -> Result<TaskResult> {
    let factory = NormalModuleFactory::new(
      NormalModuleFactoryContext {
        module_name: self.module_name,
        module_type: self.module_type,
        side_effects: self.side_effects,
        options: self.options,
        lazy_visit_modules: self.lazy_visit_modules,
      },
      self.dependencies[0].clone(),
      self.plugin_driver,
      self.cache,
    );

    let (module, context, dependency) = factory.create(self.is_entry, self.resolve_options).await?;
    let mgm = ModuleGraphModule::new(
      context.module_name.clone(),
      module.identifier(),
      self.dependencies.clone(),
      context.module_type.ok_or_else(|| {
        Error::InternalError(internal_error!(format!(
          "Unable to get the module type for module {}, did you forget to configure `Rule.type`? ",
          module.identifier()
        )))
      })?,
      !context.options.builtins.side_effects,
    );

    Ok(TaskResult::Factorize(FactorizeTaskResult {
      original_module_identifier: self.original_module_identifier,
      module,
      module_graph_module: Box::new(mgm),
      dependencies: self.dependencies,
    }))
  }
}

pub type FactorizeQueue = WorkerQueue<FactorizeTask>;

pub struct AddTask<'m> {
  original_module_identifier: Option<ModuleIdentifier>,
  module: Box<dyn Module>,
  module_graph_module: Box<ModuleGraphModule>,
  dependencies: Vec<Dependency>,

  module_graph: &'m mut ModuleGraph,
}

pub enum AddTaskResult {
  ModuleReused(Box<dyn Module>),
  ModuleAdded(Box<dyn Module>),
}

impl WorkerTaskSync for AddTask<'_> {
  fn run(self) -> Result<TaskResult> {
    let module_identifier = self.module.identifier();

    if self.module_graph.module_exists(&self.module.identifier()) {
      self.set_resolved_module(module_identifier);

      return Ok(TaskResult::Add(AddTaskResult::ModuleReused(self.module)));
    }

    self
      .module_graph
      .add_module_graph_module(*self.module_graph_module);

    self.set_resolved_module(module_identifier);

    Ok(TaskResult::Add(AddTaskResult::ModuleAdded(self.module)))
  }
}

impl AddTask<'_> {
  fn set_resolved_module(&mut self, module_identifier: ModuleIdentifier) {
    for dependency in &self.dependencies {
      let dep_id = self
        .module_graph
        .add_dependency(dependency, module_identifier);

      self.module_graph.set_resolved_module(
        self.original_module_identifier,
        dep_id,
        module_identifier,
      );
    }
  }
}

pub type AddQueue<'m> = WorkerQueue<AddTask<'m>>;

pub struct BuildTask {
  pub module: Box<dyn Module>,

  pub loader_runner_runner: Arc<LoaderRunnerRunner>,
  pub compiler_options: Arc<CompilerOptions>,
  pub cache: Arc<Cache>,
}

pub struct BuildTaskResult {
  pub module: Box<dyn Module>,
}

#[async_trait::async_trait]
impl WorkerTask for BuildTask {
  async fn run(self) -> Result<TaskResult> {
    self.module.build();
    Ok(TaskResult::Build(BuildTaskResult {
      module: self.module,
    }))
  }
}

pub type BuildQueue = WorkerQueue<BuildTask>;

pub struct ProcessDependenciesTask {}

pub struct ProcessDependenciesResult {}

pub type ProcessDependenciesQueue = WorkerQueue<ProcessDependenciesTask>;
