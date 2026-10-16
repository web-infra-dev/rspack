use rspack_collections::IdentifierSet;
use rspack_core::{ChunkGraph, Compilation, DependencyId, Module, ModuleGraph};
use rspack_hash::{RspackHash, RspackHasher};
use rustc_hash::FxHashSet;
use smol_str::SmolStr;

use crate::{
  dependency::{
    CssIcssExportDependency, CssIcssImportDependency, CssIcssSymbolDependency, CssIcssSymbolKind,
  },
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
  let Some(options) = module
    .as_normal_module()
    .and_then(|module| module.get_generator_options())
    .and_then(rspack_core::GeneratorOptions::get_css_module)
  else {
    return name.into();
  };
  let convention = options.exports_convention.unwrap_or_default();
  export_locals_convention(name, convention)
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

pub(crate) fn hash_icss_imports(
  compilation: &Compilation,
  module: &dyn Module,
  hasher: &mut RspackHasher,
) {
  let graph = compilation.get_module_graph();
  let mut pending = module
    .get_dependencies()
    .iter()
    .filter_map(|dep| {
      dep
        .downcast_ref::<CssIcssSymbolDependency>()
        .filter(|dep| dep.kind != CssIcssSymbolKind::LocalDeclaration)
        .map(|dep| dep.target)
    })
    .collect::<Vec<_>>();
  let mut seen = FxHashSet::default();
  let mut modules = IdentifierSet::default();
  while let Some(id) = pending.pop() {
    if !seen.insert(id) {
      continue;
    }
    let dep = graph.dependency_by_id(&id);
    if let Some(dep) = dep.downcast_ref::<CssIcssExportDependency>() {
      pending.extend(
        dep
          .references
          .iter()
          .map(|reference| reference.dependency_id),
      );
      pending.extend(dep.composes.iter().copied());
    } else if let Some(dep) = dep.downcast_ref::<CssIcssImportDependency>()
      && let Some(target) = find_css_export_target(compilation, &id)
    {
      let identifier = target.identifier();
      if modules.insert(identifier) {
        identifier.hash(hasher);
        ChunkGraph::get_module_id(&compilation.module_ids_artifact, identifier).hash(hasher);
        target.build_info().hash.hash(hasher);
      }
      if let Some(id) = find_export(target, dep.import_name()) {
        pending.push(id);
      }
    }
  }
}
