use rspack_core::{
  module_id, CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource,
  ContextOptions, Dependency, DependencyCategory, DependencyId, DependencyType, ErrorSpan,
  ModuleDependency,
};

#[derive(Debug, Clone)]
pub struct RequireResolveDependency {
  pub start: u32,
  pub end: u32,
  pub id: Option<DependencyId>,
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
      id: None,
      optional,
    }
  }
}

impl Dependency for RequireResolveDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::RequireResolve
  }
}

impl ModuleDependency for RequireResolveDependency {
  fn request(&self) -> &str {
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

  fn as_code_generatable_dependency(&self) -> Option<Box<&dyn CodeGeneratableDependency>> {
    Some(Box::new(self))
  }

  fn set_request(&mut self, request: String) {
    self.request = request;
  }
}

impl CodeGeneratableDependency for RequireResolveDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let id: DependencyId = self.id().expect("should have dependency id");

    source.replace(
      self.start,
      self.end,
      module_id(
        code_generatable_context.compilation,
        &id,
        &self.request,
        self.weak,
      )
      .as_str(),
      None,
    );
  }
}
