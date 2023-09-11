use rspack_core::{DependencyTemplate, TemplateContext, TemplateReplaceSource};

#[derive(Debug, Clone)]
pub struct PureExpressionDependency {
  pub start: u32,
  pub end: u32,
}

impl PureExpressionDependency {
  pub fn new(start: u32, end: u32) -> Self {
    Self { start, end }
  }
}

impl DependencyTemplate for PureExpressionDependency {
  fn apply(
    &self,
    _source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    // TODO
  }
}
