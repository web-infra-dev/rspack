use std::collections::{hash_map::Entry, VecDeque};

use rayon::prelude::*;
use rspack_collections::{Identifier, IdentifierMap, UkeyMap};
use rspack_core::{
  get_entry_runtime, incremental::IncrementalPasses, is_exports_object_referenced,
  is_no_exports_referenced, AsyncDependenciesBlockIdentifier, BuildMetaExportsType, Compilation,
  CompilationOptimizeDependencies, ConnectionState, DependenciesBlock, DependencyId, ExportsInfo,
  ExtendedReferencedExport, GroupOptions, Inlinable, ModuleIdentifier, Plugin, ReferencedExport,
  RuntimeSpec, UsageState,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::swc::join_atom;
use rustc_hash::FxHashMap as HashMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum ModuleOrAsyncDependenciesBlock {
  Module(ModuleIdentifier),
  AsyncDependenciesBlock(AsyncDependenciesBlockIdentifier),
}

#[derive(Debug, Clone)]
enum ProcessModuleReferencedExports {
  Map(HashMap<String, ExtendedReferencedExport>),
  ExtendRef(Vec<ExtendedReferencedExport>),
}
#[allow(unused)]
pub struct FlagDependencyUsagePluginProxy<'a> {
  global: bool,
  compilation: &'a mut Compilation,
  exports_info_module_map: UkeyMap<ExportsInfo, ModuleIdentifier>,
}

#[allow(unused)]
impl<'a> FlagDependencyUsagePluginProxy<'a> {
  pub fn new(global: bool, compilation: &'a mut Compilation) -> Self {
    Self {
      global,
      compilation,
      exports_info_module_map: UkeyMap::default(),
    }
  }

  fn apply(&mut self) {
    let mut module_graph = self.compilation.get_module_graph_mut();
    for mgm in module_graph.module_graph_modules().values() {
      self
        .exports_info_module_map
        .insert(mgm.exports, mgm.module_identifier);
    }
    let mut batch = Vec::new();

    let mg = &mut module_graph;
    for exports_info in self.exports_info_module_map.keys() {
      exports_info.set_has_use_info(mg);
    }
    // SAFETY: we can make sure that entries will not be used other place at the same time,
    // this take is aiming to avoid use self ref and mut ref at the same time;
    let mut global_runtime: Option<RuntimeSpec> = None;
    let entries = std::mem::take(&mut self.compilation.entries);
    for (entry_name, entry) in entries.iter() {
      let runtime = if self.global {
        None
      } else {
        Some(get_entry_runtime(entry_name, &entry.options, &entries))
      };
      if let Some(runtime) = runtime.as_ref() {
        global_runtime.get_or_insert_default().extend(runtime);
      }
      for &dep in entry.dependencies.iter() {
        self.process_entry_dependency(dep, runtime.clone(), &mut batch);
      }
      for &dep in entry.include_dependencies.iter() {
        self.process_entry_dependency(dep, runtime.clone(), &mut batch);
      }
    }
    for dep in self.compilation.global_entry.dependencies.clone() {
      self.process_entry_dependency(dep, global_runtime.clone(), &mut batch);
    }
    for dep in self.compilation.global_entry.include_dependencies.clone() {
      self.process_entry_dependency(dep, global_runtime.clone(), &mut batch);
    }
    self.compilation.entries = entries;

    while !batch.is_empty() {
      let modules = std::mem::take(&mut batch);
      let module_graph = self.compilation.get_module_graph();
      let module_graph_cache = &self.compilation.module_graph_cache_artifact;
      let mut module_referenced_exports = modules
        .into_par_iter()
        .map(|(module_id, runtime)| {
          self.collect_module_referenced_exports(
            ModuleOrAsyncDependenciesBlock::Module(module_id),
            runtime,
            false,
          )
        })
        .collect::<Vec<_>>();

      for referenced_exports in module_referenced_exports {
        for (module_id, referenced_exports, runtime, force_side_effects) in referenced_exports {
          let normalized_refs = match referenced_exports {
            ProcessModuleReferencedExports::Map(map) => map.into_values().collect::<Vec<_>>(),
            ProcessModuleReferencedExports::ExtendRef(extend_ref) => extend_ref,
          };
          self.process_referenced_module(
            module_id,
            normalized_refs,
            runtime.clone(),
            force_side_effects,
            &mut batch,
          );
        }
      }
    }
  }

