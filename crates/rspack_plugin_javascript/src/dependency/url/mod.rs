use rspack_core::{
  get_dependency_used_by_exports_condition, module_id, Dependency, DependencyCategory,
  DependencyCondition, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ModuleDependency, RuntimeGlobals, TemplateContext, TemplateReplaceSource, UsedByExports,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct URLDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  request: JsWord,
  span: Option<ErrorSpan>,
  used_by_exports: Option<UsedByExports>,
}

impl URLDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      id: DependencyId::new(),
      request,
      span,
      used_by_exports: None,
    }
  }
}

impl Dependency for URLDependency {
  fn dependency_debug_name(&self) -> &'static str {
    "URLDependency"
  }

  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewUrl
  }

  fn span(&self) -> Option<ErrorSpan> {
    self.span
  }
}

impl ModuleDependency for URLDependency {
  fn request(&self) -> &str {
    &self.request
  }

  fn user_request(&self) -> &str {
    &self.request
  }

  fn set_request(&mut self, request: String) {
    self.request = request.into();
  }

  fn get_condition(&self) -> Option<DependencyCondition> {
    get_dependency_used_by_exports_condition(self.id, self.used_by_exports.as_ref())
  }
}

impl DependencyTemplate for URLDependency {
  fn apply(
    &self,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;

    runtime_requirements.insert(RuntimeGlobals::BASE_URI);
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    source.replace(
      self.start,
      self.end,
      format!(
        "/* asset import */{}({}), {}",
        RuntimeGlobals::REQUIRE,
        module_id(compilation, &self.id, &self.request, false),
        RuntimeGlobals::BASE_URI
      )
      .as_str(),
      None,
    );
  }
}
