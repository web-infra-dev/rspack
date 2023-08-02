use rspack_core::{
  module_id, ContextOptions, Dependency, DependencyCategory, DependencyId, DependencyTemplate,
  DependencyType, ErrorSpan, ExportsReferencedType, ModuleDependency, ModuleGraph, RuntimeSpec,
  TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct RequireResolveDependency {
  pub start: u32,
  pub end: u32,
  pub id: DependencyId,
  pub request: JsWord,
  pub weak: bool,
  span: ErrorSpan,
  optional: bool,
}

impl RequireResolveDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: JsWord,
    weak: bool,
    span: ErrorSpan,
    optional: bool,
  ) -> Self {
    Self {
      start,
      end,
      request,
      weak,
      span,
      id: DependencyId::new(),
      optional,
    }
  }
}

impl Dependency for RequireResolveDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireResolve
  }
}

impl ModuleDependency for RequireResolveDependency {
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
    Some(&self.span)
  }

  fn weak(&self) -> bool {
    self.weak
  }

  fn options(&self) -> Option<&ContextOptions> {
    None
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: JsWord) {
    self.request = request;
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _runtime: &RuntimeSpec,
  ) -> ExportsReferencedType {
    ExportsReferencedType::No
  }
}

impl DependencyTemplate for RequireResolveDependency {
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
        self.weak,
      )
      .as_str(),
      None,
    );
  }
}
