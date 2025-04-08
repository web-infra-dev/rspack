use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyId, DependencyLocation,
  DependencyRange, DependencyTemplate, DependencyType, DynamicDependencyTemplate,
  DynamicDependencyTemplateType, SharedSourceMap, TemplateContext, TemplateReplaceSource,
};

// Remove `export` label.
// Before: `export const a = 1`
// After: `const a = 1`
#[cacheable]
#[derive(Debug, Clone)]
pub struct ESMExportHeaderDependency {
  id: DependencyId,
  range: DependencyRange,
  range_decl: Option<DependencyRange>,
  #[cacheable(with=Skip)]
  source_map: Option<SharedSourceMap>,
}

impl ESMExportHeaderDependency {
  pub fn new(
    range: DependencyRange,
    range_decl: Option<DependencyRange>,
    source_map: Option<SharedSourceMap>,
  ) -> Self {
    Self {
      range,
      range_decl,
      source_map,
      id: DependencyId::default(),
    }
  }
}

#[cacheable_dyn]
impl Dependency for ESMExportHeaderDependency {
  fn id(&self) -> &rspack_core::DependencyId {
    &self.id
  }

  fn loc(&self) -> Option<DependencyLocation> {
    self.range.to_loc(self.source_map.as_ref())
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::EsmExportHeader
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}

#[cacheable_dyn]
impl DependencyTemplate for ESMExportHeaderDependency {
  fn dynamic_dependency_template(&self) -> Option<DynamicDependencyTemplateType> {
    Some(ESMExportHeaderDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ESMExportHeaderDependency {}
impl AsContextDependency for ESMExportHeaderDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMExportHeaderDependencyTemplate;

impl ESMExportHeaderDependencyTemplate {
  pub fn template_type() -> DynamicDependencyTemplateType {
    DynamicDependencyTemplateType::DependencyType(DependencyType::EsmExportHeader)
  }
}

impl DynamicDependencyTemplate for ESMExportHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyTemplate,
    source: &mut TemplateReplaceSource,
    _code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<ESMExportHeaderDependency>()
      .expect(
        "ESMExportHeaderDependencyTemplate should only be used for ESMExportHeaderDependency",
      );
    source.replace(
      dep.range.start,
      if let Some(range) = &dep.range_decl {
        range.start
      } else {
        dep.range.end
      },
      "",
      None,
    );
  }
}
