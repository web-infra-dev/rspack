use std::collections::hash_map::Entry;
use std::collections::VecDeque;

use rspack_core::{
  is_exports_object_referenced, is_no_exports_referenced, BuildMetaExportsType, Compilation,
  ConnectionState, DependencyId, ExportsInfoId, ExtendedReferencedExport, ModuleIdentifier, Plugin,
  ReferencedExport, RuntimeSpec, UsageState,
};
use rspack_error::Result;
use rspack_identifier::IdentifierMap;
use rustc_hash::FxHashMap as HashMap;

use crate::utils::join_jsword;

#[allow(unused)]
pub struct FlagDependencyUsagePluginProxy<'a> {
  global: bool,
  compilation: &'a mut Compilation,
  exports_info_module_map: HashMap<ExportsInfoId, ModuleIdentifier>,
}

#[allow(unused)]
impl<'a> FlagDependencyUsagePluginProxy<'a> {
  pub fn new(global: bool, compilation: &'a mut Compilation) -> Self {
    Self {
      global,
      compilation,
      exports_info_module_map: HashMap::default(),
    }
  }

  fn apply(&mut self) {
    for mgm in self
      .compilation
      .module_graph
      .module_graph_modules()
      .values()
    {
      self
        .exports_info_module_map
        .insert(mgm.exports, mgm.module_identifier);
    }
    let mut q = VecDeque::new();
    let mg = &mut self.compilation.module_graph;
    for exports_info_id in self.exports_info_module_map.keys() {
      exports_info_id.set_has_use_info(mg);
    }
    // SAFETY: we can make sure that entries will not be used other place at the same time,
    // this take is aiming to avoid use self ref and mut ref at the same time;
    let entries = std::mem::take(&mut self.compilation.entries);
    for entry in entries.values() {
      for &dep in entry.dependencies.iter() {
        self.process_entry_dependency(dep, None, &mut q);
      }
    }
    // TODO: compilation.globalEntry.dependencies, we don't have now https://github.com/webpack/webpack/blob/3f71468514ae2f179ff34c837ce82fcc8f97e24c/lib/FlagDependencyUsagePlugin.js#L328-L330
    self.compilation.entries = entries;

    while let Some((module_id, runtime)) = q.pop_front() {
      self.process_module(module_id, runtime, false, &mut q);
    }
  }

