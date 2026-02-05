use std::sync::{Arc, RwLock};

use rspack_core::{
  AsyncDependenciesBlockIdentifier, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationDependencyReferencedExports,
  CompilationOptimizeDependencies, CompilationProcessAssets, DependenciesBlock, Dependency,
  DependencyId, DependencyType, ExtendedReferencedExport, Module, ModuleGraph, ModuleIdentifier,
  Plugin, RuntimeGlobals, RuntimeModule, RuntimeModuleExt, RuntimeSpec,
  SideEffectsOptimizeArtifact,
  build_module_graph::BuildModuleGraphArtifact,
  rspack_sources::{RawStringSource, SourceExt, SourceValue},
};
use rspack_error::{Diagnostic, Result};
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::dependency::{ESMImportSpecifierDependency, ImportDependency};
use rspack_util::atom::Atom;
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
  consume_shared_module::ConsumeSharedModule, provide_shared_module::ProvideSharedModule,
  shared_used_exports_optimizer_runtime_module::SharedUsedExportsOptimizerRuntimeModule,
};
use crate::{container::container_entry_module::ContainerEntryModule, manifest::StatsRoot};
#[derive(Debug, Clone)]
pub struct OptimizeSharedConfig {
  pub share_key: String,
  pub tree_shaking: bool,
  pub used_exports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SharedUsedExportsOptimizerPluginOptions {
  pub shared: Vec<OptimizeSharedConfig>,
  pub inject_tree_shaking_used_exports: bool,
  pub stats_file_name: Option<String>,
  pub manifest_file_name: Option<String>,
}

#[derive(Debug, Clone)]
struct SharedEntryData {
  used_exports: Vec<Atom>,
}

#[plugin]
#[derive(Debug, Clone)]
pub struct SharedUsedExportsOptimizerPlugin {
  shared_map: FxHashMap<String, SharedEntryData>,
  shared_referenced_exports: Arc<RwLock<FxHashMap<String, FxHashSet<String>>>>,
  inject_tree_shaking_used_exports: bool,
  stats_file_name: Option<String>,
  manifest_file_name: Option<String>,
}

impl SharedUsedExportsOptimizerPlugin {
  pub fn new(options: SharedUsedExportsOptimizerPluginOptions) -> Self {
    let mut shared_map = FxHashMap::default();
    let inject_tree_shaking_used_exports = options.inject_tree_shaking_used_exports;
    for config in options.shared.into_iter().filter(|c| c.tree_shaking) {
      let atoms = config
        .used_exports
        .into_iter()
        .map(Atom::from)
        .collect::<Vec<_>>();
      shared_map.insert(
        config.share_key,
        SharedEntryData {
          used_exports: atoms,
        },
      );
    }

    let shared_referenced_exports = Arc::new(RwLock::new(
      FxHashMap::<String, FxHashSet<String>>::default(),
    ));

    Self::new_inner(
      shared_map,
      shared_referenced_exports,
      inject_tree_shaking_used_exports,
      options.stats_file_name,
      options.manifest_file_name,
    )
  }

  fn apply_custom_exports(&self) {
    let mut shared_referenced_exports = self
      .shared_referenced_exports
      .write()
      .expect("lock poisoned");
    for (share_key, shared_entry_data) in &self.shared_map {
      let export_set = shared_referenced_exports
        .entry(share_key.clone())
        .or_default();
      for used_export in &shared_entry_data.used_exports {
        export_set.insert(used_export.to_string());
      }
    }
  }
}

fn collect_processed_modules(
  module_graph: &ModuleGraph,
  module_blocks: &[AsyncDependenciesBlockIdentifier],
  module_deps: &[DependencyId],
  out: &mut Vec<ModuleIdentifier>,
) {
  for dep_id in module_deps {
    if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
      out.push(*target_id);
    }
  }

  for block_id in module_blocks {
    if let Some(block) = module_graph.block_by_id(block_id) {
      for dep_id in block.get_dependencies() {
        if let Some(target_id) = module_graph.module_identifier_by_dependency_id(dep_id) {
          out.push(*target_id);
        }
      }
    }
  }
}

