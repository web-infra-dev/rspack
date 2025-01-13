mod meta;
mod module_graph;

use std::sync::Arc;

use rspack_error::Result;

use super::super::{cacheable_context::CacheableContext, Storage};
use crate::{make::MakeArtifact, FileCounter};

#[derive(Debug)]
pub struct MakeOccasion {
  context: Arc<CacheableContext>,
  storage: Arc<dyn Storage>,
}

impl MakeOccasion {
  pub fn new(storage: Arc<dyn Storage>, context: Arc<CacheableContext>) -> Self {
    Self { storage, context }
  }

  #[tracing::instrument(name = "Cache::Occasion::Make::save", skip_all)]
  pub fn save(&self, artifact: &MakeArtifact) {
    let MakeArtifact {
      // write all of field here to avoid forget to update occasion when add new fields
      // for module graph
      module_graph_partial,
      revoked_modules,
      built_modules,
      // for meta
      make_failed_dependencies,
      make_failed_module,
      // skip
      entry_dependencies: _,
      file_dependencies: _,
      context_dependencies: _,
      missing_dependencies: _,
      build_dependencies: _,
      initialized: _,
      has_module_graph_change: _,
      diagnostics: _,
    } = artifact;

    module_graph::save_module_graph(
      module_graph_partial,
      revoked_modules,
      built_modules,
      &self.storage,
      &self.context,
    );

    meta::save_meta(make_failed_dependencies, make_failed_module, &self.storage);
  }

  #[tracing::instrument(name = "Cache::Occasion::Make::recovery", skip_all)]
  pub async fn recovery(&self) -> Result<MakeArtifact> {
    let mut artifact = MakeArtifact::default();

    // TODO can call recovery with multi thread
    // TODO return DeserializeError not panic
    let (make_failed_dependencies, make_failed_module) = meta::recovery_meta(&self.storage).await?;
    artifact.make_failed_dependencies = make_failed_dependencies;
    artifact.make_failed_module = make_failed_module;

    let (partial, make_failed_dependencies) =
      module_graph::recovery_module_graph(&self.storage, &self.context).await?;
    artifact.module_graph_partial = partial;
    artifact
      .make_failed_dependencies
      .extend(make_failed_dependencies);

    // TODO remove it after code splitting support incremental rebuild
    artifact.has_module_graph_change = true;

    // regenerate statistical data
    let mut file_dep = FileCounter::default();
    let mut context_dep = FileCounter::default();
    let mut missing_dep = FileCounter::default();
    let mut build_dep = FileCounter::default();
    for (_, module) in artifact.get_module_graph().modules() {
      if let Some(build_info) = module.build_info() {
        file_dep.add_batch_file(&build_info.file_dependencies);
        context_dep.add_batch_file(&build_info.context_dependencies);
        missing_dep.add_batch_file(&build_info.missing_dependencies);
        build_dep.add_batch_file(&build_info.build_dependencies);
      }
    }
    artifact.file_dependencies = file_dep;
    artifact.context_dependencies = context_dep;
    artifact.missing_dependencies = missing_dep;
    artifact.build_dependencies = build_dep;
    artifact.reset_dependencies_incremental_info();

    Ok(artifact)
  }
}
