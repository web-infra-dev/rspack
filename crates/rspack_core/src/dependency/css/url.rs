use derivative::Derivative;

use crate::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, CssAstPath, Dependency,
  DependencyCategory, DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
};

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CssUrlDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  span: Option<ErrorSpan>,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  ast_path: CssAstPath,
}

impl CssUrlDependency {
  pub fn new(request: String, span: Option<ErrorSpan>, ast_path: CssAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      span,
      ast_path,
    }
  }
}

impl Dependency for CssUrlDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = identifier;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssUrl
  }
}

impl ModuleDependency for CssUrlDependency {
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

impl CodeGeneratable for CssUrlDependency {
  fn generate(&self, _code_generatable_context: &CodeGeneratableContext) -> CodeGeneratableResult {
    todo!()
  }
}
