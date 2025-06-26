mod artifact;
mod graph_updater;
mod module_executor;

use rspack_error::Result;

pub use self::{
  artifact::{MakeArtifact, MakeArtifactState},
  graph_updater::{update_module_graph, UpdateParam},
  module_executor::{ExecuteModuleId, ExecutedRuntimeModule, ModuleExecutor},
};
use crate::Compilation;

pub async fn make(compilation: &Compilation, mut artifact: MakeArtifact) -> Result<MakeArtifact> {
  let mut params = Vec::with_capacity(6);

  if !compilation.entries.is_empty() {
    params.push(UpdateParam::BuildEntry(
      compilation
        .entries
        .values()
        .flat_map(|item| item.all_dependencies())
        .chain(compilation.global_entry.all_dependencies())
        .copied()
        .collect(),
    ));
  }
  params.push(UpdateParam::CheckNeedBuild);
  if !compilation.modified_files.is_empty() {
    params.push(UpdateParam::ModifiedFiles(
      compilation.modified_files.clone(),
    ));
  }
  if !compilation.removed_files.is_empty() {
    params.push(UpdateParam::RemovedFiles(compilation.removed_files.clone()));
  }
  if let MakeArtifactState::Uninitialized(force_build_deps) = &artifact.state {
    params.push(UpdateParam::ForceBuildDeps(force_build_deps.clone()));
  }

  // reset temporary data
  artifact.reset_temporary_data();
  artifact = update_module_graph(compilation, artifact, params).await?;
  Ok(artifact)
}
