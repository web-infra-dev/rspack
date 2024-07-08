use rustc_hash::FxHashMap as HashMap;

use super::super::MakeArtifact;
use crate::{BuildMeta, Module, ModuleIdentifier, NormalModuleSource};

#[derive(Debug, Default)]
pub struct FixBuildMeta {
  origin_module_build_meta: HashMap<ModuleIdentifier, BuildMeta>,
}

impl FixBuildMeta {
  pub fn analyze_force_build_module(
    &mut self,
    artifact: &MakeArtifact,
    module_identifier: &ModuleIdentifier,
  ) {
    let module_graph = artifact.get_module_graph();
    let module = module_graph
      .module_by_identifier(module_identifier)
      .expect("should have module");
    if let Some(build_meta) = module.build_meta() {
      self
        .origin_module_build_meta
        .insert(*module_identifier, build_meta.clone());
    }
  }

  pub fn fix_artifact(self, artifact: &mut MakeArtifact) {
    let mut module_graph = artifact.get_module_graph_mut();
    for (id, build_meta) in self.origin_module_build_meta.into_iter() {
      if let Some(module) = module_graph.module_by_identifier_mut(&id) {
        if let Some(module) = module.as_normal_module_mut() {
          if matches!(module.source(), NormalModuleSource::BuiltFailed(_)) {
            module.set_build_meta(build_meta);
          }
        }
      }
    }
  }
}
