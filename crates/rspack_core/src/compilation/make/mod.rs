mod artifact;
mod graph_updater;
mod module_executor;

use rspack_error::Result;

pub use self::{
  artifact::{MakeArtifact, MakeArtifactState},
  graph_updater::{UpdateParam, update_module_graph},
  module_executor::{ExecuteModuleId, ExecutedRuntimeModule, ModuleExecutor},
};
use crate::Compilation;

/// make module graph, support incremental rebuild
///
/// The main method for updating the module graph in the make phase,
/// it will use entries, modified_files, removed_files to update the module graph.
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
  // MakeArtifactState::Uninitialized set by persistent cache.
  if let MakeArtifactState::Uninitialized(force_build_deps, isolated_modules) = &artifact.state {
    params.push(UpdateParam::ForceBuildDeps(force_build_deps.clone()));
    params.push(UpdateParam::CheckIsolatedModules(isolated_modules.clone()));
  }

  // reset temporary data
  artifact.reset_temporary_data();
  artifact = update_module_graph(compilation, artifact, params).await?;
  Ok(artifact)
}

/// Clean up module graph when finish make.
///
/// Theoretically, we can extract the make stage into a pure function, but some hooks
/// such as `compiler.hooks.finish_make` require a complete compilation structure,
/// so the current approach is to split the make stage into `make` and `finish_make`.
///
/// TODO after hooks support using artifact as a parameter, consider merging make and finish_make.
pub async fn finish_make(
  compilation: &Compilation,
  artifact: MakeArtifact,
) -> Result<MakeArtifact> {
  update_module_graph(
    compilation,
    artifact,
    vec![UpdateParam::BuildEntryAndClean(
      compilation
        .entries
        .values()
        .flat_map(|item| item.all_dependencies())
        .chain(compilation.global_entry.all_dependencies())
        .copied()
        .collect(),
    )],
  )
  .await
}
