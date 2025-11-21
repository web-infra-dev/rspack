use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, RuntimeGlobals,
  SharedSourceMap, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl RequireHeaderDependency {
  pub fn new(range: DependencyRange, source_map: Option<SharedSourceMap>) -> Self {
    let loc = range.to_loc(source_map.as_ref());
    Self {
      id: DependencyId::new(),
      range,
      loc,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RequireHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

impl AsModuleDependency for RequireHeaderDependency {}
impl AsContextDependency for RequireHeaderDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireHeaderDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireHeaderDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireHeaderDependencyTemplate;

impl RequireHeaderDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("RequireHeaderDependency")
  }
}

impl DependencyTemplate for RequireHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
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
