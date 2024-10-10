use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  get_dependency_used_by_exports_condition, module_id, AsContextDependency, Compilation,
  Dependency, DependencyCategory, DependencyCondition, DependencyId, DependencyTemplate,
  DependencyType, ModuleDependency, RealDependencyLocation, RuntimeGlobals, RuntimeSpec,
  TemplateContext, TemplateReplaceSource, UsedByExports,
};
use swc_core::ecma::atoms::Atom;

#[cacheable]
#[derive(Debug, Clone)]
pub struct URLDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: RealDependencyLocation,
  range_url: RealDependencyLocation,
  used_by_exports: Option<UsedByExports>,
  relative: bool,
}

impl URLDependency {
  pub fn new(
    request: Atom,
    range: RealDependencyLocation,
    range_url: RealDependencyLocation,
    relative: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      range_url,
      used_by_exports: None,
      relative,
    }
  }
}

#[cacheable_dyn]
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

  fn range(&self) -> Option<&RealDependencyLocation> {
    Some(&self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

#[cacheable_dyn]
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

#[cacheable_dyn]
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
        self.range.start,
        self.range.end,
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
        self.range_url.start,
        self.range_url.end,
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

  fn update_hash(
    &self,
    _hasher: &mut dyn std::hash::Hasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }
}

impl AsContextDependency for URLDependency {}
