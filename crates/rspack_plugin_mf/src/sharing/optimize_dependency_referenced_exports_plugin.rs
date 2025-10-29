use std::{
  collections::{BTreeMap, HashMap},
  sync::{Arc, Mutex, OnceLock},
};

use rspack_core::{
  BoxModule, ChunkByUkey, ChunkGraph, ChunkUkey, Compilation,
  CompilationAdditionalTreeRuntimeRequirements, CompilationId, CompilationOptimizeDependencies,
  CompilationParams, CompilationProcessAssets, CompilerCompilation, DependencyType,
  ExtendedReferencedExport, ModuleGraph, ModuleGraphCacheArtifact, ModuleIdentifier, Plugin,
  RuntimeGlobals, RuntimeModuleExt, RuntimeSpec, UsageState,
};
use rspack_error::{Result, error};
use rspack_hook::{plugin, plugin_hook};
use rspack_sources::{RawStringSource, SourceExt};
use rspack_util::{
  atom::Atom,
  fx_hash::{FxHashMap, FxHashSet},
};
use tracing::warn;

use super::{
  optimize_dependency_referenced_exports_runtime_module::OptimizeDependencyReferencedExportsRuntimeModule,
  provide_shared_module::ProvideSharedModule,
};
#[derive(Debug, Clone)]
pub struct OptimizeSharedConfig {
  pub share_key: String,
  pub treeshake: bool,
  pub used_exports: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OptimizeDependencyReferencedExportsPluginOptions {
  pub shared: Vec<OptimizeSharedConfig>,
  pub ignored_runtime: Vec<String>,
}

#[derive(Debug, Default)]
struct RuntimeExportsEntry {
  runtime: RuntimeSpec,
  exports: FxHashSet<Atom>,
}

#[derive(Debug, Default)]
struct OptimizeCompilationState {
  shared_referenced_exports: FxHashMap<String, Vec<RuntimeExportsEntry>>,
  runtime_specs: FxHashMap<String, RuntimeSpec>,
  share_key_to_modules: FxHashMap<String, (ModuleIdentifier, ModuleIdentifier)>,
}

impl OptimizeCompilationState {
  fn new(shared_keys: impl Iterator<Item = String>) -> Self {
    let mut shared_referenced_exports = FxHashMap::default();
    for key in shared_keys {
      shared_referenced_exports.insert(key, Vec::new());
    }
    Self {
      shared_referenced_exports,
      runtime_specs: FxHashMap::default(),
      share_key_to_modules: FxHashMap::default(),
    }
  }

  fn reset(&mut self, shared_keys: impl Iterator<Item = String>) {
    self.shared_referenced_exports.clear();
    for key in shared_keys {
      self.shared_referenced_exports.insert(key, Vec::new());
    }
    self.runtime_specs.clear();
    self.share_key_to_modules.clear();
  }

  fn get_runtime_entry_mut(
    &mut self,
    share_key: &str,
    runtime: &RuntimeSpec,
  ) -> &mut FxHashSet<Atom> {
    let runtime_entries = self
      .shared_referenced_exports
      .entry(share_key.to_string())
      .or_default();
    if let Some(index) = runtime_entries
      .iter()
      .position(|entry| entry.runtime == *runtime)
    {
      let entry = runtime_entries
        .get_mut(index)
        .expect("should have runtime entry");
      return &mut entry.exports;
    }
    runtime_entries.push(RuntimeExportsEntry {
      runtime: runtime.clone(),
      exports: FxHashSet::default(),
    });
    let entry = runtime_entries
      .last_mut()
      .expect("should have runtime entry");
    &mut entry.exports
  }

  fn clear_exports_for_share(&mut self, share_key: &str) {
    if let Some(entries) = self.shared_referenced_exports.get_mut(share_key) {
      for entry in entries {
        entry.exports.clear();
      }
    }
  }

  fn build_runtime_used_exports_map(&self) -> BTreeMap<String, BTreeMap<String, Vec<String>>> {
    let mut result: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    for (share_key, entries) in &self.shared_referenced_exports {
      let mut runtime_map: BTreeMap<String, Vec<String>> = BTreeMap::new();
      for entry in entries {
        if entry.exports.is_empty() {
          continue;
        }
        let runtime_key = entry.runtime.as_str().to_string();
        let exports = runtime_map.entry(runtime_key).or_default();
        for atom in entry.exports.iter() {
          exports.push(atom.to_string());
        }
        exports.sort();
        exports.dedup();
      }
      if !runtime_map.is_empty() {
        result.insert(share_key.clone(), runtime_map);
      }
    }
    result
  }

