mod dependencies;
mod meta;
mod module_graph;

use std::sync::Arc;

use rspack_cacheable::DeserializeError;

use super::super::{cacheable_context::CacheableContext, Storage};
use crate::make::MakeArtifact;

#[derive(Debug)]
pub struct MakeOccasion {
  context: Arc<CacheableContext>,
  storage: Arc<dyn Storage>,
}

impl MakeOccasion {
  pub fn new(storage: Arc<dyn Storage>, context: Arc<CacheableContext>) -> Self {
    Self { storage, context }
  }

  #[tracing::instrument(name = "MakeOccasion::save", skip_all)]
  pub fn save(&self, artifact: &MakeArtifact) {
    let MakeArtifact {
      // write all of field here to avoid forget to update occasion when add new fields
      // for dependencies
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      build_dependencies,
      // for module graph
      module_graph_partial,
      revoked_modules,
      built_modules,
      // for meta
      make_failed_dependencies,
      make_failed_module,
      // skip
      entry_dependencies: _,
      initialized: _,
      has_module_graph_change: _,
      diagnostics: _,
    } = artifact;
    dependencies::save_dependencies_info(
      file_dependencies,
      context_dependencies,
      missing_dependencies,
      build_dependencies,
      &self.storage,
    )
    .expect("should save dependencies success");

    module_graph::save_module_graph(
      module_graph_partial,
      revoked_modules,
      built_modules,
      &self.storage,
      &self.context,
    );

    meta::save_meta(make_failed_dependencies, make_failed_module, &self.storage)
      .expect("should save make meta");
  }

  #[tracing::instrument(name = "MakeOccasion::recovery", skip_all)]
  pub async fn recovery(&self) -> Result<MakeArtifact, DeserializeError> {
    let mut artifact = MakeArtifact::default();

    let (file_dependencies, context_dependencies, missing_dependencies, build_dependencies) =
      dependencies::recovery_dependencies_info(&self.storage).await?;
    artifact.file_dependencies = file_dependencies;
    artifact.context_dependencies = context_dependencies;
    artifact.missing_dependencies = missing_dependencies;
    artifact.build_dependencies = build_dependencies;

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

    Ok(artifact)
  }
}
