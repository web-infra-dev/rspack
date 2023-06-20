use std::path::PathBuf;

use rustc_hash::FxHashSet as HashSet;

use crate::{BuildDependency, DependencyId, ModuleIdentifier};

mod rebuild_deps_builder;
pub use rebuild_deps_builder::RebuildDepsBuilder;

#[derive(Debug)]
pub enum MakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

impl MakeParam {
  pub fn add_force_build_dependency(
    &mut self,
    dep: DependencyId,
    module: Option<ModuleIdentifier>,
  ) -> bool {
    match self {
      MakeParam::ForceBuildDeps(set) => set.insert((dep, module)),
      _ => false,
    }
  }
}

// TODO @jerrykingxyz migrate make method here
// pub async fn updateModuleGraph(params: Vec<MakeParam>, module_graph: &mut ModuleGraph) {
// }
