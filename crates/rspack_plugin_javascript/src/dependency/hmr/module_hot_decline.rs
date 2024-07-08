use rspack_core::{
  module_id, AsContextDependency, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ModuleDependency, TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::Atom;

#[derive(Debug, Clone)]
pub struct ModuleHotDeclineDependency {
  id: DependencyId,
  request: Atom,
  start: u32,
  end: u32,
  span: Option<ErrorSpan>,
}

impl ModuleHotDeclineDependency {
  pub fn new(start: u32, end: u32, request: Atom, span: Option<ErrorSpan>) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      span,
      start,
      end,
    }
  }
}

impl Dependency for ModuleHotDeclineDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ModuleHotDecline
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }
}

impl ModuleDependency for ModuleHotDeclineDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn weak(&self) -> bool {
    true
  }
}

impl DependencyTemplate for ModuleHotDeclineDependency {
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
        self.weak(),
      )
      .as_str(),
      None,
    );
  }

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for ModuleHotDeclineDependency {}
