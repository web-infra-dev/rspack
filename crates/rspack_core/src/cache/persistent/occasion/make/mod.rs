mod module_graph;

use std::sync::Arc;

use rspack_error::Result;
use rustc_hash::FxHashSet as HashSet;

use super::super::{Storage, cacheable_context::CacheableContext};
use crate::{
  FactorizeInfo, FileCounter,
  compilation::make::{MakeArtifact, MakeArtifactState},
};

/// Make Occasion is used to save MakeArtifact
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
      module_to_lazy_make,
      revoked_modules,
      built_modules,
      // skip
      entry_dependencies: _,
      file_dependencies: _,
      context_dependencies: _,
      missing_dependencies: _,
      build_dependencies: _,
      state: _,
      make_failed_dependencies: _,
      make_failed_module: _,
    } = artifact;

    module_graph::save_module_graph(
      module_graph_partial,
      module_to_lazy_make,
      revoked_modules,
      built_modules,
      &self.storage,
      &self.context,
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::Make::recovery", skip_all)]
  pub async fn recovery(&self) -> Result<MakeArtifact> {
    let mut artifact = MakeArtifact::default();

    let (partial, module_to_lazy_make, force_build_dependencies, isolated_modules) =
      module_graph::recovery_module_graph(&self.storage, &self.context).await?;
    artifact.module_graph_partial = partial;
    artifact.module_to_lazy_make = module_to_lazy_make;
    artifact.state = MakeArtifactState::Uninitialized(force_build_dependencies, isolated_modules);

    // regenerate statistical data
    // TODO set make_failed_module after module.diagnostic are cacheable
    // make failed module include diagnostic that do not support cache, so recovery will not include failed module
    artifact.make_failed_module = Default::default();
    // recovery *_dep
    let mg = artifact.get_module_graph();
    let mut file_dep = FileCounter::default();
    let mut context_dep = FileCounter::default();
    let mut missing_dep = FileCounter::default();
    let mut build_dep = FileCounter::default();
    for (_, module) in mg.modules() {
      let build_info = module.build_info();
      file_dep.add_batch_file(&build_info.file_dependencies);
      context_dep.add_batch_file(&build_info.context_dependencies);
      missing_dep.add_batch_file(&build_info.missing_dependencies);
      build_dep.add_batch_file(&build_info.build_dependencies);
    }
    // recovery make_failed_dependencies
    let mut make_failed_dependencies = HashSet::default();
    for (dep_id, dep) in mg.dependencies() {
      if let Some(info) = FactorizeInfo::get_from(dep)
        && !info.is_success()
      {
        make_failed_dependencies.insert(dep_id);
        file_dep.add_batch_file(&info.file_dependencies());
        context_dep.add_batch_file(&info.context_dependencies());
        missing_dep.add_batch_file(&info.missing_dependencies());
      }
    }
    artifact.make_failed_dependencies = make_failed_dependencies;
    artifact.file_dependencies = file_dep;
    artifact.context_dependencies = context_dep;
    artifact.missing_dependencies = missing_dep;
    artifact.build_dependencies = build_dep;

    Ok(artifact)
  }
}
