use rspack_core::{Compilation, DependencyId, Module, ModuleGraph};
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

use crate::{
  css_syntax::unescape_identifier,
  dependency::{CssExportKind, CssIcssExportDependency, CssIcssImportDependency},
  utils::export_locals_convention,
};

pub(crate) fn css_export_dependency<'a>(
  graph: &'a ModuleGraph,
  id: &DependencyId,
) -> &'a CssIcssExportDependency {
  graph
    .dependency_by_id(id)
    .downcast_ref::<CssIcssExportDependency>()
    .expect("CSS export index must refer to an export definition")
}

pub(crate) fn css_export_name(module: &dyn Module, name: &str) -> SmolStr {
  let info = module.build_info();
  let Some(css) = info.css.as_deref() else {
    return name.into();
  };
  if css.exports.contains_key(name) {
    return name.into();
  }
  let unescaped = unescape_identifier(name);
  if unescaped.as_ref() != name && css.exports.contains_key(unescaped.as_ref()) {
    return unescaped.as_ref().into();
  }
  let Some(options) = module
    .as_normal_module()
    .and_then(|module| module.get_generator_options())
    .and_then(rspack_core::GeneratorOptions::get_css_module)
  else {
    return name.into();
  };
  let convention = options.exports_convention.unwrap_or_default();
  export_locals_convention(unescaped.as_ref(), convention)
    .into_iter()
    .find(|name| css.exports.contains_key(name.as_str()))
    .map_or_else(|| name.into(), SmolStr::from)
}

pub(crate) fn find_export(module: &dyn Module, name: &str) -> Option<DependencyId> {
  let name = css_export_name(module, name);
  module
    .build_info()
    .css
    .as_deref()?
    .exports
    .get(name.as_str())
    .copied()
}

pub(crate) fn find_css_export_target<'a>(
  compilation: &'a Compilation,
  id: &DependencyId,
) -> Option<&'a dyn Module> {
  compilation
    .get_module_graph()
    .get_module_by_dependency_id(id)
    .map(|module| module.as_ref())
}

/// Follow an imported ICSS binding across re-exports to the target `LocalIdent`
/// definition and its owning module. Returns `None` for plain `@value` strings,
/// custom properties (`LocalDashedIdent`), missing exports, or cycles.
pub(crate) fn resolve_imported_local_ident<'a>(
  compilation: &'a Compilation,
  mut id: DependencyId,
) -> Option<(&'a dyn Module, &'a CssIcssExportDependency)> {
  let graph = compilation.get_module_graph();
  let mut current_module: Option<&'a dyn Module> = None;
  let mut seen = FxHashSet::default();
  while seen.insert(id) {
    let dependency = graph.dependency_by_id(&id);
    if let Some(import) = dependency.downcast_ref::<CssIcssImportDependency>() {
      let module = find_css_export_target(compilation, &id)?;
      current_module = Some(module);
      id = find_export(module, import.import_name())?;
      continue;
    }
    let export = dependency.downcast_ref::<CssIcssExportDependency>()?;
    match export.kind {
      CssExportKind::LocalIdent => {
        let module = current_module.or_else(|| {
          let parent = graph.get_parent_module(&id)?;
          graph.module_by_identifier(parent).map(AsRef::as_ref)
        })?;
        return Some((module, export));
      }
      CssExportKind::LocalDashedIdent => return None,
      CssExportKind::ImportedSelector { import }
      | CssExportKind::ImportedSelectorExport { import } => {
        id = import;
      }
      CssExportKind::Value => {
        if export.references.len() != 1 || !export.composes.is_empty() {
          return None;
        }
        let reference = &export.references[0];
        if reference.range.start != 0 || reference.range.end as usize != export.value.len() {
          return None;
        }
        id = reference.dependency_id;
      }
    }
  }
  None
}
