use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, ConnectionState, Dependency, DependencyCategory, DependencyCodeGeneration,
  DependencyCondition, DependencyConditionFn, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, FactorizeInfo, ModuleDependency, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleGraphConnection, RuntimeGlobals, RuntimeSpec, TemplateContext,
  TemplateReplaceSource, UsedByExports, module_id,
};
use swc_core::ecma::atoms::Atom;

use crate::connection_active_used_by_exports;

#[cacheable]
#[derive(Debug, Clone)]
pub struct URLDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  request: Atom,
  range: DependencyRange,
  range_url: DependencyRange,
  used_by_exports: Option<UsedByExports>,
  relative: bool,
  factorize_info: FactorizeInfo,
}

impl URLDependency {
  pub fn new(
    request: Atom,
    range: DependencyRange,
    range_url: DependencyRange,
    relative: bool,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      request,
      range,
      range_url,
      used_by_exports: None,
      relative,
      factorize_info: Default::default(),
    }
  }

  pub fn set_used_by_exports(&mut self, used_by_exports: Option<UsedByExports>) {
    self.used_by_exports = used_by_exports;
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

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
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

  fn get_condition(&self) -> Option<DependencyCondition> {
    Some(DependencyCondition::new(URLDependencyCondition))
  }

  fn factorize_info(&self) -> &FactorizeInfo {
    &self.factorize_info
  }

  fn factorize_info_mut(&mut self) -> &mut FactorizeInfo {
    &mut self.factorize_info
  }
}

#[cacheable_dyn]
impl DependencyCodeGeneration for URLDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(URLDependencyTemplate::template_type())
  }
}

impl AsContextDependency for URLDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct URLDependencyTemplate;

impl URLDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::NewUrl)
  }
}

impl DependencyTemplate for URLDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<URLDependency>()
      .expect("URLDependencyTemplate should be used for URLDependency");
    let TemplateContext {
      compilation,
      runtime_requirements,
      ..
    } = code_generatable_context;

    runtime_requirements.insert(RuntimeGlobals::REQUIRE);

    if dep.relative {
      runtime_requirements.insert(RuntimeGlobals::RELATIVE_URL);
      source.replace(
        dep.range.start,
        dep.range.end,
        format!(
          "/* asset import */ new {}({}({}))",
          RuntimeGlobals::RELATIVE_URL,
          RuntimeGlobals::REQUIRE,
          module_id(compilation, &dep.id, &dep.request, false),
        )
        .as_str(),
        None,
      );
    } else {
      runtime_requirements.insert(RuntimeGlobals::BASE_URI);
      source.replace(
        dep.range_url.start,
        dep.range_url.end,
        format!(
          "/* asset import */{}({}), {}",
          RuntimeGlobals::REQUIRE,
          module_id(compilation, &dep.id, &dep.request, false),
          RuntimeGlobals::BASE_URI
        )
        .as_str(),
        None,
      );
    }
  }
}

struct URLDependencyCondition;

impl DependencyConditionFn for URLDependencyCondition {
  fn get_connection_state(
    &self,
    connection: &ModuleGraphConnection,
    runtime: Option<&RuntimeSpec>,
    module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
  ) -> ConnectionState {
    let dependency = module_graph
      .dependency_by_id(&connection.dependency_id)
      .expect("should have dependency");
    let dependency = dependency
      .downcast_ref::<URLDependency>()
      .expect("should be URLDependency");
    ConnectionState::Active(connection_active_used_by_exports(
      connection,
      runtime,
      module_graph,
      dependency.used_by_exports.as_ref(),
    ))
  }
}
