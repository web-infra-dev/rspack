use rspack_core::{
  create_css_visitor, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, CssAstPath,
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
  ModuleIdentifier,
};
use swc_core::{
  common::util::take::Take,
  css::ast::{AtRulePrelude, Rule, UrlValue},
};

use crate::visitors::is_url_requestable;

#[derive(Debug, Eq, Clone)]
pub struct CssImportDependency {
  id: Option<DependencyId>,
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: CssAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for CssImportDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier && self.request == other.request
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for CssImportDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
  }
}

impl CssImportDependency {
  pub fn new(request: String, span: Option<ErrorSpan>, ast_path: CssAstPath) -> Self {
    Self {
      id: None,
      parent_module_identifier: None,
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
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = identifier;
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
            Rule::AtRule(at_rule) => {
              if let Some(box AtRulePrelude::ImportPrelude(prelude)) = &at_rule.prelude {
                let href_string = match &prelude.href {
                  box swc_core::css::ast::ImportHref::Url(url) => {
                    let href_string = url
                      .value
                      .as_ref()
                      .map(|box value| match value {
                        UrlValue::Str(str) => str.value.clone(),
                        UrlValue::Raw(raw) => raw.value.clone(),
                      })
                      .unwrap_or_default();
                    href_string
                  }
                  box swc_core::css::ast::ImportHref::Str(str) => str.value.clone(),
                };
                !is_url_requestable(&href_string)
              } else {
                true
              }
            }
            _ => true,
          })
          .collect();
      }),
    );

    Ok(code_gen)
  }
}
