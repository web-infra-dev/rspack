use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Diagnosable;

use super::BuildModuleGraphArtifact;
use crate::{BuildMeta, Module};

/// A toolkit for cutout to fix build meta
///
/// If a module rebuild failed, its build meta will be reset.
/// This toolkit will restore build meta from successful build to keep importing state.
#[derive(Debug, Default)]
pub struct FixBuildMeta {
  origin_module_build_meta: IdentifierMap<BuildMeta>,
}

impl FixBuildMeta {
  pub fn analyze_force_build_modules(
    &mut self,
    artifact: &BuildModuleGraphArtifact,
    ids: &IdentifierSet,
  ) {
    let module_graph = artifact.get_module_graph();
    for module_identifier in ids {
      let module = module_graph
        .module_by_identifier(module_identifier)
        .expect("should have module");
      self
        .origin_module_build_meta
        .insert(*module_identifier, module.build_meta().clone());
    }
  }

  pub fn fix_artifact(self, artifact: &mut BuildModuleGraphArtifact) {
    let mut module_graph = artifact.get_module_graph_mut();
    for (id, build_meta) in self.origin_module_build_meta {
      if let Some(module) = module_graph.module_by_identifier_mut(&id)
        && let Some(module) = module.as_normal_module_mut()
        && module.first_error().is_some()
      {
        *module.build_meta_mut() = build_meta;
      }
    }
  }
}
