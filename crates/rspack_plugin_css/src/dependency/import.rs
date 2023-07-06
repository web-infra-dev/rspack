use rspack_core::{
  CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource, Dependency,
  DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct CssImportDependency {
  id: DependencyId,
  request: String,
  span: Option<ErrorSpan>,
  start: u32,
  end: u32,
}

impl CssImportDependency {
  pub fn new(request: String, span: Option<ErrorSpan>, start: u32, end: u32) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      span,
      start,
      end,
    }
  }
}

impl Dependency for CssImportDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssImport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssImport
  }
}

impl ModuleDependency for CssImportDependency {
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

  fn as_code_generatable_dependency(&self) -> Option<&dyn CodeGeneratableDependency> {
    Some(self)
  }
}

impl CodeGeneratableDependency for CssImportDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) {
    source.replace(self.start - 8 /* @import */, self.end, "", None);
  }
}
