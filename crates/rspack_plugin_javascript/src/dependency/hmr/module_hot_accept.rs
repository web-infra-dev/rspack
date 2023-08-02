use rspack_core::{
  module_id, Dependency, DependencyCategory, DependencyId, DependencyTemplate, DependencyType,
  ErrorSpan, ModuleDependency, TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ModuleHotAcceptDependency {
  id: DependencyId,
  request: JsWord,
  start: u32,
  end: u32,
  category: &'static DependencyCategory,
  dependency_type: &'static DependencyType,

  span: Option<ErrorSpan>,
}

impl ModuleHotAcceptDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      category: &DependencyCategory::CommonJS,
      dependency_type: &DependencyType::ModuleHotAccept,
      span,
      start,
      end,
    }
  }
}

impl Dependency for ModuleHotAcceptDependency {
  fn category(&self) -> &DependencyCategory {
    self.category
  }

  fn dependency_type(&self) -> &DependencyType {
    self.dependency_type
  }
}

impl ModuleDependency for ModuleHotAcceptDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn request(&self) -> &JsWord {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: JsWord) {
    self.request = request;
  }
}

impl DependencyTemplate for ModuleHotAcceptDependency {
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
