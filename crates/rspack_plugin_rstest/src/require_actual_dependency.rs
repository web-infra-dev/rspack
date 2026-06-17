use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, RuntimeGlobals,
  TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RstestRequireActualDependency {
  id: DependencyId,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl RstestRequireActualDependency {
  pub fn new(range: DependencyRange, loc: Option<DependencyLocation>) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      loc,
    }
  }
}

#[cacheable_dyn]
impl Dependency for RstestRequireActualDependency {
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

impl AsModuleDependency for RstestRequireActualDependency {}
impl AsContextDependency for RstestRequireActualDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for RstestRequireActualDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RstestRequireActualDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RstestRequireActualDependencyTemplate;

impl RstestRequireActualDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("RstestRequireActualDependency")
  }
}

impl DependencyTemplate for RstestRequireActualDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RstestRequireActualDependency>()
      .expect(
        "RstestRequireActualDependencyTemplate should only be used for \
         RstestRequireActualDependency",
      );

    let TemplateContext {
      runtime_template, ..
    } = code_generatable_context;
    source.replace(
      dep.range.start,
      dep.range.end,
      format!(
        "{}.rstest_require_actual",
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE_SCOPE)
      ),
      None,
    );
  }
}