#[plugin_hook(
  CompilationOptimizeDependencies for SharedUsedExportsOptimizerPlugin,
  stage = 1
)]
async fn optimize_dependencies(
  &self,
  _compilation: &Compilation,
  _side_effects_optimize_artifact: &mut SideEffectsOptimizeArtifact,
  build_module_graph_artifact: &mut BuildModuleGraphArtifact,
  _diagnostics: &mut Vec<Diagnostic>,
) -> Result<Option<bool>> {
  // Collect module info first to minimize lock contention
  let module_infos: Vec<_> = {
    let module_graph = build_module_graph_artifact.get_module_graph();
    let module_ids: Vec<_> = module_graph.modules().keys().copied().collect();

    module_ids
      .into_iter()
      .filter_map(|module_id| {
        let module = module_graph.module_by_identifier(&module_id)?;
        let module_type = module.module_type();

        if !matches!(
          module_type,
          rspack_core::ModuleType::ConsumeShared
            | rspack_core::ModuleType::ProvideShared
            | rspack_core::ModuleType::ShareContainerShared
        ) {
          return None;
        }

        let mut modules_to_process = Vec::new();
        let share_key = match module_type {
          rspack_core::ModuleType::ConsumeShared => {
            let consume_shared_module = module.as_any().downcast_ref::<ConsumeSharedModule>()?;
            let identifier =
              consume_shared_module.readable_identifier(&rspack_core::Context::default());
            let identifier_str = identifier.to_string();
            let parts: Vec<&str> = identifier_str.split(") ").collect();
            if parts.len() < 2 {
              return None;
            }
            let share_key_part = parts[1];
            let share_key_end = if let Some(stripped) = share_key_part.strip_prefix('@') {
              stripped.find('@').map_or(share_key_part.len(), |i| i + 1)
            } else {
              share_key_part.find('@').unwrap_or(share_key_part.len())
            };
            let sk: String = share_key_part[..share_key_end].to_string();
            collect_processed_modules(
              module_graph,
              consume_shared_module.get_blocks(),
              consume_shared_module.get_dependencies(),
              &mut modules_to_process,
            );
            sk
          }
          rspack_core::ModuleType::ProvideShared => {
            let provide_shared_module = module.as_any().downcast_ref::<ProvideSharedModule>()?;
            let sk = provide_shared_module.share_key().to_string();
            collect_processed_modules(
              module_graph,
              provide_shared_module.get_blocks(),
              provide_shared_module.get_dependencies(),
              &mut modules_to_process,
            );
            sk
          }
          rspack_core::ModuleType::ShareContainerShared => {
            let share_container_entry_module =
              module.as_any().downcast_ref::<ContainerEntryModule>()?;
            let sk = share_container_entry_module.name().to_string();
            collect_processed_modules(
              module_graph,
              share_container_entry_module.get_blocks(),
              share_container_entry_module.get_dependencies(),
              &mut modules_to_process,
            );
            sk
          }
          _ => return None,
        };

        if share_key.is_empty() || !self.shared_map.contains_key(&share_key) {
          return None;
        }

        let real_shared_identifier = modules_to_process.first().copied()?;
        let is_side_effect_free = module_graph
          .module_by_identifier(&real_shared_identifier)
          .and_then(|m| m.factory_meta().and_then(|meta| meta.side_effect_free))
          .unwrap_or(false);

        Some((
          share_key,
          modules_to_process,
          real_shared_identifier,
          is_side_effect_free,
        ))
      })
      .collect()
  };

  self.apply_custom_exports();

  // Get runtime referenced exports
  let runtime_refs: FxHashMap<String, FxHashSet<String>> = {
    let guard = self
      .shared_referenced_exports
      .read()
      .expect("lock poisoned");
    guard.clone()
  };

  // Batch mutations to minimize write lock time
  let mutations: Vec<_> = module_infos
    .into_iter()
    .filter_map(
      |(share_key, modules_to_process, real_shared_identifier, is_side_effect_free)| {
        let runtime_reference_exports = runtime_refs.get(&share_key)?;

        if runtime_reference_exports.is_empty() {
          return None;
        }

        if !is_side_effect_free {
          return Some((share_key, None, vec![]));
        }

        Some((share_key, Some(real_shared_identifier), modules_to_process))
      },
    )
    .collect();

  // Apply all mutations in a single write lock
  if !mutations.is_empty() {
    let module_graph_mut = build_module_graph_artifact.get_module_graph_mut();
    module_graph_mut.active_all_exports_info();

    for (share_key, real_shared_id, modules_to_process) in mutations {
      if let Some(real_shared_identifier) = real_shared_id {
        let runtime_reference_exports = match runtime_refs.get(&share_key) {
          Some(refs) => refs,
          None => continue,
        };

        // Mark used for collected modules
        for module_id in &modules_to_process {
          let exports_info = module_graph_mut.get_exports_info(module_id);
          let exports_info_data = exports_info.as_data_mut(module_graph_mut);

          for export_name in runtime_reference_exports.iter() {
            let export_atom = Atom::from(export_name.as_str());
            if let Some(export_info) = exports_info_data.named_exports_mut(&export_atom) {
              export_info.set_used(rspack_core::UsageState::Used, None);
            }
          }
        }

        // Check if can update real share module
        let exports_info = module_graph_mut.get_exports_info(&real_shared_identifier);
        let exports_info_data = exports_info.as_data_mut(module_graph_mut);
        let can_update_module_used_stage = {
          let exports_view = exports_info_data.exports();
          if exports_view.is_empty() {
            false
          } else {
            exports_view.iter().all(|(name, export_info)| {
              let used = export_info.get_used(None);
              if used != rspack_core::UsageState::Unknown && used != rspack_core::UsageState::Unused
              {
                runtime_reference_exports.contains(&name.to_string())
              } else {
                true
              }
            })
          }
        };

        if can_update_module_used_stage {
          for export_info in exports_info_data.exports_mut().values_mut() {
            export_info.set_used_conditionally(
              Box::new(|used| *used == rspack_core::UsageState::Unknown),
              rspack_core::UsageState::Unused,
              None,
            );
            export_info.set_can_mangle_provide(Some(false));
            export_info.set_can_mangle_use(Some(false));
          }
        }
      } else {
        // Clear referenced exports for side-effect modules
        if let Ok(mut shared_referenced_exports) = self.shared_referenced_exports.write() {
          if let Some(set) = shared_referenced_exports.get_mut(&share_key) {
            set.clear();
          }
        }
      }
    }
  }

  Ok(None)
}

