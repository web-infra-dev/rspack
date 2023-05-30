use rspack_core::{
  module_id, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency, RuntimeGlobals,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct NewURLDependency {
  start: u32,
  end: u32,
  id: Option<DependencyId>,
  request: JsWord,
  span: Option<ErrorSpan>,
}

impl NewURLDependency {
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

impl Dependency for NewURLDependency {
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

impl ModuleDependency for NewURLDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn as_code_replace_source_dependency(&self) -> Option<Box<dyn CodeReplaceSourceDependency>> {
    Some(Box::new(self.clone()))
  }
}

impl CodeGeneratable for NewURLDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    todo!()
  }
}

impl CodeReplaceSourceDependency for NewURLDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let CodeReplaceSourceDependencyContext {
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
