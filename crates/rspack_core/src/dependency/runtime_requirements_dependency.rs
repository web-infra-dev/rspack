use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_util::ext::DynHash;

use crate::{
  AsDependency, Compilation, DependencyTemplate, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub struct RuntimeRequirementsDependency {
  pub runtime_requirements: RuntimeGlobals,
}

#[cacheable_dyn]
impl DependencyTemplate for RuntimeRequirementsDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    code_generatable_context
      .runtime_requirements
      .insert(self.runtime_requirements);
  }

  fn dependency_id(&self) -> Option<crate::DependencyId> {
    None
  }

  fn update_hash(
    &self,
    hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    self.runtime_requirements.dyn_hash(hasher);
  }
}
impl AsDependency for RuntimeRequirementsDependency {}

impl RuntimeRequirementsDependency {
  pub fn new(runtime_requirements: RuntimeGlobals) -> Self {
    Self {
      runtime_requirements,
    }
  }
}
