use std::path::PathBuf;

use rustc_hash::FxHashSet as HashSet;

use crate::{BuildDependency, DependencyId, ModuleIdentifier};

mod rebuild_deps_builder;
pub use rebuild_deps_builder::RebuildDepsBuilder;

#[derive(Debug)]
pub enum MakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  DeletedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

impl MakeParam {
  pub fn new_force_build_dep_param(dep: DependencyId, module: Option<ModuleIdentifier>) -> Self {
    let mut data = HashSet::default();
    data.insert((dep, module));
    Self::ForceBuildDeps(data)
  }
}

// TODO @jerrykingxyz migrate make method here
// pub async fn updateModuleGraph(params: Vec<MakeParam>, module_graph: &mut ModuleGraph) {
// }
