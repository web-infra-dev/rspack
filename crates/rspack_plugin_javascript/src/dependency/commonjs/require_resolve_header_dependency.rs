use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration,
  DependencyId, DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType,
  SharedSourceMap, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct RequireResolveHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  loc: Option<DependencyLocation>,
}

impl RequireResolveHeaderDependency {
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
impl Dependency for RequireResolveHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.loc.clone()
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AsModuleDependency for RequireResolveHeaderDependency {}
impl AsContextDependency for RequireResolveHeaderDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for RequireResolveHeaderDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(RequireResolveHeaderDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct RequireResolveHeaderDependencyTemplate;

impl RequireResolveHeaderDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("RequireResolveHeaderDependency")
  }
}

impl DependencyTemplate for RequireResolveHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<RequireResolveHeaderDependency>()
      .expect("RequireResolveHeaderDependencyTemplate should only be used for RequireResolveHeaderDependency");

    source.replace(dep.range.start, dep.range.end, "/*require.resolve*/", None);
  }
}
