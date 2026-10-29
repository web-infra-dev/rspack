use rspack_core::{Compilation, DependencyId, Module, ModuleGraph};
use smol_str::SmolStr;

use crate::{dependency::CssIcssExportDependency, utils::export_locals_convention};

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
