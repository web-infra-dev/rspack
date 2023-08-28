use std::collections::VecDeque;

use rspack_core::{
  export_info_mut, DependencyId, ExportInfo, ExportInfoProvided, ExportNameOrSpec, ExportsInfo,
  ExportsOfExportsSpec, ExportsSpec, ModuleGraph, ModuleGraphConnection, ModuleIdentifier,
  UsageState,
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

  pub fn merge_exports(
    &mut self,
    exports_info: &mut ExportsInfo,
    exports: &Vec<ExportNameOrSpec>,
    global_export_info: DefaultExportInfo,
    dep_id: DependencyId,
  ) {
    for export_name_or_spec in exports {
      let (name, can_mangle, terminal_binding, exports, from, from_export, priority, hidden) =
        match export_name_or_spec {
          ExportNameOrSpec::String(name) => (
            name.clone(),
            global_export_info.can_mangle,
            global_export_info.terminal_binding,
            None::<&Vec<ExportNameOrSpec>>,
            global_export_info.from.cloned(),
            None::<&Vec<JsWord>>,
            global_export_info.priority,
            false,
          ),
          ExportNameOrSpec::ExportSpec(spec) => (
            spec.name.clone(),
            spec.can_mangle.unwrap_or(global_export_info.can_mangle),
            spec
              .terminal_binding
              .unwrap_or(global_export_info.terminal_binding),
            spec.exports.as_ref(),
            if spec.from.is_some() {
              spec.from.clone()
            } else {
              global_export_info.from.cloned()
            },
            spec.export.as_ref(),
            spec.priority.unwrap_or(global_export_info.priority),
            spec.hidden.unwrap_or(false),
          ),
        };
      let export_info = exports_info.export_info_mut(&name);
      if let Some(provided) = export_info.provided && matches!(provided, ExportInfoProvided::False | ExportInfoProvided::Null) {
        provided = ExportInfoProvided::True;
        // TODO; adjust global changed
      }

      if Some(false) != export_info.can_mangle_provide && can_mangle == false {
        export_info.can_mangle_provide = Some(false);
        // TODO; adjust global changed
      }

      if terminal_binding && !export_info.terminal_binding {
        export_info.terminal_binding = true;
        // TODO; adjust global changed
      }

      if let Some(exports) = exports {
        // let nested_exports_info = export_info.create_nested_exports_info();
        // self.merge_exports(nested_exports_info, exports, global_export_info);
      }

      if let Some(from) = from {
        let changed = if hidden {
          export_info.unuset_target(&dep_id)
        } else {
          export_info.set_target(&dep_id, Some(from), export_name, Some(priority))
        };
        if changed {
          // TODO; adjust global changed
        }
      }
    }
  }
}

/// Used for reducing nums of params
pub struct DefaultExportInfo<'a> {
  can_mangle: bool,
  terminal_binding: bool,
  from: Option<&'a ModuleGraphConnection>,
  priority: u8,
}
