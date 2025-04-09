use rspack_cacheable::{cacheable, cacheable_dyn, with::Skip};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCodeGeneration, DependencyId,
  DependencyLocation, DependencyRange, DependencyTemplate, DependencyTemplateType, DependencyType,
  SharedSourceMap, TemplateContext, TemplateReplaceSource,
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
impl DependencyCodeGeneration for ESMExportHeaderDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(ESMExportHeaderDependencyTemplate::template_type())
  }
}

impl AsModuleDependency for ESMExportHeaderDependency {}
impl AsContextDependency for ESMExportHeaderDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct ESMExportHeaderDependencyTemplate;

impl ESMExportHeaderDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::EsmExportHeader)
  }
}

impl DependencyTemplate for ESMExportHeaderDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
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
