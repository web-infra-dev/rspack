use rspack_core::{
  Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct CssComposeDependency {
  id: DependencyId,
  request: String,
  span: Option<ErrorSpan>,
}

impl CssComposeDependency {
  pub fn new(request: String, span: Option<ErrorSpan>) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      span,
    }
  }
}

impl Dependency for CssComposeDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssCompose
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssCompose
  }
}

impl ModuleDependency for CssComposeDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}
