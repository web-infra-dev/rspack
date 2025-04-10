use rspack_cacheable::{cacheable, cacheable_dyn, with::AsRefStr};
use rspack_util::ext::DynHash;

use crate::{
  Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType,
  RuntimeGlobals, RuntimeSpec, TemplateContext, TemplateReplaceSource,
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
impl DependencyCodeGeneration for ConstDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ConstDependencyTemplate::template_type())
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

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ConstDependencyTemplate;

impl ConstDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ConstDependency")
  }
}

impl DependencyTemplate for ConstDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ConstDependency>()
      .expect("ConstDependencyTemplate should be used for ConstDependency");

    if let Some(runtime_requirements) = &dep.runtime_requirements {
      code_generatable_context
        .runtime_requirements
        .insert(*runtime_requirements);
    }
    source.replace(dep.start, dep.end, dep.content.as_ref(), None);
  }
}
