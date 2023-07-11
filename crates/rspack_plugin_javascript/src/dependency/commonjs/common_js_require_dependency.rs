use rspack_core::{
  module_id, Dependency, DependencyCategory, DependencyId, DependencyTemplate, DependencyType,
  ErrorSpan, ModuleDependency, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
};
use swc_core::ecma::atoms::JsWord;

// Webpack RequireHeaderDependency + CommonJsRequireDependency
#[derive(Debug, Clone)]
pub struct CommonJsRequireDependency {
  id: DependencyId,
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
      id: DependencyId::new(),
      request,
      optional,
      start,
      end,
      span,
    }
  }
}

impl Dependency for CommonJsRequireDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CommonJS
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CjsRequire
  }
}

impl ModuleDependency for CommonJsRequireDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

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

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }
}

impl DependencyTemplate for CommonJsRequireDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      runtime_requirements,
      compilation,
      ..
    } = code_generatable_context;

    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    source.replace(
      self.start,
      self.end,
      format!(
        "{}({})",
        RuntimeGlobals::REQUIRE,
        module_id(compilation, &self.id, &self.request, false).as_str()
      )
      .as_str(),
      None,
    );
  }
}