  fn build_flat_used_exports(&self) -> BTreeMap<String, Vec<String>> {
    let mut result: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (share_key, entries) in &self.shared_referenced_exports {
      let mut exports_set: FxHashSet<Atom> = FxHashSet::default();
      for entry in entries {
        exports_set.extend(entry.exports.iter().cloned());
      }
      if exports_set.is_empty() {
        continue;
      }
      let mut exports_vec: Vec<String> = exports_set.iter().map(|atom| atom.to_string()).collect();
      exports_vec.sort();
      result.insert(share_key.clone(), exports_vec);
    }
    result
  }
}

#[derive(Debug, Clone)]
struct SharedEntryData {
  used_exports: Vec<Atom>,
}

#[plugin]
#[derive(Debug)]
pub struct OptimizeDependencyReferencedExportsPlugin {
  shared_map: FxHashMap<String, SharedEntryData>,
  ignored_runtime: FxHashSet<String>,
  custom_referenced_exports: FxHashMap<String, Vec<Atom>>,
}

impl OptimizeDependencyReferencedExportsPlugin {
  pub fn new(options: OptimizeDependencyReferencedExportsPluginOptions) -> Self {
    let mut shared_map = FxHashMap::default();
    for config in options.shared.into_iter().filter(|c| c.treeshake) {
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

    let ignored_runtime = options.ignored_runtime.into_iter().collect();
    let custom_referenced_exports = Self::load_custom_referenced_exports();

    Self::new_inner(shared_map, ignored_runtime, custom_referenced_exports)
  }

  fn load_custom_referenced_exports() -> FxHashMap<String, Vec<Atom>> {
    match std::env::var("MF_CUSTOM_REFERENCED_EXPORTS") {
      Ok(raw) if !raw.trim().is_empty() => {
        match serde_json::from_str::<HashMap<String, Vec<String>>>(&raw) {
          Ok(map) => map
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(Atom::from).collect()))
            .collect(),
          Err(err) => {
            warn!(
              "OptimizeDependencyReferencedExportsPlugin: failed to parse MF_CUSTOM_REFERENCED_EXPORTS: {err}"
            );
            FxHashMap::default()
          }
        }
      }
      _ => FxHashMap::default(),
    }
  }

