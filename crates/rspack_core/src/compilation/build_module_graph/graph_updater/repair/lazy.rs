use super::{TaskContext, process_dependencies::ProcessDependenciesTask};
use crate::{
  DependencyRef, ModuleIdentifier, compilation::build_module_graph::ModuleToLazyMake,
  module_graph::ModuleGraph, utils::task_loop::Task,
};

pub fn request_lazy_dependencies(
  module_to_lazy_make: &mut ModuleToLazyMake,
  module_graph: &mut ModuleGraph,
  module_identifier: ModuleIdentifier,
  side_effect_free: bool,
  dependencies: &[DependencyRef],
) -> Vec<Box<dyn Task<TaskContext>>> {
  module_to_lazy_make
    .request(
      module_identifier,
      side_effect_free,
      dependencies,
      module_graph,
    )
    .into_iter()
    .map(|(original_module_identifier, dependencies)| {
      Box::new(ProcessDependenciesTask {
        dependencies,
        original_module_identifier,
        from_unlazy: true,
      }) as Box<dyn Task<TaskContext>>
    })
    .collect()
}
