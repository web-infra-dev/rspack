use std::collections::{VecDeque, hash_map::Entry};

use rayon::prelude::*;
use rspack_collections::{IdentifierMap, UkeyMap};
use rspack_core::{
  AsyncDependenciesBlockIdentifier, BuildMetaExportsType, Compilation,
  CompilationOptimizeDependencies, ConnectionState, DependenciesBlock, DependencyId, ExportsInfo,
  ExportsInfoData, ExtendedReferencedExport, GroupOptions, Inlinable, ModuleGraph,
  ModuleGraphCacheArtifact, ModuleIdentifier, Plugin, ReferencedExport, RuntimeSpec, UsageState,
  get_entry_runtime, incremental::IncrementalPasses, is_exports_object_referenced,
  is_no_exports_referenced,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::{queue::Queue, swc::join_atom};
use rustc_hash::FxHashMap as HashMap;
use serde::Deserialize;

type ProcessBlockTask = (ModuleOrAsyncDependenciesBlock, Option<RuntimeSpec>, bool);
type NonNestedTask = (Option<RuntimeSpec>, bool, Vec<ExtendedReferencedExport>);

#[derive(Deserialize)]
struct ShareUsageReport {
  #[serde(rename = "treeShake", default)]
  tree_shake: std::collections::HashMap<String, ShareUsageEntry>,
}

#[derive(Deserialize)]
struct ShareUsageEntry {
  #[serde(flatten)]
  exports: std::collections::HashMap<String, bool>,
  #[serde(rename = "chunkCharacteristics", default)]
  _chunk_characteristics: serde_json::Value,
}

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
    let context_path = self.compilation.options.context.as_path().to_path_buf();
    let mut module_graph = self.compilation.get_module_graph_mut();

    // Apply shared module usage information from share-usage.json if available
    let usage_path = context_path.join("share-usage.json");
    if let Ok(content) = std::fs::read_to_string(&usage_path) {
      if let Ok(report) = serde_json::from_str::<ShareUsageReport>(&content) {
        let module_ids: Vec<_> = module_graph.modules().keys().copied().collect();
        for module_id in module_ids {
          let module = module_graph
            .module_by_identifier(&module_id)
            .expect("module not found");
          let shared_key = {
            let meta = module.build_meta();
            meta
              .effective_shared_key
              .clone()
              .or_else(|| meta.shared_key.clone())
          };
          if let Some(key) = shared_key {
            if let Some(usage) = report.tree_shake.get(&key) {
              let exports_info = module_graph.get_exports_info(&module_id);
              let exports_info_data = exports_info.as_data_mut(&mut module_graph);
              for (export_name, used) in &usage.exports {
                let export_atom = rspack_util::atom::Atom::from(export_name.as_str());
                let info = exports_info_data.ensure_owned_export_info(&export_atom);
                let state = if *used {
                  UsageState::Used
                } else {
                  UsageState::Unused
                };
                info.set_used(state, None);
              }
            }
          }
        }
      }
    }

    for mgm in module_graph.module_graph_modules().values() {
      self
        .exports_info_module_map
        .insert(mgm.exports, mgm.module_identifier);
    }
    let mut q = Queue::new();
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
        self.process_entry_dependency(dep, runtime.clone(), &mut q);
      }
      for &dep in entry.include_dependencies.iter() {
        self.process_entry_dependency(dep, runtime.clone(), &mut q);
      }
    }
    for dep in self.compilation.global_entry.dependencies.clone() {
      self.process_entry_dependency(dep, global_runtime.clone(), &mut q);
    }
    for dep in self.compilation.global_entry.include_dependencies.clone() {
      self.process_entry_dependency(dep, global_runtime.clone(), &mut q);
    }
    self.compilation.entries = entries;

    loop {
      let mut batch = vec![];
      while let Some(task) = q.dequeue() {
        batch.push(task);
      }

      self.compilation.module_graph_cache_artifact.freeze();

      // collect referenced exports from modules by calling `dependency.get_referenced_exports`
      // and also added referenced modules to the queue for further processing
      let batch_res = batch
        .into_par_iter()
        .map(|(block_id, runtime, force_side_effects)| {
          let (referenced_exports, module_tasks) =
            self.process_module(block_id, runtime.as_ref(), force_side_effects, self.global);
          (
            runtime,
            force_side_effects,
            referenced_exports,
            module_tasks,
          )
        })
        .collect::<Vec<_>>();

      let mut nested_tasks = vec![];
      let mut non_nested_tasks: IdentifierMap<Vec<NonNestedTask>> = IdentifierMap::default();

      {
        // partition collected referenced exports to two parts:
        // 1. if the exports info data has `redirect_to`, the redirected exports info will also be modified, so the referenced exports should not be processed parallelly
        // 2. if the referenced exports has nested properties, the nested exports info will also be modified, the referenced exports should not be processed parallelly

        let mg = self.compilation.get_module_graph();

        let collected = batch_res
          .into_par_iter()
          .map(
            |(runtime, force_side_effects, referenced_exports, module_tasks)| {
              let mut nested_tasks = vec![];
              let mut non_nested_tasks = vec![];
              for (module_id, exports) in referenced_exports {
                let exports_info = mg.get_exports_info(&module_id).as_data(&mg);
                let has_nested = exports_info.redirect_to().is_some()
                  || exports.iter().any(|e| match e {
                    ExtendedReferencedExport::Array(arr) => arr.len() > 1,
                    ExtendedReferencedExport::Export(export) => export.name.len() > 1,
                  });
                if has_nested {
                  nested_tasks.push((
                    runtime.clone(),
                    force_side_effects,
                    module_id,
                    exports_info.id(),
                    exports,
                  ));
                } else {
                  non_nested_tasks
                    .push((module_id, (runtime.clone(), force_side_effects, exports)));
                }
              }
              (nested_tasks, non_nested_tasks, module_tasks)
            },
          )
          .collect::<Vec<_>>();

        for (module_nested_tasks, module_non_nested_tasks, module_tasks) in collected {
          for i in module_tasks {
            q.enqueue(i);
          }
          for (module_id, task) in module_non_nested_tasks {
            non_nested_tasks.entry(module_id).or_default().push(task);
          }
          nested_tasks.extend(module_nested_tasks);
        }
      }

      // we can ensure that only the module's exports info data will be modified
      // so we can process these non-nested tasks parallelly by cloning the exports info data
      let non_nested_res = {
        let mg = self.compilation.get_module_graph();
        non_nested_tasks
          .into_par_iter()
          .map(|(module_id, tasks)| {
            let mut exports_info = mg.get_exports_info(&module_id).as_data(&mg).clone();
            let module = mg
              .module_by_identifier(&module_id)
              .expect("should have module");
            let is_exports_type_unset = matches!(
              module.build_meta().exports_type,
              BuildMetaExportsType::Unset
            );
            let is_side_effect_free = match module.factory_meta() {
              Some(meta) => meta.side_effect_free.unwrap_or_default(),
              None => false,
            };

            let mut res = vec![];
            for (runtime, force_side_effects, exports) in tasks {
              let module_res = process_referenced_module_without_nested(
                module_id,
                is_exports_type_unset,
                is_side_effect_free,
                &mut exports_info,
                exports,
                runtime,
                force_side_effects,
              );
              res.extend(module_res);
            }
            (exports_info, res)
          })
          .collect::<Vec<_>>()
      };

      {
        // after processing, we will set the exports info data back to the module graph
        let mut mg = self.compilation.get_module_graph_mut();
        for (exports_info, res) in non_nested_res {
          for i in res {
            q.enqueue(i);
          }

          mg.set_exports_info(exports_info.id(), exports_info);
        }
      }

      // for nested tasks, just process them one by one to prevent conflicts while modifying the exports info data
      for (runtime, force_side_effects, module_id, exports_info, referenced_exports) in nested_tasks
      {
        let res = self.process_referenced_module(
          exports_info,
          module_id,
          referenced_exports,
          runtime.clone(),
          force_side_effects,
        );
        for i in res {
          q.enqueue(i);
        }
      }

      self.compilation.module_graph_cache_artifact.unfreeze();

      if q.is_empty() {
        break;
      }
    }
  }

  fn process_module(
    &self,
    block_id: ModuleOrAsyncDependenciesBlock,
    runtime: Option<&RuntimeSpec>,
    force_side_effects: bool,
    global: bool,
  ) -> (
    IdentifierMap<Vec<ExtendedReferencedExport>>,
    Vec<ProcessBlockTask>,
  ) {
    let mut q = vec![];
    let mut map: IdentifierMap<ProcessModuleReferencedExports> = IdentifierMap::default();

    let (dependencies, async_blocks) = collect_active_dependencies(
      block_id,
      runtime,
      &self.compilation.get_module_graph(),
      &self.compilation.module_graph_cache_artifact,
      global,
    );
    q.extend(async_blocks);

    for (dep_id, module_id) in dependencies.into_iter() {
      let old_referenced_exports = map.remove(&module_id);
      let Some(referenced_exports) = get_dependency_referenced_exports(
        dep_id,
        &self.compilation.get_module_graph(),
        &self.compilation.module_graph_cache_artifact,
        runtime,
      ) else {
        continue;
      };

      if let Some(new_referenced_exports) =
        merge_referenced_exports(old_referenced_exports, referenced_exports)
      {
        map.insert(module_id, new_referenced_exports);
      }
    }

    (
      map
        .into_iter()
        .map(|(module_id, referenced_exports)| {
          (
            module_id,
            match referenced_exports {
              ProcessModuleReferencedExports::Map(map) => map.into_values().collect::<Vec<_>>(),
              ProcessModuleReferencedExports::ExtendRef(extend_ref) => extend_ref,
            },
          )
        })
        .collect::<IdentifierMap<_>>(),
      q,
    )
  }

  fn process_entry_dependency(
    &mut self,
    dep: DependencyId,
    runtime: Option<RuntimeSpec>,
    queue: &mut Queue<ProcessBlockTask>,
  ) {
    if let Some(module) = self
      .compilation
      .get_module_graph()
      .module_graph_module_by_dependency_id(&dep)
    {
      let mg = self.compilation.get_module_graph();
      let exports_info = mg.get_exports_info(&module.module_identifier);
      let res = self.process_referenced_module(
        exports_info,
        module.module_identifier,
        vec![],
        runtime,
        true,
      );
      for i in res {
        queue.enqueue(i);
      }
    }
  }

  fn process_referenced_module(
    &mut self,
    mgm_exports_info: ExportsInfo,
    module_id: ModuleIdentifier,
    used_exports: Vec<ExtendedReferencedExport>,
    runtime: Option<RuntimeSpec>,
    force_side_effects: bool,
  ) -> Vec<ProcessBlockTask> {
    let mut queue = vec![];
    let mut module_graph = self.compilation.get_module_graph_mut();
    let module = module_graph
      .module_by_identifier(&module_id)
      .expect("should have module");
    if !used_exports.is_empty() {
      let need_insert = matches!(
        module.build_meta().exports_type,
        BuildMetaExportsType::Unset
      );
      if need_insert {
        let flag = mgm_exports_info.set_used_without_info(&mut module_graph, runtime.as_ref());
        if flag {
          queue.push((
            ModuleOrAsyncDependenciesBlock::Module(module_id),
            None,
            false,
          ));
        }
        return queue;
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
            queue.push((
              ModuleOrAsyncDependenciesBlock::Module(module_id),
              runtime.clone(),
              false,
            ));
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
            if !last_one && let Some(nested_info) = export_info.exports_info() {
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
                    .copied()
                };
                if let Some(current_module) = current_module {
                  queue.push((
                    ModuleOrAsyncDependenciesBlock::Module(current_module),
                    runtime.clone(),
                    false,
                  ));
                }
              }
              current_exports_info = nested_info;
              continue;
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
                queue.push((
                  ModuleOrAsyncDependenciesBlock::Module(current_module),
                  runtime.clone(),
                  false,
                ));
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
        return queue;
      }
      let changed_flag = mgm_exports_info
        .as_data_mut(&mut module_graph)
        .set_used_for_side_effects_only(runtime.as_ref());
      if changed_flag {
        queue.push((
          ModuleOrAsyncDependenciesBlock::Module(module_id),
          runtime,
          false,
        ));
      }
    }
    queue
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
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    Ok(())
  }
}

