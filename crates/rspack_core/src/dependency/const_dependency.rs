use cow_utils::CowUtils;
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
  /// Identifier introduced by `content` that must participate in top-level
  /// deconfliction when faster module concatenation skips the codegen parse.
  pub concatenation_scope_identifier: Option<Box<str>>,
}

impl ConstDependency {
  pub fn new(range: DependencyRange, content: Box<str>) -> Self {
    Self {
      range,
      content,
      concatenation_scope_identifier: None,
    }
  }

  pub fn set_concatenation_scope_identifier(&mut self, identifier: Box<str>) {
    self.concatenation_scope_identifier = Some(identifier);
  }
}

impl RspackHash for ConstDependency {
  fn hash(&self, state: &mut RspackHasher) {
    self.range.hash(state);
    state.write(b"|");
    self.content.hash(state);
    self.concatenation_scope_identifier.hash(state);
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

    let rendered_content =
      if let Some(scope) = code_generatable_context.faster_concatenation_scope() {
        scope.remove_original_range(dep.range);
        if let Some(identifier) = &dep.concatenation_scope_identifier {
          let placeholder = scope.ensure_generated_top_level_symbol(identifier);
          dep
            .content
            .cow_replace(identifier.as_ref(), placeholder.as_ref())
            .into_owned()
        } else {
          scope.add_used_names_from_generated_code(&dep.content);
          dep.content.to_string()
        }
      } else {
        dep.content.to_string()
      };

    source.replace(dep.range.start, dep.range.end, rendered_content, None);
  }
}
