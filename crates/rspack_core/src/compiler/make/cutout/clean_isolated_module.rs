use std::collections::VecDeque;

use rustc_hash::FxHashSet as HashSet;

use super::super::MakeArtifact;
use crate::{DependencyId, ModuleIdentifier};

#[derive(Debug, Default)]
pub struct CleanIsolatedModule {
  need_check_isolated_module_ids: HashSet<ModuleIdentifier>,
}

impl CleanIsolatedModule {
  pub fn analyze_force_build_module(
    &mut self,
    artifact: &MakeArtifact,
    module_identifier: &ModuleIdentifier,
  ) {
    let module_graph = artifact.get_module_graph();
    for connection in module_graph.get_outgoing_connections(module_identifier) {
      self
        .need_check_isolated_module_ids
        .insert(*connection.module_identifier());
    }
  }

  pub fn analyze_removed_deps(&mut self, artifact: &MakeArtifact, dep_id: &DependencyId) {
    let module_graph = artifact.get_module_graph();
    let connection = module_graph
      .connection_by_dependency(dep_id)
      .expect("should have connection");
    self
      .need_check_isolated_module_ids
      .insert(*connection.module_identifier());
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let module_graph = artifact.get_module_graph_mut();
    let mut need_remove_modules = HashSet::default();
    let mut queue = VecDeque::from(
      self
        .need_check_isolated_module_ids
        .into_iter()
        .collect::<Vec<_>>(),
    );
    while let Some(module_identifier) = queue.pop_front() {
      let Some(mgm) = module_graph.module_graph_module_by_identifier(&module_identifier) else {
        tracing::trace!("Module is cleaned: {}", module_identifier);
        continue;
      };
      if !mgm.incoming_connections().is_empty() {
        tracing::trace!("Module is used: {}", module_identifier);
        continue;
      }

      for connection in module_graph.get_outgoing_connections(&module_identifier) {
        // clean child module
        queue.push_back(*connection.module_identifier());
      }
      tracing::trace!("Module is cleaned: {}", module_identifier);
      need_remove_modules.insert(module_identifier);
    }
    artifact.revoke_modules(need_remove_modules);
  }
}
