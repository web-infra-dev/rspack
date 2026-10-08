use rspack_cacheable::{cacheable, cacheable_dyn};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyType, RuntimeSpec,
};
use rspack_hash::{RspackHash, RspackHasher};

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssIcssSymbolKind {
  /// A selector or declaration is rewritten without marking its export used.
  LocalDeclaration,
  /// A local reference also keeps its definition alive, such as `animation: spin`.
  LocalReference,
  /// An ICSS value is substituted into the source without identifier escaping.
  IcssReference,
}

/// One source occurrence of a CSS symbol, pointing to its shared definition.
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
  pub kind: CssIcssSymbolKind,
}
impl CssIcssSymbolDependency {
  pub fn new(target: DependencyId, range: DependencyRange, kind: CssIcssSymbolKind) -> Self {
    Self {
      id: DependencyId::new(),
      target,
      range,
      kind,
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
  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    (self.range.start, self.range.end).hash(hasher);
    (self.kind as u8).hash(hasher);
    super::hash::hash_binding(compilation.get_module_graph(), self.target, hasher);
  }
}
impl AsContextDependency for CssIcssSymbolDependency {}
impl AsModuleDependency for CssIcssSymbolDependency {}
