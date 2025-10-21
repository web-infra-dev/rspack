//! # HoistContainerReferencesPlugin
//!
//! Optimizes Module Federation chunk placement by hoisting container references and
//! their dependencies to runtime chunks. This plugin enhances module hoisting and
//! runtime chunk handling for Module Federation by:
//!
//! - Separating dependency handling for container, federation runtime, and remote modules
//! - Enhanced runtime chunk detection supporting `runtimeChunk: 'single'` configurations
//! - Recursive collection of referenced modules with proper async dependency exclusion
//! - Efficient cleanup of empty non-runtime chunks after hoisting
//!
//! The plugin coordinates with FederationModulesPlugin through a hook-based system
//! to collect and manage federation-specific dependencies across the compilation.

use std::{
  collections::VecDeque,
  sync::{Arc, Mutex},
};

use async_trait::async_trait;
use rspack_core::{
  ChunkUkey, Compilation, CompilationOptimizeChunks, CompilerCompilation, Dependency, DependencyId,
  ModuleIdentifier, Plugin, RuntimeSpec,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
  container_entry_dependency::ContainerEntryDependency, fallback_dependency::FallbackDependency,
  federation_modules_plugin::FederationModulesPlugin,
  federation_runtime_dependency::FederationRuntimeDependency,
  remote_to_external_dependency::RemoteToExternalDependency,
};

#[plugin]
#[derive(Debug, Default)]
pub struct HoistContainerReferencesPlugin {
  federation_deps: Arc<Mutex<FxHashSet<DependencyId>>>,
}

