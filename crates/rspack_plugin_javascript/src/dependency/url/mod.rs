use rspack_core::{
  module_id, CodeGeneratableContext, CodeGeneratableDependency, CodeGeneratableSource, Dependency,
  DependencyCategory, DependencyId, DependencyType, ErrorSpan, ModuleDependency, RuntimeGlobals,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct URLDependency {
  start: u32,
  end: u32,
  id: Option<DependencyId>,
  request: JsWord,
  span: Option<ErrorSpan>,
}

impl URLDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      id: None,
      request,
      span,
    }
  }
}

impl Dependency for URLDependency {
  fn id(&self) -> Option<DependencyId> {
    self.id
  }
  fn set_id(&mut self, id: Option<DependencyId>) {
    self.id = id;
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewUrl
  }
}

impl ModuleDependency for URLDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_generatable_dependency(&self) -> Option<Box<&dyn CodeGeneratableDependency>> {
    Some(Box::new(self))
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl CodeGeneratableDependency for URLDependency {
  fn apply(
    &self,
    source: &mut CodeGeneratableSource,
    code_generatable_context: &mut CodeGeneratableContext,
  ) {
    let CodeGeneratableContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;
    let id: DependencyId = self.id().expect("should have dependency id");

    runtime_requirements.insert(RuntimeGlobals::BASE_URI);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    source.replace(
      self.start,
      self.end,
      format!(
        "/* asset import */{}({}), {}",
        RuntimeGlobals::REQUIRE,
        module_id(compilation, &id, &self.request, false),
        RuntimeGlobals::BASE_URI
      )
      .as_str(),
      None,
    );
  }
}
