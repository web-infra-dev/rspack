use rspack_core::{
  get_dependency_used_by_exports_condition, module_id, AsContextDependency, Dependency,
  DependencyCategory, DependencyCondition, DependencyId, DependencyTemplate, DependencyType,
  ErrorSpan, ModuleDependency, RuntimeGlobals, TemplateContext, TemplateReplaceSource,
  UsedByExports,
};
use swc_core::ecma::atoms::Atom;

#[derive(Debug, Clone)]
pub struct URLDependency {
  start: u32,
  end: u32,
  outer_start: u32,
  outer_end: u32,
  id: DependencyId,
  request: Atom,
  span: Option<ErrorSpan>,
  used_by_exports: Option<UsedByExports>,
  relative: bool,
}

impl URLDependency {
  pub fn new(
    start: u32,
    end: u32,
    outer_start: u32,
    outer_end: u32,
    request: Atom,
    span: Option<ErrorSpan>,
    relative: bool,
  ) -> Self {
    Self {
      start,
      end,
      outer_start,
      outer_end,
      id: DependencyId::new(),
      request,
      span,
      used_by_exports: None,
      relative,
    }
  }
}

impl Dependency for URLDependency {
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

    runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    if self.relative {
      runtime_requirements.insert(RuntimeGlobals::RELATIVE_URL);
      source.replace(
        self.outer_start,
        self.outer_end,
        format!(
          "/* asset import */ new {}({}({}))",
          RuntimeGlobals::RELATIVE_URL,
          RuntimeGlobals::REQUIRE,
          module_id(compilation, &self.id, &self.request, false),
        )
        .as_str(),
        None,
      );
    } else {
      runtime_requirements.insert(RuntimeGlobals::BASE_URI);
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

  fn dependency_id(&self) -> Option<DependencyId> {
    Some(self.id)
  }
}

impl AsContextDependency for URLDependency {}
