use rspack_core::{
  get_dependency_used_by_exports_condition, module_id, Dependency, DependencyCategory,
  DependencyCondition, DependencyId, DependencyTemplate, DependencyType, ErrorSpan,
  ModuleDependency, ModuleGraph, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
  UsedByExports,
};
use swc_core::ecma::atoms::JsWord;

#[derive(Debug, Clone)]
pub struct URLDependency {
  start: u32,
  end: u32,
  id: DependencyId,
  request: JsWord,
  span: Option<ErrorSpan>,
  used_by_exports: UsedByExports,
}

impl URLDependency {
  pub fn new(start: u32, end: u32, request: JsWord, span: Option<ErrorSpan>) -> Self {
    Self {
      start,
      end,
      id: DependencyId::new(),
      request,
      span,
      used_by_exports: UsedByExports::default(),
    }
  }
}

impl Dependency for URLDependency {
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Url
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::NewUrl
  }
}

impl ModuleDependency for URLDependency {
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
    self.span.as_ref()
  }

  fn as_code_generatable_dependency(&self) -> Option<&dyn DependencyTemplate> {
    Some(self)
  }

  fn set_request(&mut self, request: JsWord) {
    self.request = request;
  }

  fn get_condition(&self, module_graph: &ModuleGraph) -> DependencyCondition {
    get_dependency_used_by_exports_condition(&self.id, &self.used_by_exports, module_graph)
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
