use rspack_core::{
  CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult, Dependency, DependencyCategory,
  DependencyId, DependencyType, ErrorSpan, ModuleDependency, ModuleIdentifier,
};

#[derive(Debug, Eq, Clone)]
pub struct CssComposeDependency {
  id: Option<DependencyId>,
  parent_module_identifier: Option<ModuleIdentifier>,
  request: String,
  span: Option<ErrorSpan>,
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl PartialEq for CssComposeDependency {
  fn eq(&self, other: &Self) -> bool {
    self.parent_module_identifier == other.parent_module_identifier && self.request == other.request
  }
}

// Do not edit this, as it is used to uniquely identify the dependency.
impl std::hash::Hash for CssComposeDependency {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.parent_module_identifier.hash(state);
    self.request.hash(state);
  }
}

impl CssComposeDependency {
  pub fn new(request: String, span: Option<ErrorSpan>) -> Self {
    Self {
      id: None,
      parent_module_identifier: None,
      request,
      span,
    }
  }
}

impl Dependency for CssComposeDependency {
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
    &DependencyCategory::CssCompose
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssCompose
  }
}

impl ModuleDependency for CssComposeDependency {
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

impl CodeGeneratable for CssComposeDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    Ok(CodeGeneratableResult::default())
  }
}
