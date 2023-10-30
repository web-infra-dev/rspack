use std::collections::hash_map::Entry;
use std::collections::VecDeque;

use rspack_core::{
  BuildMetaExportsType, Compilation, DependencyId, ExportInfoProvided, ExportNameOrSpec,
  ExportsInfoId, ExportsOfExportsSpec, ExportsSpec, ModuleGraph, ModuleGraphConnection,
  ModuleIdentifier, Plugin,
};
use rspack_error::Result;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};
use swc_core::ecma::atoms::JsWord;

struct FlagDependencyExportsProxy<'a> {
  mg: &'a mut ModuleGraph,
  changed: bool,
  current_module_id: ModuleIdentifier,
  dependencies: HashMap<ModuleIdentifier, HashSet<ModuleIdentifier>>,
}

impl<'a> FlagDependencyExportsProxy<'a> {
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

    // take the ownership of module_identifier_to_module_graph_module to avoid borrow ref and
    // mut ref of `ModuleGraph` at the same time
    let module_graph_modules =
      std::mem::take(&mut self.mg.module_identifier_to_module_graph_module);
    for mgm in module_graph_modules.values() {
      let exports_id = mgm.exports;
      let is_module_without_exports = if let Some(ref build_meta) = mgm.build_meta {
        build_meta.exports_type == BuildMetaExportsType::Unset
      } else {
        true
      } && {
        let exports_info = self.mg.get_exports_info_by_id(&exports_id);
        let other_exports_info_id = exports_info.other_exports_info;
        let other_exports_info = self.mg.get_export_info_by_id(&other_exports_info_id);
        other_exports_info.provided.is_some()
      };

      // TODO: mem cache
      exports_id.set_has_provide_info(self.mg);
      q.push_back(mgm.module_identifier);
      if is_module_without_exports {
        exports_id.set_unknown_exports_provided(self.mg, false, None, None, None, None);
      }
    }
    self.mg.module_identifier_to_module_graph_module = module_graph_modules;

    while let Some(module_id) = q.pop_back() {
      self.changed = false;
      self.current_module_id = module_id;
      let mut exports_specs_from_dependencies: HashMap<DependencyId, ExportsSpec> =
        HashMap::default();
      self.process_dependencies_block(module_id, &mut exports_specs_from_dependencies);
      let exports_info_id = self.mg.get_exports_info(&module_id).id;
      for (dep_id, exports_spec) in exports_specs_from_dependencies.into_iter() {
        self.process_exports_spec(dep_id, exports_spec, exports_info_id);
      }
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
    // this is why we can bubble here. https://github.com/webpack/webpack/blob/ac7e531436b0d47cd88451f497cdfd0dad41535d/lib/FlagDependencyExportsPlugin.js#L140
    let exports_specs = dep.get_exports(self.mg)?;
    exports_specs_from_dependencies.insert(*dep_id, exports_specs);
    Some(())
  }

  pub fn process_exports_spec(
    &mut self,
    dep_id: DependencyId,
    export_desc: ExportsSpec,
    exports_info_id: ExportsInfoId,
  ) {
    let exports = &export_desc.exports;
    let global_can_mangle = &export_desc.can_mangle;
    let global_from = export_desc.from.as_ref();
    let global_priority = &export_desc.priority;
    let global_terminal_binding = export_desc.terminal_binding.unwrap_or(false);
    let export_dependencies = &export_desc.dependencies;
    if let Some(hide_export) = export_desc.hide_export {
      for name in hide_export.iter() {
        let from_exports_info_id = exports_info_id.get_export_info(name, self.mg);
        let export_info = self
          .mg
          .export_info_map
          .get_mut(&from_exports_info_id)
          .expect("should have export info");
        export_info.unset_target(&dep_id);
      }
    }
    match exports {
      ExportsOfExportsSpec::True => {
        exports_info_id.set_unknown_exports_provided(
          self.mg,
          global_can_mangle.unwrap_or_default(),
          export_desc.exclude_exports,
          global_from.map(|_| dep_id),
          global_from.cloned(),
          *global_priority,
        );
      }
      ExportsOfExportsSpec::Null => {}
      ExportsOfExportsSpec::Array(ele) => {
        // dbg!(ele);
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
        // dbg!(&ele, exports_info_id.get_exports_info(self.mg));
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
      // dbg!(&export_name_or_spec);
      let (name, can_mangle, terminal_binding, exports, from, from_export, priority, hidden) =
        match export_name_or_spec {
          ExportNameOrSpec::String(name) => (
            name.clone(),
            global_export_info.can_mangle,
            global_export_info.terminal_binding,
            None::<&Vec<ExportNameOrSpec>>,
            global_export_info.from.cloned(),
            None::<&rspack_core::Nullable<Vec<JsWord>>>,
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
              spec.from
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
      let export_info_id = exports_info.get_export_info(&name, self.mg);

      let mut export_info = self
        .mg
        .export_info_map
        .get_mut(&export_info_id)
        .expect("should have export info")
        .clone();
      // dbg!(&export_info);
      if let Some(ref mut provided) = export_info.provided
        && matches!(
          provided,
          ExportInfoProvided::False | ExportInfoProvided::Null
        )
      {
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
        // dbg!(&exports);
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
          export_info.unset_target(&dep_id)
        } else {
          let fallback = rspack_core::Nullable::Value(vec![name.clone()]);
          let export_name = if let Some(from) = from_export {
            Some(from)
          } else {
            Some(&(fallback))
          };
          // dbg!(&from, &export_name);
          export_info.set_target(Some(dep_id), Some(from), export_name, priority)
        };
        self.changed |= changed;
      }

      // Recalculate target exportsInfo
      let target = export_info.get_target(self.mg, None);
      // dbg!(&target);
      let export_info_old = self
        .mg
        .export_info_map
        .get_mut(&export_info_id)
        .expect("should have export info");
      _ = std::mem::replace(export_info_old, export_info);

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

      let export_info = self
        .mg
        .export_info_map
        .get_mut(&export_info_id)
        .expect("should have export info");
      if export_info.exports_info_owned {
        let changed = export_info
          .exports_info
          .expect("should have exports_info when exports_info_owned is true")
          .set_redirect_name_to(self.mg, target_exports_info);
        if changed {
          self.changed = true;
        }
      } else if export_info.exports_info != target_exports_info {
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

#[derive(Debug, Default)]
pub struct FlagDependencyExportsPlugin;

#[async_trait::async_trait]
impl Plugin for FlagDependencyExportsPlugin {
  async fn finish_modules(&self, compilation: &mut Compilation) -> Result<()> {
    let mut proxy = FlagDependencyExportsProxy::new(&mut compilation.module_graph);
    proxy.apply();
    Ok(())
  }
}
