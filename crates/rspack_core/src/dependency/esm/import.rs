use derivative::Derivative;
use rspack_error::Result;

use swc_core::ecma::atoms::JsWord;

use crate::{
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  ErrorSpan, JsAstPath, ModuleIdentifier,
};

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq, Clone)]
pub struct EsmImportDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  // user_request: String,
  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  span: Option<ErrorSpan>,

  #[derivative(PartialEq = "ignore")]
  #[derivative(Hash = "ignore")]
  ast_path: JsAstPath,
}

impl EsmImportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      // user_request,
      span,
      ast_path,
    }
  }
}

impl Dependency for EsmImportDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }
}

impl ModuleDependency for EsmImportDependency {
  fn request(&self) -> &str {
    &*self.request
  }

  fn user_request(&self) -> &str {
    &*self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }
}

impl CodeGeneratable for EsmImportDependency {
  fn generate(
    &self,
    code_generatable_context: CodeGeneratableContext,
  ) -> Result<CodeGeneratableResult> {
    todo!()
  }
}
