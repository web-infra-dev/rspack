use rspack_core::{
  module_id, AsContextDependency, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ModuleDependency, TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::Atom;

#[derive(Debug, Clone)]
pub struct ImportMetaHotAcceptDependency {
  id: DependencyId,
  request: Atom,
  start: u32,
  end: u32,
  span: Option<ErrorSpan>,
}

impl ImportMetaHotAcceptDependency {
  pub fn new(start: u32, end: u32, request: Atom, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      request,
      span,
      id: DependencyId::new(),
    }
  }
}

impl Dependency for ImportMetaHotAcceptDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::ImportMetaHotAccept
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }
}

impl ModuleDependency for ImportMetaHotAcceptDependency {
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

impl DependencyTemplate for ImportMetaHotAcceptDependency {
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

impl AsContextDependency for ImportMetaHotAcceptDependency {}
