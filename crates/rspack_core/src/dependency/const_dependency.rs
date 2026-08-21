use rspack_cacheable::{cacheable, cacheable_dyn, with::AsRefStr};
use rspack_hash::{RspackHash, RspackHasher};

use super::DependencyRange;
use crate::{
  Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ConstDependency {
  pub range: DependencyRange,
  #[cacheable(with=AsRefStr)]
  pub content: Box<str>,
}

impl ConstDependency {
  pub fn new(range: DependencyRange, content: Box<str>) -> Self {
    Self { range, content }
  }
}

impl RspackHash for ConstDependency {
  fn hash(&self, state: &mut RspackHasher) {
    self.range.hash(state);
    state.write(b"|");
    self.content.hash(state);
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for ConstDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ConstDependencyTemplate::template_type())
  }

  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHash::hash(self, hasher);
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
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ConstDependency>()
      .expect("ConstDependencyTemplate should be used for ConstDependency");

    source.replace(
      dep.range.start,
      dep.range.end,
      dep.content.to_string(),
      None,
    );
  }
}