  fn compilation_states() -> &'static Mutex<HashMap<CompilationId, OptimizeCompilationState>> {
    static STATES: OnceLock<Mutex<HashMap<CompilationId, OptimizeCompilationState>>> =
      OnceLock::new();
    STATES.get_or_init(|| Mutex::new(HashMap::new()))
  }

  fn shared_keys(&self) -> impl Iterator<Item = String> + '_ {
    self.shared_map.keys().cloned()
  }

  fn extend_referenced_exports(
    target: &mut FxHashSet<Atom>,
    referenced: &[ExtendedReferencedExport],
  ) {
    for item in referenced {
      match item {
        ExtendedReferencedExport::Array(arr) => {
          target.extend(arr.iter().cloned());
        }
        ExtendedReferencedExport::Export(export) => {
          target.extend(export.name.iter().cloned());
        }
      }
    }
  }

  fn populate_provide_mappings(
    &self,
    module_graph: &ModuleGraph,
    state: &mut OptimizeCompilationState,
  ) {
    for (module_id, module) in module_graph.modules().into_iter() {
      let module_id = module_id;
      let module: &BoxModule = module;
      if let Some(provide_module) = module
        .as_ref()
        .as_any()
        .downcast_ref::<ProvideSharedModule>()
      {
        let share_key = provide_module.share_key();
        if !self.shared_map.contains_key(share_key) {
          continue;
        }
        if let Some(fallback) = Self::find_fallback_module(module_graph, module_id) {
          state
            .share_key_to_modules
            .insert(share_key.to_string(), (module_id, fallback));
        }
      }
    }
  }

  fn find_fallback_module(
    module_graph: &ModuleGraph,
    module_identifier: ModuleIdentifier,
  ) -> Option<ModuleIdentifier> {
    for connection in module_graph.get_outgoing_connections(&module_identifier) {
      let dep_id = connection.dependency_id;
      let dependency = module_graph.dependency_by_id(&dep_id)?;
      if dependency.dependency_type() == &DependencyType::ProvideModuleForShared {
        return Some(*connection.module_identifier());
      }
    }
    None
  }

  fn collect_referenced_exports(
    &self,
    module_graph: &ModuleGraph,
    chunk_graph: &ChunkGraph,
    chunk_by_ukey: &ChunkByUkey,
    module_graph_cache: &ModuleGraphCacheArtifact,
    state: &mut OptimizeCompilationState,
  ) {
    for (module_id, _module) in module_graph.modules().into_iter() {
      let runtimes: Vec<RuntimeSpec> = chunk_graph
        .get_module_runtimes_iter(module_id, chunk_by_ukey)
        .cloned()
        .collect();
      if runtimes.is_empty() {
        continue;
      }

      for connection in module_graph.get_outgoing_connections(&module_id) {
        let dep_id = connection.dependency_id;
        let dependency = match module_graph.dependency_by_id(&dep_id) {
          Some(dep) => dep,
          None => continue,
        };
        let Some(module_dep) = dependency.as_module_dependency() else {
          continue;
        };
        if dependency.dependency_type() != &DependencyType::EsmImportSpecifier {
          continue;
        }

        let request = module_dep.request();
        let Some(shared_entry) = self.shared_map.get(request) else {
          continue;
        };

        for runtime in &runtimes {
          let runtime_key = runtime.as_str().to_string();
          if self.ignored_runtime.contains(&runtime_key) {
            continue;
          }

          let active_state =
            connection.active_state(module_graph, Some(runtime), module_graph_cache);
          if !active_state.is_true() {
            continue;
          }

          let runtime_spec = state
            .runtime_specs
            .entry(runtime_key.clone())
            .or_insert_with(|| runtime.clone())
            .clone();

          let referenced_exports =
            module_dep.get_referenced_exports(module_graph, module_graph_cache, Some(runtime));

          if referenced_exports.is_empty()
            && shared_entry.used_exports.is_empty()
            && !self
              .custom_referenced_exports
              .get(request)
              .map_or(false, |v| !v.is_empty())
          {
            continue;
          }

          let exports_set = state.get_runtime_entry_mut(request, &runtime_spec);
          Self::extend_referenced_exports(exports_set, &referenced_exports);
        }
      }
    }
  }

  fn apply_custom_exports(&self, state: &mut OptimizeCompilationState) {
    if state.runtime_specs.is_empty() {
      return;
    }

    for (share_key, shared_entry) in self.shared_map.iter() {
      let custom_entry = self.custom_referenced_exports.get(share_key);
      if shared_entry.used_exports.is_empty() && custom_entry.map_or(true, |entry| entry.is_empty())
      {
        continue;
      }

      let runtime_specs: Vec<(String, RuntimeSpec)> = state
        .runtime_specs
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
      for (runtime_key, runtime_spec) in runtime_specs {
        if self.ignored_runtime.contains(&runtime_key) {
          continue;
        }
        let exports_set = state.get_runtime_entry_mut(share_key, &runtime_spec);
        exports_set.extend(shared_entry.used_exports.iter().cloned());
        if let Some(custom) = custom_entry {
          exports_set.extend(custom.iter().cloned());
        }
      }
    }
  }

  fn mark_exports(&self, compilation: &mut Compilation, state: &mut OptimizeCompilationState) {
    let mut module_graph = compilation.get_module_graph_mut();
    let module_pairs: Vec<(String, (ModuleIdentifier, ModuleIdentifier))> = state
      .share_key_to_modules
      .iter()
      .map(|(k, v)| (k.clone(), *v))
      .collect();
    for (share_key, (provide_id, fallback_id)) in module_pairs {
      if let Some(module) = module_graph.module_by_identifier_mut(&provide_id) {
        let mut meta = module.factory_meta().cloned().unwrap_or_default();
        if meta.side_effect_free != Some(true) {
          meta.side_effect_free = Some(true);
          module.set_factory_meta(meta);
        }
      }

      let fallback_module = match module_graph.module_by_identifier(&fallback_id) {
        Some(module) => module,
        None => continue,
      };
      let is_side_effect_free = fallback_module
        .factory_meta()
        .and_then(|meta| meta.side_effect_free)
        .unwrap_or(false);

      if !is_side_effect_free {
        state.clear_exports_for_share(&share_key);
        continue;
      }

      let exports_info = module_graph.get_exports_info(&fallback_id);
      let exports_info_data = exports_info.as_data_mut(&mut module_graph);

      if let Some(runtime_entries) = state.shared_referenced_exports.get(&share_key) {
        for entry in runtime_entries {
          for atom in &entry.exports {
            let export_info = exports_info_data.ensure_owned_export_info(atom);
            export_info.set_used(UsageState::Used, Some(&entry.runtime));
          }
        }

        for entry in runtime_entries {
          if entry.exports.is_empty() {
            continue;
          }

          let can_update = {
            let exports_view = exports_info_data.exports();
            exports_view.iter().all(|(name, export_info)| {
              let used = export_info.get_used(Some(&entry.runtime));
              if used != UsageState::Unknown {
                entry.exports.contains(name)
              } else {
                true
              }
            })
          };

          if can_update {
            for export_info in exports_info_data.exports_mut().values_mut() {
              export_info.set_used_conditionally(
                Box::new(|used| *used == UsageState::Unknown),
                UsageState::Unused,
                Some(&entry.runtime),
              );
            }
            exports_info_data
              .other_exports_info_mut()
              .set_used_conditionally(
                Box::new(|used| *used == UsageState::Unknown),
                UsageState::Unused,
                Some(&entry.runtime),
              );
          }
        }
      }
    }
  }
}

