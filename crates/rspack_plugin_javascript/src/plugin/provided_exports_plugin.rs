use std::collections::VecDeque;

use rspack_core::{
  export_info_mut, DependencyId, ExportInfo, ExportNameOrSpec, ExportsInfo, ExportsOfExportsSpec,
  ExportsSpec, ModuleGraph, ModuleGraphConnection, ModuleIdentifier, UsageState,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::ecma::atoms::JsWord;

pub struct ProvidedExportsPlugin<'a> {
  mg: &'a mut ModuleGraph,
}

impl<'a> ProvidedExportsPlugin<'a> {
  pub fn apply(&mut self) {
    let mut dependencies: HashMap<ModuleIdentifier, HashSet<ModuleIdentifier>> = HashMap::default();
    let mut q = VecDeque::new();
    while let Some(module_id) = q.pop_back() {
      let mut changed = false;
      let exports_specs_from_dependencies: HashMap<DependencyId, ExportsSpec> = HashMap::default();
      self.process_dependencies_block(module_id);
      // for (const [dep, exportsSpec] of exportsSpecsFromDependencies) {
      // 	processExportsSpec(dep, exportsSpec);
      // }
    }
  }

  pub fn process_dependencies_block(&mut self, mi: ModuleIdentifier) -> Option<()> {
    None
  }
  pub fn process_exports_spec(
    &mut self,
    dep_id: DependencyId,
    exports_desc: ExportsSpec,
    exports_info: &mut ExportsInfo,
  ) {
    let exports = &exports_desc.exports;
    let global_can_mangle = &exports_desc.can_mangle;
    let global_from = exports_desc.from.as_ref();
    let global_priority = &exports_desc.priority;
    let global_terminal_binding = exports_desc.terminal_binding.clone().unwrap_or(false);
    let export_dependencies = &exports_desc.dependencies;
    if !exports_desc.hide_export.is_empty() {
      for name in exports_desc.hide_export.iter() {
        let export_info = exports_info.export_info_mut(name);
        export_info.unuset_target(&dep_id);
      }
    }
    match exports {
      ExportsOfExportsSpec::True => {
        // TODO: unknown exports https://github.com/webpack/webpack/blob/853bfda35a0080605c09e1bdeb0103bcb9367a10/lib/FlagDependencyExportsPlugin.js#L165-L175
      }
      ExportsOfExportsSpec::Null => {}
      ExportsOfExportsSpec::Array(ele) => {}
    }
  }

  // pub fn merge_exports(
  //   &mut self,
  //   exports_info: &mut ExportsInfo,
  //   exports: &Vec<ExportNameOrSpec>,
  //   global_export_info: DefaultExportInfo,
  // ) {
  //   for export_name_or_spec in exports {
  //     let (name, can_mangle, terminal_binding, exports, from, from_export, priority, hidden) =
  //       match export_name_or_spec {
  //         ExportNameOrSpec::String(name) => (
  //           name,
  //           global_export_info.can_mangle,
  //           global_export_info.terminal_binding,
  //           None,
  //           global_export_info.from,
  //           None,
  //           global_export_info.priority,
  //           false,
  //         ),
  //         ExportNameOrSpec::ExportSpec(spec) => (spec.name, spec.can_mangle.unwrap_or(global), spec),
  //       };
  //   }
  // }
}

/// Used for reducing nums of params
struct DefaultExportInfo<'a> {
  can_mangle: bool,
  terminal_binding: bool,
  from: Option<&'a ModuleGraphConnection>,
  priority: u8,
}
