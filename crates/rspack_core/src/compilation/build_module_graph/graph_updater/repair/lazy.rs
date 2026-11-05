use rustc_hash::FxHashSet;

use super::process_dependencies::ProcessDependenciesTask;
use crate::{
  DependencyId, DependencyRef, ModuleIdentifier,
  compilation::build_module_graph::{ForwardId, ForwardedIdSet, ModuleToLazyMake},
  module_graph::ModuleGraph,
};

pub fn collect_forwarded_ids(
  module_to_lazy_make: &mut ModuleToLazyMake,
  module_graph: &ModuleGraph,
  module_identifier: ModuleIdentifier,
  dependencies: &[DependencyRef],
) -> ForwardedIdSet {
  let eager_on_local_reference = module_to_lazy_make.eager_on_local_reference;
  let mut ids = FxHashSet::default();
  for dependency in dependencies {
    match dependency.forward_id() {
      ForwardId::All => return ForwardedIdSet::All,
      ForwardId::Id(id) => {
        ids.insert(id);
      }
      ForwardId::Empty => {}
      ForwardId::StarReexport => {
        let Some(parent) = module_graph.get_parent_module(dependency.id()) else {
          return ForwardedIdSet::All;
        };
        let Some(barrel) = module_to_lazy_make.get_lazy_dependencies_mut(parent) else {
          return ForwardedIdSet::All;
        };
        let star_ids = barrel.star_reexport_ids(module_identifier, eager_on_local_reference);
        module_to_lazy_make.mark_updated(*parent);
        match star_ids {
          ForwardedIdSet::All => return ForwardedIdSet::All,
          ForwardedIdSet::IdSet(star_ids) => ids.extend(star_ids),
        }
      }
    }
  }
  ForwardedIdSet::IdSet(ids)
}

pub fn request_lazy_dependencies(
  module_to_lazy_make: &mut ModuleToLazyMake,
  module_graph: &ModuleGraph,
  module_identifier: ModuleIdentifier,
  forwarded_ids: ForwardedIdSet,
) -> Vec<ProcessDependenciesTask> {
  let eager_on_local_reference = module_to_lazy_make.eager_on_local_reference;
  let mut tasks = vec![];
  let mut queue = vec![(module_identifier, forwarded_ids)];
  while let Some((module_identifier, forwarded_ids)) = queue.pop() {
    if forwarded_ids.is_empty() {
      continue;
    }
    if module_graph
      .module_by_identifier(&module_identifier)
      .is_none()
    {
      if module_graph
        .module_graph_module_by_identifier(&module_identifier)
        .is_some()
      {
        module_to_lazy_make
          .pending_forwarded_ids(module_identifier)
          .append(forwarded_ids);
      }
      continue;
    }
    let Some(lazy_dependencies) = module_to_lazy_make.get_lazy_dependencies_mut(&module_identifier)
    else {
      continue;
    };
    let Some(new_ids) = lazy_dependencies.merge_requested_ids(forwarded_ids) else {
      continue;
    };
    let (dependencies, fallback_ids) =
      lazy_dependencies.requested_lazy_dependencies(&new_ids, eager_on_local_reference);
    if let Some(fallback_ids) = fallback_ids {
      queue.extend(
        lazy_dependencies
          .fallback_targets()
          .iter()
          .map(|target| (*target, fallback_ids.clone())),
      );
    }
    module_to_lazy_make.mark_updated(module_identifier);
    let dependencies: Vec<DependencyId> = dependencies
      .into_iter()
      .filter(|dep| module_graph.dependency_by_id(dep).unset_lazy())
      .collect();
    if !dependencies.is_empty() {
      tasks.push(ProcessDependenciesTask {
        dependencies,
        original_module_identifier: module_identifier,
        from_unlazy: true,
      });
    }
  }
  tasks
}
