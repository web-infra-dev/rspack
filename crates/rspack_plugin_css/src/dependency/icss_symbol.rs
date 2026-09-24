use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, AsModuleDependency, CssExport, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyType, TemplateReplaceSource,
};

#[cacheable]
#[derive(Debug)]
pub struct CssIcssSymbolDependency {
  id: DependencyId,
  value: CssExport,
  range: DependencyRange,
}

impl CssIcssSymbolDependency {
  pub fn new(value: CssExport, range: DependencyRange) -> Self {
    Self {
      id: DependencyId::new(),
      value,
      range,
    }
  }

  pub(crate) fn value(&self) -> &CssExport {
    &self.value
  }

  // The CSS generator owns the precomputed value and passes it directly.
  pub(crate) fn render(&self, source: &mut TemplateReplaceSource, value: Option<&str>) {
    if let Some(value) = value {
      source.replace(self.range.start, self.range.end, value.to_owned(), None);
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssIcssSymbolDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssExport
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssIcssSymbol
  }

  fn range(&self) -> Option<DependencyRange> {
    Some(self.range)
  }

  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::True
  }
}

impl AsDependencyCodeGeneration for CssIcssSymbolDependency {}
impl AsContextDependency for CssIcssSymbolDependency {}
impl AsModuleDependency for CssIcssSymbolDependency {}
