mod cutout;
pub mod repair;

use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rspack_paths::InternedPathSet;
use rustc_hash::FxHashSet;

use self::{cutout::Cutout, repair::repair};
use super::BuildModuleGraphArtifact;
use crate::{
  CacheFacade, Compilation, DependencyId, ExportsInfoArtifact, incremental::IncrementalPasses,
};

/// The param to update module graph
#[derive(Debug, Clone)]
pub enum UpdateParam {
  /// Build some entries, this param will only ensure that those entries are built,
  /// but will not remove entries that are not in this lists.
  BuildEntry(FxHashSet<DependencyId>),
  /// Build some entries and clean up the entries that not in this list.
  BuildEntryAndClean(FxHashSet<DependencyId>),
  /// Build the module which module.need_build is true, i.e. modules where loader.cacheable is false
  CheckNeedBuild,
  /// Build the module and dependency which depend on these modified file.
  ModifiedFiles(InternedPathSet),
  /// Build the module and dependency which depend on these removed file.
  RemovedFiles(InternedPathSet),
  /// Force build some modules.
  ForceBuildModules(IdentifierSet),
}

/// Update module graph through `UpdateParam`
pub async fn update_module_graph(
  compilation: &Compilation,
  artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  update_module_graph_impl(
    compilation,
    artifact,
    exports_info_artifact,
    params,
    None,
    false,
  )
  .await
}

/// Update a non-incremental make with webpack's `Compilation/modules` cache.
///
/// A hot incremental make always delegates to `update_module_graph`, so its
/// artifact update path never reads or writes the module cache.
pub async fn update_module_graph_with_module_cache(
  compilation: &Compilation,
  artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  if compilation
    .incremental
    .mutations_readable(IncrementalPasses::BUILD_MODULE_GRAPH)
  {
    return update_module_graph(compilation, artifact, exports_info_artifact, params).await;
  }
  update_module_graph_impl(
    compilation,
    artifact,
    exports_info_artifact,
    params,
    compilation
      .file_system_info
      .as_ref()
      .map(|_| compilation.get_cache("Compilation/modules")),
    true,
  )
  .await
}

/// Force a non-incremental module rebuild without restoring its previous build
/// result, while still storing the successful replacement like webpack.
///
/// A hot incremental make delegates to the cache-free update path before the
/// module cache is accessed.
pub async fn rebuild_module_graph_with_module_cache(
  compilation: &Compilation,
  artifact: BuildModuleGraphArtifact,
  exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  if compilation
    .incremental
    .mutations_readable(IncrementalPasses::BUILD_MODULE_GRAPH)
  {
    return update_module_graph(compilation, artifact, exports_info_artifact, params).await;
  }
  update_module_graph_impl(
    compilation,
    artifact,
    exports_info_artifact,
    params,
    compilation
      .file_system_info
      .as_ref()
      .map(|_| compilation.get_cache("Compilation/modules")),
    false,
  )
  .await
}

async fn update_module_graph_impl(
  compilation: &Compilation,
  mut artifact: BuildModuleGraphArtifact,
  mut exports_info_artifact: ExportsInfoArtifact,
  params: Vec<UpdateParam>,
  module_cache: Option<CacheFacade>,
  restore_module_cache: bool,
) -> Result<(BuildModuleGraphArtifact, ExportsInfoArtifact)> {
  let mut cutout = Cutout::default();

  let build_dependencies = cutout.cutout_artifact(compilation, &mut artifact, params);

  let revoked_modules = artifact.revoked_modules().copied().collect();
  compilation
    .plugin_driver
    .compilation_hooks
    .revoked_modules
    .call(compilation, &revoked_modules)
    .await?;

  (artifact, exports_info_artifact) = repair(
    compilation,
    artifact,
    exports_info_artifact,
    build_dependencies,
    module_cache,
    restore_module_cache,
  )
  .await?;
  cutout.fix_artifact(&mut artifact);
  Ok((artifact, exports_info_artifact))
}
