use derivative::Derivative;

use swc_core::ecma::atoms::JsWord;

use crate::{
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  DependencyType, ErrorSpan, JsAstPath, ModuleIdentifier,
};

#[derive(Derivative)]
#[derivative(Debug, Hash, PartialEq, Eq, Clone)]
pub struct CommonJSRequireDependency {
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

impl CommonJSRequireDependency {
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

impl Dependency for CommonJSRequireDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }
}

impl ModuleDependency for CommonJSRequireDependency {
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

impl CodeGeneratable for CommonJSRequireDependency {
  fn generate(
    &self,
    _code_generatable_context: CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}