  fn collect_module_referenced_exports(
    &self,
    block_id: ModuleOrAsyncDependenciesBlock,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
  ) -> Vec<(
    Identifier,
    ProcessModuleReferencedExports,
    Option<RuntimeSpec>,
    bool,
  )> {
    let mut res = Vec::new();
    let mut map: IdentifierMap<ProcessModuleReferencedExports> = IdentifierMap::default();
    let mut queue = VecDeque::new();
    queue.push_back(block_id);
    while let Some(module_id) = queue.pop_front() {
      let module_graph = self.compilation.get_module_graph();
      let (blocks, dependencies) = match module_id {
        ModuleOrAsyncDependenciesBlock::Module(module) => {
          let block = module_graph
            .module_by_identifier(&module)
            .expect("should have module");
          (block.get_blocks(), block.get_dependencies())
        }
        ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(async_dependencies_block_id) => {
          let block = module_graph
            .block_by_id(&async_dependencies_block_id)
            .expect("should have module");
          (block.get_blocks(), block.get_dependencies())
        }
      };
      let (block_id_list, dep_id_list) = (blocks.to_vec(), dependencies.to_vec());
      for block_id in block_id_list {
        let module_graph = self.compilation.get_module_graph();
        let block = module_graph
          .block_by_id(&block_id)
          .expect("should have block");
        if !self.global
          && let Some(GroupOptions::Entrypoint(options)) = block.get_group_options()
        {
          let runtime = RuntimeSpec::from_entry_options(options);
          res.extend(self.collect_module_referenced_exports(
            ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(block_id),
            runtime,
            true,
          ));
        } else {
          queue.push_back(ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(
            block_id,
          ));
        }
      }
      for dep_id in dep_id_list.into_iter() {
        let module_graph = self.compilation.get_module_graph();
        let module_graph_cache = &self.compilation.module_graph_cache_artifact;
        let connection = module_graph.connection_by_dependency_id(&dep_id);

        let connection = if let Some(connection) = connection {
          connection
        } else {
          continue;
        };
        let active_state =
          connection.active_state(&module_graph, runtime.as_ref(), module_graph_cache);

        match active_state {
          ConnectionState::Active(false) => {
            continue;
          }
          ConnectionState::TransitiveOnly => {
            res.extend(self.collect_module_referenced_exports(
              ModuleOrAsyncDependenciesBlock::Module(*connection.module_identifier()),
              runtime.clone(),
              false,
            ));
            continue;
          }
          _ => {}
        }
        let old_referenced_exports = map.remove(connection.module_identifier());
        let dep = module_graph
          .dependency_by_id(&dep_id)
          .expect("should have dep");

        let referenced_exports = if let Some(md) = dep.as_module_dependency() {
          md.get_referenced_exports(&module_graph, module_graph_cache, runtime.as_ref())
        } else if dep.as_context_dependency().is_some() {
          vec![ExtendedReferencedExport::Array(vec![])]
        } else {
          continue;
        };

        if old_referenced_exports.is_none()
          || matches!(old_referenced_exports, Some(ProcessModuleReferencedExports::ExtendRef(ref v)) if is_no_exports_referenced(v))
          || is_exports_object_referenced(&referenced_exports)
        {
          map.insert(
            *connection.module_identifier(),
            ProcessModuleReferencedExports::ExtendRef(referenced_exports),
          );
        } else if let Some(old_referenced_exports) = old_referenced_exports {
          if is_no_exports_referenced(&referenced_exports) {
            map.insert(
              *connection.module_identifier(),
              old_referenced_exports.clone(),
            );
            continue;
          }

          let mut exports_map = match old_referenced_exports {
            ProcessModuleReferencedExports::Map(map) => map,
            ProcessModuleReferencedExports::ExtendRef(ref_items) => {
              let mut exports_map = HashMap::default();
              for item in ref_items.iter() {
                match item {
                  ExtendedReferencedExport::Array(arr) => {
                    exports_map.insert(join_atom(arr.iter(), "\n"), item.clone());
                  }
                  ExtendedReferencedExport::Export(export) => {
                    exports_map.insert(join_atom(export.name.iter(), "\n"), item.clone());
                  }
                }
              }
              exports_map
            }
          };

          for mut item in referenced_exports.into_iter() {
            match item {
              ExtendedReferencedExport::Array(ref arr) => {
                let key = join_atom(arr.iter(), "\n");
                exports_map.entry(key).or_insert(item);
              }
              ExtendedReferencedExport::Export(ref mut export) => {
                let key = join_atom(export.name.iter(), "\n");
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
                          can_inline: export.can_inline && old_item.can_inline,
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
          map.insert(
            *connection.module_identifier(),
            ProcessModuleReferencedExports::Map(exports_map),
          );
        }
      }
    }

    res.extend(map.into_iter().map(|(module_id, referenced_exports)| {
      (
        module_id,
        referenced_exports,
        runtime.clone(),
        force_side_effects,
      )
    }));

    res
  }

  fn process_entry_dependency(
    &mut self,
    dep: DependencyId,
    runtime: Option<RuntimeSpec>,
    batch: &mut Vec<(Identifier, Option<RuntimeSpec>)>,
  ) {
    if let Some(module) = self
      .compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(&dep)
    {
      self.process_referenced_module(module.module_identifier, vec![], runtime, true, batch);
    }
  }

  fn process_referenced_module(
    &mut self,
    module_id: ModuleIdentifier,
    used_exports: Vec<ExtendedReferencedExport>,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
    batch: &mut Vec<(Identifier, Option<RuntimeSpec>)>,
  ) {
    let mut module_graph = self.compilation.get_module_graph_mut();
    let mgm = module_graph
      .module_graph_module_by_identifier(&module_id)
      .expect("should have mgm");
    let module = module_graph
      .module_by_identifier(&module_id)
      .expect("should have module");
    let mgm_exports_info = mgm.exports;

    // Special handling for ConsumeShared modules
    // ConsumeShared modules need enhanced usage tracking to work properly with tree-shaking
    if module.module_type() == &rspack_core::ModuleType::ConsumeShared {
      self.process_consume_shared_module(
        module_id,
        used_exports,
        runtime,
        force_side_effects,
        batch,
      );
      return;
    }
    if !used_exports.is_empty() {
      let need_insert = matches!(
        module.build_meta().exports_type,
        BuildMetaExportsType::Unset
      );
      if need_insert {
        let flag = mgm_exports_info.set_used_without_info(&mut module_graph, runtime.as_ref());
        if flag {
          batch.push((module_id, None));
        }
        return;
      }

      for used_export_info in used_exports {
        let (used_exports, can_mangle, can_inline) = match used_export_info {
          ExtendedReferencedExport::Array(used_exports) => (used_exports, true, true),
          ExtendedReferencedExport::Export(export) => {
            (export.name, export.can_mangle, export.can_inline)
          }
        };
        if used_exports.is_empty() {
          let flag = mgm_exports_info.set_used_in_unknown_way(&mut module_graph, runtime.as_ref());

          if flag {
            batch.push((module_id, runtime.clone()));
          }
        } else {
          let mut current_exports_info = mgm_exports_info;
          let len = used_exports.len();
          for (i, used_export) in used_exports.into_iter().enumerate() {
            let export_info = current_exports_info
              .get_export_info(&mut module_graph, &used_export)
              .as_data_mut(&mut module_graph);
            if !can_mangle {
              export_info.set_can_mangle_use(Some(false));
            }
            if !can_inline {
              export_info.set_inlinable(Inlinable::NoByUse);
            }
            let last_one = i == len - 1;
            if !last_one {
              let nested_info = export_info.exports_info();
              if let Some(nested_info) = nested_info {
                let changed_flag = export_info.set_used_conditionally(
                  Box::new(|used| used == &UsageState::Unused),
                  UsageState::OnlyPropertiesUsed,
                  runtime.as_ref(),
                );
                if changed_flag {
                  let current_module = if current_exports_info == mgm_exports_info {
                    Some(module_id)
                  } else {
                    self
                      .exports_info_module_map
                      .get(&current_exports_info)
                      .cloned()
                  };
                  if let Some(current_module) = current_module {
                    batch.push((current_module, runtime.clone()));
                  }
                }
                current_exports_info = nested_info;
                continue;
              }
            }

            let changed_flag = export_info.set_used_conditionally(
              Box::new(|v| v != &UsageState::Used),
              UsageState::Used,
              runtime.as_ref(),
            );
            if changed_flag {
              let current_module = if current_exports_info == mgm_exports_info {
                Some(module_id)
              } else {
                self
                  .exports_info_module_map
                  .get(&current_exports_info)
                  .cloned()
              };
              if let Some(current_module) = current_module {
                batch.push((current_module, runtime.clone()));
              }
            }
            break;
          }
        }
      }
    } else {
      if !force_side_effects
        && match module.factory_meta() {
          Some(meta) => meta.side_effect_free.unwrap_or_default(),
          None => false,
        }
      {
        return;
      }
      let changed_flag = mgm_exports_info
        .as_data_mut(&mut module_graph)
        .set_used_for_side_effects_only(runtime.as_ref());
      if changed_flag {
        batch.push((module_id, runtime));
      }
    }
  }

  /// Enhanced processing for ConsumeShared modules
  /// This ensures ConsumeShared modules get proper usage tracking like normal modules
  fn process_consume_shared_module(
    &mut self,
    module_id: ModuleIdentifier,
    used_exports: Vec<ExtendedReferencedExport>,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
    batch: &mut Vec<(Identifier, Option<RuntimeSpec>)>,
  ) {
    let mut module_graph = self.compilation.get_module_graph_mut();
    let mgm = module_graph
      .module_graph_module_by_identifier(&module_id)
      .expect("should have mgm");
    let mgm_exports_info = mgm.exports;

    // Process ConsumeShared modules the same as normal modules for usage tracking
    // This allows proper tree-shaking of ConsumeShared exports
    if !used_exports.is_empty() {
      // Handle export usage same as normal modules
      for used_export_info in used_exports {
        let (used_exports, can_mangle, can_inline) = match used_export_info {
          rspack_core::ExtendedReferencedExport::Array(used_exports) => (used_exports, true, true),
          rspack_core::ExtendedReferencedExport::Export(export) => {
            (export.name, export.can_mangle, export.can_inline)
          }
        };

        if used_exports.is_empty() {
          // Namespace usage - mark all exports as used in unknown way
          let flag = mgm_exports_info.set_used_in_unknown_way(&mut module_graph, runtime.as_ref());
          if flag {
            batch.push((module_id, runtime.clone()));
          }
        } else {
          // Specific export usage - process each export in the path
          let mut current_exports_info = mgm_exports_info;
          let len = used_exports.len();

          for (i, used_export) in used_exports.into_iter().enumerate() {
            let export_info = current_exports_info.get_export_info(&mut module_graph, &used_export);

            // Apply mangling and inlining constraints
            if !can_mangle {
              export_info
                .as_data_mut(&mut module_graph)
                .set_can_mangle_use(Some(false));
            }
            if !can_inline {
              export_info
                .as_data_mut(&mut module_graph)
                .set_inlinable(rspack_core::Inlinable::NoByUse);
            }

            let last_one = i == len - 1;
            if !last_one {
              // Intermediate property access - mark as OnlyPropertiesUsed
              let nested_info = export_info.as_data(&module_graph).exports_info();
              if let Some(nested_info) = nested_info {
                let changed_flag = export_info
                  .as_data_mut(&mut module_graph)
                  .set_used_conditionally(
                    Box::new(|used| used == &rspack_core::UsageState::Unused),
                    rspack_core::UsageState::OnlyPropertiesUsed,
                    runtime.as_ref(),
                  );
                if changed_flag {
                  let current_module = if current_exports_info == mgm_exports_info {
                    Some(module_id)
                  } else {
                    self
                      .exports_info_module_map
                      .get(&current_exports_info)
                      .cloned()
                  };
                  if let Some(current_module) = current_module {
                    batch.push((current_module, runtime.clone()));
                  }
                }
                current_exports_info = nested_info;
                continue;
              }
            }

            // Final property or direct export - mark as Used
            let changed_flag = export_info
              .as_data_mut(&mut module_graph)
              .set_used_conditionally(
                Box::new(|v| v != &rspack_core::UsageState::Used),
                rspack_core::UsageState::Used,
                runtime.as_ref(),
              );
            if changed_flag {
              let current_module = if current_exports_info == mgm_exports_info {
                Some(module_id)
              } else {
                self
                  .exports_info_module_map
                  .get(&current_exports_info)
                  .cloned()
              };
              if let Some(current_module) = current_module {
                batch.push((current_module, runtime.clone()));
              }
            }
            break;
          }
        }
      }
    } else {
      // No specific exports used - handle side effects
      let changed_flag = mgm_exports_info
        .as_data_mut(&mut module_graph)
        .set_used_for_side_effects_only(runtime.as_ref());
      if changed_flag {
        batch.push((module_id, runtime));
      }
    }
  }
}

#[plugin]
#[derive(Debug)]
pub struct FlagDependencyUsagePlugin {
  global: bool,
}

impl FlagDependencyUsagePlugin {
  pub fn new(global: bool) -> Self {
    Self::new_inner(global)
  }
}

#[plugin_hook(CompilationOptimizeDependencies for FlagDependencyUsagePlugin)]
async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if let Some(diagnostic) = compilation.incremental.disable_passes(
    IncrementalPasses::MODULES_HASHES,
    "FlagDependencyUsagePlugin (optimization.usedExports = true)",
    "it requires calculating the used exports based on all modules, which is a global effect",
  ) {
    if let Some(diagnostic) = diagnostic {
      compilation.push_diagnostic(diagnostic);
    }
    compilation.cgm_hash_artifact.clear();
  }

  let mut proxy = FlagDependencyUsagePluginProxy::new(self.global, compilation);
  proxy.apply();
  Ok(None)
}

impl Plugin for FlagDependencyUsagePlugin {
  fn apply(
    &self,
    ctx: rspack_core::PluginContext<&mut rspack_core::ApplyContext>,
    _options: &rspack_core::CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    Ok(())
  }
}
