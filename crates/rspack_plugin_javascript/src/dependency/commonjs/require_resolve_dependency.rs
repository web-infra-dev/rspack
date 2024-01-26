use rspack_core::{
  module_id, AsContextDependency, AsNullDependency, Dependency, DependencyCategory, DependencyId,
  DependencyTemplate, DependencyType, ErrorSpan, ModuleDependency, TemplateContext,
  TemplateReplaceSource,
};

#[derive(Debug, Clone)]
pub struct RequireResolveDependency {
  pub start: u32,
  pub end: u32,
  pub id: DependencyId,
  pub request: String,
  pub weak: bool,
  span: ErrorSpan,
  optional: bool,
}

impl RequireResolveDependency {
  pub fn new(
    start: u32,
    end: u32,
    request: String,
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
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireResolve
  }

  fn span(&self) -> Option<ErrorSpan> {
    Some(self.span)
  }

  fn dependency_debug_name(&self) -> &'static str {
    "RequireResolveDependency"
  }
}

impl ModuleDependency for RequireResolveDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn weak(&self) -> bool {
    self.weak
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
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

impl AsContextDependency for RequireResolveDependency {}
impl AsNullDependency for RequireResolveDependency {}
