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
  span: Option<ErrorSpan>,
}

impl ModuleHotAcceptDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      span,
      start,
      end,
    }
  }
}

impl Dependency for ModuleHotAcceptDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ModuleHotAccept
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "ModuleHotAcceptDependency"
  }
}

impl ModuleDependency for ModuleHotAcceptDependency {
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
