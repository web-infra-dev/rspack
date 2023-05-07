use std::fmt::Debug;

use rspack_core::{CodeGeneratable, Dependency, ModuleDependency};

#[derive(Clone)]
pub struct ClientReferenceDependency {
  id: Option<rspack_core::DependencyId>,
  request: String,
}

impl Debug for ClientReferenceDependency {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("ClientReferenceDependency").finish()
  }
}

impl ClientReferenceDependency {
  pub fn new(request: String) -> Self {
    Self { request, id: None }
  }
}

impl CodeGeneratable for ClientReferenceDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut rspack_core::CodeGeneratableContext,
  ) -> rspack_error::Result<rspack_core::CodeGeneratableResult> {
    Ok(Default::default())
  }
}

impl Dependency for ClientReferenceDependency {
  fn id(&self) -> Option<rspack_core::DependencyId> {
    self.id
  }

  fn set_id(&mut self, id: Option<rspack_core::DependencyId>) {
    self.id = id
  }

  fn category(&self) -> &rspack_core::DependencyCategory {
    &rspack_core::DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &rspack_core::DependencyType {
    // TODO: should be 'client-reference'
    &rspack_core::DependencyType::ReactFlight
  }
}

impl ModuleDependency for ClientReferenceDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&rspack_core::ErrorSpan> {
    None
  }
}
