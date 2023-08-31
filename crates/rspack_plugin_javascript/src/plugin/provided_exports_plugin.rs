use std::collections::hash_map::Entry;
use std::collections::VecDeque;

use rspack_core::{
  DependencyId, ExportInfoProvided, ExportNameOrSpec, ExportsInfo, ExportsInfoId,
  ExportsOfExportsSpec, ExportsSpec, ModuleGraph, ModuleGraphConnection, ModuleIdentifier,
};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::ecma::atoms::JsWord;

pub struct ProvidedExportsPlugin<'a> {
  mg: &'a mut ModuleGraph,
  changed: bool,
  current_module_id: ModuleIdentifier,
  dependencies: HashMap<ModuleIdentifier, HashSet<ModuleIdentifier>>,
}

impl<'a> ProvidedExportsPlugin<'a> {
  pub fn new(mg: &'a mut ModuleGraph) -> Self {
    Self {
      mg,
      changed: false,
      current_module_id: ModuleIdentifier::default(),
      dependencies: HashMap::default(),
    }
  }

  pub fn apply(&mut self) {
    let mut q = VecDeque::new();
    while let Some(module_id) = q.pop_back() {
      self.changed = false;
      self.current_module_id = module_id;
      let mut exports_specs_from_dependencies: HashMap<DependencyId, ExportsSpec> =
        HashMap::default();
      self.process_dependencies_block(module_id, &mut exports_specs_from_dependencies);
      // I use this trick because of rustc borrow rules, it is safe becuase dependency provide plugin is sync, there are no other methods using it at the same time.
      let mut exports_info_id = { self.mg.get_exports_info(&module_id).id };
      for (dep_id, exports_spec) in exports_specs_from_dependencies.into_iter() {
        self.process_exports_spec(dep_id, exports_spec, exports_info_id);
      }
      // Swap it back
      if self.changed {
        self.notify_dependencies(&mut q);
      }
    }
  }

  pub fn notify_dependencies(&mut self, q: &mut VecDeque<ModuleIdentifier>) {
    if let Some(set) = self.dependencies.get(&self.current_module_id) {
      for mi in set.iter() {
        q.push_back(*mi);
      }
    }
  }

  pub fn process_dependencies_block(
    &mut self,
    mi: ModuleIdentifier,
    exports_specs_from_dependencies: &mut HashMap<DependencyId, ExportsSpec>,
  ) -> Option<()> {
    let mgm = self.mg.module_graph_module_by_identifier(&mi)?;
    // This clone is aiming to avoid use mut ref and immutable ref at the same time.
    for ele in mgm.dependencies.clone().iter() {
      self.process_dependency(ele, exports_specs_from_dependencies);
    }
    None
  }

  pub fn process_dependency(
    &mut self,
    dep_id: &DependencyId,
    exports_specs_from_dependencies: &mut HashMap<DependencyId, ExportsSpec>,
  ) -> Option<()> {
    let dep = self.mg.dependency_by_id(dep_id)?;
    let exports_specs = dep.get_exports()?;
    exports_specs_from_dependencies.insert(*dep_id, exports_specs);
    Some(())
  }

  pub fn process_exports_spec(
    &mut self,
    dep_id: DependencyId,
    exports_spec: ExportsSpec,
    exports_info_id: ExportsInfoId,
  ) {
    let exports = &exports_spec.exports;
    let global_can_mangle = &exports_spec.can_mangle;
    let global_from = exports_spec.from.as_ref();
    let global_priority = &exports_spec.priority;
    let global_terminal_binding = exports_spec.terminal_binding.unwrap_or(false);
    let export_dependencies = &exports_spec.dependencies;
    if let Some(hide_export) = exports_spec.hide_export {
      for name in hide_export.iter() {
        let from_exports_info_id = exports_info_id.export_info_mut(name, self.mg);
        let exports_info = self.mg.get_exports_info_mut_by_id(&from_exports_info_id);
        let export_info = exports_info
          .exports
          .get_mut(name)
          .expect("should have exports info");
        export_info.unuset_target(&dep_id);
      }
    }
    match exports {
      ExportsOfExportsSpec::True => {
        // TODO: unknown exports https://github.com/webpack/webpack/blob/853bfda35a0080605c09e1bdeb0103bcb9367a10/lib/FlagDependencyExportsPlugin.js#L165-L175
      }
      ExportsOfExportsSpec::Null => {}
      ExportsOfExportsSpec::Array(ele) => {
        self.merge_exports(
          exports_info_id,
          ele,
          DefaultExportInfo {
            can_mangle: *global_can_mangle,
            terminal_binding: global_terminal_binding,
            from: global_from,
            priority: *global_priority,
          },
          dep_id,
        );
      }
    }

    if let Some(export_dependencies) = export_dependencies {
      for export_dep in export_dependencies {
        match self.dependencies.entry(*export_dep) {
          Entry::Occupied(mut occ) => {
            occ.get_mut().insert(self.current_module_id);
          }
          Entry::Vacant(vac) => {
            vac.insert(HashSet::from_iter([self.current_module_id]));
          }
        }
      }
    }
  }

