use cow_utils::CowUtils;
use rspack_cacheable::{cacheable, cacheable_dyn, with::AsRefStr};
use rspack_hash::{RspackHash, RspackHasher};

use super::DependencyRange;
use crate::{
  Compilation, DependencyCodeGeneration, DependencyTemplate, DependencyTemplateType,
  GeneratedCodeRebinding, RuntimeSpec, TemplateContext, TemplateReplaceSource,
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
  /// Generated declarations that recreate bindings from the replaced source.
  pub generated_code_rebindings: Vec<GeneratedCodeRebinding>,
}

impl ConstDependency {
  pub fn new(range: DependencyRange, content: Box<str>) -> Self {
    Self {
      range,
      content,
      concatenation_scope_identifier: None,
      generated_code_rebindings: Vec::new(),
    }
  }

  pub fn set_concatenation_scope_identifier(&mut self, identifier: Box<str>) {
    assert!(
      self.generated_code_rebindings.is_empty(),
      "a const dependency cannot create and rebind the same generated identifier"
    );
    self.concatenation_scope_identifier = Some(identifier);
  }

  pub fn set_generated_code_rebindings(&mut self, rebindings: Vec<GeneratedCodeRebinding>) {
    assert!(
      self.concatenation_scope_identifier.is_none(),
      "a const dependency cannot create and rebind the same generated identifier"
    );
    self.generated_code_rebindings = rebindings;
  }
}

impl RspackHash for ConstDependency {
  fn hash(&self, state: &mut RspackHasher) {
    self.range.hash(state);
    state.write(b"|");
    self.content.hash(state);
    self.concatenation_scope_identifier.hash(state);
    if !self.generated_code_rebindings.is_empty() {
      state.write(b"|rebindings|");
      self.generated_code_rebindings.hash(state);
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
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ConstDependency>()
      .expect("ConstDependencyTemplate should be used for ConstDependency");

    let rendered_content = if let Some(scope) = source.faster_concatenation_scope() {
      if let Some(identifier) = &dep.concatenation_scope_identifier {
        let placeholder = scope.ensure_generated_top_level_symbol(identifier);
        dep
          .content
          .cow_replace(identifier.as_ref(), placeholder.as_ref())
          .into_owned()
      } else {
        dep.content.to_string()
      }
    } else {
      dep.content.to_string()
    };

    if dep.generated_code_rebindings.is_empty() {
      source.replace(dep.range.start, dep.range.end, rendered_content, None);
    } else {
      source.replace_with_rebindings(
        dep.range.start,
        dep.range.end,
        rendered_content,
        None,
        &dep.generated_code_rebindings,
      );
    }
  }
}