  fn process_module(
    &mut self,
    root_module_id: ModuleIdentifier,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
    q: &mut VecDeque<(ModuleIdentifier, Option<RuntimeSpec>)>,
  ) {
    #[derive(Debug)]
    enum ProcessModuleReferencedExports {
      Map(HashMap<String, ExtendedReferencedExport>),
      ExtendRef(Vec<ExtendedReferencedExport>),
    }

    let mut map: IdentifierMap<ProcessModuleReferencedExports> = IdentifierMap::default();
    let mut queue = VecDeque::new();
    queue.push_back(root_module_id);
    while let Some(module_id) = queue.pop_front() {
      // TODO: we don't have blocks.blocks https://github.com/webpack/webpack/blob/3f71468514ae2f179ff34c837ce82fcc8f97e24c/lib/FlagDependencyUsagePlugin.js#L180-L194
      let mgm = self
        .compilation
        .module_graph
        .module_graph_module_by_identifier(&module_id)
        .expect("should have module graph module");
      let dep_id_list = mgm.dependencies.clone();
      dbg!(&module_id);
      for dep_id in dep_id_list.into_iter() {
        let connection = self
          .compilation
          .module_graph
          .connection_by_dependency(&dep_id);
        let connection = if let Some(connection) = connection {
          connection
        } else {
          continue;
        };
        let active_state =
          connection.get_active_state(&self.compilation.module_graph, runtime.as_ref());
        match active_state {
          ConnectionState::Bool(false) => {
            continue;
          }
          ConnectionState::TransitiveOnly => {
            self.process_module(connection.module_identifier, runtime.clone(), false, q);
            continue;
          }
          _ => {}
        }
        let old_referenced_exports = map.remove(&connection.module_identifier);
        let dep = self
          .compilation
          .module_graph
          .dependency_by_id(&dep_id)
          .expect("should have dep");
        let referenced_exports = if let Some(md) = dep.as_module_dependency() {
          md.get_referenced_exports(&self.compilation.module_graph, runtime.as_ref())
        } else {
          continue;
        };

        dbg!(
          &connection,
          dep.as_module_dependency().unwrap().dependency_type()
        );
        dbg!(&referenced_exports, &old_referenced_exports);
        if old_referenced_exports.is_none()
          || matches!(old_referenced_exports.as_ref().expect("should be some"), ProcessModuleReferencedExports::ExtendRef(v) if is_no_exports_referenced(v))
          || is_exports_object_referenced(&referenced_exports)
        {
          map.insert(
            connection.module_identifier,
            ProcessModuleReferencedExports::ExtendRef(referenced_exports),
          );
        } else if old_referenced_exports.is_some() && is_no_exports_referenced(&referenced_exports)
        {
          map.insert(
            connection.module_identifier,
            old_referenced_exports.expect("should be Some"),
          );
          continue;
        } else {
          let mut exports_map = if let Some(old_referenced_exports) = old_referenced_exports {
            match old_referenced_exports {
              ProcessModuleReferencedExports::Map(map) => map,
              ProcessModuleReferencedExports::ExtendRef(ref_items) => {
                let mut exports_map = HashMap::default();
                for item in ref_items.iter() {
                  match item {
                    ExtendedReferencedExport::Array(arr) => {
                      exports_map.insert(join_jsword(arr, "\n"), item.clone());
                    }
                    ExtendedReferencedExport::Export(export) => {
                      exports_map.insert(join_jsword(&export.name, "\n"), item.clone());
                    }
                  }
                }
                exports_map
              }
            }
          } else {
            // in else branch above old_referenced_exports must be `Some(T)`, use if let Pattern
            // just avoid rust clippy complain
            unreachable!()
          };

          // FIXME: fix this issue
          for mut item in referenced_exports.into_iter() {
            match item {
              ExtendedReferencedExport::Array(ref arr) => {
                let key = join_jsword(arr, "\n");
                exports_map.entry(key).or_insert(item);
              }
              ExtendedReferencedExport::Export(ref mut export) => {
                let key = join_jsword(&export.name, "\n");
                match exports_map.entry(key) {
                  Entry::Occupied(mut occ) => {
                    let old_item = occ.get();
                    match old_item {
                      ExtendedReferencedExport::Array(_) => {
                        occ.insert(item);
                      }
                      ExtendedReferencedExport::Export(old_item) => {
                        occ.insert(ExtendedReferencedExport::Export(ReferencedExport {
                          name: std::mem::take(&mut export.name),
                          can_mangle: export.can_mangle && old_item.can_mangle,
                        }));
                      }
                    }
                  }
                  Entry::Vacant(vac) => {
                    vac.insert(item);
                  }
                }
              }
            }
          }
          dbg!(&exports_map);
          map.insert(
            connection.module_identifier,
            ProcessModuleReferencedExports::Map(exports_map),
          );
        }
      }
    }

    for (module_id, referenced_exports) in map {
      let normalized_refs = match referenced_exports {
        ProcessModuleReferencedExports::Map(map) => map.into_values().collect::<Vec<_>>(),
        ProcessModuleReferencedExports::ExtendRef(extend_ref) => extend_ref,
      };
      self.process_referenced_module(
        module_id,
        normalized_refs,
        runtime.clone(),
        force_side_effects,
        q,
      );
    }
  }

  fn process_entry_dependency(
    &mut self,
    dep: DependencyId,
    _runtime: Option<RuntimeSpec>,
    queue: &mut VecDeque<(ModuleIdentifier, Option<RuntimeSpec>)>,
  ) {
    if let Some(module) = self
      .compilation
      .module_graph
      .module_graph_module_by_dependency_id(&dep)
    {
      self.process_referenced_module(module.module_identifier, vec![], None, true, queue);
    }
  }