#[plugin_hook(CompilationProcessAssets for SharedUsedExportsOptimizerPlugin, stage = 1)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  let file_names = vec![
    self.stats_file_name.clone(),
    self.manifest_file_name.clone(),
  ];
  for file_name in file_names {
    if let Some(file_name) = &file_name
      && let Some(file) = compilation.assets().get(file_name)
      && let Some(source) = file.get_source()
      && let SourceValue::String(content) = source.source()
      && let Ok(mut stats_root) = serde_json::from_str::<StatsRoot>(&content)
    {
      let shared_referenced_exports = self
        .shared_referenced_exports
        .read()
        .expect("lock poisoned");

      for shared in &mut stats_root.shared {
        if let Some(exports_set) = shared_referenced_exports.get(&shared.name) {
          shared.usedExports = exports_set.iter().cloned().collect::<Vec<_>>();
        }
      }

      let updated_content = serde_json::to_string_pretty(&stats_root)
        .map_err(|e| rspack_error::error!("Failed to serialize stats root: {}", e))?;

      compilation.update_asset(file_name, |_, info| {
        Ok((RawStringSource::from(updated_content).boxed(), info))
      })?;
    }
  }

  Ok(())
}

#[plugin_hook(
  CompilationAdditionalTreeRuntimeRequirements for SharedUsedExportsOptimizerPlugin
)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
  runtime_modules: &mut Vec<Box<dyn RuntimeModule>>,
) -> Result<()> {
  if self.shared_map.is_empty() {
    return Ok(());
  }

  runtime_requirements.insert(RuntimeGlobals::RUNTIME_ID);
  runtime_modules.push(
    SharedUsedExportsOptimizerRuntimeModule::new(
      &compilation.runtime_template,
      Arc::new(
        self
          .shared_referenced_exports
          .read()
          .expect("lock poisoned")
          .clone(),
      ),
    )
    .boxed(),
  );

  Ok(())
}

