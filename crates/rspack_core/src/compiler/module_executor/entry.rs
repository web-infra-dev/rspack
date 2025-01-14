use crate::{
  compiler::make::repair::{factorize::FactorizeTask, MakeTaskContext},
  utils::task_loop::{Task, TaskResult, TaskType},
  Dependency, LoaderImportDependency, ModuleProfile,
};

#[derive(Debug)]
pub struct EntryTask {
  pub dep: Box<LoaderImportDependency>,
  pub layer: Option<String>,
}
#[async_trait::async_trait]
impl Task<MakeTaskContext> for EntryTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Sync
  }

  async fn main_run(self: Box<Self>, context: &mut MakeTaskContext) -> TaskResult<MakeTaskContext> {
    let Self { dep, layer } = *self;
    let mut module_graph =
      MakeTaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);

    module_graph.add_dependency(dep.clone());
    Ok(vec![Box::new(FactorizeTask {
      compiler_id: context.compiler_id,
      compilation_id: context.compilation_id,
      module_factory: context
        .dependency_factories
        .get(dep.dependency_type())
        .unwrap_or_else(|| {
          panic!(
            "should have dependency_factories for dependency_type: {}",
            dep.dependency_type()
          )
        })
        .clone(),
      original_module_identifier: None,
      original_module_source: None,
      issuer: None,
      issuer_layer: layer,
      original_module_context: None,
      dependencies: vec![dep],
      resolve_options: None,
      options: context.compiler_options.clone(),
      current_profile: context
        .compiler_options
        .profile
        .then(Box::<ModuleProfile>::default),
      resolver_factory: context.resolver_factory.clone(),
    })])
  }
}