  /// TODO: currently we don't impl runtime optimization, runtime is always none
  fn process_referenced_module(
    &mut self,
    module_id: ModuleIdentifier,
    used_exports: Vec<ExtendedReferencedExport>,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
    queue: &mut VecDeque<(ModuleIdentifier, Option<RuntimeSpec>)>,
  ) {
    let mgm = self
      .compilation
      .module_graph
      .module_graph_module_by_identifier(&module_id)
      .expect("should have mgm");
    let mgm_exports_info_id = mgm.exports;
    if !used_exports.is_empty() {
      let need_insert = match mgm.build_meta {
        Some(ref build_meta) => matches!(build_meta.exports_type, BuildMetaExportsType::Unset),
        None => true,
      };
      if need_insert {
        let flag = mgm_exports_info_id
          .set_used_without_info(&mut self.compilation.module_graph, runtime.as_ref());
        if flag {
          queue.push_back((module_id, None));
        }
        return;
      }
      for used_export_info in used_exports {
        let (can_mangle, used_exports) = match used_export_info {
          ExtendedReferencedExport::Array(used_exports) => (true, used_exports),
          ExtendedReferencedExport::Export(export) => (export.can_mangle, export.name),
        };
        if used_exports.is_empty() {
          let flag = mgm_exports_info_id
            .set_used_in_unknown_way(&mut self.compilation.module_graph, runtime.as_ref());
          if flag {
            queue.push_back((module_id, runtime.clone()));
          }
        } else {
          let mut current_exports_info_id = mgm_exports_info_id;
          let len = used_exports.len();
          for (i, used_export) in used_exports.into_iter().enumerate() {
            let export_info_id = current_exports_info_id
              .get_export_info(&used_export, &mut self.compilation.module_graph);
            let export_info = self
              .compilation
              .module_graph
              .get_export_info_mut_by_id(&export_info_id);
            if !can_mangle {
              export_info.can_mangle_use = Some(false);
            }
            let last_one = i == len - 1;
            if !last_one {
              let nested_info =
                export_info_id.get_nested_exports_info(&self.compilation.module_graph);
              if let Some(nested_info) = nested_info {
                let changed_flag = export_info_id.set_used_conditionally(
                  &mut self.compilation.module_graph,
                  Box::new(|used| used == &UsageState::Unused),
                  UsageState::OnlyPropertiesUsed,
                  runtime.as_ref(),
                );
                if changed_flag {
                  let current_module = if current_exports_info_id == mgm_exports_info_id {
                    Some(module_id)
                  } else {
                    self
                      .exports_info_module_map
                      .get(&current_exports_info_id)
                      .cloned()
                  };
                  if let Some(current_module) = current_module {
                    queue.push_back((current_module, runtime.clone()));
                  }
                }
                current_exports_info_id = nested_info;
                continue;
              }
            }

            let changed_flag = export_info_id.set_used_conditionally(
              &mut self.compilation.module_graph,
              Box::new(|v| v != &UsageState::Used),
              UsageState::Used,
              runtime.as_ref(),
            );
            if changed_flag {
              // dbg!(&current_exports_info_id);
              let current_module = if current_exports_info_id == mgm_exports_info_id {
                Some(module_id)
              } else {
                self
                  .exports_info_module_map
                  .get(&current_exports_info_id)
                  .cloned()
              };
              if let Some(current_module) = current_module {
                queue.push_back((current_module, runtime.clone()));
              }
            }
            break;
          }
        }
      }
    } else {
      if !force_side_effects
        && match mgm.factory_meta {
          Some(ref meta) => meta.side_effect_free.unwrap_or_default(),
          None => false,
        }
      {
        return;
      }
      let changed_flag = mgm_exports_info_id
        .set_used_for_side_effects_only(&mut self.compilation.module_graph, runtime.as_ref());
      if changed_flag {
        queue.push_back((module_id, runtime));
      }
    }
  }
}

#[derive(Debug, Default)]
pub struct FlagDependencyUsagePlugin;

#[async_trait::async_trait]
impl Plugin for FlagDependencyUsagePlugin {
  async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<()>> {
    // TODO: `global` is always `true`, until we finished runtime optimization.
    let mut proxy = FlagDependencyUsagePluginProxy::new(true, compilation);
    proxy.apply();
    Ok(None)
  }
}
