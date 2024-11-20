use rspack_cacheable::{cacheable, cacheable_dyn, with::AsRefStr};
use rspack_util::ext::DynHash;

use crate::{
  AsDependency, Compilation, DependencyTemplate, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ConstDependency {
  pub start: u32,
  pub end: u32,
  #[cacheable(with=AsRefStr)]
  pub content: Box<str>,
  pub runtime_requirements: Option<RuntimeGlobals>,
}

impl ConstDependency {
  pub fn new(
    start: u32,
    end: u32,
    content: Box<str>,
    runtime_requirements: Option<RuntimeGlobals>,
  ) -> Self {
    Self {
      start,
      end,
      content,
      runtime_requirements,
    }
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ConstDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    if let Some(runtime_requirements) = &self.runtime_requirements {
      code_generatable_context
        .runtime_requirements
        .insert(*runtime_requirements);
    }
    source.replace(self.start, self.end, self.content.as_ref(), None);
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
    self.start.dyn_hash(hasher);
    self.end.dyn_hash(hasher);
    self.content.dyn_hash(hasher);
    self.runtime_requirements.dyn_hash(hasher);
  }
}

impl AsDependency for ConstDependency {}
