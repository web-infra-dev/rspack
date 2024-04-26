use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::super::MakeArtifact;
use crate::{AsyncDependenciesBlockIdentifier, GroupOptions, ModuleGraph, ModuleIdentifier};

#[derive(Debug, Default, Eq, PartialEq)]
struct ModuleDeps {
  // child module identifier of current module
  child_modules: HashSet<ModuleIdentifier>,
  // blocks in current module
  module_blocks: HashSet<(AsyncDependenciesBlockIdentifier, Option<GroupOptions>)>,
}

impl ModuleDeps {
  fn from_module(module_graph: &ModuleGraph, module_identifier: &ModuleIdentifier) -> Self {
    let mut res = Self::default();
    for connection in module_graph.get_outgoing_connections(module_identifier) {
      res.child_modules.insert(*connection.module_identifier());
    }
    let module = module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module");
    for block_id in module.get_blocks() {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");
      res
        .module_blocks
        .insert((*block_id, block.get_group_options().cloned()));
    }

    res
  }
}

#[derive(Debug, Default)]
pub struct HasModuleGraphChange {
  origin_module_deps: HashMap<ModuleIdentifier, ModuleDeps>,
}

impl HasModuleGraphChange {
  pub fn analyze_force_build_module(
    &mut self,
    artifact: &MakeArtifact,
    module_identifier: &ModuleIdentifier,
  ) {
    let module_graph = &artifact.get_module_graph();
    self.origin_module_deps.insert(
      *module_identifier,
      ModuleDeps::from_module(module_graph, module_identifier),
    );
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let module_graph = &artifact.get_module_graph();
    if self.origin_module_deps.is_empty() {
      // origin_module_deps empty means no force_build_module and no file changed
      // this only happens when build from entry
      artifact.has_module_graph_change = true;
      return;
    }
    // if artifact.has_module_graph_change is true, no need to recalculate
    if !artifact.has_module_graph_change {
      for (module_identifier, module_deps) in self.origin_module_deps {
        if module_graph
          .module_by_identifier(&module_identifier)
          .is_none()
          || ModuleDeps::from_module(module_graph, &module_identifier) != module_deps
        {
          artifact.has_module_graph_change = true;
          return;
        }
      }
    }
  }
}
