use rspack_core::{
  module_id, Dependency, DependencyCategory, DependencyId, DependencyTemplate, DependencyType,
  ErrorSpan, ModuleDependency, TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ImportMetaHotDeclineDependency {
  id: DependencyId,
  request: JsWord,
  start: u32,
  end: u32,
  span: Option<ErrorSpan>,
}

impl ImportMetaHotDeclineDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      request,
      span,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for ImportMetaHotDeclineDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "ImportMetaHotDeclineDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaHotDecline
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }
}

impl ModuleDependency for ImportMetaHotDeclineDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl DependencyTemplate for ImportMetaHotDeclineDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
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