  pub fn merge_exports(
    &mut self,
    exports_info: ExportsInfoId,
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
            match spec.can_mangle {
              Some(v) => Some(v),
              None => global_export_info.can_mangle,
            },
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
            match spec.priority {
              Some(v) => Some(v),
              None => global_export_info.priority,
            },
            spec.hidden.unwrap_or(false),
          ),
        };
      let exports_info_id = exports_info.export_info_mut(&name, self.mg);
      let exports_info = self.mg.get_exports_info_mut_by_id(&exports_info_id);
      let mut export_info = exports_info.exports.get_mut(&name).unwrap().clone();
      if let Some(ref mut provided) = export_info.provided && matches!(provided, ExportInfoProvided::False | ExportInfoProvided::Null) {
        *provided = ExportInfoProvided::True;
        self.changed = true;
      }

      if Some(false) != export_info.can_mangle_provide && can_mangle == Some(false) {
        export_info.can_mangle_provide = Some(false);
        self.changed = true;
      }

      if terminal_binding && !export_info.terminal_binding {
        export_info.terminal_binding = true;
        self.changed = true;
      }

      if let Some(exports) = exports {
        let nested_exports_info = export_info.create_nested_exports_info(self.mg);
        self.merge_exports(
          nested_exports_info,
          exports,
          global_export_info.clone(),
          dep_id,
        );
      }

      if let Some(from) = from {
        let changed = if hidden {
          export_info.unuset_target(&dep_id)
        } else {
          let fallback = vec![name.clone()];
          let export_name = if let Some(from) = from_export {
            Some(from)
          } else {
            Some(&fallback)
          };
          export_info.set_target(&dep_id, from, export_name, priority)
        };
        self.changed |= changed;
      }

      // Recalculate target exportsInfo
      let target = export_info.get_target(self.mg, None);

      let exports_info = self.mg.get_exports_info_mut_by_id(&exports_info_id);
      let mut export_info_old = exports_info.exports.get_mut(&name).unwrap();
      std::mem::replace(export_info_old, export_info);

      let mut target_exports_info: Option<ExportsInfoId> = None;
      if let Some(target) = target {
        let target_module_exports_info = self.mg.get_exports_info(&target.module);
        target_exports_info = target_module_exports_info
          .id
          .get_nested_exports_info(target.exports, self.mg);
        match self.dependencies.entry(target.module) {
          Entry::Occupied(mut occ) => {
            occ.get_mut().insert(self.current_module_id);
          }
          Entry::Vacant(vac) => {
            vac.insert(HashSet::from_iter([self.current_module_id]));
          }
        }
      }
      let exports_info = self.mg.get_exports_info_mut_by_id(&exports_info_id);
      let mut export_info = exports_info.exports.get_mut(&name).unwrap();
      if export_info.exports_info_owned {
        let changed = export_info
          .exports_info
          .expect("should have exports_info when exports_info_owned is true")
          .set_redirect_name_to(self.mg, target_exports_info);
        if changed {
          self.changed = true;
        }
      } else if (export_info.exports_info != target_exports_info) {
        export_info.exports_info = target_exports_info;
        self.changed = true;
      }
    }
  }
}

/// Used for reducing nums of params
#[derive(Debug, Clone)]
pub struct DefaultExportInfo<'a> {
  can_mangle: Option<bool>,
  terminal_binding: bool,
  from: Option<&'a ModuleGraphConnection>,
  priority: Option<u8>,
}
