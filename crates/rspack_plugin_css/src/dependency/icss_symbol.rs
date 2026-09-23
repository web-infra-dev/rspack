use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, CssExport, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, TemplateContext, TemplateReplaceSource,
};

use crate::css_exports::get_icss_symbol;

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

#[cacheable_dyn]
impl DependencyCodeGeneration for CssIcssSymbolDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssIcssSymbolDependencyTemplate::template_type())
  }
}

impl AsContextDependency for CssIcssSymbolDependency {}
impl AsModuleDependency for CssIcssSymbolDependency {}

#[cacheable]
#[derive(Debug, Clone, Default)]
pub struct CssIcssSymbolDependencyTemplate;

impl CssIcssSymbolDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssIcssSymbol)
  }
}

impl DependencyTemplate for CssIcssSymbolDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssIcssSymbolDependency>()
      .expect("CssIcssSymbolDependencyTemplate should be used for CssIcssSymbolDependency");

    let value = get_icss_symbol(
      code_generatable_context.data,
      code_generatable_context.module.identifier(),
      &dep.id,
    );

    if let Some(value) = value {
      source.replace(dep.range.start, dep.range.end, value.to_owned(), None);
    }
  }
}