#[plugin_hook(CompilerCompilation for OptimizeDependencyReferencedExportsPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  if self.shared_map.is_empty() {
    return Ok(());
  }
  let mut states = Self::compilation_states()
    .lock()
    .expect("state lock poisoned");
  states.insert(
    compilation.id(),
    OptimizeCompilationState::new(self.shared_keys()),
  );
  Ok(())
}

#[plugin_hook(CompilationOptimizeDependencies for OptimizeDependencyReferencedExportsPlugin)]
async fn optimize_dependencies(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  if self.shared_map.is_empty() {
    return Ok(None);
  }

  let mut states = Self::compilation_states()
    .lock()
    .expect("state lock poisoned");
  let Some(state) = states.get_mut(&compilation.id()) else {
    return Ok(None);
  };

  state.reset(self.shared_keys());

  {
    let module_graph = compilation.get_module_graph();
    self.populate_provide_mappings(&module_graph, state);
    self.collect_referenced_exports(
      &module_graph,
      &compilation.chunk_graph,
      &compilation.chunk_by_ukey,
      &compilation.module_graph_cache_artifact,
      state,
    );
  }
  self.apply_custom_exports(state);
  self.mark_exports(compilation, state);

  Ok(None)
}

#[plugin_hook(
  CompilationProcessAssets for OptimizeDependencyReferencedExportsPlugin,
  stage = Compilation::PROCESS_ASSETS_STAGE_OPTIMIZE_TRANSFER
)]
async fn process_assets(&self, compilation: &mut Compilation) -> Result<()> {
  if self.shared_map.is_empty() {
    return Ok(());
  }

  let flat_exports = {
    let states = Self::compilation_states()
      .lock()
      .expect("state lock poisoned");
    states
      .get(&compilation.id())
      .map(|state| state.build_flat_used_exports())
      .unwrap_or_default()
  };

  if flat_exports.is_empty() {
    return Ok(());
  }

  if !compilation.assets().contains_key("mf-stats.json") {
    return Ok(());
  }

  compilation.update_asset("mf-stats.json", |old_source, info| {
    let content = old_source.source().to_string();
    let mut json: serde_json::Value = serde_json::from_str(&content).map_err(|err| {
      error!("OptimizeDependencyReferencedExportsPlugin: Failed to parse mf-stats.json: {err}")
    })?;

    if let Some(shared) = json.get_mut("shared").and_then(|v| v.as_array_mut()) {
      for entry in shared.iter_mut() {
        let Some(name) = entry.get("name").and_then(|v| v.as_str()) else {
          continue;
        };
        if let Some(exports) = flat_exports.get(name) {
          entry
            .as_object_mut()
            .expect("shared entry should be an object")
            .insert(
              "usedExports".to_string(),
              serde_json::Value::Array(exports.iter().map(|e| e.clone().into()).collect()),
            );
        }
      }
    }

    let updated = serde_json::to_string(&json).map_err(|err| {
      error!("OptimizeDependencyReferencedExportsPlugin: Failed to serialize mf-stats.json: {err}")
    })?;
    Ok((RawStringSource::from(updated).boxed(), info))
  })?;

  Ok(())
}

#[plugin_hook(
  CompilationAdditionalTreeRuntimeRequirements for OptimizeDependencyReferencedExportsPlugin
)]
async fn additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if self.shared_map.is_empty() {
    return Ok(());
  }

  let runtime_map = {
    let states = Self::compilation_states()
      .lock()
      .expect("state lock poisoned");
    states
      .get(&compilation.id())
      .map(|state| state.build_runtime_used_exports_map())
      .unwrap_or_default()
  };

  if runtime_map.is_empty() {
    return Ok(());
  }

  runtime_requirements.insert(RuntimeGlobals::RUNTIME_ID);
  compilation.add_runtime_module(
    chunk_ukey,
    OptimizeDependencyReferencedExportsRuntimeModule::new(Arc::new(runtime_map)).boxed(),
  )?;

  Ok(())
}

impl Plugin for OptimizeDependencyReferencedExportsPlugin {
  fn name(&self) -> &'static str {
    "rspack.sharing.OptimizeDependencyReferencedExportsPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    if self.shared_map.is_empty() {
      return Ok(());
    }
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .optimize_dependencies
      .tap(optimize_dependencies::new(self));
    ctx
      .compilation_hooks
      .process_assets
      .tap(process_assets::new(self));
    ctx
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(additional_tree_runtime_requirements::new(self));
    Ok(())
  }

  fn clear_cache(&self, id: CompilationId) {
    if let Ok(mut states) = Self::compilation_states().lock() {
      states.remove(&id);
    }
  }
}
