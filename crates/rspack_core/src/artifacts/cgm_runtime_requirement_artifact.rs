use rspack_collections::IdentifierMap;

use super::ArtifactExt;
use crate::{
  ModuleIdentifier, RuntimeGlobals, RuntimeSpec, RuntimeSpecMap, incremental::IncrementalPasses,
};

#[derive(Debug, Default)]
pub struct CgmRuntimeRequirementsArtifact {
  module_to_runtime_requirements: IdentifierMap<RuntimeSpecMap<RuntimeGlobals>>,
}

impl CgmRuntimeRequirementsArtifact {
  pub fn is_empty(&self) -> bool {
    self.module_to_runtime_requirements.is_empty()
  }

  pub fn get(&self, module: &ModuleIdentifier, runtime: &RuntimeSpec) -> Option<&RuntimeGlobals> {
    let requirements = self.module_to_runtime_requirements.get(module)?;
    requirements.get(runtime)
  }

  pub fn set_runtime_requirements(
    &mut self,
    module: ModuleIdentifier,
    runtime_requirements_map: RuntimeSpecMap<RuntimeGlobals>,
  ) {
    self
      .module_to_runtime_requirements
      .insert(module, runtime_requirements_map);
  }

  pub fn remove(&mut self, module: &ModuleIdentifier) -> Option<RuntimeSpecMap<RuntimeGlobals>> {
    self.module_to_runtime_requirements.remove(module)
  }

  pub fn clear(&mut self) {
    self.module_to_runtime_requirements.clear();
  }
}

impl ArtifactExt for CgmRuntimeRequirementsArtifact {
  const PASS: IncrementalPasses = IncrementalPasses::MODULES_RUNTIME_REQUIREMENTS;

  fn reset(&mut self) {
    self.clear();
  }
}