struct ContainerEntryDepCollector {
  set: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddContainerEntryDependencyHook
  for ContainerEntryDepCollector
{
  async fn run(&self, dep: &ContainerEntryDependency) -> Result<()> {
    self
      .set
      .lock()
      .expect("Failed to lock federation deps")
      .insert(*dep.id());
    Ok(())
  }
}

struct FederationRuntimeDepCollector {
  set: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddFederationRuntimeDependencyHook
  for FederationRuntimeDepCollector
{
  async fn run(&self, dep: &FederationRuntimeDependency) -> Result<()> {
    if std::env::var("RSPACK_DEBUG_HOIST").is_ok() {
      eprintln!("[hoist] federation runtime dep {:?}", dep.id());
    }
    self
      .set
      .lock()
      .expect("Failed to lock federation deps")
      .insert(*dep.id());
    Ok(())
  }
}

struct RemoteDepCollector {
  set: Arc<Mutex<FxHashSet<DependencyId>>>,
}

#[async_trait]
impl super::federation_modules_plugin::AddRemoteDependencyHook for RemoteDepCollector {
  async fn run(&self, dep: &dyn Dependency) -> Result<()> {
    if std::env::var("RSPACK_DEBUG_HOIST").is_ok() {
      eprintln!("[hoist] remote dep {:?}", dep.id());
    }
    if let Some(dep) = dep.downcast_ref::<RemoteToExternalDependency>() {
      self
        .set
        .lock()
        .expect("Failed to lock federation deps")
        .insert(*dep.id());
    }
    if let Some(dep) = dep.downcast_ref::<FallbackDependency>() {
      self
        .set
        .lock()
        .expect("Failed to lock federation deps")
        .insert(*dep.id());
    }
    Ok(())
  }
}

#[plugin_hook(CompilerCompilation for HoistContainerReferencesPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut rspack_core::CompilationParams,
) -> Result<()> {
  let hooks = FederationModulesPlugin::get_compilation_hooks(compilation);

  hooks
    .add_container_entry_dependency
    .lock()
    .await
    .tap(ContainerEntryDepCollector {
      set: Arc::clone(&self.federation_deps),
    });

  hooks
    .add_federation_runtime_dependency
    .lock()
    .await
    .tap(FederationRuntimeDepCollector {
      set: Arc::clone(&self.federation_deps),
    });

  hooks
    .add_remote_dependency
    .lock()
    .await
    .tap(RemoteDepCollector {
      set: Arc::clone(&self.federation_deps),
    });

  Ok(())
}

#[plugin_hook(CompilationOptimizeChunks for HoistContainerReferencesPlugin, stage = Compilation::OPTIMIZE_CHUNKS_STAGE_ADVANCED + 1)]
async fn optimize_chunks(&self, compilation: &mut Compilation) -> Result<Option<bool>> {
  let debug_hoist = std::env::var("RSPACK_DEBUG_HOIST").is_ok();
  if debug_hoist {
    eprintln!("[hoist] optimize_chunks invoked");
  }

  // Collect all federation (container, runtime, and remote) referenced modules
  let mut all_modules_to_hoist = self
    .federation_deps
    .lock()
    .expect("Failed to lock federation deps")
    .iter()
    .filter_map(|dep| {
      let module_graph = compilation.get_module_graph();
      module_graph
        .module_identifier_by_dependency_id(dep)
        .copied()
        .or_else(|| {
          module_graph
            .get_module_by_dependency_id(dep)
            .map(|module| module.identifier())
        })
    })
    .flat_map(|module| {
      collect_referenced_modules(module, true, |current| {
        let module_graph = compilation.get_module_graph();
        module_graph
          .get_outgoing_connections(&current)
          .map(|conn| {
            let is_async = module_graph.get_parent_block(&conn.dependency_id).is_some();
            (*conn.module_identifier(), is_async)
          })
          .collect()
      })
    })
    .collect::<FxHashSet<_>>();

  // Include runtime helper modules that may not be referenced directly via dependencies
  for (chunk_ukey, _) in compilation.chunk_by_ukey.iter() {
    let module_ids = compilation
      .chunk_graph
      .get_ordered_chunk_modules_identifier(chunk_ukey);
    for module_id in module_ids {
      if all_modules_to_hoist.contains(&module_id) {
        continue;
      }

      let module_name = module_id.to_string();
      if module_name.contains("__module_federation_bundler_runtime__") {
        all_modules_to_hoist.insert(module_id);
      }
    }
  }

  if std::env::var("RSPACK_DEBUG_HOIST").is_ok() {
    let mut ids = all_modules_to_hoist
      .iter()
      .map(|id| id.to_string())
      .collect::<Vec<_>>();
    ids.sort();
    eprintln!("[hoist] modules to hoist: {:?}", ids);
  }

  // Build runtime chunk mapping
  let entry_chunks_iter = compilation.get_chunk_graph_entries().filter_map(|entry| {
    compilation
      .chunk_by_ukey
      .get(&entry)
      .map(|chunk| (chunk.runtime().clone(), entry))
  });
  let all_chunks_iter = compilation
    .chunk_by_ukey
    .iter()
    .map(|(chunk_ukey, chunk)| (*chunk_ukey, chunk.runtime().clone()));
  let runtime_chunk_map = build_runtime_chunk_map_from_chunks(entry_chunks_iter, all_chunks_iter);

  for module in &all_modules_to_hoist {
    let module_runtimes = compilation
      .chunk_graph
      .get_module_runtimes_iter(*module, &compilation.chunk_by_ukey)
      .cloned()
      .collect::<Vec<_>>();
    let target_chunks = find_connected_runtime_chunks(&runtime_chunk_map, module_runtimes);

    if debug_hoist {
      if module
        .to_string()
        .contains("__module_federation_bundler_runtime__")
      {
        eprintln!("[hoist] bundler runtime module seen");
      }
      let chunk_ids = target_chunks
        .iter()
        .map(|chunk| {
          compilation
            .chunk_by_ukey
            .get(chunk)
            .and_then(|c| c.name().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", chunk))
        })
        .collect::<Vec<_>>();
      eprintln!(
        "[hoist] connect module {} -> {:?}",
        module.to_string(),
        chunk_ids
      );
    }

    for runtime_chunk in target_chunks {
      if !compilation
        .chunk_graph
        .is_module_in_chunk(module, runtime_chunk)
      {
        compilation
          .chunk_graph
          .connect_chunk_and_module(runtime_chunk, *module);
      }
    }
  }

  // Disconnect hoisted modules from non-runtime chunks
  // Collect actual runtime chunks (get_chunk_graph_entries already returns runtime chunks)
  let runtime_chunks: FxHashSet<ChunkUkey> = compilation.get_chunk_graph_entries().collect();

  for module in &all_modules_to_hoist {
    let containing_chunks = compilation
      .chunk_graph
      .get_module_chunks(*module)
      .iter()
      .copied()
      .collect::<Vec<_>>();

    for chunk_ukey in containing_chunks {
      // Only disconnect from chunks that are NOT runtime chunks
      if !runtime_chunks.contains(&chunk_ukey) {
        compilation
          .chunk_graph
          .disconnect_chunk_and_module(&chunk_ukey, *module);
      }
    }
  }

  Ok(None)
}

/// Recursively collect all referenced modules from a starting module.
///
/// # Arguments
/// * `compilation` - The compilation context
/// * `module_id` - Starting module identifier
/// * `skip_async_blocks` - Whether to skip async block dependencies
///
/// # Returns
/// Set of all transitively referenced module identifiers
pub(crate) fn collect_referenced_modules<F>(
  module_id: ModuleIdentifier,
  skip_async_blocks: bool,
  mut successors: F,
) -> FxHashSet<ModuleIdentifier>
where
  F: FnMut(ModuleIdentifier) -> Vec<(ModuleIdentifier, bool)>,
{
  let mut collected = FxHashSet::default();
  let mut visited = FxHashSet::default();
  let mut stack = VecDeque::new();

  collected.insert(module_id);
  visited.insert(module_id);
  stack.push_back(module_id);

  while let Some(current_module_id) = stack.pop_front() {
    for (connected_id, is_async_block) in successors(current_module_id) {
      if visited.contains(&connected_id) {
        continue;
      }

      if skip_async_blocks && is_async_block {
        continue;
      }

      collected.insert(connected_id);
      visited.insert(connected_id);
      stack.push_back(connected_id);
    }
  }

  collected
}

/// Build a map from RuntimeSpec to ChunkUkey for all runtime chunks.
///
/// This map is used to determine which runtime chunk should host a given module.
///
/// # Returns
/// Map from RuntimeSpec to the ChunkUkey that represents that runtime
pub(crate) fn build_runtime_chunk_map_from_chunks<I, J>(
  entry_chunks: I,
  all_chunks: J,
) -> FxHashMap<RuntimeSpec, ChunkUkey>
where
  I: IntoIterator<Item = (RuntimeSpec, ChunkUkey)>,
  J: IntoIterator<Item = (ChunkUkey, RuntimeSpec)>,
{
  let mut runtime_chunk_map: FxHashMap<RuntimeSpec, ChunkUkey> = entry_chunks.into_iter().collect();

  for (chunk_ukey, runtime) in all_chunks.into_iter() {
    runtime_chunk_map.entry(runtime).or_insert(chunk_ukey);
  }

  runtime_chunk_map
}

/// Find all runtime chunks connected to a module.
///
/// # Arguments
/// * `module` - The module to find runtime chunks for
/// * `runtime_chunk_map` - Map from RuntimeSpec to ChunkUkey
/// * `compilation` - The compilation context
///
/// # Returns
/// Set of ChunkUkeys representing runtime chunks for this module
pub(crate) fn find_connected_runtime_chunks<I>(
  runtime_chunk_map: &FxHashMap<RuntimeSpec, ChunkUkey>,
  module_runtimes: I,
) -> FxHashSet<ChunkUkey>
where
  I: IntoIterator<Item = RuntimeSpec>,
{
  let mut connected = FxHashSet::default();

  for runtime_spec in module_runtimes {
    if let Some(runtime_chunk) = runtime_chunk_map.get(&runtime_spec).copied() {
      connected.insert(runtime_chunk);
    } else {
      // Try individual runtime keys
      for runtime_key in runtime_spec.iter() {
        if let Some(runtime_chunk) = runtime_chunk_map
          .get(&RuntimeSpec::from_iter([*runtime_key]))
          .copied()
        {
          connected.insert(runtime_chunk);
        }
      }
    }
  }

  // Fallback: if no specific runtime found, use all runtimes
  if connected.is_empty() {
    connected.extend(runtime_chunk_map.values().copied());
  }

  connected
}

impl Plugin for HoistContainerReferencesPlugin {
  fn name(&self) -> &'static str {
    "HoistContainerReferencesPlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .optimize_chunks
      .tap(optimize_chunks::new(self));
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use std::iter;

  use rustc_hash::{FxHashMap, FxHashSet};
  use ustr::Ustr;

  use super::*;

  fn module_id(name: &str) -> ModuleIdentifier {
    ModuleIdentifier::from(name)
  }

  fn runtime(name: &str) -> RuntimeSpec {
    RuntimeSpec::from_iter(iter::once(Ustr::from(name)))
  }

  #[test]
  fn test_plugin_name() {
    let plugin = HoistContainerReferencesPlugin::default();
    assert_eq!(plugin.name(), "HoistContainerReferencesPlugin");
  }

  #[test]
  fn test_plugin_initialization_has_empty_federation_deps() {
    let plugin = HoistContainerReferencesPlugin::default();
    let deps = plugin.federation_deps.lock().unwrap();
    assert_eq!(deps.len(), 0);
  }

  #[test]
  fn collect_referenced_modules_traverses_sync_graph() {
    let entry = module_id("entry");
    let runtime_mod = module_id("runtime");
    let shared = module_id("shared");

    let mut graph: FxHashMap<ModuleIdentifier, Vec<(ModuleIdentifier, bool)>> =
      FxHashMap::default();
    graph.insert(entry, vec![(runtime_mod, false), (shared, false)]);
    graph.insert(runtime_mod, vec![(shared, false)]);

    let collected = collect_referenced_modules(entry, true, |module| {
      graph.get(&module).cloned().unwrap_or_default()
    });

    assert_eq!(
      collected,
      FxHashSet::from_iter([entry, runtime_mod, shared]),
      "All reachable synchronous modules should be collected"
    );
  }

  #[test]
  fn collect_referenced_modules_skips_async_edges() {
    let entry = module_id("entry");
    let dynamic = module_id("dynamic");

    let mut graph: FxHashMap<ModuleIdentifier, Vec<(ModuleIdentifier, bool)>> =
      FxHashMap::default();
    graph.insert(entry, vec![(dynamic, true)]);

    let skip_async = collect_referenced_modules(entry, true, |module| {
      graph.get(&module).cloned().unwrap_or_default()
    });
    assert!(
      !skip_async.contains(&dynamic),
      "Async modules should be omitted when skipping async blocks"
    );

    let include_async = collect_referenced_modules(entry, false, |module| {
      graph.get(&module).cloned().unwrap_or_default()
    });
    assert!(
      include_async.contains(&dynamic),
      "Async modules should be included when not skipping async blocks"
    );
  }

  #[test]
  fn build_runtime_chunk_map_prioritises_entries() {
    let main_runtime = runtime("main");
    let secondary_runtime = runtime("secondary");

    let entry_chunk = ChunkUkey::new();
    let fallback_chunk = ChunkUkey::new();
    let secondary_chunk = ChunkUkey::new();

    let map = build_runtime_chunk_map_from_chunks(
      vec![
        (main_runtime.clone(), entry_chunk),
        (secondary_runtime.clone(), secondary_chunk),
      ],
      vec![
        (fallback_chunk, main_runtime.clone()),
        (secondary_chunk, secondary_runtime.clone()),
      ],
    );

    assert_eq!(
      map.get(&main_runtime),
      Some(&entry_chunk),
      "Entry chunk should be chosen for its runtime"
    );
    assert_eq!(
      map.get(&secondary_runtime),
      Some(&secondary_chunk),
      "Secondary runtime should be registered"
    );
  }

  #[test]
  fn find_connected_runtime_chunks_handles_combined_specs() {
    let main_runtime = runtime("main");
    let async_runtime = runtime("async");
    let combined_runtime = RuntimeSpec::from_iter([Ustr::from("main"), Ustr::from("async")]);

    let main_chunk = ChunkUkey::new();
    let async_chunk = ChunkUkey::new();

    let map = build_runtime_chunk_map_from_chunks(
      vec![
        (main_runtime.clone(), main_chunk),
        (async_runtime.clone(), async_chunk),
      ],
      Vec::<(ChunkUkey, RuntimeSpec)>::new(),
    );

    let connected = find_connected_runtime_chunks(&map, vec![combined_runtime]);
    assert_eq!(
      connected,
      FxHashSet::from_iter([main_chunk, async_chunk]),
      "Combined runtime specs should match both runtime chunks"
    );
  }

  #[test]
  fn find_connected_runtime_chunks_falls_back_to_all_chunks() {
    let known_runtime = runtime("known");
    let unknown_runtime = RuntimeSpec::from_iter([Ustr::from("unknown"), Ustr::from("runtime")]);
    let chunk = ChunkUkey::new();

    let map = build_runtime_chunk_map_from_chunks(
      vec![(known_runtime.clone(), chunk)],
      Vec::<(ChunkUkey, RuntimeSpec)>::new(),
    );

    let connected = find_connected_runtime_chunks(&map, vec![unknown_runtime]);
    assert_eq!(
      connected,
      FxHashSet::from_iter([chunk]),
      "Unknown runtime specs should fall back to all runtime chunks"
    );
  }
}
