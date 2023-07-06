use rspack_core::{
  module_id, CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource, Dependency,
  DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ImportMetaHotDeclineDependency {
  id: DependencyId,
  request: JsWord,
  start: u32,
  end: u32,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
}

impl ImportMetaHotDeclineDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      request,
      category: &DependencyCategory::Esm,
      dependency_type: &DependencyType::ImportMetaHotDecline,
      span,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for ImportMetaHotDeclineDependency {
  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for ImportMetaHotDeclineDependency {
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

  fn as_code_generatable_dependency(&self) -> Option<&dyn CodeGeneratableDependency> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl CodeGeneratableDependency for ImportMetaHotDeclineDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    source.replace(
      self.start,
      self.end,
      module_id(
        code_generatable_context.compilation,
        &self.id,
        &self.request,
        false,
      )
      .as_str(),
      None,
    );
  }
}
