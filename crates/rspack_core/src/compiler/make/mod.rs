use std::path::PathBuf;

use rustc_hash::FxHashSet as HashSet;

use crate::{BuildDependency, ModuleIdentifier};

mod rebuild_deps_builder;
pub use rebuild_deps_builder::RebuildDepsBuilder;

#[derive(Debug)]
pub enum MakeParam {
  ModifiedFiles(HashSet<PathBuf>),
  ForceBuildDeps(HashSet<BuildDependency>),
  ForceBuildModules(HashSet<ModuleIdentifier>),
}

// TODO @jerrykingxyz migrate make method here
// pub async fn updateModuleGraph(params: Vec<MakeParam>, module_graph: &mut ModuleGraph) {
// }
