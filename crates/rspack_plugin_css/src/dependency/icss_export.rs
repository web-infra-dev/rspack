use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyType, ExportNameOrSpec,
  ExportSpec, ExportsInfoArtifact, ExportsOfExportsSpec, ExportsSpec, ModuleGraph, RuntimeSpec,
};
use rspack_hash::{RspackHash, RspackHasher};
use smol_str::SmolStr;

use crate::utils::export_locals_convention;

/// A reference inside one value, such as `color` in `@value shadow: 0 0 color`.
/// The surrounding text stays in the owning definition; the referenced value
/// is read from its dependency instead of copied into a second expression tree.
#[cacheable]
#[derive(Debug, RspackHash)]
pub struct CssIcssReference {
  pub range: DependencyRange,
  #[rspack_hash(skip)]
  pub dependency_id: DependencyId,
}

#[cacheable]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CssExportKind {
  /// Plain `@value`, `:export`, global compose literal, or alias-collision wrapper.
  Value,
  /// Local `.class`, `#id`, `@keyframes`, `@counter-style`, `@container`, or grid identifier.
  LocalIdent,
  /// Local `--custom-property`, `@property`, `@function`, or `@font-palette-values`.
  LocalDashedIdent,
  /// `.class` or `#id` selector bound to an imported ICSS value.
  /// Resolves to `import`'s scoped `LocalIdent` when the target is one; otherwise
  /// falls back to the local identifier stored in `value`.
  ImportedSelector {
    import: DependencyId,
    already_exported: bool,
  },
}

impl RspackHash for CssExportKind {
  fn hash(&self, state: &mut RspackHasher) {
    match self {
      Self::Value => state.write(b"Value"),
      Self::LocalIdent => state.write(b"LocalIdent"),
      Self::LocalDashedIdent => state.write(b"LocalDashedIdent"),
      Self::ImportedSelector {
        already_exported, ..
      } => {
        state.write(b"ImportedSelector:");
        RspackHash::hash(already_exported, state);
      }
    }
  }
}

/// One CSS definition and its composition relationships.
/// `.button { composes: base from "./base.css" }` owns the generated button
/// identifier and the ID of the `base` import. `@value gap: 10px` instead owns
/// the literal `10px`. Local compositions refer directly to local definitions.
/// Aliases in BuildInfo point to this same dependency; they do not copy its value.
#[cacheable]
#[derive(Debug, RspackHash)]
pub struct CssIcssExportDependency {
  #[rspack_hash(skip)]
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  pub name: SmolStr,
  #[cacheable(with=AsPreset)]
  pub value: SmolStr,
  /// Substitutions within `value`, such as the `color` in `0 0 color`.
  pub references: Vec<CssIcssReference>,
  /// Space-separated values appended by `composes`; never copied definitions.
  #[rspack_hash(skip)]
  pub composes: Vec<DependencyId>,
  pub kind: CssExportKind,
  /// `None` for definitions used internally without declaring a JS export.
  // The module graph hash already includes the resulting export names and state.
  #[rspack_hash(skip)]
  pub can_mangle: Option<bool>,
}

impl CssIcssExportDependency {
  pub fn new(
    name: SmolStr,
    value: SmolStr,
    references: Vec<CssIcssReference>,
    kind: CssExportKind,
    can_mangle: Option<bool>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      name,
      value,
      references,
      composes: Vec::new(),
      kind,
      can_mangle,
    }
  }
}

#[cacheable_dyn]
impl Dependency for CssIcssExportDependency {
  fn id(&self) -> &DependencyId {
    &self.id
  }
  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::CssExport
  }
  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::CssIcssExport
  }
  fn get_exports(
    &self,
    mg: &ModuleGraph,
    _cache: &rspack_core::ModuleGraphCacheArtifact,
    _artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    let can_mangle = self.can_mangle?;
    let module = mg.module_by_identifier(mg.get_parent_module(&self.id)?)?;
    let build_info = module.build_info();
    let css = build_info.css.as_deref()?;
    let exports = export_locals_convention(&self.name, css.exports_convention)
      .into_iter()
      .filter(|name| css.exports.get(name.as_str()) == Some(&self.id))
      .map(|name| {
        ExportNameOrSpec::ExportSpec(ExportSpec {
          name: name.into(),
          can_mangle: Some(can_mangle),
          ..Default::default()
        })
      })
      .collect();
    Some(ExportsSpec {
      exports: ExportsOfExportsSpec::Names(exports),
      ..Default::default()
    })
  }
  fn could_affect_referencing_module(&self) -> rspack_core::AffectType {
    rspack_core::AffectType::False
  }
}
#[cacheable_dyn]
impl DependencyCodeGeneration for CssIcssExportDependency {
  fn update_hash(
    &self,
    hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
    RspackHash::hash(self, hasher);
  }
}
impl AsContextDependency for CssIcssExportDependency {}
impl AsModuleDependency for CssIcssExportDependency {}
