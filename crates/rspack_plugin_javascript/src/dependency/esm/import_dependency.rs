use rspack_core::Dependency;
use rspack_core::{module_namespace_promise, DependencyType, ErrorSpan, ImportDependencyTrait};
use rspack_core::{DependencyCategory, DependencyId, DependencyTemplate};
use rspack_core::{ModuleDependency, TemplateContext, TemplateReplaceSource};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct ImportDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  request: JsWord,
  span: Option<ErrorSpan>,
  referenced_exports: Option<Vec<JsWord>>,
}

impl ImportDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: JsWord,
    span: Option<ErrorSpan>,
    referenced_exports: Option<Vec<JsWord>>,
  ) -> Self {
    Self {
      start,
      end,
      request,
      span,
      id: DependencyId::new(),
      referenced_exports,
    }
  }
}

impl Dependency for ImportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Esm
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::DynamicImport
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }

  fn dependency_debug_name(&self) -> &'static str {
    "ImportDependency"
  }
}

impl ModuleDependency for ImportDependency {
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

impl ImportDependencyTrait for ImportDependency {
  fn referenced_exports(&self) -> Option<&Vec<JsWord>> {
    self.referenced_exports.as_ref()
  }
}

impl DependencyTemplate for ImportDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    source.replace(
      self.start,
      self.end,
      module_namespace_promise(
        code_generatable_context,
        &self.id,
        &self.request,
        true,
        self.dependency_type().as_str().as_ref(),
        false,
      )
      .as_str(),
      None,
    );
  }
}
