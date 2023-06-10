use rustc_hash::FxHashSet as HashSet;

use super::MakeParam;
use crate::{BuildDependency, ModuleGraph, ModuleIdentifier};

#[derive(Debug, Default)]
pub struct RebuildDepsBuilder {
  /// the modules that need to be built
  force_build_modules: HashSet<ModuleIdentifier>,
  /// the deps that need to be built
  force_build_deps: HashSet<BuildDependency>,
}

impl RebuildDepsBuilder {
  pub fn new(params: Vec<MakeParam>, module_graph: &ModuleGraph) -> Self {
    let mut builder = Self::default();

    for item in params {
      match item {
        MakeParam::ModifiedFiles(files) => {
          builder.extend_force_build_modules(module_graph.modules().values().filter_map(|module| {
            // check has dependencies modified
            if module_graph.has_dependencies(&module.identifier(), &files) {
              Some(module.identifier())
            } else {
              None
            }
          }))
        }
        MakeParam::ForceBuildDeps(deps) => {
          builder.extend_force_build_deps(module_graph, deps);
        }
        MakeParam::ForceBuildModules(modules) => {
          builder.extend_force_build_modules(modules);
        }
      };
    }

    builder
  }

  pub fn extend_force_build_modules<I: IntoIterator<Item = ModuleIdentifier>>(
    &mut self,
    modules: I,
  ) {
    self.force_build_modules.extend(modules);
  }

  pub fn extend_force_build_deps<I: IntoIterator<Item = BuildDependency>>(
    &mut self,
    module_graph: &ModuleGraph,
    deps: I,
  ) {
    for item in deps {
      let (dependency_id, _) = &item;
      // add deps bindings module to force_build_modules
      if let Some(mid) = module_graph.module_identifier_by_dependency_id(dependency_id) {
        self.force_build_modules.insert(*mid);
      }
      self.force_build_deps.insert(item);
    }
  }

  pub fn get_force_build_modules(&self) -> &HashSet<ModuleIdentifier> {
    &self.force_build_modules
  }

  //  pub fn get_force_build_deps(&self) -> &HashSet<BuildDependency> {
  //    &self.force_build_deps
  //  }

  pub fn revoke_modules(mut self, module_graph: &mut ModuleGraph) -> HashSet<BuildDependency> {
    self.force_build_deps.extend(
      self
        .force_build_modules
        .iter()
        .flat_map(|id| module_graph.revoke_module(id)),
    );
    self.force_build_deps
  }
}
