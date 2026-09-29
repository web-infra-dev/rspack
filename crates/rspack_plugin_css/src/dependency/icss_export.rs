use rspack_cacheable::{cacheable, cacheable_dyn, with::AsPreset};
use rspack_core::{
  AsContextDependency, AsModuleDependency, Compilation, Dependency, DependencyCategory,
  DependencyCodeGeneration, DependencyId, DependencyRange, DependencyTemplate,
  DependencyTemplateType, DependencyType, ExportNameOrSpec, ExportSpec, ExportsInfoArtifact,
  ExportsOfExportsSpec, ExportsSpec, ModuleGraph, RuntimeSpec, TemplateContext,
  TemplateReplaceSource,
};
use rspack_hash::{RspackHash, RspackHasher};
use smol_str::SmolStr;

use crate::{
  css_exports::resolve_css_dependency,
  css_syntax::escape_identifier,
  utils::{css_generator_options, export_locals_convention},
};

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
  /// Declaration locations that use this definition's own generated identifier.
  pub ranges: Vec<DependencyRange>,
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
      ranges: Vec::new(),
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
  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    Some(CssIcssExportDependencyTemplate::template_type())
  }
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

#[cacheable]
#[derive(Debug, Default)]
pub struct CssIcssExportDependencyTemplate;
impl CssIcssExportDependencyTemplate {
  pub fn template_type() -> DependencyTemplateType {
    DependencyTemplateType::Dependency(DependencyType::CssIcssExport)
  }
}
impl DependencyTemplate for CssIcssExportDependencyTemplate {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    context: &mut TemplateContext,
  ) {
    let dep = dep
      .as_any()
      .downcast_ref::<CssIcssExportDependency>()
      .expect("CSS export template requires a CSS export dependency");
    if dep.ranges.is_empty() {
      return;
    }
    // A declaration's source spelling uses its own identifier, not its composes.
    let value = crate::utils::replace_css_module_id_placeholder(
      &dep.value,
      context.compilation,
      context.module,
    );
    let value = if dep.local_ident {
      escape_identifier(&value).into_owned()
    } else {
      resolve_css_dependency(context.compilation, context.module, *dep.id())
        .map(String::from)
        .unwrap_or_default()
    };
    for range in &dep.ranges {
      source.replace(range.start, range.end, value.clone(), None);
    }
  }
}
