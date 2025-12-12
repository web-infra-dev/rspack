use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AffectType, AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration,
  DependencyId, DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType,
  SharedSourceMap, TemplateContext, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug, Clone)]
pub struct ImportMetaResolveHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
}

impl ImportMetaResolveHeaderDependency {
  pub fn new(range: DependencyRange, source_map: Option<SharedSourceMap>) -> Self {
    Self {
      id: DependencyId::new(),
      range,
      source_map,
    }
  }
}

#[cacheable_dyn]
impl Dependency for ImportMetaResolveHeaderDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_deref())
  }

  fn could_affect_referencing_module(&self) -> AffectType {
    AffectType::False
  }
}

impl AsModuleDependency for ImportMetaResolveHeaderDependency {}
impl AsContextDependency for ImportMetaResolveHeaderDependency {}

#[cacheable_dyn]
impl DependencyCodeGeneration for ImportMetaResolveHeaderDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ImportMetaResolveHeaderDependencyTemplate::template_type())
  }
}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ImportMetaResolveHeaderDependencyTemplate;

impl ImportMetaResolveHeaderDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Custom("ImportMetaResolveHeaderDependency")
  }
}

impl DependencyTemplate for ImportMetaResolveHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ImportMetaResolveHeaderDependency>()
      .expect("ImportMetaResolveHeaderDependencyTemplate should only be used for ImportMetaResolveHeaderDependency");

    source.replace(
      dep.range.start,
      dep.range.end,
      "/*import.meta.resolve*/",
      None,
    );
  }
}
