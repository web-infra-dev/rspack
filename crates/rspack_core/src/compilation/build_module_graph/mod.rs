mod graph_updater;
mod lazy_barrel_artifact;
mod module_executor;
pub mod pass;

use std::sync::atomic::Ordering;

use rspack_error::Result;
use rspack_util::tracing_preset::TRACING_BENCH_TARGET;
use tracing::instrument;

pub use self::{
  graph_updater::{UpdateParam, update_module_graph},
  lazy_barrel_artifact::{
    ForwardId, ForwardedIdSet, HasLazyDependencies, LazyDependencies, LazyUntil, ModuleToLazyMake,
  },
  module_executor::{ExecuteModuleId, ExecutedRuntimeModule, ModuleExecutor},
};
pub use crate::{BuildModuleGraphArtifact, BuildModuleGraphArtifactState};
use crate::{Compilation, ExportsInfoArtifact, logger::Logger};

pub async fn build_module_graph_pass(compilation: &mut Compilation) -> Result<()> {
  let logger = compilation.get_logger("rspack.Compiler");
  let start = logger.time("build module graph");
  compilation.do_build_module_graph().await?;
  logger.time_end(start);
  Ok(())
}

impl Compilation {
  #[instrument("Compilation:build_module_graph",target=TRACING_BENCH_TARGET, skip_all)]
  async fn do_build_module_graph(&mut self) -> Result<()> {
    // run module_executor
    if let Some(module_executor) = &mut self.module_executor {
      let mut module_executor = std::mem::take(module_executor);
      module_executor.before_build_module_graph(self).await?;
      self.module_executor = Some(module_executor);
    }

    let artifact = self.build_module_graph_artifact.steal();
    let exports_info_artifact = self.exports_info_artifact.steal();
    let (artifact, exports_info_artifact) =
      build_module_graph(self, artifact, exports_info_artifact).await?;
    self.build_module_graph_artifact = artifact.into();
    self.exports_info_artifact = exports_info_artifact.into();

    self.in_finish_make.store(true, Ordering::Release);

    Ok(())
  }
}

/// make module graph, support incremental rebuild
///
/// The main method for updating the module graph in the make phase,
/// it will use entries, modified_files, removed_files to update the module graph.
pub async fn build_module_graph(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
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
  let artifacts = update_module_graph(compilation, artifact, exports_info_artifact, params).await?;
  Ok(artifacts)
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
  exports_info_artifact: ExportsInfoArtifact,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  update_module_graph(
    compilation,
    artifact,
    exports_info_artifact,
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