#[plugin_hook(CompilationDependencyReferencedExports for SharedUsedExportsOptimizerPlugin)]
async fn dependency_referenced_exports(
  &self,
  compilation: &Compilation,
  dependency_id: &DependencyId,
  referenced_exports: &Option<Vec<ExtendedReferencedExport>>,
  _runtime: Option<&RuntimeSpec>,
  module_graph: Option<&ModuleGraph>,
) -> Result<()> {
  let module_graph = module_graph.unwrap_or_else(|| compilation.get_module_graph());
  if referenced_exports.is_none() {
    return Ok(());
  }
  let Some(exports) = referenced_exports else {
    return Ok(());
  };

  let dependency = module_graph.dependency_by_id(dependency_id);

  let Some(module_dependency) = dependency.as_module_dependency() else {
    return Ok(());
  };

  let share_key: &str = module_dependency.request();

  // Check if dependency type is EsmImportSpecifier and share_key is in shared_map
  if !self.shared_map.contains_key(share_key) {
    return Ok(());
  }
  let mut final_exports = exports.clone();

  // If it's an import dependency and referenced exports indicate "exports object referenced",
  // clear any recorded shared referenced exports for this share key and stop here.
  let is_exports_object = matches!(
    final_exports.as_slice(),
    [ExtendedReferencedExport::Array(arr)] if arr.is_empty()
  );
  if dependency
    .as_any()
    .downcast_ref::<ImportDependency>()
    .is_some()
    && is_exports_object
  {
    let mut shared_referenced_exports = self
      .shared_referenced_exports
      .write()
      .expect("lock poisoned");
    shared_referenced_exports.remove(share_key);
    return Ok(());
  }
  if (final_exports.is_empty() || is_exports_object)
    && dependency.dependency_type() == &DependencyType::EsmImportSpecifier
    && let Some(esm_dep) = dependency
      .as_any()
      .downcast_ref::<ESMImportSpecifierDependency>()
  {
    let ids: &[Atom] = esm_dep.get_ids(module_graph);
    if ids.is_empty() {
      return Ok(());
    }
    if let Some(first) = ids.first()
      && *first == "default"
    {
      final_exports = esm_dep.get_referenced_exports_in_destructuring(Some(ids));
    } else {
      final_exports = esm_dep.get_referenced_exports(
        module_graph,
        &compilation.module_graph_cache_artifact,
        _runtime,
      );
    }
  }

  // Process each referenced export
  if self.shared_map.contains_key(share_key) {
    let mut shared_referenced_exports = self
      .shared_referenced_exports
      .write()
      .expect("lock poisoned");
    let export_set = shared_referenced_exports
      .entry(share_key.to_string())
      .or_default();

    for referenced_export in &final_exports {
      match referenced_export {
        ExtendedReferencedExport::Array(exports_array) => {
          for export in exports_array {
            export_set.insert(export.to_string());
          }
        }
        ExtendedReferencedExport::Export(referenced) => {
          if referenced.name.is_empty() {
            continue;
          }
          for atom in &referenced.name {
            export_set.insert(atom.to_string());
          }
        }
      }
    }
  }
  Ok(())
}

impl Plugin for SharedUsedExportsOptimizerPlugin {
  fn name(&self) -> &'static str {
    "rspack.sharing.SharedUsedExportsOptimizerPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    if self.shared_map.is_empty() {
      return Ok(());
    }
    ctx
      .compilation_hooks
      .dependency_referenced_exports
      .tap(dependency_referenced_exports::new(self));
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    if self.inject_tree_shaking_used_exports {
      ctx
        .compilation_hooks
        .additional_tree_runtime_requirements
        .tap(additional_tree_runtime_requirements::new(self));
    }
    Ok(())
  }
}
