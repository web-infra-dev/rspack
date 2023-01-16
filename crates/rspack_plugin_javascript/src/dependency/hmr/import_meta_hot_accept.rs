use rspack_core::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyType, ErrorSpan, JsAstPath, ModuleDependency, ModuleIdentifier,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Eq, Clone)]
pub struct ImportMetaModuleHotAcceptDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  // user_request: String,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
  #[allow(unused)]
  ast_path: JsAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for ImportMetaModuleHotAcceptDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for ImportMetaModuleHotAcceptDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl ImportMetaModuleHotAcceptDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::ImportMetaHotAccept,
      span,
      ast_path,
    }
  }
}

impl Dependency for ImportMetaModuleHotAcceptDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for ImportMetaModuleHotAcceptDependency {
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

impl CodeGeneratable for ImportMetaModuleHotAcceptDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    // The rewrite introduced to much hacks, we cannot do it in the dependency code generation right now. So a noop is returned.
    Ok(CodeGeneratableResult::default())
  }
}
