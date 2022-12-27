use rspack_error::{Diagnostic, Result};

use crate::{
  Dependency, Module, ModuleGraphModule, ModuleIdentifier, NormalModuleFactory, WorkerQueue,
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
  original_module_identifier: Option<ModuleIdentifier>,
  dependencies: Vec<Dependency>,
}

pub struct FactorizeResult {
  module: Box<dyn Module>,
  module_graph_module: ModuleGraphModule,
  dependencies: Vec<Dependency>,
  diagnostics: Vec<Diagnostic>,
}

#[async_trait::async_trait]
impl WorkerTask for FactorizeContext {
  async fn run(self) -> Result<TaskResult> {
    NormalModuleFactory::new();
    // let mgm = ModuleGraphModule::new(
    //   self.context.module_name.clone(),
    //   module.identifier(),
    //   vec![],
    //   self.context.module_type.ok_or_else(|| {
    //     Error::InternalError(internal_error!(format!(
    //       "Unable to get the module type for module {}, did you forget to configure `Rule.type`? ",
    //       module.identifier()
    //     )))
    //   })?,
    //   !self.context.options.builtins.side_effects,
    // );
    Ok(TaskResult::Factorize(FactorizeResult {
      module: todo!(),
      module_graph_module: todo!(),
      dependencies: vec![],
      diagnostics: vec![],
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
