use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use super::super::MakeArtifact;
use crate::{AsyncDependenciesBlockIdentifier, GroupOptions, ModuleGraph, ModuleIdentifier};

#[derive(Debug, Default, Eq, PartialEq)]
struct ModuleDeps {
  // child module identifier of current module
  child_modules: Vec<ModuleIdentifier>,
  // blocks in current module
  module_blocks: Vec<(AsyncDependenciesBlockIdentifier, Option<GroupOptions>)>,
}

impl ModuleDeps {
  fn from_module(module_graph: &ModuleGraph, module_identifier: &ModuleIdentifier) -> Self {
    let mut res = Self::default();
    let module = module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module");

    let deps = module
      .get_dependencies()
      .iter()
      .filter_map(|dep_id| module_graph.connection_by_dependency(dep_id))
      .map(|conn| *conn.module_identifier())
      .collect::<Vec<_>>();

    res.child_modules = remove_dup(deps);
    for block_id in module.get_blocks() {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");
      res
        .module_blocks
        .push((*block_id, block.get_group_options().cloned()));
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

fn remove_dup<T: Copy + std::cmp::Eq + std::hash::Hash>(items: Vec<T>) -> Vec<T> {
  let mut new_items = vec![];
  let mut s = HashSet::default();

  for item in items {
    if !s.contains(&item) {
      new_items.push(item);
      s.insert(item);
    }
  }

  new_items
}
