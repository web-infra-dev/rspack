use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsDependencyCodeGeneration, AsModuleDependency, Dependency,
  DependencyCategory, DependencyId, DependencyRange, DependencyType,
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
impl AsDependencyCodeGeneration for CssIcssSymbolDependency {}
impl AsContextDependency for CssIcssSymbolDependency {}
impl AsModuleDependency for CssIcssSymbolDependency {}