fn merge_referenced_exports(
  old_referenced_exports: Option<ProcessModuleReferencedExports>,
  referenced_exports: Vec<ExtendedReferencedExport>,
) -> Option<ProcessModuleReferencedExports> {
  if old_referenced_exports.is_none()
    || matches!(old_referenced_exports, Some(ProcessModuleReferencedExports::ExtendRef(ref v)) if is_no_exports_referenced(v))
    || is_exports_object_referenced(&referenced_exports)
  {
    return Some(ProcessModuleReferencedExports::ExtendRef(
      referenced_exports,
    ));
  } else if let Some(old_referenced_exports) = old_referenced_exports {
    if is_no_exports_referenced(&referenced_exports) {
      return Some(old_referenced_exports);
    }

    let mut exports_map = match old_referenced_exports {
      ProcessModuleReferencedExports::Map(map) => map,
      ProcessModuleReferencedExports::ExtendRef(ref_items) => ref_items
        .into_iter()
        .map(|item| {
          let key = match &item {
            ExtendedReferencedExport::Array(arr) => join_atom(arr.iter(), "\n"),
            ExtendedReferencedExport::Export(export) => join_atom(export.name.iter(), "\n"),
          };
          (key, item)
        })
        .collect::<HashMap<_, _>>(),
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
    return Some(ProcessModuleReferencedExports::Map(exports_map));
  }
  None
}

fn collect_active_dependencies(
  block_id: ModuleOrAsyncDependenciesBlock,
  runtime: Option<&RuntimeSpec>,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  global: bool,
) -> (Vec<(DependencyId, ModuleIdentifier)>, Vec<ProcessBlockTask>) {
  let mut q = vec![];
  let mut queue = VecDeque::new();
  let mut dependencies = vec![];
  queue.push_back(block_id);
  while let Some(block_id) = queue.pop_front() {
    let (blocks, block_dependencies) = match block_id {
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
    dependencies.extend(block_dependencies);
    for block_id in blocks {
      let block = module_graph
        .block_by_id(block_id)
        .expect("should have block");
      if !global && let Some(GroupOptions::Entrypoint(options)) = block.get_group_options() {
        let runtime = RuntimeSpec::from_entry_options(options);
        q.push((
          ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(*block_id),
          runtime,
          true,
        ));
      } else {
        queue.push_back(ModuleOrAsyncDependenciesBlock::AsyncDependenciesBlock(
          *block_id,
        ));
      }
    }
  }

  let dependencies = dependencies
    .into_iter()
    .filter_map(|dep_id| {
      let connection = module_graph.connection_by_dependency_id(&dep_id)?;
      let active_state = connection.active_state(module_graph, runtime, module_graph_cache);

      match active_state {
        ConnectionState::Active(false) => {
          return None;
        }
        ConnectionState::TransitiveOnly => {
          q.push((
            ModuleOrAsyncDependenciesBlock::Module(*connection.module_identifier()),
            runtime.cloned(),
            false,
          ));
          return None;
        }
        _ => {}
      }
      Some((dep_id, *connection.module_identifier()))
    })
    .collect::<Vec<_>>();

  (dependencies, q)
}

fn get_dependency_referenced_exports(
  dep_id: DependencyId,
  module_graph: &ModuleGraph,
  module_graph_cache: &ModuleGraphCacheArtifact,
  runtime: Option<&RuntimeSpec>,
) -> Option<Vec<ExtendedReferencedExport>> {
  let dep = module_graph
    .dependency_by_id(&dep_id)
    .expect("should have dep");

  if let Some(md) = dep.as_module_dependency() {
    Some(md.get_referenced_exports(module_graph, module_graph_cache, runtime))
  } else if dep.as_context_dependency().is_some() {
    Some(vec![ExtendedReferencedExport::Array(vec![])])
  } else {
    None
  }
}

fn process_referenced_module_without_nested(
  module_id: ModuleIdentifier,
  is_exports_type_unset: bool,
  is_side_effect_free: bool,
  exports_info: &mut ExportsInfoData,
  used_exports: Vec<ExtendedReferencedExport>,
  runtime: Option<RuntimeSpec>,
  force_side_effects: bool,
) -> Vec<ProcessBlockTask> {
  let mut queue = vec![];
  if !used_exports.is_empty() {
    if is_exports_type_unset {
      let flag = exports_info.set_owned_used_without_info(runtime.as_ref());
      if flag {
        queue.push((
          ModuleOrAsyncDependenciesBlock::Module(module_id),
          None,
          false,
        ));
      }
      return queue;
    }

    for used_export_info in used_exports {
      let (used_exports, can_mangle, can_inline) = match used_export_info {
        ExtendedReferencedExport::Array(used_exports) => (used_exports, true, true),
        ExtendedReferencedExport::Export(export) => {
          (export.name, export.can_mangle, export.can_inline)
        }
      };
      if used_exports.is_empty() {
        let flag = exports_info.set_owned_used_in_unknown_way(runtime.as_ref());

        if flag {
          queue.push((
            ModuleOrAsyncDependenciesBlock::Module(module_id),
            runtime.clone(),
            false,
          ));
        }
      } else {
        let used_export = &used_exports[0];
        let export_info = exports_info.ensure_owned_export_info(used_export);
        if !can_mangle {
          export_info.set_can_mangle_use(Some(false));
        }
        if !can_inline {
          export_info.set_inlinable(Inlinable::NoByUse);
        }

        let changed_flag = export_info.set_used_conditionally(
          Box::new(|v| v != &UsageState::Used),
          UsageState::Used,
          runtime.as_ref(),
        );
        if changed_flag {
          queue.push((
            ModuleOrAsyncDependenciesBlock::Module(module_id),
            runtime.clone(),
            false,
          ));
        }
      }
    }
  } else {
    if !force_side_effects && is_side_effect_free {
      return queue;
    }
    let changed_flag = exports_info.set_used_for_side_effects_only(runtime.as_ref());
    if changed_flag {
      queue.push((
        ModuleOrAsyncDependenciesBlock::Module(module_id),
        runtime,
        false,
      ));
    }
  }
  queue
}
