use rspack_core::{
  AsContextDependency, AsDependencyTemplate, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct CssComposeDependency {
  id: DependencyId,
  request: String,
  span: ErrorSpan,
}

impl CssComposeDependency {
  pub fn new(request: String, span: ErrorSpan) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      span,
    }
  }
}

impl Dependency for CssComposeDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssCompose
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssCompose
  }

  fn span(&self) -> Option<ErrorSpan> {
    Some(self.span)
  }
}

impl ModuleDependency for CssComposeDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl AsDependencyTemplate for CssComposeDependency {}
impl AsContextDependency for CssComposeDependency {}
