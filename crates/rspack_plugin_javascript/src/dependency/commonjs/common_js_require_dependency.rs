use rspack_core::{
  module_id, CodeGeneratable, CodeGeneratableContext, CodeGeneratableResult,
  CodeReplaceSourceDependency, CodeReplaceSourceDependencyContext,
  CodeReplaceSourceDependencyReplaceSource, Dependency, DependencyCategory, DependencyId,
  DependencyType, ErrorSpan, ModuleDependency, RuntimeGlobals,
};
use swc_core::ecma::atoms::JsWord;

// Webpack RequireHeaderDependency + CommonJsRequireDependency
#[derive(Debug, Clone)]
pub struct CommonJsRequireDependency {
  id: Option<DependencyId>,
  request: JsWord,
  optional: bool,
  start: u32,
  end: u32,
  span: Option<ErrorSpan>,
}

impl CommonJsRequireDependency {
  pub fn new(
    request: JsWord,
    span: Option<ErrorSpan>,
    start: u32,
    end: u32,
    optional: bool,
  ) -> Self {
    Self {
      id: None,
      request,
      optional,
      start,
      end,
      span,
    }
  }
}

impl Dependency for CommonJsRequireDependency {
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
    &DependencyType::CjsRequire
  }
}

impl ModuleDependency for CommonJsRequireDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn span(&self) -> Option<&ErrorSpan> {
    self.span.as_ref()
  }

  fn get_optional(&self) -> bool {
    self.optional
  }

  fn as_code_replace_source_dependency(&self) -> Option<Box<dyn CodeReplaceSourceDependency>> {
    Some(Box::new(self.clone()))
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl CodeGeneratable for CommonJsRequireDependency {
  fn generate(
    &self,
    _code_generatable_context: &mut CodeGeneratableContext,
  ) -> rspack_error::Result<CodeGeneratableResult> {
    Ok(CodeGeneratableResult::default())
  }
}

impl CodeReplaceSourceDependency for CommonJsRequireDependency {
  fn apply(
    &self,
    source: &mut CodeReplaceSourceDependencyReplaceSource,
    code_generatable_context: &mut CodeReplaceSourceDependencyContext,
  ) {
    let CodeReplaceSourceDependencyContext {
      runtime_requirements,
      compilation,
      ..
    } = code_generatable_context;

    let id: DependencyId = self.id().expect("should have dependency id");

    runtime_requirements.add(RuntimeGlobals::REQUIRE);
    source.replace(
      self.start,
      self.end,
      format!(
        "{}({})",
        RuntimeGlobals::REQUIRE,
        module_id(compilation, &id, &self.request, false).as_str()
      )
      .as_str(),
      None,
    );
  }
}
