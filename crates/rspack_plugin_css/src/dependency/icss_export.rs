use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyType, ExportNameOrSpec,
  ExportSpec, ExportsInfoArtifact, ExportsOfExportsSpec, ExportsSpec, ModuleGraph, RuntimeSpec,
};
use rspack_hash::{RspackHash, RspackHasher};
use smol_str::SmolStr;

use crate::utils::{css_generator_options, export_locals_convention};

/// A reference inside one value, such as `color` in `@value shadow: 0 0 color`.
/// The surrounding text stays in the owning definition; the referenced value
/// is read from its dependency instead of copied into a second expression tree.
#[cacheable]
#[derive(Debug)]
pub struct CssIcssReference {
  pub range: DependencyRange,
  pub dependency_id: DependencyId,
}

/// One CSS definition and its composition relationships.
/// `.button { composes: base from "./base.css" }` owns the generated button
/// identifier and the ID of the `base` import. `@value gap: 10px` instead owns
/// the literal `10px`. Local compositions refer directly to local definitions.
/// Aliases in BuildInfo point to this same dependency; they do not copy its value.
#[cacheable]
#[derive(Debug)]
pub struct CssIcssExportDependency {
  id: DependencyId,
  #[cacheable(with=AsPreset)]
  pub name: SmolStr,
  #[cacheable(with=AsPreset)]
  pub value: SmolStr,
  /// Substitutions within `value`, such as the `color` in `0 0 color`.
  pub references: Vec<CssIcssReference>,
  /// Space-separated values appended by `composes`; never copied definitions.
  pub composes: Vec<DependencyId>,
  pub local_ident: bool,
  /// `None` for definitions used internally without declaring a JS export.
  pub can_mangle: Option<bool>,
}

impl CssIcssExportDependency {
  pub fn new(
    name: SmolStr,
    value: SmolStr,
    references: Vec<CssIcssReference>,
    local_ident: bool,
    can_mangle: Option<bool>,
  ) -> Self {
    Self {
      id: DependencyId::new(),
      name,
      value,
      references,
      composes: Vec::new(),
      local_ident,
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
    let options = module
      .as_normal_module()
      .and_then(|module| module.get_generator_options());
    let convention = css_generator_options(options)
      .exports_convention
      .unwrap_or_default();
    let info = module.build_info();
    let index = &info.css.as_deref()?.exports;
    let exports = export_locals_convention(&self.name, convention)
      .into_iter()
      .filter(|name| index.get(name.as_str()) == Some(&self.id))
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
    self.value.hash(hasher);
  }
}
impl AsContextDependency for CssIcssExportDependency {}
impl AsModuleDependency for CssIcssExportDependency {}
