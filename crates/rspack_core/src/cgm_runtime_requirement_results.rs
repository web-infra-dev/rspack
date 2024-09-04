use rspack_collections::IdentifierMap;

use crate::{ModuleIdentifier, RuntimeGlobals, RuntimeSpec, RuntimeSpecMap};

#[derive(Debug, Default)]
pub struct CgmRuntimeRequirementsResults {
  module_to_runtime_requirements: IdentifierMap<RuntimeSpecMap<RuntimeGlobals>>,
}

impl CgmRuntimeRequirementsResults {
  pub fn get(&self, module: &ModuleIdentifier, runtime: &RuntimeSpec) -> Option<&RuntimeGlobals> {
    let requirements = self.module_to_runtime_requirements.get(module)?;
    requirements.get(runtime)
  }

  pub fn set(
    &mut self,
    module: ModuleIdentifier,
    runtime: RuntimeSpec,
    runtime_requirements: RuntimeGlobals,
  ) {
    let requirements = self
      .module_to_runtime_requirements
      .entry(module)
      .or_default();
    requirements.set(runtime, runtime_requirements);
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
}
