use super::{TaskContext, process_dependencies::ProcessDependenciesTask};
use crate::{
  DependencyId, ModuleIdentifier,
  compilation::build_module_graph::ForwardedIdSet,
  task_loop::{Task, TaskResult, TaskType},
};

#[derive(Debug)]
pub struct ProcessUnlazyDependenciesTask {
  pub forwarded_ids: ForwardedIdSet,
  pub original_module_identifier: ModuleIdentifier,
}

#[async_trait::async_trait]
impl Task<TaskContext> for ProcessUnlazyDependenciesTask {
  fn get_task_type(&self) -> TaskType {
    TaskType::Main
  }

  async fn main_run(self: Box<Self>, context: &mut TaskContext) -> TaskResult<TaskContext> {
    let module_graph =
      &mut TaskContext::get_module_graph_mut(&mut context.artifact.module_graph_partial);
    let ProcessUnlazyDependenciesTask {
      forwarded_ids,
      original_module_identifier,
    } = *self;

    let lazy_dependencies = context
      .artifact
      .module_to_lazy_make
      .get_lazy_dependencies(&original_module_identifier)
      .expect("only module has lazy dependencies should run into ProcessUnlazyDependenciesTask");
    let dependencies_to_process: Vec<DependencyId> = lazy_dependencies
      .requested_lazy_dependencies(&forwarded_ids)
      .into_iter()
      .filter(|dep| {
        let Some(dep) = module_graph.dependency_by_id_mut(dep) else {
          return false;
        };
        dep.unset_lazy()
      })
      .collect();
    if dependencies_to_process.is_empty() {
      return Ok(vec![]);
    }
    return Ok(vec![Box::new(ProcessDependenciesTask {
      dependencies: dependencies_to_process,
      original_module_identifier,
      from_unlazy: true,
    })]);
  }
}
