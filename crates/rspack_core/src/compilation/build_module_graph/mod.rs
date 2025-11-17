mod artifact;
mod graph_updater;
mod lazy_barrel_artifact;
mod module_executor;

use rspack_error::Result;

pub use self::{
  artifact::{BuildModuleGraphArtifact, BuildModuleGraphArtifactState},
  graph_updater::{UpdateParam, update_module_graph},
  lazy_barrel_artifact::{
    ForwardId, ForwardedIdSet, HasLazyDependencies, LazyDependencies, LazyUntil, ModuleToLazyMake,
  },
  module_executor::{ExecuteModuleId, ExecutedRuntimeModule, ModuleExecutor},
};
use crate::Compilation;

/// make module graph, support incremental rebuild
///
/// The main method for updating the module graph in the make phase,
/// it will use entries, modified_files, removed_files to update the module graph.
pub async fn build_module_graph(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
) -> Result<BuildModuleGraphArtifact> {
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
pub async fn finish_build_module_graph(
  compilation: &Compilation,
  artifact: BuildModuleGraphArtifact,
) -> Result<BuildModuleGraphArtifact> {
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
