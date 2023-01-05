use derivative::Derivative;

use swc_core::ecma::atoms::JsWord;

use crate::{
  dependency::{
    CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
    ModuleDependency,
  },
  DependencyType, ErrorSpan, JsAstPath, ModuleIdentifier,
};

#[derive(Debug, Eq, Clone)]
pub struct EsmDynamicImportDependency {
  parent_module_identifier: Option<ModuleIdentifier>,
  request: JsWord,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,
  // user_request: String,
  span: Option<ErrorSpan>,

  #[allow(unused)]
  ast_path: JsAstPath,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for EsmDynamicImportDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier
      && self.request == other.request
      && self.category == other.category
      && self.dependency_type == other.dependency_type
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for EsmDynamicImportDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
    self.category.hash(state);
    self.dependency_type.hash(state);
  }
}

impl EsmDynamicImportDependency {
  pub fn new(request: JsWord, span: Option<ErrorSpan>, ast_path: JsAstPath) -> Self {
    Self {
      parent_module_identifier: None,
      request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::DynamicImport,
      span,
      ast_path,
    }
  }
}

impl Dependency for EsmDynamicImportDependency {
  fn parent_module_identifier(&self) -> Option<&ModuleIdentifier> {
    self.parent_module_identifier.as_ref()
  }

  fn set_parent_module_identifier(&mut self, module_identifier: Option<ModuleIdentifier>) {
    self.parent_module_identifier = module_identifier;
  }

  fn category(&self) -> &DependencyCategory {
    &self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    &self.dependency_type
  }
}

impl ModuleDependency for EsmDynamicImportDependency {
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

impl CodeGeneratable for EsmDynamicImportDependency {
  fn generate(&self, _code_generatable_context: &CodeGeneratableContext) -> CodeGeneratableResult {
    todo!()
  }
}
