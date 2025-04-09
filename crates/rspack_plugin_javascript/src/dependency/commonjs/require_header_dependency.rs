use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyId, DependencyLocation,
  DependencyRange, DependencyTemplate, DynamicDependencyTemplate, DynamicDependencyTemplateType,
  RuntimeGlobals, SharedSourceMap, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
}

impl RequireHeaderDependency {
  pub fn new(range: DependencyRange, source_map: Option<SharedSourceMap>) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      source_map,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for RequireHeaderDependency {}
impl AsContextDependency for RequireHeaderDependency {}

#[cacheable_dyn]
impl DependencyTemplate for RequireHeaderDependency {
  fn dynamic_dependency_template(&self) -> Option<DynamicDependencyTemplateType> {
    Some(RequireHeaderDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireHeaderDependencyTemplate;

impl RequireHeaderDependencyTemplate {
  pub fn template_type() -> DynamicDependencyTemplateType {
    DynamicDependencyTemplateType::CustomType("RequireHeaderDependency")
  }
}

impl DynamicDependencyTemplate for RequireHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyTemplate,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireHeaderDependency>()
      .expect("RequireHeaderDependencyTemplate should only be used for RequireHeaderDependency");

    let TemplateContext {
      runtime_requirements,
      ..
    } = code_generatable_context;
    runtime_requirements.insert(RuntimeGlobals::REQUIRE);
    source.replace(
      dep.range.start,
      dep.range.end,
      RuntimeGlobals::REQUIRE.name(),
      None,
    );
  }
}
