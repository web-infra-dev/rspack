mod alternatives;
mod module_graph;

use std::sync::Arc;

pub use module_graph::SCOPE;
use rspack_collections::IdentifierSet;
use rspack_error::Result;
use rustc_hash::FxHashSet;

use super::super::{Storage, cacheable_context::CacheableContext};
use crate::{
  FactorizeInfo, ModuleGraph,
  compilation::build_module_graph::{BuildModuleGraphArtifact, BuildModuleGraphArtifactState},
  utils::{FileCounter, ResourceId},
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
  pub fn save(&self, artifact: &BuildModuleGraphArtifact) {
    let BuildModuleGraphArtifact {
      // write all of field here to avoid forget to update occasion when add new fields
      // for module graph
      module_graph_partial,
      module_to_lazy_make,
      affected_modules,
      affected_dependencies,
      issuer_update_modules,
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

    let mut need_update_modules = issuer_update_modules.clone();
    need_update_modules.extend(affected_modules.active());

    // The updated dependencies should be synced to persistent cache.
    let mg = ModuleGraph::new_ref([Some(module_graph_partial), None]);
    for dep_id in affected_dependencies.active() {
      if let Some(m) = mg.get_parent_module(dep_id) {
        need_update_modules.insert(*m);
      }
    }

    module_graph::save_module_graph(
      module_graph_partial,
      module_to_lazy_make,
      affected_modules.removed(),
      &need_update_modules,
      &self.storage,
      &self.context,
    );
  }

  #[tracing::instrument(name = "Cache::Occasion::Make::recovery", skip_all)]
  pub async fn recovery(&self) -> Result<BuildModuleGraphArtifact> {
    let (partial, module_to_lazy_make, entry_dependencies) =
      module_graph::recovery_module_graph(&self.storage, &self.context).await?;

    // regenerate statistical data
    let mg = ModuleGraph::new_ref([Some(&partial), None]);
    // recovery make_failed_module
    let mut make_failed_module = IdentifierSet::default();
    // recovery *_dep
    let mut file_dep = FileCounter::default();
    let mut context_dep = FileCounter::default();
    let mut missing_dep = FileCounter::default();
    let mut build_dep = FileCounter::default();
    for (mid, module) in mg.modules() {
      let build_info = module.build_info();
      let resource_id = ResourceId::from(mid);
      file_dep.add_files(&resource_id, &build_info.file_dependencies);
      context_dep.add_files(&resource_id, &build_info.context_dependencies);
      missing_dep.add_files(&resource_id, &build_info.missing_dependencies);
      build_dep.add_files(&resource_id, &build_info.build_dependencies);
      if !module.diagnostics().is_empty() {
        make_failed_module.insert(mid);
      }
    }

    // recovery make_failed_dependencies
    let mut make_failed_dependencies = FxHashSet::default();
    for (dep_id, dep) in mg.dependencies() {
      if let Some(info) = FactorizeInfo::get_from(dep) {
        if !info.is_success() {
          make_failed_dependencies.insert(dep_id);
        }
        let resource = dep_id.into();
        file_dep.add_files(&resource, info.file_dependencies());
        context_dep.add_files(&resource, info.context_dependencies());
        missing_dep.add_files(&resource, info.missing_dependencies());
      }
    }

    Ok(BuildModuleGraphArtifact {
      // write all of field here to avoid forget to update occasion when add new fields
      // temporary data set to default
      affected_modules: Default::default(),
      affected_dependencies: Default::default(),
      issuer_update_modules: Default::default(),

      state: BuildModuleGraphArtifactState::Initialized,
      module_graph_partial: partial,
      module_to_lazy_make,

      make_failed_module,
      make_failed_dependencies,
      entry_dependencies,
      file_dependencies: file_dep,
      context_dependencies: context_dep,
      missing_dependencies: missing_dep,
      build_dependencies: build_dep,
    })
  }
}
