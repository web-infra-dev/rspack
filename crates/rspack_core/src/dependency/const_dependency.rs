use rspack_cacheable::{cacheable, cacheable_dyn, with::AsRefStr};
use rspack_hash::{RspackHash, RspackHasher};
use rspack_util::{atom::Atom, fx_hash::FxHashMap};

use super::DependencyRange;
use crate::{
  Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};

#[derive(Debug, Clone, Default)]
pub struct ConstDependencyPreferredNames(pub FxHashMap<Atom, Atom>);

#[cacheable]
#[derive(Debug, Clone)]
struct ConstDependencyPreferredName {
  #[cacheable(with=AsRefStr)]
  generated: Box<str>,
  #[cacheable(with=AsRefStr)]
  preferred: Box<str>,
}

#[cacheable]
#[derive(Debug, Clone)]
pub struct ConstDependency {
  pub range: DependencyRange,
  #[cacheable(with=AsRefStr)]
  pub content: Box<str>,
  preferred_name: Option<ConstDependencyPreferredName>,
}

impl ConstDependency {
  pub fn new(range: DependencyRange, content: Box<str>) -> Self {
    Self {
      range,
      content,
      preferred_name: None,
    }
  }

  pub fn with_concatenation_scope_preferred_name(
    mut self,
    generated: Box<str>,
    preferred: Box<str>,
  ) -> Self {
    self.preferred_name = Some(ConstDependencyPreferredName {
      generated,
      preferred,
    });
    self
  }
}

impl RspackHash for ConstDependency {
  fn hash(&self, state: &mut RspackHasher) {
    self.range.hash(state);
    state.write(b"|");
    self.content.hash(state);
    if let Some(preferred_name) = &self.preferred_name {
      state.write(b"|");
      preferred_name.generated.hash(state);
      state.write(b"|");
      preferred_name.preferred.hash(state);
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
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ConstDependency>()
      .expect("ConstDependencyTemplate should be used for ConstDependency");

    if let Some(preferred_name) = &dep.preferred_name
      && let Some(concatenation_scope) = code_generatable_context.concatenation_scope.as_deref_mut()
    {
      if concatenation_scope
        .data
        .get::<ConstDependencyPreferredNames>()
        .is_none()
      {
        concatenation_scope
          .data
          .insert(ConstDependencyPreferredNames::default());
      }
      concatenation_scope
        .data
        .get_mut::<ConstDependencyPreferredNames>()
        .expect("preferred names were initialized")
        .0
        .insert(
          preferred_name.generated.as_ref().into(),
          preferred_name.preferred.as_ref().into(),
        );
    }

    source.replace(
      dep.range.start,
      dep.range.end,
      dep.content.to_string(),
      None,
    );
  }
}
