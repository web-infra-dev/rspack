use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, TemplateContext, TemplateReplaceSource,
};

use crate::{
  css_exports::{css_export_dependency, resolve_css_dependency},
  css_syntax::escape_identifier,
  utils::replace_css_module_id_placeholder,
};

/// One use of a CSS symbol, not its definition or a module import.
/// In `@value color: red; .button { color: color }`, this stores the range of
/// the last `color` and the ID of its definition. `animation: spin` similarly
/// references the local keyframe definition and escapes its generated identifier.
/// Binding the ID during parse preserves uses before a later redefinition.
#[cacheable]
#[derive(Debug)]
pub struct CssIcssSymbolDependency {
  id: DependencyId,
  pub target: DependencyId,
  range: DependencyRange,
  pub local_ident: bool,
}
impl CssIcssSymbolDependency {
  pub fn new(target: DependencyId, range: DependencyRange, local_ident: bool) -> Self {
    Self {
      id: DependencyId::new(),
      target,
      range,
      local_ident,
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
#[cacheable_dyn]
impl DependencyCodeGeneration for CssIcssSymbolDependency {
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssIcssSymbolDependencyTemplate::template_type())
  }
}
impl AsContextDependency for CssIcssSymbolDependency {}
impl AsModuleDependency for CssIcssSymbolDependency {}
#[cacheable]
#[derive(Debug, Default)]
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
    context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssIcssSymbolDependency>()
      .expect("CSS symbol template requires a CSS symbol dependency");
    let value = if dep.local_ident {
      // `animation: spin` uses only the generated identifier, even when a
      // class named `spin` also has compositions in its JavaScript export.
      let definition = css_export_dependency(context.compilation.get_module_graph(), &dep.target);
      let ident =
        replace_css_module_id_placeholder(&definition.value, context.compilation, context.module);
      Some(escape_identifier(&ident).into_owned())
    } else {
      resolve_css_dependency(context.compilation, context.module, dep.target).map(String::from)
    };
    if let Some(value) = value {
      source.replace(dep.range.start, dep.range.end, value, None);
    }
  }
}
