use std::collections::VecDeque;

use rspack_core::tree_shaking::visitor::ModuleIdOrDepId;
use rspack_core::{
  BuildMetaExportsType, Compilation, Dependency, DependencyId, ExportsInfoId, ExportsType,
  ModuleGraph, ModuleIdentifier, ReferencedExport, RuntimeSpec, UsageState,
  WrappedReferencedExport,
};
use rustc_hash::FxHashMap as HashMap;

pub struct FlagDependencyUsagePlugin<'a> {
  global: bool,
  compilation: &'a mut Compilation,
  exports_info_module_map: HashMap<ExportsInfoId, ModuleIdentifier>,
}

impl<'a> FlagDependencyUsagePlugin<'a> {
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
    _ = std::mem::replace(&mut self.compilation.entries, entries);
  }

  fn process_entry_dependency(
    &mut self,
    dep: DependencyId,
    runtime: Option<RuntimeSpec>,
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
    used_exports: Vec<WrappedReferencedExport>,
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
          WrappedReferencedExport::Array(used_exports) => (true, used_exports),
          WrappedReferencedExport::Export(export) => (export.can_mangle, export.name),
        };
        if used_exports.len() == 0 {
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
    }
  }
}
