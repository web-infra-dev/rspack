use rspack_core::{
  create_css_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, CssAstPath,
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
};
use swc_core::{
  common::util::take::Take,
  css::ast::{AtRulePrelude, Rule},
};

#[derive(Debug, Clone)]
pub struct CssImportDependency {
  id: Option<DependencyId>,
  request: String,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: CssAstPath,
}

impl CssImportDependency {
  pub fn new(request: String, span: Option<ErrorSpan>, ast_path: CssAstPath) -> Self {
    Self {
      id: None,
      request,
      span,
      ast_path,
    }
  }
}

impl Dependency for CssImportDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssImport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssImport
  }
}

impl ModuleDependency for CssImportDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }
}

impl CodeGeneratable for CssImportDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    let mut code_gen = CodeGeneratableResult::default();
    code_gen.visitors.push(
      create_css_visitor!(visit_mut_stylesheet(n: &mut Stylesheet) {
        n.rules = n
          .rules
          .take()
          .into_iter()
          .filter(|rule| match rule {
            Rule::AtRule(at_rule) => !matches!(at_rule.prelude, Some(box AtRulePrelude::ImportPrelude(_))),
            _ => true,
          })
          .collect();
      }),
    );

    Ok(code_gen)
  }
}
