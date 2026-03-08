use super::process_dependencies::ProcessDependenciesTask;
use crate::{
  DependencyId, ModuleIdentifier,
  compilation::build_module_graph::{ForwardedIdSet, ModuleToLazyMake},
  module_graph::ModuleGraph,
};

pub fn process_unlazy_dependencies(
  module_to_lazy_make: &ModuleToLazyMake,
  module_graph: &mut ModuleGraph,
  forwarded_ids: ForwardedIdSet,
  original_module_identifier: ModuleIdentifier,
) -> Option<ProcessDependenciesTask> {
  let lazy_dependencies = module_to_lazy_make
    .get_lazy_dependencies(&original_module_identifier)
    .expect("only module has lazy dependencies should run into process_unlazy_dependencies");

  let dependencies_to_process: Vec<DependencyId> = lazy_dependencies
    .requested_lazy_dependencies(&forwarded_ids)
    .into_iter()
    .filter(|dep| module_graph.dependency_by_id_mut(dep).unset_lazy())
    .collect();

  if dependencies_to_process.is_empty() {
    return None;
  }

  Some(ProcessDependenciesTask {
    dependencies: dependencies_to_process,
    original_module_identifier,
    from_unlazy: true,
  })
}
